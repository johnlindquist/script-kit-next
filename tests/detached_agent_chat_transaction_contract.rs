//! Contract tests for the ActionsDialog batch selection API. These are
//! source-level audits that confirm the wiring exists without requiring a live
//! GPUI window.

// ============================================================
// ActionsDialog direct mutation methods exist
// ============================================================

#[test]
fn actions_dialog_has_select_action_by_id_method() {
    let source = include_str!("../src/actions/dialog.rs");
    assert!(
        source.contains("pub fn select_action_by_id"),
        "ActionsDialog must have a pub select_action_by_id method for batch selectByValue"
    );
}

#[test]
fn actions_dialog_has_select_action_by_semantic_id_method() {
    let source = include_str!("../src/actions/dialog.rs");
    assert!(
        source.contains("pub fn select_action_by_semantic_id"),
        "ActionsDialog must have a pub select_action_by_semantic_id method for batch selectBySemanticId"
    );
}

// ============================================================
// batch request with actionsDialog target parses correctly
// ============================================================

#[test]
fn batch_with_actions_dialog_target_parses() {
    use script_kit_gpui::protocol::Message;

    let json = serde_json::json!({
        "type": "batch",
        "requestId": "b-actions",
        "target": {"type": "kind", "kind": "actionsDialog"},
        "commands": [
            {"type": "setInput", "text": "edit"},
            {"type": "selectByValue", "value": "edit-script", "submit": false},
            {"type": "selectBySemanticId", "semanticId": "choice:0:edit-script", "submit": true}
        ],
        "options": {"stopOnError": true}
    });

    let msg: Message =
        serde_json::from_value(json).expect("batch with actionsDialog target should parse");
    match msg {
        Message::Batch {
            request_id,
            commands,
            target,
            ..
        } => {
            assert_eq!(request_id, "b-actions");
            assert_eq!(commands.len(), 3);
            assert!(target.is_some(), "target should be present");
        }
        other => panic!("Expected Batch, got: {other:?}"),
    }
}
