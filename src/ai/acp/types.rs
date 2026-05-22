use crate::ai::window::context_picker::types::{ContextPickerItem, ContextPickerTrigger};

/// Maps a Script Kit UI session to the ACP session ID returned by `session/new`.
#[derive(Debug, Clone)]
pub(crate) struct AcpSessionBinding {
    /// Script Kit's internal chat identifier (from `Chat.id`).
    pub ui_session_id: String,
    /// ACP session ID returned by the agent in `NewSessionResponse`.
    pub agent_session_id: String,
}

/// Convert `ProviderMessage` list into ACP `ContentBlock` list.
///
/// Only emits text blocks in this cycle. Images/audio fail closed — they are
/// logged and skipped rather than being silently passed through.
pub(crate) fn build_prompt_blocks(
    messages: &[crate::ai::providers::ProviderMessage],
) -> Vec<agent_client_protocol::ContentBlock> {
    let mut blocks = Vec::new();
    for msg in messages {
        if msg.role == "system" {
            // ACP agents handle system prompts via session config, not inline.
            // Emit as a labelled text block so the agent at least sees the content.
            blocks.push(agent_client_protocol::ContentBlock::Text(
                agent_client_protocol::TextContent::new(format!("[system]\n{}", msg.content)),
            ));
            continue;
        }
        if msg.has_images() {
            tracing::warn!(
                role = %msg.role,
                image_count = msg.images.len(),
                "acp_prompt_images_skipped: images not supported in this ACP cycle"
            );
        }
        if !msg.content.is_empty() {
            blocks.push(agent_client_protocol::ContentBlock::Text(
                agent_client_protocol::TextContent::new(&msg.content),
            ));
        }
    }
    blocks
}

// ── Extracted ACP view types ────────────────────────────────────────────

/// Active @-mention session state for the ACP inline context picker.
#[derive(Debug, Clone)]
pub(crate) struct AcpMentionSession {
    /// Which trigger character opened this session (`@` or `/`).
    pub(crate) trigger: ContextPickerTrigger,
    /// Character range of the trigger+query in the input text.
    pub(crate) trigger_range: std::ops::Range<usize>,
    /// Query text typed after the trigger.
    pub(crate) query: String,
    /// Currently highlighted row index.
    pub(crate) selected_index: usize,
    /// First visible row in the popup list.
    pub(crate) visible_start: usize,
    /// Ranked picker items for the current query.
    pub(crate) items: Vec<ContextPickerItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AcpDismissedMentionTrigger {
    pub(crate) trigger: ContextPickerTrigger,
    pub(crate) trigger_range: std::ops::Range<usize>,
    pub(crate) query: String,
    pub(crate) cursor: usize,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct AcpMentionPopupParentWindow {
    pub(crate) handle: gpui::AnyWindowHandle,
    pub(crate) bounds: gpui::Bounds<gpui::Pixels>,
    pub(crate) display_id: Option<gpui::DisplayId>,
    pub(crate) display_bounds: Option<gpui::Bounds<gpui::Pixels>>,
}

#[derive(Debug, Clone)]
pub(crate) struct AcpFocusedMentionPreview {
    pub(crate) token: String,
    pub(crate) detail: String,
}

#[derive(Debug, Clone)]
pub(crate) struct AcpPendingPortalSession {
    pub(crate) contract: crate::ai::acp::portal_contract::AcpPortalLaunchContract,
    pub(crate) composer_text: String,
    pub(crate) composer_cursor: usize,
    pub(crate) state: crate::ai::acp::portal_contract::AcpPortalSessionState,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::providers::ProviderMessage;

    #[test]
    fn build_prompt_blocks_text_only() {
        let messages = vec![
            ProviderMessage::user("Hello"),
            ProviderMessage::assistant("Hi there"),
        ];
        let blocks = build_prompt_blocks(&messages);
        assert_eq!(blocks.len(), 2);
        match &blocks[0] {
            agent_client_protocol::ContentBlock::Text(t) => assert_eq!(t.text, "Hello"),
            other => panic!("expected Text, got {:?}", other),
        }
    }

    #[test]
    fn build_prompt_blocks_system_prefix() {
        let messages = vec![ProviderMessage::system("You are helpful")];
        let blocks = build_prompt_blocks(&messages);
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            agent_client_protocol::ContentBlock::Text(t) => {
                assert!(t.text.starts_with("[system]"));
                assert!(t.text.contains("You are helpful"));
            }
            other => panic!("expected Text, got {:?}", other),
        }
    }

    #[test]
    fn build_prompt_blocks_empty_content_skipped() {
        let messages = vec![ProviderMessage::user("")];
        let blocks = build_prompt_blocks(&messages);
        assert!(blocks.is_empty());
    }
}
