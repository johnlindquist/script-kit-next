//! Structured tool-call card metadata for the Agent Chat transcript.
//!
//! Pi tool events carry the tool name, raw input `args`, and (for edit/write
//! tools) a pre-rendered diff in `result.details.diff`. This module turns
//! those protocol fields into typed metadata the transcript can render as a
//! card — status badge, kind glyph, subject line, and colored diff body —
//! without re-parsing formatted message text.

use serde_json::Value;

/// Semantic kind of a tool call, derived from the Pi tool name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentChatToolKind {
    Read,
    Edit,
    Write,
    Execute,
    Search,
    Fetch,
    Other,
}

impl AgentChatToolKind {
    pub(crate) fn from_tool_name(name: &str) -> Self {
        let normalized = name.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "read" | "read_file" | "cat" => Self::Read,
            "edit" | "hashline_edit" | "multi_edit" | "apply_patch" => Self::Edit,
            "write" | "create_file" | "write_file" => Self::Write,
            "bash" | "shell" | "exec" | "terminal" => Self::Execute,
            "grep" | "glob" | "find" | "ls" | "list" | "rg" | "search" => Self::Search,
            "fetch" | "web_fetch" | "web_search" | "curl" => Self::Fetch,
            _ => Self::Other,
        }
    }

    /// Compact glyph rendered before the tool title.
    pub(crate) fn glyph(self) -> &'static str {
        match self {
            Self::Read => "\u{1F4C4}",   // 📄
            Self::Edit => "\u{270E}",    // ✎
            Self::Write => "\u{1F4DD}",  // 📝
            Self::Execute => "\u{276F}", // ❯
            Self::Search => "\u{1F50D}", // 🔍
            Self::Fetch => "\u{1F310}",  // 🌐
            Self::Other => "\u{2699}",   // ⚙
        }
    }
}

/// Lifecycle status of a tool call, derived from Pi status strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentChatToolStatus {
    Pending,
    Running,
    Complete,
    Failed,
}

impl AgentChatToolStatus {
    pub(crate) fn from_status_str(status: &str) -> Self {
        match status.trim().to_ascii_lowercase().as_str() {
            "pending" => Self::Pending,
            "complete" | "completed" | "done" | "success" => Self::Complete,
            "failed" | "error" => Self::Failed,
            _ => Self::Running,
        }
    }

    pub(crate) fn glyph(self) -> &'static str {
        match self {
            Self::Pending => "\u{25CB}",  // ○
            Self::Running => "\u{25CF}",  // ●
            Self::Complete => "\u{2713}", // ✓
            Self::Failed => "\u{2715}",   // ✕
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Complete => "done",
            Self::Failed => "failed",
        }
    }
}

/// Typed card metadata attached to a Tool transcript message.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AgentChatToolCardMeta {
    /// Raw Pi tool name (e.g. "bash", "edit").
    pub tool_name: String,
    pub kind: AgentChatToolKind,
    pub status: AgentChatToolStatus,
    /// Compact subject extracted from the tool args (path, command, query).
    pub subject: Option<String>,
    /// Pre-rendered line-numbered diff from `result.details.diff` (edit/write).
    pub diff: Option<String>,
    pub is_error: bool,
}

const SUBJECT_MAX_CHARS: usize = 96;

/// Extract a one-line human subject from tool args.
///
/// Priority follows what users scan for: file path, then command, then
/// query-ish fields. Falls back to a compact JSON rendering of the args.
pub(crate) fn subject_from_args(args: &Value) -> Option<String> {
    let object = args.as_object()?;
    const PRIORITY_KEYS: [&str; 9] = [
        "path",
        "file_path",
        "filename",
        "cmd",
        "command",
        "pattern",
        "query",
        "url",
        "prompt",
    ];
    for key in PRIORITY_KEYS {
        if let Some(text) = object.get(key).and_then(Value::as_str) {
            let compact = compact_single_line(text);
            if !compact.is_empty() {
                return Some(compact);
            }
        }
    }
    if object.is_empty() {
        return None;
    }
    let rendered = Value::Object(object.clone()).to_string();
    let compact = compact_single_line(&rendered);
    (!compact.is_empty()).then_some(compact)
}

fn compact_single_line(text: &str) -> String {
    let mut joined = text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();
    if joined.chars().count() > SUBJECT_MAX_CHARS {
        joined = joined.chars().take(SUBJECT_MAX_CHARS - 1).collect();
        joined.push('\u{2026}');
    }
    joined
}

/// Classification of one line in a Pi-rendered diff string.
///
/// Pi's `generate_diff_string` emits `+<num> <line>`, `-<num> <line>`, and
/// ` <num> <line>` context rows (plus ` ... ` skip markers).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiffLineKind {
    Added,
    Removed,
    Context,
}

pub(crate) fn classify_diff_line(line: &str) -> DiffLineKind {
    match line.as_bytes().first() {
        Some(b'+') => DiffLineKind::Added,
        Some(b'-') => DiffLineKind::Removed,
        _ => DiffLineKind::Context,
    }
}

/// Extract `details.diff` from a Pi tool result value (`result` or
/// `partialResult`), if present.
pub(crate) fn diff_from_tool_result(result: &Value) -> Option<String> {
    let diff = result.get("details")?.get("diff")?.as_str()?;
    (!diff.trim().is_empty()).then(|| diff.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn tool_kind_maps_known_pi_tool_names() {
        assert_eq!(
            AgentChatToolKind::from_tool_name("bash"),
            AgentChatToolKind::Execute
        );
        assert_eq!(
            AgentChatToolKind::from_tool_name("edit"),
            AgentChatToolKind::Edit
        );
        assert_eq!(
            AgentChatToolKind::from_tool_name("hashline_edit"),
            AgentChatToolKind::Edit
        );
        assert_eq!(
            AgentChatToolKind::from_tool_name("write"),
            AgentChatToolKind::Write
        );
        assert_eq!(
            AgentChatToolKind::from_tool_name("read"),
            AgentChatToolKind::Read
        );
        assert_eq!(
            AgentChatToolKind::from_tool_name("grep"),
            AgentChatToolKind::Search
        );
        assert_eq!(
            AgentChatToolKind::from_tool_name("mystery_tool"),
            AgentChatToolKind::Other
        );
    }

    #[test]
    fn tool_status_maps_pi_status_strings() {
        assert_eq!(
            AgentChatToolStatus::from_status_str("pending"),
            AgentChatToolStatus::Pending
        );
        assert_eq!(
            AgentChatToolStatus::from_status_str("running"),
            AgentChatToolStatus::Running
        );
        assert_eq!(
            AgentChatToolStatus::from_status_str("complete"),
            AgentChatToolStatus::Complete
        );
        assert_eq!(
            AgentChatToolStatus::from_status_str("failed"),
            AgentChatToolStatus::Failed
        );
        // Unknown strings stay visually "in flight" rather than lying about completion.
        assert_eq!(
            AgentChatToolStatus::from_status_str("in_progress"),
            AgentChatToolStatus::Running
        );
    }

    #[test]
    fn subject_prefers_path_then_command() {
        assert_eq!(
            subject_from_args(&json!({"path": "src/main.rs", "oldText": "a"})).as_deref(),
            Some("src/main.rs")
        );
        assert_eq!(
            subject_from_args(&json!({"cmd": "  cargo   test  "})).as_deref(),
            Some("cargo test")
        );
        assert_eq!(subject_from_args(&json!({})), None);
        assert_eq!(subject_from_args(&json!("not-an-object")), None);
    }

    #[test]
    fn subject_falls_back_to_compact_json_and_truncates() {
        let subject = subject_from_args(&json!({"depth": 3})).unwrap();
        assert!(subject.contains("depth"));

        let long = "x".repeat(500);
        let subject = subject_from_args(&json!({ "cmd": long })).unwrap();
        assert!(subject.chars().count() <= SUBJECT_MAX_CHARS);
        assert!(subject.ends_with('\u{2026}'));
    }

    #[test]
    fn diff_lines_classify_by_pi_marker_prefix() {
        assert_eq!(classify_diff_line("+ 12 added"), DiffLineKind::Added);
        assert_eq!(classify_diff_line("-  9 removed"), DiffLineKind::Removed);
        assert_eq!(classify_diff_line("  10 context"), DiffLineKind::Context);
        assert_eq!(classify_diff_line("     ..."), DiffLineKind::Context);
    }

    #[test]
    fn diff_extraction_reads_details_diff() {
        let result = json!({
            "content": [{"type": "text", "text": "Successfully replaced"}],
            "details": {"diff": "+1 new line", "firstChangedLine": 1}
        });
        assert_eq!(
            diff_from_tool_result(&result).as_deref(),
            Some("+1 new line")
        );
        assert_eq!(diff_from_tool_result(&json!({"details": {}})), None);
        assert_eq!(
            diff_from_tool_result(&json!({"details": {"diff": "  "}})),
            None
        );
    }
}
