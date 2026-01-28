# Architecture

Apicentric follows **Hexagonal Architecture** (also known as Ports and Adapters) to separate core business logic from external concerns like databases, UI, and network protocols.

## High-Level Overview

The application is structured into concentric layers:

1.  **Domain (Core):** Contains the business rules, entities, and logic. It has no external dependencies.
2.  **Ports (Boundaries):** Interfaces (traits) that define how the domain interacts with the outside world (e.g., `ConfigRepository`, `HttpClient`).
3.  **Adapters (Infrastructure):** Implementations of the ports (e.g., `ConfigFileLoader`, `ReqwestClient`).
4.  **Application (App):** Orchestrates the flow of data between adapters and the domain.

<img width="2752" height="1536" alt="Architecture Diagram" src="https://github.com/user-attachments/assets/52ef5f9c-149d-45cf-ab56-3fc8d4345f55" />

## Directory Structure

The `src/` directory reflects this architecture:

- **`domain/`**: Core business logic and types.
- **`adapters/`**: Infrastructure implementations (HTTP clients, file loaders).
- **`simulator/`**: The core API simulation engine.
- **`config/`**: Configuration management.
- **`cli/`** & **`cli_ui/`**: Command-line interface adapters.
- **`cloud/`**: Cloud synchronization logic.
- **`iot/`**: Digital Twin and IoT protocols.

## Key Modules

- **`app`**: Application bootstrap and command execution.
- **`context`**: Dependency injection container (`Context`) and execution context.
- **`errors`**: Domain-specific error types (`ApicentricError`).
- **`validation`**: Cross-cutting validation utilities.
- **`storage`**: Persistence adapters.

## Design Principles

- **Dependency Rule**: Dependencies point *inward*. The domain knows nothing about the CLI or the database.
- **Explicit Dependencies**: Use the `Context` to inject dependencies rather than global state.
- **Rich Domain Models**: Encapsulate logic within domain entities rather than anemic data structures.
