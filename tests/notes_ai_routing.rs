//! Source-contract tests for Notes → ACP Chat handoff.
//!
//! These lock the invariant that Notes opens its embedded ACP surface and
//! stages the selected note cart as inline `@mentions` (not raw string
//! injection or secondary-window handoff), and that Cmd+Enter is claimed in
//! the Notes keyboard handler.

use std::fs;

#[test]
fn notes_send_to_ai_routes_through_embedded_acp_handoff() {
    let source = fs::read_to_string("src/notes/window/panels.rs")
        .expect("Failed to read src/notes/window/panels.rs");

    assert!(
        source.contains("open_selected_note_cart_in_embedded_acp(\"NotesAction::SendToAi\""),
        "NotesAction::SendToAi must route through the note-cart ACP handoff"
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

    assert!(
        source.contains("self.open_selected_note_cart_in_embedded_acp(\"NotesWindowCmdEnter\""),
        "Notes Cmd+Enter must route through the note-cart ACP handoff"
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
fn notes_embedded_acp_stages_note_cart_as_mentions() {
    let source = fs::read_to_string("src/notes/window/panels.rs")
        .expect("Failed to read src/notes/window/panels.rs");

    assert!(
        source.contains("notes_cart_open_embedded_acp_requested"),
        "Notes cart handoff must log notes_cart_open_embedded_acp_requested"
    );
    assert!(
        source.contains("AiContextPart::TextBlock"),
        "Notes cart handoff must stage the note body as a TextBlock part"
    );
    assert!(
        source.contains("\"Selected Text\".to_string()"),
        "Notes cart handoff must stage selections with a canonical Selected Text label"
    );
    assert!(
        source.contains("notes://{}#selection={}-{}"),
        "Notes cart handoff must preserve selection provenance in the text-block source"
    );
    assert!(
        source.contains("part_to_inline_token(&part)"),
        "Notes cart handoff must render staged parts as inline @mention tokens"
    );
    assert!(
        source.contains("chat.register_typed_alias(inline_token.clone(), part)"),
        "Notes cart handoff must register typed aliases for staged mention tokens"
    );
    assert!(
        source.contains("chat.register_inline_owned_token(inline_token);"),
        "Notes cart handoff must claim inline ownership for staged mention tokens"
    );
}

#[test]
fn selected_note_text_blocks_render_as_selected_inline_mentions() {
    let source = fs::read_to_string("src/ai/context_mentions/mod.rs")
        .expect("Failed to read src/ai/context_mentions/mod.rs");

    assert!(
        source.contains("source.contains(\"#selection=\")"),
        "Selection-backed text blocks must render as the @selected inline token"
    );
    assert!(
        source.contains("Some(\"@selected\".to_string())"),
        "Selection-backed text blocks must map to the @selected inline token"
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
