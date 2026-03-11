// Regression tests for scriptlet ranking and scriptlet action handlers:
// reset_ranking, scriptlet_action:* prefix.

// ---------------------------------------------------------------------------
// reset_ranking — success path
// ---------------------------------------------------------------------------

#[test]
fn reset_ranking_removes_frecency_entry_and_refreshes() {
    let content = super::read_all_handle_action_sources();

    let reset_pos = content
        .find("\"reset_ranking\"")
        .expect("Expected reset_ranking action handler");
    let block = &content[reset_pos..content.len().min(reset_pos + 3000)];

    assert!(
        block.contains("frecency_store.remove("),
        "reset_ranking should remove the frecency entry"
    );
    assert!(
        block.contains("frecency_store.save()"),
        "reset_ranking should persist the updated frecency store"
    );
    assert!(
        block.contains("invalidate_grouped_cache()"),
        "reset_ranking should invalidate the grouped cache"
    );
    assert!(
        block.contains("refresh_scripts(cx)"),
        "reset_ranking should refresh scripts to rebuild the list"
    );
}

#[test]
fn reset_ranking_shows_hud_with_item_name() {
    let content = super::read_all_handle_action_sources();

    let reset_pos = content
        .find("\"reset_ranking\"")
        .expect("Expected reset_ranking action handler");
    let block = &content[reset_pos..content.len().min(reset_pos + 3000)];

    assert!(
        block.contains("Ranking reset for"),
        "reset_ranking should show HUD with item name on success"
    );
    assert!(
        block.contains("HUD_MEDIUM_MS"),
        "reset_ranking should use named duration constant"
    );
}

// ---------------------------------------------------------------------------
// reset_ranking — edge cases
// ---------------------------------------------------------------------------

#[test]
fn reset_ranking_shows_feedback_when_no_frecency_entry() {
    let content = super::read_all_handle_action_sources();

    let reset_pos = content
        .find("\"reset_ranking\"")
        .expect("Expected reset_ranking action handler");
    let block = &content[reset_pos..content.len().min(reset_pos + 3000)];

    assert!(
        block.contains("Item has no ranking to reset"),
        "reset_ranking should show feedback when item has no frecency entry"
    );
}

#[test]
fn reset_ranking_shows_error_when_no_selection() {
    let content = super::read_all_handle_action_sources();

    let reset_pos = content
        .find("\"reset_ranking\"")
        .expect("Expected reset_ranking action handler");
    let block = &content[reset_pos..content.len().min(reset_pos + 3000)];

    assert!(
        block.contains("selection_required_message_for_action(action_id)"),
        "reset_ranking should use selection_required_message when nothing is selected"
    );
}

// ---------------------------------------------------------------------------
// scriptlet_action:* — success path
// ---------------------------------------------------------------------------

#[test]
fn scriptlet_action_prefix_is_handled() {
    let content = super::read_all_handle_action_sources();

    assert!(
        content.contains("action_id.starts_with(\"scriptlet_action:\")"),
        "handle_action should match scriptlet_action: prefix for scriptlet-specific actions"
    );
}

#[test]
fn scriptlet_action_strips_prefix_and_logs() {
    let content = super::read_all_handle_action_sources();

    let scriptlet_pos = content
        .find("starts_with(\"scriptlet_action:\")")
        .expect("Expected scriptlet_action handler");
    let block = &content[scriptlet_pos..content.len().min(scriptlet_pos + 12000)];

    assert!(
        block.contains("strip_prefix(\"scriptlet_action:\")"),
        "scriptlet_action handler should strip the prefix to get the action command"
    );
    assert!(
        block.contains("scriptlet action triggered"),
        "scriptlet_action handler should log the triggered action"
    );
}

// ---------------------------------------------------------------------------
// scriptlet_action:* — error path
// ---------------------------------------------------------------------------

#[test]
fn scriptlet_action_shows_error_when_no_selection() {
    let content = super::read_all_handle_action_sources();

    let scriptlet_pos = content
        .find("starts_with(\"scriptlet_action:\")")
        .expect("Expected scriptlet_action handler");
    let block = &content[scriptlet_pos..content.len().min(scriptlet_pos + 12000)];

    assert!(
        block.contains("selection_required_message_for_action(action_id)"),
        "scriptlet_action should use selection_required_message when nothing is selected"
    );
}

#[test]
fn scriptlet_action_shows_error_when_item_is_not_scriptlet() {
    let content = super::read_all_handle_action_sources();

    let scriptlet_pos = content
        .find("starts_with(\"scriptlet_action:\")")
        .expect("Expected scriptlet_action handler");
    let block = &content[scriptlet_pos..content.len().min(scriptlet_pos + 12000)];

    assert!(
        block.contains("SearchResult::Scriptlet("),
        "scriptlet_action handler should check that the selected item is a Scriptlet"
    );
}
