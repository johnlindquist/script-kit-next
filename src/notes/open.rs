//! Opening a note. Notes open in the main window, under its input, as the note view; this is the
//! one way in for code that holds only the app (hotkeys, the tray, stdin, MCP, other windows).
//! The binary registers how the main window opens one, since this module is compiled into the
//! library too and can't name the main window's view.

use std::sync::OnceLock;

use anyhow::Result;
use gpui::App;

use super::{storage, Note, NoteId};

/// Which note to open, and what to do with it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NoteTarget {
    /// Today's day note.
    Today,
    Note(NoteId),
    Day(chrono::NaiveDate),
    /// A note that doesn't exist yet: the first line added creates it.
    New,
    /// Today's day note, with these words ready to add as a line.
    Append(String),
    /// The notes, listed by name, to pick one.
    Search,
}

pub type OpenNoteHook = fn(&mut App, NoteTarget) -> Result<()>;

static OPEN_NOTE_HOOK: OnceLock<OpenNoteHook> = OnceLock::new();

/// Called once by the binary at startup.
pub fn register_open_note_hook(hook: OpenNoteHook) {
    if OPEN_NOTE_HOOK.set(hook).is_err() {
        tracing::warn!(target: "script_kit::notes", event = "open_note_hook_already_registered");
    }
}

/// Shows the main window with `target` open under its input.
pub fn open_note(cx: &mut App, target: NoteTarget) -> Result<()> {
    let hook = OPEN_NOTE_HOOK
        .get()
        .ok_or_else(|| anyhow::anyhow!("The main window can't open notes yet"))?;
    hook(cx, target)
}

/// Saves `content` as a new note and opens it.
pub fn save_note_with_content(cx: &mut App, content: String) -> Result<()> {
    save_note_with_content_and_source(cx, content, None)
}

/// Like [`save_note_with_content`], and records where the note came from (`source:` frontmatter)
/// so it points back at the conversation or surface that produced it.
pub fn save_note_with_content_and_source(
    cx: &mut App,
    content: String,
    source: Option<String>,
) -> Result<()> {
    let content = match source {
        Some(source) => super::metadata::merge_frontmatter(
            &content,
            super::metadata::MetadataFrontmatterPatch {
                tags: vec![],
                aliases: vec![],
                source: Some(source),
            },
        ),
        None => content,
    };
    storage::init_notes_db()?;
    let note = Note::with_content(content);
    storage::save_note(&note)?;
    open_note(cx, NoteTarget::Note(note.id))
}
