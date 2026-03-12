//! Cross-cutting source audit asserting consistent structured tracing fields
//! across all action dispatch files.
//!
//! Unlike the per-function tests in `structured_logging.rs` and
//! `trace_propagation.rs`, these tests scan *all* action dispatch files
//! collectively to enforce field-level consistency.

use super::{count_occurrences, read_all_handle_action_sources, read_source as read};

// ---------------------------------------------------------------------------
// 1. All sub-handlers accept DispatchContext (trace_id carrier)
// ---------------------------------------------------------------------------

/// Every `handle_*_action` method in the modular handler files must accept a
/// `&DispatchContext` parameter — this is the sole carrier of `trace_id` into
/// the handler chain.
#[test]
fn all_sub_handlers_accept_dispatch_context() {
    let handler_fns = [
        ("clipboard.rs", "fn handle_clipboard_action("),
        ("files.rs", "fn handle_file_action("),
        ("scripts.rs", "fn handle_script_action("),
        ("scriptlets.rs", "fn handle_scriptlet_action("),
        ("shortcuts.rs", "fn handle_shortcut_alias_action("),
    ];

    for (file, sig) in &handler_fns {
        let path = format!("src/app_actions/handle_action/{file}");
        let content = read(&path);

        let fn_start = content
            .find(sig)
            .unwrap_or_else(|| panic!("Expected {sig} in {path}"));
        let signature = &content[fn_start..content.len().min(fn_start + 300)];

        assert!(
            signature.contains("dctx: &DispatchContext"),
            "{path}: {sig} must accept dctx: &DispatchContext for trace_id propagation"
        );
    }
}

/// Every sub-handler must return `DispatchOutcome` — the structured result
/// that carries `status`, `error_code`, and optional `trace_id`.
#[test]
fn all_sub_handlers_return_dispatch_outcome() {
    let handler_fns = [
        ("clipboard.rs", "fn handle_clipboard_action("),
        ("files.rs", "fn handle_file_action("),
        ("scripts.rs", "fn handle_script_action("),
        ("scriptlets.rs", "fn handle_scriptlet_action("),
        ("shortcuts.rs", "fn handle_shortcut_alias_action("),
    ];

    for (file, sig) in &handler_fns {
        let path = format!("src/app_actions/handle_action/{file}");
        let content = read(&path);

        let fn_start = content
            .find(sig)
            .unwrap_or_else(|| panic!("Expected {sig} in {path}"));
        let signature = &content[fn_start..content.len().min(fn_start + 400)];

        assert!(
            signature.contains("-> DispatchOutcome"),
            "{path}: {sig} must return DispatchOutcome"
        );
    }
}

// ---------------------------------------------------------------------------
// 2. Dispatch boundary logs trace_id in every outcome path
// ---------------------------------------------------------------------------

/// The top-level `handle_action` must create a `DispatchContext` (which
/// generates a trace_id) before dispatching to any sub-handler.
#[test]
fn handle_action_creates_dispatch_context_before_dispatch() {
    let content = read("src/app_actions/handle_action/mod.rs");

    let fn_start = content
        .find("fn handle_action(")
        .expect("Expected handle_action function");
    let fn_body = &content[fn_start..content.len().min(fn_start + 500)];

    assert!(
        fn_body.contains("DispatchContext::for_action("),
        "handle_action must create DispatchContext::for_action before dispatch"
    );
}

/// The dispatch outcome logger (`log_dispatch_outcome`) must include all
/// five mandatory structured fields: action, trace_id, handler, status,
/// duration_ms.
#[test]
fn log_dispatch_outcome_includes_all_mandatory_fields() {
    let content = read("src/app_actions/handle_action/mod.rs");

    let fn_start = content
        .find("fn log_dispatch_outcome(")
        .expect("Expected log_dispatch_outcome function");
    let fn_body = &content[fn_start..content.len().min(fn_start + 400)];

    let required_fields = [
        ("action =", "action identifier"),
        ("trace_id =", "correlation trace_id"),
        ("handler =", "handler name"),
        ("status =", "outcome status"),
        ("duration_ms =", "elapsed duration"),
    ];

    for (field, desc) in &required_fields {
        assert!(
            fn_body.contains(field),
            "log_dispatch_outcome must include {desc} as a structured field ({field})"
        );
    }
}

/// `log_dispatch_outcome` must also include `error_code` so that failed
/// dispatches carry a machine-readable code alongside the status.
#[test]
fn log_dispatch_outcome_includes_error_code_field() {
    let content = read("src/app_actions/handle_action/mod.rs");

    let fn_start = content
        .find("fn log_dispatch_outcome(")
        .expect("Expected log_dispatch_outcome function");
    let fn_body = &content[fn_start..content.len().min(fn_start + 400)];

    assert!(
        fn_body.contains("error_code ="),
        "log_dispatch_outcome must include error_code as a structured field"
    );
}

// ---------------------------------------------------------------------------
// 3. Error code constants are defined centrally
// ---------------------------------------------------------------------------

/// All stable error code constants must be defined in `action_helpers.rs`
/// (the canonical location), not scattered across handler files.
#[test]
fn error_code_constants_defined_in_action_helpers() {
    let content = read("src/action_helpers.rs");

    let expected_codes = [
        "ERROR_CHANNEL_FULL",
        "ERROR_CHANNEL_DISCONNECTED",
        "ERROR_UNSUPPORTED_PLATFORM",
        "ERROR_LAUNCH_FAILED",
        "ERROR_REVEAL_FAILED",
        "ERROR_MODAL_FAILED",
        "ERROR_ACTION_FAILED",
        "ERROR_CANCELLED",
        "ERROR_NO_SENDER",
    ];

    for code in &expected_codes {
        assert!(
            content.contains(&format!("pub const {code}")),
            "action_helpers.rs must define stable error code constant: {code}"
        );
    }
}

/// No handler file should define its own `ERROR_*` constants — they must
/// reference the canonical definitions from `action_helpers`.
#[test]
fn handler_files_do_not_define_own_error_constants() {
    let sources = read_all_handle_action_sources();

    // `const ERROR_` would indicate a local definition
    assert!(
        !sources.contains("const ERROR_"),
        "Handler files must not define their own ERROR_* constants; \
         use crate::action_helpers::ERROR_* instead"
    );
}

// ---------------------------------------------------------------------------
// 4. show_error_toast_with_code pairs error_code with user-facing message
// ---------------------------------------------------------------------------

/// The canonical error helper must log both `error_code` and `message` as
/// structured fields — not just one or the other.
#[test]
fn show_error_toast_with_code_logs_both_error_code_and_message() {
    let content = read("src/app_actions/handle_action/mod.rs");

    let fn_start = content
        .find("fn show_error_toast_with_code(")
        .expect("Expected show_error_toast_with_code function");
    let fn_body = &content[fn_start..content.len().min(fn_start + 500)];

    assert!(
        fn_body.contains("error_code ="),
        "show_error_toast_with_code must log error_code as a structured field"
    );
    assert!(
        fn_body.contains("message ="),
        "show_error_toast_with_code must log message as a structured field"
    );
}

/// `show_error_toast_with_code` must use `tracing::warn!` (not println or
/// log crate) so that structured fields are captured by the tracing
/// subscriber.
#[test]
fn show_error_toast_with_code_uses_tracing() {
    let content = read("src/app_actions/handle_action/mod.rs");

    let fn_start = content
        .find("fn show_error_toast_with_code(")
        .expect("Expected show_error_toast_with_code function");
    let fn_body = &content[fn_start..content.len().min(fn_start + 500)];

    assert!(
        fn_body.contains("tracing::warn!("),
        "show_error_toast_with_code must use tracing::warn! for structured logging"
    );
}

// ---------------------------------------------------------------------------
// 5. DispatchOutcome carries both status and error_code
// ---------------------------------------------------------------------------

/// `DispatchOutcome` must have both a `status` and an `error_code` field
/// so that every dispatch result can be logged with machine-readable codes.
#[test]
fn dispatch_outcome_has_status_and_error_code_fields() {
    let content = read("src/action_helpers.rs");

    let struct_start = content
        .find("pub struct DispatchOutcome")
        .expect("Expected DispatchOutcome struct");
    let struct_body = &content[struct_start..content.len().min(struct_start + 600)];

    assert!(
        struct_body.contains("pub status: ActionOutcomeStatus"),
        "DispatchOutcome must have a status field of type ActionOutcomeStatus"
    );
    assert!(
        struct_body.contains("pub error_code: Option<&'static str>"),
        "DispatchOutcome must have an error_code field"
    );
}

// ---------------------------------------------------------------------------
// 6. Action dispatch files use tracing (not log/println) exclusively
// ---------------------------------------------------------------------------

/// No action dispatch file should use `println!` for logging — only
/// `tracing::*` macros are permitted.
#[test]
fn no_println_in_action_dispatch_files() {
    let dispatch_files = [
        "src/app_actions/handle_action/mod.rs",
        "src/app_actions/handle_action/clipboard.rs",
        "src/app_actions/handle_action/files.rs",
        "src/app_actions/handle_action/scripts.rs",
        "src/app_actions/handle_action/scriptlets.rs",
        "src/app_actions/handle_action/shortcuts.rs",
        "src/app_actions/sdk_actions.rs",
    ];

    for path in &dispatch_files {
        let content = read(path);
        assert!(
            !content.contains("println!("),
            "{path} must not use println! — use tracing macros instead"
        );
    }
}

/// No action dispatch file should introduce new `log::` crate usage —
/// the canonical logging API is `tracing`.
#[test]
fn no_log_crate_in_action_dispatch_files() {
    let dispatch_files = [
        "src/app_actions/handle_action/mod.rs",
        "src/app_actions/handle_action/clipboard.rs",
        "src/app_actions/handle_action/files.rs",
        "src/app_actions/handle_action/scripts.rs",
        "src/app_actions/handle_action/scriptlets.rs",
        "src/app_actions/handle_action/shortcuts.rs",
        "src/app_actions/sdk_actions.rs",
    ];

    for path in &dispatch_files {
        let content = read(path);
        assert!(
            !content.contains("log::info!") && !content.contains("log::warn!") && !content.contains("log::error!"),
            "{path} must not use log:: macros — use tracing:: instead"
        );
    }
}

// ---------------------------------------------------------------------------
// 7. Dispatch boundary consistently calls log_dispatch_outcome for all paths
// ---------------------------------------------------------------------------

/// Every dispatch path in `handle_action` must call `log_dispatch_outcome`
/// so that *all* action invocations are observable, not just some.
#[test]
fn handle_action_calls_log_dispatch_outcome_for_all_paths() {
    let content = read("src/app_actions/handle_action/mod.rs");

    let fn_start = content
        .find("fn handle_action(")
        .expect("Expected handle_action function");
    let fn_body = &content[fn_start..content.len().min(fn_start + 3000)];

    // Must call log_dispatch_outcome at least twice:
    // once for the early clipboard return, once for the main chain
    let call_count = count_occurrences(fn_body, "log_dispatch_outcome(");
    assert!(
        call_count >= 2,
        "handle_action must call log_dispatch_outcome in all dispatch paths \
         (found {call_count}, expected >= 2)"
    );
}

/// The initial "dispatch started" log must include trace_id so the start
/// event can be correlated with the completion event.
#[test]
fn handle_action_start_log_includes_trace_id() {
    let content = read("src/app_actions/handle_action/mod.rs");

    let start_log = content
        .find("Action dispatch started")
        .expect("Expected 'Action dispatch started' log message");
    let window = &content[start_log.saturating_sub(200)..content.len().min(start_log + 50)];

    assert!(
        window.contains("trace_id = %dctx.trace_id"),
        "Action dispatch started log must include trace_id from DispatchContext"
    );
}

// ---------------------------------------------------------------------------
// 8. Async handlers propagate trace_id through owned copies
// ---------------------------------------------------------------------------

/// Async handlers that use `cx.spawn` must clone trace_id into an owned
/// `String` before the async boundary — raw `&str` references cannot cross
/// `.await` points.
#[test]
fn async_handlers_clone_trace_id_before_spawn() {
    let content = read("src/app_actions/handle_action/mod.rs");

    // The file contains async handlers that convert trace_id to owned
    let has_owned_trace = content.contains("trace_id.to_string()")
        || content.contains("trace_id = trace_id.to_string()");

    assert!(
        has_owned_trace,
        "Async handlers in handle_action/mod.rs must clone trace_id to an owned \
         String before cx.spawn boundaries"
    );
}
