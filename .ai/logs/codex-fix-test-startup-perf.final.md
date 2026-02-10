Changed files:
- `tests/startup_perf_event_driven.rs`

What I changed:
- Updated test source readers to match current code layout:
  - `src/app_impl/startup.rs` for `ScriptListApp::new`
  - `src/app_impl/prompt_ai.rs` for `rebuild_provider_registry_async`
  - `src/main_entry/app_run_setup.rs` for deferred tray startup behavior
- Kept the event-driven assertions, but pointed them at the real startup/tray code paths in the split-module architecture.
- Added assertions that `src/main.rs` still includes `main_entry/app_run_setup.rs` so tests stay tied to the actual entrypoint wiring.

How to test:
- `cargo test --test startup_perf_event_driven`

Verification run:
- `cargo test --test startup_perf_event_driven` (4 passed, 0 failed)

Risks / known gaps:
- These tests are string-pattern/source-structure tests, so they are sensitive to file moves/renames and log text changes.
- I ran scoped verification only (per parallel-agent constraints), not full workspace `check/clippy/test`.

Commits:
- `9cb8d9d` `test(startup): align event-driven startup perf assertions`