# Terminal UI Guide

The Apicentric Terminal UI (TUI) provides an interactive dashboard for managing API services directly from your terminal.

## Prerequisites

The TUI feature must be enabled during installation:

```bash
# Install with TUI support
cargo install apicentric --features cli-tools

# Or with full features
cargo install apicentric --features full
```

## Starting the TUI

Launch the TUI with:

```bash
apicentric tui
```

The TUI will automatically:
1. Load services from your configured services directory
2. Display the current status of all services
3. Start streaming request logs in real-time

## Layout Overview

The TUI uses a three-panel layout:

```
┌─ Services ────────┬─ Request Logs ──────────────────┬─ Actions ─────┐
│ ● api-service     │ 2024-11-08 10:23:45             │ q: Quit       │
│   :9001           │ GET /api/users → 200 OK         │ ↑↓: Navigate  │
│                   │                                  │ ⏎: Start/Stop │
│ ● user-service    │ 2024-11-08 10:23:46             │ f: Filter     │
│   :9002           │ POST /api/login → 201 Created   │ r: Refresh    │
│                   │                                  │ c: Clear      │
│ ○ auth-service    │ 2024-11-08 10:23:47             │ s: Save       │
│   :9003 (stopped) │ GET /api/profile → 200 OK       │ /: Search     │
│                   │                                  │ ?: Help       │
│                   │ [Filtered: GET, 200]            │               │
│                   │ [Scroll: 1-10 of 156]           │ Status: ✓     │
└───────────────────┴─────────────────────────────────┴───────────────┘
```

### Services Panel (Left)

Displays all configured services with:
- **Status indicator**: ● (running) or ○ (stopped)
- **Service name**: The name from the YAML definition
- **Port**: The port the service is listening on
- **Selection**: Highlighted service can be controlled

### Request Logs Panel (Center)

Shows real-time request logs with:
- **Timestamp**: When the request was received
- **Method**: HTTP method (GET, POST, etc.)
- **Path**: Request path
- **Status**: Response status code
- **Color coding**:
  - Green: 2xx success
  - Yellow: 3xx redirect
  - Red: 4xx/5xx errors

### Actions Panel (Right)

Lists available keyboard shortcuts and shows:
- **Current filter**: Active log filters
- **Scroll position**: Current position in logs
- **Status**: Overall simulator status

## Keyboard Shortcuts

### Navigation

| Key | Action |
|-----|--------|
| `↑` | Move selection up |
| `↓` | Move selection down |
| `Tab` | Switch focus between panels |
| `Page Up` | Scroll logs up |
| `Page Down` | Scroll logs down |
| `Home` | Jump to top |
| `End` | Jump to bottom |

### Service Control

| Key | Action |
|-----|--------|
| `Enter` | Start/stop selected service |
| `r` | Refresh service status |

### Log Management

| Key | Action |
|-----|--------|
| `f` | Open filter dialog |
| `c` | Clear all logs |
| `s` | Save logs to file |
| `/` | Open search dialog |
| `Esc` | Clear filter/close dialog |

### General

| Key | Action |
|-----|--------|
| `?` | Show help dialog |
| `q` | Quit TUI |
| `Ctrl+C` | Force quit |

## Filtering Logs

Press `f` to open the filter dialog:

```
┌─ Filter Logs ─────────────────────────────────────┐
│                                                    │
│ Method: [GET____]                                  │
│ Status: [200____]                                  │
│ Service: [api-service_____]                        │
│                                                    │
│ Press Enter to apply, Esc to cancel                │
└────────────────────────────────────────────────────┘
```

### Filter Options

**Method**: Filter by HTTP method
- Examples: `GET`, `POST`, `PUT`, `DELETE`
- Leave empty to show all methods

**Status**: Filter by status code
- Examples: `200`, `404`, `500`
- Leave empty to show all statuses

**Service**: Filter by service name
- Examples: `api-service`, `user-service`
- Leave empty to show all services

### Applying Filters

1. Press `f` to open filter dialog
2. Type filter values (use Tab to move between fields)
3. Press `Enter` to apply
4. Press `Esc` to clear filters

### Active Filter Indicator

When filters are active, you'll see:

```
[Filtered: GET, 200, api-service]
```

## Searching Logs

Press `/` to open the search dialog:

```
┌─ Search Logs ─────────────────────────────────────┐
│                                                    │
│ Search: [/api/users___________________________]    │
│                                                    │
│ Press Enter to search, Esc to cancel               │
└────────────────────────────────────────────────────┘
```

Search matches are highlighted in the logs panel.

## Managing Services

### Starting a Service

1. Use `↑`/`↓` to select a stopped service (○)
2. Press `Enter`
3. Wait for status to update to running (●)

### Stopping a Service

1. Use `↑`/`↓` to select a running service (●)
2. Press `Enter`
3. Wait for status to update to stopped (○)

### Refreshing Status

Press `r` to manually refresh service status. The TUI automatically refreshes every second, but you can force an immediate update.

## Saving Logs

Press `s` to save current logs to a file:

```
Logs saved to: apicentric-logs-2024-11-08-102345.json
```

The file includes:
- Timestamp
- Method
- Path
- Status code
- Service name
- Request/response details

### Log File Format

```json
[
  {
    "timestamp": "2024-11-08T10:23:45Z",
    "method": "GET",
    "path": "/api/users",
    "status": 200,
    "service": "api-service",
    "duration_ms": 12
  }
]
```

## Real-Time Updates

The TUI updates automatically:

- **Service status**: Every 1 second
- **Request logs**: Immediately as requests arrive
- **UI refresh**: Every 250ms

### Performance

The TUI is designed to be efficient:
- Logs are limited to 1000 entries (oldest are removed)
- Updates are debounced to prevent flickering
- Only changed regions are redrawn

## Configuration

### Services Directory

The TUI loads services from the configured directory:

```json
{
  "simulator": {
    "services_dir": "services"
  }
}
```

### Port Range

Services are assigned ports from the configured range:

```json
{
  "simulator": {
    "port_range": {
      "start": 9000,
      "end": 9099
    }
  }
}
```

## Troubleshooting

### TUI Won't Start

**Error**: `TUI feature not enabled`

**Solution**: Reinstall with TUI support:

```bash
cargo install apicentric --features cli-tools --force
```

### Services Not Showing

**Problem**: No services appear in the services panel

**Solutions**:
1. Check services directory exists
2. Verify YAML files are valid
3. Check configuration file

```bash
# Validate services
apicentric simulator validate --path services --recursive
```

### Logs Not Appearing

**Problem**: Request logs don't show up

**Solutions**:
1. Verify service is running (● indicator)
2. Make requests to the service
3. Check if filters are active (press `Esc` to clear)

### Terminal Display Issues

**Problem**: UI looks corrupted or misaligned

**Solutions**:
1. Resize terminal window
2. Restart TUI
3. Check terminal supports ANSI colors

```bash
# Test terminal capabilities
echo -e "\033[32mGreen\033[0m"
```

### Performance Issues

**Problem**: TUI is slow or laggy

**Solutions**:
1. Clear logs (press `c`)
2. Reduce number of services
3. Check system resources

## Tips and Tricks

### Quick Service Toggle

1. Press `↑` or `↓` to select service
2. Press `Enter` to toggle
3. Repeat for other services

### Monitor Specific Service

1. Press `f` to open filter
2. Enter service name
3. Press `Enter`

Now you'll only see logs for that service.

### Export Logs for Analysis

1. Press `s` to save logs
2. Open saved JSON file
3. Use `jq` or similar tools for analysis

```bash
# Count requests by status
cat apicentric-logs-*.json | jq '[.[] | .status] | group_by(.) | map({status: .[0], count: length})'
```

### Watch Specific Endpoint

1. Press `/` to search
2. Enter endpoint path (e.g., `/api/users`)
3. Press `Enter`

Matching logs are highlighted.

## Advanced Usage

### Multiple TUI Instances

You can run multiple TUI instances for different service directories:

```bash
# Terminal 1
apicentric tui --config config1.json

# Terminal 2
apicentric tui --config config2.json
```

### TUI with P2P

If P2P is enabled, the TUI shows shared services:

```bash
apicentric simulator start --services-dir services --p2p
apicentric tui
```

Shared services are marked with a 🌐 indicator.

### Scripting TUI Actions

While the TUI is interactive, you can automate service management:

```bash
# Start services via CLI
apicentric simulator start --services-dir services

# Then monitor with TUI
apicentric tui
```

## Comparison with CLI

| Feature | TUI | CLI |
|---------|-----|-----|
| Real-time logs | ✅ | ❌ |
| Service status | ✅ | ✅ |
| Start/stop services | ✅ | ✅ |
| Log filtering | ✅ | ✅ |
| Scriptable | ❌ | ✅ |
| Visual | ✅ | ❌ |

Use TUI for:
- Interactive development
- Monitoring services
- Quick service management

Use CLI for:
- Automation
- CI/CD pipelines
- Scripting

## Keyboard Reference

Quick reference for all shortcuts:

```
Navigation:
  ↑/↓     - Navigate services
  Tab     - Switch panels
  PgUp/Dn - Scroll logs

Service Control:
  Enter   - Start/stop service
  r       - Refresh status

Logs:
  f       - Filter logs
  c       - Clear logs
  s       - Save logs
  /       - Search logs
  Esc     - Clear filter

General:
  ?       - Show help
  q       - Quit
  Ctrl+C  - Force quit
```

## Getting Help

- Press `?` in the TUI for help
- Check the [documentation](../../README.md)
- Open an [issue](https://github.com/pmaojo/apicentric/issues)
- Start a [discussion](https://github.com/pmaojo/apicentric/discussions)

## What's Next?

- Learn about [filtering and search](quick-start.md#filtering-logs)
- Explore [service definitions](quick-start.md#your-first-mock-api)
- Read about [contract testing](contract-testing.md)
