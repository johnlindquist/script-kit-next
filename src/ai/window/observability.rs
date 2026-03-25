use super::types::{AiUiEventKind, AiWindowMode};
use super::ChatId;

/// Payload for a single canonical AI UI event marker.
pub(super) struct AiUiEvent<'a> {
    pub(super) kind: AiUiEventKind,
    pub(super) action: &'a str,
    pub(super) source: &'a str,
    pub(super) window_mode: AiWindowMode,
    pub(super) selected_chat_id: Option<&'a ChatId>,
    pub(super) overlay_visible: bool,
    pub(super) search_active: bool,
}

/// Emit a single canonical machine-readable AI UI event marker.
///
/// All significant state transitions in the AI window should funnel through
/// this helper so agents and log parsers can filter on one `"ai_ui_event"`
/// target with a deterministic JSON payload.
pub(super) fn emit_ai_ui_event(event: &AiUiEvent<'_>, extra: Option<serde_json::Value>) {
    let payload = serde_json::json!({
        "kind": format!("{:?}", event.kind),
        "action": event.action,
        "source": event.source,
        "window_mode": format!("{:?}", event.window_mode),
        "selected_chat_id": event.selected_chat_id.map(|id| id.as_str()),
        "overlay_visible": event.overlay_visible,
        "search_active": event.search_active,
        "extra": extra,
    });

    tracing::info!(
        target: "ai",
        payload = %payload,
        "ai_ui_event"
    );
}
