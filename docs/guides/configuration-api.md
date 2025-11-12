# Configuration Management API

The Apicentric Cloud Server provides REST API endpoints for managing the application configuration programmatically.

## Endpoints

### GET /api/config

Retrieves the current Apicentric configuration.

**Response:**
```json
{
  "success": true,
  "data": {
    "cypress_config_path": "cypress.config.ts",
    "base_url": "http://localhost:5173",
    "specs_pattern": "app/routes/**/test/*.cy.ts",
    "routes_dir": "app/routes",
    "specs_dir": "app/routes",
    "reports_dir": "cypress/reports",
    "index_cache_path": ".apicentric/route-index.json",
    "default_timeout": 30000,
    "server": {
      "auto_start": false,
      "start_command": "npm run dev",
      "startup_timeout_ms": 30000,
      "health_check_retries": 5,
      "skip_health_check": false
    },
    "execution": {
      "mode": "development",
      "continue_on_failure": true,
      "dry_run": false,
      "verbose": false
    },
    "npm": {
      "apicentric_script": "cargo run --manifest-path utils/apicentric/Cargo.toml --",
      "apicentric_watch_script": "cargo run --manifest-path utils/apicentric/Cargo.toml -- watch",
      "dev_script": "npm run dev"
    },
    "ai": {
      "provider": "openai",
      "api_key": "sk-...",
      "model": "gpt-4"
    },
    "simulator": {
      "enabled": true,
      "services_dir": "services",
      "admin_port": 9999
    }
  },
  "error": null
}
```

### PUT /api/config

Updates the Apicentric configuration. The configuration is validated before being saved.

**Request Body:**
```json
{
  "config": {
    "base_url": "http://localhost:3000",
    "default_timeout": 60000,
    "ai": {
      "provider": "openai",
      "api_key": "sk-new-key",
      "model": "gpt-4"
    }
  }
}
```

**Response (Success):**
```json
{
  "success": true,
  "data": "Configuration updated successfully",
  "error": null
}
```

**Response (Validation Error):**
```json
{
  "success": false,
  "data": null,
  "error": "Configuration validation failed:\nbase_url: Invalid URL format\ndefault_timeout: Value must be between 1000 and 300000"
}
```

### POST /api/config/validate

Validates a configuration without saving it. Useful for checking configuration changes before applying them.

**Request Body:**
```json
{
  "config": {
    "base_url": "not-a-valid-url",
    "default_timeout": 500000
  }
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "is_valid": false,
    "errors": [
      "base_url: Invalid URL format",
      "default_timeout: Value must be between 1000 and 300000"
    ]
  },
  "error": null
}
```

## Configuration File Location

By default, the configuration is loaded from and saved to `apicentric.json` in the current working directory. You can override this by setting the `APICENTRIC_CONFIG_PATH` environment variable:

```bash
export APICENTRIC_CONFIG_PATH=/path/to/custom-config.json
```

## Configuration Validation Rules

The configuration API validates all settings according to the following rules:

### Core Settings
- `base_url`: Must be a valid HTTP/HTTPS URL
- `default_timeout`: Must be between 1000 and 300000 milliseconds
- `routes_dir`: Directory must exist
- `specs_dir`: Directory must exist

### Server Settings
- `start_command`: Cannot be empty
- `startup_timeout_ms`: Must be between 5000 and 120000 milliseconds
- `health_check_retries`: Must be between 1 and 20

### AI Settings
- For `openai` provider: `api_key` is required
- For `gemini` provider: `api_key` is required (or `GEMINI_API_KEY` environment variable)
- For `local` provider: `model_path` is required and file must exist

### Simulator Settings
- `services_dir`: Directory must exist or be creatable
- `admin_port`: Must be a valid port number (1-65535)

## Example Usage

### Using curl

**Get current configuration:**
```bash
curl http://localhost:8000/api/config
```

**Update configuration:**
```bash
curl -X PUT http://localhost:8000/api/config \
  -H "Content-Type: application/json" \
  -d '{
    "config": {
      "base_url": "http://localhost:3000",
      "default_timeout": 60000
    }
  }'
```

**Validate configuration:**
```bash
curl -X POST http://localhost:8000/api/config/validate \
  -H "Content-Type: application/json" \
  -d '{
    "config": {
      "base_url": "http://localhost:3000",
      "default_timeout": 60000
    }
  }'
```

### Using JavaScript/TypeScript

```typescript
// Get configuration
const response = await fetch('http://localhost:8000/api/config');
const { data: config } = await response.json();

// Update configuration
await fetch('http://localhost:8000/api/config', {
  method: 'PUT',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    config: {
      ...config,
      default_timeout: 60000,
      ai: {
        provider: 'openai',
        api_key: 'sk-new-key',
        model: 'gpt-4'
      }
    }
  })
});

// Validate before saving
const validateResponse = await fetch('http://localhost:8000/api/config/validate', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    config: newConfig
  })
});
const { data: validation } = await validateResponse.json();
if (!validation.is_valid) {
  console.error('Validation errors:', validation.errors);
}
```

## Security Considerations

- Configuration endpoints may contain sensitive information (API keys, database credentials)
- In production, ensure these endpoints are protected with authentication
- Consider masking sensitive fields in GET responses
- Use HTTPS in production to protect API keys in transit
- Set `APICENTRIC_PROTECT_SERVICES=true` to require JWT authentication for all API endpoints

## Related Documentation

- [Configuration Guide](./configuration.md) - Detailed configuration options
- [Quick Start Guide](./quick-start.md) - Getting started with Apicentric
- [Features Guide](./features.md) - Overview of all features
