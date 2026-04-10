//! Source-contract tests for Notes → ACP Chat handoff.
//!
//! These lock the invariant that Notes opens its embedded ACP surface and
//! stages a canonical `FocusedTarget` chip (not raw string injection or
//! secondary-window handoff), and that Cmd+Enter is claimed in the Notes
//! keyboard handler.

use std::fs;

#[test]
fn notes_send_to_ai_routes_through_embedded_acp_handoff() {
    let source = fs::read_to_string("src/notes/window/panels.rs")
        .expect("Failed to read src/notes/window/panels.rs");

    assert!(
        source.contains("open_selected_note_in_embedded_acp(\"NotesAction::SendToAi\""),
        "NotesAction::SendToAi must route through the embedded ACP handoff"
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

#[test]
fn notes_embedded_acp_stages_focused_target_chip() {
    let source = fs::read_to_string("src/notes/window/panels.rs")
        .expect("Failed to read src/notes/window/panels.rs");

    assert!(
        source.contains("notes_embedded_acp_target_staged"),
        "Notes embedded ACP handoff must log notes_embedded_acp_target_staged"
    );
    assert!(
        source.contains("AiContextPart::FocusedTarget"),
        "Notes embedded ACP handoff must stage a FocusedTarget chip"
    );
    assert!(
        source.contains("format_explicit_target_chip_label"),
        "Notes embedded ACP handoff must use the shared canonical target-chip label helper"
    );
}

#[test]
fn shared_chip_label_formatter_exists_in_ai_module() {
    let source = fs::read_to_string("src/ai/mod.rs").expect("Failed to read src/ai/mod.rs");

    assert!(
        source.contains("pub(crate) fn format_explicit_target_chip_label"),
        "Shared chip-label formatter must be defined in the ai module"
    );

    assert!(
        source.contains("\"note\" => \"Note\""),
        "Shared chip-label formatter must map note kind to Note prefix"
    );
}

#[test]
fn tab_ai_mode_delegates_to_shared_chip_label_formatter() {
    let source = fs::read_to_string("src/app_impl/tab_ai_mode.rs")
        .expect("Failed to read src/app_impl/tab_ai_mode.rs");

    assert!(
        source.contains("crate::ai::format_explicit_target_chip_label(target)"),
        "tab_ai_mode chip label formatter must delegate to the shared formatter"
    );
}
