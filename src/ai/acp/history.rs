//! ACP conversation history persistence.
//!
//! Saves conversation summaries to `~/.scriptkit/acp-history.jsonl` for
//! future Cmd+P browsing. Each line is a JSON object with timestamp,
//! first user message, and message count.

use serde::{Deserialize, Serialize};

/// A single conversation history entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AcpHistoryEntry {
    pub timestamp: String,
    pub first_message: String,
    pub message_count: usize,
    pub session_id: String,
}

/// Append a history entry to the JSONL file.
pub(crate) fn save_history_entry(entry: &AcpHistoryEntry) {
    let path = crate::setup::get_kit_path().join("acp-history.jsonl");
    let Ok(json) = serde_json::to_string(entry) else {
        return;
    };

    // Append to file (create if needed)
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
}

/// Load history entries from the JSONL file (most recent first).
pub(crate) fn load_history() -> Vec<AcpHistoryEntry> {
    let path = crate::setup::get_kit_path().join("acp-history.jsonl");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };

    let mut entries: Vec<AcpHistoryEntry> = content
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    // Most recent first
    entries.reverse();

    // Keep at most 100 entries
    entries.truncate(100);
    entries
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
}
