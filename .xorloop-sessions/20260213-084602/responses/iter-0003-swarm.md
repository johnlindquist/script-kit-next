All 4 security fixes are committed. The only remaining uncommitted files are `.ai/roster.json` and `.xorloop-sessions/` (swarm metadata — not source code).

## Swarm Summary

All 4 security tasks completed successfully:

| Task | Commit | Status |
|------|--------|--------|
| **env-scrub-type-submit-osascript** | `aac5ca0c` | Applied `apply_scriptlet_environment_allowlist()` to `execute_type()` and `execute_submit()` osascript calls in `src/executor/scriptlet.rs` |
| **scrub-system-actions-env** | `46e43957` | Added `scrub_command_env()` helper and applied to all `Command::new` calls in `src/system_actions/mod.rs` |
| **quicklink-query-url-encode** | `d7648bb4` | Replaced raw query substitution with RFC 3986 percent-encoding in `src/quicklinks/mod.rs` |
| **pid-file-permissions** | `e4abede4` | Set PID files to 0o600 and parent dirs to 0o700 (Unix only) in `src/process_manager/mod.rs` |

**Build verification:** `cargo check` and `cargo clippy -- -D warnings` both pass clean. The `--all-targets` clippy has 190 pre-existing test compilation errors unrelated to these changes.
