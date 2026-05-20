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

#[test]
fn expanded_main_window_is_width_only_not_taller_than_mini() {
    let resize = read("src/window_resize/mod.rs");
    let height_match = source_between(
        &resize,
        "fn height_for_view_with_layout(",
        "fn initial_window_height_with_layout(",
    );
    assert!(
        height_match.contains("ViewType::ExpandedMainWindow => height_for_expanded_main_window()"),
        "expanded list/detail surfaces must use the dedicated width-only height path"
    );
    assert!(
        resize.contains("pub(crate) fn height_for_expanded_main_window() -> Pixels")
            && resize.contains("px(MINI_MAIN_WINDOW_MAX_HEIGHT)"),
        "expanded list/detail height must remain locked to the mini main-window height"
    );

    let width_match = source_between(
        &resize,
        "pub fn width_for_view(view_type: ViewType)",
        "pub fn initial_window_height()",
    );
    assert!(
        width_match.contains(
            "ViewType::ScriptList | ViewType::ExpandedMainWindow => Some(FULL_MAIN_WINDOW_WIDTH)"
        ),
        "expanded list/detail surfaces may widen to full width"
    );
}

#[test]
fn file_search_full_presentation_uses_expanded_main_window_sizing() {
    let source = read("src/app_impl/filter_input_core.rs");
    let resize_for_presentation = source_between(
        &source,
        "pub(crate) fn resize_file_search_window_for_presentation(",
        "pub(crate) fn resize_file_search_window_after_results_change(",
    );
    assert!(
        resize_for_presentation.contains(
            "FileSearchPresentation::Full => resize_to_view_sync(ViewType::ExpandedMainWindow, 0)"
        ),
        "full Search Files must widen only through ExpandedMainWindow sizing"
    );

    let resize_after_results = source_between(
        &source,
        "pub(crate) fn resize_file_search_window_after_results_change(",
        "    /// Shared helper that opens file search",
    );
    assert!(
        resize_after_results.contains("resize_to_view_sync(ViewType::ExpandedMainWindow, 0);"),
        "file-search result updates must preserve the expanded width-only sizing"
    );
    assert!(
        !resize_for_presentation.contains("resize_to_view_sync(ViewType::ScriptList, 0)")
            && !resize_after_results.contains("resize_to_view_sync(ViewType::ScriptList, 0)"),
        "Search Files must not regress to taller ScriptList sizing"
    );
}

#[test]
fn minimal_and_expanded_scaffolds_share_header_geometry_tokens() {
    let source = read("src/components/prompt_layout_shell.rs");

    for (fn_name, end_marker) in [
        (
            "render_minimal_list_prompt_scaffold",
            "pub(crate) fn render_minimal_list_prompt_shell_with_footer",
        ),
        (
            "render_minimal_list_prompt_shell_with_footer",
            "pub(crate) fn render_expanded_view_scaffold",
        ),
        (
            "render_expanded_view_scaffold",
            "pub(crate) fn render_expanded_view_scaffold_with_hints",
        ),
        (
            "render_expanded_view_scaffold_with_hints",
            "pub(crate) fn render_expanded_view_scaffold_with_footer",
        ),
        (
            "render_expanded_view_scaffold_with_footer",
            "pub(crate) fn render_expanded_view_prompt_shell",
        ),
    ] {
        let body = source_between(&source, &format!("pub(crate) fn {fn_name}("), end_marker);
        for expected in [
            "HEADER_PADDING_X",
            "HEADER_PADDING_Y",
            "HEADER_BUTTON_HEIGHT",
            "render_header_divider()",
        ] {
            assert!(
                body.contains(expected),
                "{fn_name} must keep shared header/input geometry token `{expected}`"
            );
        }
    }
}

#[test]
fn shared_search_input_pins_inner_text_line_height_to_cursor_contract() {
    let source = read("src/render_builtins/common.rs");
    let body = source_between(
        &source,
        "pub(crate) fn render_search_input(&self)",
        "    /// Emit a structured scroll log line for builtin views.",
    );

    for expected in [
        "crate::panel::CURSOR_HEIGHT_LG + (crate::panel::CURSOR_MARGIN_Y * 2.0)",
        ".line_height(gpui::px(crate::panel::CURSOR_HEIGHT_LG))",
        ".with_size(gpui_component::Size::Size(gpui::px(self.theme_font_size_xl())))",
    ] {
        assert!(
            body.contains(expected),
            "shared search input must keep the header text baseline independent of expanded-view inheritance via `{expected}`"
        );
    }
}
