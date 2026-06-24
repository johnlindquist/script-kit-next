//! Bind, save, and external-refresh rules for `brain/days/YYYY-MM-DD.md`.
//!
//! ## External append while the Day Page is open
//!
//! When another writer appends to today's file (for example `;todo`), this
//! session polls the on-disk modification time on [`maybe_refresh_from_disk`].
//! If the editor buffer is **clean** (`!dirty`), the file is re-read and the
//! editor content is replaced. If the buffer is **dirty**, the append still
//! lands in the file but the open editor keeps the in-progress edit; the file
//! on disk is refreshed on the next [`bind_today`] (re-entry or day rollover),
//! which saves any dirty buffer first then reloads from disk.

use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use anyhow::{Context as _, Result};
use chrono::{DateTime, NaiveDate, Utc};

use crate::brain::substrate::{io, BrainSubstrate};

/// Which markdown file the Day Page editor is bound to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DayPageBinding {
    Day,
    Note {
        note_id: String,
        title: String,
        return_day_path: PathBuf,
        return_day_date: NaiveDate,
    },
    Fragment {
        fragment_path: PathBuf,
        return_day_path: PathBuf,
        return_day_date: NaiveDate,
    },
}

/// Substrate-backed session for a single day-page markdown file.
#[derive(Debug, Clone)]
pub struct DayPageDocumentSession {
    substrate: BrainSubstrate,
    bound_date: Option<NaiveDate>,
    path: Option<PathBuf>,
    binding: DayPageBinding,
    dirty: bool,
    disk_content: String,
    last_mtime: Option<SystemTime>,
}

impl DayPageDocumentSession {
    pub fn new(substrate: BrainSubstrate) -> Self {
        Self {
            substrate,
            bound_date: None,
            path: None,
            binding: DayPageBinding::Day,
            dirty: false,
            disk_content: String::new(),
            last_mtime: None,
        }
    }

    pub fn binding(&self) -> &DayPageBinding {
        &self.binding
    }

    pub fn is_viewing_fragment(&self) -> bool {
        matches!(self.binding, DayPageBinding::Fragment { .. })
    }

    pub fn is_viewing_note(&self) -> bool {
        matches!(self.binding, DayPageBinding::Note { .. })
    }

    pub fn viewing_note_id(&self) -> Option<&str> {
        match &self.binding {
            DayPageBinding::Note { note_id, .. } => Some(note_id.as_str()),
            _ => None,
        }
    }

    pub fn viewing_note_title(&self) -> Option<&str> {
        match &self.binding {
            DayPageBinding::Note { title, .. } => Some(title.as_str()),
            _ => None,
        }
    }

    pub fn substrate(&self) -> &BrainSubstrate {
        &self.substrate
    }

    pub fn bound_date(&self) -> Option<NaiveDate> {
        self.bound_date
    }

    pub fn path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn disk_content(&self) -> &str {
        &self.disk_content
    }

    /// Bind to today's day page for `now` (timezone from substrate). Creates an
    /// empty file when missing. Day rollover binds a new path when the local
    /// date changes.
    pub fn bind_today(&mut self, now: DateTime<Utc>) -> Result<String> {
        let date = now.with_timezone(&self.substrate.timezone()).date_naive();
        self.bind_date(date, now)
    }

    pub fn bind_date(&mut self, date: NaiveDate, now: DateTime<Utc>) -> Result<String> {
        if self.dirty {
            match self.bound_date {
                // Same-day re-bind refreshes from disk so external appends (e.g. `;todo`)
                // appear even when the user had an in-progress edit.
                Some(bound) if bound == date => {}
                _ => self.save(now)?,
            }
        }

        let path = self.substrate.paths().day_page(date);
        if !path.exists() {
            let parent = path
                .parent()
                .with_context(|| format!("day page path has no parent: {}", path.display()))?;
            fs::create_dir_all(parent)
                .with_context(|| format!("creating days dir {}", parent.display()))?;
            io::atomic_write(&path, "")
                .with_context(|| format!("creating day page {}", path.display()))?;
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("reading day page {}", path.display()))?;
        let mtime = fs::metadata(&path).and_then(|meta| meta.modified()).ok();

        self.bound_date = Some(date);
        self.path = Some(path);
        self.binding = DayPageBinding::Day;
        self.dirty = false;
        self.disk_content = content.clone();
        self.last_mtime = mtime;

        Ok(content)
    }

    /// Bind the editor to a fragment file opened from today's day page.
    pub fn bind_fragment(&mut self, fragment_path: PathBuf, now: DateTime<Utc>) -> Result<String> {
        let day_path = self
            .path
            .clone()
            .with_context(|| "fragment open without day bind")?;
        let day_date = self
            .bound_date
            .with_context(|| "fragment open without day date")?;

        if self.dirty {
            self.save(now)?;
        }

        let content = fs::read_to_string(&fragment_path)
            .with_context(|| format!("reading fragment {}", fragment_path.display()))?;
        let mtime = fs::metadata(&fragment_path)
            .and_then(|meta| meta.modified())
            .ok();

        self.path = Some(fragment_path.clone());
        self.binding = DayPageBinding::Fragment {
            fragment_path,
            return_day_path: day_path,
            return_day_date: day_date,
        };
        self.dirty = false;
        self.disk_content = content.clone();
        self.last_mtime = mtime;

        Ok(content)
    }

    fn return_day_anchor(&self, context: &'static str) -> Result<(PathBuf, NaiveDate)> {
        match &self.binding {
            DayPageBinding::Day => {
                let path = self
                    .path
                    .clone()
                    .with_context(|| format!("{context} without day bind"))?;
                let date = self
                    .bound_date
                    .with_context(|| format!("{context} without day date"))?;
                Ok((path, date))
            }
            DayPageBinding::Note {
                return_day_path,
                return_day_date,
                ..
            }
            | DayPageBinding::Fragment {
                return_day_path,
                return_day_date,
                ..
            } => Ok((return_day_path.clone(), *return_day_date)),
        }
    }

    /// Bind the editor to a regular Notes document while keeping the Day Page
    /// surface local to the main window.
    pub fn bind_note_content(
        &mut self,
        note_id: String,
        title: String,
        content: String,
        path: Option<PathBuf>,
        now: DateTime<Utc>,
    ) -> Result<String> {
        let (day_path, day_date) = self.return_day_anchor("note open")?;

        if self.dirty {
            self.save(now)?;
        }

        let mtime = path
            .as_ref()
            .and_then(|path| fs::metadata(path).and_then(|meta| meta.modified()).ok());

        self.path = path;
        self.bound_date = None;
        self.binding = DayPageBinding::Note {
            note_id,
            title: if title.trim().is_empty() {
                "Untitled Note".to_string()
            } else {
                title
            },
            return_day_path: day_path,
            return_day_date: day_date,
        };
        self.dirty = false;
        self.disk_content = content.clone();
        self.last_mtime = mtime;

        Ok(content)
    }

    /// Return from an inline fragment view back to the bound day page.
    pub fn return_to_day(&mut self, now: DateTime<Utc>) -> Result<String> {
        let (return_day_path, return_day_date) = match &self.binding {
            DayPageBinding::Fragment {
                return_day_path,
                return_day_date,
                ..
            }
            | DayPageBinding::Note {
                return_day_path,
                return_day_date,
                ..
            } => (return_day_path.clone(), *return_day_date),
            DayPageBinding::Day => return self.bind_today(now),
        };

        if self.dirty {
            self.save(now)?;
        }

        let content = fs::read_to_string(&return_day_path)
            .with_context(|| format!("reading day page {}", return_day_path.display()))?;
        let mtime = fs::metadata(&return_day_path)
            .and_then(|meta| meta.modified())
            .ok();

        self.path = Some(return_day_path);
        self.bound_date = Some(return_day_date);
        self.binding = DayPageBinding::Day;
        self.dirty = false;
        self.disk_content = content.clone();
        self.last_mtime = mtime;

        Ok(content)
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Write `content` through the substrate atomic writer when dirty.
    pub fn save_content(&mut self, content: &str, now: DateTime<Utc>) -> Result<()> {
        if !self.dirty {
            return Ok(());
        }

        let note_binding_id = match &self.binding {
            DayPageBinding::Note { note_id, .. } => Some(note_id.clone()),
            _ => None,
        };

        if let Some(note_id_text) = note_binding_id {
            let note_id = crate::notes::NoteId::parse(&note_id_text)
                .with_context(|| format!("parsing day page note id {note_id_text}"))?;
            crate::notes::init_notes_db().context("initializing notes before day page save")?;
            let mut note = crate::notes::get_note(note_id)?
                .with_context(|| format!("loading note before day page save {note_id}"))?;
            note.set_content(content);
            crate::notes::save_note(&note)
                .with_context(|| format!("saving note from day page {note_id}"))?;
            let saved_path = crate::notes::note_file_path(note.id)?
                .with_context(|| format!("resolving saved note path {note_id}"))?;
            if let DayPageBinding::Note { title, .. } = &mut self.binding {
                *title = note.title.clone();
            }
            self.path = Some(saved_path.clone());
            self.dirty = false;
            self.disk_content = content.to_string();
            self.last_mtime = fs::metadata(&saved_path)
                .and_then(|meta| meta.modified())
                .ok();
            let _ = now;
            return Ok(());
        }

        let path = self
            .path
            .clone()
            .with_context(|| "day page save without bind")?;

        io::atomic_write(&path, content)
            .with_context(|| format!("writing day page {}", path.display()))?;

        self.dirty = false;
        self.disk_content = content.to_string();
        self.last_mtime = fs::metadata(&path).and_then(|meta| meta.modified()).ok();

        let _ = now;
        Ok(())
    }

    pub fn save(&mut self, now: DateTime<Utc>) -> Result<()> {
        if !self.dirty {
            return Ok(());
        }
        let content = self.disk_content.clone();
        self.save_content(&content, now)
    }

    /// Apply editor buffer; marks the session dirty when content diverges from disk.
    pub fn apply_editor_content(&mut self, content: &str) {
        if content != self.disk_content {
            self.disk_content = content.to_string();
            self.dirty = true;
        }
    }

    /// Align the session with authoritative on-disk content after an external
    /// append (for example hold-to-talk dictation via substrate `append_to_day`).
    pub fn adopt_disk_content_after_external_write(&mut self, content: String) -> Result<()> {
        let path = self
            .path
            .clone()
            .with_context(|| "adopt disk content without bind")?;
        self.disk_content = content;
        self.dirty = false;
        self.last_mtime = fs::metadata(&path).and_then(|meta| meta.modified()).ok();
        Ok(())
    }

    /// Re-read from disk when the file changed externally and the buffer is clean.
    pub fn maybe_refresh_from_disk(&mut self) -> Result<Option<String>> {
        let path = match self.path.as_ref() {
            Some(path) => path.clone(),
            None => return Ok(None),
        };

        let mtime = fs::metadata(&path).and_then(|meta| meta.modified()).ok();
        if mtime == self.last_mtime {
            return Ok(None);
        }

        if self.dirty {
            return Ok(None);
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("re-reading day page {}", path.display()))?;
        self.disk_content = content.clone();
        self.last_mtime = mtime;
        self.dirty = false;

        Ok(Some(content))
    }

    /// Simulate an external append for tests — updates mtime without going through the session.
    pub fn simulate_external_append_for_test(&mut self, line: &str) -> Result<()> {
        let path = self
            .path
            .clone()
            .with_context(|| "external append without bind")?;
        io::atomic_append_line(&path, line)?;
        Ok(())
    }

    /// Append to the currently bound day file without mutating the in-memory
    /// editor session. Hosts use this for external writers that should be
    /// picked up by `maybe_refresh_from_disk` after the editor surface returns.
    pub fn append_external_line_to_bound_file(&self, line: &str) -> Result<()> {
        let path = self
            .path
            .clone()
            .with_context(|| "external append without bind")?;
        io::atomic_append_line(&path, line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono_tz::Tz;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_session() -> (tempfile::TempDir, DayPageDocumentSession) {
        let dir = tempfile::tempdir().expect("tempdir");
        let substrate = BrainSubstrate::with_timezone(dir.path().join("brain"), Tz::UTC);
        (dir, DayPageDocumentSession::new(substrate))
    }

    fn utc(now: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(now)
            .expect("parse time")
            .with_timezone(&Utc)
    }

    fn notes_test_guard() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn unique_note_content(label: &str) -> String {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .unwrap_or(0);
        format!("# {label} {millis}\n\nbody {}", crate::notes::NoteId::new())
    }

    fn saved_note(content: &str) -> crate::notes::Note {
        let note = crate::notes::Note::with_content(content);
        crate::notes::save_note(&note).expect("save note");
        note
    }

    #[test]
    fn bind_today_creates_file() {
        let (_dir, mut session) = test_session();
        let now = utc("2026-06-11T09:42:00Z");

        let content = session.bind_today(now).expect("bind");
        assert_eq!(content, "");
        assert!(session.path().expect("path").exists());
        assert_eq!(session.bound_date(), Some(now.date_naive()));
    }

    #[test]
    fn save_persists_through_substrate() {
        let (_dir, mut session) = test_session();
        let now = utc("2026-06-11T09:42:00Z");
        session.bind_today(now).expect("bind");

        session.apply_editor_content("morning thought");
        session.save_content("morning thought", now).expect("save");

        let disk = fs::read_to_string(session.path().expect("path")).expect("read");
        assert_eq!(disk, "morning thought");
        assert!(!session.is_dirty());
    }

    #[test]
    fn day_rollover_binds_new_file() {
        let (_dir, mut session) = test_session();
        let day_one = utc("2026-06-11T23:59:00Z");
        let day_two = utc("2026-06-12T00:01:00Z");

        session.bind_today(day_one).expect("bind day one");
        session.apply_editor_content("june 11");
        session.save_content("june 11", day_one).expect("save");

        let content = session.bind_today(day_two).expect("bind day two");
        assert_eq!(content, "");
        assert_eq!(session.bound_date(), Some(day_two.date_naive()));
        assert_ne!(
            session.path().expect("path"),
            &session.substrate().paths().day_page(day_one.date_naive())
        );
    }

    #[test]
    fn binding_different_day_saves_dirty_current_day_first() {
        let (_dir, mut session) = test_session();
        let day_one = utc("2026-06-11T09:42:00Z");
        let day_two = utc("2026-06-12T09:42:00Z");
        session.bind_today(day_one).expect("bind day one");
        let day_one_path = session.path().expect("day one path").clone();

        session.apply_editor_content("dirty before day switch");
        session
            .bind_today(day_two)
            .expect("switching days saves dirty original");

        assert_eq!(
            fs::read_to_string(day_one_path).expect("read original day"),
            "dirty before day switch"
        );
        assert_eq!(session.bound_date(), Some(day_two.date_naive()));
        assert!(!session.is_dirty());
    }

    #[test]
    fn binding_regular_note_saves_dirty_day_before_local_switch() {
        let (dir, mut session) = test_session();
        let now = utc("2026-06-11T09:42:00Z");
        session.bind_today(now).expect("bind day");
        let day_path = session.path().expect("day path").clone();
        let note_path = dir.path().join("note.md");
        fs::write(&note_path, "note body").expect("write note path");

        session.apply_editor_content("dirty before note switch");
        session
            .bind_note_content(
                "note-id".to_string(),
                "Note Title".to_string(),
                "note body".to_string(),
                Some(note_path),
                now,
            )
            .expect("switch to note");

        assert_eq!(
            fs::read_to_string(day_path).expect("read day"),
            "dirty before note switch"
        );
        assert!(session.is_viewing_note());
        assert!(!session.is_dirty());
    }

    #[test]
    fn regular_note_save_uses_notes_storage_and_updates_index() {
        let _guard = notes_test_guard();
        crate::notes::init_notes_db().expect("init notes db");
        let (_dir, mut session) = test_session();
        let now = utc("2026-06-11T09:42:00Z");
        session.bind_today(now).expect("bind day");

        let note = saved_note(&unique_note_content("day-page-save-original"));
        let note_path = crate::notes::note_file_path(note.id)
            .expect("note path result")
            .expect("note path");
        session
            .bind_note_content(
                note.id.as_str().to_string(),
                note.title.clone(),
                note.content.clone(),
                Some(note_path.clone()),
                now,
            )
            .expect("bind note");

        let updated = unique_note_content("day-page-save-updated");
        session.apply_editor_content(&updated);
        session.save_content(&updated, now).expect("save note");

        let stored = crate::notes::get_note(note.id)
            .expect("get note")
            .expect("stored note");
        assert_eq!(stored.content, updated);
        assert_eq!(
            stored.title,
            session.viewing_note_title().unwrap_or_default()
        );
        assert!(
            fs::read_to_string(note_path)
                .expect("read canonical note")
                .contains(&updated),
            "canonical note file should contain updated body"
        );
        assert!(!session.is_dirty());
    }

    #[test]
    fn binding_second_regular_note_preserves_original_return_day_anchor() {
        let _guard = notes_test_guard();
        crate::notes::init_notes_db().expect("init notes db");
        let (_dir, mut session) = test_session();
        let now = utc("2026-06-11T09:42:00Z");
        session.bind_today(now).expect("bind day");
        let original_day_path = session.path().expect("day path").clone();
        let original_day_date = session.bound_date().expect("day date");

        let note_a = saved_note(&unique_note_content("anchor-note-a"));
        let note_b = saved_note(&unique_note_content("anchor-note-b"));
        let note_a_path = crate::notes::note_file_path(note_a.id)
            .expect("note a path result")
            .expect("note a path");
        let note_b_path = crate::notes::note_file_path(note_b.id)
            .expect("note b path result")
            .expect("note b path");

        session
            .bind_note_content(
                note_a.id.as_str().to_string(),
                note_a.title.clone(),
                note_a.content.clone(),
                Some(note_a_path),
                now,
            )
            .expect("bind note a");
        let edited_a = unique_note_content("anchor-note-a-edited");
        session.apply_editor_content(&edited_a);
        session
            .bind_note_content(
                note_b.id.as_str().to_string(),
                note_b.title.clone(),
                note_b.content.clone(),
                Some(note_b_path),
                now,
            )
            .expect("switch note a to note b");

        let stored_a = crate::notes::get_note(note_a.id)
            .expect("get note a")
            .expect("stored note a");
        assert_eq!(stored_a.content, edited_a);

        session.return_to_day(now).expect("return to day");
        assert_eq!(session.path(), Some(&original_day_path));
        assert_eq!(session.bound_date(), Some(original_day_date));
        assert!(!session.is_dirty());
    }

    #[test]
    fn failed_regular_note_save_preserves_dirty_state() {
        let (_dir, mut session) = test_session();
        let now = utc("2026-06-11T09:42:00Z");
        session.bind_today(now).expect("bind day");
        session
            .bind_note_content(
                "not-a-note-id".to_string(),
                "Missing Note".to_string(),
                "stale body".to_string(),
                None,
                now,
            )
            .expect("bind synthetic note");

        session.apply_editor_content("dirty missing note");
        let error = session
            .save_content("dirty missing note", now)
            .expect_err("missing note save should fail");

        assert!(
            error.to_string().contains("parsing day page note id"),
            "{error:#}"
        );
        assert!(session.is_dirty());
    }

    #[test]
    fn external_append_refreshes_when_clean() {
        let (_dir, mut session) = test_session();
        let now = utc("2026-06-11T09:42:00Z");
        session.bind_today(now).expect("bind");

        session
            .simulate_external_append_for_test("- [ ] buy milk")
            .expect("append");

        let refreshed = session
            .maybe_refresh_from_disk()
            .expect("refresh")
            .expect("should refresh");
        assert!(refreshed.contains("buy milk"));
    }

    #[test]
    fn external_append_targets_bound_day_not_wall_clock_today() {
        let (_dir, mut session) = test_session();
        let bound_day = utc("2026-06-11T09:42:00Z");
        let later_day = utc("2026-06-12T09:42:00Z");
        session.bind_today(bound_day).expect("bind");
        let bound_path = session.path().expect("bound path").clone();
        let later_path = session.substrate().paths().day_page(later_day.date_naive());

        session
            .append_external_line_to_bound_file("09:43 Agent Chat\n\nkeep this")
            .expect("append to bound day");

        assert!(fs::read_to_string(&bound_path)
            .expect("read bound day")
            .contains("keep this"));
        assert!(
            !later_path.exists(),
            "external append must not choose a new path from wall-clock today"
        );
    }

    #[test]
    fn external_append_skipped_when_dirty_until_rebind() {
        let (_dir, mut session) = test_session();
        let now = utc("2026-06-11T09:42:00Z");
        session.bind_today(now).expect("bind");
        session.apply_editor_content("typing...");

        session
            .simulate_external_append_for_test("- [ ] buy milk")
            .expect("append");
        assert!(session
            .maybe_refresh_from_disk()
            .expect("refresh")
            .is_none());

        let rebound = session.bind_today(now).expect("rebind");
        assert!(rebound.contains("buy milk"));
        assert!(!rebound.contains("typing..."));
    }

    /// View-entry contract: bind creates today's file, typed content persists, rollover binds anew.
    #[test]
    fn entering_day_page_view_creates_binds_and_rollovers() {
        let (_dir, mut session) = test_session();
        let now = utc("2026-06-11T09:42:00Z");

        let content = session.bind_today(now).expect("bind creates today");
        assert_eq!(content, "");
        assert!(session.path().expect("path").exists());

        session.apply_editor_content("typed on today's page");
        session
            .save_content("typed on today's page", now)
            .expect("save");
        let disk = fs::read_to_string(session.path().expect("path")).expect("read");
        assert_eq!(disk, "typed on today's page");

        let next_day = utc("2026-06-12T00:05:00Z");
        let rebound = session.bind_today(next_day).expect("rollover bind");
        assert_eq!(rebound, "");
        assert_eq!(session.bound_date(), Some(next_day.date_naive()));
        assert_ne!(
            session.path().expect("path"),
            &session.substrate().paths().day_page(now.date_naive())
        );
    }
}
