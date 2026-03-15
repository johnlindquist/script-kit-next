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

    // Verify the cancel path returns early
    let cancel_block_start = content
        .find("if !confirmed {")
        .expect("Expected cancel check");
    let cancel_block = &content[cancel_block_start..cancel_block_start + 400];

    assert!(
        cancel_block.contains("return;"),
        "Expected cancel path to return early without executing builtin"
    );
    assert!(
        cancel_block.contains("builtin_confirmation_cancelled"),
        "Expected cancel path to log cancellation via DispatchOutcome"
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

#[test]
fn handle_builtin_confirmation_accepts_dispatch_context() {
    let content = builtin_confirmation_content();

    let fn_start = content
        .find("fn handle_builtin_confirmation(")
        .expect("Expected handle_builtin_confirmation");
    let signature = &content[fn_start..content.len().min(fn_start + 400)];

    assert!(
        signature.contains("dctx: &crate::action_helpers::DispatchContext"),
        "handle_builtin_confirmation must accept &DispatchContext instead of &str trace_id"
    );
}

#[test]
fn handle_builtin_confirmation_logs_outcome_on_cancel() {
    let content = builtin_confirmation_content();

    assert!(
        content.contains("DispatchOutcome::cancelled()"),
        "Expected cancel path to create DispatchOutcome::cancelled()"
    );
    assert!(
        content.contains("log_builtin_outcome("),
        "Expected cancel path to call log_builtin_outcome"
    );
}

#[test]
fn handle_builtin_confirmation_delegates_all_arms_to_execute_builtin_inner() {
    let content = builtin_confirmation_content();

    // After the refactor, all confirmed builtins (including system actions)
    // go through execute_builtin_inner which returns DispatchOutcome.
    assert!(
        content.contains("self.execute_builtin_inner("),
        "Expected confirmed builtins to delegate to execute_builtin_inner"
    );

    // The confirmed path must NOT contain a separate match on SystemAction —
    // execute_builtin_inner handles that internally.
    assert!(
        !content.contains("BuiltInFeature::SystemAction("),
        "Confirmed path should delegate ALL arms to execute_builtin_inner, not special-case SystemAction"
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
        fn_body.contains("self.dispatch_system_action(action_type,"),
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

#[test]
fn builtin_execution_uses_confirm_with_modal_not_direct_open() {
    let execution_content = read("src/app_execute/builtin_execution.rs");

    // Must NOT contain direct open_confirm_window calls — use confirm_with_modal instead
    assert!(
        !execution_content.contains("open_confirm_window("),
        "builtin_execution.rs must not call open_confirm_window directly; use confirm_with_modal"
    );

    // Must use the shared confirm_with_modal helper
    assert!(
        execution_content.contains("confirm_with_modal("),
        "builtin_execution.rs must use confirm_with_modal for confirmation dialogs"
    );
}

#[test]
fn builtin_confirmation_does_not_call_open_confirm_window() {
    let content = builtin_confirmation_content();

    assert!(
        !content.contains("open_confirm_window("),
        "builtin_confirmation.rs must not call open_confirm_window directly"
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

// ---------------------------------------------------------------------------
// Destructive builtins — every DEFAULT_CONFIRMATION_COMMANDS entry goes
// through requires_confirmation → confirm_with_modal
// ---------------------------------------------------------------------------

#[test]
fn destructive_builtins_are_gated_by_confirmation_defaults() {
    // Verify the default confirmation commands constant exists and contains
    // the expected destructive builtins. This ensures the list is not
    // accidentally truncated or cleared.
    let defaults = read("src/config/defaults.rs");

    let expected_destructive = [
        "builtin-shut-down",
        "builtin-restart",
        "builtin-log-out",
        "builtin-empty-trash",
        "builtin-sleep",
        "builtin-quit-script-kit",
        "builtin-force-quit",
        "builtin-stop-all-processes",
        "builtin-clear-suggested",
    ];

    for cmd in &expected_destructive {
        assert!(
            defaults.contains(cmd),
            "DEFAULT_CONFIRMATION_COMMANDS must include destructive builtin: {cmd}"
        );
    }
}

#[test]
fn confirmation_path_spawns_task_with_confirm_with_modal() {
    // The confirmation branch in execute_builtin_with_query must use
    // cx.spawn + confirm_with_modal — not inline blocking.
    let content = read("src/app_execute/builtin_execution.rs");

    let confirm_check = content
        .find("self.config.requires_confirmation(&entry.id)")
        .expect("Expected requires_confirmation gate in builtin_execution.rs");
    let after_check = &content[confirm_check..content.len().min(confirm_check + 800)];

    assert!(
        after_check.contains("cx.spawn("),
        "Confirmation path must use cx.spawn for async modal flow"
    );
    assert!(
        after_check.contains("confirm_with_modal("),
        "Confirmation path must call confirm_with_modal inside the spawned task"
    );
}

#[test]
fn confirmation_path_handles_accept_cancel_and_error() {
    // The spawned confirmation task must handle all three result arms
    let content = read("src/app_execute/builtin_execution.rs");

    let confirm_check = content
        .find("self.config.requires_confirmation(&entry.id)")
        .expect("Expected requires_confirmation gate");
    let after_check = &content[confirm_check..content.len().min(confirm_check + 3000)];

    assert!(
        after_check.contains("Ok(true)"),
        "Confirmation spawn must handle Ok(true) — user accepted"
    );
    assert!(
        after_check.contains("Ok(false)"),
        "Confirmation spawn must handle Ok(false) — user cancelled"
    );
    assert!(
        after_check.contains("Err(e)") || after_check.contains("Err(_)"),
        "Confirmation spawn must handle Err — modal open failure"
    );
}

// ---------------------------------------------------------------------------
// active_favorites lifecycle — set on Favorites builtin, reset on dismiss
// ---------------------------------------------------------------------------

#[test]
fn favorites_builtin_opens_browse_view() {
    let content = read("src/app_execute/builtin_execution.rs");

    let favorites_branch = content
        .find("BuiltInFeature::Favorites")
        .expect("Expected BuiltInFeature::Favorites branch in builtin_execution.rs");
    let block = &content[favorites_branch..content.len().min(favorites_branch + 2200)];

    // Must transition to FavoritesBrowseView
    assert!(
        block.contains("AppView::FavoritesBrowseView"),
        "Favorites builtin must open FavoritesBrowseView"
    );
}

#[test]
fn favorites_builtin_uses_shared_filterable_view_helper() {
    let content = read("src/app_execute/builtin_execution.rs");

    let favorites_branch = content
        .find("BuiltInFeature::Favorites")
        .expect("Expected BuiltInFeature::Favorites branch");
    let block = &content[favorites_branch..content.len().min(favorites_branch + 2200)];

    // Must use the shared helper which handles filter clearing and cx.notify()
    assert!(
        block.contains("self.open_builtin_filterable_view("),
        "Favorites builtin must use open_builtin_filterable_view shared helper"
    );
}

#[test]
fn open_builtin_filterable_view_clears_filter_and_notifies() {
    let content = read("src/app_execute/builtin_execution.rs");

    let helper_start = content
        .find("fn open_builtin_filterable_view(")
        .expect("Expected open_builtin_filterable_view helper");
    let block = &content[helper_start..content.len().min(helper_start + 800)];

    // The shared helper must clear filter text
    assert!(
        block.contains("self.filter_text.clear()"),
        "open_builtin_filterable_view must clear filter text"
    );

    // The shared helper must call cx.notify()
    assert!(
        block.contains("cx.notify()"),
        "open_builtin_filterable_view must call cx.notify()"
    );
}

#[test]
fn favorites_render_module_handles_errors() {
    let content = read("src/render_builtins/favorites.rs");

    // Must handle remove errors gracefully
    assert!(
        content.contains("show_error_toast"),
        "Favorites render must show error toast on failure"
    );
    // Must handle not-found case
    assert!(
        content.contains("favorite_not_found"),
        "Favorites render must handle script not found"
    );
}

#[test]
fn reset_to_script_list_clears_active_favorites() {
    let content = read("src/app_impl/registries_state.rs");

    // Verify the function exists
    assert!(
        content.contains("fn reset_to_script_list("),
        "Expected reset_to_script_list function in registries_state.rs"
    );

    // The function body is large; search the full file for the clearing pattern.
    // Both must be present in the same file to guarantee the lifecycle.
    assert!(
        content.contains("self.active_favorites = None"),
        "reset_to_script_list must clear active_favorites = None to prevent stale filter"
    );
}

#[test]
fn active_favorites_initialized_to_none_at_startup() {
    let startup = read("src/app_impl/startup.rs");

    assert!(
        startup.contains("active_favorites: None"),
        "App state must initialize active_favorites to None at startup"
    );
}
