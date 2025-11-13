# Configuration Guide

This guide explains Apicentric's configuration system, including default values and when configuration is required versus optional.

## Configuration File

Apicentric uses `apicentric.json` as its main configuration file. This file is **optional** for basic simulator usage but required for advanced features like contract testing and CI integration.

### Location

By default, Apicentric looks for `apicentric.json` in the current directory. You can specify a different location:

```bash
apicentric --config /path/to/config.json simulator start
```

### Minimal Configuration

For basic simulator usage, you don't need a configuration file at all. Just create service definitions and run:

```bash
apicentric simulator start --services-dir ./services
```

### Full Configuration Example

```json
{
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
  "simulator": {
    "enabled": true,
    "services_dir": "services",
    "port_range": {
      "start": 8000,
      "end": 8999
    },
    "db_path": "apicentric.db"
  },
  "ai": {
    "provider": "openai",
    "api_key": "your-api-key",
    "model": "gpt-4"
  }
}
```

## Configuration Sections

### Core Settings

These settings are used for contract testing and Cypress integration.

| Field | Default | Required | Description |
|-------|---------|----------|-------------|
| `base_url` | `"http://localhost:5173"` | No | Base URL for the application under test |
| `default_timeout` | `30000` | No | Default timeout in milliseconds |
| `routes_dir` | `"app/routes"` | No | Directory containing route files |
| `specs_dir` | `"app/routes"` | No | Directory containing test specs |
| `reports_dir` | `"cypress/reports"` | No | Directory for test reports |

**When Required**: Only needed when using contract testing features.

### Server Configuration

Controls automatic server startup for testing.

| Field | Default | Required | Description |
|-------|---------|----------|-------------|
| `server.auto_start` | `false` | No | Automatically start server before tests |
| `server.start_command` | `"npm run dev"` | No | Command to start the server |
| `server.startup_timeout_ms` | `30000` | No | Time to wait for server startup |
| `server.health_check_retries` | `5` | No | Number of health check attempts |
| `server.skip_health_check` | `false` | No | Skip health check after startup |

**When Required**: Only needed when using `auto_start` feature.

**Sensible Defaults**: The defaults work for most Node.js projects using `npm run dev`.

### Execution Configuration

Controls test execution behavior.

| Field | Default | Required | Description |
|-------|---------|----------|-------------|
| `execution.mode` | `"development"` | No | Execution mode: `ci`, `development`, or `debug` |
| `execution.continue_on_failure` | `true` | No | Continue running tests after failures |
| `execution.dry_run` | `false` | No | Show what would be executed without running |
| `execution.verbose` | `false` | No | Enable verbose output |

**When Required**: Optional. Defaults are suitable for development.

**Override**: Can be overridden with CLI flags:
- `--mode ci`
- `--dry-run`
- `--verbose`

### Simulator Configuration

Controls the API simulator behavior.

| Field | Default | Required | Description |
|-------|---------|----------|-------------|
| `simulator.enabled` | `false` | No | Enable the simulator |
| `simulator.services_dir` | `"services"` | No | Directory containing service definitions |
| `simulator.port_range.start` | `8000` | No | Start of port range for services |
| `simulator.port_range.end` | `8999` | No | End of port range for services |
| `simulator.db_path` | `"apicentric.db"` | No | Path to SQLite database |

**When Required**: Optional. The simulator can be used without a configuration file.

**Sensible Defaults**: 
- Port range `8000-8999` avoids conflicts with common development ports
- Services directory `services` is a standard convention
- Database path `apicentric.db` keeps data in the project root

### NPM Integration

Controls NPM script integration.

| Field | Default | Required | Description |
|-------|---------|----------|-------------|
| `npm.apicentric_script` | `"cargo run ..."` | No | Command to run Apicentric |
| `npm.apicentric_watch_script` | `"cargo run ... watch"` | No | Command for watch mode |
| `npm.dev_script` | `"npm run dev"` | No | Development server command |

**When Required**: Only needed when integrating with NPM scripts.

### AI Configuration

Controls AI-assisted generation features.

| Field | Default | Required | Description |
|-------|---------|----------|-------------|
| `ai.provider` | - | Yes* | AI provider: `local` or `openai` |
| `ai.api_key` | - | Yes* | API key for OpenAI |
| `ai.model` | - | No | Model identifier (e.g., `gpt-4`) |
| `ai.model_path` | - | Yes* | Path to local model file |

**When Required**: Only when using `apicentric ai generate` command.

*Required fields depend on provider:
- OpenAI: `provider`, `api_key` required
- Local: `provider`, `model_path` required

## Service Definition Configuration

Service definitions are YAML files that describe mock APIs. They have their own configuration structure.

### Minimal Service Definition

```yaml
name: my-api
server:
  port: 9000
  base_path: /api
endpoints:
  - method: GET
    path: /hello
    responses:
      200:
        content_type: application/json
        body: '{"message": "Hello, World!"}'
```

### Server Configuration (Service Level)

| Field | Default | Required | Description |
|-------|---------|----------|-------------|
| `server.port` | Auto-assigned | No | Port number for the service |
| `server.base_path` | `"/"` | Yes | Base path for all endpoints |
| `server.proxy_base_url` | - | No | Proxy URL for unmatched requests |
| `server.cors.enabled` | `false` | No | Enable CORS |
| `server.cors.origins` | `["*"]` | No | Allowed origins |

**Sensible Defaults**:
- If `port` is omitted, Apicentric assigns a port from the configured range
- `base_path` must be specified to avoid ambiguity
- CORS is disabled by default for security

### Endpoint Configuration

| Field | Default | Required | Description |
|-------|---------|----------|-------------|
| `method` | - | Yes | HTTP method (GET, POST, etc.) |
| `path` | - | Yes | Endpoint path |
| `description` | - | No | Human-readable description |
| `parameters` | `[]` | No | Path/query parameters |
| `request_body` | - | No | Request body schema |
| `responses` | - | Yes | Response definitions by status code |

**Required Fields**: Only `method`, `path`, and at least one response are required.

### Response Configuration

| Field | Default | Required | Description |
|-------|---------|----------|-------------|
| `content_type` | `"application/json"` | No | Response content type |
| `body` | `""` | No | Response body (string or template) |
| `headers` | `{}` | No | Additional response headers |
| `delay_ms` | `0` | No | Artificial delay in milliseconds |

**Sensible Defaults**:
- `content_type` defaults to JSON, the most common API format
- Empty body is allowed for 204 No Content responses
- No delay by default for fast development

## Configuration Best Practices

### 1. Start Minimal

Don't create a configuration file until you need it. For basic simulator usage:

```bash
# No config needed!
apicentric simulator start
```

### 2. Use Environment Variables

For sensitive data like API keys, use environment variables:

```json
{
  "ai": {
    "provider": "openai",
    "api_key": "${OPENAI_API_KEY}"
  }
}
```

### 3. Document Custom Values

If you change defaults, document why in comments (use a `.jsonc` file):

```jsonc
{
  "simulator": {
    "port_range": {
      "start": 9000,  // Avoid conflict with other services
      "end": 9999
    }
  }
}
```

### 4. Use Sensible Defaults

The defaults are chosen to work for most projects:
- Port range `8000-8999` avoids common conflicts
- Timeout `30000ms` works for most development servers
- `continue_on_failure: true` is better for development

Only change defaults if you have a specific reason.

### 5. Validate Configuration

Use the validate command to check your configuration:

```bash
apicentric simulator validate --path services/
```

## Common Configuration Scenarios

### Scenario 1: Basic Simulator Only

**No configuration file needed!**

```bash
apicentric simulator start --services-dir ./services
```

### Scenario 2: Simulator with Custom Port Range

Create `apicentric.json`:

```json
{
  "simulator": {
    "enabled": true,
    "port_range": {
      "start": 9000,
      "end": 9999
    }
  }
}
```

### Scenario 3: Contract Testing

Create `apicentric.json`:

```json
{
  "base_url": "http://localhost:3000",
  "default_timeout": 30000,
  "simulator": {
    "enabled": true,
    "services_dir": "mocks"
  }
}
```

### Scenario 4: CI/CD Pipeline

Create `apicentric.json`:

```json
{
  "execution": {
    "mode": "ci",
    "continue_on_failure": false
  },
  "server": {
    "auto_start": true,
    "start_command": "npm run start:ci"
  }
}
```

Override in CI:

```bash
apicentric --mode ci simulator start
```

### Scenario 5: AI-Assisted Development

Create `apicentric.json`:

```json
{
  "ai": {
    "provider": "openai",
    "api_key": "${OPENAI_API_KEY}",
    "model": "gpt-4"
  },
  "simulator": {
    "enabled": true
  }
}
```

## Configuration Validation

Apicentric validates configuration on startup and provides helpful error messages:

```
❌ Configuration error: Invalid port range
💡 Suggestion: Port range must be between 1024 and 65535
🔍 Field: simulator.port_range
```

Common validation errors:

1. **Invalid URL format**: Use `http://` or `https://` prefix
2. **Port out of range**: Use ports 1024-65535
3. **Invalid base path**: Must start with `/`
4. **Missing required field**: Check the error message for the field name

## Configuration Reference

For a complete reference of all configuration options, see:
- [Simulator Configuration](./simulator.md)
- [Contract Testing Configuration](./contract-testing.md)
- [AI Configuration](./ai-generation.md)

## Troubleshooting

### Configuration Not Found

If Apicentric can't find your configuration:

```bash
# Specify the path explicitly
apicentric --config ./config/apicentric.json simulator start
```

### Invalid Configuration

If your configuration is invalid:

1. Check the error message for the specific field
2. Refer to this guide for correct format
3. Use the validate command: `apicentric simulator validate`

### Environment Variables Not Working

Ensure environment variables are exported:

```bash
export OPENAI_API_KEY=your-key
apicentric ai generate "create a user API"
```

## Summary

- **Configuration is optional** for basic simulator usage
- **Defaults are sensible** and work for most projects
- **Only configure what you need** - start minimal
- **Validate your configuration** before running
- **Use environment variables** for sensitive data
