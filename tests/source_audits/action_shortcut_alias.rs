// Regression tests for shortcut and alias action handlers:
// add_shortcut/update_shortcut, remove_shortcut, add_alias/update_alias, remove_alias.

// ---------------------------------------------------------------------------
// add_shortcut / update_shortcut — success path
// ---------------------------------------------------------------------------

#[test]
fn shortcut_actions_open_recorder_for_non_scripts() {
    let content = super::read_all_handle_action_sources();

    let shortcut_pos = content
        .find("\"configure_shortcut\" | \"add_shortcut\" | \"update_shortcut\"")
        .expect("Expected shortcut action handler");
    let block = &content[shortcut_pos..content.len().min(shortcut_pos + 5000)];

    assert!(
        block.contains("show_shortcut_recorder("),
        "Shortcut actions should open the shortcut recorder for non-script items"
    );
}

#[test]
fn shortcut_actions_use_launcher_command_ids() {
    let content = super::read_all_handle_action_sources();

    let shortcut_pos = content
        .find("\"configure_shortcut\" | \"add_shortcut\" | \"update_shortcut\"")
        .expect("Expected shortcut action handler");
    let block = &content[shortcut_pos..content.len().min(shortcut_pos + 5000)];

    assert!(
        block.contains("result.launcher_command_id()"),
        "Shortcut actions should resolve command IDs through SearchResult::launcher_command_id"
    );
}

// ---------------------------------------------------------------------------
// add_shortcut / update_shortcut — error path
// ---------------------------------------------------------------------------

#[test]
fn shortcut_actions_show_error_when_no_selection() {
    let content = super::read_all_handle_action_sources();

    let shortcut_pos = content
        .find("\"configure_shortcut\" | \"add_shortcut\" | \"update_shortcut\"")
        .expect("Expected shortcut action handler");
    let block = &content[shortcut_pos..content.len().min(shortcut_pos + 5000)];

    assert!(
        block.contains("selection_required_message_for_action(action_id)"),
        "Shortcut actions should use selection_required_message when nothing is selected"
    );
}

#[test]
fn shortcut_actions_reject_window_items() {
    let content = super::read_all_handle_action_sources();

    let shortcut_pos = content
        .find("\"configure_shortcut\" | \"add_shortcut\" | \"update_shortcut\"")
        .expect("Expected shortcut action handler");
    let block = &content[shortcut_pos..content.len().min(shortcut_pos + 5000)];

    assert!(
        block.contains("shortcut_action.unsupported_message()")
            && content.contains("Shortcuts not supported for this item type"),
        "Shortcut actions should reject Window items with clear error message"
    );
}

// ---------------------------------------------------------------------------
// remove_shortcut — success path
// ---------------------------------------------------------------------------

#[test]
fn remove_shortcut_calls_persistence_and_shows_hud() {
    let content = super::read_all_handle_action_sources();

    let remove_pos = content
        .find("\"remove_shortcut\"")
        .expect("Expected remove_shortcut action handler");
    let block = &content[remove_pos..content.len().min(remove_pos + 5000)];

    assert!(
        block.contains("remove_config_command_shortcut("),
        "remove_shortcut should remove command shortcuts through config.ts"
    );
    assert!(
        block.contains("Shortcut removed"),
        "remove_shortcut should show HUD on success"
    );
    assert!(
        block.contains("refresh_scripts(cx)"),
        "remove_shortcut should refresh scripts to update shortcut display"
    );
}

#[test]
fn remove_shortcut_checks_live_unregister_result() {
    let content = super::read_all_handle_action_sources();

    let remove_pos = content
        .find("\"remove_shortcut\"")
        .expect("Expected remove_shortcut action handler");
    let block = &content[remove_pos..content.len().min(remove_pos + 5000)];

    assert!(
        block.contains("match crate::hotkeys::unregister_dynamic_shortcut(&command_id)"),
        "remove_shortcut should inspect live unregister result instead of discarding it"
    );
    assert!(
        block.contains("Config shortcut removed, but live unregister failed"),
        "remove_shortcut should log recoverable live unregister failures after config removal"
    );
}

#[test]
fn shortcut_assignment_wires_live_conflict_checker() {
    let recorder = super::read_source("src/app_impl/shortcut_recorder.rs");

    assert!(
        recorder.contains(".with_conflict_checker(")
            && recorder.contains("shortcut_conflict_for_recording("),
        "shortcut recorder entry paths should wire the live hotkey route conflict checker"
    );
}

#[test]
fn shortcut_save_rechecks_live_conflict_before_config_write() {
    let recorder = super::read_source("src/app_impl/shortcut_recorder.rs");
    let save_pos = recorder
        .find("pub(crate) fn handle_shortcut_save")
        .expect("handle_shortcut_save not found");
    let block = &recorder[save_pos..recorder.len().min(save_pos + 5000)];
    let conflict_pos = block
        .find("shortcut_conflict_for_recording(&command_id, &shortcut_str)")
        .expect("handle_shortcut_save should re-check live conflicts");
    let write_pos = block
        .find("self.write_config_command_shortcut(")
        .expect("handle_shortcut_save should write config through the wrapper");

    assert!(
        conflict_pos < write_pos,
        "shortcut saves should re-check live conflicts before mutating config.ts"
    );
}

// ---------------------------------------------------------------------------
// remove_shortcut — error path
// ---------------------------------------------------------------------------

#[test]
fn remove_shortcut_shows_error_on_persistence_failure() {
    let content = super::read_all_handle_action_sources();

    let remove_pos = content
        .find("\"remove_shortcut\"")
        .expect("Expected remove_shortcut action handler");
    let block = &content[remove_pos..content.len().min(remove_pos + 5000)];

    assert!(
        block.contains("Failed to remove shortcut:"),
        "remove_shortcut should surface error on persistence failure"
    );
    // Error may be surfaced via DispatchOutcome::error (centralized feedback)
    // or show_error_toast (inline feedback).
    assert!(
        block.contains("DispatchOutcome::error(") || block.contains("show_error_toast("),
        "remove_shortcut should return DispatchOutcome::error or call show_error_toast"
    );
}

#[test]
fn remove_shortcut_rejects_unsupported_item_types() {
    let content = super::read_all_handle_action_sources();

    let remove_pos = content
        .find("\"remove_shortcut\"")
        .expect("Expected remove_shortcut action handler");
    let block = &content[remove_pos..content.len().min(remove_pos + 5000)];

    assert!(
        block.contains("Cannot remove shortcut for this item type"),
        "remove_shortcut should show error for unsupported item types (e.g. Window)"
    );
}

// ---------------------------------------------------------------------------
// add_alias / update_alias — success path
// ---------------------------------------------------------------------------

#[test]
fn alias_actions_open_alias_input() {
    let content = super::read_all_handle_action_sources();

    let alias_pos = content
        .find("\"add_alias\" | \"update_alias\"")
        .expect("Expected alias action handler");
    let block = &content[alias_pos..content.len().min(alias_pos + 5000)];

    assert!(
        block.contains("show_alias_input("),
        "Alias actions should open the alias input dialog"
    );
}

// ---------------------------------------------------------------------------
// add_alias / update_alias — error path
// ---------------------------------------------------------------------------

#[test]
fn alias_actions_show_error_when_no_selection() {
    let content = super::read_all_handle_action_sources();

    let alias_pos = content
        .find("\"add_alias\" | \"update_alias\"")
        .expect("Expected alias action handler");
    let block = &content[alias_pos..content.len().min(alias_pos + 5000)];

    assert!(
        block.contains("selection_required_message_for_action(action_id)"),
        "Alias actions should use selection_required_message when nothing is selected"
    );
}

#[test]
fn alias_actions_reject_window_items() {
    let content = super::read_all_handle_action_sources();

    let alias_pos = content
        .find("\"add_alias\" | \"update_alias\"")
        .expect("Expected alias action handler");
    let block = &content[alias_pos..content.len().min(alias_pos + 5000)];

    assert!(
        block.contains("alias_action.unsupported_message()")
            && content.contains("Aliases not supported for this item type"),
        "Alias actions should reject Window items with clear error message"
    );
}

#[test]
fn agent_script_context_does_not_advertise_unsupported_shortcut_or_alias_actions() {
    let builder = super::read_source("src/actions/builders/script_context.rs");
    let shortcut_start = builder
        .find("if !script.is_agent")
        .expect("script context builder should guard shortcut/alias actions for agents");
    let agent_actions_start = builder
        .find("if script.is_agent {")
        .expect("script context builder should still define agent-specific actions");
    let agent_actions_end = builder[agent_actions_start..]
        .find("let deeplink_name = to_deeplink_name")
        .map(|offset| agent_actions_start + offset)
        .expect(
            "script context builder should leave the agent-specific block before deeplink actions",
        );
    let guarded_block = &builder[shortcut_start..agent_actions_start];

    for action_id in [
        "\"update_shortcut\"",
        "\"remove_shortcut\"",
        "\"add_shortcut\"",
        "\"update_alias\"",
        "\"remove_alias\"",
        "\"add_alias\"",
    ] {
        assert!(
            guarded_block.contains(action_id),
            "{action_id} should only be built inside the non-agent shortcut/alias guard"
        );
    }

    let agent_block = &builder[agent_actions_start..agent_actions_end];
    for action_id in [
        "\"update_shortcut\"",
        "\"remove_shortcut\"",
        "\"add_shortcut\"",
        "\"update_alias\"",
        "\"remove_alias\"",
        "\"add_alias\"",
    ] {
        assert!(
            !agent_block.contains(action_id),
            "{action_id} must not be advertised by the agent-specific action block"
        );
    }
}

// ---------------------------------------------------------------------------
// remove_alias — success path
// ---------------------------------------------------------------------------

#[test]
fn remove_alias_calls_persistence_and_shows_hud() {
    let content = super::read_all_handle_action_sources();

    let remove_pos = content
        .find("\"remove_alias\"")
        .expect("Expected remove_alias action handler");
    let block = &content[remove_pos..content.len().min(remove_pos + 5000)];

    assert!(
        block.contains("remove_alias_override("),
        "remove_alias should call remove_alias_override for persistence"
    );
    assert!(
        block.contains("Alias removed"),
        "remove_alias should show HUD on success"
    );
    assert!(
        block.contains("refresh_scripts(cx)"),
        "remove_alias should refresh scripts to update alias display"
    );
}

// ---------------------------------------------------------------------------
// remove_alias — error path
// ---------------------------------------------------------------------------

#[test]
fn remove_alias_shows_error_on_persistence_failure() {
    let content = super::read_all_handle_action_sources();

    let remove_pos = content
        .find("\"remove_alias\"")
        .expect("Expected remove_alias action handler");
    let block = &content[remove_pos..content.len().min(remove_pos + 5000)];

    assert!(
        block.contains("Failed to remove alias:"),
        "remove_alias should surface error on persistence failure"
    );
    // Error may be surfaced via DispatchOutcome::error (centralized feedback)
    // or show_error_toast (inline feedback).
    assert!(
        block.contains("DispatchOutcome::error(") || block.contains("show_error_toast("),
        "remove_alias should return DispatchOutcome::error or call show_error_toast"
    );
}

#[test]
fn remove_alias_rejects_unsupported_item_types() {
    let content = super::read_all_handle_action_sources();

    let remove_pos = content
        .find("\"remove_alias\"")
        .expect("Expected remove_alias action handler");
    let block = &content[remove_pos..content.len().min(remove_pos + 5000)];

    assert!(
        block.contains("Cannot remove alias for this item type"),
        "remove_alias should show error for unsupported item types (e.g. Window)"
    );
}
