use axum::{
    body::Body,
    http::{HeaderValue, Request},
    middleware::Next,
    response::Response,
};

/// Middleware to add security headers to all responses.
///
/// This middleware adds the following headers:
/// - `X-Content-Type-Options: nosniff` - Prevents MIME sniffing.
/// - `X-Frame-Options: SAMEORIGIN` - Prevents Clickjacking (allows same origin).
/// - `X-XSS-Protection: 1; mode=block` - Legacy XSS protection.
/// - `Referrer-Policy: strict-origin-when-cross-origin` - Controls referrer information.
/// - `Content-Security-Policy: frame-ancestors 'self';` - Prevents embedding in iframes from other origins.
pub async fn add_security_headers(req: Request<Body>, next: Next) -> Response {
    let mut response = next.run(req).await;

    let headers = response.headers_mut();

    // Prevent MIME sniffing
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );

    // Prevent Clickjacking
    headers.insert(
        "X-Frame-Options",
        HeaderValue::from_static("SAMEORIGIN"),
    );

    // XSS Protection (Legacy but still useful for older browsers)
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );

    // Referrer Policy
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Content Security Policy
    // We start with a minimal policy to avoid breaking the UI but still preventing embedding
    headers.insert(
        "Content-Security-Policy",
        HeaderValue::from_static("frame-ancestors 'self';"),
    );

    response
}
