//! Tests for shortcut, alias, and file search action handlers.
//!
//! These tests verify:
//! - Shortcut add/update/remove handlers exist and follow consistent patterns
//! - Alias add/update/remove handlers exist and follow consistent patterns
//! - File search actions (open_file, quick_look, open_with, show_info) follow patterns
//! - Error paths show Toast feedback for all action categories
//! - Selection-required guards are present on all interactive actions

use super::{count_occurrences, read_source as read};

// ---------------------------------------------------------------------------
// Shortcut action handler tests
// ---------------------------------------------------------------------------

#[test]
fn shortcut_add_and_update_actions_are_handled() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("\"configure_shortcut\" | \"add_shortcut\" | \"update_shortcut\""),
        "Expected handle_action/ to handle configure_shortcut, add_shortcut, and update_shortcut"
    );
}

#[test]
fn shortcut_add_opens_recorder_for_non_script_types() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("self.show_shortcut_recorder(command_id, command_name, window, cx)"),
        "Expected shortcut actions to use show_shortcut_recorder for bindable launcher results"
    );
}

#[test]
fn shortcut_add_uses_launcher_command_ids() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("result.launcher_command_id()"),
        "Expected shortcut actions to resolve command IDs through SearchResult::launcher_command_id"
    );
}

#[test]
fn shortcut_add_shows_error_for_window_type() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("Shortcuts not supported for this item type"),
        "Expected shortcut actions to show error toast for Window type"
    );
}

#[test]
fn shortcut_add_shows_selection_required_on_no_selection() {
    let actions = super::read_all_handle_action_sources();

    // The shortcut block should call selection_required_message_for_action when no item selected
    assert!(
        actions.contains("selection_required_message_for_action(action_id)"),
        "Expected shortcut actions to show selection-required message when nothing is selected"
    );
}

#[test]
fn shortcut_remove_action_is_handled() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("\"remove_shortcut\""),
        "Expected handle_action/ to handle remove_shortcut"
    );
}

#[test]
fn shortcut_remove_calls_persistence_layer() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("remove_config_command_shortcut(&command_id)"),
        "Expected remove_shortcut to remove shortcuts from config.ts"
    );
}

#[test]
fn shortcut_remove_shows_success_hud_on_ok() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("\"Shortcut removed\""),
        "Expected remove_shortcut to show 'Shortcut removed' HUD on success"
    );
}

#[test]
fn shortcut_remove_shows_error_toast_on_failure() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("remove_action.failure_message(e)")
            && actions
                .contains("Self::Shortcut => format!(\"Failed to remove shortcut: {error}\")"),
        "Expected remove_shortcut to show error toast when persistence fails"
    );
}

#[test]
fn shortcut_remove_refreshes_scripts_after_success() {
    let actions = super::read_all_handle_action_sources();

    // After removing shortcut, scripts should be refreshed to update display
    assert!(
        actions.contains("self.refresh_scripts(cx)"),
        "Expected remove_shortcut to call refresh_scripts after successful removal"
    );
}

#[test]
fn shortcut_remove_rejects_unsupported_window_type() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("\"Cannot remove shortcut for this item type\""),
        "Expected remove_shortcut to show error for unsupported item types (Window)"
    );
}

#[test]
fn shortcut_remove_builds_command_id_for_all_supported_types() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("result.launcher_command_id()"),
        "Expected remove_shortcut to resolve command IDs through SearchResult::launcher_command_id"
    );
}

// ---------------------------------------------------------------------------
// Alias action handler tests
// ---------------------------------------------------------------------------

#[test]
fn alias_add_and_update_actions_are_handled() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("\"add_alias\" | \"update_alias\""),
        "Expected handle_action/ to handle add_alias and update_alias"
    );
}

#[test]
fn alias_add_opens_alias_input_dialog() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("self.show_alias_input(command_id, command_name, cx)"),
        "Expected alias actions to open alias input dialog via show_alias_input"
    );
}

#[test]
fn alias_add_shows_error_for_window_type() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("Aliases not supported for this item type"),
        "Expected alias actions to show error toast for Window type"
    );
}

#[test]
fn alias_add_shows_selection_required_on_no_selection() {
    let actions = super::read_all_handle_action_sources();

    // Alias actions should use the same selection_required_message pattern
    // (already verified the function is called; verify the specific messages exist)
    let helpers = read("src/app_actions/helpers.rs");
    assert!(
        helpers.contains("\"add_alias\" | \"update_alias\""),
        "Expected selection_required_message_for_action to handle add_alias and update_alias"
    );
    assert!(
        helpers.contains("\"remove_alias\""),
        "Expected selection_required_message_for_action to handle remove_alias"
    );
    // Verify the actual error messages are contextual
    assert!(
        helpers.contains("Select an item to add or update its alias."),
        "Expected contextual message for add/update alias"
    );
    assert!(
        helpers.contains("Select an item to remove its alias."),
        "Expected contextual message for remove alias"
    );

    // Also verify the main handler uses this function
    assert!(
        actions.contains("selection_required_message_for_action(action_id)"),
        "Expected alias actions to use selection_required_message_for_action"
    );
}

#[test]
fn alias_remove_action_is_handled() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("\"remove_alias\""),
        "Expected handle_action/ to handle remove_alias"
    );
}

#[test]
fn alias_remove_calls_persistence_layer() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("crate::aliases::remove_alias_override(&command_id)"),
        "Expected remove_alias to call aliases::remove_alias_override"
    );
}

#[test]
fn alias_remove_shows_success_hud_on_ok() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("\"Alias removed\""),
        "Expected remove_alias to show 'Alias removed' HUD on success"
    );
}

#[test]
fn alias_remove_shows_error_toast_on_failure() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("remove_action.failure_message(e)")
            && actions.contains("Self::Alias => format!(\"Failed to remove alias: {error}\")"),
        "Expected remove_alias to show error toast when persistence fails"
    );
}

#[test]
fn alias_remove_refreshes_scripts_after_success() {
    let actions = super::read_all_handle_action_sources();

    // refresh_scripts is called after alias removal
    // (already verified it exists globally; check there are multiple calls for both shortcut and alias)
    let refresh_count = count_occurrences(&actions, "self.refresh_scripts(cx)");
    assert!(
        refresh_count >= 2,
        "Expected refresh_scripts to be called after both shortcut and alias removal (found {refresh_count} calls)"
    );
}

#[test]
fn alias_remove_rejects_unsupported_window_type() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("\"Cannot remove alias for this item type\""),
        "Expected remove_alias to show error for unsupported item types (Window)"
    );
}

// ---------------------------------------------------------------------------
// Shortcut and alias consistency tests
// ---------------------------------------------------------------------------

#[test]
fn shortcut_and_alias_use_consistent_command_id_formats() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        count_occurrences(&actions, ".launcher_command_id()") >= 4,
        "Expected shortcut and alias handlers to share SearchResult::launcher_command_id"
    );
}

#[test]
fn shortcut_and_alias_both_call_hide_main_and_reset_after_remove() {
    let actions = super::read_all_handle_action_sources();

    // Both remove_shortcut and remove_alias should call hide_main_and_reset
    let hide_count = count_occurrences(&actions, "self.hide_main_and_reset(cx)");
    assert!(
        hide_count >= 2,
        "Expected hide_main_and_reset to be called from both shortcut and alias removal paths (found {hide_count} calls)"
    );
}

#[test]
fn shortcut_and_alias_remove_use_consistent_error_handling_pattern() {
    let actions = super::read_all_handle_action_sources();

    // Both remove handlers should match Ok(()) => success, Err(e) => error
    // Verify both patterns exist
    assert!(
        actions.contains("Self::Shortcut => format!(\"Failed to remove shortcut: {error}\")")
            && actions.contains("Self::Alias => format!(\"Failed to remove alias: {error}\")")
            && count_occurrences(&actions, "remove_action.failure_message(e)") >= 2,
        "Expected both shortcut and alias remove to format error messages consistently"
    );
}

// ---------------------------------------------------------------------------
// File search action handler tests
// ---------------------------------------------------------------------------

#[test]
fn file_search_actions_are_handled() {
    let actions = super::read_all_handle_action_sources();

    for action_id in &[
        "open_file",
        "open_directory",
        "quick_look",
        "open_with",
        "show_info",
    ] {
        assert!(
            actions.contains(&format!("\"{}\"", action_id)),
            "Expected handle_action/ to handle '{action_id}'"
        );
    }
}

#[test]
fn file_search_actions_dispatch_to_file_search_module() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("crate::file_search::open_file(&path)"),
        "Expected open_file to dispatch to file_search::open_file"
    );
    assert!(
        actions.contains("crate::file_search::quick_look(&path)"),
        "Expected quick_look to dispatch to file_search::quick_look"
    );
    assert!(
        actions.contains("crate::file_search::open_with(&path)"),
        "Expected open_with to dispatch to file_search::open_with"
    );
    assert!(
        actions.contains("crate::file_search::show_info(&path)"),
        "Expected show_info to dispatch to file_search::show_info"
    );
}

#[test]
fn file_search_actions_require_file_search_actions_path() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("if let Some(path) = self.file_search_actions_path.clone()"),
        "Expected file search actions to guard on file_search_actions_path being set"
    );
}

#[test]
fn file_search_actions_show_success_hud() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("file_search_action_success_hud(action_id)"),
        "Expected file search actions to use file_search_action_success_hud for success feedback"
    );
}

#[test]
fn file_search_actions_show_error_toast_on_failure() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("file_search_action_error_hud_prefix(action_id)"),
        "Expected file search actions to use file_search_action_error_hud_prefix for error feedback"
    );

    // Error toast should include both the prefix and the error message
    assert!(
        actions.contains("format!(\"{}: {}\", prefix, e)"),
        "Expected file search error toast to combine prefix with error message"
    );
}

#[test]
fn file_search_open_file_hides_main_window() {
    let actions = super::read_all_handle_action_sources();

    // open_file and open_directory should hide the window
    assert!(
        actions.contains("fn hides_main_after_success(self) -> bool")
            && actions.contains("matches!(self, Self::Open)")
            && actions.contains("file_action.hides_main_after_success()"),
        "Expected open_file/open_directory to check action_id for hide_main_and_reset"
    );
}

#[test]
fn file_search_actions_clear_path_after_completion() {
    let actions = super::read_all_handle_action_sources();

    // file_search_actions_path is now consumed through take() and also explicitly
    // cleared on mutation error paths.
    let clear_count = count_occurrences(&actions, "self.file_search_actions_path = None")
        + count_occurrences(&actions, "self.file_search_actions_path.take()");
    assert!(
        clear_count >= 2,
        "Expected file_search_actions_path to be cleared on both success and error paths (found {clear_count})"
    );
}

#[test]
fn copy_filename_action_is_handled() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("\"copy_filename\""),
        "Expected handle_action/ to handle copy_filename"
    );
}

#[test]
fn copy_filename_extracts_filename_from_path() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains(".file_name()"),
        "Expected copy_filename to extract filename component from path"
    );
}

#[test]
fn copy_filename_shows_error_when_no_filename() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("\"No file selected\""),
        "Expected copy_filename to report that no file target was resolved"
    );
}

#[test]
fn copy_filename_uses_clipboard_feedback_helper() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("self.copy_to_clipboard_with_feedback("),
        "Expected copy_filename to use copy_to_clipboard_with_feedback"
    );
}

// ---------------------------------------------------------------------------
// Cancel action clears file search state
// ---------------------------------------------------------------------------

#[test]
fn cancel_action_clears_file_search_actions_path() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("\"__cancel__\""),
        "Expected handle_action/ to handle __cancel__"
    );
    assert!(
        actions.contains("self.file_search_actions_path = None"),
        "Expected __cancel__ to clear file_search_actions_path"
    );
}

// ---------------------------------------------------------------------------
// Cross-category consistency: all action categories log with tracing
// ---------------------------------------------------------------------------

#[test]
fn all_action_categories_use_tracing_for_logging() {
    let actions = super::read_all_handle_action_sources();

    // Shortcut actions
    assert!(
        actions.contains(
            "tracing::info!(category = \"UI\", action = action_id, \"action triggered\")"
        ) || actions.contains("tracing::info!(category = \"UI\","),
        "Expected shortcut actions to use tracing for logging"
    );

    // Remove shortcut
    assert!(
        actions.contains("tracing::info!(category = \"UI\", \"remove shortcut action\")"),
        "Expected remove_shortcut to log with tracing"
    );

    // Remove alias
    assert!(
        actions.contains("tracing::info!(category = \"UI\", \"remove alias action\")"),
        "Expected remove_alias to log with tracing"
    );

    // File search actions
    assert!(
        actions.contains("tracing::info!(category = \"UI\", action = action_id, path ="),
        "Expected file search actions to log with tracing including path"
    );
}

// ---------------------------------------------------------------------------
// Error logging: error paths use tracing::error
// ---------------------------------------------------------------------------

#[test]
fn shortcut_and_alias_error_paths_log_with_tracing_error() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("tracing::error!(error = %e, \"failed to remove shortcut\")"),
        "Expected shortcut removal failure to log with tracing::error"
    );
    assert!(
        actions.contains("tracing::error!(error = %e, \"failed to remove alias\")"),
        "Expected alias removal failure to log with tracing::error"
    );
}

#[test]
fn file_search_error_path_logs_with_tracing_error() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("tracing::error!(action = action_id, path ="),
        "Expected file search action failure to log with tracing::error including action and path"
    );
}

// ---------------------------------------------------------------------------
// HUD duration constants: verify consistent usage
// ---------------------------------------------------------------------------

#[test]
fn shortcut_and_alias_success_use_hud_medium_ms() {
    let actions = super::read_all_handle_action_sources();

    // Both "Shortcut removed" and "Alias removed" should use HUD_MEDIUM_MS
    // The show_hud call is split across lines, so check for the string and duration separately
    assert!(
        actions.contains("\"Shortcut removed\"") && actions.contains("HUD_MEDIUM_MS"),
        "Expected shortcut removal success HUD to use HUD_MEDIUM_MS"
    );
    assert!(
        actions.contains("\"Alias removed\"") && actions.contains("HUD_MEDIUM_MS"),
        "Expected alias removal success HUD to use HUD_MEDIUM_MS"
    );
}

#[test]
fn file_search_success_uses_hud_short_ms() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("Some(HUD_SHORT_MS)"),
        "Expected file search success HUD to use HUD_SHORT_MS"
    );
}
