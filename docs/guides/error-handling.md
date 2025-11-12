# Error Handling Guide

This guide explains the comprehensive error handling system implemented in the Apicentric cloud API.

## Overview

The cloud API uses a structured error handling system with:
- **Standard error codes** for consistent error identification
- **Typed error responses** with detailed information
- **Input validation** for all user-provided data
- **HTTP status code mapping** for proper REST semantics

## Error Response Format

All API errors follow this standard format:

```json
{
  "success": false,
  "code": "SERVICE_NOT_FOUND",
  "message": "Service 'my-service' not found",
  "details": {
    // Optional additional error details
  }
}
```

### Fields

- `success`: Always `false` for errors
- `code`: Machine-readable error code (see Error Codes section)
- `message`: Human-readable error description
- `details`: Optional JSON object with additional context

## Error Codes

### Service Errors

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `SERVICE_NOT_FOUND` | 404 | The requested service does not exist |
| `SERVICE_ALREADY_EXISTS` | 409 | A service with this name already exists |
| `SERVICE_ALREADY_RUNNING` | 409 | The service is already running |
| `SERVICE_NOT_RUNNING` | 400 | The service is not currently running |
| `SERVICE_START_FAILED` | 500 | Failed to start the service |
| `SERVICE_STOP_FAILED` | 500 | Failed to stop the service |

### File System Errors

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `FILE_NOT_FOUND` | 404 | The requested file does not exist |
| `FILE_ALREADY_EXISTS` | 409 | A file with this name already exists |
| `FILE_READ_ERROR` | 500 | Failed to read the file |
| `FILE_WRITE_ERROR` | 500 | Failed to write the file |
| `DIRECTORY_CREATE_ERROR` | 500 | Failed to create directory |

### Validation Errors

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `INVALID_YAML` | 400 | The YAML content is malformed |
| `INVALID_SERVICE_NAME` | 400 | The service name contains invalid characters |
| `INVALID_CONFIGURATION` | 400 | The configuration is invalid |
| `VALIDATION_FAILED` | 400 | Configuration validation failed |
| `YAML_TOO_LARGE` | 400 | YAML exceeds maximum size limit |
| `SERVICE_NAME_MISMATCH` | 400 | Service name in YAML doesn't match URL parameter |

### Recording Errors

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `RECORDING_NOT_ACTIVE` | 400 | No active recording session |
| `RECORDING_ALREADY_ACTIVE` | 409 | A recording session is already active |
| `RECORDING_START_FAILED` | 500 | Failed to start recording |
| `RECORDING_STOP_FAILED` | 500 | Failed to stop recording |
| `NO_REQUESTS_CAPTURED` | 400 | No requests were captured during recording |

### AI Errors

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `AI_NOT_CONFIGURED` | 400 | AI provider is not configured |
| `AI_GENERATION_FAILED` | 500 | AI generation failed |
| `AI_PROVIDER_ERROR` | 500 | Error from AI provider |
| `INVALID_AI_PROVIDER` | 400 | Unknown AI provider specified |

### Code Generation Errors

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `CODE_GENERATION_FAILED` | 500 | Failed to generate code |

### Configuration Errors

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `CONFIG_LOAD_ERROR` | 500 | Failed to load configuration |
| `CONFIG_SAVE_ERROR` | 500 | Failed to save configuration |
| `CONFIG_VALIDATION_ERROR` | 400 | Configuration validation failed |

### Authentication Errors

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `AUTHENTICATION_REQUIRED` | 401 | Authentication is required |
| `INVALID_TOKEN` | 401 | The provided token is invalid |
| `TOKEN_EXPIRED` | 401 | The authentication token has expired |

### General Errors

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `INTERNAL_ERROR` | 500 | An internal server error occurred |
| `INVALID_REQUEST` | 400 | The request is malformed |
| `INVALID_PARAMETER` | 400 | A parameter has an invalid value |
| `MISSING_PARAMETER` | 400 | A required parameter is missing |

## Input Validation

The API performs comprehensive input validation to prevent errors and security issues.

### Service Name Validation

Service names must:
- Not be empty
- Be 100 characters or less
- Contain only alphanumeric characters, hyphens, and underscores
- Not contain path separators (`/`, `\`) or parent directory references (`..`)

**Example Error:**
```json
{
  "success": false,
  "code": "INVALID_SERVICE_NAME",
  "message": "Invalid service name '../etc/passwd': name cannot contain path separators or parent directory references"
}
```

### YAML Size Validation

YAML content is limited to **10 MB** to prevent memory exhaustion attacks.

**Example Error:**
```json
{
  "success": false,
  "code": "YAML_TOO_LARGE",
  "message": "YAML size (15728640 bytes) exceeds maximum allowed size (10485760 bytes)"
}
```

### Parameter Validation

Required parameters are validated and missing parameters result in clear error messages.

**Example Error:**
```json
{
  "success": false,
  "code": "MISSING_PARAMETER",
  "message": "Required parameter 'service_name' is missing"
}
```

## Error Handling in Client Code

### JavaScript/TypeScript Example

```typescript
async function startService(name: string) {
  try {
    const response = await fetch(`/api/services/${name}/start`, {
      method: 'POST',
    });
    
    const data = await response.json();
    
    if (!data.success) {
      // Handle specific error codes
      switch (data.code) {
        case 'SERVICE_NOT_FOUND':
          console.error(`Service ${name} does not exist`);
          break;
        case 'SERVICE_ALREADY_RUNNING':
          console.warn(`Service ${name} is already running`);
          break;
        case 'INVALID_SERVICE_NAME':
          console.error(`Invalid service name: ${data.message}`);
          break;
        default:
          console.error(`Error: ${data.message}`);
      }
      return;
    }
    
    console.log('Service started successfully');
  } catch (error) {
    console.error('Network error:', error);
  }
}
```

### Rust Example

```rust
use apicentric::cloud::error::{ApiError, ApiErrorCode, ErrorResponse};

async fn handle_request() -> Result<(), ApiError> {
    // Validate service name
    validation::validate_service_name(&name)
        .map_err(ApiError::from)?;
    
    // Perform operation
    match simulator.start_service(&name).await {
        Ok(_) => Ok(()),
        Err(e) => {
            if e.to_string().contains("not found") {
                Err(ErrorResponse::service_not_found(&name).into())
            } else {
                Err(ApiError::internal_server_error(e.to_string()))
            }
        }
    }
}
```

## Best Practices

### 1. Always Check Error Codes

Don't rely solely on HTTP status codes. Use the `code` field for precise error handling:

```typescript
if (response.code === 'SERVICE_NOT_FOUND') {
  // Show "create service" dialog
} else if (response.code === 'SERVICE_ALREADY_RUNNING') {
  // Update UI to show running state
}
```

### 2. Display User-Friendly Messages

The `message` field is human-readable but may contain technical details. Consider mapping error codes to user-friendly messages:

```typescript
const ERROR_MESSAGES = {
  SERVICE_NOT_FOUND: 'The service you requested could not be found.',
  INVALID_YAML: 'The service configuration contains syntax errors.',
  YAML_TOO_LARGE: 'The service configuration is too large. Please reduce its size.',
};

const userMessage = ERROR_MESSAGES[response.code] || response.message;
```

### 3. Use Details for Additional Context

When available, the `details` field provides structured information:

```typescript
if (response.code === 'VALIDATION_FAILED' && response.details?.errors) {
  response.details.errors.forEach(error => {
    console.error(`Validation error: ${error}`);
  });
}
```

### 4. Handle Network Errors Separately

Distinguish between API errors and network failures:

```typescript
try {
  const response = await fetch('/api/services');
  const data = await response.json();
  
  if (!data.success) {
    // API returned an error
    handleApiError(data);
  }
} catch (error) {
  // Network error or JSON parsing error
  handleNetworkError(error);
}
```

## Security Considerations

The error handling system includes several security features:

1. **Input Validation**: Prevents path traversal, injection attacks, and resource exhaustion
2. **Size Limits**: YAML content is limited to 10 MB
3. **Safe Error Messages**: Error messages don't expose sensitive system information
4. **Consistent Responses**: All errors follow the same format, preventing information leakage

## Testing Error Handling

Example test for error handling:

```rust
#[tokio::test]
async fn test_invalid_service_name() {
    let result = start_service(
        Path("../etc/passwd".to_string()),
        State(simulator),
    ).await;
    
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.response.code, ApiErrorCode::InvalidServiceName);
}
```

## Migration Guide

If you're updating existing code to use the new error handling:

### Before

```rust
pub async fn start_service(name: String) -> Result<Json<ApiResponse<String>>, StatusCode> {
    match simulator.start_service(&name).await {
        Ok(_) => Ok(Json(ApiResponse::success("Started".to_string()))),
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}
```

### After

```rust
pub async fn start_service(name: String) -> Result<Json<ApiResponse<String>>, ApiError> {
    // Add validation
    validation::validate_service_name(&name)
        .map_err(ApiError::from)?;
    
    match simulator.start_service(&name).await {
        Ok(_) => Ok(Json(ApiResponse::success("Started".to_string()))),
        Err(e) => {
            // Return structured error
            if e.to_string().contains("not found") {
                Err(ErrorResponse::service_not_found(&name).into())
            } else {
                Err(ApiError::internal_server_error(e.to_string()))
            }
        }
    }
}
```

## Summary

The comprehensive error handling system provides:

- ✅ **Consistent error format** across all endpoints
- ✅ **Machine-readable error codes** for precise error handling
- ✅ **Input validation** to prevent security issues
- ✅ **Clear error messages** for debugging
- ✅ **Proper HTTP status codes** for REST semantics
- ✅ **Type safety** with Rust's type system

For questions or issues, please refer to the [API documentation](./configuration-api.md) or open an issue on GitHub.
