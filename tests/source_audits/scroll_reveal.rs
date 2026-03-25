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
