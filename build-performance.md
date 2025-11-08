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

### Dependency Tree Analysis

Total direct dependencies: ~30
Key heavy dependency chains identified:
- **reqwest**: Pulls in hyper, tokio, native-tls, h2 (~50+ transitive deps)
- **ratatui + crossterm**: TUI stack (~20+ transitive deps)
- **rusqlite**: With bundled feature (~10+ transitive deps)
- **fake**: Mock data generation (~15+ transitive deps)
- **handlebars**: Template engine (~10+ transitive deps)
- **wasm-bindgen**: NOT needed for CLI tool (can be removed)

### Missing Dependencies Added

The following dependencies were missing from Cargo-full.toml but required by the code:
- hyper = { version = "1.0", features = ["full"] }
- hyper-util = { version = "0.1", features = ["full"] }
- http = "1.0"
- http-body-util = "0.1"
- bytes = "1.0"
- futures-util = "0.3"
- tokio-stream = { version = "0.1", features = ["sync"] }
- tracing = "0.1"
- tokio features: "sync", "time", "macros" (added to existing)

### Recommendations for Feature Flags

Based on analysis, create these feature groups:

1. **simulator** (default, core):
   - tokio, serde, clap, anyhow, thiserror
   - handlebars, regex, chrono, uuid
   - hyper, http, bytes (for HTTP server)

2. **tui** (optional):
   - ratatui, crossterm, indicatif, console, colored, inquire

3. **contract-testing** (optional):
   - reqwest, async-trait

4. **mock-data** (optional):
   - fake, rand

5. **database** (optional):
   - rusqlite

6. **file-watch** (optional):
   - notify

7. **cli-tools** (convenience bundle):
   - simulator + tui + contract-testing

8. **full** (all features):
   - All of the above

### Items to Remove
- wasm-bindgen: Not needed for CLI tool

## Next Steps

1. ✅ Complete baseline build time measurement (in progress)
2. ✅ Analyze `cargo tree` output to identify actual heavy dependencies
3. Create feature flag structure in Cargo.toml
4. Make TUI dependencies optional
5. Add conditional compilation for TUI code
6. Test builds with different feature combinations


## Build Testing Results

### Test 1: Minimal Build
**Command**: `cargo build --no-default-features --features minimal`
**Status**: ❌ Failed - requires additional conditional compilation throughout codebase
**Errors**: Code uses optional dependencies (rusqlite, fake, notify, etc.) without #[cfg] guards

### Test 2: Default Build  
**Command**: `cargo build`
**Status**: ✅ Testing in progress...

### Test 3: CLI Tools Build
**Command**: `cargo build --features cli-tools`
**Status**: Pending

### Test 4: Full Build
**Command**: `cargo build --features full`
**Status**: Pending

## Implementation Status

### ✅ Completed
1. Feature flag structure created in Cargo.toml
2. TUI dependencies made optional
3. TUI code wrapped with #[cfg(feature = "tui")]
4. CLI conditionally shows TUI command based on feature
5. Colored kept as always-available (lightweight)

### ⚠️ Partial / Needs More Work
1. **Database (rusqlite)**: Made optional in Cargo.toml but code needs #[cfg(feature = "database")]
2. **Mock data (fake, rand)**: Made optional but code needs conditional compilation
3. **File watching (notify)**: Made optional but code needs conditional compilation
4. **Contract testing (reqwest)**: Made optional but code needs conditional compilation

### 📝 Additional Work Required for Full Feature Flag Support

To make minimal builds work, the following files need conditional compilation:

1. **Storage module** (src/storage/sqlite.rs):
   - Wrap with #[cfg(feature = "database")]
   - Provide alternative or stub implementation

2. **Template helpers** (src/simulator/template/helpers/faker.rs):
   - Wrap fake/rand usage with #[cfg(feature = "mock-data")]

3. **File watcher** (src/simulator/watcher.rs):
   - Wrap with #[cfg(feature = "file-watch")]

4. **Contract testing** (src/adapters/http_client.rs, src/domain/contract/*):
   - Wrap reqwest usage with #[cfg(feature = "contract-testing")]

5. **WebSocket support** (src/simulator/router.rs):
   - tokio-tungstenite needs to be added and made optional
   - Wrap WebSocket code with feature flag

6. **P2P/Collaboration** (src/collab/*):
   - Already commented out in main binary
   - Dependencies not in Cargo.toml
   - Can be safely ignored for now

## Summary

Feature flags have been successfully implemented with the following structure:

- **default**: simulator
- **minimal**: simulator only
- **cli-tools**: simulator + contract-testing + tui
- **full**: all features

Optional dependencies made conditional:
- TUI stack (ratatui, crossterm, indicatif, console, colored, inquire)
- Contract testing (reqwest, async-trait)
- Mock data (fake, rand)
- Database (rusqlite)
- File watching (notify)

Conditional compilation added to:
- src/commands/tui.rs
- src/commands/tui_state.rs
- src/commands/tui_render.rs
- src/commands/tui_events.rs
- src/commands/mod.rs
- src/bin/apicentric.rs
