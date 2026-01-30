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

## 2024-05-27 - Memory Exhaustion in File Uploads
**Vulnerability:** The `upload_replay_data` handler buffered the entire uploaded file into memory (`field.bytes().await`) before checking its size, allowing a denial-of-service (DoS) attack via memory exhaustion with large files.
**Learning:** Checking file size *after* reading into memory defeats the purpose of the check for DoS protection. `axum`'s `Multipart` extractor respects global limits, but if those are not strictly configured or if `bytes()` is used on a field without per-field limits, it reads until OOM.
**Prevention:** Use streaming (processing chunks) for file uploads. Check size incrementally while writing to disk or processing, and abort early if the limit is exceeded.
