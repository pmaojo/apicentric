# Pulse Development Plan

## Completed

- Initial project setup with Cargo
- Added necessary dependencies (serde, serde_yaml, actix-web, tokio)
- Created mocks.yaml with example API definitions
- Created basic module structure (src/config, src/server)
- Implemented YAML configuration loading in src/config/mod.rs
- Implement a basic actix-web server in src/server/mod.rs
- Updated src/main.rs to load config and run the server

## To Do

- Implement request handling in the server to match incoming requests against loaded mock configurations.
- Implement dynamic response generation based on the mock configuration (status, headers, body).
- Add routing logic to handle different HTTP methods and paths.
- Implement path parameter extraction (e.g., `{id}` from `/users/{id}`).
- Implement basic request body parsing for POST/PUT/PATCH requests.
- Add support for different response body types (e.g., JSON, plain text).
- Implement error handling for cases where no mock matches a request.
- Plan and implement the authentication flow mocking.
- Plan and implement the validation of mock responses.
- Plan and implement the selective Cypress test execution based on file changes.
- Add command-line argument parsing for configuration file path, server address, etc.
- Add logging.
- Write unit and integration tests.