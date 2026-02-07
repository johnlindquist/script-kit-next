# Startup Performance Audit

Date: 2026-02-07  
Agent: `codex-startup-perf`  
Scope: `src/main.rs`, `src/lib.rs`, `src/app_impl.rs`, `src/app_actions.rs`

## Summary

Startup-to-first-render is currently dominated by synchronous tray initialization on the main GPUI startup path.

Measured run (`SCRIPT_KIT_AI_LOG=1`, stdin protocol):

- `session_start`: `44.684`
- `GPUI Application starting`: `44.767` (+0.083s)
- `Tray icon initialized successfully`: `47.102` (+2.418s)
- `Window opened, creating ScriptListApp`: `47.112` (+2.428s)
- First `[RENDER_SCRIPT_LIST_END]`: `47.118` (+2.434s)

Primary conclusion: most startup latency before first render is in work done before `cx.open_window(...)`, especially tray setup.

## Findings

### P0: Tray init blocks first render

Evidence:

- Tray is created synchronously before window creation in `app.run`: `src/main.rs:2261`.
- Window open happens only afterward: `src/main.rs:2306`.
- Fresh run shows ~2.33s gap between `GPUI Application starting` and `Tray icon initialized successfully`.
- `TrayManager::new()` does synchronous SVG parsing + icon rasterization + menu construction: `src/tray.rs:208`, `src/tray.rs:293`.

Impact:

- First render waits on tray even though tray is not needed for immediate UI readiness.

Recommendation:

1. Defer tray initialization until after `cx.open_window(...)` returns.
2. Run tray setup in a detached background task, then register tray event loop when ready.
3. Keep fallback entry-point logic (hotkey/tray) but do not block first render on tray.

Expected gain:

- Should remove most of the ~2.33s pre-render stall.

### P0: Duplicate script/scriptlet scanning during startup

Evidence:

- Main app model loads scripts + scriptlets: `src/app_impl.rs:10`, `src/app_impl.rs:16`.
- Hotkey startup thread loads scripts + scriptlets again for shortcut registration: `src/hotkeys.rs:1136`, `src/hotkeys.rs:1148`.
- Scheduler registration scans script dirs again: `src/main.rs:2177`, `src/scripts/scheduling.rs:25`.

Impact:

- Redundant filesystem traversal and metadata parsing at startup.
- Extra allocations (path/name strings, vector population, metadata structures).

Recommendation:

1. Build a `StartupScriptSnapshot` once (scripts, scriptlets, shortcut metadata, schedule metadata).
2. Pass shared snapshot (`Arc`) to:
   - `ScriptListApp::new`
   - hotkey registration bootstrap
   - scheduler bootstrap
3. Keep watcher-driven incremental refresh after startup as-is.

Expected gain:

- Lower cold-start I/O and allocations, especially on large kits.

### P0: Heavy pre-`app.run` initialization can be deferred

Evidence:

Before entering `app.run`, `main()` currently performs multiple startup actions sequentially:

- setup/migration: `src/main.rs:1963`, `src/main.rs:1969`
- clipboard monitor init: `src/main.rs:2058`
- MCP server creation/start + discovery writes + bind: `src/main.rs:2109`, `src/mcp_server.rs:220`
- watcher startup: `src/main.rs:2137`, `src/main.rs:2149`, `src/main.rs:2162`
- scheduler scan/registration: `src/main.rs:2177`

Impact:

- Non-UI services compete for CPU/disk before app UI loop is fully live.

Recommendation:

1. Split startup into:
   - Phase A (blocking): minimal UI-critical setup only.
   - Phase B (deferred): MCP server, scheduler scan/start, watcher startup, keyword system bootstrap.
2. Execute Phase B from `cx.spawn` shortly after first render.

Expected gain:

- More predictable and faster launch-to-usable-window latency.

### P1: Hidden window still does full initial model work

Evidence:

- Main window is created hidden: `show: false` in `src/main.rs:2318`.
- `ScriptListApp::new(...)` still runs immediately with script/scriptlet loading, frecency/input-history load, registry build, and initial grouping/render: `src/main.rs:2328`, `src/app_impl.rs:2`, `src/app_impl.rs:428`.

Impact:

- Startup pays most menu construction cost even when user has not opened the launcher yet.

Recommendation:

1. Two-phase `ScriptListApp` construction:
   - Phase 1: lightweight shell + placeholders.
   - Phase 2: hydrate scripts/scriptlets/groups asynchronously.
2. Trigger hydration on first show (or in background after app-ready if prefetched behavior is desired).

Tradeoff:

- If moved fully to first-show, first hotkey-open may shift some latency from launch time to first invocation.

### P1: `ScriptListApp::new` has serial and duplicated startup work

Evidence:

- Serial loading: `read_scripts()` then `load_scriptlets()` in sequence: `src/app_impl.rs:10`, `src/app_impl.rs:16`.
- Theme loaded again in app constructor even after theme load in `main` for window background: `src/main.rs:2286`, `src/app_impl.rs:19`.

Impact:

- Unnecessary startup wall time and duplicate allocations in larger data sets.

Recommendation:

1. Parallelize scripts/scriptlets loading (e.g. `rayon::join` or scoped threads).
2. Pass already-loaded theme/config from `main` into constructor to avoid duplicate theme parse/load.

### P1: Polling bridges add startup/idle wakeups

Evidence:

- Provider registry startup + rebuild poll every 50ms: `src/app_impl.rs:483`, `src/app_impl.rs:7357`.
- App scan completion bridge polls every 50ms: `src/app_impl.rs:87`.
- Config/script watchers use adaptive polling loops (200ms-2000ms): `src/main.rs:2667`, `src/main.rs:2712`.

Impact:

- Extra wakeups and jitter; more background churn during startup and idle.

Recommendation:

1. Replace one-shot bridges with event-driven `recv().await` channels.
2. For std mpsc producer threads, forward completion into async channels once.
3. Keep polling only where underlying API requires it.

### P2: Startup log volume is high for keyword registration

Evidence:

- Startup logs include one line per keyword trigger registration (59 in this run) before app ready.
- Trigger registration starts from startup thread kicked in `main`: `src/main.rs:2071`.

Impact:

- Extra string formatting and I/O during launch.

Recommendation:

1. Log summary counts at `info`.
2. Move per-trigger lines to `debug`/`trace`.

## Scope Notes

- `src/lib.rs` contains shared module wiring and window visibility globals; no major first-render bottleneck identified.
- `src/app_actions.rs` is action-handling heavy but not in the critical startup path to first render.

## Suggested Implementation Order

1. Defer tray initialization until after window creation (`P0`).
2. Split startup into blocking UI-critical vs deferred service initialization (`P0`).
3. Introduce shared startup script snapshot to remove triple scanning (`P0`).
4. Parallelize `read_scripts`/`load_scriptlets` and remove duplicate theme load (`P1`).
5. Replace 50ms polling bridges with event-driven receives (`P1`).
6. Reduce per-item startup logging (`P2`).

## Verification Plan

Use this before/after sequence for each startup optimization step:

1. `cargo check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test`
4. Launch timing capture:

```bash
timeout 12 bash -c 'echo '"'"'{"type":"show"}'"'"' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1' > /tmp/startup-perf-show.log || true
```

5. Extract timeline markers:

```bash
awk -F'|' '
  /Session started event_type=session_start/ {s=$1; print "session_start=" $1}
  /GPUI Application starting/ {a=$1; print "gpui_start=" $1 " delta=" sprintf("%.3f", a-s) "s"}
  /Tray icon initialized successfully/ {t=$1; print "tray_ready=" $1 " delta=" sprintf("%.3f", t-s) "s"}
  /Window opened, creating ScriptListApp wrapped in Root/ {w=$1; print "window_opened=" $1 " delta=" sprintf("%.3f", w-s) "s"}
  /\[RENDER_SCRIPT_LIST_END\]/ {r=$1; print "first_render_end=" $1 " delta=" sprintf("%.3f", r-s) "s"; exit}
' /tmp/startup-perf-show.log
```

## Risks / Known Gaps

- Deferring tray init changes startup ordering; event-handler lifetime and fallback logic must be revalidated.
- Deferring watchers/MCP/scheduler may delay feature availability briefly after launch.
- Moving to shared startup snapshots requires careful invalidation when file watchers fire to avoid stale state.
- First-show lazy hydration can regress perceived responsiveness if not prefetched in background.
