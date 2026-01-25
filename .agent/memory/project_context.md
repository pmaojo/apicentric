# Project Context

## Current Status

- **Core Simulator**: Functional, supports OpenAPI 3.0 and Swagger 2.0.
- **WebUI**: Active, integrated with backend for service management and log viewing.
- **Digital Twin**: Initial support integrated into `ServiceDefinition`.
- **Code Quality**: Warning-free build maintained.

## Technical Debt & Considerations

- **Parser Simplifications**: `convert_swagger2` simplifies some fields (e.g., parameters, request bodies) compared to `convert_openapi3`. May need expansion for complex Swagger 2.0 specs.
- **Error Mapping**: Conflict mapping (409) implemented for service registration; consider extending this pattern to other simulator errors.
- **P2P Collaboration**: Currently disabled in CLI build to keep it light.

## Recent Fixes

- Fixed 500 Internal Server Error when importing large Swagger 2.0 specs (like Kubernetes).
- Resolved WebUI runtime crashes in `PluginGenerator` and `LogsViewer`.
- Aligned backend and frontend API endpoints for codegen and validation.
