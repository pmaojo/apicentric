#!/usr/bin/env python3
"""
Comprehensive API endpoint tester for Apicentric Cloud Server
"""

import subprocess
import time
import requests
import json
import sys
from typing import Optional

API_URL = "http://localhost:8080"
TOKEN: Optional[str] = None

class Colors:
    GREEN = '\033[0;32m'
    RED = '\033[0;31m'
    YELLOW = '\033[1;33m'
    BLUE = '\033[0;34m'
    NC = '\033[0m'

def print_result(success: bool, message: str):
    if success:
        print(f"{Colors.GREEN}✓ {message}{Colors.NC}")
    else:
        print(f"{Colors.RED}✗ {message}{Colors.NC}")

def api_request(method: str, endpoint: str, data: dict = None, auth: bool = False):
    headers = {"Content-Type": "application/json"}
    if auth and TOKEN:
        headers["Authorization"] = f"Bearer {TOKEN}"
    
    url = f"{API_URL}{endpoint}"
    try:
        if method == "GET":
            response = requests.get(url, headers=headers)
        elif method == "POST":
            response = requests.post(url, json=data, headers=headers)
        elif method == "PUT":
            response = requests.put(url, json=data, headers=headers)
        elif method == "DELETE":
            response = requests.delete(url, headers=headers)
        else:
            raise ValueError(f"Unsupported method: {method}")
        
        return response
    except Exception as e:
        print(f"{Colors.RED}Error: {e}{Colors.NC}")
        return None

def wait_for_server(timeout=30):
    print(f"{Colors.YELLOW}⏳ Waiting for server to start...{Colors.NC}")
    for i in range(timeout):
        try:
            response = requests.get(f"{API_URL}/health", timeout=1)
            if response.status_code == 200:
                print(f"{Colors.GREEN}✓ Server is ready{Colors.NC}")
                return True
        except:
            pass
        time.sleep(1)
    print(f"{Colors.RED}✗ Server failed to start within {timeout} seconds{Colors.NC}")
    return False

def test_health():
    print("\n" + "="*50)
    print("1. HEALTH CHECK")
    print("="*50)
    
    response = api_request("GET", "/health")
    if response and response.status_code == 200:
        print(json.dumps(response.json(), indent=2))
        print_result(True, "GET /health")
        return True
    else:
        print_result(False, "GET /health")
        return False

def test_authentication():
    global TOKEN
    print("\n" + "="*50)
    print("2. AUTHENTICATION API")
    print("="*50)
    
    # Register
    print("\nTesting: POST /api/auth/register")
    response = api_request("POST", "/api/auth/register", {
        "username": "testuser",
        "password": "testpass123"
    })
    if response and response.status_code in [200, 201]:
        data = response.json()
        print(json.dumps(data, indent=2))
        TOKEN = data.get("token")
        print_result(True, "POST /api/auth/register")
    else:
        print(f"Status: {response.status_code if response else 'N/A'}")
        print(response.text if response else "No response")
        print_result(False, "POST /api/auth/register")
    
    # Login
    print("\nTesting: POST /api/auth/login")
    response = api_request("POST", "/api/auth/login", {
        "username": "testuser",
        "password": "testpass123"
    })
    if response and response.status_code == 200:
        data = response.json()
        print(json.dumps(data, indent=2))
        TOKEN = data.get("token")
        print_result(True, "POST /api/auth/login")
    else:
        print(f"Status: {response.status_code if response else 'N/A'}")
        print_result(False, "POST /api/auth/login")
    
    # Get current user
    print("\nTesting: GET /api/auth/me")
    response = api_request("GET", "/api/auth/me", auth=True)
    if response and response.status_code == 200:
        print(json.dumps(response.json(), indent=2))
        print_result(True, "GET /api/auth/me")
    else:
        print(f"Status: {response.status_code if response else 'N/A'}")
        print_result(False, "GET /api/auth/me")
    
    # Refresh token
    print("\nTesting: POST /api/auth/refresh")
    response = api_request("POST", "/api/auth/refresh", auth=True)
    if response and response.status_code == 200:
        data = response.json()
        print(json.dumps(data, indent=2))
        new_token = data.get("token")
        if new_token:
            TOKEN = new_token
        print_result(True, "POST /api/auth/refresh")
    else:
        print(f"Status: {response.status_code if response else 'N/A'}")
        print_result(False, "POST /api/auth/refresh")

def test_service_management():
    print("\n" + "="*50)
    print("3. SERVICE MANAGEMENT API")
    print("="*50)
    
    # List services
    print("\nTesting: GET /api/services")
    response = api_request("GET", "/api/services", auth=True)
    if response and response.status_code == 200:
        print(json.dumps(response.json(), indent=2))
        print_result(True, "GET /api/services")
    else:
        print(f"Status: {response.status_code if response else 'N/A'}")
        print_result(False, "GET /api/services")
    
    # Create service
    print("\nTesting: POST /api/services")
    service_yaml = """name: test-service
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
          {"message": "Hello World"}"""
    
    response = api_request("POST", "/api/services", {"yaml": service_yaml}, auth=True)
    if response and response.status_code in [200, 201]:
        print(json.dumps(response.json(), indent=2))
        print_result(True, "POST /api/services")
    else:
        print(f"Status: {response.status_code if response else 'N/A'}")
        if response:
            print(response.text)
        print_result(False, "POST /api/services")
    
    # Get specific service
    print("\nTesting: GET /api/services/test-service")
    response = api_request("GET", "/api/services/test-service", auth=True)
    if response and response.status_code == 200:
        print(json.dumps(response.json(), indent=2))
        print_result(True, "GET /api/services/test-service")
    else:
        print(f"Status: {response.status_code if response else 'N/A'}")
        print_result(False, "GET /api/services/test-service")
    
    # Start service
    print("\nTesting: POST /api/services/test-service/start")
    response = api_request("POST", "/api/services/test-service/start", auth=True)
    if response and response.status_code == 200:
        print(json.dumps(response.json(), indent=2))
        print_result(True, "POST /api/services/test-service/start")
    else:
        print(f"Status: {response.status_code if response else 'N/A'}")
        print_result(False, "POST /api/services/test-service/start")
    
    time.sleep(1)  # Give service time to start
    
    # Get service status
    print("\nTesting: GET /api/services/test-service/status")
    response = api_request("GET", "/api/services/test-service/status", auth=True)
    if response and response.status_code == 200:
        print(json.dumps(response.json(), indent=2))
        print_result(True, "GET /api/services/test-service/status")
    else:
        print(f"Status: {response.status_code if response else 'N/A'}")
        print_result(False, "GET /api/services/test-service/status")
    
    # Stop service
    print("\nTesting: POST /api/services/test-service/stop")
    response = api_request("POST", "/api/services/test-service/stop", auth=True)
    if response and response.status_code == 200:
        print(json.dumps(response.json(), indent=2))
        print_result(True, "POST /api/services/test-service/stop")
    else:
        print(f"Status: {response.status_code if response else 'N/A'}")
        print_result(False, "POST /api/services/test-service/stop")

def test_logs():
    print("\n" + "="*50)
    print("4. REQUEST LOGS API")
    print("="*50)
    
    # Query logs
    print("\nTesting: GET /api/logs")
    response = api_request("GET", "/api/logs", auth=True)
    if response and response.status_code == 200:
        print(json.dumps(response.json(), indent=2))
        print_result(True, "GET /api/logs")
    else:
        print(f"Status: {response.status_code if response else 'N/A'}")
        print_result(False, "GET /api/logs")

def test_recording():
    print("\n" + "="*50)
    print("5. RECORDING API")
    print("="*50)
    
    # Get recording status
    print("\nTesting: GET /api/recording/status")
    response = api_request("GET", "/api/recording/status", auth=True)
    if response and response.status_code == 200:
        print(json.dumps(response.json(), indent=2))
        print_result(True, "GET /api/recording/status")
    else:
        print(f"Status: {response.status_code if response else 'N/A'}")
        print_result(False, "GET /api/recording/status")

def test_ai():
    print("\n" + "="*50)
    print("6. AI GENERATION API")
    print("="*50)
    
    # Get AI config
    print("\nTesting: GET /api/ai/config")
    response = api_request("GET", "/api/ai/config", auth=True)
    if response and response.status_code == 200:
        print(json.dumps(response.json(), indent=2))
        print_result(True, "GET /api/ai/config")
    else:
        print(f"Status: {response.status_code if response else 'N/A'}")
        print_result(False, "GET /api/ai/config")

def test_codegen():
    print("\n" + "="*50)
    print("7. CODE GENERATION API")
    print("="*50)
    
    # Generate TypeScript
    print("\nTesting: POST /api/codegen/typescript")
    response = api_request("POST", "/api/codegen/typescript", 
                          {"service_name": "test-service"}, auth=True)
    if response and response.status_code == 200:
        data = response.json()
        if "data" in data and "code" in data["data"]:
            print(f"Generated {len(data['data']['code'])} characters of TypeScript code")
        print_result(True, "POST /api/codegen/typescript")
    else:
        print(f"Status: {response.status_code if response else 'N/A'}")
        print_result(False, "POST /api/codegen/typescript")

def test_config():
    print("\n" + "="*50)
    print("8. CONFIGURATION API")
    print("="*50)
    
    # Get config
    print("\nTesting: GET /api/config")
    response = api_request("GET", "/api/config", auth=True)
    if response and response.status_code == 200:
        print(json.dumps(response.json(), indent=2))
        print_result(True, "GET /api/config")
    else:
        print(f"Status: {response.status_code if response else 'N/A'}")
        print_result(False, "GET /api/config")

def test_cleanup():
    print("\n" + "="*50)
    print("9. CLEANUP")
    print("="*50)
    
    # Delete service
    print("\nTesting: DELETE /api/services/test-service")
    response = api_request("DELETE", "/api/services/test-service", auth=True)
    if response and response.status_code == 200:
        print(json.dumps(response.json(), indent=2))
        print_result(True, "DELETE /api/services/test-service")
    else:
        print(f"Status: {response.status_code if response else 'N/A'}")
        print_result(False, "DELETE /api/services/test-service")
    
    # Logout
    print("\nTesting: POST /api/auth/logout")
    response = api_request("POST", "/api/auth/logout", auth=True)
    if response and response.status_code == 200:
        print(json.dumps(response.json(), indent=2))
        print_result(True, "POST /api/auth/logout")
    else:
        print(f"Status: {response.status_code if response else 'N/A'}")
        print_result(False, "POST /api/auth/logout")

def main():
    print(f"{Colors.BLUE}🚀 Starting API endpoint tests...{Colors.NC}")
    
    # Wait for server
    if not wait_for_server():
        print(f"{Colors.RED}Server is not running. Please start it first.{Colors.NC}")
        sys.exit(1)
    
    # Run all tests
    test_health()
    test_authentication()
    test_service_management()
    test_logs()
    test_recording()
    test_ai()
    test_codegen()
    test_config()
    test_cleanup()
    
    print("\n" + "="*50)
    print(f"{Colors.GREEN}✅ API TESTING COMPLETE{Colors.NC}")
    print("="*50)

if __name__ == "__main__":
    main()
