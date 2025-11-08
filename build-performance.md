# Build Performance Analysis

## Baseline Measurements

### Current Build Configuration
- **Date**: 2025-11-08
- **Cargo.toml**: Using Cargo-full.toml as baseline
- **Build Command**: `cargo build --release`

### Build Time Measurement
**Status**: In progress...

### Dependency Analysis

#### Current Dependencies (from Cargo-full.toml)

**Core CLI Dependencies:**
- anyhow = "1.0.99"
- clap = { version = "4.4.8", features = ["derive"] }
- glob = "0.3.3"
- lazy_static = "1.5.0"
- log = "0.4.20"
- env_logger = "0.10"
- serde = { version = "1.0.219", features = ["derive"] }
- serde_json = "1.0.143"
- serde_yaml = "0.9.34"
- thiserror = "1.0.69"
- url = "2.5.4"
- regex = "1.11.1"
- chrono = { version = "0.4.31", features = ["serde"] }
- uuid = { version = "1.11.0", features = ["v4"] }
- tempfile = "3.8.0"

**Async Runtime:**
- tokio = { version = "1.47.1", features = ["rt-multi-thread", "fs", "process", "net"] }

**File Operations:**
- notify = "8.2.0"

**Template Engine:**
- handlebars = "6.2.0"

**Mock Data Generation:**
- fake = "4.4.0"
- rand = "0.8"

**OpenAPI Support:**
- openapi = "0.1.5"

**HTTP Client:**
- reqwest = { version = "0.11", features = ["json"] }
- async-trait = "0.1.74"

**Database:**
- rusqlite = { version = "0.31", features = ["bundled"] }

**Plugin System:**
- libloading = "0.8"

**Terminal UI:**
- indicatif = "0.17.7"
- console = "0.15.7"
- colored = "2.1.0"
- inquire = "0.6"
- ratatui = "0.26.0"
- crossterm = "0.27"

#### Heavy Dependencies Identified

Based on the design document analysis, the following dependencies are identified as heavy and should be made optional:

1. **P2P/Collaboration** (NOT in current Cargo-full.toml):
   - libp2p (0.56) - VERY HEAVY
   - automerge (0.6) - HEAVY
   - tokio-tungstenite (0.21) - MODERATE

2. **Scripting** (NOT in current Cargo-full.toml):
   - deno_core (0.272.0) - VERY HEAVY

3. **GraphQL** (NOT in current Cargo-full.toml):
   - async-graphql (7.0) - HEAVY

4. **Terminal UI** (PRESENT):
   - ratatui = "0.26.0"
   - crossterm = "0.27"
   - indicatif = "0.17.7"
   - console = "0.15.7"
   - colored = "2.1.0"
   - inquire = "0.6"

5. **HTTP Server** (NOT explicitly listed, likely using hyper):
   - hyper + hyper-util - MODERATE to HEAVY

#### Observations

1. The current Cargo-full.toml does NOT include the heaviest dependencies mentioned in the design:
   - No libp2p
   - No deno_core
   - No async-graphql
   - No automerge
   - No tokio-tungstenite

2. This suggests these features may have already been removed or were planned but not implemented.

3. The TUI dependencies ARE present and should be made optional.

4. The codebase references these missing dependencies, which explains the build errors seen earlier.

## Next Steps

1. Complete baseline build time measurement
2. Analyze `cargo tree` output to identify actual heavy dependencies
3. Create feature flag structure based on actual dependencies
4. Make TUI dependencies optional
5. Add conditional compilation for TUI code
