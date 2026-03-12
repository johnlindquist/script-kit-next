//! Tests for builtin_confirmation.rs — accept/cancel flow, missing entry handling,
//! and centralized execution via execute_builtin_inner.

use super::read_source as read;

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
        content.contains("Builtin entry not found for confirmed action"),
        "Expected error log when confirmed entry_id has no matching builtin"
    );
}

// ---------------------------------------------------------------------------
// Centralized execution — confirmed path uses execute_builtin_inner
// ---------------------------------------------------------------------------

#[test]
fn handle_builtin_confirmation_delegates_to_execute_builtin_inner() {
    let content = builtin_confirmation_content();

    assert!(
        content.contains("self.execute_builtin_inner("),
        "Expected handle_builtin_confirmation to delegate to execute_builtin_inner"
    );
}

#[test]
fn handle_builtin_confirmation_preserves_query_override() {
    let content = builtin_confirmation_content();

    // The function signature must accept query_override
    assert!(
        content.contains("query_override: Option<String>"),
        "Expected handle_builtin_confirmation to accept query_override parameter"
    );

    // query_override must be forwarded to execute_builtin_inner
    assert!(
        content.contains("query_override.as_deref()"),
        "Expected query_override to be forwarded to execute_builtin_inner"
    );
}

#[test]
fn handle_builtin_confirmation_logs_accepted_action_before_execution() {
    let content = builtin_confirmation_content();

    // Verify acceptance is logged with the entry_id for observability
    assert!(
        content.contains("Builtin confirmation accepted, executing"),
        "Expected acceptance to be logged before execution"
    );
}

// ---------------------------------------------------------------------------
// execute_builtin_inner — single execution path for all builtins
// ---------------------------------------------------------------------------

#[test]
fn execute_builtin_inner_exists_in_builtin_execution() {
    let execution_content = read("src/app_execute/builtin_execution.rs");

    assert!(
        execution_content.contains("fn execute_builtin_inner("),
        "Expected execute_builtin_inner method in builtin_execution.rs"
    );
}

#[test]
fn execute_builtin_inner_handles_system_actions() {
    let execution_content = read("src/app_execute/builtin_execution.rs");

    let fn_start = execution_content
        .find("fn execute_builtin_inner(")
        .expect("Expected execute_builtin_inner function");
    let fn_body = &execution_content[fn_start..];

    assert!(
        fn_body.contains("BuiltInFeature::SystemAction("),
        "Expected execute_builtin_inner to match SystemAction variant"
    );
    assert!(
        fn_body.contains("self.dispatch_system_action(action_type, cx)"),
        "Expected execute_builtin_inner to call dispatch_system_action for SystemAction"
    );
}

#[test]
fn execute_builtin_with_query_delegates_to_inner() {
    let execution_content = read("src/app_execute/builtin_execution.rs");

    let fn_start = execution_content
        .find("fn execute_builtin_with_query(")
        .expect("Expected execute_builtin_with_query function");
    // Look within the method body for the delegation call
    let fn_body = &execution_content[fn_start..execution_content.len().min(fn_start + 6000)];

    assert!(
        fn_body.contains("self.execute_builtin_inner("),
        "Expected execute_builtin_with_query to delegate to execute_builtin_inner"
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
    let fn_body = &helpers[fn_start..helpers.len().min(fn_start + 3000)];

    // Must await the receiver to get the confirmation result
    assert!(
        fn_body.contains("confirm_rx.recv().await"),
        "confirm_with_modal must await confirm_rx.recv() for the confirmation result"
    );
}

#[test]
fn confirm_with_modal_delegates_to_open_confirm_window() {
    let helpers = read("src/app_actions/helpers.rs");

    let fn_start = helpers
        .find("async fn confirm_with_modal(")
        .expect("Expected confirm_with_modal function");
    let fn_body = &helpers[fn_start..helpers.len().min(fn_start + 3000)];

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
    let handle_action = super::read_all_handle_action_sources();

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
    let handle_action = super::read_all_handle_action_sources();

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
