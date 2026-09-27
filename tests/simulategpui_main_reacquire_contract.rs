//! Source-level contract test for the Run 2 Pass #30
//! `tool-simulategpui-main-reacquire-contract` user story.
//!
//! Run 2 Pass #5 (commit `92c8d66cb`) fixed the handle-staleness gap
//! that blocked Pass #2's `clipboard-to-agent_chat-paste` story: stdin
//! commands run inside `window_for_stdin.update(...)`, so any
//! subsequent `handle.update()` on the Main handle returns Err because
//! reentrancy-protection trips (`cx.windows.get_mut(id)?.take()?`).
//! `get_valid_runtime_window_handle` misread this as staleness and
//! evicted; the role-based fallback then failed because
//! `register_window(WindowRole::Main, ...)` is never invoked.
//! Dispatch collapsed to `handle_unavailable` on every attempt.
//!
//! Two parts of that fix still hold in `dispatch_gpui_event`: when a
//! Main target has neither a runtime nor a role handle, the target
//! resolves to the durable `MAIN_WINDOW_HANDLE` global via
//! `crate::get_main_window_handle()`, and every dispatch is deferred
//! with `cx.defer` so the stdin owner's window borrow unwinds before
//! `handle.update` runs. The deferred body dispatches through the same
//! `apply_simulated_event` helper as every other path.

const SIMULATOR: &str = include_str!("../src/platform/gpui_event_simulator.rs");

fn dispatch_gpui_event_body() -> &'static str {
    let start = SIMULATOR
        .find("pub(crate) fn dispatch_gpui_event(")
        .expect("dispatch_gpui_event must exist");
    let rest = &SIMULATOR[start..];
    let end = rest
        .find("\n/// Production protocol response owner")
        .expect("dispatch_gpui_event must be followed by handle_gpui_event_message");
    &rest[..end]
}

#[test]
fn main_kind_branch_reacquires_from_global_handle() {
    // The global-handle fallback is gated on the Main kind so attached
    // popups and other windows keep resolving through their own
    // registered handles and the coordinate-rebasing step for mouse events.
    let compact: String = SIMULATOR.split_whitespace().collect();
    assert!(
        compact.contains(
            "(dispatch_info.kind==AutomationWindowKind::Main).then(crate::get_main_window_handle)"
        ),
        "src/platform/gpui_event_simulator.rs must fall back to \
         `crate::get_main_window_handle()` only when the dispatch target is \
         `AutomationWindowKind::Main`. The role registry is never populated \
         for Main, so the durable global is the single source of truth for a \
         handle that survives reentrancy."
    );
    assert!(
        compact.contains("(dispatch_info.kind==crate::protocol::AutomationWindowKind::Main).then(crate::get_main_window_handle)"),
        "src/platform/gpui_event_simulator.rs must re-resolve the same \
         Main-only global fallback when validating a queued dispatch, so a \
         replaced main window fails as `stale_target` instead of receiving \
         the event."
    );
}

#[test]
fn dispatch_defers_until_the_outer_update_unwinds() {
    // Pass #2's blocker was a synchronous `handle.update` from inside the
    // stdin owner's `window.update`. Deferring every dispatch lets the outer
    // update unwind first, so the nested update no longer trips reentrancy.
    let body = dispatch_gpui_event_body();
    let defer = body
        .find("cx.defer(move |cx| {")
        .expect("dispatch_gpui_event must defer its dispatch via `cx.defer`");
    let update = body
        .find("resolved.handle.update(cx, |_, window, cx| {")
        .expect("the deferred dispatch must update the resolved handle");
    let apply = body
        .find("apply_simulated_event(window, cx, &event);")
        .expect("the deferred dispatch must call apply_simulated_event");
    assert!(
        defer < update && update < apply,
        "src/platform/gpui_event_simulator.rs must update the resolved handle \
         and apply the event inside the `cx.defer` callback; a synchronous \
         update would re-hit the reentrancy guard and collapse back to \
         `handle_unavailable` — the exact Pass #2 blocker."
    );
    assert!(
        body.contains("state.was_deferred.set(true);"),
        "src/platform/gpui_event_simulator.rs must report `wasDeferred` for \
         deferred dispatches so receipts can tell them from rejected targets."
    );
}

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
