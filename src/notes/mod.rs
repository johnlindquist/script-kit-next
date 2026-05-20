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
//! # API Visibility
//!
//! Storage internals are private implementation details of the notes window.
//!
//! ```compile_fail
//! use script_kit_gpui::notes::save_note;
//! use script_kit_gpui::notes::get_all_notes;
//! ```
//!

// Allow dead code in this module - many functions are designed for future use
#![allow(dead_code)]

mod actions_panel;
mod browse_panel;
pub(crate) mod code_highlight;
mod markdown;
mod markdown_highlighting;
pub(crate) mod metadata;
mod model;
mod storage;
pub(crate) mod window;

// Re-export actions panel types for use by window.rs
#[allow(unused_imports)]
pub use actions_panel::{NotesAction, NotesActionCallback, NotesActionItem, NotesActionsPanel};

// Re-export browse panel types for use by window.rs
#[allow(unused_imports)]
pub use browse_panel::{BrowsePanel, NoteAction, NoteListItem};

#[allow(unused_imports)]
pub(crate) use model::{Note, NoteCartItem, NoteCartItemPayload, NoteId};
#[allow(unused_imports)]
pub(crate) use storage::{
    delete_note_cart_item, delete_note_cart_items, delete_note_permanently, get_all_notes,
    get_deleted_notes, get_note, get_note_aliases, get_note_backlink_count, get_note_backlinks,
    get_note_outbound_link_count, get_note_tags, init_notes_db, list_note_cart_items,
    list_note_cart_items_deduped, root_notes_query_is_eligible, save_note, save_note_cart_item,
    search_notes, search_root_notes_meta, search_root_notes_meta_cached,
    search_root_notes_meta_direct, NoteBacklinkSummary, RootNoteSearchHit, RootNotesSectionOptions,
};

// Re-export key types - suppress unused warnings since these are public API
#[allow(unused_imports)]
pub use window::{
    apply_mcp_notes_mutation_on_main_thread, close_notes_embedded_acp, close_notes_window,
    get_notes_app_entity_and_handle, get_notes_editor_text, inject_text_into_notes,
    is_notes_window, is_notes_window_open, open_note_in_notes_window, open_notes_window,
    open_notes_window_without_launcher_restore, quick_capture, save_note_with_content, NotesApp,
    NotesSurfaceMode,
};
