use crate::ai::message_parts::AiContextPart;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ParsedContextMentions {
    pub(crate) cleaned_content: String,
    pub(crate) parts: Vec<AiContextPart>,
}

impl ParsedContextMentions {
    pub(crate) fn has_parts(&self) -> bool {
        !self.parts.is_empty()
    }
}

/// A single inline `@mention` token found in text, with its character range
/// and the resolved `AiContextPart`.
#[derive(Debug, Clone, PartialEq)]
pub struct InlineContextMention {
    /// Character-level range of the token in the source text.
    pub range: std::ops::Range<usize>,
    /// The raw token text (e.g. `@browser`, `@file:/tmp/demo.rs`).
    pub token: String,
    /// The canonical token for this mention, used for ownership tracking and
    /// highlight matching. Aliases resolve to the primary mention token.
    pub canonical_token: String,
    /// The resolved context part for this mention.
    pub part: AiContextPart,
}

pub(crate) fn parse_context_mentions(raw_content: &str) -> ParsedContextMentions {
    let mut cleaned_lines = Vec::new();
    let mut parts = Vec::new();

    for line in raw_content.lines() {
        if let Some(part) = parse_context_mention_line(line) {
            tracing::info!(
                target: "ai",
                directive = line.trim(),
                label = part.label(),
                "context_mention_parsed"
            );
            parts.push(part);
        } else {
            cleaned_lines.push(line);
        }
    }

    let cleaned_content = cleaned_lines
        .join("\n")
        .trim_matches(|c| c == '\n' || c == '\r')
        .to_string();

    tracing::info!(
        target: "ai",
        raw_len = raw_content.len(),
        cleaned_len = cleaned_content.len(),
        parts_count = parts.len(),
        "context_mentions_parse_complete"
    );

    ParsedContextMentions {
        cleaned_content,
        parts,
    }
}

/// Trim trailing punctuation that is not part of the mention token itself.
/// E.g. `@browser,` → `@browser`, `@git-diff.` → `@git-diff`.
fn trim_inline_token_trailing_punctuation(token: &str) -> &str {
    token.trim_end_matches([',', '.', ';', ':', ')', ']', '}'])
}

/// Scan a single inline token starting at `start` (which must be `@`).
/// Handles quoted `@word:"..."` / `@word:'...'` tokens (any prefix, not just
/// `@file:`) that may contain spaces and escape sequences.
/// Returns `(raw_token, next_index)`.
fn scan_inline_token(chars: &[char], start: usize) -> (String, usize) {
    // Look for `@word:` prefix followed by a quote — generalized from @file: only.
    // Scan ahead for a colon within the first ~10 chars after @.
    let mut colon_pos = None;
    for (offset, &ch) in chars
        .get(start + 1..chars.len().min(start + 12))
        .unwrap_or(&[])
        .iter()
        .enumerate()
    {
        if ch == ':' {
            colon_pos = Some(start + 1 + offset);
            break;
        }
        if ch.is_whitespace() || ch == '@' {
            break;
        }
    }

    if let Some(cp) = colon_pos {
        // Check if char after colon is a quote
        if let Some(&quote) = chars.get(cp + 1) {
            if quote == '"' || quote == '\'' {
                let mut i = cp + 2;
                let mut escaped = false;
                while i < chars.len() {
                    let ch = chars[i];
                    if escaped {
                        escaped = false;
                        i += 1;
                        continue;
                    }
                    if ch == '\\' {
                        escaped = true;
                        i += 1;
                        continue;
                    }
                    if ch == quote {
                        i += 1; // consume closing quote
                        break;
                    }
                    i += 1;
                }
                return (chars[start..i].iter().collect(), i);
            }
        }
    }

    // Default: whitespace-delimited token
    let mut i = start + 1;
    while i < chars.len() && !chars[i].is_whitespace() {
        i += 1;
    }
    (chars[start..i].iter().collect(), i)
}

/// Strip outer quotes and unescape `\\`, `\"`, `\'` inside a quoted path.
fn unescape_quoted_path(path: &str) -> String {
    let bytes = path.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        let inner = &path[1..path.len() - 1];
        inner
            .replace("\\\\", "\0ESCAPED_BACKSLASH\0")
            .replace("\\\"", "\"")
            .replace("\\'", "'")
            .replace("\0ESCAPED_BACKSLASH\0", "\\")
    } else {
        path.to_string()
    }
}

/// Format a file path as a canonical inline `@file:` token, quoting paths
/// that contain whitespace.
pub(crate) fn format_inline_file_token(path: &str) -> String {
    if path.chars().any(char::is_whitespace) {
        format!(
            "@file:\"{}\"",
            path.replace('\\', "\\\\").replace('"', "\\\"")
        )
    } else {
        format!("@file:{path}")
    }
}

/// Maximum display name length for typed mention tokens.
const TYPED_MENTION_NAME_MAX_LEN: usize = 7;
const TYPED_MENTION_LABEL_MAX_LEN: usize = 14;

/// Map a file extension to a short type prefix for typed mentions.
pub(crate) fn typed_mention_prefix(path: &str) -> &'static str {
    let p = Path::new(path);

    // Directories first.
    if p.extension().is_none() && !path.contains('.') {
        // Heuristic: no extension and no dots = likely a directory or binary.
        // For actual directory detection, the caller would need to check fs.
        // We default to "file" and let the caller override for known dirs.
        return "file";
    }

    match p
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
        .as_str()
    {
        "rs" => "rs",
        "ts" | "tsx" => "ts",
        "js" | "jsx" | "mjs" | "cjs" => "js",
        "py" => "py",
        "rb" => "rb",
        "go" => "go",
        "java" => "java",
        "swift" => "swift",
        "c" | "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" => "cpp",
        "md" | "mdx" => "md",
        "json" => "json",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "xml" => "xml",
        "html" | "htm" => "html",
        "css" | "scss" | "less" => "css",
        "sh" | "bash" | "zsh" | "fish" => "sh",
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "ico" | "bmp" | "tiff" => "img",
        "mp4" | "mov" | "avi" | "mkv" | "webm" => "vid",
        "mp3" | "wav" | "flac" | "ogg" | "aac" => "audio",
        "sql" => "sql",
        "txt" | "log" => "txt",
        _ => "file",
    }
}

/// Map a file path to a "dir" prefix if it looks like a directory.
pub(crate) fn typed_mention_prefix_for_dir() -> &'static str {
    "dir"
}

/// Extract a short display name from a file path: filename without extension,
/// truncated to `TYPED_MENTION_NAME_MAX_LEN` characters.
pub(crate) fn typed_mention_display_name(path: &str) -> String {
    let p = Path::new(path);
    let stem = p
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_else(|| p.file_name().and_then(|n| n.to_str()).unwrap_or("file"));

    let mut name: String = stem.chars().take(TYPED_MENTION_NAME_MAX_LEN).collect();
    if stem.len() > TYPED_MENTION_NAME_MAX_LEN {
        name.push('\u{2026}'); // ellipsis
    }
    name
}

/// Format a typed mention token: `@type:name` or `@type:"name with spaces"`.
pub(crate) fn format_typed_mention_token(prefix: &str, name: &str) -> String {
    if name.chars().any(|ch| ch.is_whitespace()) {
        format!(
            "@{prefix}:\"{}\"",
            name.replace('\\', "\\\\").replace('"', "\\\"")
        )
    } else {
        format!("@{prefix}:{name}")
    }
}

fn strip_typed_mention_label_prefix(label: &str) -> &str {
    let trimmed = label.trim();
    let Some((prefix, rest)) = trimmed.split_once(':') else {
        return trimmed;
    };

    let normalized = prefix.trim().to_ascii_lowercase();
    let is_known_prefix = matches!(
        normalized.as_str(),
        "file"
            | "folder"
            | "search"
            | "input"
            | "clipboard"
            | "command"
            | "window"
            | "app"
            | "process"
            | "menu command"
            | "agent"
            | "suggestion"
            | "selection"
            | "context"
            | "ambient"
    );

    if is_known_prefix && !rest.trim().is_empty() {
        rest.trim()
    } else {
        trimmed
    }
}

fn typed_mention_label_name(label: &str) -> String {
    let display = strip_typed_mention_label_prefix(label);
    let display_len = display.chars().count();
    let mut name: String = display.chars().take(TYPED_MENTION_LABEL_MAX_LEN).collect();
    if display_len > TYPED_MENTION_LABEL_MAX_LEN {
        name.push('\u{2026}');
    }
    name
}

/// Scan `text` for inline `@mention` tokens and resolve each to an
/// `AiContextPart`. Supports built-in mentions (`@browser`, `@git-status`,
/// etc.), file mentions (`@file:/path`), and typed mentions (`@type:name`)
/// via the session alias registry.
pub fn parse_inline_context_mentions(text: &str) -> Vec<InlineContextMention> {
    parse_inline_context_mentions_with_aliases(text, &std::collections::HashMap::new())
}

/// Parse inline mentions with a session-local alias registry for typed
/// `@type:name` tokens. Aliases map short display tokens to full `AiContextPart`s.
pub fn parse_inline_context_mentions_with_aliases(
    text: &str,
    aliases: &std::collections::HashMap<String, AiContextPart>,
) -> Vec<InlineContextMention> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '@' {
            i += 1;
            continue;
        }
        // `@` must be at start or preceded by whitespace/punctuation
        if i > 0 && (chars[i - 1].is_alphanumeric() || chars[i - 1] == '_') {
            i += 1;
            continue;
        }

        let start = i;
        let (raw_token, next_i) = scan_inline_token(&chars, start);
        i = next_i;

        let trimmed = trim_inline_token_trailing_punctuation(&raw_token);
        let trimmed_char_len = trimmed.chars().count();
        let end = start + trimmed_char_len;

        // Resolution order: built-in → @file:/path → alias registry
        let part = resolve_builtin_mention_token(trimmed)
            .or_else(|| parse_file_mention(trimmed))
            .or_else(|| aliases.get(trimmed).cloned());

        if let Some(part) = part {
            let canonical_token =
                part_to_inline_token(&part).unwrap_or_else(|| trimmed.to_string());
            tracing::info!(
                target: "ai",
                event = "inline_context_token_resolved",
                token = %trimmed,
                canonical_token = %canonical_token,
                source = %part.source(),
                label = %part.label(),
            );
            out.push(InlineContextMention {
                range: start..end,
                token: trimmed.to_string(),
                canonical_token,
                part,
            });
        }
    }
    out
}

/// Return the character-level range of the inline mention token whose span
/// covers `cursor`. Returns `None` when the cursor is not inside or at the
/// boundary of any recognised mention.
pub fn mention_range_at_cursor(text: &str, cursor: usize) -> Option<std::ops::Range<usize>> {
    mention_range_at_cursor_with_aliases(text, cursor, &std::collections::HashMap::new())
}

/// Alias-aware variant of `mention_range_at_cursor`.
pub fn mention_range_at_cursor_with_aliases(
    text: &str,
    cursor: usize,
    aliases: &std::collections::HashMap<String, AiContextPart>,
) -> Option<std::ops::Range<usize>> {
    parse_inline_context_mentions_with_aliases(text, aliases)
        .into_iter()
        .find(|mention| cursor > mention.range.start && cursor <= mention.range.end)
        .map(|mention| mention.range)
}

/// Return the character-level range of the inline mention token to remove for
/// atomic deletion.
///
/// Backspace matches when the cursor is inside or at the trailing boundary.
/// Delete matches when the cursor is inside or at the leading boundary.
pub fn mention_range_for_atomic_delete(
    text: &str,
    cursor: usize,
    delete_forward: bool,
) -> Option<std::ops::Range<usize>> {
    mention_range_for_atomic_delete_with_aliases(
        text,
        cursor,
        delete_forward,
        &std::collections::HashMap::new(),
    )
}

/// Alias-aware variant of `mention_range_for_atomic_delete`.
pub fn mention_range_for_atomic_delete_with_aliases(
    text: &str,
    cursor: usize,
    delete_forward: bool,
    aliases: &std::collections::HashMap<String, AiContextPart>,
) -> Option<std::ops::Range<usize>> {
    if !delete_forward {
        return mention_range_at_cursor_with_aliases(text, cursor, aliases);
    }
    if let Some(range) = mention_range_at_cursor_with_aliases(text, cursor, aliases) {
        return Some(range);
    }
    mention_range_at_cursor_with_aliases(text, cursor.saturating_add(1), aliases)
        .filter(|range| cursor == range.start)
}

/// Convert an `AiContextPart` back to its canonical inline `@token` form.
///
/// Uses typed `@type:name` format for files, commands, and ambient context.
/// Built-in `ResourceUri` parts use their existing `@mention` tokens.
pub(crate) fn part_to_inline_token(part: &AiContextPart) -> Option<String> {
    match part {
        AiContextPart::ResourceUri { uri, .. } => {
            crate::ai::context_contract::context_attachment_specs()
                .iter()
                .find(|spec| spec.uri == uri.as_str())
                .and_then(|spec| spec.mention)
                .map(ToString::to_string)
        }
        AiContextPart::FilePath { path, .. } => {
            let prefix = typed_mention_prefix(path);
            let name = typed_mention_display_name(path);
            Some(format_typed_mention_token(prefix, &name))
        }
        AiContextPart::FocusedTarget {
            target, label: _, ..
        } => {
            // File/directory targets use typed file prefixes (@rs:, @dir:, etc.)
            if let Some(path) = (target.kind == "file" || target.kind == "directory")
                .then_some(target.metadata.as_ref())
                .flatten()
                .and_then(|metadata| metadata.get("path"))
                .and_then(|path| path.as_str())
            {
                let prefix = if target.kind == "directory" {
                    typed_mention_prefix_for_dir()
                } else {
                    typed_mention_prefix(path)
                };
                let name = typed_mention_display_name(path);
                return Some(format_typed_mention_token(prefix, &name));
            }

            // Non-file targets use @cmd: prefix.
            let name = typed_mention_label_name(&target.label);
            Some(format_typed_mention_token("cmd", &name))
        }
        AiContextPart::AmbientContext { label } => {
            let name = typed_mention_label_name(label);
            Some(format_typed_mention_token("env", &name))
        }
    }
}

/// Legacy format for backward compatibility — used only by the parser to
/// recognize manually typed `@file:/path` tokens.
#[allow(dead_code)]
fn format_inline_file_token_legacy(path: &str) -> String {
    format_inline_file_token(path)
}

/// Returns `true` when the provider-backed mention kind has real data
/// available (slot or env var), as opposed to only the static fallback.
fn provider_backed_mention_available(
    kind: crate::ai::context_contract::ContextAttachmentKind,
) -> bool {
    kind.provider_data_available()
}

/// Resolve a built-in mention token, gating provider-backed kinds on data
/// availability so that manual typing cannot bypass the picker's provider
/// check.
fn resolve_builtin_mention_token(trimmed: &str) -> Option<AiContextPart> {
    let kind = crate::ai::context_contract::ContextAttachmentKind::from_mention_line(trimmed)?;
    if !provider_backed_mention_available(kind) {
        tracing::info!(
            target: "ai",
            event = "inline_context_token_skipped_provider_unavailable",
            token = %trimmed,
            kind = ?kind,
        );
        return None;
    }
    Some(kind.part())
}

fn parse_context_mention_line(line: &str) -> Option<AiContextPart> {
    let trimmed = line.trim();
    resolve_builtin_mention_token(trimmed).or_else(|| parse_file_mention(trimmed))
}

fn parse_file_mention(trimmed: &str) -> Option<AiContextPart> {
    let raw_path = trimmed
        .strip_prefix("@file:")
        .or_else(|| trimmed.strip_prefix("@file "))
        .or_else(|| trimmed.strip_prefix("@file\t"))?
        .trim();

    if raw_path.is_empty() {
        return None;
    }

    let path = unescape_quoted_path(raw_path);

    if path.is_empty() {
        return None;
    }

    let label = Path::new(&path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| path.clone());

    Some(AiContextPart::FilePath { path, label })
}

mod sync;
pub(crate) use sync::{
    build_inline_mention_sync_plan, build_inline_mention_sync_plan_with_aliases,
    caret_after_replacement, remove_inline_mention_at_cursor, replace_text_in_char_range,
    should_claim_inline_mention_ownership, visible_context_chip_indices, InlineMentionSyncPlan,
};

#[cfg(test)]
mod tests;
