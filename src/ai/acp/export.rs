//! Shared ACP conversation markdown serializer.
//!
//! Used by both the shared action handler (`acp_export_markdown`,
//! `acp_save_as_note`) and the detached ACP Chat window export path.

use super::conversation_export::{AcpConversationExport, AcpExportPurpose};
use super::thread::{AcpThread, AcpThreadMessage, AcpThreadMessageRole};

const ACP_CHAT_CONVERSATION_HEADING: &str = "# Agent Chat Conversation\n\n";

fn role_label(role: &AcpThreadMessageRole) -> &'static str {
    match role {
        AcpThreadMessageRole::User => "**You**",
        AcpThreadMessageRole::Assistant => "**Assistant**",
        AcpThreadMessageRole::Thought => "**Thinking**",
        AcpThreadMessageRole::Tool => "**Tool**",
        AcpThreadMessageRole::System => "**System**",
        AcpThreadMessageRole::Error => "**Error**",
    }
}

/// Build a markdown document from ACP thread messages. Returns `None` if no
/// messages have non-empty renderable body text.
pub(crate) fn build_acp_conversation_markdown(messages: &[AcpThreadMessage]) -> Option<String> {
    let mut md = String::from(ACP_CHAT_CONVERSATION_HEADING);
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

pub(crate) fn build_acp_conversation_markdown_from_thread(thread: &AcpThread) -> Option<String> {
    let export = thread.export_conversation(AcpExportPurpose::CopyTranscript);
    build_acp_conversation_markdown_from_export(&export)
}

pub(crate) fn build_acp_conversation_markdown_from_export(
    export: &AcpConversationExport,
) -> Option<String> {
    let mut md = String::from(ACP_CHAT_CONVERSATION_HEADING);
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
