//! Source audits for builtin dispatch context and structured outcome adoption.
//!
//! Ensures that `execute_builtin_with_query` uses `DispatchContext::for_builtin`,
//! that `dispatch_system_action` returns `DispatchOutcome`, and that the
//! `log_builtin_outcome` helper emits all required structured fields.

use super::read_source as read;

#[test]
fn builtin_execution_uses_dispatch_context_for_builtin_surface() {
    let content = read("src/app_execute/builtin_execution.rs");

    assert!(
        content.contains("DispatchContext::for_builtin("),
        "builtin_execution.rs must create DispatchContext::for_builtin(...)"
    );

    assert!(
        !content.contains("let trace_id = uuid::Uuid::new_v4().to_string();"),
        "builtin_execution.rs should not mint raw builtin trace ids directly"
    );
}

#[test]
fn builtin_execution_has_structured_outcome_logger() {
    let content = read("src/app_execute/builtin_execution.rs");

    let fn_start = content
        .find("fn log_builtin_outcome(")
        .expect("Expected log_builtin_outcome helper in builtin_execution.rs");
    let body = &content[fn_start..content.len().min(fn_start + 900)];

    for field in [
        "builtin_id =",
        "trace_id =",
        "surface =",
        "handler",
        "status =",
        "error_code =",
        "duration_ms",
    ] {
        assert!(
            body.contains(field),
            "log_builtin_outcome must include structured field `{field}`"
        );
    }
}

#[test]
fn dispatch_system_action_returns_dispatch_outcome() {
    let content = read("src/app_execute/builtin_execution.rs");

    let fn_start = content
        .find("fn dispatch_system_action(")
        .expect("Expected dispatch_system_action");
    let signature = &content[fn_start..content.len().min(fn_start + 260)];

    assert!(
        signature.contains("-> crate::action_helpers::DispatchOutcome"),
        "dispatch_system_action should return DispatchOutcome"
    );
}

#[test]
fn dispatch_system_action_accepts_dispatch_context() {
    let content = read("src/app_execute/builtin_execution.rs");

    let fn_start = content
        .find("fn dispatch_system_action(")
        .expect("Expected dispatch_system_action");
    let signature = &content[fn_start..content.len().min(fn_start + 260)];

    assert!(
        signature.contains("dctx: &crate::action_helpers::DispatchContext"),
        "dispatch_system_action must accept &DispatchContext"
    );
}

#[test]
fn handle_system_action_result_returns_dispatch_outcome() {
    let content = read("src/app_execute/builtin_execution.rs");

    let fn_start = content
        .find("fn handle_system_action_result(")
        .expect("Expected handle_system_action_result");
    let signature = &content[fn_start..content.len().min(fn_start + 400)];

    assert!(
        signature.contains("-> crate::action_helpers::DispatchOutcome"),
        "handle_system_action_result should return DispatchOutcome"
    );
}

#[test]
fn builtin_success_and_error_helpers_exist() {
    let content = read("src/app_execute/builtin_execution.rs");

    assert!(
        content.contains("fn builtin_success("),
        "Expected builtin_success helper in builtin_execution.rs"
    );
    assert!(
        content.contains("fn builtin_error("),
        "Expected builtin_error helper in builtin_execution.rs"
    );
}

#[test]
fn execute_builtin_with_query_logs_outcome_at_boundary() {
    let content = read("src/app_execute/builtin_execution.rs");

    let fn_start = content
        .find("fn execute_builtin_with_query(")
        .expect("Expected execute_builtin_with_query");
    let fn_body = &content[fn_start..content.len().min(fn_start + 4000)];

    assert!(
        fn_body.contains("Self::log_builtin_outcome("),
        "execute_builtin_with_query must log outcome at the dispatch boundary"
    );
}

// ---------------------------------------------------------------------------
// execute_builtin_inner — returns real DispatchOutcome
// ---------------------------------------------------------------------------

#[test]
fn execute_builtin_inner_returns_dispatch_outcome() {
    let content = read("src/app_execute/builtin_execution.rs");

    let fn_start = content
        .find("fn execute_builtin_inner(")
        .expect("Expected execute_builtin_inner");
    let signature = &content[fn_start..content.len().min(fn_start + 500)];

    assert!(
        signature.contains("-> crate::action_helpers::DispatchOutcome"),
        "execute_builtin_inner must return DispatchOutcome"
    );
}

#[test]
fn execute_builtin_with_query_uses_real_inner_outcome() {
    let content = read("src/app_execute/builtin_execution.rs");

    let fn_start = content
        .find("fn execute_builtin_with_query(")
        .expect("Expected execute_builtin_with_query");
    let fn_body = &content[fn_start..content.len().min(fn_start + 5000)];

    // Must NOT contain the old synthetic success pattern
    assert!(
        !fn_body.contains("Self::builtin_success(&dctx, \"execute_builtin_inner\")"),
        "execute_builtin_with_query must not log unconditional success for non-system builtins"
    );

    // Must use the real outcome returned by execute_builtin_inner
    assert!(
        fn_body.contains("self.execute_builtin_inner(entry, query_override, &dctx, cx)"),
        "execute_builtin_with_query must use the real outcome returned by execute_builtin_inner"
    );
}

// ---------------------------------------------------------------------------
// open_builtin_filterable_view — shared filterable view helper
// ---------------------------------------------------------------------------

#[test]
fn filterable_view_builtins_use_shared_helper() {
    let content = read("src/app_execute/builtin_execution.rs");

    assert!(
        content.contains("fn open_builtin_filterable_view("),
        "Expected shared open_builtin_filterable_view helper"
    );

    fn count_occurrences(haystack: &str, needle: &str) -> usize {
        haystack.matches(needle).count()
    }

    let use_count = count_occurrences(&content, "self.open_builtin_filterable_view(");
    assert!(
        use_count >= 5,
        "Expected at least 5 builtin arms to use open_builtin_filterable_view (found {use_count})"
    );
}

#[test]
fn open_builtin_filterable_view_sets_shared_focus_contract() {
    let content = read("src/app_execute/builtin_execution.rs");

    let fn_start = content
        .find("fn open_builtin_filterable_view(")
        .expect("Expected open_builtin_filterable_view");
    let body = &content[fn_start..content.len().min(fn_start + 1200)];

    for expected in [
        "self.filter_text.clear();",
        "self.pending_filter_sync = true;",
        "self.pending_placeholder = Some(",
        "self.current_view = view;",
        "self.hovered_index = None;",
        "self.opened_from_main_menu = true;",
        "resize_to_view_sync(ViewType::ScriptList, 0);",
        "self.pending_focus = Some(FocusTarget::MainFilter);",
        "self.focused_input = FocusedInput::MainFilter;",
        "cx.notify();",
    ] {
        assert!(
            body.contains(expected),
            "Missing required shared view-contract line: {expected}"
        );
    }
}

#[test]
fn filterable_view_builtins_are_silent_on_success() {
    let content = read("src/app_execute/builtin_execution.rs");

    for needle in [
        "builtins::BuiltInFeature::ClipboardHistory",
        "builtins::BuiltInFeature::Favorites",
        "builtins::BuiltInFeature::AppLauncher",
        "builtins::BuiltInFeature::WindowSwitcher",
        "builtins::BuiltInFeature::DesignGallery",
    ] {
        let start = content
            .find(needle)
            .unwrap_or_else(|| panic!("Missing match arm: {needle}"));
        let block = &content[start..content.len().min(start + 1600)];

        assert!(
            !block.contains("show_hud(") && !block.contains("Toast::"),
            "{needle} should stay silent on success; the view transition is the feedback"
        );
    }
}

#[test]
fn app_launch_failure_returns_error_outcome() {
    let content = read("src/app_execute/builtin_execution.rs");

    let app_branch = content
        .find("builtins::BuiltInFeature::App(app_name)")
        .expect("Expected App branch");
    let block = &content[app_branch..content.len().min(app_branch + 1200)];

    // Failed app launches must produce an error outcome
    assert!(
        block.contains("ERROR_LAUNCH_FAILED"),
        "Failed app launches must use ERROR_LAUNCH_FAILED error code"
    );
    assert!(
        block.contains("Self::builtin_error("),
        "Failed app launches must return builtin_error outcome"
    );
}

#[test]
fn window_switcher_failure_returns_error_outcome() {
    let content = read("src/app_execute/builtin_execution.rs");

    let ws_branch = content
        .find("builtins::BuiltInFeature::WindowSwitcher")
        .expect("Expected WindowSwitcher branch");
    let block = &content[ws_branch..content.len().min(ws_branch + 1600)];

    // list_windows failure must produce an error outcome
    assert!(
        block.contains("Self::builtin_error("),
        "Window switcher failure must return builtin_error outcome"
    );
    assert!(
        block.contains("open_window_switcher_failed"),
        "Window switcher failure detail must indicate what failed"
    );
}
