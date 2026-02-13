`cargo check` passes. Here's the summary:

## Swarm Results

| Task | Worker ID | Status | Commit |
|------|-----------|--------|--------|
| **harden-agent-build-terminal-cmd** | `harden-agent-build-terminal-cmd` | Done | `4dfce476` - Hardened `build_terminal_command()` with path validation, Result return type, no unwrap |
| **clipboard-restore-race-fix** | `clipboard-restore-race-fix-v2` | Done | Landed in `4dfce476` (concurrent commit) - 150ms post-paste delay, guaranteed clipboard restore |
| **pty-env-inheritance-scrub** | `pty-env-inheritance-scrub` | Done | `58256686` - Added `env_clear()` + safe allowlist in PTY spawn |
| **runner-env-scrub-script-spawn** | `runner-env-scrub-script-spawn` | Done | `e2ce992c` - Added env scrub with `SCRIPT_KIT_*` forwarding in `spawn_script()` and `run_command()` |

All 4 tasks completed. `cargo check` passes. 3 new commits on `main` hardening environment inheritance and input validation across agents, PTY, and script runner paths.
