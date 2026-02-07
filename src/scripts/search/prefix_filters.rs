use super::super::types::{Script, Scriptlet};
use super::contains_ignore_ascii_case;

// ============================================
// PREFIX FILTER SEARCH SYNTAX
// ============================================
// Supports structured prefix filters like:
//   tag:productivity, author:john, kit:cleanshot,
//   is:cron, is:bg, is:watch, is:system, is:scheduled,
//   type:script, type:snippet, type:command, type:app,
//   group:dev, tool:bash

/// Represents a parsed query with an optional structured filter prefix.
/// E.g., "tag:productivity notes" -> filter_kind="tag", filter_value="productivity", remainder="notes"
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedQuery {
    /// Filter kind: "tag", "author", "kit", "is", "type", "group", "tool"
    pub filter_kind: Option<String>,
    /// Filter value (the text after the colon, before any space)
    pub filter_value: Option<String>,
    /// Remaining query text for fuzzy matching
    pub remainder: String,
}

/// Parse a query string for optional prefix filter syntax.
/// Supports: tag:X, author:X, kit:X, is:X, type:X, group:X, tool:X
/// E.g., "tag:productivity notes" -> { filter_kind: "tag", filter_value: "productivity", remainder: "notes" }
/// E.g., "is:cron" -> { filter_kind: "is", filter_value: "cron", remainder: "" }
/// E.g., "hello world" -> { filter_kind: None, filter_value: None, remainder: "hello world" }
pub fn parse_query_prefix(query: &str) -> ParsedQuery {
    let trimmed = query.trim();

    // Check for recognized prefix patterns
    let prefixes = ["tag:", "author:", "kit:", "is:", "type:", "group:", "tool:"];

    for prefix in &prefixes {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            let kind = prefix.trim_end_matches(':').to_string();
            // Split filter value from remainder at first space
            let (value, remainder) = match rest.find(' ') {
                Some(pos) => (rest[..pos].to_string(), rest[pos + 1..].trim().to_string()),
                None => (rest.to_string(), String::new()),
            };
            if value.is_empty() {
                // "tag:" with no value - treat as regular query
                return ParsedQuery {
                    filter_kind: None,
                    filter_value: None,
                    remainder: trimmed.to_string(),
                };
            }
            return ParsedQuery {
                filter_kind: Some(kind),
                filter_value: Some(value.to_lowercase()),
                remainder,
            };
        }
    }

    ParsedQuery {
        filter_kind: None,
        filter_value: None,
        remainder: trimmed.to_string(),
    }
}

/// Check if a script passes a prefix filter.
/// Returns true if no filter is active or if the script matches the filter.
pub(crate) fn script_passes_prefix_filter(script: &Script, parsed: &ParsedQuery) -> bool {
    let (kind, value) = match (&parsed.filter_kind, &parsed.filter_value) {
        (Some(k), Some(v)) => (k.as_str(), v.as_str()),
        _ => return true, // No filter active
    };

    match kind {
        "tag" => {
            if let Some(ref meta) = script.typed_metadata {
                meta.tags
                    .iter()
                    .any(|t| contains_ignore_ascii_case(t, value))
            } else {
                false
            }
        }
        "author" => script
            .typed_metadata
            .as_ref()
            .and_then(|m| m.author.as_deref())
            .is_some_and(|a| contains_ignore_ascii_case(a, value)),
        "kit" => script
            .kit_name
            .as_deref()
            .is_some_and(|k| contains_ignore_ascii_case(k, value)),
        "is" => {
            if let Some(ref meta) = script.typed_metadata {
                match value {
                    "cron" => meta.cron.is_some(),
                    "scheduled" | "schedule" => meta.cron.is_some() || meta.schedule.is_some(),
                    "bg" | "background" => meta.background,
                    "watch" | "watching" => !meta.watch.is_empty(),
                    "system" | "sys" => meta.system,
                    _ => false,
                }
            } else {
                false
            }
        }
        "type" => matches!(value, "script" | "scripts"),
        // group: and tool: don't apply to scripts
        "group" | "tool" => false,
        _ => true,
    }
}

/// Check if a scriptlet passes a prefix filter.
pub(crate) fn scriptlet_passes_prefix_filter(scriptlet: &Scriptlet, parsed: &ParsedQuery) -> bool {
    let (kind, value) = match (&parsed.filter_kind, &parsed.filter_value) {
        (Some(k), Some(v)) => (k.as_str(), v.as_str()),
        _ => return true,
    };

    match kind {
        "group" => scriptlet
            .group
            .as_deref()
            .is_some_and(|g| contains_ignore_ascii_case(g, value)),
        "tool" => {
            contains_ignore_ascii_case(&scriptlet.tool, value)
                || contains_ignore_ascii_case(scriptlet.tool_display_name(), value)
        }
        "type" => matches!(value, "snippet" | "snippets" | "scriptlet" | "scriptlets"),
        // tag:, author:, kit:, is: don't apply to scriptlets (they don't have these fields)
        "tag" | "author" | "kit" | "is" => false,
        _ => true,
    }
}

/// Check if a built-in passes a prefix filter.
pub(crate) fn builtin_passes_prefix_filter(parsed: &ParsedQuery) -> bool {
    let (kind, value) = match (&parsed.filter_kind, &parsed.filter_value) {
        (Some(k), Some(v)) => (k.as_str(), v.as_str()),
        _ => return true,
    };
    match kind {
        "type" => matches!(value, "command" | "commands" | "builtin" | "builtins"),
        // Other filters don't apply to builtins
        _ => false,
    }
}

/// Check if an app passes a prefix filter.
pub(crate) fn app_passes_prefix_filter(parsed: &ParsedQuery) -> bool {
    let (kind, value) = match (&parsed.filter_kind, &parsed.filter_value) {
        (Some(k), Some(v)) => (k.as_str(), v.as_str()),
        _ => return true,
    };
    match kind {
        "type" => matches!(value, "app" | "apps"),
        _ => false,
    }
}

/// Check if a window passes a prefix filter.
pub(crate) fn window_passes_prefix_filter(parsed: &ParsedQuery) -> bool {
    let (kind, value) = match (&parsed.filter_kind, &parsed.filter_value) {
        (Some(k), Some(v)) => (k.as_str(), v.as_str()),
        _ => return true,
    };
    match kind {
        "type" => matches!(value, "window" | "windows"),
        _ => false,
    }
}

/// Check if scripts should be searched at all given the filter.
/// Returns false when the filter targets a category that scripts can never match
/// (e.g., type:snippet, group:X, tool:X).
pub(crate) fn should_search_scripts(parsed: &ParsedQuery) -> bool {
    match (
        parsed.filter_kind.as_deref(),
        parsed.filter_value.as_deref(),
    ) {
        (None, _) => true,
        (Some("type"), Some(v)) => matches!(v, "script" | "scripts"),
        (Some("tag" | "author" | "kit" | "is"), _) => true,
        (Some("group" | "tool"), _) => false,
        _ => true,
    }
}

/// Check if scriptlets should be searched at all given the filter.
/// Returns false when the filter targets a category that scriptlets can never match
/// (e.g., type:script, tag:X, author:X, kit:X, is:X).
pub(crate) fn should_search_scriptlets(parsed: &ParsedQuery) -> bool {
    match (
        parsed.filter_kind.as_deref(),
        parsed.filter_value.as_deref(),
    ) {
        (None, _) => true,
        (Some("type"), Some(v)) => matches!(v, "snippet" | "snippets" | "scriptlet" | "scriptlets"),
        (Some("group" | "tool"), _) => true,
        (Some("tag" | "author" | "kit" | "is"), _) => false,
        _ => true,
    }
}
