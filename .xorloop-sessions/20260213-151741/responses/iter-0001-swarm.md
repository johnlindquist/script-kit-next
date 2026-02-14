All 4 tasks are committed. Here's the summary:

---

## Swarm Results: Watcher Refactoring

### Commits (4 total)
| Commit | Task | Description |
|--------|------|-------------|
| `234631f3` | watcher-core | Generic watcher engine: `WatcherSpec` trait, `EventSink` trait, `GenericWatcher` lifecycle + Drop bug fix |
| `5cecc818` | config-theme-migration | Migrated ConfigWatcher + ThemeWatcher to `SingleFileReloadSpec` over `GenericWatcher` |
| `ef237642` | app-migration | Migrated AppWatcher onto `GenericWatcher` with `async_channel` sink + `~/Applications` `create_dir_all` bug fix |
| `ba8a6231` | script-migration | Migrated ScriptWatcher to `ScriptWatcherSpec` with dynamic watch registration + `setup()` re-discovery bug fix |

### Bug Fixes Included
1. **Drop wake-up** — `GenericWatcher::drop()` now sends `ControlMsg::Stop` immediately instead of blocking up to `health_check_interval_ms`
2. **Kit path re-discovery** — `discover_kit_watch_paths()` now runs in `setup()` on every supervisor restart, so kits created during backoff are picked up
3. **~/Applications** — `create_dir_all` before watching, so apps installed after startup aren't missed

### Verification
- `cargo check` — passes
- `cargo clippy --lib --bins -- -D warnings` — passes
- Test compilation failures are **pre-existing** in unrelated modules (`src/actions/tests/*`, `src/scripts/search/tests/*`)

### Worker Retries
- Tasks 1 and 2 completed on first attempt
- Task 3 (script migration) required 3 attempts due to timeout — the ScriptWatcher is the most complex
- Task 4 (app migration) required 2 attempts due to timeout on first run
