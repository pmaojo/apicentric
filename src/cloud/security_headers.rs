use axum::{
    extract::Request,
    http::header::{HeaderValue, STRICT_TRANSPORT_SECURITY, X_CONTENT_TYPE_OPTIONS},
    middleware::Next,
    response::Response,
};

/// Middleware to append essential security headers to all responses.
pub async fn add_security_headers(req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    // Prevent MIME-type sniffing (Zero unwrap policy: using from_static)
    headers.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));

    // Deny embedding in iframes to prevent clickjacking
    // Must be lowercase to avoid panic on HeaderName::from_static
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));

    // Enforce HTTPS
    headers.insert(
        STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );

    response
}
