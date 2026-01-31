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
