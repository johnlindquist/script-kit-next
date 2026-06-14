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

    /// Return from an inline fragment view back to the bound day page.
    pub fn return_to_day(&mut self, now: DateTime<Utc>) -> Result<String> {
        let (return_day_path, return_day_date) = match &self.binding {
            DayPageBinding::Fragment {
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
