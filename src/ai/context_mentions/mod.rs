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
pub(crate) struct InlineContextMention {
    /// Character-level range of the token in the source text.
    pub(crate) range: std::ops::Range<usize>,
    /// The raw token text (e.g. `@browser`, `@file:/tmp/demo.rs`).
    pub(crate) token: String,
    /// The canonical token for this mention, used for ownership tracking and
    /// highlight matching. Aliases resolve to the primary mention token.
    pub(crate) canonical_token: String,
    /// The resolved context part for this mention.
    pub(crate) part: AiContextPart,
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

/// Scan `text` for inline `@mention` tokens and resolve each to an
/// `AiContextPart`. Supports built-in mentions (`@browser`, `@git-status`,
/// etc.) and file mentions (`@file:/path`).
pub(crate) fn parse_inline_context_mentions(text: &str) -> Vec<InlineContextMention> {
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
        i += 1; // skip '@'
        while i < chars.len() && !chars[i].is_whitespace() {
            i += 1;
        }
        let raw_token: String = chars[start..i].iter().collect();
        let trimmed = trim_inline_token_trailing_punctuation(&raw_token);
        let trimmed_char_len = trimmed.chars().count();
        let end = start + trimmed_char_len;

        let part = if let Some(kind) =
            crate::ai::context_contract::ContextAttachmentKind::from_mention_line(trimmed)
        {
            Some(kind.part())
        } else {
            parse_file_mention(trimmed)
        };

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

/// Convert an `AiContextPart` back to its canonical inline `@token` form.
///
/// Returns `None` for parts that have no inline mention representation
/// (e.g. `FocusedTarget`, `AmbientContext`).
pub(crate) fn part_to_inline_token(part: &AiContextPart) -> Option<String> {
    match part {
        AiContextPart::ResourceUri { uri, .. } => {
            crate::ai::context_contract::context_attachment_specs()
                .iter()
                .find(|spec| spec.uri == uri.as_str())
                .and_then(|spec| spec.mention)
                .map(ToString::to_string)
        }
        AiContextPart::FilePath { path, .. } => Some(format!("@file:{path}")),
        _ => None,
    }
}

fn parse_context_mention_line(line: &str) -> Option<AiContextPart> {
    let trimmed = line.trim();

    if let Some(kind) =
        crate::ai::context_contract::ContextAttachmentKind::from_mention_line(trimmed)
    {
        return Some(kind.part());
    }

    parse_file_mention(trimmed)
}

fn parse_file_mention(trimmed: &str) -> Option<AiContextPart> {
    let path = trimmed
        .strip_prefix("@file:")
        .or_else(|| trimmed.strip_prefix("@file "))
        .or_else(|| trimmed.strip_prefix("@file\t"))?
        .trim();

    if path.is_empty() {
        return None;
    }

    let label = Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string());

    Some(AiContextPart::FilePath {
        path: path.to_string(),
        label,
    })
}

#[cfg(test)]
mod tests;
