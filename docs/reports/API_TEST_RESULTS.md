# Apicentric Cloud API - Test Results

## Server Status
✅ Server running on `http://localhost:8080`

## Endpoints Tested

### 1. Health Check
- **GET** `/health` ✅
- Returns server status, version, and uptime

### 2. Authentication API
- **POST** `/api/auth/register` ✅
  - Creates new user account
  - Returns JWT token
  
- **POST** `/api/auth/login` ✅
  - Authenticates user
  - Returns JWT token
  
- **GET** `/api/auth/me` ✅
  - Returns current user info
  - Requires authentication
  
- **POST** `/api/auth/refresh` ✅
  - Refreshes JWT token
  - Requires authentication
  
- **POST** `/api/auth/logout` ✅
  - Logs out user
  - Blacklists token

### 3. Service Management API
- **GET** `/api/services` ✅
  - Lists all services
  - Returns empty array when no services exist
  
- **POST** `/api/services` ✅
  - Creates new service from YAML
  - Automatically starts the service
  
- **GET** `/api/services/{name}` ✅
  - Gets service details and YAML definition
  
- **PUT** `/api/services/{name}` ✅
  - Updates service definition
  
- **DELETE** `/api/services/{name}` ✅
  - Deletes service
  
- **POST** `/api/services/{name}/start` ✅
  - Starts a service
  - Returns error if already running
  
- **POST** `/api/services/{name}/stop` ✅
  - Stops a running service
  
- **GET** `/api/services/{name}/status` ✅
  - Gets service status (running, port, endpoint count)
  
- **POST** `/api/services/reload` ✅
  - Reloads all service definitions

### 4. Request Logs API
- **GET** `/api/logs` ✅
  - Queries request logs
  - Supports filtering by service, method, status
  - Returns historical logs from database
  
- **GET** `/api/logs/export` ✅
  - Exports logs in JSON/CSV format
  
- **DELETE** `/api/logs` ✅
  - Clears all logs

### 5. Recording API
- **GET** `/api/recording/status` ✅
  - Gets current recording session status
  - Returns inactive when no recording in progress
  
- **POST** `/api/recording/start` ✅
  - Starts proxy recording mode
  - Captures requests to target URL
  
- **POST** `/api/recording/stop` ✅
  - Stops recording
  - Returns captured requests
  
- **POST** `/api/recording/generate` ✅
  - Generates service definition from recorded requests

### 6. AI Generation API
- **GET** `/api/ai/config` ✅
  - Gets AI provider configuration
  - Shows configured provider (Gemini in this case)
  
- **POST** `/api/ai/generate` ✅
  - Generates service definition from natural language prompt
  - Requires AI provider to be configured
  
- **POST** `/api/ai/validate` ✅
  - Validates YAML service definition

### 7. Code Generation API
- **POST** `/api/codegen/typescript` ✅
  - Generates TypeScript types from service definition
  - Uses OpenAPI format
  
- **POST** `/api/codegen/react-query` ✅
  - Generates React Query hooks
  - Creates useQuery hooks for each endpoint
  
- **POST** `/api/codegen/axios` ✅
  - Generates Axios client class
  - Creates methods for each endpoint

### 8. Configuration API
- **GET** `/api/config` ✅
  - Gets current configuration
  - Includes simulator, AI, metrics settings
  
- **PUT** `/api/config` ✅
  - Updates configuration
  
- **POST** `/api/config/validate` ✅
  - Validates configuration before applying

### 9. WebSocket API
- **GET** `/ws` ✅
  - WebSocket endpoint for real-time updates
  - Broadcasts service status changes
  - Streams request logs
  - Sends recording capture events

## Test Summary

**Total Endpoints Tested:** 30+
**Success Rate:** 100%
**Authentication:** JWT-based with token refresh
**Authorization:** Optional (can be enabled with `APICENTRIC_PROTECT_SERVICES=true`)

## Example Service Test

Created a test service that:
- Runs on port 9001
- Has a `/api/hello` endpoint
- Returns `{"message": "Hello World"}`
- Successfully started, queried, stopped, and deleted

## Notes

1. **Authentication is optional by default** - Set `APICENTRIC_PROTECT_SERVICES=true` to require auth for all endpoints
2. **Services auto-start** when created via API
3. **Logs are persisted** in SQLite database
4. **AI provider configured** - Gemini 2.5 Flash is active
5. **Code generation works** for TypeScript, React Query, and Axios
6. **WebSocket support** for real-time updates

## How to Run

```bash
# Build the server
cargo build --release --example cloud_server --features cli-tools

# Start the server
./target/release/examples/cloud_server

# Test endpoints
curl http://localhost:8080/health
```

## Environment Variables

- `APICENTRIC_PROTECT_SERVICES` - Enable authentication requirement (default: false)
- `APICENTRIC_JWT_SECRET` - JWT signing secret (default: "dev-secret-change-me")
- `APICENTRIC_AUTH_DB` - Auth database path (default: "data/auth.db")
