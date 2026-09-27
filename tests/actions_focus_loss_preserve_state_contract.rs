//! Source-audit contract for desktop click-away from the main ScriptList: the
//! passive hide keeps state and is not delayed or gated on staged actions.

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
