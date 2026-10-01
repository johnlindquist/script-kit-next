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

/// How long the list is.
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
}

fn today() -> NaiveDate {
    crate::brain::substrate::BrainSubstrate::default_kit()
        .capture_day_destination(Utc::now())
        .0
}

fn day_title(date: NaiveDate) -> String {
    if date == today() {
        "Today".to_string()
    } else {
        notes::day_switcher::day_note_title(date)
    }
}

/// Pinned notes first, then the rest in `sort`'s order.
pub(crate) fn order_found(found: &mut [Found], sort: NotesSort) {
    found.sort_by(|a, b| {
        b.pinned.cmp(&a.pinned).then_with(|| match sort {
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
        let mut list = Self {
            query,
            found: Vec::new(),
            sel: 0,
            sort: chosen_sort(),
        };
        list.search();
        list
    }

    fn search(&mut self) {
        let query = self.query.trim().to_string();
        let lower = query.to_lowercase();
        let has = |text: &str| lower.is_empty() || text.to_lowercase().contains(&lower);
        let mut found = Vec::new();
        for day in
            notes::day_switcher::load_day_note_switcher_entries(&notes::notes_brain_days_dir())
        {
            let title = day_title(day.date);
            if has(&title) || has(&day.title) || has(&day.content) {
                let at = day
                    .date
                    .and_hms_opt(0, 0, 0)
                    .map(|at| at.and_utc())
                    .unwrap_or(day.updated_at);
                found.push(Found {
                    doc: NoteDoc::Day(day.date),
                    title,
                    sub: "Day page".to_string(),
                    pinned: false,
                    changed: day.updated_at,
                    created: at,
                });
            }
        }
        if notes::init_notes_db().is_ok() {
            let named = if query.is_empty() {
                notes::get_all_notes()
            } else {
                notes::search_notes(&query)
            };
            for note in named.unwrap_or_default() {
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
                    sub: format!("Note · changed {}", note.updated_at.format("%b %-d")),
                    title,
                    pinned: note.is_pinned,
                    changed: note.updated_at,
                    created: note.created_at,
                });
            }
        }
        order_found(&mut found, self.sort);
        found.truncate(LISTED_NOTES);
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
        self.sel = 0;
    }

    pub(crate) fn label(&self) -> String {
        format!("Notes · by {} · Tab to change", self.sort.words())
    }

    pub(crate) fn enter(&self) -> EnterTarget {
        match self.found.get(self.sel) {
            Some(Found {
                doc: NoteDoc::New(title),
                ..
            }) => EnterTarget::new(format!("notes:new:{title}"), "Start Note"),
            Some(found) => EnterTarget::new(format!("notes:open:{}", found.title), "Open Note"),
            None => EnterTarget::new("notes:new", "New Note"),
        }
    }

    pub(crate) fn typed(&mut self, text: String) {
        if text != self.query {
            self.query = text;
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
        let target = match doc {
            Some(NoteDoc::Day(date)) => NoteTarget::Day(date),
            Some(NoteDoc::Note(id)) => NoteTarget::Note(id),
            Some(NoteDoc::New(title)) => {
                match notes::new_note_to_edit(Some(&format!("# {title}\n\n"))) {
                    Ok(id) => NoteTarget::Note(id),
                    Err(error) => {
                        self.one_window_notice(crate::line_notice::FAILED, error.to_string(), cx);
                        return Vec::new();
                    }
                }
            }
            None => NoteTarget::New,
        };
        self.open_note_in_editor_soon(target, crate::DayPageBack::NotesList, cx);
        Vec::new()
    }
}
