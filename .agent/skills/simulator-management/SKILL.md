# Skill: Simulator Management

description: Capabilities for managing and troubleshooting the API Simulator.

## Instructions

This skill provides guidance on how to interact with the Apicentric Simulator backend and handle common configuration issues.

### Importing Specifications

- Prefer **OpenAPI 3.0** for best compatibility.
- **Swagger 2.0** is supported for legacy specs (e.g., Kubernetes).
- Use `cargo test --test import_export` to verify parser changes.

### Validating Definitions

- Run `cargo run -- simulator validate <file>` to check for schema errors.
- Ensure all paths start with `/`.
- Ensure at least one response (200 OK) is defined per endpoint.

### Common Troubleshooting

- **Conflict (409)**: Occurs when a service with the same name is already registered.
- **Internal Error (500)**: Often points to a panic in the parser or a resource issue. Check backend logs for backtraces.
- **Port Conflict**: The simulator automatically assigns ports in the 8000-8999 range. Ensure this range is available.
