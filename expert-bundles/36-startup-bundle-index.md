# Script Kit GPUI - Startup Process Bundle Index

## Overview

This bundle series documents the complete application startup process for Script Kit GPUI.

---

## Bundle Files

| File | Description |
|------|-------------|
| `36-startup-overview.md` | High-level overview of all 5 startup phases |
| `36-startup-phase1-pre-gpui.md` | Pre-GPUI initialization (logging, setup, config) |
| `36-startup-phase2-watchers.md` | File watcher and scheduler creation |
| `36-startup-phase3-gpui-init.md` | GPUI application startup and component init |
| `36-startup-phase4-window-creation.md` | Main window creation and ScriptListApp init |
| `36-startup-phase5-async-tasks.md` | Async task spawning (hotkeys, watchers, stdin) |
| `36-startup-logging-system.md` | Dual-output logging system deep dive |
| `36-startup-environment-setup.md` | ~/.scriptkit environment setup deep dive |

---

## Quick Reference

### Startup Sequence Summary

```
1. logging::init()               ← JSONL + stderr logging
2. migrate_from_kenv()           ← Legacy migration
3. ensure_kit_setup()            ← Directory structure, SDK extraction
4. PROCESS_MANAGER.write_pid()   ← Orphan detection
5. cleanup_orphans()             ← Kill zombie processes
6. signal handlers               ← Async-signal-safe shutdown
7. config::load_config()         ← Bun eval (~100-300ms)
8. clipboard_history::init()     ← Background thread
9. ExpandManager::enable()       ← Text expansion (macOS)
10. McpServer::start()           ← AI agent integration
11. hotkeys::start_listener()    ← Global hotkey thread
12. Watchers created             ← Appearance, config, scripts
13. Scheduler::new()             ← Cron-based execution
14. Application::new().run()     ← GPUI event loop starts
15. configure_as_accessory_app() ← No dock icon
16. register_bundled_fonts()     ← JetBrains Mono
17. gpui_component::init()       ← Widget library
18. TrayManager::new()           ← System tray
19. cx.open_window()             ← Hidden main window
20. ScriptListApp::new()         ← Load scripts, theme
21. Async tasks spawned          ← Event handlers
```

### Key Files

| File | Role |
|------|------|
| `src/main.rs` | Entry point, orchestrates startup |
| `src/logging.rs` | Dual-output logging |
| `src/setup.rs` | Environment setup |
| `src/config/mod.rs` | Configuration loading |
| `src/theme/mod.rs` | Theme system |
| `src/hotkeys.rs` | Global hotkey registration |
| `src/watcher.rs` | File watchers |
| `src/tray.rs` | System tray |
| `src/stdin_commands.rs` | Stdin protocol |
| `src/app_impl.rs` | ScriptListApp implementation |

### Environment Paths

| Path | Purpose |
|------|---------|
| `~/.scriptkit/` | Root directory |
| `~/.scriptkit/kit/main/scripts/` | User scripts |
| `~/.scriptkit/kit/main/extensions/` | Scriptlets/extensions |
| `~/.scriptkit/kit/config.ts` | Configuration |
| `~/.scriptkit/kit/theme.json` | Theme |
| `~/.scriptkit/sdk/kit-sdk.ts` | SDK runtime |
| `~/.scriptkit/logs/` | Application logs |
| `~/.scriptkit/db/` | SQLite databases |

### Timing Expectations

| Phase | Duration |
|-------|----------|
| Pre-GPUI (mostly config load) | ~300ms |
| GPUI init | <10ms |
| Window creation | <15ms |
| Script loading | ~5ms |
| **Total to hotkey response** | ~400ms |

### Error Recovery

The startup is designed for graceful degradation:

| Component | Fallback |
|-----------|----------|
| Logging | /dev/null |
| Config | Defaults |
| Theme | System appearance |
| Hotkeys | Tray-only |
| Tray | Hotkey-only |
| Both | Show window at startup |
| Watchers | No live reload |
| Clipboard | No history |

---

## Testing Startup

### With AI Log Mode

```bash
SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
```

### Filter Startup Logs

```bash
# All startup events
SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1 | head -50

# Just APP category
SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1 | grep '|A|'

# Performance issues
grep '"duration_ms":' ~/.scriptkit/logs/script-kit-gpui.jsonl | \
  jq 'select(.fields.duration_ms > 100)'
```

### Simulate Fresh Install

```bash
rm -rf ~/.scriptkit
./target/debug/script-kit-gpui
```

### Test SK_PATH Override

```bash
export SK_PATH=/tmp/test-kit
./target/debug/script-kit-gpui
```

---

## Related Bundles

- `31-dev-build-bundle.md` - Development workflow and hot reload
- `10-hotkey-bundle.md` - Global hotkey system
- `11-watcher-bundle.md` - File watcher system
- `20-tray-bundle.md` - System tray integration
- `22-vibrancy-bundle.md` - Window vibrancy configuration
