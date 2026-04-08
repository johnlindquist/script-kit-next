//! ACP history attachment artifacts.
//!
//! Writes a deterministic markdown file under `~/.scriptkit/acp-history-attachments/`
//! that can be attached to a new ACP chat via the existing `AiContextPart::FilePath` path.

use anyhow::{Context, Result};
use std::path::PathBuf;

/// Whether to attach a short summary or the full transcript.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AcpHistoryAttachMode {
    Summary,
    Transcript,
}

impl AcpHistoryAttachMode {
    pub(crate) fn file_stem(self) -> &'static str {
        match self {
            Self::Summary => "summary",
            Self::Transcript => "transcript",
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Summary => "Summary",
            Self::Transcript => "Transcript",
        }
    }
}

fn attachments_dir() -> PathBuf {
    crate::setup::get_kit_path().join("acp-history-attachments")
}

fn one_line(value: &str, max_chars: usize) -> String {
    let collapsed: String = value.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut out: String = collapsed.chars().take(max_chars).collect();
    if collapsed.chars().count() > max_chars {
        out.push('\u{2026}');
    }
    out
}

/// Format a conversation as a markdown attachment.
pub(crate) fn format_history_attachment_markdown(
    conversation: &super::history::SavedConversation,
    mode: AcpHistoryAttachMode,
) -> String {
    let entry = super::history::build_history_entry(conversation);
    let title = entry
        .as_ref()
        .map(|e| e.title_display().to_string())
        .unwrap_or_else(|| "Conversation".to_string());

    let mut out = String::new();
    out.push_str("# ACP Conversation\n\n");
    out.push_str(&format!("- session_id: {}\n", conversation.session_id));
    out.push_str(&format!("- timestamp: {}\n", conversation.timestamp));
    out.push_str(&format!("- mode: {}\n\n", mode.label()));
    out.push_str(&format!("## Title\n{}\n\n", title));

    match mode {
        AcpHistoryAttachMode::Summary => {
            out.push_str("## Summary\n");
            for msg in conversation.messages.iter().take(6) {
                out.push_str(&format!(
                    "- **{}**: {}\n",
                    msg.role,
                    one_line(&msg.body, 220)
                ));
            }
            out.push('\n');
        }
        AcpHistoryAttachMode::Transcript => {
            out.push_str("## Transcript\n\n");
            for msg in &conversation.messages {
                out.push_str(&format!("### {}\n{}\n\n", msg.role, msg.body));
            }
        }
    }

    out
}

/// Write a markdown attachment to disk and return (path, label).
pub(crate) fn write_history_attachment(
    session_id: &str,
    mode: AcpHistoryAttachMode,
) -> Result<(PathBuf, String)> {
    let conversation = super::history::load_conversation(session_id)
        .with_context(|| format!("missing ACP conversation {session_id}"))?;

    let dir = attachments_dir();
    std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;

    let path = dir.join(format!("{session_id}-{}.md", mode.file_stem()));
    let markdown = format_history_attachment_markdown(&conversation, mode);
    std::fs::write(&path, markdown).with_context(|| format!("write {}", path.display()))?;

    let title = super::history::build_history_entry(&conversation)
        .map(|e| e.title_display().to_string())
        .unwrap_or_else(|| session_id.to_string());

    tracing::info!(
        target: "script_kit::tab_ai",
        event = "acp_history_attachment_written",
        session_id = %session_id,
        mode = ?mode,
        path = %path.display(),
        title = %title,
    );

    Ok((
        path,
        format!("History \u{00b7} {} \u{00b7} {}", mode.label(), title),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::acp::history::{SavedConversation, SavedMessage};

    fn test_conversation() -> SavedConversation {
        SavedConversation {
            session_id: "test-attach-1".to_string(),
            timestamp: "2026-04-05T12:00:00Z".to_string(),
            messages: vec![
                SavedMessage {
                    role: "user".to_string(),
                    body: "help me fix login".to_string(),
                },
                SavedMessage {
                    role: "assistant".to_string(),
                    body: "The root cause is an expired OAuth redirect URI".to_string(),
                },
            ],
        }
    }

    #[test]
    fn summary_format_includes_title_and_messages() {
        let md =
            format_history_attachment_markdown(&test_conversation(), AcpHistoryAttachMode::Summary);
        assert!(md.contains("# ACP Conversation"));
        assert!(md.contains("help me fix login"));
        assert!(md.contains("mode: Summary"));
        assert!(md.contains("## Summary"));
    }

    #[test]
    fn transcript_format_includes_full_messages() {
        let md = format_history_attachment_markdown(
            &test_conversation(),
            AcpHistoryAttachMode::Transcript,
        );
        assert!(md.contains("## Transcript"));
        assert!(md.contains("### user"));
        assert!(md.contains("### assistant"));
        assert!(md.contains("expired OAuth redirect URI"));
    }

    #[test]
    fn one_line_truncates() {
        assert_eq!(one_line("hello world", 5), "hello\u{2026}");
        assert_eq!(one_line("hi", 10), "hi");
    }

    #[test]
    fn attach_mode_labels() {
        assert_eq!(AcpHistoryAttachMode::Summary.label(), "Summary");
        assert_eq!(AcpHistoryAttachMode::Transcript.label(), "Transcript");
        assert_eq!(AcpHistoryAttachMode::Summary.file_stem(), "summary");
        assert_eq!(AcpHistoryAttachMode::Transcript.file_stem(), "transcript");
    }
}
