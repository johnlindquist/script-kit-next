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
// Target resolution: Notes is accepted alongside Main and AgentChatDetached
// ============================================================

// ============================================================
// waitFor: Notes targets are no longer rejected
// ============================================================

// ============================================================
// batch: Notes targets are accepted and routed
// ============================================================

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

// ============================================================
// Notes skill documents automation-only surface and SDK has no
// public Notes globals
// ============================================================

#[test]
fn notes_skill_documents_automation_only_surface_and_sdk_has_no_public_notes_globals() {
    let skill = include_str!("../kit-init/skills/manage-notes/SKILL.md");
    assert!(
        skill.contains(
            "The current public Notes script surface is the automation target (`kind: notes`)"
        ),
        "Notes skill must document the automation-only surface"
    );

    let sdk = include_str!("../scripts/kit-sdk.ts");
    for forbidden in [
        "globalThis.notesOpen",
        "globalThis.notesCreate",
        "globalThis.notesSearch",
    ] {
        assert!(
            !sdk.contains(forbidden),
            "scripts/kit-sdk.ts must not expose invented Notes global: {forbidden}"
        );
    }
}

// ============================================================
// Notes ↔ Agent Chat handoff labels are consistent
// ============================================================

#[test]
fn notes_agent_chat_handoff_documented_in_skill_and_guide() {
    let skill = include_str!("../kit-init/skills/manage-notes/SKILL.md");
    let guide = include_str!("../kit-init/GUIDE.md");

    for doc in [skill, guide] {
        assert!(
            doc.contains("## Agent Chat Handoffs"),
            "Notes docs must have Agent Chat Handoffs section"
        );
        assert!(
            doc.contains("**Send to Agent Chat**"),
            "Notes docs must document Send to Agent Chat handoff"
        );
        assert!(
            doc.contains("**Save as Note**"),
            "Notes docs must document Save as Note handoff"
        );
    }
}
