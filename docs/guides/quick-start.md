# Quick Start Guide

Get up and running with Apicentric in 5 minutes.

## Prerequisites

- A terminal (macOS Terminal, Linux shell, or Windows PowerShell)
- Basic familiarity with command-line tools
- (Optional) curl or similar tool for testing APIs

## Installation

Choose your preferred installation method:

### Homebrew (macOS/Linux)

```bash
brew install pmaojo/tap/apicentric
```

### Install Script (Unix)

```bash
curl -fsSL https://raw.githubusercontent.com/pmaojo/apicentric/main/scripts/install.sh | sh
```

### Cargo

```bash
cargo install apicentric --features cli-tools
```

### Verify Installation

```bash
apicentric --version
```

You should see output like: `apicentric 0.1.0`

## Your First Mock API

Let's create a simple API that returns a list of users.

### Step 1: Create a Service Definition

Create a file named `users-api.yaml`:

```yaml
name: users-api
server:
  port: 9000
  base_path: /api
endpoints:
  - method: GET
    path: /users
    responses:
      200:
        content_type: application/json
        body: |
          [
            {"id": 1, "name": "Alice", "email": "alice@example.com"},
            {"id": 2, "name": "Bob", "email": "bob@example.com"}
          ]
  
  - method: GET
    path: /users/{id}
    responses:
      200:
        content_type: application/json
        body: |
          {"id": 1, "name": "Alice", "email": "alice@example.com"}
  
  - method: POST
    path: /users
    responses:
      201:
        content_type: application/json
        body: |
          {"id": 3, "name": "New User", "email": "new@example.com"}
```

### Step 2: Validate the Service

Before starting the simulator, validate your service definition:

```bash
apicentric simulator validate --path users-api.yaml
```

You should see: `✅ Validation successful`

### Step 3: Start the Simulator

Start the API simulator:

```bash
apicentric simulator start --services-dir .
```

You should see output indicating the service is running:

```
🚀 Starting API Simulator...
✅ Service 'users-api' started on http://localhost:9000
```

### Step 4: Test Your API

In a new terminal window, test your endpoints:

```bash
# Get all users
curl http://localhost:9000/api/users

# Get a specific user
curl http://localhost:9000/api/users/1

# Create a new user
curl -X POST http://localhost:9000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name": "Charlie", "email": "charlie@example.com"}'
```

### Step 5: Stop the Simulator

Press `Ctrl+C` in the terminal where the simulator is running to stop it.

## Using the Terminal UI

Apicentric includes an interactive terminal UI for managing services.

### Start the TUI

```bash
apicentric tui
```

### TUI Features

The TUI provides three panels:

1. **Services Panel** (left): List of running services
2. **Logs Panel** (center): Real-time request logs
3. **Actions Panel** (right): Keyboard shortcuts

### Keyboard Shortcuts

- `↑`/`↓` - Navigate services
- `Enter` - Start/stop selected service
- `f` - Filter logs
- `c` - Clear logs
- `r` - Refresh status
- `q` - Quit

## Next Steps

### Add Dynamic Responses

Use Handlebars templates for dynamic responses:

```yaml
endpoints:
  - method: GET
    path: /users/{id}
    responses:
      200:
        content_type: application/json
        body: |
          {
            "id": "{{params.id}}",
            "name": "User {{params.id}}",
            "timestamp": "{{now}}"
          }
```

### Add Multiple Services

Create a directory structure:

```
services/
├── users-api.yaml
├── products-api.yaml
└── orders-api.yaml
```

Start all services:

```bash
apicentric simulator start --services-dir services
```

### Generate TypeScript Types

Export TypeScript interfaces from your service:

```bash
apicentric simulator export-types \
  --input users-api.yaml \
  --output types.ts
```

### Generate React Query Hooks

Export React Query hooks:

```bash
apicentric simulator export-query \
  --input users-api.yaml \
  --output api-hooks.ts
```

## Common Patterns

### REST CRUD API

```yaml
name: crud-api
server:
  port: 9000
  base_path: /api
endpoints:
  - method: GET
    path: /items
    responses:
      200:
        content_type: application/json
        body: '[]'
  
  - method: GET
    path: /items/{id}
    responses:
      200:
        content_type: application/json
        body: '{"id": "{{params.id}}"}'
  
  - method: POST
    path: /items
    responses:
      201:
        content_type: application/json
        body: '{"id": "new-id"}'
  
  - method: PUT
    path: /items/{id}
    responses:
      200:
        content_type: application/json
        body: '{"id": "{{params.id}}", "updated": true}'
  
  - method: DELETE
    path: /items/{id}
    responses:
      204:
        content_type: text/plain
        body: ''
```

### API with Authentication

```yaml
name: auth-api
server:
  port: 9000
  base_path: /api
endpoints:
  - method: POST
    path: /login
    responses:
      200:
        content_type: application/json
        headers:
          Set-Cookie: "session=abc123; HttpOnly"
        body: |
          {"token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."}
  
  - method: GET
    path: /profile
    header_match:
      Authorization: "Bearer *"
    responses:
      200:
        content_type: application/json
        body: |
          {"id": 1, "name": "User", "email": "user@example.com"}
```

### API with Error Responses

```yaml
name: error-api
server:
  port: 9000
  base_path: /api
endpoints:
  - method: GET
    path: /items/{id}
    responses:
      200:
        content_type: application/json
        body: '{"id": "{{params.id}}"}'
      404:
        content_type: application/json
        body: '{"error": "Item not found"}'
      500:
        content_type: application/json
        body: '{"error": "Internal server error"}'
```

## Troubleshooting

### Port Already in Use

If you see an error about the port being in use:

```bash
# Change the port in your YAML file
server:
  port: 9001  # Use a different port
```

### Service Not Starting

Check the logs for errors:

```bash
apicentric simulator start --services-dir . --verbose
```

### Invalid YAML

Validate your YAML syntax:

```bash
apicentric simulator validate --path your-service.yaml --verbose
```

### Can't Connect to API

Verify the service is running:

```bash
# Check if the port is listening
lsof -i :9000

# Or use netstat
netstat -an | grep 9000
```

## Getting Help

- Check the [documentation](https://github.com/pmaojo/apicentric)
- Open an [issue](https://github.com/pmaojo/apicentric/issues)
- Start a [discussion](https://github.com/pmaojo/apicentric/discussions)

## What's Next?

- Learn about [advanced features](../README.md#features)
- Explore [contract testing](contract-testing.md)
- Read the [architecture guide](../../ARCHITECTURE.md)
- Check out [examples](../../examples/)
