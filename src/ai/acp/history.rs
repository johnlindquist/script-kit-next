//! ACP conversation history persistence.
//!
//! - `acp-history.jsonl` — One-line summaries for Cmd+P browsing
//! - `acp-conversations/{session_id}.json` — Full message history for resume

use serde::{Deserialize, Serialize};

/// A single conversation history entry (summary for the index).
///
/// New fields (`title`, `preview`, `search_text`) are populated on save and
/// back-filled on read for older JSONL lines that lack them.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub(crate) struct AcpHistoryEntry {
    pub timestamp: String,
    pub first_message: String,
    pub message_count: usize,
    pub session_id: String,
    /// Short title derived from the first user message (max 100 chars).
    pub title: String,
    /// Preview derived from the last assistant message (max 160 chars).
    pub preview: String,
    /// Lowercased searchable text from the first few transcript turns.
    pub search_text: String,
}

/// Which field produced the strongest match in a history search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AcpHistorySearchField {
    Title,
    Preview,
    SearchText,
    Timestamp,
}

/// A single ranked search hit from [`search_history`].
#[derive(Debug, Clone)]
pub(crate) struct AcpHistorySearchHit {
    pub entry: AcpHistoryEntry,
    pub score: u32,
    pub matched_field: AcpHistorySearchField,
}

impl AcpHistoryEntry {
    /// Returns `title` if populated, otherwise falls back to `first_message`.
    pub(crate) fn title_display(&self) -> &str {
        if self.title.is_empty() {
            &self.first_message
        } else {
            &self.title
        }
    }

    /// Returns `preview` if populated, otherwise falls back to `first_message`.
    pub(crate) fn preview_display(&self) -> &str {
        if self.preview.is_empty() {
            &self.first_message
        } else {
            &self.preview
        }
    }
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

// ── Text helpers ─────────────────────────────────────────────────────

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    let mut out: String = value.chars().take(max_chars).collect();
    if value.chars().count() > max_chars {
        out.push('\u{2026}'); // …
    }
    out
}

fn normalize_search_text(value: &str) -> String {
    collapse_whitespace(value).to_lowercase()
}

// ── Index builder ────────────────────────────────────────────────────

/// Build a rich history entry from a full saved conversation.
///
/// Returns `None` if the conversation has no user message.
pub(crate) fn build_history_entry(conversation: &SavedConversation) -> Option<AcpHistoryEntry> {
    let first_user = conversation
        .messages
        .iter()
        .find(|m| m.role.eq_ignore_ascii_case("user"))?;

    let last_assistant = conversation
        .messages
        .iter()
        .rev()
        .find(|m| m.role.eq_ignore_ascii_case("assistant"));

    let title = truncate_chars(&collapse_whitespace(&first_user.body), 100);

    let preview_source = last_assistant
        .map(|m| m.body.as_str())
        .unwrap_or(first_user.body.as_str());
    let preview = truncate_chars(&collapse_whitespace(preview_source), 160);

    // Build a small transcript sample for full-text search.
    let mut transcript_sample = String::new();
    for msg in conversation.messages.iter().take(8) {
        transcript_sample.push_str(msg.role.as_str());
        transcript_sample.push_str(": ");
        transcript_sample.push_str(&collapse_whitespace(&msg.body));
        transcript_sample.push('\n');
    }

    Some(AcpHistoryEntry {
        timestamp: conversation.timestamp.clone(),
        first_message: truncate_chars(&collapse_whitespace(&first_user.body), 100),
        message_count: conversation.messages.len(),
        session_id: conversation.session_id.clone(),
        title: title.clone(),
        preview: preview.clone(),
        search_text: normalize_search_text(&format!(
            "{}\n{}\n{}\n{}",
            title, preview, transcript_sample, conversation.timestamp
        )),
    })
}

// ── Search / ranking ─────────────────────────────────────────────────

/// Rank history entries against a query using field-weighted scoring.
///
/// Empty query returns up to `limit` entries in recency order (no filtering).
fn rank_history_entries(
    entries: Vec<AcpHistoryEntry>,
    query: &str,
    limit: usize,
) -> Vec<AcpHistorySearchHit> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return entries
            .into_iter()
            .take(limit)
            .map(|entry| AcpHistorySearchHit {
                entry,
                score: 0,
                matched_field: AcpHistorySearchField::Title,
            })
            .collect();
    }

    let tokens: Vec<String> = normalize_search_text(trimmed)
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect();

    let mut hits = Vec::new();

    for entry in entries {
        let title = normalize_search_text(entry.title_display());
        let preview = normalize_search_text(entry.preview_display());
        let full = normalize_search_text(&entry.search_text);
        let timestamp = normalize_search_text(&entry.timestamp);
        let combined = format!("{title} {preview} {full} {timestamp}");

        // All query tokens must appear somewhere.
        if !tokens.iter().all(|token| combined.contains(token.as_str())) {
            continue;
        }

        let title_score = tokens.iter().fold(0u32, |acc, token| {
            acc + if title.starts_with(token.as_str()) {
                80
            } else if title.contains(token.as_str()) {
                40
            } else {
                0
            }
        });

        let preview_score = tokens.iter().fold(0u32, |acc, token| {
            acc + if preview.contains(token.as_str()) {
                20
            } else {
                0
            }
        });

        let full_score = tokens.iter().fold(0u32, |acc, token| {
            acc + if full.contains(token.as_str()) { 8 } else { 0 }
        });

        let timestamp_score = tokens.iter().fold(0u32, |acc, token| {
            acc + if timestamp.contains(token.as_str()) {
                4
            } else {
                0
            }
        });

        let total_score = title_score + preview_score + full_score + timestamp_score;

        let matched_field = if title_score >= preview_score
            && title_score >= full_score
            && title_score >= timestamp_score
        {
            AcpHistorySearchField::Title
        } else if preview_score >= full_score && preview_score >= timestamp_score {
            AcpHistorySearchField::Preview
        } else if full_score >= timestamp_score {
            AcpHistorySearchField::SearchText
        } else {
            AcpHistorySearchField::Timestamp
        };

        hits.push(AcpHistorySearchHit {
            entry,
            score: total_score,
            matched_field,
        });
    }

    // Deterministic: highest score first, recency as tie-break.
    hits.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| b.entry.timestamp.cmp(&a.entry.timestamp))
    });

    hits.truncate(limit);
    hits
}

/// Search loaded history entries, returning ranked hits.
///
/// Emits `acp_history_search_executed` structured log on every call.
pub(crate) fn search_history(query: &str, limit: usize) -> Vec<AcpHistorySearchHit> {
    let hits = rank_history_entries(load_history(), query, limit);
    tracing::info!(
        target: "script_kit::tab_ai",
        event = "acp_history_search_executed",
        query = %query,
        limit,
        hit_count = hits.len(),
    );
    hits
}

// ── Persistence paths ────────────────────────────────────────────────

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
///
/// Older entries written before the `title`/`preview`/`search_text` fields
/// existed are back-filled on read from `first_message` so that callers
/// always see populated display fields.
pub(crate) fn load_history() -> Vec<AcpHistoryEntry> {
    let path = history_path();
    let Ok(content) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };

    let mut entries: Vec<AcpHistoryEntry> = content
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .map(|mut entry: AcpHistoryEntry| {
            // Back-fill missing fields from legacy entries.
            if entry.title.is_empty() {
                entry.title = entry.first_message.clone();
            }
            if entry.preview.is_empty() {
                entry.preview = entry.first_message.clone();
            }
            if entry.search_text.is_empty() {
                entry.search_text = normalize_search_text(&format!(
                    "{}\n{}\n{}",
                    entry.title, entry.preview, entry.timestamp
                ));
            }
            entry
        })
        .collect();

    // Most recent first, then deduplicate (keeps latest per session_id)
    entries.reverse();
    let mut seen = std::collections::HashSet::new();
    entries.retain(|e| seen.insert(e.session_id.clone()));
    entries.truncate(100);
    entries
}

/// Load a full conversation by session ID.
pub(crate) fn load_conversation(session_id: &str) -> Option<SavedConversation> {
    let path = conversations_dir().join(format!("{session_id}.json"));
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Delete a single conversation by session ID.
///
/// Removes the saved conversation file and rewrites `acp-history.jsonl`
/// without the deleted `session_id`. Returns `Ok(())` even if the
/// session was not found (idempotent).
pub(crate) fn delete_conversation(session_id: &str) -> anyhow::Result<()> {
    use anyhow::Context;

    // Remove the conversation JSON file if it exists.
    let conversation_path = conversations_dir().join(format!("{session_id}.json"));
    if conversation_path.exists() {
        std::fs::remove_file(&conversation_path).with_context(|| {
            format!("remove saved conversation {}", conversation_path.display())
        })?;
    }

    // Rewrite the history index without the deleted session.
    let hp = history_path();
    if hp.exists() {
        let entries: Vec<AcpHistoryEntry> = load_history()
            .into_iter()
            .filter(|entry| entry.session_id != session_id)
            .collect();

        let mut out = String::new();
        for entry in &entries {
            if let Ok(json) = serde_json::to_string(entry) {
                out.push_str(&json);
                out.push('\n');
            }
        }
        std::fs::write(&hp, out).with_context(|| format!("rewrite {}", hp.display()))?;
    }

    tracing::info!(event = "acp_history_item_deleted", session_id = %session_id);
    Ok(())
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

    // ── Helpers ──────────────────────────────────────────────────────

    fn make_conversation(
        session_id: &str,
        timestamp: &str,
        messages: Vec<(&str, &str)>,
    ) -> SavedConversation {
        SavedConversation {
            session_id: session_id.to_string(),
            timestamp: timestamp.to_string(),
            messages: messages
                .into_iter()
                .map(|(role, body)| SavedMessage {
                    role: role.to_string(),
                    body: body.to_string(),
                })
                .collect(),
        }
    }

    // ── Serde roundtrip ─────────────────────────────────────────────

    #[test]
    fn history_entry_serializes_with_new_fields() {
        let entry = AcpHistoryEntry {
            timestamp: "2026-04-01T18:00:00Z".to_string(),
            first_message: "hello world".to_string(),
            message_count: 5,
            session_id: "test-123".to_string(),
            title: "hello world".to_string(),
            preview: "The answer is 42".to_string(),
            search_text: "hello world the answer is 42".to_string(),
        };
        let json = serde_json::to_string(&entry).expect("serialize");
        let parsed: AcpHistoryEntry = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.title, "hello world");
        assert_eq!(parsed.preview, "The answer is 42");
        assert!(!parsed.search_text.is_empty());
    }

    #[test]
    fn legacy_entry_without_new_fields_deserializes() {
        // Simulates an old JSONL line that has no title/preview/search_text.
        let legacy_json = r#"{"timestamp":"2026-03-01T12:00:00Z","first_message":"fix the login","message_count":3,"session_id":"legacy-1"}"#;
        let entry: AcpHistoryEntry =
            serde_json::from_str(legacy_json).expect("legacy entry should deserialize");
        assert_eq!(entry.first_message, "fix the login");
        // New fields default to empty strings.
        assert!(entry.title.is_empty());
        assert!(entry.preview.is_empty());
        assert!(entry.search_text.is_empty());
    }

    #[test]
    fn saved_conversation_serializes() {
        let conv = make_conversation(
            "test-456",
            "2026-04-01T18:00:00Z",
            vec![("user", "hello"), ("assistant", "hi there!")],
        );
        let json = serde_json::to_string_pretty(&conv).expect("serialize");
        assert!(json.contains("hello"));
        assert!(json.contains("hi there!"));
    }

    // ── build_history_entry ─────────────────────────────────────────

    #[test]
    fn build_entry_populates_title_preview_search_text() {
        let conv = make_conversation(
            "build-1",
            "2026-04-01T10:00:00Z",
            vec![
                ("user", "help me fix login"),
                (
                    "assistant",
                    "The root cause is an expired OAuth redirect URI",
                ),
            ],
        );
        let entry = build_history_entry(&conv).expect("should build");
        assert_eq!(entry.title, "help me fix login");
        assert!(entry.preview.contains("expired OAuth redirect URI"));
        assert!(entry.search_text.contains("oauth"));
        assert!(entry.search_text.contains("redirect"));
        assert_eq!(entry.message_count, 2);
    }

    #[test]
    fn build_entry_returns_none_without_user_message() {
        let conv = make_conversation(
            "no-user",
            "2026-04-01T10:00:00Z",
            vec![("assistant", "hello")],
        );
        assert!(build_history_entry(&conv).is_none());
    }

    #[test]
    fn build_entry_uses_first_user_for_preview_when_no_assistant() {
        let conv = make_conversation(
            "user-only",
            "2026-04-01T10:00:00Z",
            vec![("user", "just a question")],
        );
        let entry = build_history_entry(&conv).expect("should build");
        assert_eq!(entry.preview, "just a question");
    }

    #[test]
    fn build_entry_truncates_title_at_100_chars() {
        let long_msg = "a".repeat(200);
        let conv = make_conversation(
            "long-title",
            "2026-04-01T10:00:00Z",
            vec![("user", &long_msg)],
        );
        let entry = build_history_entry(&conv).expect("should build");
        // 100 chars + ellipsis
        assert!(entry.title.chars().count() <= 101);
        assert!(entry.title.ends_with('\u{2026}'));
    }

    #[test]
    fn build_entry_truncates_preview_at_160_chars() {
        let long_reply = "b".repeat(300);
        let conv = make_conversation(
            "long-preview",
            "2026-04-01T10:00:00Z",
            vec![("user", "question"), ("assistant", &long_reply)],
        );
        let entry = build_history_entry(&conv).expect("should build");
        assert!(entry.preview.chars().count() <= 161);
    }

    // ── title_display / preview_display ─────────────────────────────

    #[test]
    fn title_display_falls_back_to_first_message() {
        let entry = AcpHistoryEntry {
            first_message: "fallback title".to_string(),
            ..Default::default()
        };
        assert_eq!(entry.title_display(), "fallback title");

        let entry2 = AcpHistoryEntry {
            first_message: "ignored".to_string(),
            title: "real title".to_string(),
            ..Default::default()
        };
        assert_eq!(entry2.title_display(), "real title");
    }

    #[test]
    fn preview_display_falls_back_to_first_message() {
        let entry = AcpHistoryEntry {
            first_message: "fallback preview".to_string(),
            ..Default::default()
        };
        assert_eq!(entry.preview_display(), "fallback preview");
    }

    // ── Text helpers ────────────────────────────────────────────────

    #[test]
    fn collapse_whitespace_normalizes() {
        assert_eq!(collapse_whitespace("  a  b  c  "), "a b c");
        assert_eq!(collapse_whitespace("hello\n\nworld"), "hello world");
    }

    #[test]
    fn truncate_chars_adds_ellipsis() {
        assert_eq!(truncate_chars("abcde", 3), "abc\u{2026}");
        assert_eq!(truncate_chars("ab", 5), "ab");
    }

    // ── rank_history_entries / search ────────────────────────────────

    fn sample_entries() -> Vec<AcpHistoryEntry> {
        vec![
            AcpHistoryEntry {
                timestamp: "2026-04-01T10:00:00Z".to_string(),
                first_message: "help me fix login".to_string(),
                message_count: 4,
                session_id: "s1".to_string(),
                title: "help me fix login".to_string(),
                preview: "The root cause is an expired OAuth redirect URI".to_string(),
                search_text: normalize_search_text(
                    "help me fix login\nThe root cause is an expired OAuth redirect URI\nuser: help me fix login\nassistant: The root cause is an expired OAuth redirect URI",
                ),
            },
            AcpHistoryEntry {
                timestamp: "2026-04-02T10:00:00Z".to_string(),
                first_message: "add dark mode".to_string(),
                message_count: 3,
                session_id: "s2".to_string(),
                title: "add dark mode".to_string(),
                preview: "I added CSS variables for theming".to_string(),
                search_text: normalize_search_text(
                    "add dark mode\nI added CSS variables for theming\nuser: add dark mode\nassistant: I added CSS variables for theming",
                ),
            },
            AcpHistoryEntry {
                timestamp: "2026-04-03T10:00:00Z".to_string(),
                first_message: "review PR 42".to_string(),
                message_count: 6,
                session_id: "s3".to_string(),
                title: "review PR 42".to_string(),
                preview: "The PR looks good but the OAuth scope is too broad".to_string(),
                search_text: normalize_search_text(
                    "review PR 42\nThe PR looks good but the OAuth scope is too broad\nuser: review PR 42\nassistant: The PR looks good but the OAuth scope is too broad",
                ),
            },
        ]
    }

    #[test]
    fn empty_query_returns_all_up_to_limit() {
        let hits = rank_history_entries(sample_entries(), "", 100);
        assert_eq!(hits.len(), 3);
        // All scores should be 0 for empty query.
        assert!(hits.iter().all(|h| h.score == 0));
    }

    #[test]
    fn search_matches_later_transcript_content() {
        let hits = rank_history_entries(sample_entries(), "oauth redirect", 10);
        // "oauth redirect" appears in s1's preview and s3's preview.
        assert!(!hits.is_empty());
        // s1 has "redirect" in preview AND search_text → higher score.
        assert_eq!(hits[0].entry.session_id, "s1");
    }

    #[test]
    fn search_excludes_non_matching_entries() {
        let hits = rank_history_entries(sample_entries(), "nonexistent xyz", 10);
        assert!(hits.is_empty());
    }

    #[test]
    fn search_is_case_insensitive() {
        let hits = rank_history_entries(sample_entries(), "OAUTH", 10);
        assert!(!hits.is_empty());
    }

    #[test]
    fn search_multi_token_requires_all_tokens() {
        // "dark" matches s2, "oauth" matches s1/s3 → no entry has both.
        let hits = rank_history_entries(sample_entries(), "dark oauth", 10);
        assert!(hits.is_empty());
    }

    #[test]
    fn search_title_prefix_scores_highest() {
        let hits = rank_history_entries(sample_entries(), "help", 10);
        assert_eq!(hits[0].entry.session_id, "s1");
        assert_eq!(hits[0].matched_field, AcpHistorySearchField::Title);
    }

    #[test]
    fn search_respects_limit() {
        let hits = rank_history_entries(sample_entries(), "oauth", 1);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn search_recency_breaks_ties() {
        // Both s1 and s3 match "oauth", but with different scores.
        // If scores tied, s3 (later timestamp) would come first.
        let mut entries = sample_entries();
        // Make s1 and s3 have identical search_text so score is equal.
        let shared_text = normalize_search_text("oauth common content");
        entries[0].search_text = shared_text.clone();
        entries[0].title = "oauth common content".to_string();
        entries[0].preview = "oauth common content".to_string();
        entries[2].search_text = shared_text;
        entries[2].title = "oauth common content".to_string();
        entries[2].preview = "oauth common content".to_string();

        let hits = rank_history_entries(entries, "oauth", 10);
        assert!(hits.len() >= 2);
        // s3 has later timestamp → should come first when scores tie.
        assert_eq!(hits[0].entry.session_id, "s3");
        assert_eq!(hits[1].entry.session_id, "s1");
    }

    #[test]
    fn search_whitespace_only_query_returns_all() {
        let hits = rank_history_entries(sample_entries(), "   ", 100);
        assert_eq!(hits.len(), 3);
    }
}
