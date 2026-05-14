//! Source-level contract for Notes ACP actions popup parity and stale-target safety.

const NOTES_ACP_HOST_SOURCE: &str = include_str!("../src/notes/window/acp_host.rs");
const NOTES_WINDOW_SOURCE: &str = include_str!("../src/notes/window.rs");

fn body<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_idx = source.find(start).expect("start marker should exist");
    let rest = &source[start_idx..];
    let end_idx = rest.find(end).unwrap_or(rest.len());
    &rest[..end_idx]
}

// @lat: [[tests/notes-acp#Notes ACP actions originating view#Actions popup refreshes models before snapshot]]
#[test]
fn notes_acp_actions_refresh_models_before_dialog_context_snapshot() {
    let toggle = body(
        NOTES_ACP_HOST_SOURCE,
        "pub(super) fn toggle_acp_actions(",
        "tracing::info!(event = \"notes_acp_actions_opened\")",
    );
    let refresh = toggle.find("thread.refresh_models(cx)").unwrap();
    let snapshot = toggle.find("AcpActionsDialogContext").unwrap();
    assert!(refresh < snapshot);
    assert!(toggle.contains("let actions_target = acp_view.downgrade();"));
    assert!(toggle.contains("let actions_generation = self.notes_acp_generation;"));
    assert!(toggle.contains("parent_automation_id = Some(\"notes\".to_string())"));
}

// @lat: [[tests/notes-acp#Notes ACP actions originating view#Actions dispatch rejects stale Notes ACP generation]]
#[test]
fn dispatch_uses_originating_acp_target_and_generation_not_current_cache() {
    assert!(NOTES_WINDOW_SOURCE.contains("notes_acp_generation: u64"));
    let dispatch = body(
        NOTES_ACP_HOST_SOURCE,
        "fn dispatch_notes_acp_action(",
        "    // Handle model switch.",
    );
    assert!(dispatch.contains("acp_target: gpui::WeakEntity<crate::ai::acp::view::AcpChatView>"));
    assert!(dispatch.contains("let Some(acp_entity) = acp_target.upgrade()"));
    assert!(dispatch.contains("entity.read(cx).notes_acp_generation != acp_generation"));
    assert!(dispatch.contains("notes_acp_action_stale_view"));
    assert!(
        !dispatch.contains("entity.read(cx).embedded_acp_chat.clone()"),
        "dispatch must not use the current embedded_acp_chat cache as the action target"
    );
    assert!(dispatch.contains("app.open_or_focus_embedded_acp(None, window, cx)"));
    assert!(dispatch.contains("chat.restore_draft_snapshot(snapshot, cx);"));
}
