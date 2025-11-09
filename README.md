# Apicentric

Apicentric is a Rust command-line application that helps teams design and exercise HTTP APIs without relying on a live backend. The binary bundles a local API simulator, utilities for importing and exporting service definitions, and code generators that keep client stubs in sync with the mock services.

## Status

The codebase is actively developed. All simulator, import/export, and code generation commands compile and run from the CLI. The `share` and `connect` subcommands are present but intentionally ship as no-ops that print a placeholder message while the heavier peer-to-peer stack remains disabled for lighter builds.

## Confirmed capabilities

The following functionality is implemented in the current tree:

- **Service-driven simulator** – Service definitions written in YAML are loaded and orchestrated by the `ApiSimulatorManager`, which starts and stops HTTP mocks, tracks status, and records traffic for later replay.
- **Contract tooling** – The CLI routes `simulator` subcommands such as `start`, `stop`, `validate`, `status`, and `record` through concrete handlers that operate on the simulator state.
- **Specification import/export** – OpenAPI, Mockoon, WireMock, and Postman collections can be converted into Apicentric YAML, and services can be exported back to OpenAPI, Postman collections, TypeScript types, React Query hooks, and React views.
- **Code scaffolding** – New services and endpoints can be scaffolded from the CLI to accelerate setup, and TypeScript assets are rendered from the live simulator configuration.
- **Peer-to-peer placeholders** – The `share` and `connect` commands compile but only emit an informational message until the libp2p layer is re-enabled.

## Installation

### Prerequisites

- Rust toolchain (tested with `rustc 1.89.0`).
- `cargo` for building from source.

### Build from source

```bash
# Clone the repository
git clone https://github.com/pmaojo/apicentric.git
cd apicentric

# Compile the CLI with all default features
cargo build

# Optionally install it into your PATH
cargo install --path .
```

If you need to slim the binary you can disable default features and opt into only what you need, for example:

```bash
cargo build --no-default-features --features simulator
```

## Running the simulator

Create a service definition (for example `services/hello.yaml`) and start the simulator:

```bash
cat > services/hello.yaml <<'YAML'
name: hello
server:
  port: 9000
  base_path: /
endpoints:
  - method: GET
    path: /greet
    responses:
      200:
        content_type: application/json
        body: '{"message": "hi"}'
YAML

cargo run -- simulator start --services-dir services
```

While the simulator runs you can:

- Check status: `cargo run -- simulator status --detailed`
- Validate definitions: `cargo run -- simulator validate --path services`
- Export OpenAPI: `cargo run -- simulator export --input services/hello.yaml --output hello.yaml`

## Tests

The repository includes unit and integration tests that exercise the CLI handlers and simulator modules. Run them with:

```bash
cargo test
```

The current default feature set compiles successfully after resolving warnings from optional modules.

## Repository layout

- `src/commands/` – CLI command handlers (`simulator`, `contract`, `tui`).
- `src/simulator/` – HTTP simulator engine, configuration, templates, and code generators.
- `src/domain/` – Contract testing domain models.
- `src/storage/` – SQLite-backed persistence and registries.
- `services/` – Example service definitions loaded by the simulator (create your own during development).

## Contributing

Open pull requests with reproducible steps and test output. Discussions, bug reports, and feature requests are welcome through GitHub issues.
