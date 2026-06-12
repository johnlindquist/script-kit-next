//! Brain filesystem substrate — canonical markdown under `~/.scriptkit/brain/`.
//!
//! This module owns every path under `brain/{days,fragments,notes,trash}`.
//! Callers must not construct those locations directly.

mod day;
mod fragment;
mod frontmatter;
pub(crate) mod io;
mod paths;
mod slug;
mod trash;
mod words;

use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use chrono_tz::Tz;

pub use day::DayEntry;
pub use fragment::{FragmentReference, FRAGMENT_EXCERPT_WORDS, FRAGMENT_WORD_THRESHOLD};
pub use frontmatter::BrainFrontmatter;
pub use paths::BrainPaths;
pub use slug::{dedupe_slug_in_dir, slugify, source_slug};
pub use trash::{restore_file, trash_file};
pub use words::{excerpt_words, word_count};

/// Which brain subdirectory a slug allocation targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrainSlugDir {
    Notes,
    Fragments,
}

/// Entry point for brain filesystem operations.
#[derive(Debug, Clone)]
pub struct BrainSubstrate {
    paths: BrainPaths,
    tz: Tz,
}

impl BrainSubstrate {
    pub fn new(base: impl AsRef<Path>) -> Self {
        Self {
            paths: BrainPaths::new(base),
            tz: chrono_tz::UTC,
        }
    }

    pub fn with_timezone(base: impl AsRef<Path>, tz: Tz) -> Self {
        Self {
            paths: BrainPaths::new(base),
            tz,
        }
    }

    pub fn default_kit() -> Self {
        Self::new(BrainPaths::default_kit().base())
    }

    pub fn paths(&self) -> &BrainPaths {
        &self.paths
    }

    pub fn timezone(&self) -> Tz {
        self.tz
    }

    /// Append a timestamped entry to today's day page. Append-only — earlier
    /// content is never rewritten.
    pub fn append_to_day(&self, now: DateTime<Utc>, entry: DayEntry) -> Result<()> {
        let (date, timestamp) = day::local_day_and_time(now, self.tz);
        let path = self.paths.day_page(date);
        let line = entry.format_line(&timestamp);
        io::atomic_append_line(&path, &line)
    }

    /// Write a long capture as a fragment when it exceeds
    /// [`FRAGMENT_WORD_THRESHOLD`] words. Returns excerpt + relative link for a
    /// [`DayEntry::FragmentRef`] line.
    pub fn write_fragment(
        &self,
        now: DateTime<Utc>,
        source_label: &str,
        source_uri: &str,
        content: &str,
    ) -> Result<Option<FragmentReference>> {
        fragment::write_fragment(&self.paths, now, self.tz, source_label, source_uri, content)
    }

    /// Write a long capture as a fragment, optionally attaching a post-copy why.
    pub fn write_fragment_with_why(
        &self,
        now: DateTime<Utc>,
        source_label: &str,
        source_uri: &str,
        content: &str,
        why: Option<&str>,
    ) -> Result<Option<FragmentReference>> {
        fragment::write_fragment_with_why(
            &self.paths,
            now,
            self.tz,
            source_label,
            source_uri,
            content,
            why,
        )
    }

    /// Remove sediment lines written for a clipboard entry (T12 reject undo).
    pub fn undo_clipboard_sediment_lines(
        &self,
        now: DateTime<Utc>,
        entry_id: &str,
        text: &str,
        kept_url_day: Option<&str>,
        brain_kept: bool,
    ) -> Result<()> {
        day::undo_clipboard_sediment_lines(
            &self.paths,
            self.tz,
            now,
            entry_id,
            text,
            kept_url_day,
            brain_kept,
        )
    }

    /// Serialize a brain document with frontmatter and write it atomically.
    pub fn write_document(
        &self,
        path: &Path,
        frontmatter: &BrainFrontmatter,
        body: &str,
    ) -> Result<()> {
        if !self.paths.contains(path) {
            bail!("refusing to write outside brain tree: {}", path.display());
        }
        let document = frontmatter.render(body);
        io::atomic_write(path, &document)
    }

    /// Parse a brain markdown document into frontmatter and body.
    pub fn parse_document(&self, content: &str) -> Result<(BrainFrontmatter, String)> {
        BrainFrontmatter::parse(content)
    }

    /// Allocate a unique lowercase hyphenated slug in the given directory.
    pub fn allocate_slug(&self, base: &str, dir: BrainSlugDir) -> String {
        let parent = match dir {
            BrainSlugDir::Notes => self.paths.notes_dir(),
            BrainSlugDir::Fragments => self.paths.fragments_dir(),
        };
        dedupe_slug_in_dir(&parent, base)
    }

    /// Move a brain file into `brain/trash/`.
    pub fn trash(&self, path: &Path) -> Result<PathBuf> {
        trash_file(&self.paths, path)
    }

    /// Restore a trashed file to its original location.
    pub fn restore(&self, trashed: &Path, destination: &Path) -> Result<()> {
        restore_file(&self.paths, trashed, destination)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone as _;
    use std::fs;
    use std::thread;
    use std::time::Duration;

    use crate::notes::NoteId;

    fn test_substrate() -> (tempfile::TempDir, BrainSubstrate) {
        let dir = tempfile::tempdir().expect("tempdir");
        let substrate = BrainSubstrate::with_timezone(dir.path().join("brain"), chrono_tz::UTC);
        (dir, substrate)
    }

    fn fixed_now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 11, 9, 42, 0).unwrap()
    }

    #[test]
    fn append_to_day_preserves_order_and_timestamps() {
        let (_dir, substrate) = test_substrate();
        let base = fixed_now();

        substrate
            .append_to_day(
                base,
                DayEntry::Capture {
                    text: "first capture".to_string(),
                },
            )
            .expect("first append");
        substrate
            .append_to_day(
                base + chrono::Duration::minutes(3),
                DayEntry::Task {
                    body: "buy milk".to_string(),
                    tags: vec!["errand".to_string()],
                    due: Some("2026-06-12".to_string()),
                },
            )
            .expect("second append");
        substrate
            .append_to_day(
                base + chrono::Duration::minutes(8),
                DayEntry::KeptUrl {
                    url: "https://example.com".to_string(),
                },
            )
            .expect("third append");

        let path = substrate.paths().day_page(base.date_naive());
        let contents = fs::read_to_string(path).expect("read day page");
        let lines: Vec<&str> = contents.lines().collect();

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "09:42 first capture");
        assert_eq!(lines[1], "09:45 - [ ] buy milk #errand due:2026-06-12");
        assert_eq!(lines[2], "09:50 https://example.com");
    }

    #[test]
    fn append_to_day_is_append_only() {
        let (_dir, substrate) = test_substrate();
        let now = fixed_now();

        substrate
            .append_to_day(
                now,
                DayEntry::Capture {
                    text: "original".to_string(),
                },
            )
            .expect("first append");
        substrate
            .append_to_day(
                now + chrono::Duration::minutes(1),
                DayEntry::Capture {
                    text: "second".to_string(),
                },
            )
            .expect("second append");

        let path = substrate.paths().day_page(now.date_naive());
        let contents = fs::read_to_string(path).expect("read day page");
        assert!(contents.contains("09:42 original"));
        assert!(contents.starts_with("09:42 original"));
        assert!(contents.contains("09:43 second"));
    }

    #[test]
    fn fragment_writer_respects_threshold_and_excerpt() {
        let (_dir, substrate) = test_substrate();
        let now = fixed_now();
        let short = "short capture".to_string();
        let long = (0..250)
            .map(|index| format!("word{index}"))
            .collect::<Vec<_>>()
            .join(" ");

        let none = substrate
            .write_fragment(now, "clipboard", "scriptkit://clipboard/entry-1", &short)
            .expect("short fragment write");
        assert!(none.is_none());

        let reference = substrate
            .write_fragment(now, "Slack Paste", "scriptkit://clipboard/entry-2", &long)
            .expect("long fragment write")
            .expect("fragment reference");

        assert_eq!(
            reference.relative_link,
            "../fragments/2026-06-11-0942-slack-paste.md"
        );
        assert!(reference.excerpt.ends_with("..."));
        assert_eq!(reference.excerpt.split_whitespace().count(), 40);

        let fragment_path = substrate
            .paths()
            .fragment_file("2026-06-11-0942-slack-paste");
        let fragment = fs::read_to_string(fragment_path).expect("read fragment");
        assert!(fragment.contains("source: scriptkit://clipboard/entry-2"));
        assert!(fragment.contains("word249"));
    }

    #[test]
    fn slug_dedup_adds_numeric_suffixes() {
        let (_dir, substrate) = test_substrate();
        let notes_dir = substrate.paths().notes_dir();
        fs::create_dir_all(&notes_dir).expect("notes dir");
        fs::write(notes_dir.join("my-note.md"), "existing").expect("seed note");

        assert_eq!(
            substrate.allocate_slug("My Note", BrainSlugDir::Notes),
            "my-note-2"
        );
        fs::write(notes_dir.join("my-note-2.md"), "existing").expect("seed note 2");
        assert_eq!(
            substrate.allocate_slug("My Note", BrainSlugDir::Notes),
            "my-note-3"
        );
    }

    #[test]
    fn frontmatter_round_trip_preserves_fields() {
        let (_dir, substrate) = test_substrate();
        let id = NoteId::parse("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let created = Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap();
        let updated = Utc.with_ymd_and_hms(2026, 6, 7, 8, 9, 10).unwrap();
        let frontmatter = BrainFrontmatter {
            id,
            created,
            updated,
            tags: vec!["rust".to_string(), "notes/metadata".to_string()],
            aliases: vec!["Plan".to_string()],
            pinned: true,
            source: Some("scriptkit://agent-chat/thread-123".to_string()),
            why: None,
        };

        let rendered = frontmatter.render("# Title\n\nBody text");
        let (parsed, body) = substrate.parse_document(&rendered).expect("parse");

        assert_eq!(parsed, frontmatter);
        assert_eq!(body, "# Title\n\nBody text");
    }

    #[test]
    fn atomic_write_leaves_no_partial_files() {
        let (_dir, substrate) = test_substrate();
        let path = substrate.paths().notes_dir().join("atomic-note.md");
        let frontmatter = BrainFrontmatter::new(NoteId::new(), fixed_now(), fixed_now());

        substrate
            .write_document(&path, &frontmatter, "complete body")
            .expect("atomic write");

        let parent = path.parent().expect("parent");
        let leftovers: Vec<_> = fs::read_dir(parent)
            .expect("read dir")
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .contains(".brain-write.")
            })
            .collect();
        assert!(leftovers.is_empty());

        let contents = fs::read_to_string(path).expect("read note");
        assert!(contents.contains("complete body"));
        assert!(contents.starts_with("---\n"));
    }

    #[test]
    fn trash_and_restore_round_trip() {
        let (_dir, substrate) = test_substrate();
        let note_path = substrate.paths().note_file("restore-me");
        let frontmatter = BrainFrontmatter::new(NoteId::new(), fixed_now(), fixed_now());
        substrate
            .write_document(&note_path, &frontmatter, "keep me")
            .expect("write note");

        let trashed = substrate.trash(&note_path).expect("trash");
        assert!(!note_path.exists());
        assert!(trashed.starts_with(substrate.paths().trash_dir()));

        substrate.restore(&trashed, &note_path).expect("restore");
        assert!(note_path.exists());
        assert!(!trashed.exists());
        let contents = fs::read_to_string(note_path).expect("read restored");
        assert!(contents.contains("keep me"));
    }

    #[test]
    fn trash_collision_adds_timestamp_suffix() {
        let (_dir, substrate) = test_substrate();
        let trash_dir = substrate.paths().trash_dir();
        fs::create_dir_all(&trash_dir).expect("trash dir");
        fs::write(trash_dir.join("collision.md"), "existing").expect("seed trash");

        let note_path = substrate.paths().note_file("collision");
        substrate
            .write_document(
                &note_path,
                &BrainFrontmatter::new(NoteId::new(), fixed_now(), fixed_now()),
                "body",
            )
            .expect("write note");

        let trashed = substrate.trash(&note_path).expect("trash");
        let name = trashed.file_name().unwrap().to_string_lossy();
        assert!(name.starts_with("collision-"));
        assert!(name.ends_with(".md"));
    }

    #[test]
    fn trace_and_fragment_ref_lines_render_expected_markdown() {
        let (_dir, substrate) = test_substrate();
        let now = fixed_now();

        substrate
            .append_to_day(
                now,
                DayEntry::FragmentRef(FragmentReference {
                    excerpt: "First words of the pasted article without cutting mid-word..."
                        .to_string(),
                    relative_link: "../fragments/2026-06-11-0942-clipboard.md".to_string(),
                }),
            )
            .expect("fragment ref append");
        substrate
            .append_to_day(
                now + chrono::Duration::minutes(1),
                DayEntry::Trace {
                    summary: "Agent Chat: flaky clock test".to_string(),
                    provenance_link: "scriptkit://agent-chat/thread-9".to_string(),
                },
            )
            .expect("trace append");

        let contents =
            fs::read_to_string(substrate.paths().day_page(now.date_naive())).expect("read day");
        assert!(contents.contains("> First words of the pasted article"));
        assert!(contents.contains("../fragments/2026-06-11-0942-clipboard.md"));
        assert!(contents
            .contains("09:43 — Agent Chat: flaky clock test (scriptkit://agent-chat/thread-9)"));
    }

    #[test]
    fn excerpt_words_never_cuts_mid_word() {
        let text = "one two three four five";
        assert_eq!(excerpt_words(text, 3), "one two three...");
        assert_eq!(excerpt_words("short", 10), "short");
    }

    #[test]
    fn concurrent_appends_do_not_leave_temp_files() {
        let (_dir, substrate) = test_substrate();
        let now = fixed_now();
        let path = substrate.paths().day_page(now.date_naive());

        for index in 0..5 {
            substrate
                .append_to_day(
                    now + chrono::Duration::seconds(index),
                    DayEntry::Capture {
                        text: format!("capture {index}"),
                    },
                )
                .expect("append");
            thread::sleep(Duration::from_millis(5));
        }

        let contents = fs::read_to_string(&path).expect("read day page");
        for index in 0..5 {
            assert!(contents.contains(&format!("capture {index}")));
        }
    }
}
