🧩 Packing 6 file(s)...
📝 Files selected:
  • src/notes/model.rs
  • src/notes/storage.rs
  • src/notes/mod.rs
  • src/notes/window.rs
  • src/notes/actions_panel.rs
  • src/notes/browse_panel.rs
This file is a merged representation of the filtered codebase, combined into a single document by packx.

<file_summary>
This section contains a summary of this file.

<purpose>
This file contains a packed representation of filtered repository contents.
It is designed to be easily consumable by AI systems for analysis, code review,
or other automated processes.
</purpose>

<usage_guidelines>
- Treat this file as a snapshot of the repository's state
- Be aware that this file may contain sensitive information
</usage_guidelines>

<notes>
- Files were filtered by packx based on content and extension matching
- Total files included: 6
</notes>
</file_summary>

<directory_structure>
src/notes/model.rs
src/notes/storage.rs
src/notes/mod.rs
src/notes/window.rs
src/notes/actions_panel.rs
src/notes/browse_panel.rs
</directory_structure>

<files>
This section contains the contents of the repository's files.

<file path="src/notes/model.rs">
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

</file>

<file path="src/notes/storage.rs">
//! Notes Storage Layer
//!
//! SQLite-backed persistence for notes with CRUD operations.
//! Follows the same patterns as clipboard_history.rs for consistency.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use tracing::{debug, info};

use super::model::{Note, NoteId};

/// Global database connection for notes
static NOTES_DB: OnceLock<Arc<Mutex<Connection>>> = OnceLock::new();

/// Get the path to the notes database
fn get_notes_db_path() -> PathBuf {
    let kit_dir = dirs::home_dir()
        .map(|h| h.join(".sk/kit"))
        .unwrap_or_else(|| PathBuf::from(".sk/kit"));

    kit_dir.join("db").join("notes.sqlite")
}

/// Initialize the notes database
///
/// This function is idempotent - it's safe to call multiple times.
/// If the database is already initialized, it returns Ok(()) immediately.
pub fn init_notes_db() -> Result<()> {
    // Check if already initialized - this is the common case after first init
    if NOTES_DB.get().is_some() {
        debug!("Notes database already initialized, skipping");
        return Ok(());
    }

    let db_path = get_notes_db_path();

    // Ensure directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create notes db directory")?;
    }

    let conn = Connection::open(&db_path).context("Failed to open notes database")?;

    // Create tables
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS notes (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL DEFAULT '',
            content TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT,
            is_pinned INTEGER NOT NULL DEFAULT 0,
            sort_order INTEGER NOT NULL DEFAULT 0
        );

        CREATE INDEX IF NOT EXISTS idx_notes_updated_at ON notes(updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_notes_deleted_at ON notes(deleted_at);
        CREATE INDEX IF NOT EXISTS idx_notes_is_pinned ON notes(is_pinned);

        -- Full-text search support
        CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
            title,
            content,
            content='notes',
            content_rowid='rowid'
        );

        -- Triggers to keep FTS in sync
        CREATE TRIGGER IF NOT EXISTS notes_ai AFTER INSERT ON notes BEGIN
            INSERT INTO notes_fts(rowid, title, content) 
            VALUES (NEW.rowid, NEW.title, NEW.content);
        END;

        CREATE TRIGGER IF NOT EXISTS notes_ad AFTER DELETE ON notes BEGIN
            INSERT INTO notes_fts(notes_fts, rowid, title, content) 
            VALUES('delete', OLD.rowid, OLD.title, OLD.content);
        END;

        CREATE TRIGGER IF NOT EXISTS notes_au AFTER UPDATE ON notes BEGIN
            INSERT INTO notes_fts(notes_fts, rowid, title, content) 
            VALUES('delete', OLD.rowid, OLD.title, OLD.content);
            INSERT INTO notes_fts(rowid, title, content) 
            VALUES (NEW.rowid, NEW.title, NEW.content);
        END;
        "#,
    )
    .context("Failed to create notes tables")?;

    info!(db_path = %db_path.display(), "Notes database initialized");

    // Use get_or_init pattern to handle race condition where another thread
    // might have initialized the DB between our check and set
    let _ = NOTES_DB.get_or_init(|| Arc::new(Mutex::new(conn)));

    Ok(())
}

/// Get a reference to the notes database connection
fn get_db() -> Result<Arc<Mutex<Connection>>> {
    NOTES_DB
        .get()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Notes database not initialized"))
}

/// Save a note (insert or update)
pub fn save_note(note: &Note) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    conn.execute(
        r#"
        INSERT INTO notes (id, title, content, created_at, updated_at, deleted_at, is_pinned, sort_order)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ON CONFLICT(id) DO UPDATE SET
            title = excluded.title,
            content = excluded.content,
            updated_at = excluded.updated_at,
            deleted_at = excluded.deleted_at,
            is_pinned = excluded.is_pinned,
            sort_order = excluded.sort_order
        "#,
        params![
            note.id.as_str(),
            note.title,
            note.content,
            note.created_at.to_rfc3339(),
            note.updated_at.to_rfc3339(),
            note.deleted_at.map(|dt| dt.to_rfc3339()),
            note.is_pinned as i32,
            note.sort_order,
        ],
    )
    .context("Failed to save note")?;

    debug!(note_id = %note.id, title = %note.title, "Note saved");
    Ok(())
}

/// Get a note by ID
pub fn get_note(id: NoteId) -> Result<Option<Note>> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, title, content, created_at, updated_at, deleted_at, is_pinned, sort_order
            FROM notes
            WHERE id = ?1
            "#,
        )
        .context("Failed to prepare get_note query")?;

    let result = stmt
        .query_row(params![id.as_str()], row_to_note)
        .optional()
        .context("Failed to get note")?;

    Ok(result)
}

/// Get all active notes (not deleted), sorted by pinned first then updated_at desc
pub fn get_all_notes() -> Result<Vec<Note>> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, title, content, created_at, updated_at, deleted_at, is_pinned, sort_order
            FROM notes
            WHERE deleted_at IS NULL
            ORDER BY is_pinned DESC, updated_at DESC
            "#,
        )
        .context("Failed to prepare get_all_notes query")?;

    let notes = stmt
        .query_map([], row_to_note)
        .context("Failed to query notes")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect notes")?;

    debug!(count = notes.len(), "Retrieved all notes");
    Ok(notes)
}

/// Get notes in trash (soft-deleted)
pub fn get_deleted_notes() -> Result<Vec<Note>> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, title, content, created_at, updated_at, deleted_at, is_pinned, sort_order
            FROM notes
            WHERE deleted_at IS NOT NULL
            ORDER BY deleted_at DESC
            "#,
        )
        .context("Failed to prepare get_deleted_notes query")?;

    let notes = stmt
        .query_map([], row_to_note)
        .context("Failed to query deleted notes")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect deleted notes")?;

    debug!(count = notes.len(), "Retrieved deleted notes");
    Ok(notes)
}

/// Search notes using full-text search
pub fn search_notes(query: &str) -> Result<Vec<Note>> {
    if query.trim().is_empty() {
        return get_all_notes();
    }

    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    // FTS5 search with highlight
    let mut stmt = conn
        .prepare(
            r#"
            SELECT n.id, n.title, n.content, n.created_at, n.updated_at, 
                   n.deleted_at, n.is_pinned, n.sort_order
            FROM notes n
            INNER JOIN notes_fts fts ON n.rowid = fts.rowid
            WHERE notes_fts MATCH ?1 AND n.deleted_at IS NULL
            ORDER BY rank
            "#,
        )
        .context("Failed to prepare search query")?;

    let notes = stmt
        .query_map(params![query], row_to_note)
        .context("Failed to search notes")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect search results")?;

    debug!(query = %query, count = notes.len(), "Search completed");
    Ok(notes)
}

/// Permanently delete a note
pub fn delete_note_permanently(id: NoteId) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    conn.execute("DELETE FROM notes WHERE id = ?1", params![id.as_str()])
        .context("Failed to delete note")?;

    info!(note_id = %id, "Note permanently deleted");
    Ok(())
}

/// Prune notes deleted more than `days` ago
pub fn prune_old_deleted_notes(days: u32) -> Result<usize> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let cutoff = Utc::now() - chrono::Duration::days(days as i64);

    let count = conn
        .execute(
            "DELETE FROM notes WHERE deleted_at IS NOT NULL AND deleted_at < ?1",
            params![cutoff.to_rfc3339()],
        )
        .context("Failed to prune old deleted notes")?;

    if count > 0 {
        info!(count, days, "Pruned old deleted notes");
    }

    Ok(count)
}

/// Get total note count (active only)
pub fn get_note_count() -> Result<usize> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM notes WHERE deleted_at IS NULL",
            [],
            |row| row.get(0),
        )
        .context("Failed to count notes")?;

    Ok(count as usize)
}

/// Convert a database row to a Note
fn row_to_note(row: &rusqlite::Row) -> rusqlite::Result<Note> {
    let id_str: String = row.get(0)?;
    let title: String = row.get(1)?;
    let content: String = row.get(2)?;
    let created_at_str: String = row.get(3)?;
    let updated_at_str: String = row.get(4)?;
    let deleted_at_str: Option<String> = row.get(5)?;
    let is_pinned: i32 = row.get(6)?;
    let sort_order: i32 = row.get(7)?;

    let id = NoteId::parse(&id_str).unwrap_or_default();

    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let deleted_at = deleted_at_str.and_then(|s| {
        DateTime::parse_from_rfc3339(&s)
            .map(|dt| dt.with_timezone(&Utc))
            .ok()
    });

    Ok(Note {
        id,
        title,
        content,
        created_at,
        updated_at,
        deleted_at,
        is_pinned: is_pinned != 0,
        sort_order,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_path() {
        let path = get_notes_db_path();
        assert!(path.to_string_lossy().contains("notes.sqlite"));
    }
}

</file>

<file path="src/notes/mod.rs">
//! Notes Module - Raycast Notes Feature Parity
//!
//! A separate floating notes window built with gpui-component library.
//!
//! ## Features
//! - Floating notes window with global hotkey access
//! - Markdown/rich text editing with live preview
//! - Multiple notes management with sidebar list
//! - Quick capture from anywhere
//! - Auto-sizing window that grows with content
//! - Persistent storage (local SQLite)
//! - Formatting toolbar with keyboard shortcuts
//! - Search across all notes
//! - Export (plain text, markdown, HTML)
//! - Menu bar integration
//! - Recently deleted notes (soft delete with recovery)
//!
//! ## Architecture
//! The Notes feature runs in a completely separate window from the main Script Kit
//! launcher. It uses gpui-component for UI components (Input, Sidebar, Button, etc.)
//! and follows the Root wrapper pattern required by gpui-component.
//!

// Allow dead code in this module - many functions are designed for future use
#![allow(dead_code)]

mod actions_panel;
mod browse_panel;
mod model;
mod storage;
mod window;

// Re-export actions panel types for use by window.rs
#[allow(unused_imports)]
pub use actions_panel::{NotesAction, NotesActionCallback, NotesActionItem, NotesActionsPanel};

// Re-export browse panel types for use by window.rs
#[allow(unused_imports)]
pub use browse_panel::{BrowsePanel, NoteAction, NoteListItem};

// Re-export key types - suppress unused warnings since these are public API
#[allow(unused_imports)]
pub use model::*;
#[allow(unused_imports)]
pub use storage::*;
#[allow(unused_imports)]
pub use window::{
    close_notes_window, is_notes_window_open, open_notes_window, quick_capture, NotesApp,
};

</file>

<file path="src/notes/window.rs">
//! Notes Window
//!
//! A separate floating window for notes, built with gpui-component.
//! This is completely independent from the main Script Kit launcher window.

use anyhow::Result;
use gpui::{
    div, hsla, point, prelude::*, px, rgb, size, App, BoxShadow, Context, Entity, FocusHandle,
    Focusable, Hsla, IntoElement, KeyDownEvent, ParentElement, Render, Styled, Subscription,
    Window, WindowBounds, WindowOptions,
};

#[cfg(target_os = "macos")]
use cocoa::appkit::NSApp;
#[cfg(target_os = "macos")]
use cocoa::base::{id, nil};
use gpui_component::{
    button::{Button, ButtonVariants},
    input::{Input, InputEvent, InputState, Search},
    theme::{ActiveTheme, Theme as GpuiTheme, ThemeColor, ThemeMode},
    IconName, Root, Sizable,
};
#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};
use std::sync::{Arc, Mutex};
use tracing::{debug, info};

use super::actions_panel::{
    panel_height_for_rows, NotesAction, NotesActionItem, NotesActionsPanel,
};
use super::browse_panel::{BrowsePanel, NoteAction, NoteListItem};
use super::model::{ExportFormat, Note, NoteId};
use super::storage;
use crate::watcher::ThemeWatcher;

/// Global handle to the notes window
static NOTES_WINDOW: std::sync::OnceLock<std::sync::Mutex<Option<gpui::WindowHandle<Root>>>> =
    std::sync::OnceLock::new();

/// View mode for the notes list
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NotesViewMode {
    /// Show all active notes
    #[default]
    AllNotes,
    /// Show deleted notes (trash)
    Trash,
}

/// The main notes application view
///
/// Raycast-style single-note view:
/// - No sidebar - displays one note at a time
/// - Titlebar with note title and hover-reveal action icons
/// - Auto-resize: window height grows with content
/// - Footer with type indicator and character count
pub struct NotesApp {
    /// All notes (cached from storage)
    notes: Vec<Note>,

    /// Deleted notes (for trash view)
    deleted_notes: Vec<Note>,

    /// Current view mode
    view_mode: NotesViewMode,

    /// Currently selected note ID
    selected_note_id: Option<NoteId>,

    /// Editor input state (using gpui-component's Input)
    editor_state: Entity<InputState>,

    /// Search input state (for future browse panel)
    search_state: Entity<InputState>,

    /// Current search query (for future browse panel)
    search_query: String,

    /// Whether the titlebar is being hovered (for showing/hiding icons)
    titlebar_hovered: bool,

    /// Whether the entire window is being hovered (for traffic lights)
    window_hovered: bool,

    /// Forces hover chrome for visual tests
    force_hovered: bool,

    /// Whether the formatting toolbar is pinned open
    show_format_toolbar: bool,

    /// Last known content line count for auto-resize
    last_line_count: usize,

    /// Initial window height - used as minimum for auto-resize
    initial_height: f32,

    /// Whether auto-sizing is enabled
    /// When enabled: window grows AND shrinks to fit content (min = initial_height)
    /// When disabled: window size is fixed until user re-enables via actions panel
    /// Disabled automatically when user manually resizes the window
    auto_sizing_enabled: bool,

    /// Last known window height - used to detect manual resize
    last_window_height: f32,

    /// Focus handle for keyboard navigation
    focus_handle: FocusHandle,

    /// Subscriptions to keep alive
    _subscriptions: Vec<Subscription>,

    /// Whether the actions panel is shown (Cmd+K)
    show_actions_panel: bool,

    /// Whether the browse panel is shown (Cmd+P)
    show_browse_panel: bool,

    /// Entity for the actions panel (when shown)
    actions_panel: Option<Entity<NotesActionsPanel>>,

    /// Entity for the browse panel (when shown)
    browse_panel: Option<Entity<super::browse_panel::BrowsePanel>>,

    /// Pending action from actions panel clicks
    pending_action: Arc<Mutex<Option<NotesAction>>>,

    /// Previous height before showing the actions panel
    actions_panel_prev_height: Option<f32>,

    /// Cached box shadows from theme (avoid reloading theme on every render)
    cached_box_shadows: Vec<BoxShadow>,

    /// Pending note selection from browse panel
    pending_browse_select: Arc<Mutex<Option<NoteId>>>,

    /// Pending close request from browse panel
    pending_browse_close: Arc<Mutex<bool>>,

    /// Pending action from browse panel (note id + action)
    pending_browse_action: Arc<Mutex<Option<(NoteId, NoteAction)>>>,
}

impl NotesApp {
    /// Create a new NotesApp
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Initialize storage
        if let Err(e) = storage::init_notes_db() {
            tracing::error!(error = %e, "Failed to initialize notes database");
        }

        // Load notes from storage
        let notes = storage::get_all_notes().unwrap_or_default();
        let deleted_notes = storage::get_deleted_notes().unwrap_or_default();
        let selected_note_id = notes.first().map(|n| n.id);

        // Get initial content if we have a selected note
        let initial_content = selected_note_id
            .and_then(|id| notes.iter().find(|n| n.id == id))
            .map(|n| n.content.clone())
            .unwrap_or_default();

        // Calculate initial line count for auto-resize (before moving content)
        let initial_line_count = initial_content.lines().count().max(1);

        // Create input states - use multi_line for the editor
        let editor_state = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
                .searchable(true)
                .rows(20)
                .placeholder("Start typing your note...")
                .default_value(initial_content)
        });

        let search_state = cx.new(|cx| InputState::new(window, cx).placeholder("Search notes..."));

        let focus_handle = cx.focus_handle();

        // Subscribe to editor changes - passes window for auto-resize
        let editor_sub = cx.subscribe_in(&editor_state, window, {
            move |this, _, ev: &InputEvent, window, cx| {
                if matches!(ev, InputEvent::Change) {
                    this.on_editor_change(window, cx);
                }
            }
        });

        // Subscribe to search changes
        let search_sub = cx.subscribe_in(&search_state, window, {
            move |this, _, ev: &InputEvent, _window, cx| {
                if matches!(ev, InputEvent::Change) {
                    this.on_search_change(cx);
                }
            }
        });

        // Get initial window height to use as minimum
        let initial_height: f32 = window.bounds().size.height.into();

        info!(
            note_count = notes.len(),
            initial_height = initial_height,
            "Notes app initialized"
        );

        // Pre-compute box shadows from theme (avoid reloading on every render)
        let cached_box_shadows = Self::compute_box_shadows();

        Self {
            notes,
            deleted_notes,
            view_mode: NotesViewMode::AllNotes,
            selected_note_id,
            editor_state,
            search_state,
            search_query: String::new(),
            titlebar_hovered: false,
            window_hovered: false,
            force_hovered: false,
            show_format_toolbar: false,
            last_line_count: initial_line_count,
            initial_height,
            auto_sizing_enabled: true,          // Auto-sizing ON by default
            last_window_height: initial_height, // Track for manual resize detection
            focus_handle,
            _subscriptions: vec![editor_sub, search_sub],
            show_actions_panel: false,
            show_browse_panel: false,
            actions_panel: None,
            browse_panel: None,
            pending_action: Arc::new(Mutex::new(None)),
            actions_panel_prev_height: None,
            cached_box_shadows,
            pending_browse_select: Arc::new(Mutex::new(None)),
            pending_browse_close: Arc::new(Mutex::new(false)),
            pending_browse_action: Arc::new(Mutex::new(None)),
        }
    }

    /// Handle editor content changes with auto-resize
    fn on_editor_change(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_note_id {
            let content = self.editor_state.read(cx).value();
            let content_string = content.to_string();

            // Update the note in our cache
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == id) {
                let old_title = note.title.clone();
                note.set_content(content_string.clone());
                debug!(
                    note_id = %id,
                    old_title = %old_title,
                    new_title = %note.title,
                    content_preview = %content_string.chars().take(50).collect::<String>(),
                    "Title updated from content"
                );

                // Save to storage (debounced in a real implementation)
                if let Err(e) = storage::save_note(note) {
                    tracing::error!(error = %e, "Failed to save note");
                }
            }

            // Auto-resize: adjust window height based on content
            let new_line_count = content_string.lines().count().max(1);
            if new_line_count != self.last_line_count {
                self.last_line_count = new_line_count;
                self.update_window_height(window, new_line_count, cx);
            }

            cx.notify();
        }
    }

    /// Update window height based on content line count
    /// Raycast-style: window grows AND shrinks to fit content when auto_sizing_enabled
    /// IMPORTANT: Window never shrinks below initial_height (the height at window creation)
    fn update_window_height(
        &mut self,
        window: &mut Window,
        line_count: usize,
        _cx: &mut Context<Self>,
    ) {
        // Skip if auto-sizing is disabled (user manually resized)
        if !self.auto_sizing_enabled {
            return;
        }

        // Constants for layout calculation - adjusted for compact sticky-note style
        const TITLEBAR_HEIGHT: f32 = 32.0;
        const FOOTER_HEIGHT: f32 = 24.0;
        const PADDING: f32 = 24.0; // Top + bottom padding in editor area
        const LINE_HEIGHT: f32 = 20.0; // Approximate line height
        const MAX_HEIGHT: f32 = 600.0; // Don't grow too large

        // Use initial_height as minimum - never shrink below starting size
        let min_height = self.initial_height;

        // Calculate desired height
        let content_height = (line_count as f32) * LINE_HEIGHT;
        let total_height = TITLEBAR_HEIGHT + content_height + FOOTER_HEIGHT + PADDING;
        let clamped_height = total_height.clamp(min_height, MAX_HEIGHT);

        // Get current bounds and update height
        let current_bounds = window.bounds();
        let old_height: f32 = current_bounds.size.height.into();

        // Resize if height needs to change (both grow AND shrink)
        // Use a small threshold to avoid constant tiny adjustments
        const RESIZE_THRESHOLD: f32 = 5.0;
        if (clamped_height - old_height).abs() > RESIZE_THRESHOLD {
            let new_size = size(current_bounds.size.width, px(clamped_height));

            debug!(
                old_height = old_height,
                new_height = clamped_height,
                min_height = min_height,
                line_count = line_count,
                auto_sizing = self.auto_sizing_enabled,
                "Auto-resize: adjusting window height"
            );

            window.resize(new_size);
            self.last_window_height = clamped_height;
        }
    }

    /// Enable auto-sizing (called from actions panel)
    pub fn enable_auto_sizing(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.auto_sizing_enabled = true;
        // Re-calculate and apply the correct height
        let line_count = self.last_line_count;
        self.update_window_height(window, line_count, cx);
        info!("Auto-sizing enabled");
        cx.notify();
    }

    /// Check if user manually resized the window and disable auto-sizing if so
    fn detect_manual_resize(&mut self, window: &Window) {
        if !self.auto_sizing_enabled {
            return; // Already disabled
        }

        let current_height: f32 = window.bounds().size.height.into();

        // If height differs significantly from what we set, user resized manually
        const MANUAL_RESIZE_THRESHOLD: f32 = 10.0;
        if (current_height - self.last_window_height).abs() > MANUAL_RESIZE_THRESHOLD {
            self.auto_sizing_enabled = false;
            self.last_window_height = current_height;
            debug!(
                current_height = current_height,
                last_height = self.last_window_height,
                "Manual resize detected - auto-sizing disabled"
            );
        }
    }

    /// Handle search query changes
    fn on_search_change(&mut self, cx: &mut Context<Self>) {
        let query = self.search_state.read(cx).value().to_string();
        self.search_query = query.clone();

        // If search is not empty, use FTS search
        if !query.trim().is_empty() {
            match storage::search_notes(&query) {
                Ok(results) => {
                    self.notes = results;
                    // Update selection if current note not in results
                    if let Some(id) = self.selected_note_id {
                        if !self.notes.iter().any(|n| n.id == id) {
                            self.selected_note_id = self.notes.first().map(|n| n.id);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "Search failed");
                }
            }
        } else {
            // Reload all notes when search is cleared
            self.notes = storage::get_all_notes().unwrap_or_default();
        }

        cx.notify();
    }

    /// Create a new note
    fn create_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let note = Note::new();
        let id = note.id;

        // Save to storage
        if let Err(e) = storage::save_note(&note) {
            tracing::error!(error = %e, "Failed to create note");
            return;
        }

        // Add to cache and select it
        self.notes.insert(0, note);
        self.select_note(id, window, cx);

        info!(note_id = %id, "New note created");
    }

    /// Select a note for editing
    fn select_note(&mut self, id: NoteId, window: &mut Window, cx: &mut Context<Self>) {
        self.selected_note_id = Some(id);

        // Load content into editor
        let note_list = if self.view_mode == NotesViewMode::Trash {
            &self.deleted_notes
        } else {
            &self.notes
        };

        if let Some(note) = note_list.iter().find(|n| n.id == id) {
            let content_len = note.content.len();
            self.editor_state.update(cx, |state, cx| {
                state.set_value(&note.content, window, cx);
                // Move cursor to end of text (set selection to end..end = no selection, cursor at end)
                state.set_selection(content_len, content_len, window, cx);
            });
        }

        // Focus the editor after selecting a note
        self.editor_state.update(cx, |state, cx| {
            state.focus(window, cx);
        });

        cx.notify();
    }

    /// Delete the currently selected note (soft delete)
    fn delete_selected_note(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_note_id {
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == id) {
                note.soft_delete();

                if let Err(e) = storage::save_note(note) {
                    tracing::error!(error = %e, "Failed to delete note");
                }

                // Move to deleted notes
                self.deleted_notes.insert(0, note.clone());
            }

            // Remove from visible list and select next
            self.notes.retain(|n| n.id != id);
            self.selected_note_id = self.notes.first().map(|n| n.id);

            cx.notify();
        }
    }

    /// Permanently delete the selected note from trash
    fn permanently_delete_note(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_note_id {
            if let Err(e) = storage::delete_note_permanently(id) {
                tracing::error!(error = %e, "Failed to permanently delete note");
                return;
            }

            self.deleted_notes.retain(|n| n.id != id);
            self.selected_note_id = self.deleted_notes.first().map(|n| n.id);

            info!(note_id = %id, "Note permanently deleted");
            cx.notify();
        }
    }

    /// Restore the selected note from trash
    fn restore_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_note_id {
            if let Some(note) = self.deleted_notes.iter_mut().find(|n| n.id == id) {
                note.restore();

                if let Err(e) = storage::save_note(note) {
                    tracing::error!(error = %e, "Failed to restore note");
                    return;
                }

                // Move back to active notes
                self.notes.insert(0, note.clone());
            }

            self.deleted_notes.retain(|n| n.id != id);
            self.view_mode = NotesViewMode::AllNotes;
            self.selected_note_id = Some(id);
            self.select_note(id, window, cx);

            info!(note_id = %id, "Note restored");
            cx.notify();
        }
    }

    /// Switch view mode
    fn set_view_mode(&mut self, mode: NotesViewMode, window: &mut Window, cx: &mut Context<Self>) {
        self.view_mode = mode;

        // Select first note in new view
        let notes = match mode {
            NotesViewMode::AllNotes => &self.notes,
            NotesViewMode::Trash => &self.deleted_notes,
        };

        if let Some(note) = notes.first() {
            self.select_note(note.id, window, cx);
        } else {
            self.selected_note_id = None;
            self.editor_state.update(cx, |state, cx| {
                state.set_value("", window, cx);
            });
        }

        cx.notify();
    }

    /// Export the current note
    fn export_note(&self, format: ExportFormat) {
        if let Some(id) = self.selected_note_id {
            if let Some(note) = self.notes.iter().find(|n| n.id == id) {
                let content = match format {
                    ExportFormat::PlainText => note.content.clone(),
                    ExportFormat::Markdown => {
                        format!("# {}\n\n{}", note.title, note.content)
                    }
                    ExportFormat::Html => {
                        format!(
                            "<!DOCTYPE html>\n<html>\n<head><title>{}</title></head>\n<body>\n<h1>{}</h1>\n<pre>{}</pre>\n</body>\n</html>",
                            note.title, note.title, note.content
                        )
                    }
                };

                // Copy to clipboard
                #[cfg(target_os = "macos")]
                {
                    use std::process::Command;
                    let _ = Command::new("pbcopy")
                        .stdin(std::process::Stdio::piped())
                        .spawn()
                        .and_then(|mut child| {
                            use std::io::Write;
                            if let Some(stdin) = child.stdin.as_mut() {
                                stdin.write_all(content.as_bytes())?;
                            }
                            child.wait()
                        });
                    info!(format = ?format, "Note exported to clipboard");
                }
            }
        }
    }

    /// Insert markdown formatting at cursor position
    fn insert_formatting(&mut self, prefix: &str, suffix: &str, cx: &mut Context<Self>) {
        let current = self.editor_state.read(cx).value().to_string();
        // For simplicity, append to end. A real implementation would insert at cursor.
        let formatted = format!("{}{}{}", current, prefix, suffix);
        // Note: We can't directly update with cursor position, so this is simplified
        info!(prefix = prefix, "Formatting inserted");
        let _ = formatted; // Would update editor in full implementation
        cx.notify();
    }

    /// Get filtered notes based on search query
    fn get_visible_notes(&self) -> &[Note] {
        match self.view_mode {
            NotesViewMode::AllNotes => &self.notes,
            NotesViewMode::Trash => &self.deleted_notes,
        }
    }

    /// Get the character count of the current note
    fn get_character_count(&self, cx: &Context<Self>) -> usize {
        self.editor_state.read(cx).value().chars().count()
    }

    /// Copy the current note content to clipboard
    fn copy_note_to_clipboard(&self, cx: &Context<Self>) {
        let content = self.editor_state.read(cx).value().to_string();
        self.copy_text_to_clipboard(&content);
    }

    fn copy_text_to_clipboard(&self, content: &str) {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            let _ = Command::new("pbcopy")
                .stdin(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    use std::io::Write;
                    if let Some(stdin) = child.stdin.as_mut() {
                        stdin.write_all(content.as_bytes())?;
                    }
                    child.wait()
                });
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = content; // Avoid unused warning
        }
    }

    fn note_deeplink(&self, id: NoteId) -> String {
        format!("kit://notes/{}", id.as_str())
    }

    fn copy_note_as_markdown(&self) {
        self.export_note(ExportFormat::Markdown);
    }

    fn copy_note_deeplink(&self) {
        if let Some(id) = self.selected_note_id {
            let deeplink = self.note_deeplink(id);
            self.copy_text_to_clipboard(&deeplink);
        }
    }

    fn create_note_quicklink(&self) {
        if let Some(id) = self.selected_note_id {
            let title = self
                .notes
                .iter()
                .find(|note| note.id == id)
                .map(|note| {
                    if note.title.is_empty() {
                        "Untitled Note".to_string()
                    } else {
                        note.title.clone()
                    }
                })
                .unwrap_or_else(|| "Untitled Note".to_string());
            let deeplink = self.note_deeplink(id);
            let quicklink = format!("[{}]({})", title, deeplink);
            self.copy_text_to_clipboard(&quicklink);
        }
    }

    fn duplicate_selected_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self.selected_note_id else {
            return;
        };
        let Some(note) = self.notes.iter().find(|note| note.id == id) else {
            return;
        };

        let duplicate = Note::with_content(note.content.clone());
        if let Err(e) = storage::save_note(&duplicate) {
            tracing::error!(error = %e, "Failed to duplicate note");
            return;
        }

        self.notes.insert(0, duplicate.clone());
        self.select_note(duplicate.id, window, cx);
    }

    fn build_action_items(&self) -> Vec<NotesActionItem> {
        let has_selection = self.selected_note_id.is_some();
        let is_trash = self.view_mode == NotesViewMode::Trash;
        let can_edit = has_selection && !is_trash;

        let mut items: Vec<NotesActionItem> = NotesAction::all()
            .iter()
            .map(|action| {
                let enabled = match action {
                    NotesAction::NewNote | NotesAction::BrowseNotes => true,
                    NotesAction::DuplicateNote
                    | NotesAction::FindInNote
                    | NotesAction::CopyNoteAs
                    | NotesAction::CopyDeeplink
                    | NotesAction::CreateQuicklink
                    | NotesAction::Export
                    | NotesAction::Format => can_edit,
                    NotesAction::MoveListItemUp | NotesAction::MoveListItemDown => false,
                    NotesAction::EnableAutoSizing => !self.auto_sizing_enabled,
                    NotesAction::Cancel => true,
                };

                NotesActionItem {
                    action: *action,
                    enabled,
                }
            })
            .collect();

        if !self.auto_sizing_enabled {
            items.push(NotesActionItem {
                action: NotesAction::EnableAutoSizing,
                enabled: true,
            });
        }

        items
    }

    fn open_actions_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let actions = self.build_action_items();
        let action_count = actions.len();
        let pending_action = self.pending_action.clone();

        let panel = cx.new(|cx| {
            let focus_handle = cx.focus_handle();
            NotesActionsPanel::new(
                focus_handle,
                actions,
                Arc::new(move |action| {
                    if let Ok(mut pending) = pending_action.lock() {
                        *pending = Some(action);
                    }
                }),
            )
        });

        let panel_focus_handle = panel.read(cx).focus_handle();
        self.actions_panel = Some(panel);
        self.show_actions_panel = true;
        self.show_browse_panel = false;
        self.browse_panel = None;
        window.focus(&panel_focus_handle, cx);

        self.ensure_actions_panel_height(window, action_count);
        cx.notify();
    }

    fn close_actions_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.show_actions_panel = false;
        self.actions_panel = None;
        self.restore_actions_panel_height(window);

        // Refocus the editor after closing the actions panel
        self.editor_state.update(cx, |state, cx| {
            state.focus(window, cx);
        });

        cx.notify();
    }

    fn ensure_actions_panel_height(&mut self, window: &mut Window, row_count: usize) {
        const ACTIONS_PANEL_WINDOW_MARGIN: f32 = 64.0;

        let panel_height = panel_height_for_rows(row_count);
        let desired_height = panel_height + ACTIONS_PANEL_WINDOW_MARGIN;
        let current_bounds = window.bounds();
        let current_height: f32 = current_bounds.size.height.into();

        if current_height + 1.0 < desired_height {
            self.actions_panel_prev_height = Some(current_height);
            window.resize(size(current_bounds.size.width, px(desired_height)));
            self.last_window_height = desired_height;
        }
    }

    fn restore_actions_panel_height(&mut self, window: &mut Window) {
        let Some(prev_height) = self.actions_panel_prev_height.take() else {
            return;
        };

        let current_bounds = window.bounds();
        window.resize(size(current_bounds.size.width, px(prev_height)));
        self.last_window_height = prev_height;
    }

    fn drain_pending_action(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let pending_action = self
            .pending_action
            .lock()
            .ok()
            .and_then(|mut pending| pending.take());

        if let Some(action) = pending_action {
            self.handle_action(action, window, cx);
        }
    }

    /// Drain pending browse panel actions (select, close, note actions)
    fn drain_pending_browse_actions(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Check for pending note selection
        let pending_select = self
            .pending_browse_select
            .lock()
            .ok()
            .and_then(|mut guard| guard.take());

        if let Some(id) = pending_select {
            self.handle_browse_select(id, window, cx);
            return; // Selection closes the panel, so we're done
        }

        // Check for pending close request
        let pending_close = self
            .pending_browse_close
            .lock()
            .ok()
            .map(|mut guard| {
                let val = *guard;
                *guard = false;
                val
            })
            .unwrap_or(false);

        if pending_close {
            self.close_browse_panel(window, cx);
            return;
        }

        // Check for pending note action (pin/delete)
        let pending_action = self
            .pending_browse_action
            .lock()
            .ok()
            .and_then(|mut guard| guard.take());

        if let Some((id, action)) = pending_action {
            self.handle_browse_action(id, action, cx);
        }
    }

    /// Handle action from the actions panel (Cmd+K)
    fn handle_action(&mut self, action: NotesAction, window: &mut Window, cx: &mut Context<Self>) {
        debug!(?action, "Handling notes action");
        match action {
            NotesAction::NewNote => self.create_note(window, cx),
            NotesAction::DuplicateNote => self.duplicate_selected_note(window, cx),
            NotesAction::BrowseNotes => {
                // Close actions panel first, then open browse panel
                // Don't call close_actions_panel here - it refocuses editor
                // Instead, just clear the state and let open_browse_panel handle focus
                self.show_actions_panel = false;
                self.actions_panel = None;
                self.restore_actions_panel_height(window);
                self.show_browse_panel = true;
                self.open_browse_panel(window, cx);
                cx.notify();
                return; // Early return - browse panel handles its own focus
            }
            NotesAction::FindInNote => {
                self.close_actions_panel(window, cx);
                self.editor_state.update(cx, |state, cx| {
                    state.focus(window, cx);
                });
                cx.dispatch_action(&Search);
                return; // Early return - already handled focus
            }
            NotesAction::CopyNoteAs => self.copy_note_as_markdown(),
            NotesAction::CopyDeeplink => self.copy_note_deeplink(),
            NotesAction::CreateQuicklink => self.create_note_quicklink(),
            NotesAction::Export => self.export_note(ExportFormat::Html),
            NotesAction::MoveListItemUp | NotesAction::MoveListItemDown => {}
            NotesAction::Format => {
                self.show_format_toolbar = !self.show_format_toolbar;
            }
            NotesAction::EnableAutoSizing => {
                self.enable_auto_sizing(window, cx);
            }
            NotesAction::Cancel => {
                // Panel was cancelled, nothing to do
            }
        }
        // Default: close actions panel and refocus editor
        self.close_actions_panel(window, cx);
        cx.notify();
    }

    /// Open the browse panel with current notes
    fn open_browse_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Create NoteListItems from current notes
        let note_items: Vec<NoteListItem> = self
            .notes
            .iter()
            .map(|note| NoteListItem::from_note(note, Some(note.id) == self.selected_note_id))
            .collect();

        // Clone Arcs for the callbacks
        let pending_select = self.pending_browse_select.clone();
        let pending_close = self.pending_browse_close.clone();
        let pending_action = self.pending_browse_action.clone();

        let browse_panel = cx.new(|cx| {
            BrowsePanel::new(note_items, window, cx)
                .on_select(move |id| {
                    if let Ok(mut guard) = pending_select.lock() {
                        *guard = Some(id);
                    }
                })
                .on_close({
                    let pending_close = pending_close.clone();
                    move || {
                        if let Ok(mut guard) = pending_close.lock() {
                            *guard = true;
                        }
                    }
                })
                .on_action(move |id, action| {
                    if let Ok(mut guard) = pending_action.lock() {
                        *guard = Some((id, action));
                    }
                })
        });

        // Focus the browse panel
        let panel_focus_handle = browse_panel.read(cx).focus_handle(cx);
        window.focus(&panel_focus_handle, cx);

        self.browse_panel = Some(browse_panel);
        cx.notify();
    }

    /// Handle note selection from browse panel
    fn handle_browse_select(&mut self, id: NoteId, window: &mut Window, cx: &mut Context<Self>) {
        self.show_browse_panel = false;
        self.browse_panel = None;
        // select_note already focuses the editor
        self.select_note(id, window, cx);
        cx.notify();
    }

    /// Handle note action from browse panel
    fn handle_browse_action(&mut self, id: NoteId, action: NoteAction, cx: &mut Context<Self>) {
        match action {
            NoteAction::TogglePin => {
                if let Some(note) = self.notes.iter_mut().find(|n| n.id == id) {
                    note.is_pinned = !note.is_pinned;
                    if let Err(e) = storage::save_note(note) {
                        tracing::error!(error = %e, "Failed to save note pin state");
                    }
                }
            }
            NoteAction::Delete => {
                let current_id = self.selected_note_id;
                self.selected_note_id = Some(id);
                self.delete_selected_note(cx);
                // Restore selection if different note was deleted
                if current_id != Some(id) {
                    self.selected_note_id = current_id;
                }
            }
        }
        // Update browse panel's note list
        if let Some(ref browse_panel) = self.browse_panel {
            let note_items: Vec<NoteListItem> = self
                .notes
                .iter()
                .map(|note| NoteListItem::from_note(note, Some(note.id) == self.selected_note_id))
                .collect();
            browse_panel.update(cx, |panel, cx| {
                panel.set_notes(note_items, cx);
            });
        }
        cx.notify();
    }

    /// Close the browse panel and refocus the editor
    fn close_browse_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.show_browse_panel = false;
        self.browse_panel = None;

        // Refocus the editor after closing the browse panel
        self.editor_state.update(cx, |state, cx| {
            state.focus(window, cx);
        });

        cx.notify();
    }

    /// Render the search input
    fn render_search(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .px_2()
            .py_1()
            .child(Input::new(&self.search_state).w_full().small())
    }

    /// Render the formatting toolbar
    fn render_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_1()
            .py_1()
            .child(
                Button::new("bold")
                    .ghost()
                    .xsmall()
                    .label("B")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.insert_formatting("**", "**", cx);
                    })),
            )
            .child(
                Button::new("italic")
                    .ghost()
                    .xsmall()
                    .label("I")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.insert_formatting("_", "_", cx);
                    })),
            )
            .child(
                Button::new("heading")
                    .ghost()
                    .xsmall()
                    .label("H")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.insert_formatting("\n## ", "", cx);
                    })),
            )
            .child(
                Button::new("list")
                    .ghost()
                    .xsmall()
                    .label("•")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.insert_formatting("\n- ", "", cx);
                    })),
            )
            .child(
                Button::new("code")
                    .ghost()
                    .xsmall()
                    .label("</>")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.insert_formatting("`", "`", cx);
                    })),
            )
            .child(
                Button::new("codeblock")
                    .ghost()
                    .xsmall()
                    .label("```")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.insert_formatting("\n```\n", "\n```", cx);
                    })),
            )
            .child(
                Button::new("link")
                    .ghost()
                    .xsmall()
                    .label("🔗")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.insert_formatting("[", "](url)", cx);
                    })),
            )
    }

    /// Render the export menu
    fn render_export_menu(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .gap_1()
            .child(
                Button::new("export-txt")
                    .ghost()
                    .xsmall()
                    .label("TXT")
                    .on_click(cx.listener(|this, _, _, _cx| {
                        this.export_note(ExportFormat::PlainText);
                    })),
            )
            .child(
                Button::new("export-md")
                    .ghost()
                    .xsmall()
                    .label("MD")
                    .on_click(cx.listener(|this, _, _, _cx| {
                        this.export_note(ExportFormat::Markdown);
                    })),
            )
            .child(
                Button::new("export-html")
                    .ghost()
                    .xsmall()
                    .label("HTML")
                    .on_click(cx.listener(|this, _, _, _cx| {
                        this.export_note(ExportFormat::Html);
                    })),
            )
    }

    // Note: Sidebar removed for Raycast-style single-note view.
    // Browse panel (Cmd+P) will be implemented as a separate overlay in the future.

    /// Render the main editor area with Raycast-style clean UI
    fn render_editor(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let is_trash = self.view_mode == NotesViewMode::Trash;
        let has_selection = self.selected_note_id.is_some();
        let show_toolbar = self.show_format_toolbar;
        let char_count = self.get_character_count(cx);

        // Get note title - This reads from self.notes which is updated by on_editor_change
        // The title is extracted from the first line of content via Note::set_content()
        let title = self
            .selected_note_id
            .and_then(|id| self.get_visible_notes().iter().find(|n| n.id == id))
            .map(|n| {
                if n.title.is_empty() {
                    "Untitled Note".to_string()
                } else {
                    n.title.clone()
                }
            })
            .unwrap_or_else(|| {
                if is_trash {
                    "No deleted notes".to_string()
                } else {
                    "No note selected".to_string()
                }
            });

        // Raycast-style: titlebar only visible on hover, centered title, right-aligned actions
        let window_hovered = self.window_hovered || self.force_hovered;

        // Get muted foreground color for subtle icons/text
        let muted_color = cx.theme().muted_foreground;

        let titlebar = div()
            .id("notes-titlebar")
            .flex()
            .items_center()
            .justify_center() // Center the title
            .h(px(32.))
            .px_3()
            .relative() // For absolute positioning of icons
            // No background - blends with window
            .bg(cx.theme().background)
            // Only show titlebar elements when window is hovered
            .on_hover(cx.listener(|this, hovered, _, cx| {
                if this.force_hovered {
                    return;
                }

                this.titlebar_hovered = *hovered;
                cx.notify();
            }))
            .child(
                // Note title (truncated) - CENTERED in titlebar
                div()
                    .flex()
                    .items_center()
                    .overflow_hidden()
                    .text_ellipsis()
                    .text_sm()
                    .text_color(muted_color) // Use muted color for subtle title
                    .when(!window_hovered, |d| d.opacity(0.))
                    .child(title),
            )
            // Conditionally show icons based on state - only when window is hovered
            // Raycast-style: icons on the right - settings (actions), panel (browse), + (new)
            // Use absolute positioning to keep title centered
            // Note: "+" and "≡" icons should show even with no notes (so users can create their first note)
            // The "⌘" (actions) icon only shows when a note is selected (needs a note to act on)
            .when(window_hovered && !is_trash, |d| {
                d.child(
                    div()
                        .absolute()
                        .right_3() // Align to right with same padding as px_3
                        .flex()
                        .items_center()
                        .gap_2() // Even spacing between icons
                        // Icon 1: Command key icon - opens actions panel (⌘K)
                        // Only show when a note is selected (actions require a note)
                        .when(has_selection, |d| {
                            d.child(
                                div()
                                    .id("titlebar-cmd-icon")
                                    .text_sm()
                                    .text_color(muted_color.opacity(0.7)) // Muted, subtle icon
                                    .cursor_pointer()
                                    .hover(|s| s.text_color(muted_color)) // Slightly brighter on hover
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        if this.show_actions_panel {
                                            this.close_actions_panel(window, cx);
                                        } else {
                                            this.open_actions_panel(window, cx);
                                        }
                                    }))
                                    .child("⌘"),
                            )
                        })
                        // Icon 2: List icon - for browsing notes (always visible when hovered)
                        .child(
                            div()
                                .id("titlebar-browse-icon")
                                .text_sm()
                                .text_color(muted_color.opacity(0.7)) // Muted, subtle icon
                                .cursor_pointer()
                                .hover(|s| s.text_color(muted_color)) // Slightly brighter on hover
                                .on_click(cx.listener(|this, _, window, cx| {
                                    if this.show_browse_panel {
                                        this.close_browse_panel(window, cx);
                                    } else {
                                        this.close_actions_panel(window, cx);
                                        this.show_browse_panel = true;
                                        this.open_browse_panel(window, cx);
                                    }
                                }))
                                .child("≡"),
                        )
                        // Icon 3: Plus icon - for new note (always visible when hovered)
                        .child(
                            div()
                                .id("titlebar-new-icon")
                                .text_sm()
                                .text_color(muted_color.opacity(0.7)) // Muted, subtle icon
                                .cursor_pointer()
                                .hover(|s| s.text_color(muted_color)) // Slightly brighter on hover
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.create_note(window, cx);
                                }))
                                .child("+"),
                        ),
                )
            })
            .when(has_selection && is_trash, |d| {
                // Trash actions (always visible)
                d.child(
                    div()
                        .flex()
                        .gap_1()
                        .child(
                            Button::new("restore")
                                .ghost()
                                .xsmall()
                                .label("Restore")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.restore_note(window, cx);
                                })),
                        )
                        .child(
                            Button::new("permanent-delete")
                                .ghost()
                                .xsmall()
                                .icon(IconName::Delete)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.permanently_delete_note(cx);
                                })),
                        ),
                )
            });

        // Build character count footer - only visible on hover
        // Raycast style: character count CENTERED, T icon on RIGHT
        let footer = div()
            .flex()
            .items_center()
            .justify_center()
            .relative()
            .h(px(24.))
            .px_3()
            // No border, same background as window
            .bg(cx.theme().background)
            // Hide when window not hovered
            .when(!window_hovered, |d| d.opacity(0.))
            .child(
                // Character count CENTERED (Raycast style)
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!(
                        "{} character{}",
                        char_count,
                        if char_count == 1 { "" } else { "s" }
                    )),
            )
            .child(
                // Type indicator (T for text) on RIGHT (Raycast style)
                div()
                    .absolute()
                    .right_3()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("T"),
            );

        // Build main editor layout - Raycast style: clean, no visible input borders
        div()
            .flex_1()
            .flex()
            .flex_col()
            .h_full()
            .bg(cx.theme().background) // Unified background
            .child(titlebar)
            // Toolbar hidden by default - only show when pinned
            .when(!is_trash && has_selection && show_toolbar, |d| {
                d.child(self.render_toolbar(cx))
            })
            .child(
                div()
                    .flex_1()
                    .p_3()
                    .bg(cx.theme().background) // Same background - no visible input box
                    // Use a styled input that blends with background
                    .child(
                        Input::new(&self.editor_state).h_full().appearance(false), // No input styling - blends with background
                    ),
            )
            .when(has_selection && !is_trash, |d| d.child(footer))
    }

    /// Render the actions panel overlay (Cmd+K)
    ///
    /// IMPORTANT: Uses items_start + fixed top padding to keep the search input
    /// at a stable position. Without this, the panel would re-center when items
    /// are filtered out, causing the search input to jump around.
    fn render_actions_panel_overlay(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let panel = self
            .actions_panel
            .as_ref()
            .map(|panel| panel.clone().into_any_element())
            .unwrap_or_else(|| div().into_any_element());

        // Fixed top offset so search input stays at same position regardless of item count
        const PANEL_TOP_OFFSET: f32 = 32.0;

        div()
            .id("actions-panel-overlay")
            .absolute()
            .inset_0()
            .bg(gpui::rgba(0x00000080))
            .flex()
            .flex_col()
            .items_center() // Horizontally centered
            .justify_start() // Vertically aligned to top (not centered!)
            .pt(px(PANEL_TOP_OFFSET)) // Fixed offset from top
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    this.close_actions_panel(window, cx);
                }),
            )
            .child(
                div()
                    .on_mouse_down(gpui::MouseButton::Left, |_, _, _| {
                        // Stop propagation - don't close when clicking panel
                    })
                    .child(panel),
            )
    }

    /// Render the browse panel overlay (Cmd+P)
    fn render_browse_panel_overlay(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // If we have a browse panel entity, render it
        // Otherwise render an empty container that will close on click
        if let Some(ref browse_panel) = self.browse_panel {
            div()
                .id("browse-panel-overlay")
                .absolute()
                .inset_0()
                .child(browse_panel.clone())
        } else {
            // Fallback: create inline browse panel
            let note_items: Vec<NoteListItem> = self
                .notes
                .iter()
                .map(|note| NoteListItem::from_note(note, Some(note.id) == self.selected_note_id))
                .collect();

            // We need a simple inline version since we can't create entities in render
            div()
                .id("browse-panel-overlay")
                .absolute()
                .inset_0()
                .bg(gpui::rgba(0x00000080))
                .flex()
                .items_center()
                .justify_center()
                .on_click(cx.listener(|this, _, window, cx| {
                    this.close_browse_panel(window, cx);
                }))
                .child(
                    div()
                        .w(px(500.))
                        .max_h(px(400.))
                        .bg(cx.theme().background)
                        .border_1()
                        .border_color(cx.theme().border)
                        .rounded_lg()
                        .shadow_lg()
                        .p_4()
                        .on_mouse_down(gpui::MouseButton::Left, |_, _, _| {
                            // Stop propagation
                        })
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!("{} notes available", note_items.len())),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .mt_2()
                                .child("Press Escape to close"),
                        ),
                )
        }
    }

    /// Get cached box shadows (computed once at construction)
    fn create_box_shadows(&self) -> Vec<BoxShadow> {
        self.cached_box_shadows.clone()
    }

    /// Compute box shadows from theme configuration (called once at construction)
    fn compute_box_shadows() -> Vec<BoxShadow> {
        let theme = crate::theme::load_theme();
        let shadow_config = theme.get_drop_shadow();

        if !shadow_config.enabled {
            return vec![];
        }

        // Convert hex color to HSLA
        let r = ((shadow_config.color >> 16) & 0xFF) as f32 / 255.0;
        let g = ((shadow_config.color >> 8) & 0xFF) as f32 / 255.0;
        let b = (shadow_config.color & 0xFF) as f32 / 255.0;

        // Simple RGB to HSL conversion
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;

        let (h, s) = if max == min {
            (0.0, 0.0)
        } else {
            let d = max - min;
            let s = if l > 0.5 {
                d / (2.0 - max - min)
            } else {
                d / (max + min)
            };
            let h = if max == r {
                (g - b) / d + if g < b { 6.0 } else { 0.0 }
            } else if max == g {
                (b - r) / d + 2.0
            } else {
                (r - g) / d + 4.0
            };
            (h / 6.0, s)
        };

        vec![BoxShadow {
            color: hsla(h, s, l, shadow_config.opacity),
            offset: point(px(shadow_config.offset_x), px(shadow_config.offset_y)),
            blur_radius: px(shadow_config.blur_radius),
            spread_radius: px(shadow_config.spread_radius),
        }]
    }

    /// Update cached box shadows when theme changes
    pub fn update_theme(&mut self, _cx: &mut Context<Self>) {
        self.cached_box_shadows = Self::compute_box_shadows();
    }
}

impl Focusable for NotesApp {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for NotesApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Detect if user manually resized the window (disables auto-sizing)
        self.detect_manual_resize(window);
        self.drain_pending_action(window, cx);
        self.drain_pending_browse_actions(window, cx);

        let show_actions = self.show_actions_panel;
        let show_browse = self.show_browse_panel;

        // Raycast-style single-note view: no sidebar, editor fills full width
        // Track window hover for traffic lights visibility
        let box_shadows = self.create_box_shadows();

        div()
            .id("notes-window-root")
            .flex()
            .flex_col()
            .size_full()
            .relative()
            .bg(cx.theme().background)
            .shadow(box_shadows)
            .text_color(cx.theme().foreground)
            .track_focus(&self.focus_handle)
            // Track window hover for showing/hiding chrome
            .on_hover(cx.listener(|this, hovered, _, cx| {
                if this.force_hovered {
                    return;
                }

                this.window_hovered = *hovered;
                cx.notify();
            }))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                // Handle keyboard shortcuts
                let key = event.keystroke.key.to_lowercase();
                let modifiers = &event.keystroke.modifiers;

                if this.show_actions_panel {
                    if key == "escape" || (modifiers.platform && key == "k") || key == "esc" {
                        this.close_actions_panel(window, cx);
                        return;
                    }

                    if let Some(ref panel) = this.actions_panel {
                        match key.as_str() {
                            "up" | "arrowup" => {
                                panel.update(cx, |panel, cx| panel.move_up(cx));
                            }
                            "down" | "arrowdown" => {
                                panel.update(cx, |panel, cx| panel.move_down(cx));
                            }
                            "enter" => {
                                if let Some(action) = panel.read(cx).get_selected_action() {
                                    this.handle_action(action, window, cx);
                                }
                            }
                            "backspace" => {
                                panel.update(cx, |panel, cx| panel.handle_backspace(cx));
                            }
                            _ => {
                                if let Some(ref key_char) = event.keystroke.key_char {
                                    if let Some(ch) = key_char.chars().next() {
                                        if !ch.is_control() {
                                            panel.update(cx, |panel, cx| {
                                                panel.handle_char(ch, cx);
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }

                    return;
                }

                // Handle browse panel keyboard events
                if this.show_browse_panel {
                    if key == "escape" || (modifiers.platform && key == "p") || key == "esc" {
                        this.close_browse_panel(window, cx);
                        return;
                    }

                    if let Some(ref panel) = this.browse_panel {
                        match key.as_str() {
                            "up" | "arrowup" => {
                                panel.update(cx, |panel, cx| panel.move_up(cx));
                            }
                            "down" | "arrowdown" => {
                                panel.update(cx, |panel, cx| panel.move_down(cx));
                            }
                            "enter" => {
                                if let Some(id) = panel.read(cx).get_selected_note_id() {
                                    this.handle_browse_select(id, window, cx);
                                }
                            }
                            _ => {
                                // Let BrowsePanel handle other keys (like search input)
                            }
                        }
                    }

                    return;
                }

                // Handle Escape to close panels
                if key == "escape" {
                    if this.show_actions_panel {
                        this.close_actions_panel(window, cx);
                        return;
                    }
                    if this.show_browse_panel {
                        this.close_browse_panel(window, cx);
                        return;
                    }
                }

                // platform modifier = Cmd on macOS, Ctrl on Windows/Linux
                if modifiers.platform {
                    match key.as_str() {
                        "k" => {
                            // Toggle actions panel
                            if this.show_actions_panel {
                                this.close_actions_panel(window, cx);
                            } else {
                                this.open_actions_panel(window, cx);
                            }
                        }
                        "p" => {
                            // Toggle browse panel
                            this.show_browse_panel = !this.show_browse_panel;
                            this.close_actions_panel(window, cx);
                            if this.show_browse_panel {
                                this.open_browse_panel(window, cx);
                            } else {
                                this.browse_panel = None;
                            }
                            cx.notify();
                        }
                        "n" => this.create_note(window, cx),
                        "w" => {
                            // Close the notes window (standard macOS pattern)
                            window.remove_window();
                        }
                        "d" => this.duplicate_selected_note(window, cx),
                        "b" => this.insert_formatting("**", "**", cx),
                        "i" => this.insert_formatting("_", "_", cx),
                        _ => {}
                    }
                }
            }))
            // Single note view - editor takes full width
            .child(self.render_editor(cx))
            // Overlay panels
            .when(show_actions, |d| {
                d.child(self.render_actions_panel_overlay(cx))
            })
            .when(show_browse, |d| {
                d.child(self.render_browse_panel_overlay(cx))
            })
    }
}

/// Initialize gpui-component theme and sync with Script Kit theme
fn ensure_theme_initialized(cx: &mut App) {
    // First, initialize gpui-component (this sets up the default theme)
    gpui_component::init(cx);

    // Use the shared theme sync function from the theme module
    crate::theme::sync_gpui_component_theme(cx);

    info!("Notes window theme synchronized with Script Kit");
}

/// Calculate window bounds positioned in the top-right corner of the display containing the mouse.
fn calculate_top_right_bounds(width: f32, height: f32, padding: f32) -> gpui::Bounds<gpui::Pixels> {
    use crate::platform::{get_global_mouse_position, get_macos_displays};

    let displays = get_macos_displays();

    // Find display containing mouse
    let target_display = if let Some((mouse_x, mouse_y)) = get_global_mouse_position() {
        displays
            .iter()
            .find(|display| {
                mouse_x >= display.origin_x
                    && mouse_x < display.origin_x + display.width
                    && mouse_y >= display.origin_y
                    && mouse_y < display.origin_y + display.height
            })
            .cloned()
    } else {
        None
    };

    // Use found display or fall back to primary
    let display = target_display.or_else(|| displays.first().cloned());

    if let Some(display) = display {
        // Position in top-right corner with padding
        let x = display.origin_x + display.width - width as f64 - padding as f64;
        let y = display.origin_y + padding as f64;

        gpui::Bounds::new(
            gpui::Point::new(px(x as f32), px(y as f32)),
            gpui::Size::new(px(width), px(height)),
        )
    } else {
        // Fallback to centered on primary
        gpui::Bounds::new(
            gpui::Point::new(px(100.0), px(100.0)),
            gpui::Size::new(px(width), px(height)),
        )
    }
}

/// Toggle the notes window (open if closed, close if open)
pub fn open_notes_window(cx: &mut App) -> Result<()> {
    use crate::logging;

    logging::log("PANEL", "open_notes_window called - checking toggle state");

    // Ensure gpui-component theme is initialized before opening window
    ensure_theme_initialized(cx);

    let window_handle = NOTES_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
    let mut guard = window_handle.lock().unwrap();

    // Check if window already exists and is valid
    if let Some(ref handle) = *guard {
        // Window exists - check if it's valid and close it (toggle OFF)
        if handle
            .update(cx, |_, window, _cx| {
                window.remove_window();
            })
            .is_ok()
        {
            logging::log("PANEL", "Notes window was open - closing (toggle OFF)");
            *guard = None;

            // NOTE: We intentionally do NOT call cx.hide() here.
            // Closing Notes should not affect the main window's ability to be shown.
            // The main window hotkey handles its own visibility state.
            // If the user wants to hide everything, they can press the main hotkey
            // when the main window is visible.

            return Ok(());
        }
        // Window handle was invalid, fall through to create new window
        logging::log("PANEL", "Notes window handle was invalid - creating new");
    }

    // If main window is visible, hide it (Notes takes focus)
    // Use platform::hide_main_window() to only hide the main window, not the whole app
    // IMPORTANT: Set visibility to false so the main hotkey knows to SHOW (not hide) next time
    if crate::is_main_window_visible() {
        logging::log(
            "PANEL",
            "Main window was visible - hiding it since Notes is opening",
        );
        crate::set_main_window_visible(false);
        crate::platform::hide_main_window();
    }

    // Create new window (toggle ON)
    logging::log("PANEL", "Notes window not open - creating new (toggle ON)");
    info!("Opening new notes window");

    // Calculate position: top-right corner of the display containing the mouse
    let window_width = 350.0_f32;
    let window_height = 280.0_f32;
    let padding = 20.0_f32; // Padding from screen edges

    let bounds = calculate_top_right_bounds(window_width, window_height, padding);

    // Load theme to determine window background appearance (vibrancy)
    let theme = crate::theme::load_theme();
    let window_background = if theme.is_vibrancy_enabled() {
        gpui::WindowBackgroundAppearance::Blurred
    } else {
        gpui::WindowBackgroundAppearance::Opaque
    };

    let window_options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(gpui::TitlebarOptions {
            title: Some("Notes".into()),
            appears_transparent: true,
            traffic_light_position: Some(gpui::Point {
                x: px(8.),
                y: px(8.),
            }),
        }),
        window_background,
        focus: true,
        show: true,
        kind: gpui::WindowKind::Normal,
        ..Default::default()
    };

    // Store the NotesApp entity so we can focus it after window creation
    let notes_app_holder: std::sync::Arc<std::sync::Mutex<Option<Entity<NotesApp>>>> =
        std::sync::Arc::new(std::sync::Mutex::new(None));
    let notes_app_for_closure = notes_app_holder.clone();

    let handle = cx.open_window(window_options, |window, cx| {
        let view = cx.new(|cx| NotesApp::new(window, cx));
        *notes_app_for_closure.lock().unwrap() = Some(view.clone());
        cx.new(|cx| Root::new(view, window, cx))
    })?;

    // CRITICAL: Activate the app FIRST before focusing the window
    // This brings the app to the foreground on macOS, which is required
    // for the window to receive keyboard focus when the app wasn't already active
    cx.activate(true);

    // CRITICAL: Hide the main window AFTER activating the app
    // When we activate the app, macOS may bring all windows to the front.
    // We need to explicitly hide the main window to prevent it from appearing.
    // This uses orderOut: which hides just the main window, not the entire app.
    crate::platform::hide_main_window();

    // Focus the editor input in the Notes window
    if let Some(notes_app) = notes_app_holder.lock().unwrap().clone() {
        let _ = handle.update(cx, |_root, window, cx| {
            window.activate_window();

            // Focus the NotesApp's editor input
            notes_app.update(cx, |app, cx| {
                // Call the InputState's focus method which handles both focus handle and internal state
                app.editor_state.update(cx, |state, inner_cx| {
                    state.focus(window, inner_cx);
                });

                if std::env::var("SCRIPT_KIT_TEST_NOTES_HOVERED")
                    .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
                    .unwrap_or(false)
                {
                    app.force_hovered = true;
                    app.window_hovered = true;
                    app.titlebar_hovered = true;
                }

                if std::env::var("SCRIPT_KIT_TEST_NOTES_ACTIONS_PANEL")
                    .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
                    .unwrap_or(false)
                {
                    app.open_actions_panel(window, cx);
                }

                cx.notify();
            });
        });
    }

    *guard = Some(handle);

    // Configure as floating panel (always on top) after window is created
    configure_notes_as_floating_panel();

    // Theme hot-reload watcher for Notes window
    // Spawns a background task that watches ~/.sk/kit/theme.json for changes
    if let Some(notes_app) = notes_app_holder.lock().unwrap().clone() {
        let notes_app_for_theme = notes_app.clone();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            let (mut theme_watcher, theme_rx) = ThemeWatcher::new();
            if theme_watcher.start().is_err() {
                return;
            }
            loop {
                gpui::Timer::after(std::time::Duration::from_millis(200)).await;
                if theme_rx.try_recv().is_ok() {
                    info!("Notes window: theme.json changed, reloading");
                    let _ = cx.update(|cx| {
                        // Re-sync gpui-component theme with updated Script Kit theme
                        crate::theme::sync_gpui_component_theme(cx);
                        // Notify the Notes window to re-render with new colors
                        notes_app_for_theme.update(cx, |_app, cx| {
                            cx.notify();
                        });
                    });
                }
            }
        })
        .detach();
    }

    Ok(())
}

/// Quick capture - open notes with a new note ready for input
pub fn quick_capture(cx: &mut App) -> Result<()> {
    open_notes_window(cx)?;

    // TODO: Focus the editor and optionally create a new note
    // This requires accessing the NotesApp through the Root wrapper

    Ok(())
}

/// Close the notes window
pub fn close_notes_window(cx: &mut App) {
    let window_handle = NOTES_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
    let mut guard = window_handle.lock().unwrap();

    if let Some(handle) = guard.take() {
        let _ = handle.update(cx, |_, window, _| {
            window.remove_window();
        });
    }
}

/// Check if the notes window is currently open
///
/// Returns true if the Notes window exists and is valid.
/// This is used by other parts of the app to check if Notes is open
/// without affecting it.
pub fn is_notes_window_open() -> bool {
    let window_handle = NOTES_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
    let guard = window_handle.lock().unwrap();
    guard.is_some()
}

/// Configure the Notes window as a floating panel (always on top).
///
/// This sets:
/// - NSFloatingWindowLevel (3) - floats above normal windows
/// - NSWindowCollectionBehaviorMoveToActiveSpace - moves to current space when shown
/// - Disabled window restoration - prevents macOS position caching
#[cfg(target_os = "macos")]
fn configure_notes_as_floating_panel() {
    use crate::logging;
    use std::ffi::CStr;

    unsafe {
        let app: id = NSApp();
        let windows: id = msg_send![app, windows];
        let count: usize = msg_send![windows, count];

        for i in 0..count {
            let window: id = msg_send![windows, objectAtIndex: i];
            let title: id = msg_send![window, title];

            if title != nil {
                let title_cstr: *const i8 = msg_send![title, UTF8String];
                if !title_cstr.is_null() {
                    let title_str = CStr::from_ptr(title_cstr).to_string_lossy();

                    if title_str == "Notes" {
                        // Found the Notes window - configure it

                        // NSFloatingWindowLevel = 3
                        let floating_level: i32 = 3;
                        let _: () = msg_send![window, setLevel:floating_level];

                        // NSWindowCollectionBehaviorMoveToActiveSpace = 2
                        let collection_behavior: u64 = 2;
                        let _: () = msg_send![window, setCollectionBehavior:collection_behavior];

                        // Ensure window content is shareable for captureScreenshot()
                        let sharing_type: i64 = 1; // NSWindowSharingReadOnly
                        let _: () = msg_send![window, setSharingType:sharing_type];

                        // Disable window restoration
                        let _: () = msg_send![window, setRestorable:false];

                        logging::log(
                            "PANEL",
                            "Notes window configured as floating panel (level=3, MoveToActiveSpace)",
                        );
                        return;
                    }
                }
            }
        }

        logging::log(
            "PANEL",
            "Warning: Notes window not found by title for floating panel config",
        );
    }
}

#[cfg(not(target_os = "macos"))]
fn configure_notes_as_floating_panel() {
    // No-op on non-macOS platforms
}

</file>

<file path="src/notes/actions_panel.rs">
//! Notes Actions Panel
//!
//! Modal overlay panel triggered by Cmd+K in the Notes window.
//! Provides searchable action list for note operations.
//!
//! ## Actions
//! - New Note (⌘N) - Create a new note
//! - Duplicate Note (⌘D) - Create a copy of the current note
//! - Browse Notes (⌘P) - Open note browser/picker
//! - Find in Note (⌘F) - Search within current note
//! - Copy Note As... (⇧⌘C) - Copy note in a chosen format
//! - Copy Deeplink (⇧⌘D) - Copy a deeplink to the note
//! - Create Quicklink (⇧⌘L) - Copy a quicklink to the note
//! - Export... (⇧⌘E) - Export note content
//! - Move List Item Up (⌃⌘↑) - Reorder notes list (disabled)
//! - Move List Item Down (⌃⌘↓) - Reorder notes list (disabled)
//! - Format... (⇧⌘T) - Formatting commands
//!
//! ## Keyboard Navigation
//! - Arrow Up/Down: Navigate actions
//! - Enter: Execute selected action
//! - Escape: Close panel
//! - Type to search/filter actions

use crate::designs::icon_variations::IconName;
use gpui::{
    div, point, prelude::*, px, svg, uniform_list, App, BoxShadow, Context, FocusHandle, Focusable,
    Hsla, MouseButton, Render, ScrollStrategy, SharedString, UniformListScrollHandle, Window,
};
use gpui_component::theme::{ActiveTheme, Theme};
use std::sync::Arc;
use tracing::debug;

/// Callback type for action execution
/// The String parameter is the action ID
pub type NotesActionCallback = Arc<dyn Fn(NotesAction) + Send + Sync>;

/// Available actions in the Notes actions panel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotesAction {
    /// Create a new note
    NewNote,
    /// Duplicate the current note
    DuplicateNote,
    /// Open the note browser/picker
    BrowseNotes,
    /// Search within the current note
    FindInNote,
    /// Copy note content as a formatted export
    CopyNoteAs,
    /// Copy deeplink to the current note
    CopyDeeplink,
    /// Copy quicklink to the current note
    CreateQuicklink,
    /// Export note content
    Export,
    /// Move list item up (disabled placeholder)
    MoveListItemUp,
    /// Move list item down (disabled placeholder)
    MoveListItemDown,
    /// Open formatting commands
    Format,
    /// Enable auto-sizing (window grows/shrinks with content)
    EnableAutoSizing,
    /// Panel was cancelled (Escape pressed)
    Cancel,
}

impl NotesAction {
    /// Get all available actions (excluding Cancel)
    pub fn all() -> &'static [NotesAction] {
        &[
            NotesAction::NewNote,
            NotesAction::DuplicateNote,
            NotesAction::BrowseNotes,
            NotesAction::FindInNote,
            NotesAction::CopyNoteAs,
            NotesAction::CopyDeeplink,
            NotesAction::CreateQuicklink,
            NotesAction::Export,
            NotesAction::MoveListItemUp,
            NotesAction::MoveListItemDown,
            NotesAction::Format,
        ]
    }

    /// Get the display label for this action
    pub fn label(&self) -> &'static str {
        match self {
            NotesAction::NewNote => "New Note",
            NotesAction::DuplicateNote => "Duplicate Note",
            NotesAction::BrowseNotes => "Browse Notes",
            NotesAction::FindInNote => "Find in Note",
            NotesAction::CopyNoteAs => "Copy Note As...",
            NotesAction::CopyDeeplink => "Copy Deeplink",
            NotesAction::CreateQuicklink => "Create Quicklink",
            NotesAction::Export => "Export...",
            NotesAction::MoveListItemUp => "Move List Item Up",
            NotesAction::MoveListItemDown => "Move List Item Down",
            NotesAction::Format => "Format...",
            NotesAction::EnableAutoSizing => "Enable Auto-Sizing",
            NotesAction::Cancel => "Cancel",
        }
    }

    /// Get the keyboard shortcut key (without modifier)
    pub fn shortcut_key(&self) -> &'static str {
        match self {
            NotesAction::NewNote => "N",
            NotesAction::DuplicateNote => "D",
            NotesAction::BrowseNotes => "P",
            NotesAction::FindInNote => "F",
            NotesAction::CopyNoteAs => "C",
            NotesAction::CopyDeeplink => "D",
            NotesAction::CreateQuicklink => "L",
            NotesAction::Export => "E",
            NotesAction::MoveListItemUp => "↑",
            NotesAction::MoveListItemDown => "↓",
            NotesAction::Format => "T",
            NotesAction::EnableAutoSizing => "A",
            NotesAction::Cancel => "Esc",
        }
    }

    /// Get shortcut keys for keycap rendering
    pub fn shortcut_keys(&self) -> &'static [&'static str] {
        const CMD_N: [&str; 2] = ["⌘", "N"];
        const CMD_D: [&str; 2] = ["⌘", "D"];
        const CMD_P: [&str; 2] = ["⌘", "P"];
        const CMD_F: [&str; 2] = ["⌘", "F"];
        const SHIFT_CMD_C: [&str; 3] = ["⇧", "⌘", "C"];
        const SHIFT_CMD_D: [&str; 3] = ["⇧", "⌘", "D"];
        const SHIFT_CMD_L: [&str; 3] = ["⇧", "⌘", "L"];
        const SHIFT_CMD_E: [&str; 3] = ["⇧", "⌘", "E"];
        const CTRL_CMD_UP: [&str; 3] = ["⌃", "⌘", "↑"];
        const CTRL_CMD_DOWN: [&str; 3] = ["⌃", "⌘", "↓"];
        const SHIFT_CMD_T: [&str; 3] = ["⇧", "⌘", "T"];
        const CMD_A: [&str; 2] = ["⌘", "A"];
        const ESC: [&str; 1] = ["Esc"];

        match self {
            NotesAction::NewNote => &CMD_N,
            NotesAction::DuplicateNote => &CMD_D,
            NotesAction::BrowseNotes => &CMD_P,
            NotesAction::FindInNote => &CMD_F,
            NotesAction::CopyNoteAs => &SHIFT_CMD_C,
            NotesAction::CopyDeeplink => &SHIFT_CMD_D,
            NotesAction::CreateQuicklink => &SHIFT_CMD_L,
            NotesAction::Export => &SHIFT_CMD_E,
            NotesAction::MoveListItemUp => &CTRL_CMD_UP,
            NotesAction::MoveListItemDown => &CTRL_CMD_DOWN,
            NotesAction::Format => &SHIFT_CMD_T,
            NotesAction::EnableAutoSizing => &CMD_A,
            NotesAction::Cancel => &ESC,
        }
    }

    /// Get the formatted shortcut display string
    pub fn shortcut_display(&self) -> String {
        if self.shortcut_keys().is_empty() {
            return String::new();
        }

        self.shortcut_keys().join("")
    }

    /// Get the icon for this action (uses local IconName from designs module)
    pub fn icon(&self) -> IconName {
        match self {
            NotesAction::NewNote => IconName::Plus,
            NotesAction::DuplicateNote => IconName::Copy,
            NotesAction::BrowseNotes => IconName::FolderOpen,
            NotesAction::FindInNote => IconName::MagnifyingGlass,
            NotesAction::CopyNoteAs => IconName::Copy,
            NotesAction::CopyDeeplink => IconName::ArrowRight,
            NotesAction::CreateQuicklink => IconName::Star,
            NotesAction::Export => IconName::ArrowRight,
            NotesAction::MoveListItemUp => IconName::ArrowUp,
            NotesAction::MoveListItemDown => IconName::ArrowDown,
            NotesAction::Format => IconName::Code,
            NotesAction::EnableAutoSizing => IconName::ArrowRight,
            NotesAction::Cancel => IconName::Close,
        }
    }

    /// Get action ID for lookup
    pub fn id(&self) -> &'static str {
        match self {
            NotesAction::NewNote => "new_note",
            NotesAction::DuplicateNote => "duplicate_note",
            NotesAction::BrowseNotes => "browse_notes",
            NotesAction::FindInNote => "find_in_note",
            NotesAction::CopyNoteAs => "copy_note_as",
            NotesAction::CopyDeeplink => "copy_deeplink",
            NotesAction::CreateQuicklink => "create_quicklink",
            NotesAction::Export => "export",
            NotesAction::MoveListItemUp => "move_list_item_up",
            NotesAction::MoveListItemDown => "move_list_item_down",
            NotesAction::Format => "format",
            NotesAction::EnableAutoSizing => "enable_auto_sizing",
            NotesAction::Cancel => "cancel",
        }
    }
}

/// Action list sections for visual grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NotesActionSection {
    Primary,
    Actions,
    Move,
    Format,
    Utility,
}

impl NotesActionSection {
    fn for_action(action: NotesAction) -> Self {
        match action {
            NotesAction::NewNote | NotesAction::DuplicateNote | NotesAction::BrowseNotes => {
                NotesActionSection::Primary
            }
            NotesAction::FindInNote
            | NotesAction::CopyNoteAs
            | NotesAction::CopyDeeplink
            | NotesAction::CreateQuicklink
            | NotesAction::Export => NotesActionSection::Actions,
            NotesAction::MoveListItemUp | NotesAction::MoveListItemDown => NotesActionSection::Move,
            NotesAction::Format => NotesActionSection::Format,
            NotesAction::EnableAutoSizing | NotesAction::Cancel => NotesActionSection::Utility,
        }
    }
}

/// Action entry with enabled state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotesActionItem {
    pub action: NotesAction,
    pub enabled: bool,
}

impl NotesActionItem {
    fn section(&self) -> NotesActionSection {
        NotesActionSection::for_action(self.action)
    }
}

/// Panel dimensions and styling constants (matches main ActionsDialog)
pub const PANEL_WIDTH: f32 = 320.0;
pub const PANEL_MAX_HEIGHT: f32 = 580.0;
pub const PANEL_CORNER_RADIUS: f32 = 12.0;
pub const ACTION_ITEM_HEIGHT: f32 = 44.0;
pub const PANEL_SEARCH_HEIGHT: f32 = 44.0;
pub const PANEL_BORDER_HEIGHT: f32 = 2.0;
/// Horizontal inset for action rows (creates rounded pill appearance)
pub const ACTION_ROW_INSET: f32 = 6.0;
/// Corner radius for selected row background
pub const SELECTION_RADIUS: f32 = 8.0;

pub fn panel_height_for_rows(row_count: usize) -> f32 {
    let items_height = (row_count as f32 * ACTION_ITEM_HEIGHT)
        .min(PANEL_MAX_HEIGHT - (PANEL_SEARCH_HEIGHT + 16.0));
    items_height + PANEL_SEARCH_HEIGHT + PANEL_BORDER_HEIGHT
}

/// Notes Actions Panel - Modal overlay for note operations
pub struct NotesActionsPanel {
    /// Available actions
    actions: Vec<NotesActionItem>,
    /// Filtered action indices
    filtered_indices: Vec<usize>,
    /// Currently selected index (within filtered)
    selected_index: usize,
    /// Search text
    search_text: String,
    /// Focus handle
    focus_handle: FocusHandle,
    /// Callback for action selection
    on_action: NotesActionCallback,
    /// Scroll handle for virtualization
    scroll_handle: UniformListScrollHandle,
    /// Cursor blink visibility
    cursor_visible: bool,
}

impl NotesActionsPanel {
    /// Create a new NotesActionsPanel
    pub fn new(
        focus_handle: FocusHandle,
        actions: Vec<NotesActionItem>,
        on_action: NotesActionCallback,
    ) -> Self {
        let filtered_indices: Vec<usize> = (0..actions.len()).collect();
        let selected_index = actions.iter().position(|item| item.enabled).unwrap_or(0);

        debug!(action_count = actions.len(), "Notes actions panel created");

        Self {
            actions,
            filtered_indices,
            selected_index,
            search_text: String::new(),
            focus_handle,
            on_action,
            scroll_handle: UniformListScrollHandle::new(),
            cursor_visible: true,
        }
    }

    /// Set cursor visibility (for blink animation)
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
    }

    pub fn focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }

    /// Handle character input
    pub fn handle_char(&mut self, ch: char, cx: &mut Context<Self>) {
        self.search_text.push(ch);
        self.refilter();
        cx.notify();
    }

    /// Handle backspace
    pub fn handle_backspace(&mut self, cx: &mut Context<Self>) {
        if !self.search_text.is_empty() {
            self.search_text.pop();
            self.refilter();
            cx.notify();
        }
    }

    /// Move selection up
    pub fn move_up(&mut self, cx: &mut Context<Self>) {
        self.move_selection(-1, cx);
    }

    /// Move selection down
    pub fn move_down(&mut self, cx: &mut Context<Self>) {
        self.move_selection(1, cx);
    }

    /// Submit the selected action
    pub fn submit_selected(&mut self) {
        if let Some(&action_idx) = self.filtered_indices.get(self.selected_index) {
            if let Some(action) = self.actions.get(action_idx) {
                if action.enabled {
                    debug!(action = ?action.action, "Notes action selected");
                    (self.on_action)(action.action);
                }
            }
        }
    }

    /// Cancel and close
    pub fn cancel(&mut self) {
        debug!("Notes actions panel cancelled");
        (self.on_action)(NotesAction::Cancel);
    }

    /// Get currently selected action
    pub fn get_selected_action(&self) -> Option<NotesAction> {
        self.filtered_indices
            .get(self.selected_index)
            .and_then(|&idx| self.actions.get(idx))
            .and_then(|item| {
                if item.enabled {
                    Some(item.action)
                } else {
                    None
                }
            })
    }

    /// Refilter actions based on search text
    fn refilter(&mut self) {
        if self.search_text.is_empty() {
            self.filtered_indices = (0..self.actions.len()).collect();
        } else {
            let search_lower = self.search_text.to_lowercase();
            self.filtered_indices = self
                .actions
                .iter()
                .enumerate()
                .filter(|(_, action)| action.action.label().to_lowercase().contains(&search_lower))
                .map(|(idx, _)| idx)
                .collect();
        }

        self.ensure_valid_selection();

        // Scroll to keep selection visible
        if !self.filtered_indices.is_empty() {
            self.scroll_handle
                .scroll_to_item(self.selected_index, ScrollStrategy::Nearest);
        }
    }

    fn ensure_valid_selection(&mut self) {
        if self.filtered_indices.is_empty() {
            self.selected_index = 0;
            return;
        }

        if self.selected_index >= self.filtered_indices.len()
            || !self.is_selectable(self.selected_index)
        {
            if let Some(index) =
                (0..self.filtered_indices.len()).find(|&idx| self.is_selectable(idx))
            {
                self.selected_index = index;
            } else {
                self.selected_index = 0;
            }
        }
    }

    fn is_selectable(&self, filtered_idx: usize) -> bool {
        self.filtered_indices
            .get(filtered_idx)
            .and_then(|&idx| self.actions.get(idx))
            .map(|item| item.enabled)
            .unwrap_or(false)
    }

    fn move_selection(&mut self, delta: i32, cx: &mut Context<Self>) {
        let filtered_len = self.filtered_indices.len();
        if filtered_len == 0 {
            return;
        }

        let mut next_index = self.selected_index as i32;
        loop {
            next_index += delta;
            if next_index < 0 || next_index >= filtered_len as i32 {
                break;
            }

            let next = next_index as usize;
            if self.is_selectable(next) {
                self.selected_index = next;
                self.scroll_handle
                    .scroll_to_item(self.selected_index, ScrollStrategy::Nearest);
                cx.notify();
                return;
            }
        }
    }

    /// Create box shadow for the overlay
    fn create_shadow() -> Vec<BoxShadow> {
        vec![
            BoxShadow {
                color: Hsla {
                    h: 0.0,
                    s: 0.0,
                    l: 0.0,
                    a: 0.3,
                },
                offset: point(px(0.0), px(4.0)),
                blur_radius: px(16.0),
                spread_radius: px(0.0),
            },
            BoxShadow {
                color: Hsla {
                    h: 0.0,
                    s: 0.0,
                    l: 0.0,
                    a: 0.15,
                },
                offset: point(px(0.0), px(8.0)),
                blur_radius: px(32.0),
                spread_radius: px(-4.0),
            },
        ]
    }
}

impl Focusable for NotesActionsPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for NotesActionsPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        // Colors from gpui-component theme
        let bg_color = theme.background;
        let border_color = theme.border;
        let text_primary = theme.foreground;
        let text_muted = theme.muted_foreground;
        let accent_color = theme.accent;

        // Search display
        let search_display = if self.search_text.is_empty() {
            SharedString::from("Search for actions...")
        } else {
            SharedString::from(self.search_text.clone())
        };

        // Build search input row - Raycast style: no search icon, just placeholder with cursor
        let search_input = div()
            .w_full()
            .h(px(PANEL_SEARCH_HEIGHT))
            .px(px(12.0))
            .py(px(8.0))
            .bg(theme.secondary)
            .border_b_1()
            .border_color(border_color)
            .flex()
            .flex_row()
            .items_center()
            // Search field - full width, no icon
            .child(
                div()
                    .flex_1()
                    .h(px(28.0))
                    .px(px(8.0))
                    .bg(theme.input)
                    .rounded(px(4.0))
                    .border_1()
                    .border_color(if self.search_text.is_empty() {
                        border_color
                    } else {
                        accent_color
                    })
                    .flex()
                    .flex_row()
                    .items_center()
                    .text_sm()
                    .text_color(if self.search_text.is_empty() {
                        text_muted
                    } else {
                        text_primary
                    })
                    // Cursor when empty
                    .when(self.search_text.is_empty(), |d| {
                        d.child(
                            div()
                                .w(px(2.))
                                .h(px(16.))
                                .mr(px(2.))
                                .rounded(px(1.))
                                .when(self.cursor_visible, |d| d.bg(accent_color)),
                        )
                    })
                    .child(search_display)
                    // Cursor when has text
                    .when(!self.search_text.is_empty(), |d| {
                        d.child(
                            div()
                                .w(px(2.))
                                .h(px(16.))
                                .ml(px(2.))
                                .rounded(px(1.))
                                .when(self.cursor_visible, |d| d.bg(accent_color)),
                        )
                    }),
            );

        // Build actions list
        let selected_index = self.selected_index;
        let filtered_len = self.filtered_indices.len();

        let actions_list = if self.filtered_indices.is_empty() {
            div()
                .flex_1()
                .w_full()
                .py(px(16.0))
                .px(px(12.0))
                .text_color(text_muted)
                .text_sm()
                .child("No actions match your search")
                .into_any_element()
        } else {
            uniform_list(
                "notes-actions-list",
                filtered_len,
                cx.processor(
                    move |this: &mut NotesActionsPanel, visible_range, _window, cx| {
                        let theme = cx.theme();
                        let mut items = Vec::new();

                        for idx in visible_range {
                            if let Some(&action_idx) = this.filtered_indices.get(idx) {
                                if let Some(action) = this.actions.get(action_idx) {
                                    let action: &NotesActionItem = action;
                                    let is_enabled = action.enabled;
                                    let is_selected = idx == selected_index && is_enabled;
                                    let is_section_start = if idx > 0 {
                                        this.filtered_indices
                                            .get(idx - 1)
                                            .and_then(|&prev_idx| this.actions.get(prev_idx))
                                            .map(|prev: &NotesActionItem| {
                                                prev.section() != action.section()
                                            })
                                            .unwrap_or(false)
                                    } else {
                                        false
                                    };

                                    // Transparent Hsla for unselected state
                                    let transparent = Hsla {
                                        h: 0.0,
                                        s: 0.0,
                                        l: 0.0,
                                        a: 0.0,
                                    };

                                    // Raycast-style: rounded pill selection, no left accent bar
                                    // Outer wrapper provides horizontal inset for the rounded background
                                    let action_row = div()
                                        .id(idx)
                                        .w_full()
                                        .h(px(ACTION_ITEM_HEIGHT))
                                        .px(px(ACTION_ROW_INSET))
                                        .flex()
                                        .flex_col()
                                        .justify_center()
                                        // Section divider as top border
                                        .when(is_section_start, |d| {
                                            d.border_t_1().border_color(theme.border)
                                        })
                                        // Inner row with rounded background
                                        .child(
                                            div()
                                                .w_full()
                                                .h(px(ACTION_ITEM_HEIGHT - 8.0))
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .px(px(8.0))
                                                .rounded(px(SELECTION_RADIUS))
                                                .bg(if is_selected {
                                                    theme.list_active
                                                } else {
                                                    transparent
                                                })
                                                .when(is_enabled, |d| {
                                                    d.hover(|s| s.bg(theme.list_hover))
                                                })
                                                .when(is_enabled, |d| d.cursor_pointer())
                                                .when(!is_enabled, |d| d.opacity(0.5))
                                                // Content row: icon + label + shortcuts
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .flex()
                                                        .flex_row()
                                                        .items_center()
                                                        .justify_between()
                                                        // Left: icon + label
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .flex_row()
                                                                .items_center()
                                                                .gap(px(10.0))
                                                                // Icon
                                                                .child(
                                                                    svg()
                                                                        .external_path(action.action.icon().external_path())
                                                                        .size(px(16.))
                                                                        .text_color(if is_enabled {
                                                                            theme.foreground
                                                                        } else {
                                                                            theme.muted_foreground
                                                                        }),
                                                                )
                                                                // Label
                                                                .child(
                                                                    div()
                                                                        .text_sm()
                                                                        .text_color(if is_enabled {
                                                                            theme.foreground
                                                                        } else {
                                                                            theme.muted_foreground
                                                                        })
                                                                        .font_weight(
                                                                            if is_selected {
                                                                                gpui::FontWeight::MEDIUM
                                                                            } else {
                                                                                gpui::FontWeight::NORMAL
                                                                            },
                                                                        )
                                                                        .child(action.action.label()),
                                                                ),
                                                        )
                                                        // Right: shortcut badge
                                                        .child(render_shortcut_keys(
                                                            action.action.shortcut_keys(),
                                                            theme,
                                                        )),
                                                ),
                                        )
                                        .when(is_enabled, |d| {
                                            d.on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(move |this, _, _, cx| {
                                                    this.selected_index = idx;
                                                    this.submit_selected();
                                                    cx.notify();
                                                }),
                                            )
                                        });

                                    items.push(action_row);
                                }
                            }
                        }
                        items
                    },
                ),
            )
            .flex_1()
            .w_full()
            .track_scroll(&self.scroll_handle)
            .into_any_element()
        };

        // Calculate dynamic height
        let items_height = (filtered_len as f32 * ACTION_ITEM_HEIGHT)
            .min(PANEL_MAX_HEIGHT - (PANEL_SEARCH_HEIGHT + 16.0));
        let total_height = items_height + PANEL_SEARCH_HEIGHT + PANEL_BORDER_HEIGHT;

        // Main container
        div()
            .flex()
            .flex_col()
            .w(px(PANEL_WIDTH))
            .h(px(total_height))
            .bg(bg_color)
            .rounded(px(PANEL_CORNER_RADIUS))
            .shadow(Self::create_shadow())
            .border_1()
            .border_color(border_color)
            .overflow_hidden()
            .track_focus(&self.focus_handle)
            .child(search_input)
            .child(actions_list)
    }
}

fn render_shortcut_keys(keys: &[&'static str], theme: &Theme) -> impl IntoElement {
    if keys.is_empty() {
        return div().into_any_element();
    }

    let mut row = div().flex().flex_row().items_center().gap(px(4.0));

    for key in keys {
        row = row.child(
            div()
                .min_w(px(18.0))
                .px(px(6.0))
                .py(px(2.0))
                .bg(theme.muted)
                .border_1()
                .border_color(theme.border)
                .rounded(px(5.0))
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(*key),
        );
    }

    row.into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notes_action_labels() {
        assert_eq!(NotesAction::NewNote.label(), "New Note");
        assert_eq!(NotesAction::DuplicateNote.label(), "Duplicate Note");
        assert_eq!(NotesAction::BrowseNotes.label(), "Browse Notes");
        assert_eq!(NotesAction::FindInNote.label(), "Find in Note");
        assert_eq!(NotesAction::CopyNoteAs.label(), "Copy Note As...");
        assert_eq!(NotesAction::CopyDeeplink.label(), "Copy Deeplink");
        assert_eq!(NotesAction::CreateQuicklink.label(), "Create Quicklink");
        assert_eq!(NotesAction::Export.label(), "Export...");
        assert_eq!(NotesAction::MoveListItemUp.label(), "Move List Item Up");
        assert_eq!(NotesAction::MoveListItemDown.label(), "Move List Item Down");
        assert_eq!(NotesAction::Format.label(), "Format...");
    }

    #[test]
    fn test_notes_action_shortcuts() {
        assert_eq!(NotesAction::NewNote.shortcut_display(), "⌘N");
        assert_eq!(NotesAction::DuplicateNote.shortcut_display(), "⌘D");
        assert_eq!(NotesAction::BrowseNotes.shortcut_display(), "⌘P");
        assert_eq!(NotesAction::FindInNote.shortcut_display(), "⌘F");
        assert_eq!(NotesAction::CopyNoteAs.shortcut_display(), "⇧⌘C");
        assert_eq!(NotesAction::CopyDeeplink.shortcut_display(), "⇧⌘D");
        assert_eq!(NotesAction::CreateQuicklink.shortcut_display(), "⇧⌘L");
        assert_eq!(NotesAction::Export.shortcut_display(), "⇧⌘E");
        assert_eq!(NotesAction::MoveListItemUp.shortcut_display(), "⌃⌘↑");
        assert_eq!(NotesAction::MoveListItemDown.shortcut_display(), "⌃⌘↓");
        assert_eq!(NotesAction::Format.shortcut_display(), "⇧⌘T");
    }

    #[test]
    fn test_notes_action_all() {
        let all = NotesAction::all();
        assert_eq!(all.len(), 11);
        assert!(all.contains(&NotesAction::NewNote));
        assert!(all.contains(&NotesAction::DuplicateNote));
        assert!(all.contains(&NotesAction::BrowseNotes));
        assert!(all.contains(&NotesAction::FindInNote));
        assert!(all.contains(&NotesAction::CopyNoteAs));
        assert!(all.contains(&NotesAction::CopyDeeplink));
        assert!(all.contains(&NotesAction::CreateQuicklink));
        assert!(all.contains(&NotesAction::Export));
        assert!(all.contains(&NotesAction::MoveListItemUp));
        assert!(all.contains(&NotesAction::MoveListItemDown));
        assert!(all.contains(&NotesAction::Format));
    }

    #[test]
    fn test_notes_action_ids() {
        assert_eq!(NotesAction::NewNote.id(), "new_note");
        assert_eq!(NotesAction::DuplicateNote.id(), "duplicate_note");
        assert_eq!(NotesAction::BrowseNotes.id(), "browse_notes");
        assert_eq!(NotesAction::FindInNote.id(), "find_in_note");
        assert_eq!(NotesAction::CopyNoteAs.id(), "copy_note_as");
        assert_eq!(NotesAction::CopyDeeplink.id(), "copy_deeplink");
        assert_eq!(NotesAction::CreateQuicklink.id(), "create_quicklink");
        assert_eq!(NotesAction::Export.id(), "export");
        assert_eq!(NotesAction::MoveListItemUp.id(), "move_list_item_up");
        assert_eq!(NotesAction::MoveListItemDown.id(), "move_list_item_down");
        assert_eq!(NotesAction::Format.id(), "format");
    }

    #[test]
    fn test_panel_constants() {
        // Verify panel matches Raycast-style dimensions
        assert_eq!(PANEL_WIDTH, 320.0);
        assert_eq!(PANEL_MAX_HEIGHT, 580.0);
        assert_eq!(PANEL_CORNER_RADIUS, 12.0);
        assert_eq!(ACTION_ITEM_HEIGHT, 44.0);
        assert_eq!(ACTION_ROW_INSET, 6.0);
        assert_eq!(SELECTION_RADIUS, 8.0);
    }
}

</file>

<file path="src/notes/browse_panel.rs">
//! Browse Panel for Notes
//!
//! A modal overlay component triggered by Cmd+P that displays a searchable list
//! of notes. Follows Raycast's browse panel design pattern.
//!
//! ## Features
//! - Search input at top with "Search for notes..." placeholder
//! - "Notes" section header
//! - Note rows showing: current indicator (red dot), title, character count
//! - Hover reveals pin/delete action icons
//! - Keyboard navigation (arrow keys, enter to select, escape to close)
//! - Filter notes as user types in search

use gpui::{
    div, prelude::*, px, rgb, App, Context, Entity, FocusHandle, Focusable, IntoElement,
    KeyDownEvent, MouseButton, ParentElement, Render, Styled, Subscription, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants},
    input::{Input, InputEvent, InputState},
    theme::ActiveTheme,
    IconName, Sizable,
};

use super::model::{Note, NoteId};

/// Lightweight note data for display in the browse panel
#[derive(Debug, Clone)]
pub struct NoteListItem {
    /// Note identifier
    pub id: NoteId,
    /// Note title (or "Untitled Note" if empty)
    pub title: String,
    /// Character count
    pub char_count: usize,
    /// Whether this is the currently selected note
    pub is_current: bool,
    /// Whether this note is pinned
    pub is_pinned: bool,
}

impl NoteListItem {
    /// Create a NoteListItem from a Note
    pub fn from_note(note: &Note, is_current: bool) -> Self {
        Self {
            id: note.id,
            title: if note.title.is_empty() {
                "Untitled Note".to_string()
            } else {
                note.title.clone()
            },
            char_count: note.char_count(),
            is_current,
            is_pinned: note.is_pinned,
        }
    }
}

/// Callback type for note selection
pub type OnSelectNote = Box<dyn Fn(NoteId) + 'static>;

/// Callback type for panel close
pub type OnClose = Box<dyn Fn() + 'static>;

/// Callback type for note actions (pin, delete)
pub type OnNoteAction = Box<dyn Fn(NoteId, NoteAction) + 'static>;

/// Actions that can be performed on a note from the browse panel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteAction {
    /// Toggle pin status
    TogglePin,
    /// Delete the note
    Delete,
}

/// Browse Panel - modal overlay for browsing and selecting notes
///
/// This component is designed to be rendered as an overlay on top of the
/// main notes window. It handles:
/// - Search input with filtering
/// - Arrow key navigation
/// - Enter to select, Escape to close
/// - Pin/delete actions on hover
pub struct BrowsePanel {
    /// All notes (filtered by search)
    notes: Vec<NoteListItem>,
    /// Original unfiltered notes
    all_notes: Vec<NoteListItem>,
    /// Currently highlighted index in the list
    selected_index: usize,
    /// Search input state
    search_state: Entity<InputState>,
    /// Focus handle for keyboard events
    focus_handle: FocusHandle,
    /// Index of note row being hovered (for showing action icons)
    hovered_index: Option<usize>,
    /// Callback when a note is selected
    on_select: Option<OnSelectNote>,
    /// Callback when panel should close
    on_close: Option<OnClose>,
    /// Callback for note actions
    on_action: Option<OnNoteAction>,
    /// Subscriptions to keep alive
    _subscriptions: Vec<Subscription>,
}

impl BrowsePanel {
    /// Create a new BrowsePanel with the given notes
    ///
    /// # Arguments
    /// * `notes` - List of notes to display
    /// * `window` - Window reference for input state
    /// * `cx` - Context for creating entities
    pub fn new(notes: Vec<NoteListItem>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("Search for notes..."));

        let focus_handle = cx.focus_handle();

        // Subscribe to search input changes
        let search_sub = cx.subscribe_in(&search_state, window, {
            move |this, _, ev: &InputEvent, _window, cx| {
                if matches!(ev, InputEvent::Change) {
                    this.on_search_change(cx);
                }
            }
        });

        Self {
            notes: notes.clone(),
            all_notes: notes,
            selected_index: 0,
            search_state,
            focus_handle,
            hovered_index: None,
            on_select: None,
            on_close: None,
            on_action: None,
            _subscriptions: vec![search_sub],
        }
    }

    /// Set the callback for note selection
    pub fn on_select(mut self, callback: impl Fn(NoteId) + 'static) -> Self {
        self.on_select = Some(Box::new(callback));
        self
    }

    /// Set the callback for panel close
    pub fn on_close(mut self, callback: impl Fn() + 'static) -> Self {
        self.on_close = Some(Box::new(callback));
        self
    }

    /// Set the callback for note actions
    pub fn on_action(mut self, callback: impl Fn(NoteId, NoteAction) + 'static) -> Self {
        self.on_action = Some(Box::new(callback));
        self
    }

    /// Update the notes list
    pub fn set_notes(&mut self, notes: Vec<NoteListItem>, cx: &mut Context<Self>) {
        self.all_notes = notes.clone();
        self.notes = notes;
        self.selected_index = 0;
        cx.notify();
    }

    /// Handle search input changes
    fn on_search_change(&mut self, cx: &mut Context<Self>) {
        let query = self
            .search_state
            .read(cx)
            .value()
            .to_string()
            .to_lowercase();

        if query.is_empty() {
            self.notes = self.all_notes.clone();
        } else {
            self.notes = self
                .all_notes
                .iter()
                .filter(|note| note.title.to_lowercase().contains(&query))
                .cloned()
                .collect();
        }

        // Reset selection to first item
        self.selected_index = 0;
        cx.notify();
    }

    /// Move selection up
    pub fn move_up(&mut self, cx: &mut Context<Self>) {
        if !self.notes.is_empty() {
            self.selected_index = self.selected_index.saturating_sub(1);
            cx.notify();
        }
    }

    /// Move selection down
    pub fn move_down(&mut self, cx: &mut Context<Self>) {
        if !self.notes.is_empty() {
            self.selected_index = (self.selected_index + 1).min(self.notes.len() - 1);
            cx.notify();
        }
    }

    /// Select the current note
    fn select_current(&mut self, _cx: &mut Context<Self>) {
        if let Some(note) = self.notes.get(self.selected_index) {
            if let Some(ref on_select) = self.on_select {
                on_select(note.id);
            }
        }
    }

    /// Get the currently selected note ID (for parent window keyboard handling)
    pub fn get_selected_note_id(&self) -> Option<NoteId> {
        self.notes.get(self.selected_index).map(|n| n.id)
    }

    /// Close the panel
    fn close(&self) {
        if let Some(ref on_close) = self.on_close {
            on_close();
        }
    }

    /// Handle note action (pin/delete)
    fn handle_action(&self, id: NoteId, action: NoteAction) {
        if let Some(ref on_action) = self.on_action {
            on_action(id, action);
        }
    }

    /// Render the search input
    fn render_search(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .px_3()
            .py_2()
            .child(Input::new(&self.search_state).w_full().small())
    }

    /// Render the section header
    fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .px_3()
            .py_1()
            .text_xs()
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(cx.theme().muted_foreground)
            .child("Notes")
    }

    /// Render a single note row
    fn render_note_row(
        &self,
        index: usize,
        note: &NoteListItem,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_selected = index == self.selected_index;
        let is_hovered = self.hovered_index == Some(index);
        let note_id = note.id;

        // Row background based on state
        let bg_color = if is_selected {
            cx.theme().list_active
        } else if is_hovered {
            cx.theme().list_hover
        } else {
            gpui::transparent_black()
        };

        div()
            .id(("note-row", index))
            .w_full()
            .h(px(36.))
            .px_3()
            .flex()
            .items_center()
            .gap_2()
            .bg(bg_color)
            .rounded_sm()
            .cursor_pointer()
            .on_mouse_move(cx.listener(move |this, _, _, cx| {
                if this.hovered_index != Some(index) {
                    this.hovered_index = Some(index);
                    cx.notify();
                }
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.selected_index = index;
                    this.select_current(cx);
                }),
            )
            // Current note indicator (red dot)
            .child(
                div()
                    .w(px(8.))
                    .h(px(8.))
                    .rounded_full()
                    .when(note.is_current, |d| d.bg(rgb(0xff4444)))
                    .when(!note.is_current, |d| d.bg(gpui::transparent_black())),
            )
            // Title
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .text_ellipsis()
                    .text_sm()
                    .text_color(cx.theme().foreground)
                    .child(note.title.clone()),
            )
            // Character count
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!(
                        "{} character{}",
                        note.char_count,
                        if note.char_count == 1 { "" } else { "s" }
                    )),
            )
            // Action buttons (visible on hover)
            .when(is_hovered, |d| {
                d.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(
                            Button::new(("pin", index))
                                .ghost()
                                .xsmall()
                                .icon(IconName::Star)
                                .on_click(cx.listener(move |this, _, _, _cx| {
                                    this.handle_action(note_id, NoteAction::TogglePin);
                                })),
                        )
                        .child(
                            Button::new(("delete", index))
                                .ghost()
                                .xsmall()
                                .icon(IconName::Delete)
                                .on_click(cx.listener(move |this, _, _, _cx| {
                                    this.handle_action(note_id, NoteAction::Delete);
                                })),
                        ),
                )
            })
    }

    /// Render the notes list
    fn render_list(&self, cx: &mut Context<Self>) -> impl IntoElement {
        if self.notes.is_empty() {
            return div()
                .w_full()
                .py_8()
                .flex()
                .items_center()
                .justify_center()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("No notes found")
                .into_any_element();
        }

        let mut list = div().w_full().flex().flex_col().gap_px();

        for (index, note) in self.notes.iter().enumerate() {
            list = list.child(self.render_note_row(index, note, cx));
        }

        list.into_any_element()
    }
}

impl Focusable for BrowsePanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for BrowsePanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Modal backdrop (semi-transparent overlay)
        div()
            .id("browse-panel-backdrop")
            .absolute()
            .inset_0()
            .bg(gpui::rgba(0x00000080)) // 50% opacity black
            .flex()
            .items_center()
            .justify_center()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, _cx| {
                    this.close();
                }),
            )
            // Panel container
            .child(
                div()
                    .id("browse-panel")
                    .w(px(500.))
                    .max_h(px(400.))
                    .bg(cx.theme().background)
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded_lg()
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .track_focus(&self.focus_handle)
                    .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                        let key = event.keystroke.key.as_str();
                        match key {
                            "up" | "arrowup" => this.move_up(cx),
                            "down" | "arrowdown" => this.move_down(cx),
                            "enter" => this.select_current(cx),
                            "escape" => this.close(),
                            _ => {}
                        }
                    }))
                    // Prevent backdrop click from closing when clicking panel
                    .on_mouse_down(MouseButton::Left, |_, _, _| {})
                    // Search input
                    .child(self.render_search(cx))
                    // Section header
                    .child(self.render_header(cx))
                    // Notes list (scrollable)
                    .child(
                        div()
                            .flex_1()
                            .overflow_hidden()
                            .px_1()
                            .py_1()
                            .on_mouse_move(cx.listener(|this, _, _, cx| {
                                // Clear hover when mouse leaves list area without entering a row
                                if this.hovered_index.is_some() {
                                    // This will be overridden by row hover handlers
                                }
                                let _ = cx;
                            }))
                            .child(self.render_list(cx)),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_list_item_from_note() {
        use chrono::Utc;

        let note = Note {
            id: NoteId::new(),
            title: "Test Note".to_string(),
            content: "Hello, world!".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            is_pinned: false,
            sort_order: 0,
        };

        let item = NoteListItem::from_note(&note, true);
        assert_eq!(item.title, "Test Note");
        assert_eq!(item.char_count, 13);
        assert!(item.is_current);
        assert!(!item.is_pinned);
    }

    #[test]
    fn test_note_list_item_untitled() {
        use chrono::Utc;

        let note = Note {
            id: NoteId::new(),
            title: "".to_string(),
            content: "Some content".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            is_pinned: true,
            sort_order: 0,
        };

        let item = NoteListItem::from_note(&note, false);
        assert_eq!(item.title, "Untitled Note");
        assert!(!item.is_current);
        assert!(item.is_pinned);
    }
}

</file>

</files>
📊 Pack Summary:
────────────────
  Total Files: 6 files
  Search Mode: ripgrep (fast)
  Total Tokens: ~29.9K (29,897 exact)
  Total Chars: 144,628 chars
       Output: -

📁 Extensions Found:
────────────────────
  .rs

📂 Top 10 Files (by tokens):
──────────────────────────
     15.1K - src/notes/window.rs
      6.7K - src/notes/actions_panel.rs
      3.5K - src/notes/browse_panel.rs
      2.8K - src/notes/storage.rs
      1.6K - src/notes/model.rs
       377 - src/notes/mod.rs
