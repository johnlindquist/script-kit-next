//! Source audits for the main-window builtin and reset behavior.

use super::read_source as read;

#[test]
fn mini_main_window_builtin_uses_shared_helper() {
    // The action enum lives in an `include!`d sibling of builtin_execution.rs.
    let content = read("src/app_execute/builtin_execution.rs")
        + &read("src/app_execute/builtin_execution_utility_actions.rs");

    for expected in [
        "enum UtilityOpenBuiltinAction",
        "UtilityOpenBuiltinAction::MainWindow => self.open_main_window(cx)",
        "Self::MainWindow => Some(\"Opening Main Window\")",
        "Self::MainWindow => \"open_main_window\"",
        "Self::builtin_success(dctx, action.success_detail())",
    ] {
        assert!(
            content.contains(expected),
            "Mini Main Window builtin action state missing required line: {expected}"
        );
    }
}

#[test]
fn open_main_window_sets_mini_mode_contract() {
    let content = read("src/app_execute/builtin_execution.rs");

    let body = super::function_body(&content, "open_main_window")
        .expect("Expected open_main_window helper");

    for expected in [
        "self.filter_text.clear();",
        "self.computed_filter_text.clear();",
        "self.pending_filter_sync = true;",
        "self.pending_placeholder = Some(\"Search scripts, apps, and commands…\".to_string());",
        "self.show_script_list_with_main_filter_focus();",
        "self.set_main_window_mode_state_only(MainWindowMode::Mini, cx, \"open_main_window\");",
        "self.hovered_index = None;",
        "self.selected_index = 0;",
        "self.opened_from_main_menu = true;",
        "self.invalidate_grouped_cache();",
        "self.sync_list_state();",
        "let (grouped_items, _) = self.get_grouped_results_cached();",
        "let item_count = grouped_items.len();",
        // Skip section headers so selected_index points to a real item
        "self.selected_index = first_selectable;",
        "resize_to_view_sync(ViewType::MainWindow, item_count);",
        "cx.notify();",
    ] {
        assert!(
            body.contains(expected),
            "open_main_window missing required line: {expected}"
        );
    }
}

#[test]
fn width_for_view_returns_main_window_width_for_main_window_views() {
    let content = read("src/window_resize/mod.rs");
    assert!(
        content.contains("ViewType::MainWindow")
            && content.contains("ViewType::MiniPrompt")
            && content.contains("ViewType::MiniAiChat")
            && content.contains("Some(MAIN_WINDOW_WIDTH)"),
        "width_for_view must return MAIN_WINDOW_WIDTH for main-window-sized views"
    );
    assert!(
        content.contains("ViewType::ScriptList => Some(MAIN_WINDOW_WIDTH)"),
        "width_for_view must return MAIN_WINDOW_WIDTH for ScriptList"
    );
}

#[test]
fn resize_to_view_sync_uses_width_aware_path() {
    let content = read("src/window_resize/mod.rs");
    let fn_start = content
        .find("pub fn resize_to_view_sync(")
        .expect("Expected resize_to_view_sync function");
    let body = &content[fn_start..content.len().min(fn_start + 1400)];
    assert!(
        body.contains("width_for_view(view_type)"),
        "resize_to_view_sync must call width_for_view"
    );
    assert!(
        body.contains("resize_first_window_to_size(target_height, target_width)"),
        "resize_to_view_sync must call resize_first_window_to_size when width is Some"
    );
    assert!(
        body.contains("main_window sizing selected"),
        "resize_to_view_sync must emit a structured sizing trace for MainWindow"
    );
}

#[test]
fn lifecycle_resets_restore_full_main_window_mode_on_close_and_go_back() {
    let lifecycle = read("src/app_impl/lifecycle_reset.rs");
    let close_body = super::function_body(&lifecycle, "prepare_main_window_close")
        .expect("Expected shared main-window close preparation helper");
    assert!(
        close_body.contains("if normalize_mode_before_hide")
            && close_body.contains(
                "self.set_main_window_mode_state_only(MainWindowMode::Full, cx, reason);"
            ),
        "close_and_reset_window must restore MainWindowMode::Full"
    );

    let go_back_start = lifecycle
        .find("fn go_back_or_close(")
        .expect("Expected go_back_or_close helper");
    let go_back_body = &lifecycle[go_back_start..lifecycle.len().min(go_back_start + 1200)];
    assert!(
        go_back_body.contains("self.reset_to_script_list(cx);"),
        "go_back_or_close must return to ScriptList in the opened_from_main_menu branch"
    );
    assert!(
        go_back_body.contains("self.opened_from_main_menu = false;"),
        "go_back_or_close must clear opened_from_main_menu when returning to ScriptList"
    );
}

/// Escape ordering (popup -> filter -> launch origin -> dismiss) is decided by
/// the pure `plan_launcher_escape`, covered by unit tests in
/// `window_orchestrator::interaction`. Both keyboard paths must route through
/// that single plan so they cannot drift apart again.
#[test]
fn simulate_key_escape_delegates_to_go_back_when_opened_from_main_menu() {
    let simulate = read("src/app_impl/simulate_key_dispatch.rs");
    assert!(
        simulate.contains("view.apply_shared_launcher_escape(\"simulate_key\", window, ctx)"),
        "SimulateKey Escape must route through the shared launcher escape plan"
    );
    let script_list = read("src/render_script_list/mod.rs");
    let decision = super::function_body(&script_list, "launcher_escape_decision")
        .expect("launcher_escape_decision must build the shared plan facts");
    assert!(
        decision.contains("plan_launcher_escape(")
            && decision.contains("return_to_origin: self.opened_from_main_menu"),
        "the shared plan must receive the launch-origin fact"
    );
    let apply = super::function_body(&script_list, "apply_shared_launcher_escape")
        .expect("apply_shared_launcher_escape must exist");
    assert!(
        apply.contains("LauncherEscapeDecision::ReturnToOrigin => {")
            && apply.contains("self.go_back_or_close(window, cx);"),
        "ReturnToOrigin must go back through go_back_or_close"
    );
}

#[test]
fn physical_script_list_escape_delegates_to_go_back_when_opened_from_main_menu() {
    let source = read("src/render_script_list/mod.rs");
    assert!(
        source.contains("this.apply_shared_launcher_escape(\"physical_bubble\", window, cx)"),
        "physical ScriptList Escape must route through the shared launcher escape plan"
    );
}
