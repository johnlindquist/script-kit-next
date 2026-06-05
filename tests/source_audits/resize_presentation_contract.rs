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
    let source = read("src/prompt_handler/mod.rs");
    for expected in [
        "PromptMessage::ShowTerm {",
        "let expected_id = id.clone();",
        "AppView::TermPrompt { id, .. }",
        "id == &expected_id",
        "calculate_window_size_params_if_current_view(",
        "\"show_term_deferred_resize\"",
        "resize_to_view_sync(view_type, item_count);",
    ] {
        assert!(
            source.contains(expected),
            "ShowTerm deferred resize must contain state guard marker `{expected}`"
        );
    }
    assert!(
        !source.contains("resize_to_view_sync(ViewType::TermPrompt, 0);"),
        "ShowTerm must not spawn an unconditional TermPrompt resize"
    );
}

#[test]
fn prompt_spawned_editor_resize_is_state_guarded() {
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
fn main_window_uses_standard_height_with_main_width() {
    let resize = read("src/window_resize/mod.rs");
    let height_match = source_between(
        &resize,
        "fn height_for_view_with_layout(",
        "fn initial_window_height_with_layout(",
    );
    assert!(
        height_match.contains("ViewType::MainWindow | ViewType::MiniAiChat"),
        "main-window-sized surfaces must use the consolidated MainWindow height path"
    );
    assert!(
        resize
            .contains("pub(crate) fn height_for_main_window(_sizing: MainWindowSizing) -> Pixels")
            && resize.contains("px(MAIN_WINDOW_MAX_HEIGHT)"),
        "main-window height must follow the fixed MainWindow sizing helper"
    );

    let width_match = source_between(
        &resize,
        "pub fn width_for_view(view_type: ViewType)",
        "pub fn initial_window_height()",
    );
    assert!(
        width_match.contains("ViewType::MainWindow")
            && width_match.contains("ViewType::ScriptList => Some(MAIN_WINDOW_WIDTH)"),
        "main-window and script-list surfaces must keep MAIN_WINDOW_WIDTH"
    );
}

#[test]
fn main_menu_show_uses_named_sizing_target_not_previous_prompt_bounds() {
    let resize = read("src/window_resize/mod.rs");
    assert!(
        resize.contains("pub(crate) struct MainMenuSizingTarget(pub MainWindowSizing)")
            && resize.contains("pub(crate) fn width(self) -> f32")
            && resize.contains("pub(crate) fn height(self) -> Pixels")
            && resize.contains("height_for_main_window(self.0)"),
        "main menu sizing must have an explicit main-window target wrapper"
    );

    let visibility = read("src/main_sections/window_visibility.rs");
    let show_bounds = source_between(
        &visibility,
        "let window_size = app_entity.update(cx, |view, ctx| {",
        "logging::log(\n        \"POSITION_TRACE\",",
    );
    assert!(
        show_bounds.contains("MainMenuSizingTarget(sizing)")
            && show_bounds.contains("MainMenuSizingTarget("),
        "show_main_window must compute ScriptList size from named main-menu targets"
    );
    assert!(
        show_bounds.find("MainMenuSizingTarget(")
            < show_bounds.find("view.calculate_window_size_params_with_app("),
        "ScriptList full main menu sizing must not fall through to generic prompt/list sizing"
    );
}

#[test]
fn reset_positions_uses_named_mini_main_menu_sizing_target() {
    let lifecycle = read("src/app_impl/lifecycle_reset.rs");
    let reset = source_between(
        &lifecycle,
        "pub(crate) fn reset_window_positions_to_default_main_menu(",
        "    pub(crate) fn cancel_script_execution(",
    );
    assert!(
        reset.contains("MainMenuSizingTarget(sizing)")
            && reset.contains("target.width()")
            && reset.contains("target.height()"),
        "default mini main-menu reset must use the named main menu sizing target"
    );
}

#[test]
fn file_search_full_presentation_uses_main_window_sizing() {
    let source = read("src/app_impl/filter_input_core.rs");
    let resize_for_presentation = source_between(
        &source,
        "pub(crate) fn resize_file_search_window_for_presentation(",
        "pub(crate) fn resize_file_search_window_after_results_change(",
    );
    assert!(
        resize_for_presentation.contains(
            "FileSearchPresentation::Full => resize_to_view_sync(ViewType::MainWindow, 0)"
        ),
        "full Search Files must use MainWindow sizing"
    );

    let resize_after_results = source_between(
        &source,
        "pub(crate) fn resize_file_search_window_after_results_change(",
        "    /// Shared helper that opens file search",
    );
    assert!(
        resize_after_results.contains("resize_to_view_sync(ViewType::MainWindow, 0);"),
        "file-search result updates must preserve MainWindow sizing"
    );
    assert!(
        !resize_for_presentation.contains("resize_to_view_sync(ViewType::ScriptList, 0)")
            && !resize_after_results.contains("resize_to_view_sync(ViewType::ScriptList, 0)"),
        "Search Files must not regress to ScriptList sizing"
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
        "let search = self.current_main_menu_theme.def().search;",
        ".h(gpui::px(search.height))",
        ".line_height(gpui::px(search.height))",
        ".with_size(gpui_component::Size::Size(gpui::px(input_font_size)))",
    ] {
        assert!(
            body.contains(expected),
            "shared search input must keep the header text baseline independent of expanded-view inheritance via `{expected}`"
        );
    }
}
