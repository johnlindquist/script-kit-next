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

/// The first note anyone makes: what notes can do, pinned so it's easy to find again.
pub(crate) const WELCOME_NOTE: &str = "# Welcome to Notes

Every note is a Markdown file named after its title, in `~/.scriptkit/brain/notes` (or the folder `notesFolder` names in config.ts), so Obsidian or any editor can open it too.

- Type a few words in the main window to find a note; pinned notes come first. Type `#` on the empty input to start a new one.
- `⌘T` opens today's page. Start a line in the main window with `;` to add it to today. On today's page, ↵ after a line like `- 14:32 called Sam` starts the next one with the time.
- Link one note to another with [[Note title]]. Type `[[` and a few letters, then Tab to finish the title. `⌘.` opens the link under the cursor.
- `⌘K` lists what you can do with the open note: pin it, copy its link, duplicate it, see which notes link to it.
- `⌘↵` asks the model about the line you're on. In the main window, `@notes:` attaches a note or a day page to what you ask.

This note stays pinned until you unpin it from `⌘K`.
";

/// Makes a note to edit, starting with `content` (blank when `None`).
///
/// The very first note anyone makes comes with the welcome note, pinned. A blank first note is
/// the welcome note itself.
pub(crate) fn new_note_to_edit(content: Option<&str>) -> Result<NoteId> {
    storage::init_notes_db()?;
    let first_ever =
        storage::get_all_notes()?.is_empty() && storage::get_deleted_notes()?.is_empty();
    if first_ever {
        let mut welcome = Note::with_content(WELCOME_NOTE);
        welcome.is_pinned = true;
        storage::save_note(&welcome)?;
        if content.is_none() {
            return Ok(welcome.id);
        }
    }
    let note = Note::with_content(content.unwrap_or_default());
    storage::save_note(&note)?;
    Ok(note.id)
}
