# Pulse

Pulse is a Rust-powered toolkit for end-to-end testing and API simulation.

## Features

- **Smart test runner** for Cypress
  - run all specs or only those impacted by file changes
  - parallel execution with retries
  - optional metrics output (Prometheus, Sentry, Allure)
- **YAML-driven API simulator**
  - start and stop mock services from service definitions
  - validate service files and switch scenarios
  - import or export OpenAPI specifications
  - generate TypeScript interfaces
  - convert existing Mockoon JSON definitions
- **Desktop GUI** built with Tauri for editing and managing mock services
- **NPM integration** via `pulse setup-npm`
- Experimental foundation for **contract testing** against real APIs

## Quick start

```bash
# initialise default configuration
pulse init

# add npm scripts to package.json
pulse setup-npm

# run all tests once
npm run pulse -- run

# watch for file changes and run impacted tests
npm run pulse -- watch
```

## API Simulator examples

```bash
# validate YAML service definitions
pulse simulator validate --path services

# start simulator (Ctrl+C to stop)
pulse simulator start --services-dir services

# import an OpenAPI spec to a service YAML
pulse simulator import --input openapi.yaml --output services/petstore.yaml

# export a service definition to OpenAPI
pulse simulator export --input services/petstore.yaml --output openapi.yaml

# generate TypeScript types from a service
pulse simulator export-types --input services/petstore.yaml --output types.ts

# convert a Mockoon JSON file
pulse simulator import-mockoon --input mockoon.json --output services/mockoon.yaml
```

## GUI

Launch the desktop editor for mock services:

```bash
pulse gui
```

## Development

```bash
cargo build
cargo test
```

## License

MIT

