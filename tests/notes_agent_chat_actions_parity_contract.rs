//! Source-level contract for Notes Agent Chat actions popup parity and stale-target safety.

const NOTES_AGENT_CHAT_HOST_SOURCE: &str = include_str!("../src/notes/window/agent_chat_host.rs");
const NOTES_WINDOW_SOURCE: &str = include_str!("../src/notes/window.rs");

fn body<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_idx = source.find(start).expect("start marker should exist");
    let rest = &source[start_idx..];
    let end_idx = rest.find(end).unwrap_or(rest.len());
    &rest[..end_idx]
}

#[test]
fn notes_agent_chat_actions_refresh_models_before_dialog_context_snapshot() {
    let toggle = body(
        NOTES_AGENT_CHAT_HOST_SOURCE,
        "pub(super) fn toggle_agent_chat_actions(",
        "tracing::info!(event = \"notes_agent_chat_actions_opened\")",
    );
    let refresh = toggle.find("thread.refresh_models(cx)").unwrap();
    let snapshot = toggle.find("AgentChatActionsDialogContext").unwrap();
    assert!(refresh < snapshot);
    assert!(toggle.contains("let actions_target = agent_chat_view.downgrade();"));
    assert!(toggle.contains("let actions_generation = self.notes_agent_chat_generation;"));
    assert!(toggle.contains("parent_automation_id = Some(\"notes\".to_string())"));
}

#[test]
fn dispatch_uses_originating_agent_chat_target_and_generation_not_current_cache() {
    assert!(NOTES_WINDOW_SOURCE.contains("notes_agent_chat_generation: u64"));
    let dispatch = body(
        NOTES_AGENT_CHAT_HOST_SOURCE,
        "fn dispatch_notes_agent_chat_action(",
        "    // Handle model switch.",
    );
    assert!(dispatch
        .contains("agent_chat_target: gpui::WeakEntity<crate::ai::agent_chat::ui::AgentChatView>"));
    assert!(dispatch.contains("let Some(agent_chat_entity) = agent_chat_target.upgrade()"));
    assert!(
        dispatch.contains("entity.read(cx).notes_agent_chat_generation != agent_chat_generation")
    );
    assert!(dispatch.contains("notes_agent_chat_action_stale_view"));
    assert!(
        !dispatch.contains("entity.read(cx).embedded_agent_chat.clone()"),
        "dispatch must not use the current embedded_agent_chat cache as the action target"
    );
    assert!(dispatch.contains("app.close_embedded_agent_chat_via_host("));
}
