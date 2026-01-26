## 2024-05-23 - Path Traversal in Axum Handlers
**Vulnerability:** Path traversal in `iot_handlers` (save/get/delete twin) due to unsanitized user input from `Path(name)` being joined with a directory path.
**Learning:** `axum::extract::Path` decodes URL-encoded characters (like `%2e%2e` -> `..`), allowing attackers to bypass frontend validation and access/modify files outside the intended directory.
**Prevention:** Always sanitize file path components derived from user input using `std::path::Path::new(input).file_name()` before joining them to a base path.

## 2024-05-24 - Hardcoded JWT Secret Vulnerability
**Vulnerability:** The application fell back to a hardcoded "dev-secret-change-me" string when `APICENTRIC_JWT_SECRET` was missing, allowing token forgery in production if configuration was missed.
**Learning:** Default values for security-critical parameters (like secrets) are dangerous. Developers often forget to override defaults.
**Prevention:** Eliminate default secrets. Use a strategy of "Panic in Production" (if protection enabled but secret missing) and "Random in Development" (if protection disabled), ensuring secure-by-default behavior.
