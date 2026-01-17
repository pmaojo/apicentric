# Troubleshooting & Diagnostics

Apicentric provides built-in tools to help you identify and resolve environment issues.

## 🏥 apicentric doctor

The `doctor` command is your first line of defense. It checks your system for:

- 🦀 Rust/Cargo installation
- 🐳 Docker availability (required for `dockerize`)
- 📂 Project structure (`apicentric.json`, `services/`)
- 🌐 Basic connectivity

### Usage

```bash
apicentric doctor
```

### Common Checks

| Check        | Passing | Failing | Fix                                                |
| ------------ | ------- | ------- | -------------------------------------------------- |
| **Rust**     | `✅`    | `❌`    | Install Rust: `rustup update`                      |
| **Docker**   | `✅`    | `⚠️`    | Install Docker Desktop or start the daemon         |
| **Services** | `✅`    | `ℹ️`    | Run `apicentric new` to create a service           |
| **Config**   | `✅`    | `ℹ️`    | Create `apicentric.json` if you need custom config |

## Common Issues

### "Address already in use" (Port 9002)

**Symptom**: Simulator fails to start.
**Fix**:

1. Check what's using the port: `lsof -i :9002`
2. Kill the process or start Apicentric on a different port:
   ```json
   // apicentric.json
   {
     "simulator": { "port": 9003 }
   }
   ```

### "Docker executable not found"

**Symptom**: `apicentric simulator dockerize` fails.
**Fix**: Ensure `docker` is in your PATH and the daemon is running.

### "Connection Refused"

**Symptom**: `apicentric open` fails to load the page.
**Fix**: Ensure you have verified the simulator is running with `apicentric simulator status`.
