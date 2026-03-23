//! Source audits for the Mini Main Window builtin and reset behavior.

use super::read_source as read;

#[test]
fn mini_main_window_builtin_uses_shared_helper() {
    let content = read("src/app_execute/builtin_execution.rs");

    let branch_start = content
        .find("UtilityCommandType::MiniMainWindow => {")
        .expect("Expected UtilityCommandType::MiniMainWindow builtin arm");
    let branch = &content[branch_start..content.len().min(branch_start + 500)];

    for expected in [
        "category = \"BUILTIN\"",
        "trace_id = %dctx.trace_id",
        "\"Opening Mini Main Window\"",
        "self.open_mini_main_window(cx);",
        "Self::builtin_success(dctx, \"open_mini_main_window\")",
    ] {
        assert!(
            branch.contains(expected),
            "Mini Main Window builtin branch missing required line: {expected}"
        );
    }
}

#[test]
fn open_mini_main_window_sets_mini_mode_contract() {
    let content = read("src/app_execute/builtin_execution.rs");

    let fn_start = content
        .find("fn open_mini_main_window(&mut self, cx: &mut Context<Self>) {")
        .expect("Expected open_mini_main_window helper");
    let body = &content[fn_start..content.len().min(fn_start + 1200)];

    for expected in [
        "self.filter_text.clear();",
        "self.computed_filter_text.clear();",
        "self.pending_filter_sync = true;",
        "self.pending_placeholder = Some(\"Search scripts, apps, and commands…\".to_string());",
        "self.current_view = AppView::ScriptList;",
        "self.main_window_mode = MainWindowMode::Mini;",
        "self.hovered_index = None;",
        "self.selected_index = 0;",
        "self.opened_from_main_menu = true;",
        "self.invalidate_grouped_cache();",
        "self.sync_list_state();",
        "let (grouped_items, _) = self.get_grouped_results_cached();",
        "let item_count = grouped_items.len();",
        "resize_to_view_sync(ViewType::MiniMainWindow, item_count);",
        "self.pending_focus = Some(FocusTarget::MainFilter);",
        "self.focused_input = FocusedInput::MainFilter;",
        "cx.notify();",
    ] {
        assert!(
            body.contains(expected),
            "open_mini_main_window missing required line: {expected}"
        );
    }
}

#[test]
fn lifecycle_resets_restore_full_main_window_mode_on_close_and_go_back() {
    let lifecycle = read("src/app_impl/lifecycle_reset.rs");
    let close_start = lifecycle
        .find("fn close_and_reset_window(")
        .expect("Expected close_and_reset_window helper");
    let close_body = &lifecycle[close_start..lifecycle.len().min(close_start + 900)];
    assert!(
        close_body.contains("self.main_window_mode = MainWindowMode::Full;"),
        "close_and_reset_window must restore MainWindowMode::Full"
    );

    let go_back_start = lifecycle
        .find("fn go_back_or_close(")
        .expect("Expected go_back_or_close helper");
    let go_back_body = &lifecycle[go_back_start..lifecycle.len().min(go_back_start + 1200)];
    assert!(
        go_back_body.contains("self.current_view = AppView::ScriptList;"),
        "go_back_or_close must return to ScriptList in the opened_from_main_menu branch"
    );
    assert!(
        go_back_body.contains("self.main_window_mode = MainWindowMode::Full;"),
        "go_back_or_close must restore MainWindowMode::Full when returning to ScriptList"
    );
}
