//! CORS configuration for the cloud server.
//!
//! This module provides environment-based CORS configuration with secure
//! defaults for production deployments.

use axum::http::{header, HeaderValue, Method};
use std::env;
use tower_http::cors::CorsLayer;

/// Creates a CORS layer based on environment configuration.
///
/// # Environment Variables
///
/// * `ALLOWED_ORIGINS` - Comma-separated list of allowed origins (e.g., "https://example.com,https://app.example.com")
/// * `APICENTRIC_ENV` - Environment mode ("development" or "production")
///
/// # Behavior
///
/// * **Development mode**: Permissive CORS allowing all origins
/// * **Production mode** (default): Restrictive CORS with specific allowed origins
///
/// # Returns
///
/// A configured `CorsLayer` for use with Axum.
pub fn create_cors_layer() -> CorsLayer {
    // 🛡️ Sentinel: Fail Secure - default to production (restrictive) CORS
    // if APICENTRIC_ENV is not explicitly set to "development"
    let env_mode = env::var("APICENTRIC_ENV").unwrap_or_else(|_| "production".to_string());

    if env_mode == "development" {
        create_development_cors()
    } else {
        create_production_cors()
    }
}

/// Creates a permissive CORS layer for development.
///
/// Allows all origins, methods, and headers for easier local development.
fn create_development_cors() -> CorsLayer {
    CorsLayer::permissive()
}

/// Creates a restrictive CORS layer for production.
///
/// Only allows specific origins from the `ALLOWED_ORIGINS` environment variable.
/// If not set, defaults to localhost for safety.
fn create_production_cors() -> CorsLayer {
    let allowed_origins =
        env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| "http://localhost:3000".to_string());

    let origins: Vec<HeaderValue> = allowed_origins
        .split(',')
        .filter_map(|origin| {
            let trimmed = origin.trim();
            if trimmed.is_empty() {
                None
            } else {
                trimmed.parse::<HeaderValue>().ok()
            }
        })
        .collect();

    if origins.is_empty() {
        // Fallback to localhost if no valid origins provided
        eprintln!("Warning: No valid ALLOWED_ORIGINS configured, defaulting to localhost");
        return CorsLayer::new()
            .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT])
            .allow_credentials(true);
    }

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT])
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(3600))
}

/// Creates a CORS layer with custom origins.
///
/// Useful for testing or when you want to programmatically set origins.
///
/// # Arguments
///
/// * `origins` - A slice of origin strings (e.g., &["https://example.com"])
///
/// # Returns
///
/// A configured `CorsLayer`.
pub fn create_cors_with_origins(origins: &[&str]) -> CorsLayer {
    let origin_values: Vec<HeaderValue> = origins
        .iter()
        .filter_map(|origin| origin.parse::<HeaderValue>().ok())
        .collect();

    if origin_values.is_empty() {
        return CorsLayer::permissive();
    }

    CorsLayer::new()
        .allow_origin(origin_values)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT])
        .allow_credentials(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_development_cors_is_permissive() {
        env::set_var("APICENTRIC_ENV", "development");
        let _cors = create_cors_layer();
        // In development, we use permissive CORS
        // This test just ensures it doesn't panic
    }

    #[test]
    fn test_default_cors_is_production() {
        env::remove_var("APICENTRIC_ENV");
        let _cors = create_cors_layer();
        // By default, if APICENTRIC_ENV is unset, it should use production CORS
    }

    #[test]
    fn test_invalid_env_cors_is_production() {
        env::set_var("APICENTRIC_ENV", "invalid_mode");
        let _cors = create_cors_layer();
        // If an unknown environment is provided, it should fail secure and use production CORS
    }

    #[test]
    fn test_production_cors_with_origins() {
        env::set_var("APICENTRIC_ENV", "production");
        env::set_var(
            "ALLOWED_ORIGINS",
            "https://example.com,https://app.example.com",
        );
        let _cors = create_cors_layer();
        // This test ensures production CORS can be created with valid origins
    }

    #[test]
    fn test_production_cors_without_origins() {
        env::set_var("APICENTRIC_ENV", "production");
        env::remove_var("ALLOWED_ORIGINS");
        let _cors = create_cors_layer();
        // Should fall back to localhost
    }

    #[test]
    fn test_custom_origins() {
        let _cors = create_cors_with_origins(&["https://example.com", "https://test.com"]);
        // This test ensures custom origins work
    }

    #[test]
    fn test_empty_custom_origins() {
        let _cors = create_cors_with_origins(&[]);
        // Should fall back to permissive
    }
}
