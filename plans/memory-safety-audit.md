# Memory Safety Audit (`src/**/*.rs`)

Date: 2026-02-07
Agent: `codex-memory-safety`

## Scope and Method

Audited unsafe code, ObjC interop, raw pointer usage, and FFI boundaries across `src/**/*.rs`, with extra depth on:

- `src/camera.rs` (AVFoundation + dispatch queue lifecycle)
- `src/menu_executor.rs` / `src/menu_bar.rs` (AX + CF ownership)
- `src/window_control.rs` / `src/window_manager.rs` (cached pointers and registry)
- dispatch queue trampoline patterns (`src/hotkeys.rs`)

Inventory snapshot from this run:

- `unsafe { ... }` blocks: 129
- `extern "C"` blocks: 36
- Raw pointer token occurrences (`*mut`/`*const`): 200

## Severity Rubric

- Critical: Clear use-after-free / memory corruption hazard in normal paths.
- High: Strong leak/dangling/lifetime hazard likely in production.
- Medium: Hardening gap that can become a safety bug under edge conditions.
- Low: Defensive improvement; low probability or process-lifetime impact.

## Executive Summary

- Findings: **1 Critical, 3 High, 2 Medium, 3 Low**.
- Highest-risk issue is a **use-after-free in AX menu navigation** where borrowed elements escape a released CFArray (`src/menu_executor.rs`).
- Next highest risks are in window pointer caching (`src/window_control.rs`) and raw `NSWindow*` registry semantics (`src/window_manager.rs`).
- Camera teardown ordering is mostly good, but AVFoundation/GCD setup needs null/alloc hardening.

## Findings

### MSA-CRIT-001: Borrowed AXUIElement pointer used after CFArray release

- Severity: **Critical**
- Files:
  - `src/menu_executor.rs:341`
  - `src/menu_executor.rs:376`
  - `src/menu_executor.rs:377`
  - `src/menu_executor.rs:514`
  - `src/menu_executor.rs:552`
  - `src/menu_executor.rs:555`

#### Evidence

- `CFArrayGetValueAtIndex` returns a borrowed element (`src/menu_executor.rs:341`, `src/menu_executor.rs:368`).
- `open_menu_at_element` releases `children` then returns `child` (`src/menu_executor.rs:376-377`).
- `navigate_and_execute_menu_path` finds `menu_item` from `children`, releases `children`, then continues using `menu_item` (`src/menu_executor.rs:514`, `src/menu_executor.rs:552`, `src/menu_executor.rs:555`).

#### Risk

Use-after-free/dangling AX pointer; can cause intermittent crashes, invalid AX calls, or undefined behavior when menu hierarchy mutates.

#### Recommended Fix

1. Add `CFRetain` binding/helper in `menu_executor.rs`.
2. Retain any child pointer that must outlive its parent CFArray.
3. Release retained menu/submenu refs on all paths (success + error), ideally via RAII wrapper.
4. Keep ownership explicit in signatures (e.g., `OwnedAxElement`).

#### Verification

- Add regression tests for menu traversal ownership behavior (mocked AX layer or integration harness).
- Run repeated menu-path execution against a dynamic menu app and check for stability/crashes.

---

### MSA-HIGH-002: Window cache overwrite leaks retained AX refs

- Severity: **High**
- Files:
  - `src/window_control.rs:466`
  - `src/window_control.rs:468`
  - `src/window_control.rs:775`
  - `src/window_control.rs:778`

#### Evidence

- `cache_window` inserts pointer without releasing any previous value for same key (`src/window_control.rs:466-469`).
- `get_frontmost_window_of_previous_app` caches using `window_id = pid << 16`, which can overwrite repeatedly (`src/window_control.rs:775-778`).

#### Risk

Reference leak of retained AX elements over long-running sessions.

#### Recommended Fix

1. In `cache_window`, capture old pointer from `insert` and `cf_release` it if replaced.
2. Optionally dedupe identical pointers to avoid redundant retain/release churn.
3. Add focused unit tests for overwrite/release behavior.

---

### MSA-HIGH-003: Cached window pointer lifetime race across lock boundary

- Severity: **High**
- Files:
  - `src/window_control.rs:472`
  - `src/window_control.rs:476`
  - `src/window_control.rs:479`
  - `src/window_control.rs:483`
  - `src/window_control.rs:853`

#### Evidence

- `get_cached_window` copies raw pointer out of mutex (`src/window_control.rs:472-477`).
- `clear_window_cache` can release cached pointers (`src/window_control.rs:479-486`).
- Callers then use returned pointer after lock is dropped (`src/window_control.rs:853+`).

#### Risk

Potential dangling pointer/UAF if cache clear/rebuild interleaves with pointer use.

#### Recommended Fix

1. Return a retained reference from `get_cached_window` (retain under lock).
2. Require caller-side release, or return RAII-owned wrapper instead of raw pointer.
3. Convert cache type from `HashMap<u32, usize>` to ownership-aware wrapper type.

---

### MSA-HIGH-004: `NSWindow*` registry uses raw address + blanket `Send`/`Sync`

- Severity: **High**
- Files:
  - `src/window_manager.rs:95`
  - `src/window_manager.rs:117`
  - `src/window_manager.rs:119`
  - `src/window_manager.rs:193`
  - `src/window_manager.rs:209`

#### Evidence

- `WindowId(usize)` stores raw `NSWindow*` address (`src/window_manager.rs:95-110`).
- `unsafe impl Send/Sync for WindowId` without lifetime/liveness tracking (`src/window_manager.rs:117-119`).
- Registry returns raw pointers from shared map (`src/window_manager.rs:193-215`).

#### Risk

If a window is destroyed/replaced, stale addresses may be dereferenced by downstream code.

#### Recommended Fix

1. Replace raw pointer registry with liveness-checked handle strategy.
2. Keep main-thread-only access invariant explicit (type-level or API-level).
3. Remove/avoid blanket `Send/Sync` unless invariants are enforced by construction.

---

### MSA-MED-005: AVFoundation capture setup lacks null/alloc hardening

- Severity: **Medium**
- Files:
  - `src/camera.rs:82`
  - `src/camera.rs:83`
  - `src/camera.rs:148`
  - `src/camera.rs:160`
  - `src/camera.rs:161`
  - `src/camera.rs:38`

#### Evidence

- `alloc/init` results are not null-checked for session/delegate before use (`src/camera.rs:82-83`, `src/camera.rs:160-161`).
- `dispatch_queue_create` result is not null-checked (`src/camera.rs:148`).
- `CaptureHandle` is `Send` and dropped on arbitrary thread (`src/camera.rs:38`) with raw ObjC/GCD pointers.

#### Risk

Mostly robustness today, but null returns or violated thread assumptions could become memory safety faults.

#### Recommended Fix

1. Add explicit null checks after `alloc/init` and `dispatch_queue_create`.
2. Centralize early-failure cleanup path (structured error contexts).
3. Document/enforce drop thread assumptions for `CaptureHandle`.

#### Notes

- Existing teardown order is good: stop session -> `dispatch_sync_f` drain -> null ivar -> reclaim sender -> release objects/queue (`src/camera.rs:40-66`).

---

### MSA-MED-006: NSWorkspace observer lifetime has no teardown path

- Severity: **Medium**
- Files:
  - `src/frontmost_app_tracker.rs:88`
  - `src/frontmost_app_tracker.rs:100`
  - `src/frontmost_app_tracker.rs:361`
  - `src/frontmost_app_tracker.rs:374`
  - `src/frontmost_app_tracker.rs:385`

#### Evidence

- Observer allocated/initialized and registered (`src/frontmost_app_tracker.rs:361-378`).
- No matching `removeObserver`/release; run loop runs forever (`src/frontmost_app_tracker.rs:385`).
- Startup is one-shot guarded by atomic (`src/frontmost_app_tracker.rs:88-92`), so leak is bounded.

#### Risk

Bounded but persistent observer lifetime leak; harder shutdown correctness.

#### Recommended Fix

1. Store observer + center refs in tracker state.
2. Add explicit stop function to remove observer and release resources.
3. Tie lifecycle to app shutdown or tracker restart logic.

---

### MSA-LOW-007: `from_raw_parts` length arithmetic should be checked

- Severity: **Low**
- Files:
  - `src/app_impl.rs:4302`
  - `src/app_impl.rs:4312`
  - `src/app_impl.rs:4314`

#### Evidence

- Slice lengths use multiplication without checked arithmetic:
  - `y_stride * height`
  - `uv_stride * uv_height`

#### Risk

The source values are usually sane from CoreVideo, but unchecked multiplication is a classic bounds hazard.

#### Recommended Fix

Use checked/saturating arithmetic and explicit error on overflow before `from_raw_parts`.

---

### MSA-LOW-008: GCD trampoline closure can leak on abnormal termination

- Severity: **Low**
- Files:
  - `src/hotkeys.rs:806`
  - `src/hotkeys.rs:810`
  - `src/hotkeys.rs:831`

#### Evidence

- Closure is boxed and reclaimed in trampoline (`Box::into_raw` -> `Box::from_raw`).
- If queued work never executes (e.g., abrupt process termination), the box leaks for process lifetime.

#### Risk

Process-lifetime leak only; no persistent leak across restarts.

#### Recommended Fix

Acceptable as-is for this pattern; optional mitigation is bounded task queue accounting and diagnostics.

---

### MSA-LOW-009: ObjC IMP swizzle uses `transmute`

- Severity: **Low**
- Files:
  - `src/platform.rs:1031`
  - `src/platform.rs:1036`

#### Evidence

- `method_setImplementation` casts function pointer via `transmute`.

#### Risk

FFI signature mismatch would be UB. Current signature appears aligned, but this deserves regression coverage.

#### Recommended Fix

1. Keep function signature tightly scoped and documented.
2. Add runtime smoke assertion/diagnostics around swizzle success and invocation counts.
3. Re-audit after GPUI/ObjC runtime updates.

## Areas Reviewed with No Immediate Memory-Safety Defect Found

- `src/menu_bar.rs`: CF releases generally balanced in scanned traversal paths.
- `src/ocr.rs`: CoreFoundation/Vision objects appear released on success/error paths reviewed.
- `src/clipboard_history/open_with.rs`: `wrap_under_create_rule` ownership usage appears correct.
- `src/main.rs` signal handling path: atomic-only handler pattern appears signal-safe.

## Priority Fix Order

1. `MSA-CRIT-001` (menu executor UAF)
2. `MSA-HIGH-003` (cache pointer lifetime race)
3. `MSA-HIGH-002` (cache overwrite leak)
4. `MSA-HIGH-004` (window registry pointer model)
5. `MSA-MED-005` (camera hardening)
6. `MSA-MED-006` (observer teardown)
7. Low-severity hardening items

## Suggested Follow-up Tests

- `test_menu_executor_retains_menu_item_before_children_release`
- `test_window_cache_releases_previous_pointer_on_overwrite`
- `test_window_cache_get_returns_retained_owned_ref`
- `test_camera_start_capture_fails_cleanly_when_queue_create_returns_null`
- `test_frontmost_tracker_unregisters_workspace_observer_on_shutdown`

## Known Gaps

- Some risks are concurrency/lifecycle dependent and are hard to prove without a macOS runtime harness and AX/UI integration tests.
- This audit focused on memory safety and ownership; it did not attempt functional behavior changes.
