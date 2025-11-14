# Apicentric Command Dependency Graph

**Generated:** 2025-11-13  
**Purpose:** Detailed mapping of CLI commands to their module dependencies  
**Project:** Apicentric v0.1.2

## Overview

This document provides a comprehensive vertical trace of each CLI command from its entry point through all referenced modules, functions, and dependencies. This enables precise identification of unused code and understanding of the codebase structure.

## Command Entry Points

All commands enter through: `src/bin/apicentric.rs` → `main()` → `run()`

## 1. Simulator Commands

### 1.1 `simulator start`

**Command Path:**
```
src/bin/apicentric.rs::main()
  └─> Commands::Simulator { action: SimulatorAction::Start }
      └─> src/commands/simulator/mod.rs::simulator_command()
          └─> src/commands/simulator/control.rs::handle_start()
```

**Module Dependencies:**
```
src/commands/simulator/control.rs
  ├─> apicentric::Context
  ├─> apicentric::ExecutionContext
  └─> context.api_simulator()
      └─> src/simulator/manager.rs::ApiSimulator
          ├─> src/simulator/lifecycle.rs
          ├─> src/simulator/service/http_server.rs
          ├─> src/simulator/service/router.rs
          ├─> src/simulator/service/state.rs
          ├─> src/simulator/registry.rs
          ├─> src/simulator/config/mod.rs
          ├─> src/simulator/config/endpoint.rs
          ├─> src/simulator/config/server.rs
          ├─> src/simulator/template/mod.rs
          ├─> src/simulator/template/helpers/
          ├─> src/storage/sqlite.rs (if database feature)
          └─> src/collab/ (if p2p flag enabled)
              ├─> src/collab/mod.rs
              ├─> src/collab/p2p.rs
              ├─> src/collab/crdt.rs
              └─> src/collab/share.rs
```

**Feature Dependencies:**
- Core: Always required
- `database`: Optional (for persistent storage)
- `p2p`: Optional (if --p2p flag used)

**External Crate Dependencies:**
- `tokio` - Async runtime
- `axum` - HTTP server
- `tower` - Middleware
- `handlebars` - Template engine
- `serde`, `serde_yaml` - Configuration parsing
- `rusqlite` - Database (if feature enabled)
- `libp2p` - P2P networking (if feature enabled)

---

### 1.2 `simulator stop`

**Command Path:**
```
src/bin/apicentric.rs::main()
  └─> Commands::Simulator { action: SimulatorAction::Stop }
      └─> src/commands/simulator/mod.rs::simulator_command()
          └─> src/commands/simulator/control.rs::handle_stop()
```

**Module Dependencies:**
```
src/commands/simulator/control.rs
  └─> context.api_simulator()
      └─> src/simulator/manager.rs::ApiSimulator::stop()
          └─> src/simulator/lifecycle.rs
```

---

### 1.3 `simulator status`

**Command Path:**
```
src/bin/apicentric.rs::main()
  └─> Commands::Simulator { action: SimulatorAction::Status }
      └─> src/commands/simulator/mod.rs::simulator_command()
          └─> src/commands/simulator/control.rs::handle_status()
```

**Module Dependencies:**
```
src/commands/simulator/control.rs
  └─> context.api_simulator()
      └─> src/simulator/manager.rs::ApiSimulator::get_status()
          ├─> src/simulator/registry.rs
          └─> src/simulator/service/state.rs
```

---

### 1.4 `simulator validate`

**Command Path:**
```
src/bin/apicentric.rs::main()
  └─> Commands::Simulator { action: SimulatorAction::Validate }
      └─> src/commands/simulator/mod.rs::simulator_command()
          └─> src/commands/simulator/inspect.rs::handle_validate()
```

**Module Dependencies:**
```
src/commands/simulator/inspect.rs
  ├─> src/simulator/config/validation/mod.rs
  ├─> src/simulator/config/validation/validators.rs
  ├─> src/simulator/config/validation/repository.rs
  ├─> src/simulator/config/validation/summarizer.rs
  ├─> src/utils/directory_scanner.rs
  └─> src/utils/file_ops.rs
```

---

### 1.5 `simulator logs`

**Command Path:**
```
src/bin/apicentric.rs::main()
  └─> Commands::Simulator { action: SimulatorAction::Logs }
      └─> src/commands/simulator/mod.rs::simulator_command()
          └─> src/commands/simulator/inspect.rs::handle_logs()
```

**Module Dependencies:**
```
src/commands/simulator/inspect.rs
  └─> context.api_simulator()
      └─> src/simulator/manager.rs::ApiSimulator
          └─> src/storage/sqlite.rs (if database feature)
              └─> rusqlite crate
```

---

### 1.6 `simulator monitor`

**Command Path:**
```
src/bin/apicentric.rs::main()
  └─> Commands::Simulator { action: SimulatorAction::Monitor }
      └─> src/commands/simulator/mod.rs::simulator_command()
          └─> src/commands/simulator/inspect.rs::handle_monitor()
```

**Module Dependencies:**
```
src/commands/simulator/inspect.rs
  ├─> context.api_simulator()
  │   └─> src/simulator/manager.rs::ApiSimulator::get_status()
  └─> tokio::time (for interval monitoring)
```

---

### 1.7 `simulator set-scenario`

**Command Path:**
```
src/bin/apicentric.rs::main()
  └─> Commands::Simulator { action: SimulatorAction::SetScenario }
      └─> src/commands/simulator/mod.rs::simulator_command()
          └─> src/commands/simulator/control.rs::handle_set_scenario()
```

**Module Dependencies:**
```
src/commands/simulator/control.rs
  └─> context.api_simulator()
      └─> src/simulator/manager.rs::ApiSimulator
          └─> src/simulator/service/scenario.rs
```

---

### 1.8 `simulator import`

**Command Path:**
```
src/bin/apicentric.rs::main()
  └─> Commands::Simulator { action: SimulatorAction::Import }
      └─> src/commands/simulator/mod.rs::simulator_command()
          └─> src/commands/simulator/import.rs::handle_import()
```

**Module Dependencies:**
```
src/commands/simulator/import.rs
  ├─> src/simulator/openapi.rs (OpenAPI import)
  ├─> src/simulator/mockoon.rs (Mockoon import)
  ├─> src/simulator/postman.rs (Postman import)
  └─> src/simulator/wiremock.rs (WireMock import)
```

**Note:** This command exposes import functionality for multiple formats.

---

### 1.9 `simulator export`

**Command Path:**
```
src/bin/apicentric.rs::main()
  └─> Commands::Simulator { action: SimulatorAction::Export }
      └─> src/commands/simulator/mod.rs::simulator_command()
          └─> src/commands/simulator/export.rs::handle_export()
```

**Module Dependencies:**
```
src/commands/simulator/export.rs
  ├─> src/simulator/openapi.rs (OpenAPI export)
  └─> src/simulator/postman.rs (Postman export)
```

---

### 1.10 `simulator generate-types`

**Command Path:**
```
src/bin/apicentric.rs::main()
  └─> Commands::Simulator { action: SimulatorAction::GenerateTypes }
      └─> src/commands/simulator/mod.rs::simulator_command()
          └─> src/commands/simulator/export.rs::handle_export_types()
```

**Module Dependencies:**
```
src/commands/simulator/export.rs
  └─> src/simulator/typescript.rs
      └─> src/simulator/config/mod.rs
```

---

### 1.11 `simulator generate-query`

**Command Path:**
```
src/bin/apicentric.rs::main()
  └─> Commands::Simulator { action: SimulatorAction::GenerateQuery }
      └─> src/commands/simulator/mod.rs::simulator_command()
          └─> src/commands/simulator/export.rs::handle_export_query()
```

**Module Dependencies:**
```
src/commands/simulator/export.rs
  └─> src/simulator/react_query.rs
      └─> src/simulator/config/mod.rs
```

---

### 1.12 `simulator generate-view`

**Command Path:**
```
src/bin/apicentric.rs::main()
  └─> Commands::Simulator { action: SimulatorAction::GenerateView }
      └─> src/commands/simulator/mod.rs::simulator_command()
          └─> src/commands/simulator/export.rs::handle_export_view()
```

**Module Dependencies:**
```
src/commands/simulator/export.rs
  └─> src/simulator/react_view.rs
      └─> src/simulator/config/mod.rs
```

---

### 1.13 `simulator new`

**Command Path:**
```
src/bin/apicentric.rs::main()
  └─> Commands::Simulator { action: SimulatorAction::New }
      └─> src/commands/simulator/mod.rs::simulator_command()
          └─> src/commands/simulator/service.rs::handle_new()
```

**Module Dependencies:**
```
src/commands/simulator/service.rs
  ├─> inquire crate (interactive prompts)
  ├─> src/simulator/template/mod.rs
  └─> src/utils/file_ops.rs
```

**Feature Dependencies:**
- `tui` feature (for inquire crate)

---

### 1.14 `simulator new-graphql`

**Command Path:**
```
src/bin/apicentric.rs::main()
  └─> Commands::Simulator { action: SimulatorAction::NewGraphql }
      └─> src/commands/simulator/mod.rs::simulator_command()
          └─> src/commands/simulator/service.rs::handle_new_graphql()
```

**Module Dependencies:**
```
src/commands/simulator/service.rs
  ├─> src/simulator/service/graphql.rs
  └─> src/utils/file_ops.rs
```

**Feature Dependencies:**
- `graphql` feature (for async-graphql)

---

### 1.15 `simulator edit`

**Command Path:**
```
src/bin/apicentric.rs::main()
  └─> Commands::Simulator { action: SimulatorAction::Edit }
      └─> src/commands/simulator/mod.rs::simulator_command()
          └─> src/commands/simulator/service.rs::handle_edit()
```

**Module Dependencies:**
```
src/commands/simulator/service.rs
  ├─> inquire crate (interactive prompts)
  ├─> src/simulator/config/mod.rs
  └─> src/utils/file_ops.rs
```

---

### 1.16 `simulator record`

**Command Path:**
```
src/bin/apicentric.rs::main()
  └─> Commands::Simulator { action: SimulatorAction::Record }
      └─> src/commands/simulator/mod.rs::simulator_command()
          └─> src/commands/simulator/service.rs::handle_record()
```

**Module Dependencies:**
```
src/commands/simulator/service.rs
  └─> src/simulator/recording_proxy.rs
      ├─> src/simulator/config/mod.rs
      └─> hyper, http crates
```

---

### 1.17 `simulator share`

**Command Path:**
```
src/bin/apicentric.rs::main()
  └─> Commands::Simulator { action: SimulatorAction::Share }
      └─> src/commands/simulator/mod.rs::simulator_command()
          └─> src/commands/simulator/service.rs::handle_share()
```

**Module Dependencies:**
```
src/commands/simulator/service.rs
  └─> src/collab/share.rs
      ├─> src/collab/p2p.rs
      ├─> src/collab/crdt.rs
      └─> libp2p crate
```

**Feature Dependencies:**
- `p2p` feature (required)

---

### 1.18 `simulator connect`

**Command Path:**
```
src/bin/apicentric.rs::main()
  └─> Commands::Simulator { action: SimulatorAction::Connect }
      └─> src/commands/simulator/mod.rs::simulator_command()
          └─> src/commands/simulator/service.rs::handle_connect()
```

**Module Dependencies:**
```
src/commands/simulator/service.rs
  └─> src/collab/p2p.rs
      └─> libp2p crate
```

**Feature Dependencies:**
- `p2p` feature (required)

---

### 1.19 `simulator dockerize`

**Command Path:**
```
src/bin/apicentric.rs::main()
  └─> Commands::Simulator { action: SimulatorAction::Dockerize }
      └─> src/commands/simulator/mod.rs::simulator_command()
          └─> src/commands/simulator/dockerize.rs::handle_dockerize()
```

**Module Dependencies:**
```
src/commands/simulator/dockerize.rs
  ├─> src/simulator/config/mod.rs
  └─> src/utils/file_ops.rs
```

---

## 2. AI Commands

### 2.1 `ai generate`

**Command Path:**
```
src/bin/apicentric.rs::main()
  └─> Commands::Ai { action: AiAction::Generate }
      └─> src/commands/ai.rs::ai_command()
```

**Module Dependencies:**
```
src/commands/ai.rs
  └─> src/ai/mod.rs
      ├─> src/ai/openai.rs (OpenAI provider)
      ├─> src/ai/gemini.rs (Google Gemini provider)
      └─> src/ai/local.rs (Local model provider)
```

**External Crate Dependencies:**
- `reqwest` - HTTP client for API calls
- `serde_json` - JSON parsing

---

## 3. TUI Command

### 3.1 `tui`

**Command Path:**
```
src/bin/apicentric.rs::main()
  └─> Commands::Tui
      └─> src/commands/tui.rs::tui_command()
```

**Module Dependencies:**
```
src/commands/tui.rs
  ├─> src/commands/tui_state.rs
  ├─> src/commands/tui_events.rs
  ├─> src/commands/tui_render.rs
  └─> context.api_simulator()
      └─> src/simulator/manager.rs
```

**Feature Dependencies:**
- `tui` feature (required)

**External Crate Dependencies:**
- `ratatui` - Terminal UI framework
- `crossterm` - Terminal manipulation
- `tokio` - Async runtime

---

## 4. GUI Command

### 4.1 `gui`

**Command Path:**
```
src/bin/apicentric.rs::main()
  └─> Commands::Gui
      └─> src/commands/gui/mod.rs::gui_command()
```

**Module Dependencies:**
```
src/commands/gui/mod.rs
  ├─> src/commands/gui/state.rs
  ├─> src/commands/gui/events.rs
  ├─> src/commands/gui/render.rs
  ├─> src/commands/gui/style.rs
  ├─> src/commands/gui/ai/mod.rs
  ├─> src/commands/gui/ai/generator.rs
  └─> context.api_simulator()
      └─> src/simulator/manager.rs
```

**Feature Dependencies:**
- `gui` feature (required)

**External Crate Dependencies:**
- `eframe` - GUI framework
- `egui` - Immediate mode GUI library
- `tokio` - Async runtime

---

## 5. Shared Module Dependencies

### 5.1 Configuration System

**Used By:** All commands

**Module Tree:**
```
src/config/mod.rs
  ├─> src/config/repository.rs
  └─> src/config/validation.rs
```

**Purpose:** Load and validate apicentric.json configuration

---

### 5.2 Context System

**Used By:** All commands

**Module Tree:**
```
src/context/mod.rs
  └─> src/context/init.rs
```

**Purpose:** Application context and dependency injection

---

### 5.3 Storage System

**Used By:** Simulator (logs, state), Cloud (auth, sessions)

**Module Tree:**
```
src/storage/mod.rs
  └─> src/storage/sqlite.rs
```

**Feature Dependencies:**
- `database` feature

---

### 5.4 Authentication System

**Used By:** Cloud server, WebUI

**Module Tree:**
```
src/auth/mod.rs
  ├─> src/auth/handlers.rs
  ├─> src/auth/middleware.rs
  ├─> src/auth/jwt.rs
  ├─> src/auth/password.rs
  ├─> src/auth/model.rs
  ├─> src/auth/repository.rs
  ├─> src/auth/extractor.rs
  └─> src/auth/blacklist.rs
```

---

### 5.5 Cloud Server

**Used By:** WebUI

**Module Tree:**
```
src/cloud/mod.rs
  ├─> src/cloud/server.rs
  ├─> src/cloud/api.rs
  ├─> src/cloud/handlers.rs
  ├─> src/cloud/websocket.rs
  ├─> src/cloud/recording_session.rs
  ├─> src/cloud/monitoring.rs
  ├─> src/cloud/cors.rs
  └─> src/cloud/error.rs
```

**Feature Dependencies:**
- `webui` feature

---

### 5.6 Utilities

**Used By:** All commands

**Module Tree:**
```
src/utils/mod.rs
  ├─> src/utils/file_ops.rs
  ├─> src/utils/fs_utils.rs
  └─> src/utils/directory_scanner.rs
```

---

## 6. Module Usage Summary

### 6.1 Core Modules (Always Used)
- `src/config/` - Configuration management
- `src/context/` - Application context
- `src/utils/` - Utility functions
- `src/simulator/` - Core simulator (used by most commands)

### 6.2 Feature-Gated Modules
- `src/collab/` - P2P collaboration (feature: `p2p`)
- `src/commands/tui*` - Terminal UI (feature: `tui`)
- `src/commands/gui/` - Desktop GUI (feature: `gui`)
- `src/cloud/` - Cloud server (feature: `webui`)
- `src/auth/` - Authentication (feature: `webui`)
- `src/storage/` - Database (feature: `database`)

### 6.3 Potentially Underutilized Modules
- `src/adapters/npm/` - NPM integration (usage unclear)
- `src/simulator/mockoon.rs` - Mockoon import (exposed via import command)
- `src/simulator/postman.rs` - Postman import/export (exposed via import/export commands)

---

## 7. Dependency Graph Visualization

### 7.1 High-Level Command Flow

```
┌─────────────────────────────────────────────────────────────┐
│                  src/bin/apicentric.rs                       │
│                        main()                                │
└────────────────────────┬────────────────────────────────────┘
                         │
         ┌───────────────┼───────────────┬──────────────┐
         │               │               │              │
    ┌────▼────┐     ┌───▼────┐     ┌───▼───┐     ┌───▼───┐
    │Simulator│     │   AI   │     │  TUI  │     │  GUI  │
    │Commands │     │Commands│     │Command│     │Command│
    └────┬────┘     └───┬────┘     └───┬───┘     └───┬───┘
         │              │              │             │
         │              │              │             │
    ┌────▼──────────────▼──────────────▼─────────────▼────┐
    │              Shared Dependencies                     │
    │  ┌────────┐  ┌────────┐  ┌────────┐  ┌────────┐   │
    │  │Config  │  │Context │  │Storage │  │ Utils  │   │
    │  └────────┘  └────────┘  └────────┘  └────────┘   │
    └──────────────────────────────────────────────────────┘
```

### 7.2 Simulator Module Dependencies

```
src/simulator/
├── manager.rs ────────────┐
├── lifecycle.rs           │
├── registry.rs            │
├── router.rs              │
├── service/               │
│   ├── http_server.rs ────┤
│   ├── router.rs          │
│   ├── routing.rs         │
│   ├── state.rs           │
│   ├── state_service.rs   │
│   ├── scenario.rs        │
│   ├── graphql.rs         │
│   └── mod.rs             │
├── config/                │
│   ├── mod.rs ────────────┤
│   ├── endpoint.rs        │
│   ├── server.rs          │
│   └── validation/        │
│       ├── mod.rs         │
│       ├── validators.rs  │
│       ├── repository.rs  │
│       └── summarizer.rs  │
├── template/              │
│   ├── mod.rs ────────────┤
│   ├── context.rs         │
│   ├── preprocessor.rs    │
│   └── helpers/           │
│       ├── core.rs        │
│       ├── faker.rs       │
│       ├── math.rs        │
│       ├── text.rs        │
│       └── bucket.rs      │
├── openapi.rs             │
├── typescript.rs          │
├── react_query.rs         │
├── react_view.rs          │
├── recording_proxy.rs     │
├── wiremock.rs            │
├── mockoon.rs             │
├── postman.rs             │
├── axios_client.rs        │
├── watcher.rs             │
└── mod.rs                 │
                           │
    All used by ───────────┘
    ApiSimulator
```

---

## 8. Conclusions

### 8.1 Well-Used Modules
- All simulator core modules are actively used
- Configuration and context systems are essential
- Feature-gated modules (TUI, GUI, P2P) are properly isolated

### 8.2 Modules Requiring Review
1. **src/adapters/npm/** - Usage pattern unclear
2. **src/simulator/mockoon.rs** - Used by import command
3. **src/simulator/postman.rs** - Used by import/export commands

### 8.3 Import/Export Module Status
- **OpenAPI:** ✅ Actively used (import/export)
- **WireMock:** ✅ Actively used (import)
- **Mockoon:** ✅ Used via import command
- **Postman:** ✅ Used via import/export commands
- **TypeScript:** ✅ Used via generate-types
- **React Query:** ✅ Used via generate-query
- **React View:** ✅ Used via generate-view

All import/export modules are exposed through CLI commands and should be retained.
