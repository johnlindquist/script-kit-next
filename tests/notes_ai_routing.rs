//! Source-contract tests for Notes → Agent Chat Chat handoff.
//!
//! These lock the invariant that Notes opens its embedded Agent Chat surface and
//! stages the selected note cart as inline `@mentions` (not raw string
//! injection or secondary-window handoff), and that Cmd+Enter is claimed in
//! the Notes keyboard handler.

use std::fs;

#[test]
fn notes_send_to_ai_routes_through_embedded_agent_chat_handoff() {
    let source = fs::read_to_string("src/notes/window/panels.rs")
        .expect("Failed to read src/notes/window/panels.rs");

    assert!(
        source.contains("open_selected_note_cart_in_embedded_agent_chat(\"NotesAction::SendToAi\""),
        "NotesAction::SendToAi must route through the note-cart Agent Chat handoff"
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
        "Notes keyboard must route Cmd+Enter through the Agent Chat handoff helper"
    );

    assert!(
        source.contains("modifiers.platform")
            && source.contains("is_key_enter(key)")
            && source.contains("cx.stop_propagation();"),
        "Notes keyboard must claim Cmd+Enter before general platform shortcuts"
    );

    assert!(
        source.contains(
            "self.open_selected_note_cart_in_embedded_agent_chat(\"NotesWindowCmdEnter\""
        ),
        "Notes Cmd+Enter must route through the note-cart Agent Chat handoff"
    );
}

#[test]
fn chip_prefix_map_includes_action_and_note() {
    let source = fs::read_to_string("src/app_impl/tab_ai_mode/mod.rs")
        .expect("Failed to read src/app_impl/tab_ai_mode/mod.rs");

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
fn notes_embedded_agent_chat_stages_note_cart_as_mentions() {
    let source = fs::read_to_string("src/notes/window/panels.rs")
        .expect("Failed to read src/notes/window/panels.rs");
    let host_source = fs::read_to_string("src/notes/window/agent_chat_host.rs")
        .expect("Failed to read src/notes/window/agent_chat_host.rs");
    let agent_chat_view_source = fs::read_to_string("src/ai/agent_chat/ui/view.rs")
        .expect("Failed to read src/ai/agent_chat/ui/view.rs");

    assert!(
        source.contains("notes_cart_open_embedded_agent_chat_requested"),
        "Notes cart handoff must log notes_cart_open_embedded_agent_chat_requested"
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
        source.contains("self.ensure_embedded_agent_chat_view(None, cx)"),
        "Notes cart handoff must reuse the shared embedded Agent Chat bootstrap helper"
    );
    assert!(
        source.contains("chat.stage_inline_context_parts_from_host(parts, source, cx)"),
        "Notes cart handoff must delegate staged mention rendering to the shared Agent Chat host helper"
    );
    assert!(
        host_source.contains("fn wire_agent_chat_host_callbacks(")
            && host_source.contains("fn ensure_embedded_agent_chat_view(")
            && host_source.contains("self.wire_agent_chat_host_callbacks(&view, cx);"),
        "Notes first-open cart handoff must go through the shared host wiring path"
    );
    assert!(
        agent_chat_view_source.contains("event = \"agent_chat_host_inline_context_staged\"")
            && agent_chat_view_source
                .contains("thread.replace_pending_context_parts(staged_parts, source, cx);")
            && agent_chat_view_source.contains("format_typed_label_mention_token("),
        "Shared Agent Chat host staging must replace prior parts and build deterministic inline tokens"
    );
    assert!(
        host_source.contains("event = \"notes_embedded_agent_chat_view_ensured\""),
        "Shared Notes Agent Chat bootstrap helper must emit the ensure log for first-open and reuse"
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
    let source = fs::read_to_string("src/app_impl/tab_ai_mode/mod.rs")
        .expect("Failed to read src/app_impl/tab_ai_mode/mod.rs");

    assert!(
        source.contains("crate::ai::format_explicit_target_chip_label(target)"),
        "tab_ai_mode chip label formatter must delegate to the shared formatter"
    );
}

#[test]
fn notes_cart_first_open_uses_shared_notes_host_contract() {
    let host_source = fs::read_to_string("src/notes/window/agent_chat_host.rs")
        .expect("Failed to read src/notes/window/agent_chat_host.rs");

    assert!(
        host_source.contains("chat.set_allowed_portal_kinds(vec![")
            && host_source.contains("PortalKind::AgentChatHistory"),
        "Notes host contract must keep Agent Chat history as the only locally hosted portal kind"
    );
    assert!(
        host_source.contains("chat.set_on_open_portal(move |kind, cx|")
            && host_source
                .contains("Self::handle_agent_chat_portal_static(chat_entity, kind, cx);"),
        "Notes host wiring must continue to route portal opens through the shared static handler"
    );
}

#[test]
fn notes_cart_reopen_replaces_previous_pending_parts() {
    let agent_chat_view_source = fs::read_to_string("src/ai/agent_chat/ui/view.rs")
        .expect("Failed to read src/ai/agent_chat/ui/view.rs");
    let thread_source = fs::read_to_string("src/ai/agent_chat/ui/thread.rs")
        .expect("Failed to read src/ai/agent_chat/ui/thread.rs");

    assert!(
        agent_chat_view_source.contains("self.typed_mention_aliases.clear();")
            && agent_chat_view_source.contains("self.inline_owned_context_tokens.clear();")
            && agent_chat_view_source
                .contains("thread.replace_pending_context_parts(staged_parts, source, cx);"),
        "Agent Chat host staging must clear stale composer aliases/tokens before replacing staged parts"
    );
    assert!(
        thread_source.contains("event = \"agent_chat_pending_context_parts_replaced\""),
        "Thread replacement path must emit an explicit replacement log for runtime verification"
    );
}

#[test]
fn notes_target_staging_uses_shared_host_replacement_path() {
    let source = fs::read_to_string("src/notes/window/panels.rs")
        .expect("Failed to read src/notes/window/panels.rs");

    let fn_start = source
        .find("fn stage_note_target_in_embedded_agent_chat(")
        .expect("stage_note_target_in_embedded_agent_chat must exist");
    let fn_body = &source[fn_start..];
    let next_fn = fn_body[1..]
        .find("\n    pub(crate) fn ")
        .unwrap_or(fn_body.len());
    let fn_body = &fn_body[..next_fn];

    assert!(
        fn_body.contains("chat.stage_inline_context_parts_from_host("),
        "stage_note_target_in_embedded_agent_chat must use the shared host replacement helper"
    );
    assert!(
        !fn_body.contains("thread.add_context_part("),
        "stage_note_target_in_embedded_agent_chat must stop appending direct thread context parts"
    );
    assert!(
        source.contains("event = \"notes_embedded_agent_chat_target_staged_via_shared_host_path\""),
        "notes target staging should emit a migration breadcrumb log for runtime verification"
    );
}
