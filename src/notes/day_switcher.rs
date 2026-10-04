use chrono::{DateTime, Datelike, NaiveDate, Utc, Weekday};
use std::fs;
use std::path::{Path, PathBuf};

use crate::actions::NoteSwitcherNoteInfo;

const DAY_NOTE_ID_PREFIX: &str = "day:";

#[derive(Debug, Clone)]
pub(crate) struct DayNoteSwitcherEntry {
    pub(crate) date: NaiveDate,
    pub(crate) path: PathBuf,
    pub(crate) title: String,
    pub(crate) content: String,
    pub(crate) updated_at: DateTime<Utc>,
}

pub(crate) fn day_note_action_id(date: NaiveDate) -> String {
    format!("{DAY_NOTE_ID_PREFIX}{date}")
}

pub(crate) fn parse_day_note_action_id(id: &str) -> Option<NaiveDate> {
    id.strip_prefix(DAY_NOTE_ID_PREFIX)
        .and_then(|date| NaiveDate::parse_from_str(date, "%Y-%m-%d").ok())
}

/// Day pages whose date or words match `query`, newest first, as files the chat can attach.
/// Today's page is called "Today".
pub(crate) fn day_page_file_matches(query: &str, limit: usize) -> Vec<crate::scripts::FileMatch> {
    let today = crate::brain::substrate::BrainSubstrate::default_kit()
        .capture_day_destination(Utc::now())
        .0;
    day_page_file_matches_in(
        &load_day_note_switcher_entries(&crate::notes::notes_brain_days_dir()),
        today,
        query,
        limit,
    )
}

/// A date a query names: "today", "yesterday", a weekday, a month, "march 3" / "3 mar", or an
/// ISO date or part of one ("2026-03").
#[derive(Debug, Clone, PartialEq, Eq)]
enum DateTest {
    On(NaiveDate),
    Weekday(Weekday),
    Month(u32),
    MonthDay(u32, u32),
    IsoText(String),
}

impl DateTest {
    fn holds(&self, date: NaiveDate) -> bool {
        match self {
            Self::On(day) => date == *day,
            Self::Weekday(weekday) => date.weekday() == *weekday,
            Self::Month(month) => date.month() == *month,
            Self::MonthDay(month, day) => date.month() == *month && date.day() == *day,
            Self::IsoText(text) => date.to_string().contains(text.as_str()),
        }
    }
}

/// Words that phrase a request about days rather than say what a day page holds.
const DAY_QUERY_FILLER: &[&str] = &[
    "a", "about", "day", "days", "find", "for", "from", "in", "last", "my", "note", "notes", "of",
    "on", "open", "page", "pages", "show", "the", "what", "did", "i", "do", "was", "were",
];

fn weekday_named(word: &str) -> Option<Weekday> {
    Some(match word {
        "monday" | "mon" => Weekday::Mon,
        "tuesday" | "tue" | "tues" => Weekday::Tue,
        "wednesday" | "wed" => Weekday::Wed,
        "thursday" | "thu" | "thur" | "thurs" => Weekday::Thu,
        "friday" | "fri" => Weekday::Fri,
        // "sat" and "sun" are ordinary words too often to read as days.
        "saturday" => Weekday::Sat,
        "sunday" => Weekday::Sun,
        _ => return None,
    })
}

fn month_named(word: &str) -> Option<u32> {
    const MONTHS: [&str; 12] = [
        "january",
        "february",
        "march",
        "april",
        "may",
        "june",
        "july",
        "august",
        "september",
        "october",
        "november",
        "december",
    ];
    if word == "sept" {
        return Some(9);
    }
    MONTHS
        .iter()
        .position(|month| *month == word || (word.len() == 3 && month.starts_with(word)))
        .map(|at| at as u32 + 1)
}

/// "3", "3rd", "21st" as a day of the month.
fn day_of_month(word: &str) -> Option<u32> {
    let digits = word.trim_end_matches(|c: char| c.is_ascii_alphabetic());
    let suffix = &word[digits.len()..];
    if digits.is_empty() || !matches!(suffix, "" | "st" | "nd" | "rd" | "th") {
        return None;
    }
    digits.parse().ok().filter(|day| (1..=31).contains(day))
}

/// What a typed query asks of day pages: the dates it names and the words a page must hold.
/// Both `@notes:` and the Notes list read day pages through this, so "march 3", "mar-3",
/// "tuesday", "yesterday" and "sam invoice" find the same pages everywhere.
#[derive(Debug, Clone, Default)]
pub(crate) struct DayQuery {
    dates: Vec<DateTest>,
    words: Vec<String>,
    everything: bool,
}

impl DayQuery {
    pub(crate) fn parse(query: &str, today: NaiveDate) -> Self {
        let lower = query.trim().to_lowercase();
        let mut parts: Vec<String> = Vec::new();
        let mut dates = Vec::new();
        for token in lower.split(|c: char| c.is_whitespace() || c == ',') {
            let token = token.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
            if token.is_empty() {
                continue;
            }
            if let Ok(date) = NaiveDate::parse_from_str(token, "%Y-%m-%d") {
                dates.push(DateTest::On(date));
            } else if token.contains('-') && token.chars().all(|c| c.is_ascii_digit() || c == '-') {
                dates.push(DateTest::IsoText(token.to_string()));
            } else {
                parts.extend(
                    token
                        .split(|c: char| !c.is_alphanumeric())
                        .filter(|part| !part.is_empty())
                        .map(str::to_string),
                );
            }
        }

        let mut words: Vec<String> = Vec::new();
        let mut at = 0;
        while at < parts.len() {
            let part = parts[at].as_str();
            if part == "today" {
                dates.push(DateTest::On(today));
            } else if part == "yesterday" {
                dates.push(DateTest::On(today.pred_opt().unwrap_or(today)));
            } else if let Some(weekday) = weekday_named(part) {
                dates.push(DateTest::Weekday(weekday));
            } else if let Some(month) = month_named(part) {
                if let Some(day) = parts.get(at + 1).and_then(|next| day_of_month(next)) {
                    dates.push(DateTest::MonthDay(month, day));
                    at += 1;
                } else if let Some(day) = words.last().and_then(|last| day_of_month(last)) {
                    words.pop();
                    dates.push(DateTest::MonthDay(month, day));
                } else {
                    dates.push(DateTest::Month(month));
                }
            } else if !DAY_QUERY_FILLER.contains(&part) {
                words.push(part.to_string());
            }
            at += 1;
        }
        // A lone letter ("c" from "c++") appears on nearly every page.
        words.retain(|word| word.chars().count() > 1);
        Self {
            dates,
            words,
            everything: lower.is_empty(),
        }
    }

    /// The query names dates and nothing else, such as "yesterday" or "notes from march 3".
    pub(crate) fn only_dates(&self) -> bool {
        !self.dates.is_empty() && self.words.is_empty()
    }

    /// Whether `date` is one of the dates the query names (true when it names none).
    pub(crate) fn date_holds(&self, date: NaiveDate) -> bool {
        self.dates.iter().all(|test| test.holds(date))
    }

    pub(crate) fn matches(&self, entry: &DayNoteSwitcherEntry, today: NaiveDate) -> bool {
        if self.everything {
            return true;
        }
        if self.dates.is_empty() && self.words.is_empty() {
            return false;
        }
        if !self.date_holds(entry.date) {
            return false;
        }
        if self.words.is_empty() {
            return true;
        }
        let mut text = format!(
            "{} {} {}",
            entry.title,
            entry.date.format("%b %-d %B"),
            entry.content
        )
        .to_lowercase();
        if entry.date == today {
            text.push_str(" today");
        }
        self.words.iter().all(|word| text.contains(word.as_str()))
    }
}

fn day_page_file_matches_in(
    entries: &[DayNoteSwitcherEntry],
    today: NaiveDate,
    query: &str,
    limit: usize,
) -> Vec<crate::scripts::FileMatch> {
    let day_query = DayQuery::parse(query, today);
    entries
        .iter()
        .filter_map(|entry| {
            let name = if entry.date == today {
                format!("Today · {}", entry.title)
            } else {
                entry.title.clone()
            };
            let matches = day_query.matches(entry, today);
            matches.then(|| crate::scripts::FileMatch {
                file: crate::file_search::FileResult {
                    path: entry.path.to_string_lossy().into_owned(),
                    name,
                    size: entry.content.len() as u64,
                    modified: entry.updated_at.timestamp().max(0) as u64,
                    file_type: crate::file_search::FileType::Document,
                },
                score: 0,
            })
        })
        .take(limit)
        .collect()
}

pub(crate) fn load_day_note_switcher_entries(days_dir: &Path) -> Vec<DayNoteSwitcherEntry> {
    load_day_note_switcher_entries_result(days_dir).unwrap_or_default()
}

/// Load day-note rows without collapsing an IO failure into an empty corpus.
///
/// The compatibility helper above retains the historical best-effort contract
/// for callers that have not adopted typed search state. Canonical Notes search
/// uses this strict result so a failed load remains distinguishable from an
/// honestly empty result set.
pub(crate) fn load_day_note_switcher_entries_result(
    days_dir: &Path,
) -> std::io::Result<Vec<DayNoteSwitcherEntry>> {
    let mut entries = Vec::new();
    if !days_dir.exists() {
        return Ok(entries);
    }
    if let Some(brain_root) = days_dir.parent() {
        crate::atomic_file::ensure_private_directory(brain_root)?;
    }
    crate::atomic_file::ensure_private_directory(days_dir)?;
    let read_dir = match fs::read_dir(days_dir) {
        Ok(read_dir) => read_dir,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(entries),
        Err(error) => return Err(error),
    };

    for entry in read_dir {
        let entry = entry?;
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
        let content = crate::atomic_file::read_private_file(&path)?;
        let updated_at = fs::metadata(&path)
            .and_then(|metadata| metadata.modified())
            .map(DateTime::<Utc>::from)?;
        entries.push(DayNoteSwitcherEntry {
            date,
            path,
            title: day_note_title(date),
            content,
            updated_at,
        });
    }

    entries.sort_by_key(|a| std::cmp::Reverse(a.date));
    Ok(entries)
}

pub(crate) fn day_note_switcher_infos(
    entries: &[DayNoteSwitcherEntry],
    current_date: Option<NaiveDate>,
) -> Vec<NoteSwitcherNoteInfo> {
    entries
        .iter()
        .map(|entry| NoteSwitcherNoteInfo {
            id: day_note_action_id(entry.date),
            title: entry.title.clone(),
            char_count: entry.content.chars().count(),
            is_current: Some(entry.date) == current_date,
            is_pinned: false,
            preview: day_note_preview(&entry.content),
            relative_time: crate::formatting::format_relative_time_short_dt(entry.updated_at),
        })
        .collect()
}

pub(crate) fn day_note_title(date: NaiveDate) -> String {
    format!("{} · {}", date, date.format("%A"))
}

fn day_note_preview(content: &str) -> String {
    content
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or_default()
        .chars()
        .take(100)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn day_note_private_switcher_repairs_legacy_permissions_before_loading() {
        use std::os::unix::fs::PermissionsExt as _;

        let fixture = tempfile::tempdir().expect("isolated day-note fixture");
        let days = fixture.path().join("brain").join("days");
        fs::create_dir_all(&days).unwrap();
        fs::set_permissions(&days, fs::Permissions::from_mode(0o755)).unwrap();
        let day = days.join("2026-08-22.md");
        fs::write(&day, "private searchable day").unwrap();
        fs::set_permissions(&day, fs::Permissions::from_mode(0o644)).unwrap();

        let entries =
            load_day_note_switcher_entries_result(&days).expect("repair private day before search");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].content, "private searchable day");
        assert_eq!(
            fs::metadata(days).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(day).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    #[cfg(unix)]
    #[test]
    fn day_note_private_switcher_refuses_hostile_directory_and_day_symlinks() {
        use std::os::unix::fs::symlink;

        let fixture = tempfile::tempdir().expect("isolated day-note symlink fixture");
        let brain = fixture.path().join("brain");
        fs::create_dir(&brain).unwrap();
        let foreign_dir = fixture.path().join("foreign-days");
        fs::create_dir(&foreign_dir).unwrap();
        fs::write(foreign_dir.join("2026-08-22.md"), "foreign day").unwrap();
        let planted_dir = brain.join("days");
        symlink(&foreign_dir, &planted_dir).expect("plant hostile day directory");
        assert!(load_day_note_switcher_entries_result(&planted_dir).is_err());

        let owned_days = brain.join("safe-days");
        fs::create_dir(&owned_days).unwrap();
        let foreign_file = fixture.path().join("foreign.md");
        fs::write(&foreign_file, "private foreign day text").unwrap();
        symlink(&foreign_file, owned_days.join("2026-08-23.md")).expect("plant hostile day file");
        assert!(load_day_note_switcher_entries_result(&owned_days).is_err());
        assert_eq!(
            fs::read_to_string(foreign_file).unwrap(),
            "private foreign day text"
        );
    }

    #[test]
    fn day_note_switcher_infos_use_shared_note_action_ids() {
        let dir = tempfile::tempdir().expect("tempdir");
        let days = dir.path().join("days");
        std::fs::create_dir_all(&days).expect("days dir");
        std::fs::write(days.join("2026-06-01.md"), "alpha day\nsecond line").expect("write day");

        let entries = load_day_note_switcher_entries(&days);
        let infos = day_note_switcher_infos(
            &entries,
            Some(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).expect("date")),
        );

        assert_eq!(infos.len(), 1);
        assert_eq!(infos[0].id, "day:2026-06-01");
        assert_eq!(infos[0].title, "2026-06-01 · Monday");
        assert!(infos[0].is_current);
        assert_eq!(
            parse_day_note_action_id(&infos[0].id),
            Some(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).expect("date"))
        );
    }

    /// People look for a day page by the date written out ("march 3", "3rd mar"), by its
    /// weekday, as "yesterday", or by words on the page that are not next to each other,
    /// often inside a request. Each of these must reach the right page and only that page.
    #[test]
    fn day_pages_are_found_by_the_dates_and_words_people_type() {
        let dir = tempfile::tempdir().expect("tempdir");
        let days = dir.path().join("days");
        std::fs::create_dir_all(&days).expect("days dir");
        std::fs::write(
            days.join("2026-03-03.md"),
            "Called Sam about the invoice.\n",
        )
        .unwrap();
        std::fs::write(days.join("2026-03-09.md"), "Dentist at noon.\n").unwrap();
        std::fs::write(days.join("2026-03-10.md"), "Quiet day.\n").unwrap();
        let entries = load_day_note_switcher_entries(&days);
        let today = NaiveDate::from_ymd_opt(2026, 3, 10).expect("date");
        let found = |query: &str| -> Vec<String> {
            day_page_file_matches_in(&entries, today, query, 10)
                .into_iter()
                .map(|hit| hit.file.path)
                .map(|path| path.rsplit('/').next().unwrap_or_default().to_string())
                .collect()
        };

        for query in [
            "march 3",
            "mar 3",
            "3rd march",
            "tuesday",
            "sam invoice",
            "notes from march 3",
            "2026-03-03",
            "2026-03-0",
        ] {
            assert!(
                found(query).contains(&"2026-03-03.md".to_string()),
                "{query:?} should find the March 3 page, found {:?}",
                found(query)
            );
        }
        assert_eq!(found("yesterday"), vec!["2026-03-09.md".to_string()]);
        assert_eq!(found("today"), vec!["2026-03-10.md".to_string()]);
        assert!(found("march 4").is_empty(), "no page is dated March 4");
        assert!(
            found("sam dentist").is_empty(),
            "words on different pages don't make a match"
        );
    }
}
