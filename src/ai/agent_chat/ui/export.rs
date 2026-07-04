//! Shared Agent Chat conversation markdown serializer.
//!
//! Used by both the shared action handler (`agent_chat_export_markdown`,
//! `agent_chat_save_as_note`) and the detached Agent Chat Chat window export path.

use super::conversation_export::{AgentChatConversationExport, AgentChatExportPurpose};
use super::thread::{AgentChatThread, AgentChatThreadMessage, AgentChatThreadMessageRole};

const AGENT_CHAT_CONVERSATION_HEADING: &str = "# Agent Chat Conversation\n\n";

fn role_label(role: &AgentChatThreadMessageRole) -> &'static str {
    match role {
        AgentChatThreadMessageRole::User => "**You**",
        AgentChatThreadMessageRole::Assistant => "**Assistant**",
        AgentChatThreadMessageRole::Thought => "**Thinking**",
        AgentChatThreadMessageRole::Tool => "**Tool**",
        AgentChatThreadMessageRole::System => "**System**",
        AgentChatThreadMessageRole::Error => "**Error**",
    }
}

/// Build a markdown document from Agent Chat thread messages. Returns `None` if no
/// messages have non-empty renderable body text.
pub(crate) fn build_agent_chat_conversation_markdown(
    messages: &[AgentChatThreadMessage],
) -> Option<String> {
    let mut md = String::from(AGENT_CHAT_CONVERSATION_HEADING);
    let mut wrote_any = false;
    for msg in messages {
        let body = msg.body.trim();
        if body.is_empty() {
            continue;
        }
        md.push_str(role_label(&msg.role));
        md.push_str("\n\n");
        md.push_str(body);
        md.push_str("\n\n---\n\n");
        wrote_any = true;
    }
    wrote_any.then_some(md)
}

pub(crate) fn build_agent_chat_conversation_markdown_from_thread(
    thread: &AgentChatThread,
) -> Option<String> {
    let export = thread.export_conversation(AgentChatExportPurpose::CopyTranscript);
    build_agent_chat_conversation_markdown_from_export(&export)
}

pub(crate) fn build_agent_chat_conversation_markdown_from_export(
    export: &AgentChatConversationExport,
) -> Option<String> {
    let mut md = String::from(AGENT_CHAT_CONVERSATION_HEADING);
    let mut wrote_any = false;
    for msg in &export.messages {
        let body = msg.body.trim();
        if body.is_empty() {
            continue;
        }
        md.push_str(match msg.role.as_str() {
            "user" => "**You**",
            "assistant" => "**Assistant**",
            "thought" => "**Thinking**",
            "tool" => "**Tool**",
            "system" => "**System**",
            "error" => "**Error**",
            _ => "**Message**",
        });
        md.push_str("\n\n");
        md.push_str(body);
        md.push_str("\n\n---\n\n");
        wrote_any = true;
    }
    wrote_any.then_some(md)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::SharedString;

    fn message(id: u64, role: AgentChatThreadMessageRole, body: &str) -> AgentChatThreadMessage {
        AgentChatThreadMessage {
            id,
            role,
            body: SharedString::from(body.to_string()),
            tool_call_id: None,
            tool_meta: None,
            attachments: Vec::new(),
        }
    }

    #[test]
    fn agent_chat_markdown_export_labels_roles_and_preserves_fences() {
        let markdown = build_agent_chat_conversation_markdown(&[
            message(1, AgentChatThreadMessageRole::User, "show rust"),
            message(
                2,
                AgentChatThreadMessageRole::Assistant,
                "```rust\nfn main() {}\n```",
            ),
            message(3, AgentChatThreadMessageRole::System, "saved"),
        ])
        .expect("markdown");

        assert!(markdown.starts_with("# Agent Chat Conversation"));
        assert!(markdown.contains("**You**\n\nshow rust"));
        assert!(markdown.contains("**Assistant**\n\n```rust\nfn main() {}\n```"));
        assert!(markdown.contains("**System**\n\nsaved"));
    }
}
