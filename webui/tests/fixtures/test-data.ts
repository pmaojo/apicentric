/**
 * Test fixtures - Sample data for E2E tests
 */

export const SAMPLE_SERVICE_YAML = `name: test-service
version: 1.0.0
description: A test service for E2E testing
server:
  port: 3001
  base_path: /api/v1
endpoints:
  - method: GET
    path: /users
    description: Get all users
    responses:
      - status: 200
        content_type: application/json
        body: |
          [
            {"id": 1, "name": "John Doe", "email": "john@example.com"},
            {"id": 2, "name": "Jane Smith", "email": "jane@example.com"}
          ]
  - method: GET
    path: /users/{id}
    description: Get user by ID
    responses:
      - status: 200
        content_type: application/json
        body: |
          {"id": 1, "name": "John Doe", "email": "john@example.com"}
      - status: 404
        content_type: application/json
        body: |
          {"error": "User not found"}
  - method: POST
    path: /users
    description: Create a new user
    responses:
      - status: 201
        content_type: application/json
        body: |
          {"id": 3, "name": "New User", "email": "new@example.com"}
  - method: PUT
    path: /users/{id}
    description: Update user
    responses:
      - status: 200
        content_type: application/json
        body: |
          {"id": 1, "name": "Updated User", "email": "updated@example.com"}
  - method: DELETE
    path: /users/{id}
    description: Delete user
    responses:
      - status: 204
        content_type: application/json
        body: ""`;

export const SAMPLE_ECOMMERCE_SERVICE_YAML = `name: ecommerce-api
version: 1.0.0
description: Sample e-commerce API for testing
server:
  port: 3002
  base_path: /api/v1
endpoints:
  - method: GET
    path: /products
    description: Get all products
    responses:
      - status: 200
        content_type: application/json
        body: |
          [
            {"id": 1, "name": "Laptop", "price": 999.99, "category": "Electronics"},
            {"id": 2, "name": "Book", "price": 19.99, "category": "Education"}
          ]
  - method: GET
    path: /products/{id}
    description: Get product by ID
    responses:
      - status: 200
        content_type: application/json
        body: |
          {"id": 1, "name": "Laptop", "price": 999.99, "category": "Electronics"}
  - method: POST
    path: /orders
    description: Create a new order
    responses:
      - status: 201
        content_type: application/json
        body: |
          {"id": 1, "status": "pending", "total": 999.99, "items": [{"product_id": 1, "quantity": 1}]}
  - method: GET
    path: /orders/{id}
    description: Get order by ID
    responses:
      - status: 200
        content_type: application/json
        body: |
          {"id": 1, "status": "completed", "total": 999.99, "items": [{"product_id": 1, "quantity": 1}]}`;

export const SAMPLE_GRAPHQL_SERVICE_YAML = `name: graphql-service
version: 1.0.0
description: Sample GraphQL service
server:
  port: 3003
  base_path: /graphql
type: graphql
endpoints:
  - method: POST
    path: /
    description: GraphQL endpoint
    responses:
      - status: 200
        content_type: application/json
        body: |
          {
            "data": {
              "users": [
                {"id": "1", "name": "John Doe", "email": "john@example.com"}
              ]
            }
          }`;

export const AI_PROMPTS = {
  simple: "Create a simple REST API for managing books with CRUD operations",
  complex: "Create a complete e-commerce API with products, orders, customers, and payments. Include authentication endpoints.",
  graphql: "Create a GraphQL API for a social media platform with users, posts, and comments"
};

export const RECORDING_TARGETS = {
  jsonPlaceholder: "https://jsonplaceholder.typicode.com",
  httpbin: "https://httpbin.org",
  localhost: "http://localhost:3000"
};

export const TEST_SCENARIOS = {
  basicCrud: {
    name: "basic-crud-test",
    yaml: SAMPLE_SERVICE_YAML
  },
  ecommerce: {
    name: "ecommerce-test",
    yaml: SAMPLE_ECOMMERCE_SERVICE_YAML
  },
  graphql: {
    name: "graphql-test",
    yaml: SAMPLE_GRAPHQL_SERVICE_YAML
  }
};

export const EXPECTED_UI_ELEMENTS = {
  sidebar: [
    'dashboard',
    'services', 
    'logs',
    'ai-generator',
    'code-generator',
    'recording',
    'configuration'
  ],
  views: [
    'Dashboard',
    'Service Definitions',
    'Simulator Logs',
    'AI Service Generator',
    'Client Code Generator',
    'Recording',
    'Configuration'
  ]
};