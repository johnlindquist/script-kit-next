Implemented the wiring so watcher runtime behavior now uses loaded config values, including `processLimits` for health-check timing.

**Changed Files**
- `src/watcher/part_000.rs`
- `src/watcher/part_001.rs`
- `src/watcher/part_002.rs`
- `src/watcher/part_003.rs`
- `src/watcher/part_004_tests/tests_part_001.rs`
- `src/config/types/part_03.rs`

**What Changed**
- Added config-driven `WatcherSettings` usage across all watcher loops (config/theme/scripts/apps):
  - Debounce interval
  - Storm threshold
  - Backoff initial/max
  - Max notify errors
- Wired `processLimits.health_check_interval_ms` into idle `recv_timeout` intervals (replacing hardcoded `500ms`).
- Updated backoff helper to use runtime settings instead of compile-time constants.
- Added watcher test coverage for process health interval wiring (`test_watcher_settings_uses_process_health_interval_when_configured`).
- Removed `#[allow(dead_code)]` from `get_process_limits()` since it is now used at runtime.

**How To Test**
1. `cargo test --lib watcher::tests:: -- --nocapture`
2. `cargo check -p script-kit-gpui`

I ran both commands successfully:
- Watcher tests: `59 passed`
- `cargo check -p script-kit-gpui`: passed (with unrelated existing warnings in other modules)

**Risks / Known Gaps**
- Watcher settings are loaded when each watcher starts; config changes do not hot-reconfigure already running watcher threads.
- `process_limits.max_memory_mb` and `process_limits.max_runtime_seconds` still have no runtime consumer in this scoped change (only `health_check_interval_ms` is now wired).

**Commit**
- `9c68db4` `fix(watcher): use config-driven watcher and health-check timings`