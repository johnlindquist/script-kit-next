//! Day Page rows for the Notes Cmd+P switcher.
//!
//! Day pages are plain markdown under `~/.scriptkit/brain/days/` and are
//! owned by the main window's Day Page surface. Notes only *lists* them —
//! read-through from disk on each switcher open, never copied into the notes
//! database — and hands a pick off to the main window. The handoff goes
//! through a binary-registered hook because this dual-compiled module cannot
//! name `ScriptListApp`.

use std::path::Path;

use chrono::NaiveDate;

/// Cap on switcher rows so years of day pages never bloat the popup; the
/// newest pages win and CommandBar search still narrows within them.
pub(crate) const DAY_PAGE_SWITCHER_ROW_LIMIT: usize = 90;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DayPageSwitcherRow {
    pub date: NaiveDate,
    pub title: String,
    pub preview: String,
}

/// Scan `days_dir` for `YYYY-MM-DD.md` files, newest first, with a
/// first-non-empty-line preview. Mirrors the Day Page switcher's own loader
/// so both surfaces describe the same files the same way.
pub(crate) fn load_day_page_switcher_rows(
    days_dir: &Path,
    today: NaiveDate,
    limit: usize,
) -> Vec<DayPageSwitcherRow> {
    let mut rows = Vec::new();
    let Ok(read_dir) = std::fs::read_dir(days_dir) else {
        return rows;
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        let Ok(date) = NaiveDate::parse_from_str(stem, "%Y-%m-%d") else {
            continue;
        };
        let preview = std::fs::read_to_string(&path)
            .ok()
            .and_then(|content| {
                content
                    .lines()
                    .map(str::trim)
                    .find(|line| !line.is_empty())
                    .map(|line| line.chars().take(80).collect::<String>())
            })
            .unwrap_or_default();
        rows.push(DayPageSwitcherRow {
            date,
            title: day_page_switcher_title(date, today),
            preview,
        });
    }
    rows.sort_by(|a, b| b.date.cmp(&a.date));
    rows.truncate(limit);
    rows
}

/// Same label shape as the Day Page's own Cmd+P switcher
/// (`day_switcher_entry_label`): `2026-06-12 · Friday`, with a `Today · `
/// prefix on the current day.
fn day_page_switcher_title(date: NaiveDate, today: NaiveDate) -> String {
    let formatted = date.format("%Y-%m-%d · %A").to_string();
    if date == today {
        format!("Today · {formatted}")
    } else {
        formatted
    }
}

/// Binary-registered hook that opens a day page in the main window's Day
/// Page surface. The main app type lives in the binary crate, so this
/// dual-compiled file cannot downcast the main window's root view itself;
/// the binary registers the closure at app startup.
static OPEN_DAY_PAGE_IN_MAIN_HOOK: std::sync::OnceLock<fn(NaiveDate, &mut gpui::App) -> bool> =
    std::sync::OnceLock::new();

pub fn register_open_day_page_in_main_hook(hook: fn(NaiveDate, &mut gpui::App) -> bool) {
    let _ = OPEN_DAY_PAGE_IN_MAIN_HOOK.set(hook);
}

/// Hand a picked day off to the main window. Returns false when no hook is
/// registered (lib-only contexts) or the main window is unavailable.
pub(crate) fn open_day_page_in_main(date: NaiveDate, cx: &mut gpui::App) -> bool {
    match OPEN_DAY_PAGE_IN_MAIN_HOOK.get() {
        Some(hook) => hook(date, cx),
        None => {
            tracing::warn!(
                target: "script_kit::notes",
                date = %date,
                "notes_day_page_handoff_hook_missing"
            );
            false
        }
    }
}

#[cfg(test)]
mod day_page_rows_tests {
    use super::*;

    fn write_day(dir: &Path, name: &str, content: &str) {
        std::fs::write(dir.join(name), content).expect("write day file");
    }

    #[test]
    fn missing_dir_yields_no_rows() {
        let dir = tempfile::tempdir().expect("tempdir");
        let missing = dir.path().join("nope");
        let today = NaiveDate::from_ymd_opt(2026, 6, 12).unwrap();
        assert!(load_day_page_switcher_rows(&missing, today, 10).is_empty());
    }

    #[test]
    fn rows_are_newest_first_with_previews_and_today_label() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_day(dir.path(), "2026-06-10.md", "\n\n  older entry line\nmore");
        write_day(dir.path(), "2026-06-12.md", "today entry");
        write_day(dir.path(), "2026-06-11.md", "");
        // Junk that must be skipped: wrong extension, non-date stem.
        write_day(dir.path(), "2026-06-09.txt", "not markdown");
        write_day(dir.path(), "notes.md", "not a date");

        let today = NaiveDate::from_ymd_opt(2026, 6, 12).unwrap();
        let rows = load_day_page_switcher_rows(dir.path(), today, 10);

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].date, today);
        assert_eq!(rows[0].title, "Today · 2026-06-12 · Friday");
        assert_eq!(rows[0].preview, "today entry");
        assert_eq!(rows[1].title, "2026-06-11 · Thursday");
        assert_eq!(rows[1].preview, "");
        assert_eq!(rows[2].title, "2026-06-10 · Wednesday");
        assert_eq!(rows[2].preview, "older entry line");
    }

    #[test]
    fn limit_keeps_only_the_newest_rows() {
        let dir = tempfile::tempdir().expect("tempdir");
        for day in 1..=9 {
            write_day(dir.path(), &format!("2026-06-0{day}.md"), "entry");
        }
        let today = NaiveDate::from_ymd_opt(2026, 6, 12).unwrap();
        let rows = load_day_page_switcher_rows(dir.path(), today, 3);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].date, NaiveDate::from_ymd_opt(2026, 6, 9).unwrap());
        assert_eq!(rows[2].date, NaiveDate::from_ymd_opt(2026, 6, 7).unwrap());
    }

    #[test]
    fn long_first_lines_are_truncated_to_80_chars() {
        let dir = tempfile::tempdir().expect("tempdir");
        let long_line = "x".repeat(200);
        write_day(dir.path(), "2026-06-01.md", &long_line);
        let today = NaiveDate::from_ymd_opt(2026, 6, 12).unwrap();
        let rows = load_day_page_switcher_rows(dir.path(), today, 10);
        assert_eq!(rows[0].preview.chars().count(), 80);
    }
}
