//! Source audit tests verifying structured logging at dispatch boundaries.
//!
//! These tests ensure that key dispatch functions emit structured tracing fields
//! (not free-form strings) so that logs are machine-parseable and observable.

use super::read_source as read;

// ---------------------------------------------------------------------------
// handle_system_action_result — action_type + status fields
// ---------------------------------------------------------------------------

#[test]
fn handle_system_action_result_logs_action_type_and_status_on_success() {
    let content = read("src/app_execute/builtin_execution.rs");

    let fn_start = content
        .find("fn handle_system_action_result(")
        .expect("Expected handle_system_action_result function");
    let fn_body = &content[fn_start..content.len().min(fn_start + 1500)];

    // Success path must use structured fields, not format strings
    assert!(
        fn_body.contains("action_type = ?action_type"),
        "handle_system_action_result success path must log action_type as a structured field"
    );
    assert!(
        fn_body.contains(r#"status = "success""#),
        "handle_system_action_result success path must log status = \"success\""
    );
}

#[test]
fn handle_system_action_result_logs_action_type_and_status_on_error() {
    let content = read("src/app_execute/builtin_execution.rs");

    let fn_start = content
        .find("fn handle_system_action_result(")
        .expect("Expected handle_system_action_result function");
    let fn_body = &content[fn_start..content.len().min(fn_start + 1500)];

    // Error path must also use structured fields
    assert!(
        fn_body.contains(r#"status = "error""#),
        "handle_system_action_result error path must log status = \"error\""
    );
    assert!(
        fn_body.contains("error_code ="),
        "handle_system_action_result error path must log an error_code field"
    );
}

#[test]
fn handle_system_action_result_uses_tracing_not_println() {
    let content = read("src/app_execute/builtin_execution.rs");

    let fn_start = content
        .find("fn handle_system_action_result(")
        .expect("Expected handle_system_action_result function");
    let fn_body = &content[fn_start..content.len().min(fn_start + 1500)];

    assert!(
        fn_body.contains("tracing::info!") || fn_body.contains("tracing::warn!"),
        "handle_system_action_result must use tracing::info or tracing::warn"
    );
    assert!(
        fn_body.contains("tracing::error!"),
        "handle_system_action_result error path must use tracing::error"
    );
}

// ---------------------------------------------------------------------------
// execute_builtin_inner — completion log with trace_id + duration_ms
// ---------------------------------------------------------------------------

#[test]
fn execute_builtin_inner_has_completion_log_with_trace_id() {
    let content = read("src/app_execute/builtin_execution.rs");

    let fn_start = content
        .find("fn execute_builtin_inner(")
        .expect("Expected execute_builtin_inner function");
    let fn_body = &content[fn_start..];

    assert!(
        fn_body.contains("trace_id = %trace_id"),
        "execute_builtin_inner must log trace_id as a structured field at completion"
    );
}

#[test]
fn execute_builtin_inner_has_completion_log_with_duration_ms() {
    let content = read("src/app_execute/builtin_execution.rs");

    let fn_start = content
        .find("fn execute_builtin_inner(")
        .expect("Expected execute_builtin_inner function");
    let fn_body = &content[fn_start..];

    assert!(
        fn_body.contains("duration_ms"),
        "execute_builtin_inner must log duration_ms as a structured field at completion"
    );
}

#[test]
fn execute_builtin_inner_completion_log_includes_builtin_id() {
    let content = read("src/app_execute/builtin_execution.rs");

    let fn_start = content
        .find("fn execute_builtin_inner(")
        .expect("Expected execute_builtin_inner function");
    let fn_body = &content[fn_start..];

    assert!(
        fn_body.contains("builtin_id ="),
        "execute_builtin_inner completion log must include builtin_id field"
    );
}

#[test]
fn execute_builtin_inner_accepts_trace_id_parameter() {
    let content = read("src/app_execute/builtin_execution.rs");

    let fn_start = content
        .find("fn execute_builtin_inner(")
        .expect("Expected execute_builtin_inner function");
    let signature = &content[fn_start..content.len().min(fn_start + 300)];

    assert!(
        signature.contains("trace_id: &str"),
        "execute_builtin_inner must accept trace_id as a parameter"
    );
}

// ---------------------------------------------------------------------------
// trigger_sdk_action_internal — status + error_code fields
// ---------------------------------------------------------------------------

#[test]
fn trigger_sdk_action_internal_logs_status_field() {
    let content = read("src/app_actions/sdk_actions.rs");

    let fn_start = content
        .find("fn trigger_sdk_action_internal(")
        .expect("Expected trigger_sdk_action_internal function");
    let fn_body = &content[fn_start..content.len().min(fn_start + 1500)];

    assert!(
        fn_body.contains("status = %status"),
        "trigger_sdk_action_internal must log status as a structured field"
    );
}

#[test]
fn trigger_sdk_action_internal_logs_error_code_field() {
    let content = read("src/app_actions/sdk_actions.rs");

    let fn_start = content
        .find("fn trigger_sdk_action_internal(")
        .expect("Expected trigger_sdk_action_internal function");
    let fn_body = &content[fn_start..content.len().min(fn_start + 1500)];

    assert!(
        fn_body.contains("error_code ="),
        "trigger_sdk_action_internal must log error_code as a structured field"
    );
}

#[test]
fn trigger_sdk_action_internal_uses_sdk_action_result() {
    let content = read("src/app_actions/sdk_actions.rs");

    let fn_start = content
        .find("fn trigger_sdk_action_internal(")
        .expect("Expected trigger_sdk_action_internal function");
    let fn_body = &content[fn_start..content.len().min(fn_start + 1500)];

    // Must call trigger_sdk_action from action_helpers
    assert!(
        fn_body.contains("trigger_sdk_action("),
        "trigger_sdk_action_internal must delegate to crate::action_helpers::trigger_sdk_action"
    );

    // Must extract status and error_code from the result
    assert!(
        fn_body.contains("result.status()"),
        "trigger_sdk_action_internal must call .status() on the SdkActionResult"
    );
    assert!(
        fn_body.contains("result.error_code()"),
        "trigger_sdk_action_internal must call .error_code() on the SdkActionResult"
    );
}

#[test]
fn trigger_sdk_action_internal_logs_handler_field() {
    let content = read("src/app_actions/sdk_actions.rs");

    let fn_start = content
        .find("fn trigger_sdk_action_internal(")
        .expect("Expected trigger_sdk_action_internal function");
    let fn_body = &content[fn_start..content.len().min(fn_start + 1500)];

    assert!(
        fn_body.contains(r#"handler = "sdk""#),
        "trigger_sdk_action_internal must identify itself with handler = \"sdk\""
    );
}
