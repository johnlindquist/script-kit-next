//! Source-contract tests for Notes → ACP Chat handoff.
//!
//! These lock the invariant that Notes uses the canonical explicit ACP
//! target handoff path (not raw string injection), and that Cmd+Enter
//! is claimed in the Notes keyboard handler.

use std::fs;

#[test]
fn notes_send_to_ai_routes_through_explicit_acp_handoff() {
    let source = fs::read_to_string("src/notes/window/panels.rs")
        .expect("Failed to read src/notes/window/panels.rs");

    assert!(
        source.contains("handoff_selected_note_to_acp(\"NotesAction::SendToAi\""),
        "NotesAction::SendToAi must route through the shared explicit ACP handoff"
    );

    assert!(
        !source.contains("set_ai_input(cx, &content, false)"),
        "NotesAction::SendToAi must stop injecting raw note text into the legacy AI window"
    );
}

#[test]
fn notes_selected_note_target_is_canonical_note_kind() {
    let source = fs::read_to_string("src/notes/window/panels.rs")
        .expect("Failed to read src/notes/window/panels.rs");

    assert!(
        source.contains("kind: \"note\".to_string()"),
        "Selected notes must emit a canonical note target kind"
    );

    assert!(
        source.contains("\"noteId\"") && source.contains("\"content\""),
        "Selected note target must include note identity and content metadata"
    );
}

#[test]
fn notes_keyboard_claims_cmd_enter() {
    let source = fs::read_to_string("src/notes/window/keyboard.rs")
        .expect("Failed to read src/notes/window/keyboard.rs");

    assert!(
        source.contains("NotesWindowCmdEnter"),
        "Notes keyboard must route Cmd+Enter through the ACP handoff helper"
    );

    assert!(
        source.contains("modifiers.platform")
            && source.contains("is_key_enter(key)")
            && source.contains("cx.stop_propagation();"),
        "Notes keyboard must claim Cmd+Enter before general platform shortcuts"
    );
}

#[test]
fn chip_prefix_map_includes_action_and_note() {
    let source = fs::read_to_string("src/app_impl/tab_ai_mode.rs")
        .expect("Failed to read src/app_impl/tab_ai_mode.rs");

    assert!(
        source.contains("\"action\" => \"Action\""),
        "Chip prefix map must include 'action' => 'Action'"
    );
    assert!(
        source.contains("\"note\" => \"Note\""),
        "Chip prefix map must include 'note' => 'Note'"
    );
}
