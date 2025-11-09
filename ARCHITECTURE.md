# Apicentric Architecture

Apicentric follows a layered, hexagonal structure that keeps the simulator engine and supporting tooling isolated from I/O concerns. The project is organised so that new adapters or interfaces can be added without rewriting core behaviour.

## High-level layout

```
CLI (src/cli, src/commands)
│
├─ Application context (src/app, src/context)
│  └─ Builds the runtime container and exposes the simulator manager
│
├─ Domain (src/domain)
│  └─ Contract-testing entities and services
│
└─ Adapters & infrastructure
   ├─ Simulator engine (src/simulator)
   ├─ External integrations (src/adapters)
   └─ Persistence (src/storage)
```

### CLI layer

- Command parsing lives in `src/cli/mod.rs` and is powered by `clap`.
- Handler implementations live in `src/commands/`. They coordinate simulator control, contract utilities, the optional TUI, and installer helpers.

### Application layer

- `ContextBuilder` assembles configuration, logging, simulator state, and optional components.
- `ExecutionContext` tracks runtime flags such as `dry_run` and `verbose` to keep command handlers side-effect free when required.

### Domain layer

- Domain modules model contracts, scenarios, and validation rules used by contract testing commands.
- No direct I/O takes place here; the layer exposes traits that are implemented by adapters.

### Adapters & infrastructure

- `src/simulator/` contains the HTTP server, router, request logging, import/export translators, and TypeScript/React generators.
- `src/adapters/` bridges the domain to concrete technologies (HTTP clients, npm integration, UI glue).
- `src/storage/` provides the SQLite-backed registries that persist simulator metadata.

## Feature toggles

Cargo features keep heavyweight subsystems optional:

- `simulator` (default) – core API simulator engine.
- `contract-testing` – HTTP client and assertions for validating contracts.
- `tui` – interactive terminal dashboard.
- `mock-data`, `database`, `file-watch`, `websockets`, `scripting` – opt-in capabilities that pull additional dependencies.

The CLI always compiles the lightweight `share` and `connect` commands, but they currently emit placeholder output because the libp2p collaboration stack is disabled in this build profile.

## Notes on SOLID and hexagonal design

- Commands depend on abstractions exposed by the context rather than concrete simulator types, keeping the CLI open for extension.
- Adapters implement traits defined in the domain so the simulator or contract modules can be exercised in isolation during tests.
- External systems (file system, HTTP, database) live at the edges of the graph, allowing new integrations to be swapped in without touching core logic.
