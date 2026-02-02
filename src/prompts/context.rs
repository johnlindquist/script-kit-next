//! Context mention system for AI chat
//!
//! Provides @ mention support for referencing:
//! - @clipboard - Current clipboard contents
//! - @selection - Current text selection
//! - @file:path - Contents of a specific file
//! - @terminal - Recent terminal output
//!
//! Based on patterns from Raycast, Cursor, and GitHub Copilot.

use std::path::PathBuf;

/// Types of context that can be mentioned
#[derive(Clone, Debug, PartialEq)]
pub enum ContextMentionType {
    /// Current clipboard contents
    Clipboard,
    /// Current text selection (from frontmost app)
    Selection,
    /// Contents of a specific file
    File(PathBuf),
    /// Recent terminal output
    Terminal,
}

/// A parsed context mention from user input
#[derive(Clone, Debug)]
pub struct ContextMention {
    /// The type of context being referenced
    pub kind: ContextMentionType,
    /// The raw text of the mention (e.g., "@clipboard")
    pub raw: String,
    /// Start position in the input string
    pub start: usize,
    /// End position in the input string
    pub end: usize,
}

/// Available context options for autocomplete
#[derive(Clone, Debug)]
pub struct ContextOption {
    /// Display label (e.g., "clipboard")
    pub label: String,
    /// Description shown in autocomplete
    pub description: String,
    /// Icon name (optional)
    pub icon: Option<String>,
    /// The mention type this option creates
    pub kind: ContextMentionType,
}

impl ContextOption {
    /// Get all available context options
    pub fn all() -> Vec<Self> {
        vec![
            ContextOption {
                label: "clipboard".to_string(),
                description: "Current clipboard contents".to_string(),
                icon: Some("clipboard".to_string()),
                kind: ContextMentionType::Clipboard,
            },
            ContextOption {
                label: "selection".to_string(),
                description: "Selected text from active app".to_string(),
                icon: Some("text-select".to_string()),
                kind: ContextMentionType::Selection,
            },
            ContextOption {
                label: "file".to_string(),
                description: "Contents of a file (type path after)".to_string(),
                icon: Some("file".to_string()),
                kind: ContextMentionType::File(PathBuf::new()),
            },
            ContextOption {
                label: "terminal".to_string(),
                description: "Recent terminal output".to_string(),
                icon: Some("terminal".to_string()),
                kind: ContextMentionType::Terminal,
            },
        ]
    }

    /// Filter options by prefix
    pub fn filter_by_prefix(prefix: &str) -> Vec<Self> {
        let prefix_lower = prefix.to_lowercase();
        Self::all()
            .into_iter()
            .filter(|opt| opt.label.to_lowercase().starts_with(&prefix_lower))
            .collect()
    }
}

/// Parse context mentions from input text
pub fn parse_mentions(input: &str) -> Vec<ContextMention> {
    let mut mentions = Vec::new();
    let mut chars = input.char_indices().peekable();

    while let Some((i, c)) = chars.next() {
        if c == '@' {
            let start = i;
            let mut mention_text = String::new();

            // Collect the mention identifier
            while let Some(&(_, next_c)) = chars.peek() {
                if next_c.is_alphanumeric() || next_c == '_' || next_c == '-' || next_c == ':' {
                    mention_text.push(chars.next().unwrap().1);
                } else {
                    break;
                }
            }

            // Also collect file path if present (after @file:)
            if mention_text.starts_with("file:") {
                while let Some(&(_, next_c)) = chars.peek() {
                    if !next_c.is_whitespace() {
                        mention_text.push(chars.next().unwrap().1);
                    } else {
                        break;
                    }
                }
            }

            if !mention_text.is_empty() {
                let end = start + 1 + mention_text.len();
                let raw = format!("@{}", mention_text);

                let kind = match mention_text.as_str() {
                    "clipboard" => Some(ContextMentionType::Clipboard),
                    "selection" => Some(ContextMentionType::Selection),
                    "terminal" => Some(ContextMentionType::Terminal),
                    _ => mention_text
                        .strip_prefix("file:")
                        .map(|path| ContextMentionType::File(PathBuf::from(path))),
                };

                if let Some(kind) = kind {
                    mentions.push(ContextMention {
                        kind,
                        raw,
                        start,
                        end,
                    });
                }
            }
        }
    }

    mentions
}

/// Check if input has an incomplete @ mention (for showing autocomplete)
pub fn get_incomplete_mention(input: &str, cursor_pos: usize) -> Option<(usize, String)> {
    // Look backwards from cursor for @
    let before_cursor = &input[..cursor_pos.min(input.len())];
    if let Some(at_pos) = before_cursor.rfind('@') {
        let after_at = &before_cursor[at_pos + 1..];
        // Check if it's a valid partial mention (alphanumeric only, no spaces)
        if after_at
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            // Check it's not already a complete mention followed by space
            let full_mention = &input[at_pos..];
            let mention_end = full_mention
                .find(|c: char| c.is_whitespace())
                .unwrap_or(full_mention.len());
            let mention_text = &full_mention[1..mention_end];

            // Only show autocomplete if not a complete known mention
            let known_mentions = ["clipboard", "selection", "terminal"];
            if !known_mentions.contains(&mention_text) && !mention_text.starts_with("file:") {
                return Some((at_pos, after_at.to_string()));
            }
        }
    }
    None
}

/// Resolve context mention to actual content
pub fn resolve_mention(mention: &ContextMention) -> Option<String> {
    match &mention.kind {
        ContextMentionType::Clipboard => {
            // Note: Clipboard access requires platform-specific implementation
            // This will be called from the GPUI context where clipboard is available
            None // Caller should handle clipboard access
        }
        ContextMentionType::Selection => {
            // Note: Selection access requires platform-specific implementation
            None // Caller should handle selection access
        }
        ContextMentionType::File(path) => {
            // Read file contents
            std::fs::read_to_string(path).ok()
        }
        ContextMentionType::Terminal => {
            // Note: Terminal output requires integration with terminal state
            None // Caller should handle terminal access
        }
    }
}

/// Expand mentions in input text with resolved content
pub fn expand_mentions(input: &str, clipboard: Option<&str>, selection: Option<&str>) -> String {
    let mentions = parse_mentions(input);
    if mentions.is_empty() {
        return input.to_string();
    }

    let mut result = input.to_string();
    // Process in reverse order to maintain correct positions
    for mention in mentions.into_iter().rev() {
        let replacement = match &mention.kind {
            ContextMentionType::Clipboard => clipboard.map(|s| s.to_string()),
            ContextMentionType::Selection => selection.map(|s| s.to_string()),
            ContextMentionType::File(path) => std::fs::read_to_string(path).ok(),
            ContextMentionType::Terminal => None, // Not implemented yet
        };

        if let Some(content) = replacement {
            // Replace the mention with the content, wrapped in a block
            let wrapped = format!("\n```\n{}\n```\n", content.trim());
            result.replace_range(mention.start..mention.end, &wrapped);
        }
    }

    result
}

/// Expand context mentions using GPUI context for clipboard access
/// This is a convenience wrapper around expand_mentions that handles clipboard access
pub fn expand_context<V: 'static>(input: &str, cx: &mut gpui::Context<V>) -> String {
    // Get clipboard content if available
    let clipboard_content = cx
        .read_from_clipboard()
        .and_then(|item| item.text().map(|s| s.to_string()));

    // Selection is not easily available - would need OS integration
    let selection: Option<&str> = None;

    expand_mentions(input, clipboard_content.as_deref(), selection)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_clipboard_mention() {
        let mentions = parse_mentions("Check this @clipboard content");
        assert_eq!(mentions.len(), 1);
        assert_eq!(mentions[0].kind, ContextMentionType::Clipboard);
        assert_eq!(mentions[0].raw, "@clipboard");
    }

    #[test]
    fn test_parse_file_mention() {
        let mentions = parse_mentions("Look at @file:src/main.rs for details");
        assert_eq!(mentions.len(), 1);
        if let ContextMentionType::File(path) = &mentions[0].kind {
            assert_eq!(path.to_str().unwrap(), "src/main.rs");
        } else {
            panic!("Expected file mention");
        }
    }

    #[test]
    fn test_parse_multiple_mentions() {
        let mentions = parse_mentions("Compare @clipboard with @selection");
        assert_eq!(mentions.len(), 2);
        assert_eq!(mentions[0].kind, ContextMentionType::Clipboard);
        assert_eq!(mentions[1].kind, ContextMentionType::Selection);
    }

    #[test]
    fn test_incomplete_mention() {
        let result = get_incomplete_mention("Check @clip", 11);
        assert!(result.is_some());
        let (pos, prefix) = result.unwrap();
        assert_eq!(pos, 6);
        assert_eq!(prefix, "clip");
    }

    #[test]
    fn test_expand_clipboard() {
        let input = "Fix this: @clipboard";
        let result = expand_mentions(input, Some("broken code"), None);
        assert!(result.contains("```"));
        assert!(result.contains("broken code"));
    }
}
