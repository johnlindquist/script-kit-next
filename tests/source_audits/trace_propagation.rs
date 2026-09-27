//! Source audit tests verifying trace_id propagation through the action dispatch chain.
//!
//! These tests ensure that trace context is threaded from the top-level dispatch
//! through sub-handlers and SDK action calls so that logs can be correlated.

use super::read_source as read;

// ---------------------------------------------------------------------------
// trigger_sdk_action accepts trace_id parameter
// ---------------------------------------------------------------------------

#[test]
fn trigger_sdk_action_accepts_trace_id_parameter() {
    let content = read("src/action_helpers.rs");

    let fn_start = content
        .find("pub fn trigger_sdk_action(")
        .expect("Expected trigger_sdk_action function");
    let signature = &content[fn_start..content.len().min(fn_start + 400)];

    assert!(
        signature.contains("trace_id: &str"),
        "trigger_sdk_action must accept trace_id as a parameter"
    );
}

#[test]
fn trigger_sdk_action_logs_trace_id_in_all_paths() {
    let content = read("src/action_helpers.rs");

    let fn_start = content
        .find("pub fn trigger_sdk_action(")
        .expect("Expected trigger_sdk_action function");
    // The function body extends to the next top-level function
    let fn_end = content[fn_start + 10..]
        .find("\npub ")
        .map(|p| fn_start + 10 + p)
        .unwrap_or(content.len());
    let fn_body = &content[fn_start..fn_end];

    // Every tracing call in the function should include trace_id
    let tracing_calls: Vec<_> = fn_body.match_indices("tracing::").collect();
    assert!(
        !tracing_calls.is_empty(),
        "trigger_sdk_action must have tracing calls"
    );

    for (pos, _) in &tracing_calls {
        let line_end = fn_body[*pos..].find('\n').unwrap_or(fn_body.len() - *pos);
        let line = &fn_body[*pos..*pos + line_end];
        // Multi-line tracing macros: check a window around the call
        let window_end = fn_body.len().min(*pos + 200);
        let window = &fn_body[*pos..window_end];
        assert!(
            window.contains("trace_id"),
            "tracing call in trigger_sdk_action must include trace_id: {line}"
        );
    }
}

// ---------------------------------------------------------------------------
// trigger_sdk_action_with_trace threads trace_id to trigger_sdk_action
// ---------------------------------------------------------------------------

#[test]
fn trigger_sdk_action_with_trace_exists_in_sdk_actions() {
    let content = read("src/app_actions/sdk_actions.rs");

    assert!(
        content.contains("fn trigger_sdk_action_with_trace("),
        "Expected trigger_sdk_action_with_trace function in sdk_actions.rs"
    );
}

#[test]
fn trigger_sdk_action_with_trace_accepts_trace_id() {
    let content = read("src/app_actions/sdk_actions.rs");

    let fn_start = content
        .find("fn trigger_sdk_action_with_trace(")
        .expect("Expected function");
    let signature = &content[fn_start..content.len().min(fn_start + 300)];

    assert!(
        signature.contains("trace_id: &str"),
        "trigger_sdk_action_with_trace must accept trace_id parameter"
    );
}

#[test]
fn trigger_sdk_action_with_trace_forwards_trace_id_to_trigger_sdk_action() {
    let content = read("src/app_actions/sdk_actions.rs");

    let fn_start = content
        .find("fn trigger_sdk_action_with_trace(")
        .expect("Expected function");
    let fn_end = content[fn_start + 10..]
        .find("\n    ///")
        .map(|p| fn_start + 10 + p)
        .unwrap_or(content.len());
    let fn_body = &content[fn_start..fn_end];

    // Must pass trace_id to trigger_sdk_action
    assert!(
        fn_body.contains("trace_id,"),
        "trigger_sdk_action_with_trace must forward trace_id to trigger_sdk_action"
    );
}

#[test]
fn trigger_sdk_action_with_trace_uses_from_sdk_with_trace() {
    let content = read("src/app_actions/sdk_actions.rs");

    let fn_start = content
        .find("fn trigger_sdk_action_with_trace(")
        .expect("Expected function");
    let fn_end = content[fn_start + 10..]
        .find("\n    ///")
        .map(|p| fn_start + 10 + p)
        .unwrap_or(content.len());
    let fn_body = &content[fn_start..fn_end];

    assert!(
        fn_body.contains("from_sdk_with_trace("),
        "trigger_sdk_action_with_trace must use DispatchOutcome::from_sdk_with_trace"
    );
}

// ---------------------------------------------------------------------------
// handle_action dispatch uses dctx.trace_id for SDK fallback
// ---------------------------------------------------------------------------

#[test]
fn handle_action_sdk_fallback_uses_trace_id_from_dctx() {
    let content = read("src/app_actions/handle_action/mod.rs");

    assert!(
        content.contains("trigger_sdk_action_with_trace("),
        "handle_action SDK fallback must use trigger_sdk_action_with_trace"
    );
    assert!(
        content.contains("&dctx.trace_id"),
        "handle_action SDK fallback must pass &dctx.trace_id"
    );
}

// ---------------------------------------------------------------------------
// dispatch_system_action accepts DispatchContext and logs trace_id
// ---------------------------------------------------------------------------

#[test]
fn dispatch_system_action_accepts_dispatch_context() {
    let content = read("src/app_execute/builtin_execution.rs");

    let fn_start = content
        .find("fn dispatch_system_action(")
        .expect("Expected dispatch_system_action function");
    let signature = &content[fn_start..content.len().min(fn_start + 300)];

    assert!(
        signature.contains("dctx: &crate::action_helpers::DispatchContext"),
        "dispatch_system_action must accept &DispatchContext"
    );
}

#[test]
fn dispatch_system_action_logs_trace_id() {
    let content = read("src/app_execute/builtin_execution.rs");

    let fn_start = content
        .find("fn dispatch_system_action(")
        .expect("Expected dispatch_system_action function");
    let fn_body = &content[fn_start..content.len().min(fn_start + 3000)];

    assert!(
        fn_body.contains("trace_id ="),
        "dispatch_system_action must log trace_id as a structured field"
    );
}

#[test]
fn dispatch_system_action_emits_dispatched_status() {
    let content = read("src/app_execute/builtin_execution.rs");

    let fn_start = content
        .find("fn dispatch_system_action(")
        .expect("Expected dispatch_system_action function");
    let fn_body = &content[fn_start..content.len().min(fn_start + 800)];

    assert!(
        fn_body.contains(r#"status = "dispatched""#),
        "dispatch_system_action must emit status = \"dispatched\" before executing"
    );
}

// ---------------------------------------------------------------------------
// handle_system_action_result accepts DispatchContext and logs trace_id
// ---------------------------------------------------------------------------

#[test]
fn handle_system_action_result_accepts_dispatch_context() {
    let content = read("src/app_execute/builtin_execution.rs");

    let fn_start = content
        .find("fn handle_system_action_result(")
        .expect("Expected handle_system_action_result function");
    let signature = &content[fn_start..content.len().min(fn_start + 400)];

    assert!(
        signature.contains("dctx: &crate::action_helpers::DispatchContext"),
        "handle_system_action_result must accept &DispatchContext"
    );
}

#[test]
fn handle_system_action_result_logs_trace_id_on_success() {
    let content = read("src/app_execute/builtin_execution.rs");

    let fn_start = content
        .find("fn handle_system_action_result(")
        .expect("Expected handle_system_action_result function");
    let fn_body = &content[fn_start..content.len().min(fn_start + 2500)];

    // The success path must include trace_id
    let success_start = fn_body
        .find(r#"status = "success""#)
        .expect("Expected success status in handle_system_action_result");
    let success_window =
        &fn_body[success_start.saturating_sub(200)..fn_body.len().min(success_start + 200)];

    assert!(
        success_window.contains("trace_id ="),
        "handle_system_action_result success path must log trace_id"
    );
}

#[test]
fn handle_system_action_result_logs_trace_id_on_error() {
    let content = read("src/app_execute/builtin_execution.rs");

    let fn_start = content
        .find("fn handle_system_action_result(")
        .expect("Expected handle_system_action_result function");
    let fn_body = &content[fn_start..content.len().min(fn_start + 2500)];

    // The error path must include trace_id
    let error_start = fn_body
        .find(r#"status = "error""#)
        .expect("Expected error status in handle_system_action_result");
    let error_window =
        &fn_body[error_start.saturating_sub(200)..fn_body.len().min(error_start + 200)];

    assert!(
        error_window.contains("trace_id ="),
        "handle_system_action_result error path must log trace_id"
    );
}

// ---------------------------------------------------------------------------
// handle_builtin_confirmation propagates trace_id via DispatchContext
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Confirmation spawned task propagates trace context via dctx
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Async paths use intermediate status, not terminal completion
// ---------------------------------------------------------------------------

#[test]
fn execute_builtin_with_query_holds_instead_of_completing() {
    let content = read("src/app_execute/builtin_execution.rs");

    let confirm_check = content
        .find("self.config.requires_confirmation(&entry.id)")
        .expect("Expected requires_confirmation gate");
    let after_check = &content[confirm_check..content.len().min(confirm_check + 4000)];

    // The held path records that it waits; the run happens when the hold's wait runs out.
    assert!(
        after_check.contains("self.one_window_hold(")
            && after_check.contains(r#""held_before_it_runs""#),
        "a built-in that used to ask first must wait in the line, not complete at once"
    );
}

// ---------------------------------------------------------------------------
// DispatchOutcome carries trace_id field
// ---------------------------------------------------------------------------

#[test]
fn dispatch_outcome_has_trace_id_field() {
    let content = read("src/action_helpers.rs");

    let struct_start = content
        .find("pub struct DispatchOutcome")
        .expect("Expected DispatchOutcome struct");
    let struct_body = &content[struct_start..content.len().min(struct_start + 800)];

    assert!(
        struct_body.contains("pub trace_id: Option<String>"),
        "DispatchOutcome must have a trace_id field"
    );
}

#[test]
fn dispatch_outcome_from_sdk_with_trace_exists() {
    let content = read("src/action_helpers.rs");

    assert!(
        content.contains("pub fn from_sdk_with_trace("),
        "DispatchOutcome must have from_sdk_with_trace constructor"
    );
}

#[test]
fn dispatch_outcome_with_trace_id_builder_exists() {
    let content = read("src/action_helpers.rs");

    assert!(
        content.contains("pub fn with_trace_id("),
        "DispatchOutcome must have with_trace_id builder method"
    );
}
