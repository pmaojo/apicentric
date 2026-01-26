## 2024-05-23 - Path Traversal in Axum Handlers
**Vulnerability:** Path traversal in `iot_handlers` (save/get/delete twin) due to unsanitized user input from `Path(name)` being joined with a directory path.
**Learning:** `axum::extract::Path` decodes URL-encoded characters (like `%2e%2e` -> `..`), allowing attackers to bypass frontend validation and access/modify files outside the intended directory.
**Prevention:** Always sanitize file path components derived from user input using `std::path::Path::new(input).file_name()` before joining them to a base path.

## 2024-05-24 - Missing Security Headers in Axum
**Vulnerability:** The cloud server was missing standard security headers (`X-Content-Type-Options`, `X-Frame-Options`, `Content-Security-Policy`, etc.), exposing it to common web attacks.
**Learning:** `axum` and `tower-http` do not add these headers by default. `tower-http` has a `set-header` feature, but if it's not enabled, you must implement custom middleware.
**Prevention:** Always explicitly add security headers middleware to public-facing servers. Use `tower_http::set_header` if available, or write a simple `axum::middleware` to inject them.
