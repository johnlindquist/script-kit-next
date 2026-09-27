//! Contract tests verifying that detached Agent Chat is wired into the batch and
//! waitFor transaction paths. These are source-level audits that confirm the
//! wiring exists without requiring a live GPUI window.

// ============================================================
// batch target resolution accepts agentChatDetached
// ============================================================

// ============================================================
// Detached Agent Chat batch commands emit structured transaction logs
// ============================================================

// ============================================================
// waitFor already supports detached Agent Chat (pre-existing)
// ============================================================

// ============================================================
// Surface collector exposes reusable detached Agent Chat elements
// ============================================================

// ============================================================
// Transaction provider exists with correct trait implementation
// ============================================================

#[test]
fn transaction_provider_implements_required_methods() {
    let source = include_str!("../src/windows/automation_transaction_provider.rs");
    assert!(
        source.contains("fn snapshot(&self) -> UiStateSnapshot"),
        "provider must implement snapshot()"
    );
    assert!(
        source.contains("fn set_input(&mut self, text: &str)"),
        "provider must implement set_input()"
    );
    assert!(
        source.contains("fn select_by_value(&mut self, value: &str"),
        "provider must implement select_by_value()"
    );
    assert!(
        source.contains("fn select_by_semantic_id(&mut self, semantic_id: &str"),
        "provider must implement select_by_semantic_id()"
    );
    assert!(
        source.contains("fn agent_chat_test_probe(&self, tail: usize)"),
        "provider must implement agent_chat_test_probe()"
    );
}

// ============================================================
// ActionsDialog batch target resolution
// ============================================================

// ============================================================
// ActionsDialog target resolution in resolve_automation_read_target
// ============================================================

// ============================================================
// ActionsDialog transaction provider exists
// ============================================================

#[test]
fn actions_dialog_transaction_provider_implements_required_methods() {
    let source = include_str!("../src/windows/automation_transaction_provider.rs");
    assert!(
        source.contains("ActionsDialogTransactionProvider"),
        "transaction provider file must contain ActionsDialogTransactionProvider"
    );
    assert!(
        source.contains("transaction_actions_dialog_set_input"),
        "ActionsDialog provider must emit set_input log"
    );
    assert!(
        source.contains("transaction_actions_dialog_select_by_value"),
        "ActionsDialog provider must emit select_by_value log"
    );
    assert!(
        source.contains("transaction_actions_dialog_select_by_semantic_id"),
        "ActionsDialog provider must emit select_by_semantic_id log"
    );
}

// ============================================================
// ActionsDialog surface collector exposes reusable collector
// ============================================================

#[test]
fn surface_collector_has_reusable_actions_dialog_collector() {
    let source = include_str!("../src/windows/automation_surface_collector.rs");
    assert!(
        source.contains("pub(crate) fn collect_actions_dialog_elements"),
        "surface collector should expose a reusable collect_actions_dialog_elements function"
    );
}

#[test]
fn actions_dialog_elements_include_search_and_actions_list() {
    let source = include_str!("../src/windows/automation_surface_collector.rs");
    assert!(
        source.contains("\"input:actions-search\""),
        "ActionsDialog collector should expose input:actions-search semantic ID"
    );
    assert!(
        source.contains("\"list:actions\""),
        "ActionsDialog collector should expose list:actions semantic ID"
    );
}

// ============================================================
// ActionsDialog direct mutation methods exist
// ============================================================

#[test]
fn actions_dialog_has_set_search_text_method() {
    let source = include_str!("../src/actions/dialog.rs");
    assert!(
        source.contains("pub fn set_search_text"),
        "ActionsDialog must have a pub set_search_text method for direct mutation"
    );
}

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

// ============================================================
// AgentChatView visibility for cross-module access
// ============================================================

// ============================================================
// Batch request with agentChatDetached target parses correctly
// ============================================================
