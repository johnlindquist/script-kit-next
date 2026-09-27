//! Source-level contract for the Run 2 Pass #34
//! `tool-setfilter-active-view-routing` user story.
//!
//! Pass #14 recorded sub-gap (2) of `empty-clipboard-state`: the
//! `SetFilter` stdin command's receiver — `set_filter_text_immediate`
//! in `src/app_impl/filter_input_updates.rs` — updated only
//! `self.filter_text` + the main-menu ranker path, never writing the
//! text into the active subview's variant `filter` field. The
//! user-facing consequence: stdin-driven `setFilter "xyz"` on
//! `ClipboardHistoryView` / `EmojiPickerView` / `AppLauncherView` /
//! `WindowSwitcherView` (and siblings) silently left the subview's
//! filter stale, so `getState.visibleChoiceCount` — computed in
//! `prompt_handler/mod.rs` from the *variant's* `filter` field
//! (see the `ClipboardHistoryView` arm around line 2120) — could
//! never drop to zero on a filter-miss, making the `empty-clipboard-state`
//! story's corrected acceptance clause (Pass #33's `visibleChoiceCount=0`)
//! structurally unreachable even after Pass #33 pinned the protocol field.
//!
//! The complementary path — UI text-change events via
//! `handle_filter_input_change` — already routes correctly through
//! per-variant match arms that call `sync_builtin_query_state` on the
//! subview's `filter` + `selected_index`. The asymmetry between the two
//! entry points is exactly what sub-gap (2) documented.
//!
//! Pass #34 fixes this by introducing
//! `ScriptListApp::write_filter_to_current_subview(&mut self, text: &str) -> bool`
//! and wiring it into `set_filter_text_immediate` so stdin-driven filter
//! writes land on the active subview's own field. A return of `true`
//! also suppresses the ScriptList-only fallback-mode branch, since
//! `get_filtered_results_cached` and `collect_fallbacks` both key on
//! the script list and would incorrectly flip a subview into fallback
//! rendering.
//!
//! This contract test pins the invariants at source level so a future
//! mechanical refactor cannot silently regress the routing or erase
//! the fallback-mode gate.

const FILTER_INPUT_UPDATES: &str = include_str!("../src/app_impl/filter_input_updates.rs");

#[test]
fn set_filter_text_immediate_delegates_to_subview_router_helper() {
    // The stdin-driven receiver MUST call `write_filter_to_current_subview`
    // before kicking off ScriptList-specific reconcile + fallback work.
    // Dropping this call would immediately reintroduce the Pass #14
    // sub-gap: stdin `setFilter` on a subview would go back to updating
    // only `self.filter_text`, leaving the variant's own `filter` field
    // stale and `getState.visibleChoiceCount` stuck on the pre-filter
    // dataset size.
    let receiver_pos = FILTER_INPUT_UPDATES
        .find("fn set_filter_text_immediate(")
        .expect(
            "src/app_impl/filter_input_updates.rs `set_filter_text_immediate` \
             receiver must exist — this is the single choke-point the \
             stdin `SetFilter` command routes through across all three \
             embedded dispatchers.",
        );
    // Scope the search to the receiver body so a stray usage elsewhere
    // cannot satisfy the contract accidentally.
    let body = &FILTER_INPUT_UPDATES[receiver_pos..];
    let body_end = body
        .find("\n    pub(crate) fn write_filter_to_current_subview(")
        .or_else(|| body.find("\n    pub(crate) fn clear_filter("))
        .unwrap_or(body.len());
    let receiver_body = &body[..body_end];

    assert!(
        receiver_body.contains("self.write_filter_to_current_subview(&text)"),
        "src/app_impl/filter_input_updates.rs `set_filter_text_immediate` must call \
         `self.write_filter_to_current_subview(&text)` so stdin-driven `setFilter` \
         writes to the active subview's variant `filter` field. Removing this \
         call regresses Pass #14 sub-gap (2) of `empty-clipboard-state`: \
         `getState.visibleChoiceCount` would never drop to zero on a \
         filter-miss because the variant's `filter` field stays stale."
    );

    assert!(
        receiver_body.contains("let handled_by_subview ="),
        "src/app_impl/filter_input_updates.rs `set_filter_text_immediate` must \
         bind the subview-router result into a `handled_by_subview` flag so \
         the ScriptList-only fallback-mode block below can gate on it. \
         Discarding the return value lets `collect_fallbacks` run on a \
         builtin subview, incorrectly flipping the subview into \
         script-list fallback rendering."
    );

    let compact_receiver_body: String = receiver_body.split_whitespace().collect();
    assert!(
        compact_receiver_body.contains("if!handled_by_subview&&!text.is_empty()"),
        "src/app_impl/filter_input_updates.rs `set_filter_text_immediate` must \
         guard its fallback-mode branch with `!handled_by_subview && !text.is_empty()`. \
         `get_filtered_results_cached` and `collect_fallbacks` are ScriptList-only; \
         running them on a builtin subview would cache stale script results and \
         activate the main-menu fallback state against the wrong surface."
    );
}

#[test]
fn set_filter_at_stays_in_shared_spine_route() {
    let receiver_pos = FILTER_INPUT_UPDATES
        .find("fn set_filter_text_immediate(")
        .expect("set_filter_text_immediate receiver must exist");
    let body = &FILTER_INPUT_UPDATES[receiver_pos..];
    let body_end = body
        .find("\n    pub(crate) fn write_filter_to_current_subview(")
        .or_else(|| body.find("\n    pub(crate) fn clear_filter("))
        .unwrap_or(body.len());
    let receiver_body = &body[..body_end];

    for needle in [
        "Self::special_entry_from_script_list_filter(&text)",
        "self.route_script_list_special_entry(entry, &text, window, cx)",
        "return;",
    ] {
        assert!(
            receiver_body.contains(needle),
            "stdin/devtools setFilter \"@\" must delegate ScriptList special entries to the shared router: {needle}"
        );
    }
}
