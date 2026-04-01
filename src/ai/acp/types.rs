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
