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

    /// Extract title from content (first non-empty line, stripped of markdown)
    fn extract_title(content: &str) -> String {
        content
            .lines()
            .find(|line| !line.trim().is_empty())
            .map(|line| {
                // Strip markdown heading markers
                let trimmed = line.trim();
                if trimmed.starts_with('#') {
                    trimmed.trim_start_matches('#').trim().to_string()
                } else {
                    trimmed.to_string()
                }
            })
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
