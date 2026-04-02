//! ACP conversation history persistence.
//!
//! - `acp-history.jsonl` — One-line summaries for Cmd+P browsing
//! - `acp-conversations/{session_id}.json` — Full message history for resume

use serde::{Deserialize, Serialize};

/// A single conversation history entry (summary for the index).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AcpHistoryEntry {
    pub timestamp: String,
    pub first_message: String,
    pub message_count: usize,
    pub session_id: String,
}

/// A saved message for full conversation persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SavedMessage {
    pub role: String,
    pub body: String,
}

/// Full conversation snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SavedConversation {
    pub session_id: String,
    pub timestamp: String,
    pub messages: Vec<SavedMessage>,
}

fn history_path() -> std::path::PathBuf {
    crate::setup::get_kit_path().join("acp-history.jsonl")
}

fn conversations_dir() -> std::path::PathBuf {
    crate::setup::get_kit_path().join("acp-conversations")
}

/// Append a history entry to the JSONL index file.
/// Compacts the file when it exceeds 200 lines.
pub(crate) fn save_history_entry(entry: &AcpHistoryEntry) {
    let path = history_path();
    let Ok(json) = serde_json::to_string(entry) else {
        return;
    };

    use std::io::Write;
    let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    else {
        tracing::debug!(path = %path.display(), "acp_history_write_failed");
        return;
    };
    let _ = writeln!(file, "{json}");

    // Compact when file grows too large (>200 lines)
    if let Ok(content) = std::fs::read_to_string(&path) {
        let line_count = content.lines().count();
        if line_count > 200 {
            let compacted = load_history();
            if let Ok(mut f) = std::fs::File::create(&path) {
                for e in compacted.iter().rev() {
                    if let Ok(j) = serde_json::to_string(e) {
                        let _ = writeln!(f, "{j}");
                    }
                }
            }
        }
    }
}

/// Save full conversation messages to a session-specific JSON file.
pub(crate) fn save_conversation(conversation: &SavedConversation) {
    let dir = conversations_dir();
    if std::fs::create_dir_all(&dir).is_err() {
        tracing::debug!(dir = %dir.display(), "acp_conversations_dir_create_failed");
        return;
    }

    let path = dir.join(format!("{}.json", conversation.session_id));
    match serde_json::to_string_pretty(conversation) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&path, json) {
                tracing::debug!(path = %path.display(), %e, "acp_conversation_write_failed");
            }
        }
        Err(e) => {
            tracing::debug!(%e, "acp_conversation_serialize_failed");
        }
    }

    // Clean up old conversations (keep most recent 50)
    cleanup_old_conversations(50);
}

/// Load history entries from the JSONL file (most recent first).
pub(crate) fn load_history() -> Vec<AcpHistoryEntry> {
    let path = history_path();
    let Ok(content) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };

    let mut entries: Vec<AcpHistoryEntry> = content
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    // Most recent first, then deduplicate (keeps latest per session_id)
    entries.reverse();
    let mut seen = std::collections::HashSet::new();
    entries.retain(|e| seen.insert(e.session_id.clone()));
    entries.truncate(100);
    entries
}

/// Remove oldest conversation files beyond the keep limit.
fn cleanup_old_conversations(keep: usize) {
    let dir = conversations_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return;
    };

    let mut files: Vec<(std::path::PathBuf, std::time::SystemTime)> = entries
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            let modified = e.metadata().ok()?.modified().ok()?;
            Some((path, modified))
        })
        .collect();

    if files.len() <= keep {
        return;
    }

    // Sort oldest first
    files.sort_by_key(|(_, t)| *t);

    // Remove oldest
    for (path, _) in files.iter().take(files.len() - keep) {
        let _ = std::fs::remove_file(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_entry_serializes() {
        let entry = AcpHistoryEntry {
            timestamp: "2026-04-01T18:00:00Z".to_string(),
            first_message: "hello world".to_string(),
            message_count: 5,
            session_id: "test-123".to_string(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("hello world"));
        let parsed: AcpHistoryEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.first_message, "hello world");
    }

    #[test]
    fn saved_conversation_serializes() {
        let conv = SavedConversation {
            session_id: "test-456".to_string(),
            timestamp: "2026-04-01T18:00:00Z".to_string(),
            messages: vec![
                SavedMessage {
                    role: "user".to_string(),
                    body: "hello".to_string(),
                },
                SavedMessage {
                    role: "assistant".to_string(),
                    body: "hi there!".to_string(),
                },
            ],
        };
        let json = serde_json::to_string_pretty(&conv).unwrap();
        assert!(json.contains("hello"));
        assert!(json.contains("hi there!"));
    }
}
