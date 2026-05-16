//! Source audits for resize presentation contracts.
//!
//! Spawned async prompt resizes must prove the current view still matches the
//! prompt that scheduled the resize before calling raw platform resize helpers.

use super::read_source as read;

fn source_between<'a>(source: &'a str, start_marker: &str, end_marker: &str) -> &'a str {
    let start = source
        .find(start_marker)
        .unwrap_or_else(|| panic!("missing source marker: {start_marker}"));
    let tail = &source[start..];
    let end = tail
        .find(end_marker)
        .unwrap_or_else(|| panic!("missing end marker after {start_marker}: {end_marker}"));
    &tail[..end]
}

#[test]
fn prompt_spawned_term_resize_is_state_guarded() {
    // doc-anchor-removed: [[removed-docs Rules]]
    let source = read("src/prompt_handler/mod.rs");
    let block = source_between(
        &source,
        "PromptMessage::ShowTerm {",
        "PromptMessage::ShowEditor {",
    );

    assert!(
        !block.contains("cx.spawn(async move |_this, _cx|")
            && !block.contains("resize_to_view_sync(ViewType::TermPrompt, 0);"),
        "ShowTerm must not spawn an unconditional TermPrompt resize"
    );
    for expected in [
        "let expected_id = id.clone();",
        "AppView::TermPrompt { id, .. }",
        "id == &expected_id",
        "calculate_window_size_params_if_current_view(",
        "\"show_term_deferred_resize\"",
        "resize_to_view_sync(view_type, item_count);",
    ] {
        assert!(
            block.contains(expected),
            "ShowTerm deferred resize must contain state guard marker `{expected}`"
        );
    }
}

#[test]
fn prompt_spawned_editor_resize_is_state_guarded() {
    // doc-anchor-removed: [[removed-docs Rules]]
    let source = read("src/prompt_handler/mod.rs");
    let block = source_between(
        &source,
        "PromptMessage::ShowEditor {",
        "PromptMessage::ScriptExit =>",
    );

    assert!(
        !block.contains("cx.spawn(async move |_this, _cx|")
            && !block.contains("resize_to_view_sync(ViewType::EditorPrompt, 0);"),
        "ShowEditor must not spawn an unconditional EditorPrompt resize"
    );
    for expected in [
        "let expected_id = id.clone();",
        "AppView::EditorPrompt { id, .. }",
        "id == &expected_id",
        "calculate_window_size_params_if_current_view(",
        "\"show_editor_deferred_resize\"",
        "resize_to_view_sync(view_type, item_count);",
    ] {
        assert!(
            block.contains(expected),
            "ShowEditor deferred resize must contain state guard marker `{expected}`"
        );
    }
}

#[test]
fn resize_guard_helper_delegates_to_canonical_state_sizing() {
    // doc-anchor-removed: [[removed-docs Rules]]
    let source = read("src/app_impl/ui_window.rs");
    let block = source_between(
        &source,
        "pub(crate) fn calculate_window_size_params_if_current_view(",
        "    /// Returns the focused button when the active view is `ConfirmPrompt`.",
    );

    for expected in [
        "is_expected_view(&self.current_view)",
        "self.calculate_window_size_params()",
        "Skipping stale deferred resize for inactive view",
    ] {
        assert!(
            block.contains(expected),
            "resize guard helper must contain `{expected}`"
        );
    }

    for forbidden in [
        "resize_to_view_sync(",
        "defer_resize_to_view(",
        "ViewType::TermPrompt",
        "ViewType::EditorPrompt",
    ] {
        assert!(
            !block.contains(forbidden),
            "resize guard helper must not become another resize primitive via `{forbidden}`"
        );
    }
}
