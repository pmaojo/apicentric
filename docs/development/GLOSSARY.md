# Apicentric Glossary

This document defines standard terminology used throughout Apicentric's codebase, documentation, and user interface.

## Core Concepts

### Service
A mock API defined in a YAML file. Each service has endpoints, configurations, and behaviors.
- **Preferred term**: "service"
- **Avoid**: "mock", "API mock", "mock service" (use "service" alone)
- **Example**: "Start a service", "The user-api service"

### Service Definition
The YAML file that describes a service's configuration, endpoints, and behaviors.
- **Preferred term**: "service definition"
- **Avoid**: "service spec", "service config", "definition file"
- **Example**: "Edit the service definition", "Load service definitions from directory"

### Endpoint
A specific HTTP route within a service (method + path + responses).
- **Preferred term**: "endpoint"
- **Avoid**: "route", "path", "API endpoint"
- **Example**: "Add an endpoint", "The GET /users endpoint"

### Simulator
The runtime engine that serves mock services.
- **Preferred term**: "simulator"
- **Avoid**: "mock server", "API server", "runtime"
- **Example**: "Start the simulator", "The simulator is running"

### Contract
A specification that defines expected API behavior for testing.
- **Preferred term**: "contract"
- **Avoid**: "test contract", "API contract"
- **Example**: "Register a contract", "Validate contracts"

### Contract Testing
The process of validating that mock services match real API behavior.
- **Preferred term**: "contract testing"
- **Avoid**: "contract validation", "API testing"
- **Example**: "Run contract testing", "Contract testing failed"

## User Interface Terms

### TUI (Terminal User Interface)
The interactive terminal dashboard for managing services.
- **Preferred term**: "TUI" or "terminal dashboard"
- **Avoid**: "terminal UI", "console interface", "CLI interface"
- **Example**: "Launch the TUI", "Use the terminal dashboard"

### CLI (Command Line Interface)
The command-line interface for executing operations.
- **Preferred term**: "CLI"
- **Avoid**: "command line", "terminal commands"
- **Example**: "Use the CLI", "CLI commands"

## Configuration Terms

### Configuration File
The apicentric.json file that contains global settings.
- **Preferred term**: "configuration file" or "apicentric.json"
- **Avoid**: "config file", "settings file"
- **Example**: "Edit the configuration file", "Check your apicentric.json"

### Feature Flag
A Cargo feature that enables optional functionality.
- **Preferred term**: "feature flag" or "feature"
- **Avoid**: "build flag", "compile flag", "feature option"
- **Example**: "Enable the TUI feature flag", "Build with full features"

## Technical Terms

### Base Path
The URL prefix for all endpoints in a service.
- **Preferred term**: "base path"
- **Avoid**: "base URL", "path prefix", "root path"
- **Example**: "Set the base path to /api", "The service base path"

### Response Template
A Handlebars template used to generate dynamic responses.
- **Preferred term**: "response template" or "template"
- **Avoid**: "template string", "response body template"
- **Example**: "Use a response template", "Template syntax"

### Scenario
A named state that changes service behavior.
- **Preferred term**: "scenario"
- **Avoid**: "state", "mode", "behavior mode"
- **Example**: "Switch to the error scenario", "Define scenarios"

### Fixture
Predefined data used in responses.
- **Preferred term**: "fixture"
- **Avoid**: "test data", "mock data", "sample data"
- **Example**: "Load fixtures", "Define fixture data"

## Action Terms

### Start
Begin running a service or the simulator.
- **Preferred term**: "start"
- **Avoid**: "run", "launch", "boot", "spin up"
- **Example**: "Start the simulator", "Start a service"

### Stop
Cease running a service or the simulator.
- **Preferred term**: "stop"
- **Avoid**: "shutdown", "terminate", "kill", "halt"
- **Example**: "Stop the simulator", "Stop all services"

### Validate
Check a service definition for errors.
- **Preferred term**: "validate"
- **Avoid**: "check", "verify", "lint"
- **Example**: "Validate the service definition", "Validation failed"

### Register
Add a contract to the repository.
- **Preferred term**: "register"
- **Avoid**: "add", "create", "save"
- **Example**: "Register a contract", "Contract registered successfully"

## Status Terms

### Running
A service is actively serving requests.
- **Preferred term**: "running"
- **Avoid**: "active", "started", "up"
- **Example**: "The service is running", "Running services"

### Stopped
A service is not serving requests.
- **Preferred term**: "stopped"
- **Avoid**: "inactive", "down", "offline"
- **Example**: "The service is stopped", "Stopped services"

### Failed
An operation or service encountered an error.
- **Preferred term**: "failed"
- **Avoid**: "errored", "broken", "crashed"
- **Example**: "Validation failed", "Service failed to start"

## Error Message Terminology

When writing error messages, use this consistent format:

```
❌ [Error Type]: [Clear description]
💡 Suggestion: [Actionable advice]
🔍 Field: [Relevant field name]
```

### Error Types
- **Configuration error**: Issues with apicentric.json or service definitions
- **Validation error**: Invalid input or data format
- **Runtime error**: Errors during execution
- **File system error**: File or directory access issues
- **Server error**: HTTP server or network issues
- **Test execution error**: Contract testing failures

## Consistency Guidelines

1. **Capitalization**
   - Capitalize proper nouns: "Apicentric", "TUI", "CLI"
   - Lowercase common terms: "service", "endpoint", "simulator"
   - Capitalize in titles: "Service Definition", "Contract Testing"

2. **Pluralization**
   - Use standard English plurals: "services", "endpoints", "contracts"
   - Avoid irregular forms unless standard: "fixtures" not "fixture data"

3. **Abbreviations**
   - Use standard abbreviations: "API", "HTTP", "JSON", "YAML"
   - Define on first use in documentation: "TUI (Terminal User Interface)"
   - Use consistently once defined

4. **Code vs. Prose**
   - In code: use snake_case for variables, PascalCase for types
   - In prose: use natural language with proper spacing
   - In CLI: use kebab-case for flags and commands

## Examples of Consistent Usage

### Good Examples
- "Start the simulator with service definitions from the services directory"
- "The TUI displays running services and their request logs"
- "Validate your service definition before starting the simulator"
- "Register a contract for contract testing"

### Avoid These
- "Run the mock server with API mocks from the mocks folder"
- "The terminal UI shows active APIs and their logs"
- "Check your service spec before running the server"
- "Add a test contract for API validation"

## Updating This Glossary

When adding new features or concepts:
1. Add the term to the appropriate section
2. Define the preferred term clearly
3. List terms to avoid
4. Provide usage examples
5. Update related documentation to use consistent terminology
