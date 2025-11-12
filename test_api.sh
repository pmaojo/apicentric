#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

API_URL="http://localhost:8080"
TOKEN=""

# Function to print test results
print_result() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓ $2${NC}"
    else
        echo -e "${RED}✗ $2${NC}"
    fi
}

# Function to make API request
api_request() {
    local method=$1
    local endpoint=$2
    local data=$3
    local auth=$4
    
    if [ -n "$auth" ] && [ -n "$TOKEN" ]; then
        curl -s -X "$method" "$API_URL$endpoint" \
            -H "Content-Type: application/json" \
            -H "Authorization: Bearer $TOKEN" \
            -d "$data"
    else
        curl -s -X "$method" "$API_URL$endpoint" \
            -H "Content-Type: application/json" \
            -d "$data"
    fi
}

echo "🚀 Starting API endpoint tests..."
echo ""

# Wait for server to be ready
echo "⏳ Waiting for server to start..."
for i in {1..30}; do
    if curl -s "$API_URL/health" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Server is ready${NC}"
        break
    fi
    sleep 1
done

echo ""
echo "========================================="
echo "1. HEALTH CHECK"
echo "========================================="
response=$(curl -s "$API_URL/health")
echo "$response" | jq '.' 2>/dev/null || echo "$response"
print_result $? "GET /health"

echo ""
echo "========================================="
echo "2. AUTHENTICATION API"
echo "========================================="

# Register a new user
echo "Testing: POST /api/auth/register"
response=$(api_request POST "/api/auth/register" '{"username":"testuser","password":"testpass123"}')
echo "$response" | jq '.' 2>/dev/null || echo "$response"
TOKEN=$(echo "$response" | jq -r '.token' 2>/dev/null)
print_result $? "POST /api/auth/register"

# Login
echo ""
echo "Testing: POST /api/auth/login"
response=$(api_request POST "/api/auth/login" '{"username":"testuser","password":"testpass123"}')
echo "$response" | jq '.' 2>/dev/null || echo "$response"
TOKEN=$(echo "$response" | jq -r '.token' 2>/dev/null)
print_result $? "POST /api/auth/login"

# Get current user
echo ""
echo "Testing: GET /api/auth/me"
response=$(api_request GET "/api/auth/me" "" "auth")
echo "$response" | jq '.' 2>/dev/null || echo "$response"
print_result $? "GET /api/auth/me"

# Refresh token
echo ""
echo "Testing: POST /api/auth/refresh"
response=$(api_request POST "/api/auth/refresh" "" "auth")
echo "$response" | jq '.' 2>/dev/null || echo "$response"
NEW_TOKEN=$(echo "$response" | jq -r '.token' 2>/dev/null)
if [ -n "$NEW_TOKEN" ] && [ "$NEW_TOKEN" != "null" ]; then
    TOKEN="$NEW_TOKEN"
fi
print_result $? "POST /api/auth/refresh"

echo ""
echo "========================================="
echo "3. SERVICE MANAGEMENT API"
echo "========================================="

# List services
echo "Testing: GET /api/services"
response=$(api_request GET "/api/services" "" "auth")
echo "$response" | jq '.' 2>/dev/null || echo "$response"
print_result $? "GET /api/services"

# Create a service
echo ""
echo "Testing: POST /api/services"
SERVICE_YAML='name: test-service
version: "1.0"
description: Test service
server:
  port: 9001
  base_path: /api
endpoints:
  - method: GET
    path: /hello
    responses:
      200:
        content_type: application/json
        body: |
          {"message": "Hello World"}'

response=$(api_request POST "/api/services" "{\"yaml\":$(echo "$SERVICE_YAML" | jq -Rs .)}" "auth")
echo "$response" | jq '.' 2>/dev/null || echo "$response"
print_result $? "POST /api/services"

# Get specific service
echo ""
echo "Testing: GET /api/services/test-service"
response=$(api_request GET "/api/services/test-service" "" "auth")
echo "$response" | jq '.' 2>/dev/null || echo "$response"
print_result $? "GET /api/services/test-service"

# Start service
echo ""
echo "Testing: POST /api/services/test-service/start"
response=$(api_request POST "/api/services/test-service/start" "" "auth")
echo "$response" | jq '.' 2>/dev/null || echo "$response"
print_result $? "POST /api/services/test-service/start"

# Get service status
echo ""
echo "Testing: GET /api/services/test-service/status"
response=$(api_request GET "/api/services/test-service/status" "" "auth")
echo "$response" | jq '.' 2>/dev/null || echo "$response"
print_result $? "GET /api/services/test-service/status"

# Stop service
echo ""
echo "Testing: POST /api/services/test-service/stop"
response=$(api_request POST "/api/services/test-service/stop" "" "auth")
echo "$response" | jq '.' 2>/dev/null || echo "$response"
print_result $? "POST /api/services/test-service/stop"

# Update service
echo ""
echo "Testing: PUT /api/services/test-service"
UPDATED_YAML='name: test-service
version: "1.1"
description: Updated test service
server:
  port: 9001
  base_path: /api
endpoints:
  - method: GET
    path: /hello
    responses:
      200:
        content_type: application/json
        body: |
          {"message": "Hello Updated World"}'

response=$(api_request PUT "/api/services/test-service" "{\"yaml\":$(echo "$UPDATED_YAML" | jq -Rs .)}" "auth")
echo "$response" | jq '.' 2>/dev/null || echo "$response"
print_result $? "PUT /api/services/test-service"

# Reload services
echo ""
echo "Testing: POST /api/services/reload"
response=$(api_request POST "/api/services/reload" "" "auth")
echo "$response" | jq '.' 2>/dev/null || echo "$response"
print_result $? "POST /api/services/reload"

echo ""
echo "========================================="
echo "4. REQUEST LOGS API"
echo "========================================="

# Query logs
echo "Testing: GET /api/logs"
response=$(api_request GET "/api/logs" "" "auth")
echo "$response" | jq '.' 2>/dev/null || echo "$response"
print_result $? "GET /api/logs"

# Export logs
echo ""
echo "Testing: GET /api/logs/export?format=json"
response=$(curl -s "$API_URL/api/logs/export?format=json" -H "Authorization: Bearer $TOKEN")
echo "$response" | jq '.' 2>/dev/null || echo "$response"
print_result $? "GET /api/logs/export"

echo ""
echo "========================================="
echo "5. RECORDING API"
echo "========================================="

# Get recording status
echo "Testing: GET /api/recording/status"
response=$(api_request GET "/api/recording/status" "" "auth")
echo "$response" | jq '.' 2>/dev/null || echo "$response"
print_result $? "GET /api/recording/status"

# Start recording
echo ""
echo "Testing: POST /api/recording/start"
response=$(api_request POST "/api/recording/start" '{"target_url":"https://jsonplaceholder.typicode.com","proxy_port":8888}' "auth")
echo "$response" | jq '.' 2>/dev/null || echo "$response"
print_result $? "POST /api/recording/start"

# Stop recording
echo ""
echo "Testing: POST /api/recording/stop"
response=$(api_request POST "/api/recording/stop" "" "auth")
echo "$response" | jq '.' 2>/dev/null || echo "$response"
print_result $? "POST /api/recording/stop"

echo ""
echo "========================================="
echo "6. AI GENERATION API"
echo "========================================="

# Get AI config
echo "Testing: GET /api/ai/config"
response=$(api_request GET "/api/ai/config" "" "auth")
echo "$response" | jq '.' 2>/dev/null || echo "$response"
print_result $? "GET /api/ai/config"

# Validate YAML
echo ""
echo "Testing: POST /api/ai/validate"
response=$(api_request POST "/api/ai/validate" "{\"yaml\":$(echo "$SERVICE_YAML" | jq -Rs .)}" "auth")
echo "$response" | jq '.' 2>/dev/null || echo "$response"
print_result $? "POST /api/ai/validate"

# AI Generate (may fail if no AI provider configured)
echo ""
echo "Testing: POST /api/ai/generate"
response=$(api_request POST "/api/ai/generate" '{"prompt":"Create a simple user API with GET and POST endpoints"}' "auth")
echo "$response" | jq '.' 2>/dev/null || echo "$response"
echo "(Note: This may fail if no AI provider is configured)"

echo ""
echo "========================================="
echo "7. CODE GENERATION API"
echo "========================================="

# Generate TypeScript
echo "Testing: POST /api/codegen/typescript"
response=$(api_request POST "/api/codegen/typescript" '{"service_name":"test-service"}' "auth")
echo "$response" | jq '.' 2>/dev/null || echo "$response"
print_result $? "POST /api/codegen/typescript"

# Generate React Query
echo ""
echo "Testing: POST /api/codegen/react-query"
response=$(api_request POST "/api/codegen/react-query" '{"service_name":"test-service"}' "auth")
echo "$response" | jq '.' 2>/dev/null || echo "$response"
print_result $? "POST /api/codegen/react-query"

# Generate Axios
echo ""
echo "Testing: POST /api/codegen/axios"
response=$(api_request POST "/api/codegen/axios" '{"service_name":"test-service"}' "auth")
echo "$response" | jq '.' 2>/dev/null || echo "$response"
print_result $? "POST /api/codegen/axios"

echo ""
echo "========================================="
echo "8. CONFIGURATION API"
echo "========================================="

# Get config
echo "Testing: GET /api/config"
response=$(api_request GET "/api/config" "" "auth")
echo "$response" | jq '.' 2>/dev/null || echo "$response"
print_result $? "GET /api/config"

# Validate config
echo ""
echo "Testing: POST /api/config/validate"
response=$(api_request POST "/api/config/validate" '{"config":{"simulator":{"services_dir":"./services"}}}' "auth")
echo "$response" | jq '.' 2>/dev/null || echo "$response"
print_result $? "POST /api/config/validate"

# Update config
echo ""
echo "Testing: PUT /api/config"
response=$(api_request PUT "/api/config" '{"config":{"simulator":{"services_dir":"./services"}}}' "auth")
echo "$response" | jq '.' 2>/dev/null || echo "$response"
print_result $? "PUT /api/config"

echo ""
echo "========================================="
echo "9. CLEANUP"
echo "========================================="

# Delete service
echo "Testing: DELETE /api/services/test-service"
response=$(api_request DELETE "/api/services/test-service" "" "auth")
echo "$response" | jq '.' 2>/dev/null || echo "$response"
print_result $? "DELETE /api/services/test-service"

# Clear logs
echo ""
echo "Testing: DELETE /api/logs"
response=$(api_request DELETE "/api/logs" "" "auth")
echo "$response" | jq '.' 2>/dev/null || echo "$response"
print_result $? "DELETE /api/logs"

# Logout
echo ""
echo "Testing: POST /api/auth/logout"
response=$(api_request POST "/api/auth/logout" "" "auth")
echo "$response" | jq '.' 2>/dev/null || echo "$response"
print_result $? "POST /api/auth/logout"

echo ""
echo "========================================="
echo "✅ API TESTING COMPLETE"
echo "========================================="
