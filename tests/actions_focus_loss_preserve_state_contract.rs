//! Source-audit contract for desktop click-away while MainList actions are open.
//!
//! External actions popup close should hide the parent ScriptList with preserved
//! state, after closing the popup, without running explicit reset paths.

const ACTIONS_TOGGLE: &str = include_str!("../src/app_impl/actions_toggle.rs");
const LIFECYCLE_RESET: &str = include_str!("../src/app_impl/lifecycle_reset.rs");

fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"));
    let after_start = &source[start_index..];
    let end_index = after_start
        .find(end)
        .unwrap_or_else(|| panic!("missing end marker after {start}: {end}"));
    &after_start[..end_index]
}

// doc-anchor-removed: [[removed-docs Actions Contract]]
#[test]
fn actions_native_close_hides_mainlist_with_preserved_state() {
    let callback = source_between(
        ACTIONS_TOGGLE,
        "pub(crate) fn make_actions_window_on_close_callback(",
        "pub(crate) fn spawn_open_actions_window(",
    );

    for required in [
        "ActionsDialogHost::MainList",
        "app.can_preserve_hide_script_list_on_passive_focus_loss()",
        "!crate::platform::is_main_window_focused()",
        "app.mark_actions_popup_closed();",
        "app.pop_focus_overlay(cx);",
        "app.hide_main_window_preserving_state_for_focus_loss(cx);",
    ] {
        assert!(
            callback.contains(required),
            "actions close callback must contain `{required}`"
        );
    }
}

#[test]
fn actions_focus_loss_hide_orders_close_before_hide_and_skips_focus_restore() {
    let callback = source_between(
        ACTIONS_TOGGLE,
        "pub(crate) fn make_actions_window_on_close_callback(",
        "pub(crate) fn spawn_open_actions_window(",
    );

    let mark_closed = callback
        .find("app.mark_actions_popup_closed();")
        .expect("must mark actions closed");
    let hide = callback
        .find("app.hide_main_window_preserving_state_for_focus_loss(cx);")
        .expect("must hide main with preserved state");
    let early_return = callback[hide..]
        .find("return;")
        .map(|index| hide + index)
        .expect("preserve-hide branch must return early");
    let focus_restore = callback
        .find("app.request_focus_restore_for_actions_host(host);")
        .expect("normal close path must still restore focus");
    let hidden_skip = callback
        .find("!script_kit_gpui::is_main_window_visible()")
        .expect("must skip focus restoration if main is already hidden");

    assert!(
        mark_closed < hide,
        "actions must be closed before main hides"
    );
    assert!(
        hide < early_return && early_return < focus_restore,
        "preserve-hide branch must skip normal focus restoration"
    );
    assert!(
        early_return < hidden_skip && hidden_skip < focus_restore,
        "hidden-main guard must also skip normal focus restoration"
    );

    let preserve_branch = &callback[hide..early_return];
    assert!(!preserve_branch.contains("close_and_reset_window"));
    assert!(!preserve_branch.contains("reset_to_script_list"));
    assert!(!preserve_branch.contains("cancel_script_execution"));
}

#[test]
fn passive_scriptlist_hide_predicate_excludes_actions_open_guard() {
    let predicate = source_between(
        LIFECYCLE_RESET,
        "pub(crate) fn can_preserve_hide_script_list_on_passive_focus_loss",
        "pub(crate) fn hide_main_window_preserving_state_for_focus_loss",
    );

    assert!(predicate.contains("matches!(self.current_view, AppView::ScriptList)"));
    assert!(predicate.contains("script_kit_gpui::is_main_window_visible()"));
    assert!(predicate.contains("!self.is_pinned"));
    assert!(
        !predicate.contains("is_within_focus_grace_period"),
        "passive focus-loss hide should not be delayed by a grace window"
    );
    assert!(
        !predicate.contains("is_actions_window_open"),
        "the shared actions close callback decides whether actions should trigger the passive hide"
    );
}
