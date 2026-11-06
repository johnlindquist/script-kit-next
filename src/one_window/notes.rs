//! The Notes list under the main input: every note and day page by name, pinned notes first.
//!
//! Typing narrows it by title and words; Tab changes the order; Enter opens the highlighted note
//! in the note editor, and Esc there comes back here. A name no note has yet offers to start one.

use std::sync::atomic::{AtomicU8, Ordering};

use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use gpui::Context;
use sk_one_window::{EnterTarget, Event, Key};

use super::views::HostedView;
use crate::notes::{self, NoteId, NoteTarget};
use crate::ScriptListApp;

pub(crate) const NOTES: &str = "notes";

/// How many notes the list shows at first, and how many more each "Show More" adds.
const LISTED_NOTES: usize = 60;

/// The order notes are listed in after the pinned ones. Tab moves to the next.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum NotesSort {
    #[default]
    Changed,
    Title,
    Created,
}

impl NotesSort {
    const ALL: [NotesSort; 3] = [NotesSort::Changed, NotesSort::Title, NotesSort::Created];

    pub(crate) fn words(self) -> &'static str {
        match self {
            NotesSort::Changed => "last changed",
            NotesSort::Title => "title",
            NotesSort::Created => "newest",
        }
    }

    fn next(self) -> Self {
        let at = Self::ALL.iter().position(|sort| *sort == self).unwrap_or(0);
        Self::ALL[(at + 1) % Self::ALL.len()]
    }
}

/// The order picked last, kept while the app runs.
static SORT: AtomicU8 = AtomicU8::new(0);

fn chosen_sort() -> NotesSort {
    NotesSort::ALL[usize::from(SORT.load(Ordering::Relaxed)) % NotesSort::ALL.len()]
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum NoteDoc {
    Day(NaiveDate),
    Note(NoteId),
    /// A new note whose first line is these words.
    New(String),
    /// The list stopped at its length; Enter shows the next notes.
    More,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Found {
    pub(crate) doc: NoteDoc,
    pub(crate) title: String,
    pub(crate) sub: String,
    pub(crate) pinned: bool,
    pub(crate) changed: DateTime<Utc>,
    pub(crate) created: DateTime<Utc>,
}

impl Found {
    pub(crate) fn icon(&self) -> &'static str {
        match self.doc {
            _ if self.pinned => "starfilled",
            NoteDoc::New(_) => "plus",
            NoteDoc::More => "chevrondown",
            NoteDoc::Day(_) | NoteDoc::Note(_) => "file",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NotesList {
    pub(crate) query: String,
    pub(crate) found: Vec<Found>,
    pub(crate) sel: usize,
    pub(crate) sort: NotesSort,
    /// How many notes are listed before "Show More".
    shown: usize,
    /// Why notes couldn't be searched; the empty list says it instead of looking empty.
    pub(crate) trouble: Option<String>,
    /// Trouble that appeared since the line last said it.
    unsaid: Option<String>,
    /// The day pages folder couldn't be read.
    days_unreadable: bool,
    /// Day pages, read once when the list opens rather than on every key.
    days: Vec<notes::day_switcher::DayNoteSwitcherEntry>,
}

fn today() -> NaiveDate {
    crate::brain::substrate::BrainSubstrate::default_kit()
        .capture_day_destination(Utc::now())
        .0
}

/// Quote the matching body line rather than making someone open every title-only
/// result. Map folded offsets back to characters before cutting a bounded excerpt.
fn matching_line(body: &str, title: &str, query: &str) -> Option<String> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return None;
    }
    for line in body.lines().map(str::trim) {
        if line.trim_start_matches('#').trim() == title.trim() {
            continue;
        }
        let lower = line.to_lowercase();
        let Some(hit) = lower.find(&query) else {
            continue;
        };
        let mut offset = 0usize;
        let mut before = 0usize;
        for ch in line.chars() {
            if offset >= hit {
                break;
            }
            offset += ch.to_lowercase().map(char::len_utf8).sum::<usize>();
            before += 1;
        }
        let start = before.saturating_sub(32usize);
        let mut chars = line.chars().skip(start);
        let mut excerpt = String::new();
        if start > 0 {
            excerpt.push('…');
        }
        excerpt.extend(chars.by_ref().take(112));
        if chars.next().is_some() {
            excerpt.push('…');
        }
        return Some(excerpt);
    }
    None
}

#[cfg(test)]
mod excerpt_tests {
    use super::matching_line;
    #[test]
    fn a_long_unicode_body_can_show_why_a_title_matched() {
        let body = format!(
            "# Agenda\n\n{}Æble REVIEW café{}",
            "ø".repeat(120),
            "!".repeat(120)
        );
        let excerpt = matching_line(&body, "Agenda", "review").unwrap();
        assert!(excerpt.contains("Æble REVIEW café"));
        assert!(excerpt.starts_with('…') && excerpt.ends_with('…'));
        assert!(matching_line("# Agenda\nBody", "Agenda", "Agenda").is_none());
    }
}

fn day_title(date: NaiveDate) -> String {
    if date == today() {
        "Today".to_string()
    } else {
        notes::day_switcher::day_note_title(date)
    }
}

/// A title named exactly what's typed first, then one starting with it, then pinned notes,
/// then the rest in `sort`'s order — the same order as the launcher's note search.
pub(crate) fn order_found(found: &mut [Found], sort: NotesSort, query: &str) {
    let typed = query.trim().to_lowercase();
    let title_rank = |found: &Found| {
        let title = found.title.to_lowercase();
        match () {
            _ if typed.is_empty() => 2,
            _ if title == typed => 0,
            _ if title.starts_with(&typed) => 1,
            _ => 2,
        }
    };
    found.sort_by(|a, b| {
        let ahead = title_rank(a)
            .cmp(&title_rank(b))
            .then_with(|| b.pinned.cmp(&a.pinned));
        ahead.then_with(|| match sort {
            NotesSort::Changed => b.changed.cmp(&a.changed),
            NotesSort::Created => b.created.cmp(&a.created),
            NotesSort::Title => {
                let day = |found: &Found| matches!(found.doc, NoteDoc::Day(_));
                day(a)
                    .cmp(&day(b))
                    .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
            }
        })
    });
}

impl NotesList {
    pub(crate) fn new(query: String) -> Self {
        let days = notes::day_switcher::load_day_note_switcher_entries_result(
            &notes::notes_brain_days_dir(),
        );
        let days_unreadable = days.is_err();
        if let Err(error) = &days {
            tracing::warn!(error = %error, "notes_list_day_pages_unreadable");
        }
        let mut list = Self {
            query,
            found: Vec::new(),
            sel: 0,
            sort: chosen_sort(),
            shown: LISTED_NOTES,
            trouble: None,
            unsaid: None,
            days: days.unwrap_or_default(),
            days_unreadable,
        };
        list.search();
        list
    }

    fn search(&mut self) {
        let query = self.query.trim().to_string();
        let today = today();
        let day_query = notes::day_switcher::DayQuery::parse(&query, today);
        let mut found = Vec::new();
        for day in &self.days {
            if day_query.matches(day, today) {
                let at = day
                    .date
                    .and_hms_opt(0, 0, 0)
                    .map(|at| at.and_utc())
                    .unwrap_or(day.updated_at);
                found.push(Found {
                    doc: NoteDoc::Day(day.date),
                    title: if day.date == today {
                        "Today".to_string()
                    } else {
                        day.title.clone()
                    },
                    sub: matching_line(&day.content, &day.title, &query)
                        .unwrap_or_else(|| "Day page".into()),
                    pinned: false,
                    changed: day.updated_at,
                    created: at,
                });
            }
        }
        let named = notes::ensure_notes_db_open().and_then(|()| {
            if query.is_empty() {
                notes::get_all_notes()
            } else {
                notes::search_notes(&query)
            }
        });
        if let Err(error) = &named {
            tracing::warn!(error = %error, "notes_list_search_failed");
        }
        let trouble = match (named.is_err(), self.days_unreadable) {
            (true, true) => Some("Couldn't read your notes or day pages".to_string()),
            (true, false) => Some("Couldn't search your notes; showing day pages only".to_string()),
            (false, true) => Some("Couldn't read your day pages; showing notes only".to_string()),
            (false, false) => None,
        };
        if trouble.is_some() && trouble != self.trouble {
            self.unsaid = trouble.clone();
        }
        self.trouble = trouble;
        if let Ok(mut named) = named {
            // "yesterday" or "notes from march 3" also means the notes changed on that day.
            if day_query.only_dates() {
                for note in
                    notes::notes_changed_on(|date| day_query.date_holds(date)).unwrap_or_default()
                {
                    if !named.iter().any(|known| known.id == note.id) {
                        named.push(note);
                    }
                }
            }
            for note in named {
                if note.deleted_at.is_some() {
                    continue;
                }
                let title = if note.title.trim().is_empty() {
                    "Untitled Note".to_string()
                } else {
                    note.title.clone()
                };
                found.push(Found {
                    doc: NoteDoc::Note(note.id),
                    sub: matching_line(&note.content, &title, &query).unwrap_or_else(|| {
                        format!("Note · changed {}", note.updated_at.format("%b %-d"))
                    }),
                    title,
                    pinned: note.is_pinned,
                    changed: note.updated_at,
                    created: note.created_at,
                });
            }
        }
        order_found(&mut found, self.sort, &query);
        let total = found.len();
        if total > self.shown {
            found.truncate(self.shown);
            let now = Utc::now();
            found.push(Found {
                doc: NoteDoc::More,
                title: format!("Show {} More", (total - self.shown).min(LISTED_NOTES)),
                sub: format!("{} of {total} shown", self.shown),
                pinned: false,
                changed: now,
                created: now,
            });
        }
        let named_exactly = found
            .iter()
            .any(|found| found.title.eq_ignore_ascii_case(&query));
        if !query.is_empty() && !named_exactly {
            let now = Utc::now();
            found.push(Found {
                doc: NoteDoc::New(query.clone()),
                title: format!("New Note “{query}”"),
                sub: "Start a note with this title".to_string(),
                pinned: false,
                changed: now,
                created: now,
            });
        }
        self.found = found;
        self.sel = self.sel.min(self.found.len().saturating_sub(1));
    }

    /// Lists the next notes after "Show More", keeping the highlight where it was.
    pub(crate) fn show_more(&mut self) {
        self.shown += LISTED_NOTES;
        self.search();
    }

    pub(crate) fn label(&self) -> String {
        format!("Notes · by {} · Tab to change", self.sort.words())
    }

    /// New trouble the line should say, once.
    pub(crate) fn take_unsaid(&mut self) -> Option<String> {
        self.unsaid.take()
    }

    pub(crate) fn enter(&self) -> EnterTarget {
        match self.found.get(self.sel) {
            Some(Found {
                doc: NoteDoc::New(title),
                ..
            }) => EnterTarget::new(format!("notes:new:{title}"), "Start Note"),
            Some(Found {
                doc: NoteDoc::More, ..
            }) => EnterTarget::new("notes:more", "Show More"),
            Some(found) => EnterTarget::new(format!("notes:open:{}", found.title), "Open Note"),
            None => EnterTarget::new("notes:new", "New Note"),
        }
    }

    pub(crate) fn typed(&mut self, text: String) {
        if text != self.query {
            self.query = text;
            self.shown = LISTED_NOTES;
            self.sel = 0;
            self.search();
        }
    }

    fn next_sort(&mut self) {
        self.sort = self.sort.next();
        let at = NotesSort::ALL
            .iter()
            .position(|sort| *sort == self.sort)
            .unwrap_or(0);
        SORT.store(at as u8, Ordering::Relaxed);
        self.sel = 0;
        self.search();
    }
}

fn move_sel(sel: &mut usize, len: usize, key: &Key) -> bool {
    match key {
        Key::Down if *sel + 1 < len => *sel += 1,
        Key::Up if *sel > 0 => *sel -= 1,
        Key::Down | Key::Up => {}
        _ => return false,
    }
    true
}

impl ScriptListApp {
    /// The Notes list under the input, narrowed by `query`.
    pub(crate) fn one_window_open_notes_list(&mut self, query: String, cx: &mut Context<Self>) {
        let list = NotesList::new(query.clone());
        self.one_window_open_hosted(HostedView::Notes(list), Some(query), cx);
        self.one_window_say_notes_trouble(cx);
    }

    /// Says in the line why the Notes list couldn't search, when that's new.
    pub(crate) fn one_window_say_notes_trouble(&mut self, cx: &mut Context<Self>) {
        let Some(HostedView::Notes(list)) = self.one_window_host.view.as_mut() else {
            return;
        };
        if let Some(trouble) = list.take_unsaid() {
            cx.defer(move |cx| crate::line_notice::say(cx, crate::line_notice::FAILED, trouble));
        }
    }

    /// A note someone asked for while the window was hidden opens once it's shown.
    pub(crate) fn one_window_open_waiting_note(&mut self, cx: &mut Context<Self>) {
        if let Some(target) = self.one_window_host.note_waiting.take() {
            self.open_note_onto_shown_window(target, cx);
        }
    }

    /// Opens `target` now when the main window shows, or once it has.
    pub(crate) fn one_window_show_note(
        &mut self,
        target: NoteTarget,
        cx: &mut Context<Self>,
    ) -> Result<()> {
        if script_kit_gpui::is_main_window_visible() || crate::runtime_policy::is_owned_evaluation()
        {
            self.open_note_in_editor_soon(target, crate::DayPageBack::Leave, cx);
            return Ok(());
        }
        self.one_window_host.note_waiting = Some(target);
        self.dispatch_window_event(
            crate::window_orchestrator::WindowEvent::ShowMain { activate_app: true },
            cx,
        );
        Ok(())
    }

    /// A key the reducer routed to the notes list.
    pub(crate) fn one_window_notes_list_key(
        &mut self,
        key: Key,
        cx: &mut Context<Self>,
    ) -> Vec<Event> {
        let Some(HostedView::Notes(list)) = self.one_window_host.view.as_mut() else {
            return Vec::new();
        };
        let before = list.enter();
        let len = list.found.len();
        if key == Key::Tab {
            list.next_sort();
            cx.notify();
            let after = list.enter();
            return if after != before {
                vec![Event::ViewEnter(after)]
            } else {
                Vec::new()
            };
        }
        if move_sel(&mut list.sel, len, &key) {
            let after = list.enter();
            return if after != before {
                vec![Event::ViewEnter(after)]
            } else {
                Vec::new()
            };
        }
        if key != Key::Enter {
            return Vec::new();
        }
        let doc = list.found.get(list.sel).map(|found| found.doc.clone());
        if doc == Some(NoteDoc::More) {
            list.show_more();
            if let Some(trouble) = list.take_unsaid() {
                cx.defer(move |cx| {
                    crate::line_notice::say(cx, crate::line_notice::FAILED, trouble)
                });
            }
            cx.notify();
            let after = list.enter();
            return if after != before {
                vec![Event::ViewEnter(after)]
            } else {
                Vec::new()
            };
        }
        let target = match doc {
            Some(NoteDoc::Day(date)) => NoteTarget::Day(date),
            Some(NoteDoc::Note(id)) => NoteTarget::Note(id),
            Some(NoteDoc::New(title)) => {
                match notes::new_note_to_edit(Some(&format!("# {title}\n\n"))) {
                    Ok(id) => NoteTarget::Note(id),
                    Err(error) => {
                        self.one_window_notice(
                            crate::line_notice::FAILED,
                            format!("Couldn't make “{title}”: {error}"),
                            cx,
                        );
                        return Vec::new();
                    }
                }
            }
            None | Some(NoteDoc::More) => NoteTarget::New,
        };
        self.open_note_in_editor_soon(target, crate::DayPageBack::NotesList, cx);
        Vec::new()
    }
}
