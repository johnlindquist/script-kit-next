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
fn shortcut_actions_open_editor_for_scripts() {
    let content = super::read_all_handle_action_sources();

    let shortcut_pos = content
        .find("\"configure_shortcut\" | \"add_shortcut\" | \"update_shortcut\"")
        .expect("Expected shortcut action handler");
    let block = &content[shortcut_pos..content.len().min(shortcut_pos + 5000)];

    assert!(
        block.contains("edit_script(") && block.contains("SearchResult::Script("),
        "Shortcut actions should open the editor for Script items (shortcut is in file comment)"
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
        block.contains("Shortcuts not supported for this item type"),
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
        block.contains("remove_shortcut_override("),
        "remove_shortcut should call remove_shortcut_override for persistence"
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
        block.contains("Aliases not supported for this item type"),
        "Alias actions should reject Window items with clear error message"
    );
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
