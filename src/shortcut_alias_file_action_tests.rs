//! Tests for shortcut, alias, and file search action handlers.
//!
//! These tests verify:
//! - Shortcut add/update/remove handlers exist and follow consistent patterns
//! - Alias add/update/remove handlers exist and follow consistent patterns
//! - File search actions (open_file, quick_look, open_with, show_info) follow patterns
//! - Error paths show Toast feedback for all action categories
//! - Selection-required guards are present on all interactive actions

use crate::test_utils::{count_occurrences, read_source as read};

// ---------------------------------------------------------------------------
// Shortcut action handler tests
// ---------------------------------------------------------------------------

#[test]
fn shortcut_add_and_update_actions_are_handled() {
    let actions = read("src/app_actions/handle_action.rs");

    assert!(
        actions.contains("\"configure_shortcut\" | \"add_shortcut\" | \"update_shortcut\""),
        "Expected handle_action.rs to handle configure_shortcut, add_shortcut, and update_shortcut"
    );
}

#[test]
fn shortcut_add_opens_recorder_for_non_script_types() {
    let actions = read("src/app_actions/handle_action.rs");

    // Scriptlets, BuiltIns, and Apps should open the shortcut recorder
    assert!(
        actions.contains("self.show_shortcut_recorder(command_id, command_name, cx)"),
        "Expected shortcut actions to use show_shortcut_recorder for non-script types"
    );

    // Verify recorder is called for multiple item types (at least 3 calls for scriptlet/builtin/app)
    let recorder_calls = count_occurrences(&actions, "self.show_shortcut_recorder(");
    assert!(
        recorder_calls >= 3,
        "Expected show_shortcut_recorder to be called for at least scriptlet, builtin, and app types (found {recorder_calls} calls)"
    );
}

#[test]
fn shortcut_add_opens_editor_for_scripts_and_agents() {
    let actions = read("src/app_actions/handle_action.rs");

    // The configure_shortcut/add_shortcut/update_shortcut block should call edit_script
    // for Script and Agent types
    assert!(
        actions.contains("self.edit_script(&m.script.path)")
            && actions.contains("self.edit_script(&m.agent.path)"),
        "Expected shortcut actions to open editor for Script and Agent types"
    );
}

#[test]
fn shortcut_add_shows_error_for_window_type() {
    let actions = read("src/app_actions/handle_action.rs");

    assert!(
        actions.contains("Window shortcuts not supported - windows are transient"),
        "Expected shortcut actions to show error toast for Window type"
    );
}

#[test]
fn shortcut_add_shows_selection_required_on_no_selection() {
    let actions = read("src/app_actions/handle_action.rs");

    // The shortcut block should call selection_required_message_for_action when no item selected
    assert!(
        actions.contains("selection_required_message_for_action(action_id)"),
        "Expected shortcut actions to show selection-required message when nothing is selected"
    );
}

#[test]
fn shortcut_remove_action_is_handled() {
    let actions = read("src/app_actions/handle_action.rs");

    assert!(
        actions.contains("\"remove_shortcut\""),
        "Expected handle_action.rs to handle remove_shortcut"
    );
}

#[test]
fn shortcut_remove_calls_persistence_layer() {
    let actions = read("src/app_actions/handle_action.rs");

    assert!(
        actions.contains("crate::shortcuts::remove_shortcut_override(&command_id)"),
        "Expected remove_shortcut to call shortcuts::remove_shortcut_override"
    );
}

#[test]
fn shortcut_remove_shows_success_hud_on_ok() {
    let actions = read("src/app_actions/handle_action.rs");

    assert!(
        actions.contains("\"Shortcut removed\""),
        "Expected remove_shortcut to show 'Shortcut removed' HUD on success"
    );
}

#[test]
fn shortcut_remove_shows_error_toast_on_failure() {
    let actions = read("src/app_actions/handle_action.rs");

    assert!(
        actions.contains("\"Failed to remove shortcut: {}\""),
        "Expected remove_shortcut to show error toast when persistence fails"
    );
}

#[test]
fn shortcut_remove_refreshes_scripts_after_success() {
    let actions = read("src/app_actions/handle_action.rs");

    // After removing shortcut, scripts should be refreshed to update display
    assert!(
        actions.contains("self.refresh_scripts(cx)"),
        "Expected remove_shortcut to call refresh_scripts after successful removal"
    );
}

#[test]
fn shortcut_remove_rejects_unsupported_window_type() {
    let actions = read("src/app_actions/handle_action.rs");

    assert!(
        actions.contains("\"Cannot remove shortcut for this item type\""),
        "Expected remove_shortcut to show error for unsupported item types (Window)"
    );
}

#[test]
fn shortcut_remove_builds_command_id_for_all_supported_types() {
    let actions = read("src/app_actions/handle_action.rs");

    // The remove_shortcut block should build command_id for script, scriptlet, builtin, app, agent, fallback
    for prefix in &["script/", "scriptlet/", "builtin/", "app/", "agent/", "fallback/"] {
        assert!(
            actions.contains(&format!("format!(\"{}\"", prefix))
                || actions.contains(&format!("format!(\"{{}}\", ", ))
                || actions.contains(prefix),
            "Expected remove_shortcut to build command_id with prefix '{prefix}'"
        );
    }
}

// ---------------------------------------------------------------------------
// Alias action handler tests
// ---------------------------------------------------------------------------

#[test]
fn alias_add_and_update_actions_are_handled() {
    let actions = read("src/app_actions/handle_action.rs");

    assert!(
        actions.contains("\"add_alias\" | \"update_alias\""),
        "Expected handle_action.rs to handle add_alias and update_alias"
    );
}

#[test]
fn alias_add_opens_alias_input_dialog() {
    let actions = read("src/app_actions/handle_action.rs");

    assert!(
        actions.contains("self.show_alias_input(command_id, command_name, cx)"),
        "Expected alias actions to open alias input dialog via show_alias_input"
    );
}

#[test]
fn alias_add_shows_error_for_window_type() {
    let actions = read("src/app_actions/handle_action.rs");

    assert!(
        actions.contains("Window aliases not supported - windows are transient"),
        "Expected alias actions to show error toast for Window type"
    );
}

#[test]
fn alias_add_shows_selection_required_on_no_selection() {
    let actions = read("src/app_actions/handle_action.rs");

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
    let actions = read("src/app_actions/handle_action.rs");

    assert!(
        actions.contains("\"remove_alias\""),
        "Expected handle_action.rs to handle remove_alias"
    );
}

#[test]
fn alias_remove_calls_persistence_layer() {
    let actions = read("src/app_actions/handle_action.rs");

    assert!(
        actions.contains("crate::aliases::remove_alias_override(&command_id)"),
        "Expected remove_alias to call aliases::remove_alias_override"
    );
}

#[test]
fn alias_remove_shows_success_hud_on_ok() {
    let actions = read("src/app_actions/handle_action.rs");

    assert!(
        actions.contains("\"Alias removed\""),
        "Expected remove_alias to show 'Alias removed' HUD on success"
    );
}

#[test]
fn alias_remove_shows_error_toast_on_failure() {
    let actions = read("src/app_actions/handle_action.rs");

    assert!(
        actions.contains("\"Failed to remove alias: {}\""),
        "Expected remove_alias to show error toast when persistence fails"
    );
}

#[test]
fn alias_remove_refreshes_scripts_after_success() {
    let actions = read("src/app_actions/handle_action.rs");

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
    let actions = read("src/app_actions/handle_action.rs");

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
    let actions = read("src/app_actions/handle_action.rs");

    // Both shortcut and alias handlers should use the same format patterns
    // Verify "script/", "scriptlet/", "builtin/", "app/", "agent/", "fallback/" prefixes
    // appear in both the shortcut and alias sections

    // Count format! calls with these prefixes - should be at least 2 of each
    // (one in shortcut handler, one in alias handler)
    for prefix in &["\"script/{}\"", "\"scriptlet/{}\"", "\"builtin/{}\"", "\"app/{}\"", "\"agent/{}\"", "\"fallback/{}\""] {
        let count = count_occurrences(&actions, prefix);
        assert!(
            count >= 2,
            "Expected command_id format '{prefix}' to appear in both shortcut and alias handlers (found {count})"
        );
    }
}

#[test]
fn shortcut_and_alias_both_call_hide_main_and_reset_after_remove() {
    let actions = read("src/app_actions/handle_action.rs");

    // Both remove_shortcut and remove_alias should call hide_main_and_reset
    let hide_count = count_occurrences(&actions, "self.hide_main_and_reset(cx)");
    assert!(
        hide_count >= 2,
        "Expected hide_main_and_reset to be called from both shortcut and alias removal paths (found {hide_count} calls)"
    );
}

#[test]
fn shortcut_and_alias_remove_use_consistent_error_handling_pattern() {
    let actions = read("src/app_actions/handle_action.rs");

    // Both remove handlers should match Ok(()) => success, Err(e) => error
    // Verify both patterns exist
    assert!(
        actions.contains("\"Failed to remove shortcut: {}\"")
            && actions.contains("\"Failed to remove alias: {}\""),
        "Expected both shortcut and alias remove to format error messages consistently"
    );
}

// ---------------------------------------------------------------------------
// File search action handler tests
// ---------------------------------------------------------------------------

#[test]
fn file_search_actions_are_handled() {
    let actions = read("src/app_actions/handle_action.rs");

    for action_id in &["open_file", "open_directory", "quick_look", "open_with", "show_info"] {
        assert!(
            actions.contains(&format!("\"{}\"", action_id)),
            "Expected handle_action.rs to handle '{action_id}'"
        );
    }
}

#[test]
fn file_search_actions_dispatch_to_file_search_module() {
    let actions = read("src/app_actions/handle_action.rs");

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
    let actions = read("src/app_actions/handle_action.rs");

    assert!(
        actions.contains("if let Some(path) = self.file_search_actions_path.clone()"),
        "Expected file search actions to guard on file_search_actions_path being set"
    );
}

#[test]
fn file_search_actions_show_success_hud() {
    let actions = read("src/app_actions/handle_action.rs");

    assert!(
        actions.contains("file_search_action_success_hud(action_id)"),
        "Expected file search actions to use file_search_action_success_hud for success feedback"
    );
}

#[test]
fn file_search_actions_show_error_toast_on_failure() {
    let actions = read("src/app_actions/handle_action.rs");

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
    let actions = read("src/app_actions/handle_action.rs");

    // open_file and open_directory should hide the window
    assert!(
        actions.contains("action_id == \"open_file\" || action_id == \"open_directory\""),
        "Expected open_file/open_directory to check action_id for hide_main_and_reset"
    );
}

#[test]
fn file_search_actions_clear_path_after_completion() {
    let actions = read("src/app_actions/handle_action.rs");

    // file_search_actions_path should be cleared after both success and error
    let clear_count = count_occurrences(&actions, "self.file_search_actions_path = None");
    assert!(
        clear_count >= 2,
        "Expected file_search_actions_path to be cleared on both success and error paths (found {clear_count})"
    );
}

#[test]
fn copy_filename_action_is_handled() {
    let actions = read("src/app_actions/handle_action.rs");

    assert!(
        actions.contains("\"copy_filename\""),
        "Expected handle_action.rs to handle copy_filename"
    );
}

#[test]
fn copy_filename_extracts_filename_from_path() {
    let actions = read("src/app_actions/handle_action.rs");

    assert!(
        actions.contains(".file_name()"),
        "Expected copy_filename to extract filename component from path"
    );
}

#[test]
fn copy_filename_shows_error_when_no_filename() {
    let actions = read("src/app_actions/handle_action.rs");

    assert!(
        actions.contains("\"No filename found for selected path\""),
        "Expected copy_filename to show error toast when path has no filename"
    );
}

#[test]
fn copy_filename_uses_clipboard_feedback_helper() {
    let actions = read("src/app_actions/handle_action.rs");

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
    let actions = read("src/app_actions/handle_action.rs");

    assert!(
        actions.contains("\"__cancel__\""),
        "Expected handle_action.rs to handle __cancel__"
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
    let actions = read("src/app_actions/handle_action.rs");

    // Shortcut actions
    assert!(
        actions.contains("tracing::info!(category = \"UI\", action = action_id, \"action triggered\")")
            || actions.contains("tracing::info!(category = \"UI\","),
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
    let actions = read("src/app_actions/handle_action.rs");

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
    let actions = read("src/app_actions/handle_action.rs");

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
    let actions = read("src/app_actions/handle_action.rs");

    // Both "Shortcut removed" and "Alias removed" should use HUD_MEDIUM_MS
    assert!(
        actions.contains("\"Shortcut removed\".to_string(), Some(HUD_MEDIUM_MS)"),
        "Expected shortcut removal success HUD to use HUD_MEDIUM_MS"
    );
    assert!(
        actions.contains("\"Alias removed\".to_string(), Some(HUD_MEDIUM_MS)"),
        "Expected alias removal success HUD to use HUD_MEDIUM_MS"
    );
}

#[test]
fn file_search_success_uses_hud_short_ms() {
    let actions = read("src/app_actions/handle_action.rs");

    assert!(
        actions.contains("Some(HUD_SHORT_MS)"),
        "Expected file search success HUD to use HUD_SHORT_MS"
    );
}
