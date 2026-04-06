//! Contract tests verifying that Notes is wired into the batch, waitFor,
//! and getElements transaction paths as a first-class automation target.
//! These are source-level audits that confirm the wiring exists without
//! requiring a live GPUI window.

// ============================================================
// getElements: Notes surface collector is registered
// ============================================================

#[test]
fn get_elements_has_notes_collector() {
    let source = include_str!("../src/windows/automation_surface_collector.rs");
    assert!(
        source.contains("AutomationWindowKind::Notes => collect_notes_snapshot"),
        "getElements must route Notes targets to collect_notes_snapshot"
    );
}

#[test]
fn notes_collector_exposes_stable_semantic_ids() {
    let source = include_str!("../src/windows/automation_surface_collector.rs");
    assert!(
        source.contains("\"panel:notes-window\""),
        "Notes collector must expose panel:notes-window semantic ID"
    );
    assert!(
        source.contains("\"input:notes-editor\""),
        "Notes collector must expose input:notes-editor semantic ID"
    );
}

// ============================================================
// Target resolution: Notes is accepted alongside Main and AcpDetached
// ============================================================

#[test]
fn automation_read_target_enum_has_notes_variant() {
    let source = include_str!("../src/prompt_handler/mod.rs");
    assert!(
        source.contains("AutomationReadTarget::Notes {"),
        "AutomationReadTarget enum must have a Notes variant"
    );
}

#[test]
fn resolve_automation_read_target_accepts_notes() {
    let source = include_str!("../src/prompt_handler/mod.rs");
    assert!(
        source.contains("AutomationWindowKind::Notes =>"),
        "resolve_automation_read_target must handle Notes kind"
    );
    assert!(
        source.contains("automation.target.notes_resolved"),
        "Notes resolution must emit a structured log"
    );
}

// ============================================================
// waitFor: Notes targets are no longer rejected
// ============================================================

#[test]
fn wait_for_uses_automation_read_target_for_non_acp_conditions() {
    let source = include_str!("../src/prompt_handler/mod.rs");
    assert!(
        source.contains("resolve_automation_read_target(&rid, \"waitFor\""),
        "waitFor should use resolve_automation_read_target for non-ACP conditions"
    );
}

#[test]
fn wait_for_checks_notes_condition_on_notes_target() {
    let source = include_str!("../src/prompt_handler/mod.rs");
    assert!(
        source.contains("notes_wait_condition_satisfied"),
        "waitFor must delegate to notes_wait_condition_satisfied for Notes targets"
    );
}

// ============================================================
// batch: Notes targets are accepted and routed
// ============================================================

#[test]
fn batch_uses_automation_read_target() {
    let source = include_str!("../src/prompt_handler/mod.rs");
    assert!(
        source.contains("resolve_automation_read_target(&rid, \"batch\""),
        "batch handler should use resolve_automation_read_target for target resolution"
    );
}

#[test]
fn batch_has_notes_path() {
    let source = include_str!("../src/prompt_handler/mod.rs");
    assert!(
        source.contains("notes_batch_target"),
        "batch handler should have a notes_batch_target variable for Notes routing"
    );
    assert!(
        source.contains("automation.batch.notes.completed"),
        "batch handler should emit Notes completion log"
    );
}

#[test]
fn notes_batch_emits_set_input_log() {
    let source = include_str!("../src/prompt_handler/mod.rs");
    assert!(
        source.contains("transaction_notes_set_input"),
        "Notes batch setInput should emit transaction_notes_set_input log"
    );
}

#[test]
fn notes_batch_emits_wait_complete_log() {
    let source = include_str!("../src/prompt_handler/mod.rs");
    assert!(
        source.contains("transaction_notes_wait_complete"),
        "Notes batch waitFor should emit transaction_notes_wait_complete log"
    );
}

// ============================================================
// Notes editor_state accessibility for transaction operations
// ============================================================

#[test]
fn notes_editor_state_is_crate_visible() {
    let source = include_str!("../src/notes/window.rs");
    assert!(
        source.contains("pub(crate) editor_state"),
        "editor_state must be pub(crate) for transaction provider access"
    );
}

#[test]
fn notes_entity_and_handle_helper_exists() {
    let source = include_str!("../src/notes/window/window_ops.rs");
    assert!(
        source.contains("pub fn get_notes_app_entity_and_handle"),
        "get_notes_app_entity_and_handle helper must be exported"
    );
}

// ============================================================
// Notes condition checker covers generic conditions
// ============================================================

#[test]
fn notes_wait_condition_handles_element_exists() {
    let source = include_str!("../src/prompt_handler/mod.rs");
    assert!(
        source.contains("WaitDetailedCondition::ElementExists"),
        "notes_wait_condition_satisfied must handle ElementExists"
    );
    assert!(
        source.contains("WaitDetailedCondition::ElementFocused"),
        "notes_wait_condition_satisfied must handle ElementFocused"
    );
}

// ============================================================
// Protocol parsing: Notes target parses correctly
// ============================================================

#[test]
fn batch_with_notes_target_parses() {
    use script_kit_gpui::protocol::Message;

    let json = serde_json::json!({
        "type": "batch",
        "requestId": "b-notes",
        "target": {"type": "kind", "kind": "notes"},
        "commands": [
            {"type": "setInput", "text": "Hello from automation"}
        ],
        "options": {"stopOnError": true}
    });

    let msg: Message = serde_json::from_value(json).expect("batch with notes target should parse");
    match msg {
        Message::Batch {
            request_id,
            commands,
            target,
            ..
        } => {
            assert_eq!(request_id, "b-notes");
            assert_eq!(commands.len(), 1);
            assert!(target.is_some(), "target should be present");
        }
        other => panic!("Expected Batch, got: {other:?}"),
    }
}

#[test]
fn wait_for_with_notes_target_parses() {
    use script_kit_gpui::protocol::Message;

    let json = serde_json::json!({
        "type": "waitFor",
        "requestId": "w-notes",
        "target": {"type": "kind", "kind": "notes"},
        "condition": {"type": "elementExists", "semanticId": "input:notes-editor"},
        "timeout": 3000,
        "pollInterval": 25
    });

    let msg: Message =
        serde_json::from_value(json).expect("waitFor with notes target should parse");
    match msg {
        Message::WaitFor {
            request_id, target, ..
        } => {
            assert_eq!(request_id, "w-notes");
            assert!(target.is_some(), "target should be present");
        }
        other => panic!("Expected WaitFor, got: {other:?}"),
    }
}

#[test]
fn get_elements_with_notes_target_parses() {
    use script_kit_gpui::protocol::Message;

    let json = serde_json::json!({
        "type": "getElements",
        "requestId": "elm-notes",
        "target": {"type": "kind", "kind": "notes"},
        "limit": 10
    });

    let msg: Message =
        serde_json::from_value(json).expect("getElements with notes target should parse");
    match msg {
        Message::GetElements {
            request_id, target, ..
        } => {
            assert_eq!(request_id, "elm-notes");
            assert!(target.is_some(), "target should be present");
        }
        other => panic!("Expected GetElements, got: {other:?}"),
    }
}

// ============================================================
// matches_state_spec is publicly accessible for non-main providers
// ============================================================

#[test]
fn matches_state_spec_is_exported() {
    let source = include_str!("../src/protocol/transaction_executor.rs");
    assert!(
        source.contains("pub fn matches_state_spec"),
        "matches_state_spec must be pub for use by non-main condition checkers"
    );
}
