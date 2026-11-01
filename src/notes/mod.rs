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
pub mod export;
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
pub(crate) use storage::{cold_content_for_new_note, verify_saved_note_content};

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

/// Most text standing instructions may add to a chat (all notes together).
pub const STANDING_INSTRUCTIONS_MAX_CHARS: usize = 12_000;

/// Instruction notes ready to send with the first message of a chat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandingInstructions {
    /// The prompt block (titles and bodies, capped).
    pub text: String,
    /// Titles of the notes included, in order.
    pub titles: Vec<String>,
    /// True when the cap cut text off.
    pub truncated: bool,
}

/// Builds the standing-instructions block from `(title, body)` pairs. Empty
/// bodies are skipped; the whole block stays under `max_chars`.
pub fn format_standing_instructions(
    notes: &[(String, String)],
    max_chars: usize,
) -> Option<StandingInstructions> {
    let header = "Standing instructions from the user's notes tagged #instructions. Follow them unless the user says otherwise.\n";
    let mut text = String::from(header);
    let mut titles = Vec::new();
    let mut truncated = false;
    for (title, body) in notes {
        let body = body.trim();
        if body.is_empty() {
            continue;
        }
        let section = format!("\n## {}\n{}\n", title.trim(), body);
        let room = max_chars.saturating_sub(text.chars().count());
        if room == 0 {
            truncated = true;
            break;
        }
        if section.chars().count() > room {
            text.extend(section.chars().take(room));
            titles.push(title.trim().to_string());
            truncated = true;
            break;
        }
        text.push_str(&section);
        titles.push(title.trim().to_string());
    }
    (!titles.is_empty()).then_some(StandingInstructions {
        text,
        titles,
        truncated,
    })
}

/// Reads every active note tagged `#instructions` and formats it for a chat.
pub fn load_standing_instructions() -> anyhow::Result<Option<StandingInstructions>> {
    let notes = search_notes(&format!("tag:{NOTES_INSTRUCTIONS_TAG}"))?;
    let pairs: Vec<(String, String)> = notes
        .into_iter()
        .map(|note| (note.title, note.content))
        .collect();
    Ok(format_standing_instructions(
        &pairs,
        STANDING_INSTRUCTIONS_MAX_CHARS,
    ))
}

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
pub(crate) use storage::search_note_ids;
#[allow(unused_imports)]
pub(crate) use storage::{resolve_note_ref, rewrite_note_links, NoteRefResolution};

#[cfg(test)]
mod standing_instructions_tests {
    use super::*;

    #[test]
    fn instructions_block_lists_each_note_and_skips_empty_ones() {
        let notes = vec![
            ("Tone".to_string(), "Be brief.".to_string()),
            ("Empty".to_string(), "   ".to_string()),
            ("Code".to_string(), "Prefer Rust.".to_string()),
        ];
        let block = format_standing_instructions(&notes, 10_000).expect("block");
        assert_eq!(block.titles, vec!["Tone", "Code"]);
        assert!(block.text.contains("## Tone\nBe brief."));
        assert!(block.text.contains("## Code\nPrefer Rust."));
        assert!(!block.truncated);
    }

    #[test]
    fn instructions_block_respects_the_cap() {
        let notes = vec![("Long".to_string(), "x".repeat(500))];
        let block = format_standing_instructions(&notes, 200).expect("block");
        assert!(block.truncated);
        assert!(block.text.chars().count() <= 200);
    }

    #[test]
    fn no_instruction_notes_means_no_block() {
        assert_eq!(format_standing_instructions(&[], 1000), None);
        let blank = vec![("Blank".to_string(), String::new())];
        assert_eq!(format_standing_instructions(&blank, 1000), None);
    }
}
