//! Unit tests for builtin dispatch context and outcome primitives.
//!
//! These tests verify that the `DispatchContext` and `DispatchOutcome` types
//! work correctly for the builtin execution surface.

use script_kit_gpui::action_helpers::{
    ActionOutcomeStatus, DispatchContext, DispatchOutcome, DispatchSurface, ERROR_CANCELLED,
    ERROR_LAUNCH_FAILED,
};

#[test]
fn dispatch_context_for_builtin_sets_builtin_surface() {
    let dctx = DispatchContext::for_builtin("builtin/clipboard-history");

    assert_eq!(dctx.surface, DispatchSurface::Builtin);
    assert_eq!(dctx.action_id, "builtin/clipboard-history");
    assert!(!dctx.trace_id.is_empty());
}

#[test]
fn dispatch_context_for_builtin_generates_unique_trace_ids() {
    let dctx1 = DispatchContext::for_builtin("builtin/notes");
    let dctx2 = DispatchContext::for_builtin("builtin/notes");

    assert_ne!(
        dctx1.trace_id, dctx2.trace_id,
        "Each DispatchContext should get a unique trace_id"
    );
}

#[test]
fn builtin_success_outcome_carries_trace_id() {
    let dctx = DispatchContext::for_builtin("builtin/notes");

    let outcome = DispatchOutcome::success()
        .with_trace_id(dctx.trace_id.clone())
        .with_detail("opened_notes");

    assert_eq!(outcome.status, ActionOutcomeStatus::Success);
    assert_eq!(outcome.trace_id.as_deref(), Some(dctx.trace_id.as_str()));
    assert_eq!(outcome.error_code, None);
    assert_eq!(outcome.detail.as_deref(), Some("opened_notes"));
}

#[test]
fn builtin_error_outcome_carries_code_message_and_trace_id() {
    let dctx = DispatchContext::for_builtin("builtin/restart");

    let outcome = DispatchOutcome::error(
        ERROR_LAUNCH_FAILED,
        "System action failed: permission denied",
    )
    .with_trace_id(dctx.trace_id.clone())
    .with_detail("system_action::Restart");

    assert_eq!(outcome.status, ActionOutcomeStatus::Error);
    assert_eq!(outcome.error_code, Some(ERROR_LAUNCH_FAILED));
    assert_eq!(
        outcome.user_message.as_deref(),
        Some("System action failed: permission denied")
    );
    assert_eq!(outcome.trace_id.as_deref(), Some(dctx.trace_id.as_str()));
    assert_eq!(outcome.detail.as_deref(), Some("system_action::Restart"));
}

#[test]
fn builtin_cancelled_outcome_has_cancelled_status_and_error_code() {
    let dctx = DispatchContext::for_builtin("builtin/shut-down");

    let outcome = DispatchOutcome::cancelled()
        .with_trace_id(dctx.trace_id.clone())
        .with_detail("builtin_confirmation_cancelled");

    assert_eq!(outcome.status, ActionOutcomeStatus::Cancelled);
    assert_eq!(outcome.error_code, Some(ERROR_CANCELLED));
    assert_eq!(outcome.user_message, None);
    assert_eq!(outcome.trace_id.as_deref(), Some(dctx.trace_id.as_str()));
}

#[test]
fn dispatch_surface_display_formats_correctly() {
    assert_eq!(format!("{}", DispatchSurface::Builtin), "builtin");
    assert_eq!(format!("{}", DispatchSurface::Action), "action");
}
