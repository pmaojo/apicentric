## 2024-05-23 - Path Traversal in Axum Handlers
**Vulnerability:** Path traversal in `iot_handlers` (save/get/delete twin) due to unsanitized user input from `Path(name)` being joined with a directory path.
**Learning:** `axum::extract::Path` decodes URL-encoded characters (like `%2e%2e` -> `..`), allowing attackers to bypass frontend validation and access/modify files outside the intended directory.
**Prevention:** Always sanitize file path components derived from user input using `std::path::Path::new(input).file_name()` before joining them to a base path.

## 2024-05-24 - Hardcoded JWT Secret Vulnerability
**Vulnerability:** The application fell back to a hardcoded "dev-secret-change-me" string when `APICENTRIC_JWT_SECRET` was missing, allowing token forgery in production if configuration was missed.
**Learning:** Default values for security-critical parameters (like secrets) are dangerous. Developers often forget to override defaults.
**Prevention:** Eliminate default secrets. Use a strategy of "Panic in Production" (if protection enabled but secret missing) and "Random in Development" (if protection disabled), ensuring secure-by-default behavior.

## 2024-05-25 - Path Traversal in Service Handlers
**Vulnerability:** Path traversal in `load_service` and `save_service` handlers where the `path` parameter from the JSON body was used directly in filesystem operations.
**Learning:** Handlers accepting file paths as strings are inherently dangerous. Even if the parameter is named "path", it should likely be treated as a "key" or "filename" and strictly sanitized.
**Prevention:** Enforce usage of managed directories (like `APICENTRIC_SERVICES_DIR`) and always strip directory components using `Path::file_name()` for any user-supplied filename.

## 2024-05-26 - Path Traversal in Service Creation Handlers
**Vulnerability:** Path traversal in `create_service` and `generate_service_from_recording` due to trusting user-provided `filename` or `service_name` directly for file creation.
**Learning:** New endpoints often repeat old mistakes. Centralized helper functions for security logic are essential to prevent regression and ensure consistency.
**Prevention:** Implemented `resolve_safe_service_path` helper in `src/cloud/handlers.rs` and enforced `validation::validate_service_name`. All handlers dealing with service files now use this shared logic.

## 2024-05-27 - DoS Vulnerability in File Uploads
**Vulnerability:** Unbounded memory allocation in `upload_replay_data` handler (`src/cloud/iot_handlers.rs`). The handler used `field.bytes().await` which buffers the entire file into RAM, allowing an attacker to cause an OOM crash by uploading a large file.
**Learning:** `axum::extract::Multipart::Field::bytes()` is convenient but dangerous for file uploads. Streaming is required for safety. Also, `DefaultBodyLimit` in Axum (2MB) protects against this by default, but if disabled or if the limit is raised globally, the handler becomes vulnerable.
**Prevention:** Always stream file uploads using `field.chunk().await` and write to disk/storage incrementally. Enforce a hard limit on the total bytes read within the loop.

## 2024-05-28 - SSRF Bypass via IPv4-Mapped IPv6 Addresses
**Vulnerability:** The SSRF protection in `is_global` failed to handle IPv4-mapped IPv6 addresses (e.g., `::ffff:127.0.0.1`), allowing attackers to bypass IPv4 blocklists (like loopback) by using IPv6 syntax.
**Learning:** Rust's `IpAddr::V6` checks do not automatically apply IPv4 constraints to mapped addresses. Attackers can often bypass filters by "encoding" their payload in a different format (like IPv6 mapped addresses) if the validator doesn't normalize it first.
**Prevention:** Explicitly check for `ipv6.to_ipv4()` and recursively validate the underlying IPv4 address. Also ensure that `0.0.0.0/8` (Current Network) is blocked in IPv4, as some IPv6 compatible addresses map to it.

## 2024-05-29 - CSV Injection in Log Exports
**Vulnerability:** The `export_logs` handler generated CSV files by formatting strings directly without sanitization, allowing attackers to inject malicious formulas (Formula Injection) into the logs (e.g., via `User-Agent` or `path`).
**Learning:** Text-based formats like CSV are deceptively simple. When opened by spreadsheet software (Excel, LibreOffice), fields starting with `=`, `+`, `-`, or `@` are executed as formulas, leading to data exfiltration or RCE on the admin's machine.
**Prevention:** Implement a dedicated sanitizer for CSV exports that escapes delimiters/quotes AND neutralizes formulas by prepending a single quote (`'`) to dangerous prefixes. Avoid ad-hoc string formatting for CSV generation.

## 2024-05-30 - Sensitive Data Exposure in Config API
**Vulnerability:** The `get_config` endpoint returned the full `ApicentricConfig` structure serialized to JSON, including the `ai.api_key` field, exposing the OpenAI/Gemini API key to anyone with access to the UI/API.
**Learning:** Default serialization (#[derive(Serialize)]) is dangerous for configuration objects containing secrets. It's easy to forget that internal config structures might be exposed via API.
**Prevention:** Implement dedicated `redact_sensitive_fields()` methods for configuration structs and ensure they are called before returning config data to the client. Alternatively, use `#[serde(skip_serializing)]` or custom serializers for secret fields if they never need to be sent back (but here we need to support updates).

## 2024-05-31 - Admin Server Authentication Bypass
**Vulnerability:** The `AdminServer` allowed unauthenticated access to sensitive endpoints (e.g., `/apicentric-admin/logs`) if the `APICENTRIC_ADMIN_TOKEN` environment variable was not set.
**Learning:** Checking for an optional configuration value (like an auth token) and proceeding without it if missing can lead to "Fail Open" scenarios where security is disabled by default.
**Prevention:** Implement "Fail Secure" logic. If a critical security configuration (like an admin token) is missing, deny access or fail to start, rather than disabling the security check.
