//! Write-through markdown mirror for notes.
//!
//! Every active note is mirrored to `~/.scriptkit/notes/<slug>-<id8>.md` so
//! agents, scripts, and external tools (grep, Obsidian, git) can read notes as
//! plain files. SQLite remains the source of truth: mirroring is strictly
//! best-effort and never fails a save.

use std::path::PathBuf;

use tracing::{debug, warn};

use super::metadata;
use super::model::{Note, NoteId};

/// Directory holding mirrored markdown files.
pub(crate) fn notes_mirror_dir() -> PathBuf {
    if let Ok(path) = std::env::var("SCRIPT_KIT_TEST_NOTES_MIRROR_DIR") {
        return PathBuf::from(path);
    }

    if cfg!(test) {
        return std::env::temp_dir()
            .join("script-kit-gpui-tests")
            .join(std::process::id().to_string())
            .join("notes-mirror");
    }

    dirs::home_dir()
        .map(|h| h.join(".scriptkit"))
        .unwrap_or_else(|| PathBuf::from(".scriptkit"))
        .join("notes")
}

/// Short id suffix that keeps mirror filenames stable and collision-free.
fn id_suffix(id: NoteId) -> String {
    id.as_str().chars().filter(|c| *c != '-').take(8).collect()
}

fn mirror_file_name(note: &Note) -> String {
    let slug = metadata::slugify_note_ref(&note.title);
    let slug = if slug.is_empty() {
        "untitled".to_string()
    } else {
        slug
    };
    format!("{}-{}.md", slug, id_suffix(note.id))
}

/// Remove any mirror files for this note id (used on delete and on rename,
/// where the slug-based filename changes).
fn remove_mirror_files(id: NoteId, keep: Option<&str>) {
    let dir = notes_mirror_dir();
    let suffix = format!("-{}.md", id_suffix(id));
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if !name.ends_with(&suffix) {
            continue;
        }
        if keep == Some(name) {
            continue;
        }
        if let Err(error) = std::fs::remove_file(entry.path()) {
            warn!(%error, file = name, "Failed to remove stale note mirror file");
        }
    }
}

/// Write (or delete) the markdown mirror for a note after a save.
///
/// Soft-deleted notes have their mirror removed; trash lives only in SQLite.
pub(crate) fn mirror_note_save(note: &Note) {
    if note.deleted_at.is_some() {
        remove_mirror_files(note.id, None);
        return;
    }

    let dir = notes_mirror_dir();
    if let Err(error) = std::fs::create_dir_all(&dir) {
        warn!(%error, "Failed to create notes mirror directory");
        return;
    }

    let file_name = mirror_file_name(note);
    let path = dir.join(&file_name);
    let tmp = dir.join(format!(".{file_name}.tmp"));

    let write_result =
        std::fs::write(&tmp, note.content.as_bytes()).and_then(|()| std::fs::rename(&tmp, &path));
    match write_result {
        Ok(()) => {
            debug!(note_id = %note.id, file = %file_name, "Note mirrored to markdown file");
            remove_mirror_files(note.id, Some(&file_name));
        }
        Err(error) => {
            let _ = std::fs::remove_file(&tmp);
            warn!(%error, note_id = %note.id, "Failed to mirror note to markdown file");
        }
    }
}

/// Remove the mirror file for a permanently deleted note.
pub(crate) fn mirror_note_delete(id: NoteId) {
    remove_mirror_files(id, None);
}

/// Mirror all active notes once per process, to backfill notes saved before
/// the mirror existed (or edited while it was unavailable).
pub(crate) fn mirror_all_active_notes(notes: &[Note]) {
    static BACKFILL_DONE: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    if BACKFILL_DONE.set(()).is_err() {
        return;
    }
    for note in notes {
        if note.deleted_at.is_none() {
            mirror_note_save(note);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn test_note(title: &str, content: &str) -> Note {
        Note {
            id: NoteId::new(),
            title: title.to_string(),
            content: content.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            is_pinned: false,
            sort_order: 0,
        }
    }

    #[test]
    fn mirror_save_writes_then_rename_replaces_file() {
        let mut note = test_note("Mirror Test", "hello mirror");
        mirror_note_save(&note);

        let dir = notes_mirror_dir();
        let first = dir.join(mirror_file_name(&note));
        assert_eq!(std::fs::read_to_string(&first).unwrap(), "hello mirror");

        note.title = "Renamed Mirror Test".to_string();
        note.content = "renamed body".to_string();
        mirror_note_save(&note);

        let second = dir.join(mirror_file_name(&note));
        assert_eq!(std::fs::read_to_string(&second).unwrap(), "renamed body");
        assert!(!first.exists(), "stale slug file should be removed");

        mirror_note_delete(note.id);
        assert!(!second.exists());
    }

    #[test]
    fn mirror_removes_file_on_soft_delete() {
        let mut note = test_note("Soft Delete Mirror", "body");
        mirror_note_save(&note);
        let path = notes_mirror_dir().join(mirror_file_name(&note));
        assert!(path.exists());

        note.deleted_at = Some(Utc::now());
        mirror_note_save(&note);
        assert!(!path.exists());
    }
}
