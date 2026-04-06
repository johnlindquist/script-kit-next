//! Contract tests verifying that detached ACP is wired into the batch and
//! waitFor transaction paths. These are source-level audits that confirm the
//! wiring exists without requiring a live GPUI window.

// ============================================================
// batch target resolution accepts acpDetached
// ============================================================

#[test]
fn batch_handler_accepts_acp_detached_target() {
    let source = include_str!("../src/prompt_handler/mod.rs");
    // The batch handler must resolve targets through resolve_automation_read_target
    // (which accepts AcpDetached and Notes) instead of resolve_main_only_target.
    assert!(
        source.contains("resolve_automation_read_target(&rid, \"batch\""),
        "batch handler should use resolve_automation_read_target for target resolution"
    );
}

#[test]
fn batch_handler_has_detached_acp_branch() {
    let source = include_str!("../src/prompt_handler/mod.rs");
    assert!(
        source.contains("detached_batch_entity"),
        "batch handler should have a detached_batch_entity variable for ACP routing"
    );
    assert!(
        source.contains("automation.batch.detached_acp.completed"),
        "batch handler should emit detached ACP completion log"
    );
}

// ============================================================
// Detached ACP batch commands emit structured transaction logs
// ============================================================

#[test]
fn detached_acp_batch_emits_set_input_log() {
    let source = include_str!("../src/prompt_handler/mod.rs");
    assert!(
        source.contains("transaction_detached_acp_set_input"),
        "detached ACP batch setInput should emit transaction_detached_acp_set_input log"
    );
}

#[test]
fn detached_acp_batch_emits_select_by_value_log() {
    let source = include_str!("../src/prompt_handler/mod.rs");
    assert!(
        source.contains("transaction_detached_acp_select_by_value"),
        "detached ACP batch selectByValue should emit transaction_detached_acp_select_by_value log"
    );
}

#[test]
fn detached_acp_batch_emits_wait_complete_log() {
    let source = include_str!("../src/prompt_handler/mod.rs");
    assert!(
        source.contains("transaction_wait_complete"),
        "detached ACP batch waitFor should emit transaction_wait_complete log"
    );
}

// ============================================================
// waitFor already supports detached ACP (pre-existing)
// ============================================================

#[test]
fn wait_for_uses_acp_read_target_for_acp_conditions() {
    let source = include_str!("../src/prompt_handler/mod.rs");
    assert!(
        source.contains("resolve_acp_read_target(&rid, \"waitFor\""),
        "waitFor handler should use resolve_acp_read_target for ACP conditions"
    );
}

#[test]
fn wait_for_passes_detached_entity_to_condition_checker() {
    let source = include_str!("../src/prompt_handler/mod.rs");
    assert!(
        source.contains("wait_condition_satisfied_for_target"),
        "waitFor should delegate to wait_condition_satisfied_for_target with detached entity"
    );
}

// ============================================================
// Surface collector exposes reusable detached ACP elements
// ============================================================

#[test]
fn surface_collector_has_reusable_detached_acp_collector() {
    let source = include_str!("../src/windows/automation_surface_collector.rs");
    assert!(
        source.contains("pub(crate) fn collect_acp_detached_elements"),
        "surface collector should expose a reusable collect_acp_detached_elements function"
    );
}

#[test]
fn detached_acp_elements_include_composer_and_messages() {
    let source = include_str!("../src/windows/automation_surface_collector.rs");
    assert!(
        source.contains("\"input:acp-composer\""),
        "detached ACP collector should expose input:acp-composer semantic ID"
    );
    assert!(
        source.contains("\"list:acp-messages\""),
        "detached ACP collector should expose list:acp-messages semantic ID"
    );
}

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
        source.contains("fn acp_test_probe(&self, tail: usize)"),
        "provider must implement acp_test_probe()"
    );
}

// ============================================================
// ActionsDialog batch target resolution
// ============================================================

#[test]
fn batch_handler_accepts_actions_dialog_target() {
    let source = include_str!("../src/prompt_handler/mod.rs");
    assert!(
        source.contains("actions_dialog_batch_entity"),
        "batch handler should have an actions_dialog_batch_entity variable for ActionsDialog routing"
    );
    assert!(
        source.contains("automation.batch.actions_dialog.completed"),
        "batch handler should emit ActionsDialog completion log"
    );
}

#[test]
fn actions_dialog_batch_emits_set_input_log() {
    let source = include_str!("../src/prompt_handler/mod.rs");
    assert!(
        source.contains("transaction_actions_dialog_set_input"),
        "ActionsDialog batch setInput should emit transaction_actions_dialog_set_input log"
    );
}

#[test]
fn actions_dialog_batch_emits_select_by_value_log() {
    let source = include_str!("../src/prompt_handler/mod.rs");
    assert!(
        source.contains("transaction_actions_dialog_select_by_value"),
        "ActionsDialog batch selectByValue should emit transaction_actions_dialog_select_by_value log"
    );
}

#[test]
fn actions_dialog_batch_emits_select_by_semantic_id_log() {
    let source = include_str!("../src/prompt_handler/mod.rs");
    assert!(
        source.contains("transaction_actions_dialog_select_by_semantic_id"),
        "ActionsDialog batch selectBySemanticId should emit transaction_actions_dialog_select_by_semantic_id log"
    );
}

#[test]
fn actions_dialog_batch_unsupported_commands_fail_closed() {
    let source = include_str!("../src/prompt_handler/mod.rs");
    assert!(
        source.contains("transaction_actions_dialog_unsupported"),
        "ActionsDialog batch should emit unsupported command log for rejected commands"
    );
}

// ============================================================
// ActionsDialog target resolution in resolve_automation_read_target
// ============================================================

#[test]
fn resolve_automation_read_target_accepts_actions_dialog() {
    let source = include_str!("../src/prompt_handler/mod.rs");
    assert!(
        source.contains("AutomationReadTarget::ActionsDialog"),
        "AutomationReadTarget must have an ActionsDialog variant"
    );
    assert!(
        source.contains("automation.target.actions_dialog_resolved"),
        "resolve_automation_read_target must emit ActionsDialog resolution log"
    );
}

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
// AcpChatView visibility for cross-module access
// ============================================================

#[test]
fn acp_mention_session_is_crate_visible() {
    let source = include_str!("../src/ai/acp/view.rs");
    assert!(
        source.contains("pub(crate) mention_session"),
        "mention_session must be pub(crate) for transaction provider access"
    );
}

#[test]
fn acp_select_mention_index_is_crate_visible() {
    let source = include_str!("../src/ai/acp/view.rs");
    assert!(
        source.contains("pub(crate) fn select_mention_index"),
        "select_mention_index must be pub(crate) for batch handler access"
    );
}

#[test]
fn acp_accept_mention_selection_is_crate_visible() {
    let source = include_str!("../src/ai/acp/view.rs");
    assert!(
        source.contains("pub(crate) fn accept_mention_selection"),
        "accept_mention_selection must be pub(crate) for batch handler access"
    );
}

// ============================================================
// Batch request with acpDetached target parses correctly
// ============================================================

#[test]
fn batch_with_acp_detached_target_parses() {
    use script_kit_gpui::protocol::Message;

    let json = serde_json::json!({
        "type": "batch",
        "requestId": "b-acp",
        "target": {"type": "kind", "kind": "acpDetached"},
        "commands": [
            {"type": "setInput", "text": "@clip"},
            {"type": "waitFor", "condition": {"type": "acpPickerOpen"}, "timeout": 3000, "pollInterval": 25},
            {"type": "selectByValue", "value": "Clipboard", "submit": true}
        ],
        "options": {"stopOnError": true}
    });

    let msg: Message =
        serde_json::from_value(json).expect("batch with acpDetached target should parse");
    match msg {
        Message::Batch {
            request_id,
            commands,
            target,
            ..
        } => {
            assert_eq!(request_id, "b-acp");
            assert_eq!(commands.len(), 3);
            assert!(target.is_some(), "target should be present");
        }
        other => panic!("Expected Batch, got: {other:?}"),
    }
}

#[test]
fn wait_for_with_acp_detached_target_parses() {
    use script_kit_gpui::protocol::Message;

    let json = serde_json::json!({
        "type": "waitFor",
        "requestId": "w-picker",
        "target": {"type": "kind", "kind": "acpDetached"},
        "condition": {"type": "acpPickerOpen"},
        "timeout": 3000,
        "pollInterval": 25
    });

    let msg: Message =
        serde_json::from_value(json).expect("waitFor with acpDetached target should parse");
    match msg {
        Message::WaitFor {
            request_id, target, ..
        } => {
            assert_eq!(request_id, "w-picker");
            assert!(target.is_some(), "target should be present");
        }
        other => panic!("Expected WaitFor, got: {other:?}"),
    }
}
