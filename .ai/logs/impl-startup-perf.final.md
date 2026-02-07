Implemented startup-performance changes in the requested scope, with runtime validation.

**Changed Files**
1. `src/app_impl.rs:15`
2. `src/main.rs:2412`
3. `tests/startup_perf_event_driven.rs:1`

`src/lib.rs` was reviewed in scope but no startup-path change was required for this task.

**What Changed**
1. `src/app_impl.rs`
- Parallelized initial `read_scripts` + `load_scriptlets` in `ScriptListApp::new` using scoped threads (`src/app_impl.rs:15`).
- Replaced startup polling bridges (`try_recv` + 50ms timers) with event-driven async receives for:
  - Background app scan completion (`src/app_impl.rs:135`)
  - Initial provider registry build (`src/app_impl.rs:524`)
  - Provider registry rebuild (`src/app_impl.rs:7396`)

2. `src/main.rs`
- Deferred tray initialization until after window creation in an async task (`src/main.rs:2412`, `src/main.rs:2421`).
- Replaced tray menu polling loop (250ms `try_recv`) with event-driven handling:
  - Blocking bridge thread (`MenuEvent::receiver().recv()`) -> `async_channel`
  - Async `recv().await` handler (`src/main.rs:2447`, `src/main.rs:2452`)
- Fallback “entry points available” check now reads tray readiness atomically (`src/main.rs:2591`).

3. `tests/startup_perf_event_driven.rs`
- Added regression tests enforcing event-driven startup behavior for both `app_impl` and `main`:
  - no startup `try_recv` polling in `ScriptListApp::new`
  - no polling in `rebuild_provider_registry_async`
  - tray init deferred past window creation
  - tray menu handling uses `recv().await`

**Verification Run**
1. `cargo check` ✅
2. `cargo test --test startup_perf_event_driven` ✅ (4/4)
3. Startup smoke timing:
- `window_opened` at `+0.163s`
- `first_render_end` at `+0.186s`
- `tray_ready` at `+1.878s`  
This confirms first render now happens before tray initialization.

Also run:
- `cargo clippy --all-targets -- -D warnings` ❌ (pre-existing unrelated failures in other modules)
- `cargo test` ❌ (10 unrelated failing tests outside this change set)

**How To Test**
1. `cargo check`
2. `cargo test --test startup_perf_event_driven`
3. 
```bash
timeout 12 bash -c 'echo "{\"type\":\"show\"}" | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1' > /tmp/startup-perf-show.log || true
awk -F'|' '
/Session started event_type=session_start/ {s=$1}
/Window opened, creating ScriptListApp wrapped in Root/ {print "window_opened delta=" sprintf("%.3f", $1-s) "s"}
/\[RENDER_SCRIPT_LIST_END\]/ {print "first_render_end delta=" sprintf("%.3f", $1-s) "s"}
/Tray icon initialized successfully/ {print "tray_ready delta=" sprintf("%.3f", $1-s) "s"}
' /tmp/startup-perf-show.log
```

**Risks / Known Gaps**
1. Pre-`app.run` heavy init (MCP/watchers/scheduler) is still not deferred in this change.
2. New regression tests are source-audit style; they enforce architecture intent but are not full behavioral integration tests.
3. Full repo `clippy -D warnings` and full `cargo test` currently fail due unrelated existing issues.

**Commits**
1. No commits were made.