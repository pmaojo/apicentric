//! Centralized logging configuration using tracing
//!
//! This module provides structured logging capabilities with:
//! - Configurable log levels via environment variables
//! - JSON output for production
//! - Performance-optimized debug logging
//! - Contextual spans for operations

use std::env;
// `tracing::info!` is used via its macro; avoid importing a conflicting symbol.
use tracing_subscriber::{filter::EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize the global tracing subscriber
///
/// This should be called early in the application lifecycle, typically in main().
/// It configures logging based on:
/// - RUST_LOG environment variable (e.g., "apicentric=debug,simulator=info")
/// - APICENTRIC_LOG_FORMAT environment variable ("json" or "pretty")
/// - Default level is INFO for production, DEBUG for development
pub fn init() {
    let filter = EnvFilter::try_from_env("RUST_LOG")
        .unwrap_or_else(|_| {
            // Default filter: info level, debug for development
            if cfg!(debug_assertions) {
                EnvFilter::new("apicentric=debug,simulator=info,domain=info")
            } else {
                EnvFilter::new("apicentric=info,simulator=warn,domain=warn")
            }
        });

    let registry = tracing_subscriber::registry().with(filter);

    // Choose output format based on environment
    let format = env::var("APICENTRIC_LOG_FORMAT")
        .unwrap_or_else(|_| "pretty".to_string());

    match format.as_str() {
        "json" => {
            // JSON format for production/log aggregation
            let json_layer = tracing_subscriber::fmt::layer()
                .json()
                .with_writer(std::io::stderr)
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_file(true)
                .with_line_number(true);

            registry.with(json_layer).init();
        }
        _ => {
            // Pretty format for development
            let pretty_layer = tracing_subscriber::fmt::layer()
                .pretty()
                .with_writer(std::io::stderr)
                .with_target(true)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_file(true)
                .with_line_number(true);

            registry.with(pretty_layer).init();
        }
    }

    // Log initialization
    tracing::info!(
        target: "apicentric::logging",
        format = %format,
        "Logging initialized"
    );
}

/// Create a span for service operations
#[macro_export]
macro_rules! service_span {
    ($service:expr) => {
        tracing::span!(
            tracing::Level::INFO,
            "service_operation",
            service = $service,
            operation = tracing::field::Empty
        )
    };
}

/// Create a span for request handling
#[macro_export]
macro_rules! request_span {
    ($method:expr, $path:expr, $service:expr) => {
        tracing::span!(
            tracing::Level::INFO,
            "http_request",
            method = $method,
            path = $path,
            service = $service,
            status = tracing::field::Empty,
            duration_ms = tracing::field::Empty
        )
    };
}

/// Log with structured fields for contract operations
#[macro_export]
macro_rules! contract_log {
    ($level:ident, $contract_id:expr, $service:expr, $($field:tt)*) => {
        tracing::event!(
            tracing::Level::$level,
            contract_id = $contract_id,
            service = $service,
            $($field)*
        )
    };
}

/// Performance-optimized debug logging that only evaluates arguments when debug is enabled
#[macro_export]
macro_rules! debug_lazy {
    ($($arg:tt)*) => {
        if tracing::level_enabled!(tracing::Level::DEBUG) {
            tracing::debug!($($arg)*);
        }
    };
}

/// Log simulator events with structured data
#[macro_export]
macro_rules! simulator_log {
    ($level:ident, $service:expr, $event:expr, $($field:tt)*) => {
        tracing::event!(
            target: "simulator",
            tracing::Level::$level,
            service = $service,
            event = $event,
            $($field)*
        )
    };
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_logging_macros() {
        // Test that macros compile
        let _span = service_span!("test-service");
        contract_log!(INFO, "contract-123", "service-1", message = "test");
        simulator_log!(INFO, "service-1", "started", port = 8080);
    }
}
