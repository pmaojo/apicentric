# Feature Flags Guide

Apicentric uses Cargo feature flags to make heavy dependencies optional, allowing you to install only what you need. This reduces build times and binary size.

## Available Features

### Core Features

#### `simulator` (included in default)

The API simulator is the core feature of Apicentric.

**Includes**:
- YAML-based service definitions
- HTTP server for mocking APIs
- Template rendering with Handlebars
- Request/response logging
- Service lifecycle management
- Importers for OpenAPI, Mockoon, Postman/Insomnia, and WireMock stub mappings

**Dependencies**: Minimal (tokio, serde, hyper)

**Use when**: You want to mock APIs locally

##### WireMock importer

Use the simulator CLI to convert WireMock mapping exports into Apicentric YAML services:

```bash
apicentric simulator import-wiremock --input ./mappings.json --output ./services/payments.yaml
```

The importer understands single stub files or `mappings` arrays produced by `__admin/mappings` exports. It maps `responses` arrays to sequential scenarios and preserves simple `equalTo` header matchers and `equalToJson` body patterns for object payloads.

**Limitations**:

- Regex URL matchers (`urlPattern`, `urlPathPattern`) are imported verbatim and may require manual cleanup.
- Only `equalTo` header matchers and object `equalToJson` body patterns are supported; other pattern types such as `matchesJsonPath`, `binaryEqualTo`, or plain string bodies are ignored.
- Advanced WireMock features including transformers, proxying, post-serve actions, scenario state transitions, and body files are not imported automatically.

#### `contract-testing`

Contract testing validates that mocks match real APIs.

**Includes**:
- Contract registration and management
- Mock vs real API comparison
- HTML report generation
- Scenario testing

**Dependencies**: reqwest, async-trait

**Use when**: You need to verify API contracts

### Advanced Features

#### `tui`

Interactive terminal user interface for managing services.

**Includes**:
- Real-time service dashboard
- Live request logs with filtering
- Keyboard-driven navigation
- Service start/stop controls

**Dependencies**: ratatui, crossterm, indicatif, console, colored, inquire

**Use when**: You want visual service management in the terminal

#### `p2p`

Peer-to-peer collaboration for sharing services.

**Includes**:
- libp2p networking
- Service sharing between developers
- CRDT-based synchronization
- WebSocket support

**Dependencies**: libp2p, automerge, tokio-tungstenite

**Use when**: You need to share services with team members

#### `graphql`

GraphQL API mocking support.

**Includes**:
- GraphQL schema parsing
- Query/mutation mocking
- SDL introspection

**Dependencies**: async-graphql, async-graphql-parser

**Use when**: You need to mock GraphQL APIs

#### `scripting`

JavaScript runtime for custom logic.

**Includes**:
- Deno runtime integration
- JavaScript/TypeScript execution
- Sandboxed environment
- Custom response logic

**Dependencies**: deno_core

**Use when**: You need dynamic response generation

#### `ai`

AI-powered service generation (experimental).

**Includes**:
- Natural language to YAML conversion
- Service generation from descriptions
- OpenAI integration

**Dependencies**: Minimal

**Use when**: You want to generate services from descriptions

### Convenience Bundles

#### `minimal`

Fastest build, smallest binary.

**Includes**: Only the simulator

**Build time**: < 1 minute

**Use when**: You only need basic API mocking

#### `default`

Balanced feature set.

**Includes**: simulator

**Build time**: < 1.5 minutes

**Use when**: You want the standard experience

#### `cli-tools` (recommended)

Essential CLI tools for developers.

**Includes**: simulator, contract-testing, tui

**Build time**: < 2 minutes

**Use when**: You want a complete CLI experience

#### `full`

All features enabled.

**Includes**: All features (tui, p2p, graphql, scripting, ai)

**Build time**: 3-5 minutes

**Use when**: You need everything

## Installation Examples

### Minimal Installation

Fastest build, smallest binary:

```bash
cargo install apicentric --no-default-features --features minimal
```

**What you get**:
- API simulator
- YAML service definitions
- Basic HTTP mocking

**What you don't get**:
- TUI
- Contract testing
- P2P sharing
- GraphQL support
- Scripting

### Recommended Installation

Best balance of features and build time:

```bash
cargo install apicentric --features cli-tools
```

**What you get**:
- API simulator
- Contract testing
- Terminal UI
- All essential CLI tools

**What you don't get**:
- P2P sharing
- GraphQL support
- Scripting

### Full Installation

Everything included:

```bash
cargo install apicentric --features full
```

**What you get**:
- Everything

**Trade-off**:
- Longer build time (3-5 minutes)
- Larger binary size

### Custom Installation

Pick exactly what you need:

```bash
# Simulator + TUI only
cargo install apicentric --no-default-features --features simulator,tui

# Simulator + Contract Testing only
cargo install apicentric --no-default-features --features simulator,contract-testing

# Simulator + GraphQL only
cargo install apicentric --no-default-features --features simulator,graphql
```

## Build Time Comparison

| Feature Set | Dependencies | Build Time | Binary Size |
|-------------|--------------|------------|-------------|
| `minimal` | ~30 crates | < 1 min | ~5 MB |
| `default` | ~40 crates | < 1.5 min | ~8 MB |
| `cli-tools` | ~50 crates | < 2 min | ~12 MB |
| `full` | ~100+ crates | 3-5 min | ~25 MB |

*Build times measured on a modern development machine (M1 Mac, 16GB RAM)*

## Feature Detection

Check which features are enabled:

```bash
apicentric --version
```

Output includes enabled features:

```
apicentric 0.1.0
Features: simulator, tui, contract-testing
```

## Using Features

### Simulator (Always Available)

```bash
# Start simulator
apicentric simulator start --services-dir services

# Validate services
apicentric simulator validate --path services
```

### TUI (Requires `tui` feature)

```bash
# Start terminal UI
apicentric tui
```

If TUI is not enabled:

```
❌ Error: TUI feature not enabled
💡 Suggestion: Install with --features tui or --features cli-tools
```

### Contract Testing (Requires `contract-testing` feature)

```bash
# Register contract
apicentric contract register -n my-api -s services/api.yaml

# Run contract test
apicentric contract demo --contract-id <id>
```

### P2P (Requires `p2p` feature)

```bash
# Start simulator with P2P
apicentric simulator start --services-dir services --p2p
```

### GraphQL (Requires `graphql` feature)

```yaml
# In your service YAML
name: graphql-api
server:
  port: 9000
graphql:
  schema_path: schema.graphql
  mocks:
    getUser: user.json
```

### Scripting (Requires `scripting` feature)

```yaml
# In your service YAML
endpoints:
  - method: GET
    path: /dynamic
    responses:
      200:
        content_type: application/json
        body: '{"result": "{{runtime.value}}"}'
        script: scripts/generate.js
```

## Upgrading Features

### Add Features to Existing Installation

```bash
# Reinstall with additional features
cargo install apicentric --features cli-tools --force
```

### Remove Features

```bash
# Reinstall with fewer features
cargo install apicentric --no-default-features --features minimal --force
```

## CI/CD Considerations

### GitHub Actions

Use minimal features for faster CI:

```yaml
- name: Install Apicentric
  run: cargo install apicentric --no-default-features --features minimal

- name: Validate services
  run: apicentric simulator validate --path services
```

### Docker

Build with specific features:

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --no-default-features --features minimal

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/apicentric /usr/local/bin/
CMD ["apicentric"]
```

## Development

### Building with Features

```bash
# Build with specific features
cargo build --features tui

# Build with all features
cargo build --features full

# Build minimal
cargo build --no-default-features --features minimal
```

### Testing with Features

```bash
# Test specific feature
cargo test --features tui

# Test all features
cargo test --all-features

# Test minimal
cargo test --no-default-features --features minimal
```

## Troubleshooting

### Feature Not Available

If you see an error about a missing feature:

```
❌ Error: TUI feature not enabled
💡 Suggestion: Install with --features tui
```

**Solution**: Reinstall with the required feature:

```bash
cargo install apicentric --features cli-tools --force
```

### Build Takes Too Long

If builds are taking too long:

**Solution**: Use fewer features:

```bash
cargo install apicentric --no-default-features --features minimal
```

### Binary Too Large

If the binary is too large:

**Solution**: Install with minimal features and strip symbols:

```bash
cargo install apicentric --no-default-features --features minimal
strip $(which apicentric)
```

## Recommendations

### For Frontend Developers

```bash
cargo install apicentric --features cli-tools
```

You get: Simulator, TUI, and contract testing

### For Backend Developers

```bash
cargo install apicentric --no-default-features --features simulator,contract-testing
```

You get: Simulator and contract testing (no TUI)

### For CI/CD

```bash
cargo install apicentric --no-default-features --features minimal
```

You get: Fast builds, basic validation

### For Full Experience

```bash
cargo install apicentric --features full
```

You get: Everything

## Future Features

Planned features for future releases:

- `cloud` - Cloud deployment support
- `metrics` - Prometheus metrics
- `tracing` - Distributed tracing
- `plugins` - Plugin system

## Questions?

- Check the [documentation](../../README.md)
- Open an [issue](https://github.com/pmaojo/apicentric/issues)
- Start a [discussion](https://github.com/pmaojo/apicentric/discussions)
