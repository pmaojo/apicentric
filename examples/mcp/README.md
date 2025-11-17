# MCP Integration Examples

This directory contains examples for integrating Apicentric with AI assistants using the Model Context Protocol (MCP).

## 🤖 Claude Desktop Integration

### 1. Configuration

Create or edit `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "apicentric": {
      "command": "apicentric",
      "args": ["mcp"]
    }
  }
}
```

### 2. Usage Examples

Once configured, you can ask Claude to create and manage API services:

**Example 1: Create a simple API**
```
"Create a mock API for a blog with posts and comments endpoints"
```

**Example 2: Authentication service**
```
"Build a user authentication API with login, register, and profile endpoints"
```

**Example 3: E-commerce API**
```
"Create an e-commerce API with products, cart, and checkout functionality"
```

**Example 4: Monitor services**
```
"Show me the status of all running mock services and their logs"
```

## 🧠 ChatGPT Integration

### Using GPT with MCP (via custom integration)

While ChatGPT doesn't have native MCP support yet, you can use the MCP tools manually or create custom integrations.

### Example Workflow

1. **Start the MCP server:**
   ```bash
   apicentric mcp
   ```

2. **Use the tools programmatically:**
   ```bash
   # List services
   echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"list_services","arguments":{}}}' | apicentric mcp

   # Create a service
   echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"create_service","arguments":{"yaml_definition":"name: test\nport: 9000\nendpoints:\n  - method: GET\n    path: /hello\n    response:\n      status: 200\n      body: \"Hello World\""}}}' | apicentric mcp
   ```

## 🔧 VS Code Integration

### Configuration

Create `.vscode/mcp.json` in your workspace:

```json
{
  "servers": {
    "apicentric": {
      "type": "stdio",
      "command": "apicentric",
      "args": ["mcp"]
    }
  }
}
```

### Usage with GitHub Copilot

When using GitHub Copilot in VS Code with MCP configured, you can:

1. Ask Copilot to create API services
2. Have it start/stop services automatically
3. Generate test data and endpoints
4. Monitor service health

## 📋 Example Conversations

### Creating a User Management API

**User:** "Create a user management API with CRUD operations"

**AI Response:**
1. Uses `create_service` to generate YAML with users, GET/POST/PUT/DELETE endpoints
2. Uses `start_service` to launch on port 9001
3. Provides curl examples for testing
4. Shows how to access logs with `get_service_logs`

### Building a Payment Service

**User:** "Build a payment processing API that handles charges and refunds"

**AI Response:**
1. Creates service with payment endpoints
2. Includes proper HTTP status codes (200, 400, 402, etc.)
3. Adds request validation
4. Generates realistic payment data

### Monitoring and Debugging

**User:** "Check if my API services are running and show me recent logs"

**AI Response:**
1. Uses `list_services` to show all services
2. Uses `get_service_logs` for each service
3. Identifies any issues or errors
4. Suggests fixes if needed

## 🛠️ Advanced MCP Usage

### Custom Service Templates

You can create templates for common API patterns:

```yaml
# Template for REST API
name: {{service_name}}
port: {{port}}
endpoints:
  - method: GET
    path: /{{resource}}
    response:
      status: 200
      body: |
        {
          "{{resource}}": {{fixtures.data}}
        }
```

### Integration with CI/CD

Use MCP in automated workflows:

```bash
# Start services for testing
apicentric mcp &
# Run integration tests
npm test
# Stop services
pkill apicentric
```

### Multi-Service Orchestration

Create complex microservice architectures:

```bash
# Start user service
# Start product service
# Start order service
# Configure cross-service communication
```

## 🔍 Troubleshooting

### MCP Server Won't Start

**Issue:** `apicentric mcp` fails to start

**Solutions:**
- Ensure you have MCP features: `cargo install apicentric --features mcp`
- Check if port 9001+ are available
- Verify service directory exists

### AI Assistant Can't Connect

**Issue:** Claude/GPT can't use MCP tools

**Solutions:**
- Verify MCP server is running
- Check configuration file syntax
- Restart the AI application
- Check logs: `apicentric mcp 2>&1 | tee mcp.log`

### Tools Return Errors

**Issue:** MCP tools fail with errors

**Solutions:**
- Check service YAML syntax
- Ensure ports are not in use
- Verify file permissions
- Check apicentric version: `apicentric --version`

## 📚 Additional Resources

- [MCP Specification](https://modelcontextprotocol.io/specification)
- [Claude Desktop Documentation](https://docs.anthropic.com/claude/docs/desktop)
- [Apicentric Documentation](https://github.com/pmaojo/apicentric)