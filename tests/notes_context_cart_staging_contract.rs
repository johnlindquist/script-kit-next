//! Source-level contract for Notes context-cart staging into embedded Agent Chat.

const PANELS_SOURCE: &str = include_str!("../src/notes/window/panels.rs");
const NOTES_SOURCE: &str = include_str!("../src/notes/window/notes.rs");
const AGENT_CHAT_HOST_SOURCE: &str = include_str!("../src/notes/window/agent_chat_host.rs");
const AGENT_CHAT_VIEW_SOURCE: &str = include_str!("../src/ai/agent_chat/ui/view.rs");

fn body<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_idx = source.find(start).expect("start marker should exist");
    let rest = &source[start_idx..];
    let end_idx = rest.find(end).unwrap_or(rest.len());
    &rest[..end_idx]
}

#[test]
fn note_cart_handoff_uses_deduped_replacement_staging_and_consumes_items() {
    let handoff = body(
        PANELS_SOURCE,
        "pub(super) fn open_selected_note_cart_in_embedded_agent_chat(",
        "event = \"notes_cart_open_embedded_agent_chat_completed\"",
    );
    assert!(PANELS_SOURCE.contains("list_note_cart_items_deduped(note_id)"));
    assert!(
        handoff.contains("crate::notes::storage::list_note_cart_items(note_id)"),
        "handoff should collect all note-scoped cart ids for deletion so duplicate persisted rows are consumed"
    );
    assert!(handoff.contains("chat.stage_inline_context_parts_from_host(parts, source, cx)"));
    assert!(handoff.contains("delete_note_cart_items(note_id, &cart_item_ids)"));
}

#[test]
fn note_switch_clears_notes_hosted_agent_chat_context_for_previous_note() {
    let select_note = body(
        NOTES_SOURCE,
        "fn select_note_internal(",
        "/// Select a note for editing",
    );
    let prev = select_note
        .find("let prev_note_id = self.selected_note_id;")
        .unwrap();
    let assign = select_note
        .find("self.selected_note_id = Some(id);")
        .unwrap();
    let clear = select_note
        .find("self.clear_notes_hosted_agent_chat_context_for_note(prev_note_id, cx);")
        .unwrap();
    assert!(prev < assign && assign < clear);
    assert!(AGENT_CHAT_HOST_SOURCE
        .contains("pub(super) fn clear_notes_hosted_agent_chat_context_for_note"));
    assert!(AGENT_CHAT_VIEW_SOURCE.contains("pub(crate) fn clear_hosted_context_parts_from_host"));
}
