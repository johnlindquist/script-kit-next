//! Notes: storage for day notes and named notes, and how the app opens one.
//!
//! Every note is a Markdown file and opens in the one note editor, which fills the main window
//! under its input (`src/main_sections/day_page_*.rs`). The Notes list (`src/one_window/notes.rs`)
//! finds them. There is no separate Notes window.
//!
//! # API Visibility
//!
//! Storage internals are private to the app.
//!
//! ```compile_fail
//! use script_kit_gpui::notes::save_note;
//! use script_kit_gpui::notes::get_all_notes;
//! ```
//!

// Allow dead code in this module - many functions are designed for future use
#![allow(dead_code)]

mod actions_panel;
pub(crate) mod ai_scope;
pub(crate) mod code_highlight;
pub(crate) mod day_switcher;
pub(crate) mod deeplink_activation;
pub(crate) mod file_names;
pub mod folder;
pub(crate) mod ghost;
pub(crate) mod ghost_llm;
pub(crate) mod markdown;
pub(crate) mod markdown_highlighting;
mod mcp_mutations;
pub(crate) mod menu_syntax_capture;
pub(crate) mod merge;
pub(crate) mod metadata;
mod model;
mod open;
pub(crate) mod search_model;
mod storage;
pub(crate) mod style;
// Used by the library's day page editor; the binary's copy of this module doesn't call it.
#[allow(unused_imports)]
pub(crate) use storage::save_note_from_editor;
pub(crate) use storage::verify_saved_note_content;

// Re-export notes action catalog for CommandBar action builders
#[allow(unused_imports)]
pub use actions_panel::{
    notes_action_descriptors, notes_action_for_id, notes_action_for_keystroke, NotesAction,
    NotesActionAvailability, NotesActionConfirmation, NotesActionContext, NotesActionDescriptor,
    NotesActionSurface,
};

#[allow(unused_imports)]
pub(crate) use model::{Note, NoteCartItem, NoteCartItemPayload, NoteId};
#[allow(unused_imports)]
pub(crate) use storage::{
    count_active_notes_with_tag, delete_note_cart_item, delete_note_cart_items,
    delete_note_permanently, discard_root_notes_search_refresh, ensure_notes_db_open,
    finish_root_notes_search_refresh, get_all_notes, get_deleted_notes, get_note, get_note_aliases,
    get_note_backlink_count, get_note_backlinks, get_note_outbound_link_count, get_note_tags,
    init_notes_db, invalidate_owned_root_notes_search_freshness,
    invalidate_root_notes_search_cache, list_note_cart_items, list_note_cart_items_deduped,
    note_editor_content_from_file, note_file_path, notes_brain_days_dir, notes_changed_on,
    notes_substrate, owned_root_notes_search_snapshot, pinned_root_notes,
    read_root_notes_search_snapshot, reset_owned_root_notes_search, root_notes_query_is_eligible,
    root_notes_search_cache_is_fresh, root_notes_search_fresh_cache_status, save_note,
    save_note_cart_item, search_notes, search_root_notes_meta, search_root_notes_meta_cached,
    search_root_notes_meta_direct, try_begin_root_notes_search_refresh, NoteBacklinkSummary,
    RootNoteSearchHit, RootNotesSearchRefresh, RootNotesSearchSnapshot, RootNotesSectionOptions,
};

/// Tag that promotes a note to a standing agent instruction.
///
/// Notes tagged `#instructions` (or `tags: [instructions]` frontmatter) are
/// automatically staged as context on new Agent Chat threads.
pub const NOTES_INSTRUCTIONS_TAG: &str = "instructions";

/// MCP resource URI that resolves all instruction notes with full bodies.
pub const NOTES_INSTRUCTIONS_RESOURCE_URI: &str = "kit://notes?tag=instructions&full=true";

/// Label shown on the instructions context chip in Agent Chat.
pub const NOTES_INSTRUCTIONS_LABEL: &str = "Note Instructions";

/// Provenance URI recorded in `source:` frontmatter for notes created from an
/// Agent Chat thread. Keeps the note ↔ conversation relationship in the data
/// layer so any surface can link back.
pub fn agent_chat_thread_source(ui_thread_id: &str) -> String {
    format!("scriptkit://agent-chat/{ui_thread_id}")
}

pub use mcp_mutations::apply_mcp_notes_mutation_on_main_thread;
pub use open::{open_note, register_open_note_hook, save_note_with_content, NoteTarget};
// The note editor lives in the binary; the library build doesn't use these.
#[allow(unused_imports)]
pub(crate) use open::new_note_to_edit;
#[allow(unused_imports)]
pub(crate) use storage::{resolve_note_ref, rewrite_note_links, NoteRefResolution};
