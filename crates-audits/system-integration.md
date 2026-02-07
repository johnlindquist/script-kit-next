# System Integration Crates Audit

## Scope
- Repository: `script-kit-gpui`
- Audit date: 2026-02-07
- Requested crates: `tray-icon` (`0.21`), `global-hotkey` (`0.7`), `smappservice-rs` (`0.1`)
- Files reviewed:
  - `Cargo.toml`, `Cargo.lock`
  - `src/tray.rs`, `src/main.rs`, `src/hotkeys.rs`, `src/app_impl.rs`, `src/login_item.rs`, `src/config/types.rs`
  - Upstream crate sources in `~/.cargo/registry/src/...`

## Dependency Baseline
- Declared:
  - `global-hotkey = "0.7"` (`Cargo.toml:28`)
  - `tray-icon = "0.21"` (`Cargo.toml:66`)
  - `smappservice-rs = "0.1"` (`Cargo.toml:130`)
- Resolved:
  - `global-hotkey 0.7.0` (`Cargo.lock:3052`)
  - `smappservice-rs 0.1.3` (`Cargo.lock:7139`)
  - `tray-icon 0.21.2` (`Cargo.lock:8026`)

## Direct Answers

### 1) Is `tray-icon` menu handling correct?
**Mostly yes, with one state-sync gap.**

What is correct:
- Menu action IDs are stable and type-safe (`TrayMenuAction::id`/`from_id`) (`src/tray.rs:141`, `src/tray.rs:158`).
- Menu items are created with explicit IDs and routed through pure conversion (`src/tray.rs:304`, `src/tray.rs:447`).
- Event bridge uses a dedicated blocking receiver thread and async channel handoff (`src/main.rs:2451`, `src/main.rs:2468`).
- Side effects for launch-at-login are centralized (`src/tray.rs:458`, `src/main.rs:2472`).

Gap:
- Launch-at-login checkmark refresh is only called after toggle; there is no call when menu opens or on periodic re-sync (`src/tray.rs:474`, `src/main.rs:2473`).
- Result: if status changes externally in System Settings while app is running, checkmark can drift until next app toggle/restart.

### 2) Is `global-hotkey` registering/unregistering hotkeys properly (no leaks)?
**Partially. Core built-in/script paths are solid; dynamic shortcut updates can leak.**

What is correct:
- Built-in hotkeys use transactional rebind (register new before unregister old) (`src/hotkeys.rs:240`, `src/hotkeys.rs:269`, `src/hotkeys.rs:306`).
- Script hotkeys support explicit unregister and diff-based updates (`src/hotkeys.rs:469`, `src/hotkeys.rs:504`, `src/app_impl.rs:2005`, `src/app_impl.rs:2027`).
- Upstream crate cleanup: `global-hotkey` unregisters registered keys in `Drop` on macOS (`.../global-hotkey-0.7.0/src/platform_impl/macos/mod.rs:290`).

Leak/staleness risks:
- **High:** Dynamic shortcut replacement does not remove existing shortcut for same `command_id` before adding new one (`src/hotkeys.rs:642`).
  - `app_impl` recorder save path calls register directly (`src/app_impl.rs:5191`) and does not call `unregister_dynamic_shortcut` first.
  - Because `script_paths` map is overwritten, older route/hotkey can become orphaned and remain active.
- **Medium:** `update_hotkeys` rebinds optional notes/AI/logs hotkeys only when `Some(...)`; it does not unregister prior hotkey when config changes to `None` (`src/hotkeys.rs:357`, `src/hotkeys.rs:366`, `src/hotkeys.rs:374`).
  - `get_ai_hotkey`/`get_logs_hotkey` can return `None` when disabled (`src/config/types.rs:748`, `src/config/types.rs:762`).
  - Old registrations can remain active after disabling.
- **Low:** `unregister_dynamic_shortcut` docs say no-op success if missing, but implementation returns error (`src/hotkeys.rs:701`, `src/hotkeys.rs:720`).

### 3) Is `smappservice-rs` launch-at-login working on modern macOS?
**Basic integration is correct for macOS 13+, but UX/compatibility handling is incomplete.**

What is correct:
- Uses `AppService::new(ServiceType::MainApp)` + `register`/`unregister`/`status` (`src/login_item.rs:33`, `src/login_item.rs:72`, `src/login_item.rs:106`).
- `smappservice-rs` exposes modern status model including approval-required state (`.../smappservice-rs-0.1.3/src/lib.rs:111`, `.../smappservice-rs-0.1.3/src/lib.rs:124`).

Gaps:
- App logic treats only `Enabled` as true and collapses `RequiresApproval` into "disabled" (`src/login_item.rs:110`).
- `open_login_items_settings()` exists but is never used in toggle flow (`src/login_item.rs:127`).
- `toggle_login_item()` does not branch on approval-required state (`src/login_item.rs:148`).
- Bundle metadata claims macOS minimum `10.15` (`Cargo.toml:174`) while SMAppService is a modern API (macOS 13+ in project docs), so older-runtime behavior needs explicit guard/strategy.

### 4) Any thread safety issues?
**No obvious data races; synchronization primitives are used consistently.**

Observed:
- Hotkey routing table guarded by `parking_lot::RwLock` (`src/hotkeys.rs:149`).
- Manager access serialized by `Mutex` (`src/hotkeys.rs:157`).
- Hotkey event dispatch uses channels and non-blocking sends for UI paths (`src/hotkeys.rs:1331`, `src/hotkeys.rs:1403`).
- Tray events are bridged through dedicated thread + channel (`src/main.rs:2451`).

Caveat:
- Main issues are logical lifecycle leaks/stale registrations (above), not low-level thread unsafety.

## Findings (Severity Ordered)

1. **High: Dynamic shortcut rebind leak for same `command_id`**
- Location: `src/hotkeys.rs:642`, `src/app_impl.rs:5191`
- Impact: Old hotkey can remain registered and active after changing shortcut in recorder flow.
- Why: New registration path does not remove old route/hotkey for same command prior to insert.

2. **Medium: Optional built-in hotkeys not unregistered when disabled/removed**
- Location: `src/hotkeys.rs:357`, `src/hotkeys.rs:366`, `src/hotkeys.rs:374`, `src/config/types.rs:748`, `src/config/types.rs:762`
- Impact: Notes/AI/logs hotkeys can continue firing after config disables them.

3. **Medium: Launch-at-login approval state not handled in UX flow**
- Location: `src/login_item.rs:102`, `src/login_item.rs:148`, `src/login_item.rs:127`
- Impact: Approval-required states are not surfaced clearly; toggle behavior can be confusing.

4. **Low: Tray launch-at-login checkmark can become stale**
- Location: `src/tray.rs:474`, `src/main.rs:2473`
- Impact: Menu state may not reflect external changes until next toggle/restart.

5. **Low: `unregister_dynamic_shortcut` contract mismatch**
- Location: `src/hotkeys.rs:701`, `src/hotkeys.rs:720`
- Impact: Callers expecting no-op semantics may get unexpected errors.

6. **Low: OS-version compatibility gap for login-item API path**
- Location: `Cargo.toml:174`, `src/login_item.rs:27`
- Impact: Requires explicit strategy if app is expected to run on macOS versions below SMAppService availability.

## Recommendations
1. Add transactional command-id rebind helper for dynamic shortcuts:
- Resolve existing route by `command_id`.
- Register new hotkey first.
- On success, unregister/remove previous hotkey entry.

2. In `update_hotkeys`, explicitly unregister notes/AI/logs when getter returns `None`.

3. Align `unregister_dynamic_shortcut` behavior with docs (true no-op when missing), or update docs to match current strict behavior.

4. Extend login-item state handling to distinguish:
- `Enabled`
- `RequiresApproval` (show guidance + open System Settings)
- `NotRegistered`/`NotFound`

5. Refresh launch-at-login checkmark when tray menu is about to open, not only after toggle.

6. Add explicit runtime guard/feature policy for SMAppService vs app minimum macOS target.

## Verification Performed
- Static audit and cross-check against upstream crate behavior:
  - `global-hotkey` receiver and drop cleanup (`.../global-hotkey-0.7.0/src/lib.rs:102`, `.../platform_impl/macos/mod.rs:290`)
  - `muda` channel receiver semantics (`.../muda-0.17.1/src/lib.rs:456`, `.../muda-0.17.1/src/lib.rs:470`)
  - `smappservice-rs` statuses and settings helper (`.../smappservice-rs-0.1.3/src/lib.rs:111`, `.../smappservice-rs-0.1.3/src/lib.rs:437`, `.../smappservice-rs-0.1.3/src/lib.rs:445`)
- No runtime behavior changes were made in this task; report-only deliverable.

## Bottom Line
- `tray-icon` integration is structurally sound, with minor state refresh gap.
- `global-hotkey` core model is good, but dynamic shortcut updates currently allow stale registrations.
- `smappservice-rs` integration works at a basic level, but approval flow and OS-compat policy need tightening for robust modern macOS behavior.
