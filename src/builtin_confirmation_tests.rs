//! Tests for builtin_confirmation.rs — accept/cancel flow, missing entry handling,
//! and system-action-only restriction.

use crate::test_utils::read_source as read;

fn builtin_confirmation_content() -> String {
    read("src/app_execute/builtin_confirmation.rs")
}

// ---------------------------------------------------------------------------
// Cancel flow — early return without side effects
// ---------------------------------------------------------------------------

#[test]
fn handle_builtin_confirmation_returns_early_on_cancel() {
    let content = builtin_confirmation_content();

    // When confirmed is false, should return immediately without executing
    assert!(
        content.contains("if !confirmed {"),
        "Expected handle_builtin_confirmation to check confirmed flag"
    );

    // Verify the cancel path logs and returns
    let cancel_block_start = content
        .find("if !confirmed {")
        .expect("Expected cancel check");
    let cancel_block = &content[cancel_block_start..cancel_block_start + 200];

    assert!(
        cancel_block.contains("return;"),
        "Expected cancel path to return early without executing builtin"
    );
    assert!(
        cancel_block.contains("confirmation cancelled"),
        "Expected cancel path to log cancellation"
    );
}

// ---------------------------------------------------------------------------
// Accept flow — entry lookup and execution
// ---------------------------------------------------------------------------

#[test]
fn handle_builtin_confirmation_looks_up_entry_by_id() {
    let content = builtin_confirmation_content();

    assert!(
        content.contains("get_builtin_entries("),
        "Expected handle_builtin_confirmation to look up builtin entries"
    );
    assert!(
        content.contains("builtin_entries.iter().find(|b| b.id == entry_id)"),
        "Expected handle_builtin_confirmation to find entry by matching id"
    );
}

#[test]
fn handle_builtin_confirmation_logs_error_when_entry_not_found() {
    let content = builtin_confirmation_content();

    assert!(
        content.contains("\"Builtin entry not found for confirmed action: \""),
        "Expected error log when confirmed entry_id has no matching builtin"
    );
}

// ---------------------------------------------------------------------------
// execute_builtin_confirmed — system action dispatch only
// ---------------------------------------------------------------------------

#[test]
fn execute_builtin_confirmed_dispatches_system_actions() {
    let content = builtin_confirmation_content();

    assert!(
        content.contains("BuiltInFeature::SystemAction(action_type)"),
        "Expected execute_builtin_confirmed to match SystemAction variant"
    );
    assert!(
        content.contains("self.dispatch_system_action(action_type, cx)"),
        "Expected execute_builtin_confirmed to call dispatch_system_action for SystemAction"
    );
}

#[test]
fn execute_builtin_confirmed_warns_on_unexpected_builtin_type() {
    let content = builtin_confirmation_content();

    // Non-SystemAction types that somehow reach confirmed path should warn
    assert!(
        content.contains("\"Unexpected confirmed builtin type:"),
        "Expected execute_builtin_confirmed to warn on non-SystemAction types"
    );
}

#[test]
fn execute_builtin_confirmed_handles_all_confirmable_types_and_wildcard() {
    let content = builtin_confirmation_content();

    let fn_start = content
        .find("fn execute_builtin_confirmed(")
        .expect("Expected execute_builtin_confirmed function");
    let fn_body = &content[fn_start..];

    // Should handle SystemAction, UtilityCommand(StopAllProcesses),
    // FrecencyCommand(ClearSuggested), and a wildcard
    assert!(
        fn_body.contains("BuiltInFeature::SystemAction("),
        "Expected execute_builtin_confirmed to match SystemAction"
    );
    assert!(
        fn_body.contains("BuiltInFeature::UtilityCommand("),
        "Expected execute_builtin_confirmed to match UtilityCommand(StopAllProcesses)"
    );
    assert!(
        fn_body.contains("BuiltInFeature::FrecencyCommand("),
        "Expected execute_builtin_confirmed to match FrecencyCommand(ClearSuggested)"
    );
}

// ---------------------------------------------------------------------------
// Contract: confirmation flow only applies to system actions
// ---------------------------------------------------------------------------

#[test]
fn confirmation_flow_is_gated_by_config_requires_confirmation() {
    // Read builtin_execution.rs to verify that confirmation is only triggered
    // when config.requires_confirmation returns true for the entry
    let execution_content = read("src/app_execute/builtin_execution.rs");

    assert!(
        execution_content.contains("self.config.requires_confirmation(&entry.id)"),
        "Expected builtin execution to check config.requires_confirmation for the entry id"
    );
}

#[test]
fn handle_builtin_confirmation_logs_accepted_action_before_execution() {
    let content = builtin_confirmation_content();

    // Verify acceptance is logged with the entry_id for observability
    assert!(
        content.contains("\"Builtin confirmation accepted, executing: \""),
        "Expected acceptance to be logged before execution"
    );
}

// ---------------------------------------------------------------------------
// confirm_with_modal helper — channel patterns
// ---------------------------------------------------------------------------

#[test]
fn confirm_with_modal_uses_bounded_channel_capacity_one() {
    let helpers = read("src/app_actions/helpers.rs");

    // The channel must be bounded(1) — not unbounded, not larger capacity
    assert!(
        helpers.contains("async_channel::bounded::<bool>(1)"),
        "confirm_with_modal must use bounded channel with capacity 1 for single-shot confirmation"
    );
}

#[test]
fn confirm_with_modal_awaits_receiver_for_result() {
    let helpers = read("src/app_actions/helpers.rs");

    // Find the confirm_with_modal function body
    let fn_start = helpers
        .find("async fn confirm_with_modal(")
        .expect("Expected confirm_with_modal function");
    let fn_body = &helpers[fn_start..helpers.len().min(fn_start + 800)];

    // Must await the receiver to get the confirmation result
    assert!(
        fn_body.contains("rx.recv().await"),
        "confirm_with_modal must await rx.recv() for the confirmation result"
    );
}

#[test]
fn confirm_with_modal_delegates_to_open_confirm_window() {
    let helpers = read("src/app_actions/helpers.rs");

    let fn_start = helpers
        .find("async fn confirm_with_modal(")
        .expect("Expected confirm_with_modal function");
    let fn_body = &helpers[fn_start..helpers.len().min(fn_start + 800)];

    assert!(
        fn_body.contains("confirm::open_confirm_window("),
        "confirm_with_modal must delegate to the shared confirm window"
    );
}

// ---------------------------------------------------------------------------
// confirm_with_modal call sites — consistent error handling pattern
// ---------------------------------------------------------------------------

#[test]
fn confirm_with_modal_callers_handle_all_three_results() {
    let handle_action = crate::test_utils::read_all_handle_action_sources();

    // Every call site should handle Ok(true), Ok(false), and Err
    let call_sites: Vec<_> = handle_action.match_indices("confirm_with_modal(").collect();
    assert!(
        !call_sites.is_empty(),
        "Expected at least one confirm_with_modal call site in handle_action/"
    );

    for (pos, _) in &call_sites {
        let block = &handle_action[*pos..handle_action.len().min(*pos + 1200)];

        assert!(
            block.contains("Ok(true)"),
            "confirm_with_modal call site must handle Ok(true) (accept) path"
        );
        assert!(
            block.contains("Ok(false)") || block.contains("return"),
            "confirm_with_modal call site must handle Ok(false) (reject/cancel) path"
        );
        assert!(
            block.contains("Err(e)") || block.contains("Err(_)"),
            "confirm_with_modal call site must handle Err (channel closed) path"
        );
    }
}

#[test]
fn confirm_with_modal_error_path_logs_failure() {
    let handle_action = crate::test_utils::read_all_handle_action_sources();

    // Every Err path from confirm_with_modal should log the error
    let call_sites: Vec<_> = handle_action.match_indices("confirm_with_modal(").collect();

    for (pos, _) in &call_sites {
        let block = &handle_action[*pos..handle_action.len().min(*pos + 1200)];

        assert!(
            block.contains("failed to open confirmation modal"),
            "confirm_with_modal Err path should log 'failed to open confirmation modal'"
        );
    }
}
