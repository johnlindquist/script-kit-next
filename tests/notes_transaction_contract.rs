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

// ============================================================
// Notes skill documents automation-only surface and SDK has no
// public Notes globals
// ============================================================

#[test]
fn notes_skill_documents_automation_only_surface_and_sdk_has_no_public_notes_globals() {
    let skill = include_str!("../kit-init/skills/notes/SKILL.md");
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
// Notes ↔ ACP handoff labels are consistent
// ============================================================

#[test]
fn notes_send_to_acp_label_is_consistent_across_builder_and_panel() {
    let builder = include_str!("../src/actions/builders/notes.rs");
    let panel = include_str!("../src/notes/actions_panel.rs");

    assert!(
        builder.contains("\"Send to ACP Chat\""),
        "Notes builder must use 'Send to ACP Chat' label"
    );
    assert!(
        panel.contains("\"Send to ACP Chat\""),
        "Notes actions panel must use 'Send to ACP Chat' display name"
    );
}

#[test]
fn notes_acp_handoff_emits_structured_logs() {
    let panels = include_str!("../src/notes/window/panels.rs");
    assert!(
        panels.contains("notes_send_to_acp"),
        "Notes handler must emit notes_send_to_acp structured log"
    );
    assert!(
        panels.contains("notes_acp_handoff_blocked"),
        "Notes handler must emit notes_acp_handoff_blocked for empty notes"
    );
    assert!(
        panels.contains("request_explicit_acp_handoff_from_secondary_window"),
        "Notes handoff must route through the canonical explicit-target secondary-window path"
    );
    let acp_mod = include_str!("../src/ai/acp/mod.rs");
    assert!(
        acp_mod.contains("pub(crate) fn open_or_focus_chat_with_input("),
        "ACP staging helper must exist for non-explicit-target secondary-window handoffs"
    );
    assert!(
        acp_mod.contains("chat_window::open_chat_window_with_thread"),
        "ACP helper must open a real ACP chat window instead of the deprecated AI window"
    );
    assert!(
        !panels.contains("crate::ai::open_ai_window(cx)"),
        "Notes handoff must not open the deprecated AI window"
    );
    assert!(
        !panels.contains("crate::ai::set_ai_input(cx, &content, false)"),
        "Notes handoff must not target the deprecated AI window input API"
    );

    let handler = include_str!("../src/app_actions/handle_action/mod.rs");
    assert!(
        handler.contains("acp_save_as_note"),
        "ACP handler must emit acp_save_as_note structured log"
    );
    assert!(
        handler.contains("self.close_acp_chat_to_script_list(false, cx);"),
        "Embedded ACP save-as-note should close ACP back to ScriptList on success"
    );

    let detached = include_str!("../src/ai/acp/chat_window.rs");
    assert!(
        detached.contains("\"acp_save_as_note\""),
        "Detached ACP handler must implement acp_save_as_note"
    );
    assert!(
        detached.contains("close_chat_window(cx);"),
        "Detached ACP save-as-note should close the detached ACP window on success"
    );

    let builder = include_str!("../src/actions/builders/script_context.rs");
    assert!(
        builder.contains("| \"acp_save_as_note\""),
        "Detached ACP host filter should allow acp_save_as_note"
    );
}

#[test]
fn notes_acp_handoff_documented_in_skill_and_guide() {
    let skill = include_str!("../kit-init/skills/notes/SKILL.md");
    let guide = include_str!("../kit-init/GUIDE.md");

    for doc in [skill, guide] {
        assert!(
            doc.contains("## ACP Handoffs"),
            "Notes docs must have ACP Handoffs section"
        );
        assert!(
            doc.contains("**Send to ACP Chat**"),
            "Notes docs must document Send to ACP Chat handoff"
        );
        assert!(
            doc.contains("**Save as Note**"),
            "Notes docs must document Save as Note handoff"
        );
    }
}
