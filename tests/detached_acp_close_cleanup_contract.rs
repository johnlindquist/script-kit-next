//! Source-level contract for Run 2 Pass #47
//! `detached-popup-close-cleanup-contract`.
//!
//! Background: Run 2 Pass #29 (`tool-trigger-action-acpdetached-host`)
//! shipped a fix for a registry-leak bug: closing the detached ACP window
//! via the external `TriggerAction host=acpDetached action_id=acp_close`
//! path left a stale entry in the automation metadata registry, because
//! `close_chat_window(cx)` `take()`s the `CHAT_WINDOW.slot` BEFORE it calls
//! `window.remove_window()` — which means the window's own on_close
//! callback (registered in `chat_window_options`) later finds the slot
//! already drained (`g.take()` returns `None`) and the callback's
//! registry-cleanup branch never runs. The fix was to move the registry
//! cleanup inside `close_chat_window` itself so the slot-take sequencing
//! cannot hide it.
//!
//! The on_close callback ALSO has its own registry-cleanup branch for the
//! "user clicked the titlebar close" path (where the callback fires first
//! and finds the slot still populated). Both paths converge on the same
//! invariant: **`remove_runtime_window_handle(id)` and
//! `remove_automation_window(id)` must be called as an adjacent pair,
//! gated by the same `if let Some(ref id) = state.automation_id` guard,
//! and called BEFORE `window.remove_window()`** so that:
//!
//!   - `listAutomationWindows` cannot briefly report a window that is
//!     in the middle of being torn down (the registry is drained first);
//!   - the runtime handle registry and the metadata registry stay
//!     synchronized — a reader that uses the handle to reach into the
//!     window after a partial cleanup would get a stale handle pointing
//!     at a torn-down window.
//!
//! The bug Pass #29 fixed was:
//!   1. External caller invokes `close_chat_window(cx)`.
//!   2. Helper `take()`s CHAT_WINDOW.slot.
//!   3. Helper was calling `window.remove_window()` WITHOUT doing the
//!      registry cleanup itself.
//!   4. GPUI's close path fired on_close on the now-dead window.
//!   5. on_close tried `slot.lock().and_then(g.take())` — got `None`.
//!   6. Registry cleanup in on_close was gated on that `Some(state)`
//!      guard, so it never ran.
//!   7. `listAutomationWindows` reported the dead window forever.
//!
//! A mechanical refactor of `close_chat_window` (e.g. "simplify by
//! delegating to `window.remove_window()` and let on_close handle
//! cleanup") would silently reintroduce exactly this regression because
//! it's not visible from either path's local perspective — the bug is in
//! the INTERACTION between the two. This contract pins the cleanup pair
//! at BOTH sites simultaneously so the interaction cannot decay.
//!
//! Acceptance:
//!   1. `close_chat_window` contains the pair
//!      `remove_runtime_window_handle(id)` + `remove_automation_window(id)`
//!      as adjacent lines, INSIDE an `if let Some(ref id) =
//!      state.automation_id` guard, and the pair appears BEFORE any
//!      `window.remove_window()` call in the same function body.
//!   2. The on_close callback body (the one that appears higher in the
//!      file, registered on `chat_window_options`) ALSO contains the
//!      same adjacent pair inside an identical automation-id guard —
//!      pinning the "user closes via titlebar" path's cleanup too.
//!   3. The `save_window_from_gpui` call for `WindowRole::AcpChat`
//!      remains in `close_chat_window` so window-state persistence
//!      isn't silently lost by a refactor targeting only the registry
//!      cleanup half.
//!
//! Complements:
//!   - `tests/detached_acp_popup_registry_surface_contract.rs` (Pass #46,
//!     supplier-side: pins the single `upsert_automation_window` call
//!     shape and kind+surface parity).
//!   - `tests/trigger_action_acp_detached_host_contract.rs` (Pass #29,
//!     external dispatcher-side: pins the `host=acpDetached` routing
//!     that calls into `close_chat_window`).

const CHAT_WINDOW_RS: &str = include_str!("../src/ai/acp/chat_window.rs");

/// Extract the body of `pub fn close_chat_window(cx: &mut App)` — from
/// the fn header up to the first blank-line-separated next `pub fn` or
/// top-level item. The slice is intentionally wider than the fn body
/// because we assert on adjacent-line patterns; extra context doesn't
/// cause false positives, a truncated slice would.
fn close_chat_window_body(src: &str) -> &str {
    let start_marker = "pub fn close_chat_window(cx: &mut App) {";
    let start = src.find(start_marker).unwrap_or_else(|| {
        panic!(
            "src/ai/acp/chat_window.rs: could not locate \
             `pub fn close_chat_window(cx: &mut App) {{`. If this helper \
             has been renamed or had its signature changed, update the \
             contract — but confirm the external TriggerAction \
             `host=acpDetached action_id=acp_close` dispatch still \
             reaches whatever replaces it, or Pass #29's registry-leak \
             regression returns."
        )
    });
    // Find the next top-level `\npub fn ` or `\nfn ` after the opening
    // brace as a conservative end marker; fall back to end of file.
    let search_from = start + start_marker.len();
    let next_item_offset = src[search_from..]
        .find("\npub fn ")
        .or_else(|| src[search_from..].find("\nfn "))
        .unwrap_or(src.len() - search_from);
    &src[start..search_from + next_item_offset]
}

/// Count adjacent cleanup-pair occurrences across the whole source file.
/// The file must carry the pair in AT LEAST two distinct sites: one in
/// `close_chat_window` (external-caller path) and one inside the
/// `on_window_should_close` callback registered in
/// `open_chat_window_with_thread` (user-titlebar-close path). Both sites
/// are required because they handle two different entry orderings — see
/// the head-of-file comment.
fn count_adjacent_cleanup_pairs(src: &str) -> usize {
    let mut count = 0usize;
    let mut cursor = 0usize;
    while let Some(pos) = src[cursor..].find("remove_runtime_window_handle(id)") {
        let abs = cursor + pos;
        // Take a 160-byte window after the call and check if the second
        // call appears with only the permitted glue between them.
        let window_end = (abs + 160).min(src.len());
        let window = &src[abs..window_end];
        if find_adjacent_cleanup_pair(window).is_some() {
            count += 1;
        }
        cursor = abs + "remove_runtime_window_handle(id)".len();
    }
    count
}

fn find_adjacent_cleanup_pair(body: &str) -> Option<(usize, usize)> {
    // Look for `remove_runtime_window_handle(id)` followed closely
    // (within ~120 chars — one module-qualified call prefix + one line
    // of whitespace) by `remove_automation_window(id)`. The adjacency
    // window is intentionally tight: inserting any non-cleanup logic
    // between the two calls is the kind of refactor that silently drops
    // one half of the pair and reintroduces the Pass #29 leak.
    const RUNTIME_CALL: &str = "remove_runtime_window_handle(id)";
    let runtime = body.find(RUNTIME_CALL)?;
    let after_runtime = runtime + RUNTIME_CALL.len();
    let tail = &body[after_runtime..];
    let auto_rel = tail.find("remove_automation_window(id)")?;
    let between = &tail[..auto_rel];
    // The acceptable leftover after stripping whitespace/semicolons is
    // the trailing `)` of the first call plus an optional module-qualified
    // prefix for the second (`crate::windows::`). Anything else means a
    // refactor has wedged logic between the calls.
    let normalized: String = between
        .chars()
        .filter(|c| !c.is_whitespace() && *c != ';')
        .collect();
    if normalized.is_empty() || normalized == "crate::windows::" {
        Some((runtime, after_runtime + auto_rel))
    } else {
        None
    }
}

// doc-anchor-removed: [[removed-docs Chat#Detached window behavior]]
#[test]
fn close_chat_window_cleans_up_registry_before_remove_window() {
    let body = close_chat_window_body(CHAT_WINDOW_RS);

    // 1. Adjacent cleanup pair exists inside close_chat_window.
    let pair = find_adjacent_cleanup_pair(body).unwrap_or_else(|| {
        panic!(
            "src/ai/acp/chat_window.rs `close_chat_window` body must \
             contain the adjacent pair `remove_runtime_window_handle(id)` \
             + `remove_automation_window(id)` on consecutive lines. \
             Pass #29 added both calls specifically because the helper \
             `take()`s CHAT_WINDOW.slot BEFORE calling \
             `window.remove_window()`, which makes the window's on_close \
             callback find an empty slot and silently skip the registry \
             cleanup. Dropping either call reintroduces the \
             `listAutomationWindows` stale-entry leak. Full context: \n{body}"
        )
    });

    // 2. The pair is gated by the automation-id guard (defensive: some
    //    downstream refactor could drop the guard and pass an undefined
    //    `id`; without the guard the `id` binding wouldn't even compile,
    //    but pinning the guard pattern keeps the reasoning local).
    assert!(
        body.contains("if let Some(ref id) = state.automation_id"),
        "src/ai/acp/chat_window.rs `close_chat_window` must gate the \
         registry cleanup pair with `if let Some(ref id) = \
         state.automation_id`. Without the guard, a detached window that \
         was registered without an automation_id (e.g. a historical \
         regression path) would panic on id-binding; with the guard, the \
         cleanup silently skips and the registry stays consistent. Full \
         body:\n{body}"
    );

    // 3. The cleanup pair appears BEFORE `window.remove_window()` so the
    //    registry is drained before GPUI tears down the window and fires
    //    the on_close callback on a half-dead state.
    let remove_window = body.find("window.remove_window();").unwrap_or_else(|| {
        panic!(
            "src/ai/acp/chat_window.rs `close_chat_window` must call \
             `window.remove_window()` to actually tear down the OS \
             window. If this helper now only does cleanup without \
             removing the window, detached popups linger visually even \
             after automation thinks they're gone. Full body:\n{body}"
        )
    });
    assert!(
        pair.1 < remove_window,
        "src/ai/acp/chat_window.rs `close_chat_window` must call the \
         cleanup pair BEFORE `window.remove_window()`. Reordering would \
         re-expose the Pass #29 race: GPUI fires the on_close callback \
         during `remove_window`, and if the registry is still populated \
         at that instant, a concurrent reader can see a live registry \
         entry for a window that is mid-teardown. Found cleanup pair at \
         {}, remove_window() at {}.\nFull body:\n{body}",
        pair.1,
        remove_window
    );

    // 4. Window-state persistence stays in the helper — a refactor that
    //    "simplified" close_chat_window by moving the cleanup to a
    //    helper could accidentally drop the save_window_from_gpui call.
    assert!(
        body.contains("save_window_from_gpui") && body.contains("WindowRole::AcpChat"),
        "src/ai/acp/chat_window.rs `close_chat_window` must persist \
         window bounds via `save_window_from_gpui(WindowRole::AcpChat, \
         wb)` so the next open restores the user's last position. \
         Removing this call silently breaks the detached popup's \
         position memory. Full body:\n{body}"
    );
}

// doc-anchor-removed: [[removed-docs Chat#Detached window behavior]]
#[test]
fn both_close_paths_carry_the_cleanup_pair() {
    // The file must have ≥ 2 adjacent-cleanup-pair sites: one for the
    // external-caller path (`close_chat_window`, pinned by the first
    // test) and one for the user-titlebar-close path (the
    // `on_window_should_close` callback registered inside
    // `open_chat_window_with_thread`). Both are required because GPUI's
    // window-close fires `on_window_should_close` BEFORE the helper is
    // called by external callers — if only one site carries the
    // cleanup, the other path leaks registry entries. The first
    // `on_window_should_close` in the file (the placeholder window
    // before a thread is attached) deliberately does NOT carry the
    // pair because the automation window isn't registered until the
    // thread opens; that's why this test asserts ≥ 2, not a fixed N.
    let pairs = count_adjacent_cleanup_pairs(CHAT_WINDOW_RS);
    assert!(
        pairs >= 2,
        "src/ai/acp/chat_window.rs must carry the adjacent cleanup pair \
         `remove_runtime_window_handle(id)` + `remove_automation_window(id)` \
         in AT LEAST two sites (external `close_chat_window` + \
         user-titlebar `on_window_should_close`). Found {pairs}. If \
         only one site carries the pair, the other close path leaks the \
         automation registry entry — Pass #29's exact regression."
    );

    // Cross-check: every `remove_automation_window(id)` call in the file
    // must be paired with an adjacent `remove_runtime_window_handle(id)`
    // — the reverse pairing matters too (handle without registry would
    // leak the runtime-handle slot, symmetric leak).
    let automation_calls = CHAT_WINDOW_RS
        .matches("remove_automation_window(id)")
        .count();
    assert_eq!(
        pairs, automation_calls,
        "Every `remove_automation_window(id)` call in \
         src/ai/acp/chat_window.rs must be preceded by an adjacent \
         `remove_runtime_window_handle(id)`. Found {pairs} adjacent \
         pairs but {automation_calls} automation_window removes — the \
         mismatch means one removal has no partner, so half the \
         cleanup is missing on that path."
    );
}
