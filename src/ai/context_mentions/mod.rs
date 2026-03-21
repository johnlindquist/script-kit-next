use crate::ai::message_parts::AiContextPart;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedContextMentions {
    pub(crate) cleaned_content: String,
    pub(crate) parts: Vec<AiContextPart>,
}

impl ParsedContextMentions {
    pub(crate) fn has_parts(&self) -> bool {
        !self.parts.is_empty()
    }
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
        .strip_prefix("@file ")
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
