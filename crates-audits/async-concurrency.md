# Async + Concurrency Crate Audit

Date: 2026-02-07
Scope: `async-channel` (2.3), `rayon` (1.10), `parking_lot` (0.12)

## Executive Summary

- `async-channel`: Used correctly for stdin command ingestion and broader event-driven listeners. The stdin path is bounded, non-polling, and has good correlation-ID logging.
- `rayon`: Used for app scanning work where CPU/IO-heavy icon extraction benefits from parallelism. Current implementation is partially serialized by SQLite mutex writes inside the parallel loop.
- `parking_lot`: Not preferred everywhere yet. It is used in a few key places, but the codebase still primarily uses `std::sync::Mutex`/`RwLock`.
- Deadlock posture: Mostly cautious. AI/Notes window modules explicitly avoid lock+`handle.update()` deadlocks. No immediate lock-order cycle found in audited paths, but mixed lock primitives and global mutex patterns keep some latent risk.

## Dependency Presence

- `Cargo.toml:57` declares `async-channel = "2.3"`.
- `Cargo.toml:82` declares `rayon = "1.10"`.
- `Cargo.toml:101` declares `parking_lot = "0.12"`.

## 1) `async-channel` audit

### Stdin listener correctness

- `src/stdin_commands.rs:520` starts a dedicated stdin listener and returns `async_channel::Receiver<ExternalCommandEnvelope>`.
- `src/stdin_commands.rs:523` uses `async_channel::bounded(100)` to prevent unbounded queue growth.
- `src/stdin_commands.rs:568-574` uses `send_blocking` from a synchronous thread, which is appropriate for this context.
- `src/stdin_commands.rs:576-583` exits cleanly if receiver is dropped (channel closed).
- `src/stdin_commands.rs:540-625` handles EOF, parse failures, oversize lines, and read errors with structured logs.
- `src/stdin_commands.rs:526-632` includes correlation IDs in listener lifecycle and parse/error events.

### Consumer side

- `src/main.rs:3164` initializes the listener.
- `src/main.rs:3193-3197` consumes via `stdin_rx.recv().await` (event-driven, no polling loop).
- `src/main.rs:3198-3204` propagates envelope correlation IDs into processing logs.

### Verdict

- For stdin commands, `async-channel` usage is correct and aligned with the repo’s event-driven guidance.

### Minor risk/optimization note

- The bounded queue plus `send_blocking` can intentionally apply backpressure if command handling stalls. This is generally good, but high burst inputs may block producer progress.

## 2) `rayon` audit

### Current use

- `src/app_launcher.rs:20` imports `rayon::prelude::*`.
- `src/app_launcher.rs:864-877` uses `.par_iter()` to process discovered `.app` bundles in parallel.
- The parallel closure performs bundle parse/icon extraction via `parse_app_bundle_with_icon` (`src/app_launcher.rs:868-912`).

### CPU-bound suitability

- Icon decode/extraction work is a valid CPU-heavy target for rayon, so usage intent is correct.

### Contention caveat

- Each rayon task calls `save_app_to_db` (`src/app_launcher.rs:871`).
- `save_app_to_db` locks a global SQLite connection mutex (`src/app_launcher.rs:419-425`).
- This introduces serialization on writes, reducing effective parallel speedup.

### Verdict

- Rayon is used in an appropriate hotspot, but current DB-write placement limits realized parallelism.

## 3) `parking_lot` audit

### Current adoption

- Observed `parking_lot` imports in:
- `src/main.rs:232` (`Mutex as ParkingMutex`)
- `src/hud_manager.rs:15` (`parking_lot::Mutex`)
- `src/hotkeys.rs:5` (`parking_lot::RwLock`)
- `src/frontmost_app_tracker.rs:40` (`parking_lot::RwLock`)

### Coverage vs `std::sync`

- `std::sync::Mutex` appears broadly across window globals, caches, DB holders, and utility modules (e.g. `src/ai/window.rs`, `src/notes/window.rs`, `src/app_launcher.rs`, `src/logging.rs`).
- Quick count found 65 `std::sync::Mutex` references under `src/**/*.rs`.

### Verdict

- `parking_lot` is not preferred everywhere today; usage is mixed and mostly still `std::sync`.

## 4) Deadlock-risk audit

### Positive patterns in code

- AI window module explicitly releases lock before calling `handle.update()`:
- `src/ai/window.rs:7818-7823`
- `src/ai/window.rs:8035-8041`
- Notes window module applies the same guard pattern:
- `src/notes/window.rs:4653-4658`
- `src/notes/window.rs:4908-4914`

### Areas to watch

- Global window handle mutexes (`AI_WINDOW`, `NOTES_WINDOW`) are widespread, so future edits that accidentally hold lock across `update()` would reintroduce deadlock risk.
- Hotkey subsystem uses multiple lock domains (`MAIN_MANAGER` mutex plus routes `RwLock`) but current audited flows keep a consistent order and do not show a concrete cycle.

### Verdict

- No immediate deadlock bug was confirmed in audited paths.
- Risk is moderate in future maintenance due many global locks and mixed locking primitives.

## 5) Parallelism opportunities

1. App scan write pipeline (`src/app_launcher.rs:864-877`, `src/app_launcher.rs:410-450`):
- Keep rayon for parse/icon extraction.
- Collect `(AppInfo, icon_bytes, mtime)` in parallel.
- Perform batched DB writes on a single writer thread or in explicit transaction blocks after parallel phase.

2. DB cache icon decode path (`src/app_launcher.rs:381-397`):
- `load_apps_from_db_with_icons` decodes icons sequentially.
- Consider parallel decode on the icon blob list (bounded chunking to avoid memory spikes).

3. Lock primitive standardization:
- For pure in-process short critical sections, migrate selected high-traffic globals to `parking_lot` to reduce poisoning/verbose lock recovery overhead and improve consistency.

## Recommended Actions (Prioritized)

1. Refactor app scan pipeline to decouple parallel parse from serialized DB writes.
2. Add a short locking policy doc (order and "never hold lock across `handle.update()`") and reference it in AI/Notes/Window modules.
3. Define a `parking_lot` adoption rule for new code and incrementally migrate high-contention global mutexes.

## Overall Answer to Assignment Questions

- Is `async-channel` used correctly for stdin command listener?
- Yes. Bounded channel, synchronous producer with `send_blocking`, async consumer with `recv().await`, and strong error/correlation logging are all in place.

- Is rayon being used for CPU-bound parallel work (app scanning)?
- Yes, for app bundle parse/icon extraction. However, DB writes inside the rayon closure serialize on a global mutex and cap throughput.

- Is `parking_lot::Mutex` preferred over `std::sync::Mutex` everywhere?
- No. `parking_lot` is used in a few modules, but `std::sync::Mutex` remains dominant across the codebase.

- Any deadlock risks?
- No immediate deadlock found in audited paths; explicit lock-release safeguards exist in AI/Notes window management. Residual risk remains due many global mutexes and mixed lock domains.

- Places that could benefit from more parallelism?
- Yes: app scan DB-write pipeline and DB icon decode path are clear candidates.
