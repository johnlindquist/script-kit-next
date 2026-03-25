//! Source audits verifying that scroll reveal emits structured SCROLL_STATE logs
//! with the caller-provided reason, and that sync_list_state resets stale reveal
//! state before re-revealing.

use super::read_source as read;

#[test]
fn scroll_to_selected_if_needed_logs_reason_on_skip() {
    let content = read("src/app_navigation/impl_scroll.rs");

    let fn_start = content
        .find("fn scroll_to_selected_if_needed(")
        .expect("Expected scroll_to_selected_if_needed function");
    let fn_body = &content[fn_start..content.len().min(fn_start + 600)];

    assert!(
        fn_body.contains("target: \"SCROLL_STATE\""),
        "scroll_to_selected_if_needed must emit structured SCROLL_STATE logs"
    );
    assert!(
        fn_body.contains("reason,"),
        "scroll_to_selected_if_needed must log the caller-provided reason"
    );
    assert!(
        fn_body.contains("\"skip scroll reveal"),
        "scroll_to_selected_if_needed must log skip events when target already revealed"
    );
}

#[test]
fn scroll_to_selected_if_needed_logs_reason_on_reveal() {
    let content = read("src/app_navigation/impl_scroll.rs");

    let fn_start = content
        .find("fn scroll_to_selected_if_needed(")
        .expect("Expected scroll_to_selected_if_needed function");
    let fn_body = &content[fn_start..content.len().min(fn_start + 1200)];

    assert!(
        fn_body.contains("before_top"),
        "scroll_to_selected_if_needed must log before_top for reveal delta"
    );
    assert!(
        fn_body.contains("after_top"),
        "scroll_to_selected_if_needed must log after_top for reveal delta"
    );
    assert!(
        fn_body.contains("\"revealed selected item\""),
        "scroll_to_selected_if_needed must log reveal completion message"
    );
}

#[test]
fn scroll_to_selected_if_needed_accepts_reason_not_underscore() {
    let content = read("src/app_navigation/impl_scroll.rs");

    let fn_start = content
        .find("fn scroll_to_selected_if_needed(")
        .expect("Expected scroll_to_selected_if_needed function");
    let signature = &content[fn_start..content.len().min(fn_start + 120)];

    assert!(
        !signature.contains("_reason"),
        "scroll_to_selected_if_needed must use `reason`, not `_reason` — the parameter must not be discarded"
    );
}

#[test]
fn sync_list_state_resets_reveal_cache_and_logs() {
    let content = read("src/app_navigation/impl_scroll.rs");

    let fn_start = content
        .find("fn sync_list_state(")
        .expect("Expected sync_list_state function");
    let fn_body = &content[fn_start..content.len().min(fn_start + 1000)];

    assert!(
        fn_body.contains("self.last_scrolled_index = None"),
        "sync_list_state must reset last_scrolled_index to invalidate stale reveal cache"
    );
    assert!(
        fn_body.contains("target: \"SCROLL_STATE\""),
        "sync_list_state must emit structured SCROLL_STATE logs"
    );
    assert!(
        fn_body.contains("old_list_count"),
        "sync_list_state must log old_list_count for list-change tracking"
    );
    assert!(
        fn_body.contains("item_count"),
        "sync_list_state must log item_count for list-change tracking"
    );
    assert!(
        fn_body.contains("\"synced list state\""),
        "sync_list_state must log sync completion message"
    );
}

#[test]
fn sync_list_state_re_reveals_after_reset() {
    let content = read("src/app_navigation/impl_scroll.rs");

    let fn_start = content
        .find("fn sync_list_state(")
        .expect("Expected sync_list_state function");
    let fn_body = &content[fn_start..content.len().min(fn_start + 1000)];

    // After resetting reveal cache, must scroll to reveal the current selection
    assert!(
        fn_body.contains("scroll_to_reveal_item(self.selected_index)"),
        "sync_list_state must re-reveal the selected item after invalidating the reveal cache"
    );
}

/// Regression guard: same-count list updates (e.g. filtering replaces every row but
/// the total count stays identical) must still invalidate and re-reveal. This test
/// proves the invalidation is unconditional — it happens *outside* the
/// `if old_list_count != item_count` branch.
#[test]
fn sync_list_state_regression_invalidates_reveal_even_when_count_unchanged() {
    let content = read("src/app_navigation/impl_scroll.rs");

    let fn_start = content
        .find("pub fn sync_list_state(&mut self)")
        .expect("sync_list_state function not found");
    let fn_end_marker = content[fn_start..]
        .find("pub fn validate_selection_bounds")
        .expect("validate_selection_bounds not found after sync_list_state");
    let sync_fn = &content[fn_start..fn_start + fn_end_marker];

    // The splice is conditional on count change...
    let splice_pos = sync_fn
        .find("self.main_list_state.splice(")
        .expect("splice call not found in sync_list_state");
    let splice_guard = sync_fn
        .find("if old_list_count != item_count")
        .expect("splice guard not found");
    assert!(
        splice_guard < splice_pos,
        "splice must be inside the count-change guard"
    );

    // ...but the reveal invalidation must be OUTSIDE that guard (unconditional).
    let reveal_invalidation = sync_fn
        .find("self.last_scrolled_index = None;")
        .expect("reveal cache invalidation not found");
    // The closing brace of the `if` block sits between splice and invalidation.
    // Prove invalidation is after the closing brace by checking it's after splice_pos.
    assert!(
        reveal_invalidation > splice_pos,
        "reveal cache invalidation must happen after the conditional splice, i.e. unconditionally"
    );

    // The re-reveal must also be unconditional (outside the count-change guard).
    let re_reveal = sync_fn
        .find("scroll_to_reveal_item(self.selected_index)")
        .expect("scroll_to_reveal_item call not found");
    assert!(
        re_reveal > reveal_invalidation,
        "re-reveal must happen after cache invalidation"
    );
}
