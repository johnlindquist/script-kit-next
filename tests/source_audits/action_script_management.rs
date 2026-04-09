// Regression tests for script management action handlers:
// edit_script, reload_scripts, settings.

// ---------------------------------------------------------------------------
// edit_script — success path
// ---------------------------------------------------------------------------

#[test]
fn edit_script_uses_async_editor_launch() {
    let content = super::read_all_handle_action_sources();

    let edit_pos = content
        .find("\"edit_script\"")
        .expect("Expected edit_script action handler");
    let block = &content[edit_pos..content.len().min(edit_pos + 3000)];

    assert!(
        block.contains("launch_editor_with_feedback_async"),
        "edit_script should use async editor launch for proper error feedback"
    );
    assert!(
        block.contains("hide_main_and_reset"),
        "edit_script should hide main window on successful editor launch"
    );
}

#[test]
fn edit_script_supports_scripts_skills_and_agents() {
    let content = super::read_all_handle_action_sources();

    let edit_pos = content
        .find("\"edit_script\"")
        .expect("Expected edit_script action handler");
    let block = &content[edit_pos..content.len().min(edit_pos + 3000)];

    assert!(
        block.contains("SearchResult::Script(m)"),
        "edit_script should support Script result type"
    );
    assert!(
        block.contains("SearchResult::Skill(m)"),
        "edit_script should support Skill result type"
    );
    assert!(
        block.contains("SearchResult::Agent(m)"),
        "edit_script should support Agent result type"
    );
}

// ---------------------------------------------------------------------------
// edit_script — error paths
// ---------------------------------------------------------------------------

#[test]
fn edit_script_shows_error_for_unsupported_item_types() {
    let content = super::read_all_handle_action_sources();

    let edit_pos = content
        .find("\"edit_script\"")
        .expect("Expected edit_script action handler");
    let block = &content[edit_pos..content.len().min(edit_pos + 4000)];

    assert!(
        block.contains("Cannot edit this item type"),
        "edit_script should show error for non-editable item types"
    );
}

#[test]
fn edit_script_shows_error_when_no_selection() {
    let content = super::read_all_handle_action_sources();

    let edit_pos = content
        .find("\"edit_script\"")
        .expect("Expected edit_script action handler");
    let block = &content[edit_pos..content.len().min(edit_pos + 4000)];

    assert!(
        block.contains("selection_required_message_for_action(action_id)"),
        "edit_script should use selection_required_message when nothing is selected"
    );
}

#[test]
fn edit_script_shows_toast_on_editor_launch_failure() {
    let content = super::read_all_handle_action_sources();

    let edit_pos = content
        .find("\"edit_script\"")
        .expect("Expected edit_script action handler");
    let block = &content[edit_pos..content.len().min(edit_pos + 4000)];

    assert!(
        block.contains("show_error_toast_with_code("),
        "edit_script should show error toast when editor launch fails"
    );
}

// ---------------------------------------------------------------------------
// reload_scripts — success path
// ---------------------------------------------------------------------------

#[test]
fn reload_scripts_refreshes_and_shows_hud() {
    let content = super::read_all_handle_action_sources();

    let reload_pos = content
        .find("\"reload_scripts\"")
        .expect("Expected reload_scripts action handler");
    let block = &content[reload_pos..content.len().min(reload_pos + 3000)];

    assert!(
        block.contains("refresh_scripts(cx)"),
        "reload_scripts should call refresh_scripts"
    );
    assert!(
        block.contains("Scripts reloaded"),
        "reload_scripts should show HUD confirming reload"
    );
    assert!(
        block.contains("HUD_SHORT_MS"),
        "reload_scripts should use named HUD duration constant"
    );
}

// ---------------------------------------------------------------------------
// settings — success path
// ---------------------------------------------------------------------------

#[test]
fn settings_opens_config_in_editor() {
    let content = super::read_all_handle_action_sources();

    let settings_pos = content
        .find("\"settings\"")
        .expect("Expected settings action handler");
    let block = &content[settings_pos..content.len().min(settings_pos + 3000)];

    assert!(
        block.contains("config.ts"),
        "settings should open config.ts"
    );
    assert!(
        block.contains("get_editor()"),
        "settings should use the configured editor"
    );
    assert!(
        block.contains("Opening config.ts in"),
        "settings should show HUD with editor name on success"
    );
}

// ---------------------------------------------------------------------------
// settings — error path
// ---------------------------------------------------------------------------

#[test]
fn settings_shows_error_when_editor_fails() {
    let content = super::read_all_handle_action_sources();

    let settings_pos = content
        .find("\"settings\"")
        .expect("Expected settings action handler");
    let block = &content[settings_pos..content.len().min(settings_pos + 4000)];

    assert!(
        block.contains("Failed to open") && block.contains("for settings"),
        "settings should show error toast when editor launch fails"
    );
    assert!(
        block.contains("show_error_toast("),
        "settings should use show_error_toast for consistent error reporting"
    );
}
