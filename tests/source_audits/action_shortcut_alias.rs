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
        block.contains("shortcut_action")
            && block.contains("target_error_message(ShortcutAliasTargetError::NoSelection"),
        "Shortcut actions should derive no-selection copy from the named action state"
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
        block.contains("shortcut_action.target_error_message(")
            && block.contains("ShortcutAliasTargetError::UnsupportedItemType")
            && content.contains("Shortcuts not supported for this item type"),
        "Shortcut actions should derive unsupported-item copy from the named action state"
    );
}

// ---------------------------------------------------------------------------
// remove_shortcut — success path
// ---------------------------------------------------------------------------

#[test]
fn remove_shortcut_calls_persistence_and_shows_hud() {
    let content = super::read_all_handle_action_sources();

    let remove_pos = content
        .find("\"remove_shortcut\" => {")
        .expect("Expected remove_shortcut action handler");
    let block = &content[remove_pos..content.len().min(remove_pos + 5000)];

    assert!(
        block.contains("remove_config_command_shortcut("),
        "remove_shortcut should remove command shortcuts through config.ts"
    );
    assert!(
        block.contains("remove_action.success_hud()") && content.contains("Shortcut removed"),
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
        .find("\"remove_shortcut\" => {")
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

#[test]
fn shortcut_recorder_close_rekeys_main_filter_after_native_popup_close() {
    let recorder = super::read_source("src/app_impl/shortcut_recorder.rs");
    let close_pos = recorder
        .find("pub fn close_shortcut_recorder")
        .expect("close_shortcut_recorder not found");
    let block = &recorder[close_pos..recorder.len().min(close_pos + 2000)];
    let close_window_pos = block
        .find("close_shortcut_recorder_window(cx);")
        .expect("close_shortcut_recorder should close the native recorder window");
    let show_main_pos = block
        .find("crate::platform::show_main_window_without_activation();")
        .expect("close_shortcut_recorder should re-key the main panel after native popup close");
    let pending_focus_pos = block
        .find("app.pending_focus = Some(FocusTarget::MainFilter);")
        .expect(
            "close_shortcut_recorder should request main-filter focus after native popup close",
        );

    assert!(
        close_window_pos < show_main_pos && show_main_pos < pending_focus_pos,
        "shortcut recorder close should remove the key popup, re-key the main panel, then restore main-filter focus"
    );
    assert!(
        block.contains("app.focused_input = FocusedInput::MainFilter;"),
        "shortcut recorder close should keep focused_input aligned with the restored main filter"
    );
}

// ---------------------------------------------------------------------------
// remove_shortcut — error path
// ---------------------------------------------------------------------------

#[test]
fn remove_shortcut_shows_error_on_persistence_failure() {
    let content = super::read_all_handle_action_sources();

    let remove_pos = content
        .find("\"remove_shortcut\" => {")
        .expect("Expected remove_shortcut action handler");
    let block = &content[remove_pos..content.len().min(remove_pos + 5000)];

    assert!(
        block.contains("remove_action.failure_message(e)")
            && content.contains("Failed to remove shortcut:"),
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
        .find("\"remove_shortcut\" => {")
        .expect("Expected remove_shortcut action handler");
    let block = &content[remove_pos..content.len().min(remove_pos + 5000)];

    assert!(
        block.contains("remove_action.target_error_message(")
            && block.contains("ShortcutAliasTargetError::MissingCommandId")
            && content.contains("Cannot remove shortcut for this item type"),
        "remove_shortcut should derive missing-target copy from the named action state"
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
        block.contains("alias_action")
            && block.contains("target_error_message(ShortcutAliasTargetError::NoSelection"),
        "Alias actions should derive no-selection copy from the named action state"
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
        block.contains("alias_action.target_error_message(")
            && block.contains("ShortcutAliasTargetError::UnsupportedItemType")
            && content.contains("Aliases not supported for this item type"),
        "Alias actions should derive unsupported-item copy from the named action state"
    );
}

#[test]
fn agent_script_context_does_not_advertise_unsupported_shortcut_or_alias_actions() {
    let builder = super::read_source("src/actions/builders/script_context.rs");
    let plan_start = builder
        .find("fn preference_action_plan")
        .expect("script context builder should derive shortcut/alias actions from a plan");
    let shortcut_append_start = builder
        .find("fn append_shortcut_preference_actions")
        .expect("script context builder should append shortcut rows from the plan");
    let alias_append_start = builder
        .find("fn append_alias_preference_actions")
        .expect("script context builder should append alias rows from the plan");
    let plan_block = &builder[plan_start..shortcut_append_start];
    let guarded_block =
        &builder[shortcut_append_start..builder.len().min(alias_append_start + 2500)];

    assert!(
        plan_block.contains("if script.is_agent")
            && plan_block.contains("ScriptContextPreferenceActionPlan::AgentNoPreferenceActions"),
        "script context builder should map agents to the no-preference-action plan"
    );

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

    assert!(
        guarded_block.contains("ScriptContextPreferenceActionPlan::AgentNoPreferenceActions => {}"),
        "agent preference plans must not append shortcut or alias action rows"
    );
}

// ---------------------------------------------------------------------------
// remove_alias — success path
// ---------------------------------------------------------------------------

#[test]
fn remove_alias_calls_persistence_and_shows_hud() {
    let content = super::read_all_handle_action_sources();

    let remove_pos = content
        .find("\"remove_alias\" => {")
        .expect("Expected remove_alias action handler");
    let block = &content[remove_pos..content.len().min(remove_pos + 5000)];

    assert!(
        block.contains("remove_alias_override("),
        "remove_alias should call remove_alias_override for persistence"
    );
    assert!(
        block.contains("remove_action.success_hud()") && content.contains("Alias removed"),
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
        .find("\"remove_alias\" => {")
        .expect("Expected remove_alias action handler");
    let block = &content[remove_pos..content.len().min(remove_pos + 5000)];

    assert!(
        block.contains("remove_action.failure_message(e)")
            && content.contains("Failed to remove alias:"),
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
        .find("\"remove_alias\" => {")
        .expect("Expected remove_alias action handler");
    let block = &content[remove_pos..content.len().min(remove_pos + 5000)];

    assert!(
        block.contains("remove_action.target_error_message(")
            && block.contains("ShortcutAliasTargetError::MissingCommandId")
            && content.contains("Cannot remove alias for this item type"),
        "remove_alias should derive missing-target copy from the named action state"
    );
}
