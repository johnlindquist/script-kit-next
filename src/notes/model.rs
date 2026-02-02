//! Notes Data Model
//!
//! Core data structures for the Notes feature.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a note
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NoteId(pub Uuid);

impl NoteId {
    /// Create a new random NoteId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a NoteId from a UUID string
    pub fn parse(s: &str) -> Option<Self> {
        Uuid::parse_str(s).ok().map(Self)
    }

    /// Get the UUID as a string
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

impl Default for NoteId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for NoteId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A single note
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    /// Unique identifier
    pub id: NoteId,

    /// Note title (first line or user-defined)
    pub title: String,

    /// Full markdown content
    pub content: String,

    /// When the note was created
    pub created_at: DateTime<Utc>,

    /// When the note was last modified
    pub updated_at: DateTime<Utc>,

    /// When the note was soft-deleted (None = not deleted)
    pub deleted_at: Option<DateTime<Utc>>,

    /// Whether the note is pinned to the top
    pub is_pinned: bool,

    /// Sort order within pinned/unpinned groups
    pub sort_order: i32,
}

impl Note {
    /// Create a new empty note
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: NoteId::new(),
            title: String::new(),
            content: String::new(),
            created_at: now,
            updated_at: now,
            deleted_at: None,
            is_pinned: false,
            sort_order: 0,
        }
    }

    /// Create a note with initial content
    pub fn with_content(content: impl Into<String>) -> Self {
        let content = content.into();
        let title = Self::extract_title(&content);
        let now = Utc::now();

        Self {
            id: NoteId::new(),
            title,
            content,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            is_pinned: false,
            sort_order: 0,
        }
    }

    /// Update the content and refresh title/timestamp
    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
        self.title = Self::extract_title(&self.content);
        self.updated_at = Utc::now();
    }

    /// Strip common markdown syntax from text for display purposes.
    ///
    /// Removes:
    /// - **bold** and __bold__ -> bold
    /// - *italic* and _italic_ -> italic
    /// - `code` -> code
    /// - # headers -> headers (leading # markers)
    /// - [links](url) -> links
    /// - ~~strikethrough~~ -> strikethrough
    fn strip_markdown_syntax(text: &str) -> String {
        let mut result = text.to_string();

        // Strip markdown heading markers (# at start of line)
        if result.starts_with('#') {
            result = result.trim_start_matches('#').trim_start().to_string();
        }

        // Strip bold: **text** or __text__
        while let Some(start) = result.find("**") {
            if let Some(end) = result[start + 2..].find("**") {
                let before = &result[..start];
                let inner = &result[start + 2..start + 2 + end];
                let after = &result[start + 2 + end + 2..];
                result = format!("{}{}{}", before, inner, after);
            } else {
                break;
            }
        }
        while let Some(start) = result.find("__") {
            if let Some(end) = result[start + 2..].find("__") {
                let before = &result[..start];
                let inner = &result[start + 2..start + 2 + end];
                let after = &result[start + 2 + end + 2..];
                result = format!("{}{}{}", before, inner, after);
            } else {
                break;
            }
        }

        // Strip strikethrough: ~~text~~
        while let Some(start) = result.find("~~") {
            if let Some(end) = result[start + 2..].find("~~") {
                let before = &result[..start];
                let inner = &result[start + 2..start + 2 + end];
                let after = &result[start + 2 + end + 2..];
                result = format!("{}{}{}", before, inner, after);
            } else {
                break;
            }
        }

        // Strip inline code: `text`
        while let Some(start) = result.find('`') {
            if let Some(end) = result[start + 1..].find('`') {
                let before = &result[..start];
                let inner = &result[start + 1..start + 1 + end];
                let after = &result[start + 1 + end + 1..];
                result = format!("{}{}{}", before, inner, after);
            } else {
                break;
            }
        }

        // Strip links: [text](url) -> text
        // Find [...](...)
        while let Some(bracket_start) = result.find('[') {
            if let Some(bracket_end) = result[bracket_start..].find(']') {
                let absolute_bracket_end = bracket_start + bracket_end;
                // Check if followed by (url)
                if result.len() > absolute_bracket_end + 1
                    && result.chars().nth(absolute_bracket_end + 1) == Some('(')
                {
                    if let Some(paren_end) = result[absolute_bracket_end + 1..].find(')') {
                        let before = &result[..bracket_start];
                        let link_text = &result[bracket_start + 1..absolute_bracket_end];
                        let after = &result[absolute_bracket_end + 1 + paren_end + 1..];
                        result = format!("{}{}{}", before, link_text, after);
                        continue;
                    }
                }
            }
            // If we couldn't process a link, break to avoid infinite loop
            break;
        }

        // Strip italic: *text* or _text_ (after bold is removed to avoid conflicts)
        // Handle *italic*
        while let Some(start) = result.find('*') {
            if let Some(end) = result[start + 1..].find('*') {
                let before = &result[..start];
                let inner = &result[start + 1..start + 1 + end];
                let after = &result[start + 1 + end + 1..];
                result = format!("{}{}{}", before, inner, after);
            } else {
                break;
            }
        }
        // Handle _italic_ (but not in the middle of words like snake_case)
        // Only strip if underscore is at word boundary
        while let Some(start) = result.find('_') {
            // Check if this is a word boundary underscore
            let at_start = start == 0
                || !result
                    .chars()
                    .nth(start - 1)
                    .unwrap_or(' ')
                    .is_alphanumeric();
            if at_start {
                if let Some(end) = result[start + 1..].find('_') {
                    let before = &result[..start];
                    let inner = &result[start + 1..start + 1 + end];
                    let after = &result[start + 1 + end + 1..];
                    result = format!("{}{}{}", before, inner, after);
                    continue;
                }
            }
            break;
        }

        result.trim().to_string()
    }

    /// Extract title from content (first non-empty line, stripped of markdown)
    fn extract_title(content: &str) -> String {
        content
            .lines()
            .find(|line| !line.trim().is_empty())
            .map(|line| Self::strip_markdown_syntax(line.trim()))
            .unwrap_or_else(|| "Untitled Note".to_string())
    }

    /// Check if this note is in the trash
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    /// Soft delete the note
    pub fn soft_delete(&mut self) {
        self.deleted_at = Some(Utc::now());
    }

    /// Restore the note from trash
    pub fn restore(&mut self) {
        self.deleted_at = None;
    }

    /// Get a preview of the content (first ~100 chars, excluding title line)
    pub fn preview(&self) -> String {
        self.content
            .lines()
            .skip(1) // Skip title line
            .filter(|line| !line.trim().is_empty())
            .take(3)
            .collect::<Vec<_>>()
            .join(" ")
            .chars()
            .take(100)
            .collect()
    }

    /// Get word count
    pub fn word_count(&self) -> usize {
        self.content.split_whitespace().count()
    }

    /// Get character count
    pub fn char_count(&self) -> usize {
        self.content.chars().count()
    }
}

impl Default for Note {
    fn default() -> Self {
        Self::new()
    }
}

/// Export format for notes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// Plain text (.txt)
    PlainText,
    /// Markdown (.md)
    Markdown,
    /// HTML (.html)
    Html,
}

impl ExportFormat {
    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::PlainText => "txt",
            ExportFormat::Markdown => "md",
            ExportFormat::Html => "html",
        }
    }

    /// Get the MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            ExportFormat::PlainText => "text/plain",
            ExportFormat::Markdown => "text/markdown",
            ExportFormat::Html => "text/html",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_creation() {
        let note = Note::new();
        assert!(!note.id.0.is_nil());
        assert!(note.title.is_empty());
        assert!(note.content.is_empty());
        assert!(!note.is_deleted());
    }

    #[test]
    fn test_note_with_content() {
        let note = Note::with_content("# My Title\n\nSome content here.");
        assert_eq!(note.title, "My Title");
        assert!(!note.content.is_empty());
    }

    #[test]
    fn test_title_extraction() {
        let mut note = Note::new();

        note.set_content("First line as title");
        assert_eq!(note.title, "First line as title");

        note.set_content("# Heading Title\nBody");
        assert_eq!(note.title, "Heading Title");

        note.set_content("## Second Level\nBody");
        assert_eq!(note.title, "Second Level");

        note.set_content("\n\n  Spaced Title  \n");
        assert_eq!(note.title, "Spaced Title");

        note.set_content("");
        assert_eq!(note.title, "Untitled Note");
    }

    #[test]
    fn test_markdown_stripping_bold() {
        let mut note = Note::new();

        note.set_content("**bold text**");
        assert_eq!(note.title, "bold text");

        note.set_content("__also bold__");
        assert_eq!(note.title, "also bold");

        note.set_content("Some **bold** words");
        assert_eq!(note.title, "Some bold words");
    }

    #[test]
    fn test_markdown_stripping_italic() {
        let mut note = Note::new();

        note.set_content("*italic text*");
        assert_eq!(note.title, "italic text");

        note.set_content("_also italic_");
        assert_eq!(note.title, "also italic");

        note.set_content("Some *italic* words");
        assert_eq!(note.title, "Some italic words");
    }

    #[test]
    fn test_markdown_stripping_code() {
        let mut note = Note::new();

        note.set_content("`code block`");
        assert_eq!(note.title, "code block");

        note.set_content("Some `inline code` here");
        assert_eq!(note.title, "Some inline code here");
    }

    #[test]
    fn test_markdown_stripping_links() {
        let mut note = Note::new();

        note.set_content("[link text](https://example.com)");
        assert_eq!(note.title, "link text");

        note.set_content("Check out [this link](url) here");
        assert_eq!(note.title, "Check out this link here");
    }

    #[test]
    fn test_markdown_stripping_headers() {
        let mut note = Note::new();

        note.set_content("# Header");
        assert_eq!(note.title, "Header");

        note.set_content("## Second Level");
        assert_eq!(note.title, "Second Level");

        note.set_content("### Third Level");
        assert_eq!(note.title, "Third Level");
    }

    #[test]
    fn test_markdown_stripping_strikethrough() {
        let mut note = Note::new();

        note.set_content("~~strikethrough~~");
        assert_eq!(note.title, "strikethrough");

        note.set_content("Some ~~deleted~~ text");
        assert_eq!(note.title, "Some deleted text");
    }

    #[test]
    fn test_markdown_stripping_combined() {
        let mut note = Note::new();

        note.set_content("# **Bold Header**");
        assert_eq!(note.title, "Bold Header");

        note.set_content("# [Link Title](url)");
        assert_eq!(note.title, "Link Title");
    }

    #[test]
    fn test_soft_delete_and_restore() {
        let mut note = Note::new();
        assert!(!note.is_deleted());

        note.soft_delete();
        assert!(note.is_deleted());

        note.restore();
        assert!(!note.is_deleted());
    }

    #[test]
    fn test_word_count() {
        let note = Note::with_content("Hello world, this is a test.");
        assert_eq!(note.word_count(), 6);
    }
}
