## 2024-05-23 - Path Traversal in Axum Handlers
**Vulnerability:** Path traversal in `iot_handlers` (save/get/delete twin) due to unsanitized user input from `Path(name)` being joined with a directory path.
**Learning:** `axum::extract::Path` decodes URL-encoded characters (like `%2e%2e` -> `..`), allowing attackers to bypass frontend validation and access/modify files outside the intended directory.
**Prevention:** Always sanitize file path components derived from user input using `std::path::Path::new(input).file_name()` before joining them to a base path.
