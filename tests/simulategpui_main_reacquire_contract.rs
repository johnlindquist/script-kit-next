//! Source-level contract test for the Run 2 Pass #30
//! `tool-simulategpui-main-reacquire-contract` user story.
//!
//! Run 2 Pass #5 (commit `92c8d66cb`) fixed the handle-staleness gap
//! that blocked Pass #2's `clipboard-to-acp-paste` story: stdin
//! commands run inside `window_for_stdin.update(...)`, so any
//! subsequent `handle.update()` on the Main handle returns Err because
//! reentrancy-protection trips (`cx.windows.get_mut(id)?.take()?`).
//! `get_valid_runtime_window_handle` misread this as staleness and
//! evicted; the role-based fallback then failed because
//! `register_window(WindowRole::Main, ...)` is never invoked.
//! Dispatch collapsed to `handle_unavailable` on every attempt.
//!
//! Pass #5's fix, between the exact-handle attempt and the role
//! fallback in `dispatch_gpui_event`, re-acquires the Main window
//! handle from the durable `MAIN_WINDOW_HANDLE` global via
//! `crate::get_main_window_handle()`. Probe with `handle.update`; on
//! Ok dispatch synchronously through `dispatch_with_any_handle` with
//! path `main_reacquire_global`. On Err (reentrancy), defer via
//! `cx.spawn(|cx: &mut AsyncApp| cx.update_window(handle, ...))` so
//! the outer update unwinds first, then report synchronous success
//! with path `main_deferred`. The deferred body emits
//! `main_deferred_complete` (Ok) or `main_deferred_failed` (Err) on
//! the next tick for off-live-verification receipts.
//!
//! Pass #30 live-verified this path end-to-end: after
//! `triggerBuiltin clipboardHistory` +
//! `show`, a `simulateGpuiEvent keyDown enter target=Main` returns
//! `{success:true, dispatchPath:"main_deferred"}` and the window
//! transitions to `windowVisible:false` on the next tick — the exact
//! behavior Pass #2's `handle_unavailable` used to block.
//!
//! This contract test pins the Pass #5 fix shape at source level so
//! a mechanical refactor of the simulator (e.g. renaming the dispatch
//! path labels, removing the global-handle re-acquisition, or
//! inlining `apply_simulated_event`) can't silently regress the
//! substrate gain behind the currently-open `clipboard-to-acp-paste`
//! story.

const SIMULATOR: &str = include_str!("../src/platform/gpui_event_simulator.rs");

// @lat: [[lat.md/automation#Automation#Window metadata]]
#[test]
fn main_kind_branch_reacquires_from_global_handle() {
    // The Main-kind branch is the exact seam Pass #5 introduced — it
    // sits between the exact-handle success path and the role-based
    // fallback. Without the `AutomationWindowKind::Main` gate, the
    // re-acquisition would fire for attached popups (which have their
    // own registered handles) and bypass coordinate rebasing for
    // mouse events.
    assert!(
        SIMULATOR.contains("matches!(resolved.kind, crate::protocol::AutomationWindowKind::Main)"),
        "src/platform/gpui_event_simulator.rs must gate the Main-handle \
         re-acquisition on `matches!(resolved.kind, \
         crate::protocol::AutomationWindowKind::Main)`. Without this \
         gate, attached popups (ActionsDialog, PromptPopup) would skip \
         their own registered-handle path and dispatch against Main, \
         silently bypassing the coordinate-rebasing step required for \
         mouse events."
    );
    assert!(
        SIMULATOR.contains("crate::get_main_window_handle()"),
        "src/platform/gpui_event_simulator.rs must call \
         `crate::get_main_window_handle()` to read the durable \
         `MAIN_WINDOW_HANDLE` global. The runtime handle registry only \
         upserts on show/hide transitions and the role registry is \
         never populated for Main, so the global is the single source \
         of truth for a handle that survives reentrancy."
    );
    assert!(
        SIMULATOR
            .contains("crate::windows::upsert_runtime_window_handle(resolved.id.clone(), handle)"),
        "src/platform/gpui_event_simulator.rs must upsert the \
         re-acquired handle into the runtime registry so subsequent \
         resolve_automation_window lookups (including from other \
         commands in the same tick) see the live handle instead of \
         re-running the re-acquisition path."
    );
}

// @lat: [[lat.md/automation#Automation#Window metadata]]
#[test]
fn ok_branch_dispatches_synchronously_with_main_reacquire_global_label() {
    // The Ok branch — reached when the outer update is NOT already
    // holding the Main handle — must dispatch synchronously and label
    // the path `main_reacquire_global` so receipts can distinguish
    // it from both the exact-handle success (`exact_handle`) and the
    // reentrancy-deferred path (`main_deferred`).
    assert!(
        SIMULATOR.contains("\"main_reacquire_global\""),
        "src/platform/gpui_event_simulator.rs must label the synchronous \
         post-re-acquisition dispatch path `\"main_reacquire_global\"`. \
         Audit log receipts (see Pass #5 and Pass #30 entries in \
         audits/afk/log.md) reference this exact string; renaming it \
         invalidates both the receipts and the audit-hygiene proofs."
    );
    assert!(
        SIMULATOR.contains("dispatch_with_any_handle("),
        "src/platform/gpui_event_simulator.rs must route the Ok branch \
         through `dispatch_with_any_handle` so it shares the same \
         error-shape and tracing instrumentation as the exact-handle \
         success path. Rolling an inline dispatch would bifurcate the \
         instrumentation surface."
    );
    assert!(
        SIMULATOR.contains("gpui_event_simulation.main_reacquired_global"),
        "src/platform/gpui_event_simulator.rs must emit \
         `tracing::info!(..., \"gpui_event_simulation.main_reacquired_global\")` \
         before dispatching on the Ok branch. The event is the primary \
         off-live-verification receipt that proves the re-acquisition \
         fired; losing it makes the substrate gain invisible to \
         `audits/afk/log.md` proofs."
    );
}

// @lat: [[lat.md/automation#Automation#Window metadata]]
#[test]
fn err_branch_defers_via_cx_spawn_and_reports_main_deferred() {
    // The Err branch is the Pass #2 blocker's actual fix: when the
    // outer stdin dispatcher is already inside `window_for_stdin.update`
    // for Main, `handle.update()` returns Err (reentrancy guard), and
    // we must defer via `cx.spawn` so the outer update can unwind
    // before the nested dispatch runs. Reporting synchronous
    // `success:true, dispatchPath:"main_deferred"` lets the caller
    // proceed immediately — which is what Pass #30 live-verified
    // against the clipboard-history view.
    assert!(
        SIMULATOR.contains("cx.spawn(async move |cx: &mut gpui::AsyncApp|"),
        "src/platform/gpui_event_simulator.rs must defer the Err-branch \
         dispatch via `cx.spawn(async move |cx: &mut gpui::AsyncApp| \
         ...)`. Any synchronous retry would re-hit the reentrancy \
         guard and collapse back to `handle_unavailable` — the exact \
         Pass #2 blocker."
    );
    assert!(
        SIMULATOR.contains("cx.update_window(handle, |_root, window, cx|"),
        "src/platform/gpui_event_simulator.rs deferred body must call \
         `cx.update_window(handle, |_root, window, cx| ...)` so the \
         dispatch runs against the live handle once the outer update \
         stack has unwound. Replacing it with `handle.update(cx, ...)` \
         inside the spawn would bypass the AsyncApp's window-resolution \
         path."
    );
    assert!(
        SIMULATOR.contains("apply_simulated_event(window, cx, &event_for_spawn)"),
        "src/platform/gpui_event_simulator.rs deferred body must call \
         `apply_simulated_event(window, cx, &event_for_spawn)` — the \
         same dispatch helper the synchronous paths call. Inlining the \
         keystroke/mouse dispatch would drift from the sync path and \
         cause the deferred dispatch to silently skip new event types."
    );
    assert!(
        SIMULATOR.contains("dispatch_path: Some(\"main_deferred\".to_string())"),
        "src/platform/gpui_event_simulator.rs Err branch must return \
         `dispatch_path: Some(\"main_deferred\".to_string())` \
         synchronously. Pass #30's live verification asserts this \
         exact string in the `simulateGpuiEventResult` response; \
         renaming it would invalidate the proof and break any replay \
         script that keys on the dispatch path."
    );
}

// @lat: [[lat.md/automation#Automation#Window metadata]]
#[test]
fn deferred_body_emits_complete_and_failed_tracing_events() {
    // The deferred body must emit matching success/failure tracing
    // events so the next-tick outcome is observable from
    // `audits/afk/log.md` receipts. Dropping either arm would make
    // failed deferred dispatches silent — a story-regression shape
    // that can't be caught by the synchronous response alone.
    assert!(
        SIMULATOR.contains("gpui_event_simulation.main_deferred_complete"),
        "src/platform/gpui_event_simulator.rs deferred body must emit \
         `tracing::info!(..., \"gpui_event_simulation.main_deferred_complete\")` \
         on the Ok arm of `cx.update_window`. Without this event, the \
         only proof the deferred dispatch actually ran is the \
         downstream state change (e.g., window hide) — which is not \
         uniquely attributable to the deferred path."
    );
    assert!(
        SIMULATOR.contains("gpui_event_simulation.main_deferred_failed"),
        "src/platform/gpui_event_simulator.rs deferred body must emit \
         `tracing::warn!(..., \"gpui_event_simulation.main_deferred_failed\")` \
         on the Err arm. Silent failure of the deferred dispatch would \
         look identical to success from the synchronous response \
         shape, masking reentrancy-recovery failures behind \
         `main_deferred + success:true`."
    );
    assert!(
        SIMULATOR.contains("gpui_event_simulation.main_deferred_scheduled"),
        "src/platform/gpui_event_simulator.rs must emit \
         `tracing::info!(..., \"gpui_event_simulation.main_deferred_scheduled\")` \
         after detaching the spawn, before returning the synchronous \
         `main_deferred` result. This event is the \
         synchronously-observable receipt that the spawn was queued — \
         required for end-to-end proof that the Err branch scheduled \
         work instead of silently dropping it."
    );
}

// @lat: [[lat.md/automation#Automation#Window metadata]]
#[test]
fn apply_simulated_event_helper_is_extracted_and_shared() {
    // `apply_simulated_event` must exist as a shared helper so the
    // synchronous paths (`exact_handle`, `main_reacquire_global`) and
    // the deferred path (`main_deferred`) all dispatch through the
    // same code. Inlining the dispatch at any one site would let that
    // path silently skip new event types when `SimulatedGpuiEvent`
    // grows a new variant.
    assert!(
        SIMULATOR.contains("fn apply_simulated_event("),
        "src/platform/gpui_event_simulator.rs must keep the \
         `apply_simulated_event` helper. Pass #5 extracted it \
         specifically so the deferred path couldn't drift from the \
         synchronous paths; re-inlining it would re-open the drift \
         risk."
    );
    // The synchronous call site (inside `dispatch_with_any_handle`'s
    // closure) must also use the helper — that's the contract that
    // binds sync and deferred paths to the same dispatch code.
    assert!(
        SIMULATOR.contains("apply_simulated_event(window, cx, event)")
            || SIMULATOR.contains("apply_simulated_event(window, cx, &event"),
        "src/platform/gpui_event_simulator.rs must call \
         `apply_simulated_event(window, cx, event)` from the \
         synchronous dispatch path as well as the deferred path. The \
         shared-helper contract is what prevents the two paths from \
         silently diverging on new `SimulatedGpuiEvent` variants."
    );
}
