//! Notes in the main window, under its input: a note's lines as rows, the input as the line you
//! add or edit, the notes listed by name, and a note handed to the model.
//!
//! Storage is the Notes storage: day notes are the brain's day files, other notes live in the
//! notes database. A delete waits in the line above the input like anything else that can't be
//! taken back, and ⌘Z there stops it.

use anyhow::{anyhow, Result};
use chrono::NaiveDate;
use gpui::Context;
use sk_one_window::{Done, EnterTarget, Event, Key};

use super::views::HostedView;
use crate::notes::{self, NoteId, NoteTarget};
use crate::ScriptListApp;

pub(crate) const NOTES: &str = "notes";

/// A key the input would otherwise take, which a note view uses on a highlighted line.
pub(crate) const DELETE_LINE_KEY: &str = "cmd-backspace";
/// Lists the notes by name from inside one.
pub(crate) const FIND_NOTE_KEY: &str = "cmd-p";
/// The source a note handed to the model's chat is tagged with.
const NOTE_SOURCE: &str = "note";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum NoteDoc {
    Day(NaiveDate),
    Note(NoteId),
    /// Not saved yet: the first line added creates it.
    New,
}

/// A change to a note, applied to what's stored now rather than to what the view last read.
enum Change {
    Append(String),
    Replace { old: String, new: String },
    Remove(String),
}

/// A line waiting to be deleted: the note view leaves it out until the wait ends.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PendingDelete {
    pub(crate) doc: NoteDoc,
    raw: String,
}

fn today() -> NaiveDate {
    crate::brain::substrate::BrainSubstrate::default_kit()
        .capture_day_destination(chrono::Utc::now())
        .0
}

fn day_path(date: NaiveDate) -> std::path::PathBuf {
    notes::notes_brain_days_dir().join(format!("{date}.md"))
}

fn day_title(date: NaiveDate) -> String {
    if date == today() {
        "Today".to_string()
    } else {
        crate::notes::day_switcher::day_note_title(date)
    }
}

/// A stored line's markup (a heading, a bullet, a checkbox, a capture's time) and its words.
fn split_line(raw: &str) -> (&str, &str) {
    let mut at = raw.len() - raw.trim_start().len();
    let skip_time = |at: &mut usize| {
        if let Some((time, _)) = raw[*at..].split_once(' ') {
            let is_time = time.len() <= 5
                && time.contains(':')
                && time.chars().all(|c| c.is_ascii_digit() || c == ':');
            if is_time {
                *at += time.len() + 1;
            }
        }
    };
    // A day line leads with its time, then any marker: `07:56 - [ ] body`.
    skip_time(&mut at);
    let marker = [
        "- [ ] ", "- [x] ", "- ", "* ", "#### ", "### ", "## ", "# ", "> ",
    ]
    .iter()
    .find(|m| raw[at..].starts_with(**m));
    if let Some(marker) = marker {
        at += marker.len();
        skip_time(&mut at);
    }
    raw.split_at(at)
}

/// What's stored for a note: its frontmatter (kept as it is) and its body.
fn read_doc(doc: &NoteDoc) -> Result<(String, String, String)> {
    match doc {
        NoteDoc::Day(date) => {
            let content =
                crate::brain::substrate::io::read_private_document_if_present(&day_path(*date))?
                    .unwrap_or_default();
            Ok((day_title(*date), String::new(), content))
        }
        NoteDoc::Note(id) => {
            notes::init_notes_db()?;
            let note = notes::get_note(*id)?
                .filter(|note| note.deleted_at.is_none())
                .ok_or_else(|| anyhow!("That note isn't there any more"))?;
            let body = notes::metadata::strip_frontmatter(&note.content).to_string();
            let head = note.content[..note.content.len() - body.len()].to_string();
            Ok((note.title, head, body))
        }
        NoteDoc::New => Ok(("New note".to_string(), String::new(), String::new())),
    }
}

/// Applies `change` to the stored note. A new note is created by its first line; returns the
/// note it wrote.
fn write_change(doc: &NoteDoc, change: Change) -> Result<NoteDoc> {
    if let (NoteDoc::Day(date), Change::Append(text)) = (doc, &change) {
        notes::notes_substrate()?.append_to_captured_day(
            *date,
            chrono::Utc::now(),
            crate::brain::substrate::DayEntry::Capture { text: text.clone() },
        )?;
        return Ok(doc.clone());
    }
    let (_, head, body) = read_doc(doc)?;
    let mut lines: Vec<String> = body.lines().map(str::to_string).collect();
    match change {
        Change::Append(text) => lines.push(text),
        Change::Replace { old, new } => {
            let at = lines
                .iter()
                .position(|l| *l == old)
                .ok_or_else(|| anyhow!("That line changed since it was read"))?;
            lines[at] = new;
        }
        Change::Remove(old) => {
            let at = lines
                .iter()
                .position(|l| *l == old)
                .ok_or_else(|| anyhow!("That line isn't there any more"))?;
            lines.remove(at);
        }
    }
    let body = lines.join("\n");
    match doc {
        NoteDoc::Day(date) => {
            crate::brain::substrate::io::atomic_write(&day_path(*date), &format!("{body}\n"))?;
            Ok(doc.clone())
        }
        NoteDoc::Note(id) => {
            let mut note = notes::get_note(*id)?.ok_or_else(|| anyhow!("That note isn't there"))?;
            note.set_content(format!("{head}{body}"));
            notes::save_note(&note)?;
            Ok(doc.clone())
        }
        NoteDoc::New => {
            notes::init_notes_db()?;
            let note = notes::Note::with_content(body);
            notes::save_note(&note)?;
            Ok(NoteDoc::Note(note.id))
        }
    }
}

/// A note open under the input.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NoteView {
    pub(crate) doc: NoteDoc,
    pub(crate) title: String,
    /// The note's lines as stored.
    raw: Vec<String>,
    /// The lines drawn as rows: the stored lines that say something.
    shown: Vec<usize>,
    /// The highlighted row; `shown.len()` is the new line at the bottom.
    pub(crate) sel: usize,
    /// The row whose words are in the input to be changed.
    pub(crate) editing: Option<usize>,
    /// The input's words: the line to add, or the line being changed.
    pub(crate) line: String,
    /// The line to add, kept while another line is being changed.
    draft: String,
}

/// A row the note view draws.
pub(crate) struct NoteRow {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) description: Option<String>,
    pub(crate) selected: bool,
    pub(crate) new_line: bool,
}

impl NoteView {
    fn load(doc: NoteDoc, pending: &[PendingDelete]) -> Result<Self> {
        let (title, _, body) = read_doc(&doc)?;
        let mut raw: Vec<String> = body.lines().map(str::to_string).collect();
        for delete in pending.iter().filter(|d| d.doc == doc) {
            if let Some(at) = raw.iter().position(|l| *l == delete.raw) {
                raw.remove(at);
            }
        }
        let shown: Vec<usize> = (0..raw.len())
            .filter(|i| !split_line(&raw[*i]).1.trim().is_empty())
            .collect();
        Ok(Self {
            sel: shown.len(),
            doc,
            title,
            raw,
            shown,
            editing: None,
            line: String::new(),
            draft: String::new(),
        })
    }

    fn at_new_line(&self) -> bool {
        self.sel >= self.shown.len()
    }

    fn raw_at(&self, row: usize) -> Option<&str> {
        self.shown.get(row).map(|i| self.raw[*i].as_str())
    }

    pub(crate) fn row_count(&self) -> usize {
        self.shown.len() + 1
    }

    pub(crate) fn rows(&self) -> Vec<NoteRow> {
        let mut rows: Vec<NoteRow> = self
            .shown
            .iter()
            .enumerate()
            .map(|(row, i)| {
                let editing = self.editing == Some(row);
                NoteRow {
                    id: format!("note-line:{i}"),
                    title: if editing {
                        self.line.clone()
                    } else {
                        split_line(&self.raw[*i]).1.to_string()
                    },
                    description: if editing {
                        Some("↵ saves it · esc leaves the note".to_string())
                    } else {
                        (self.sel == row).then(|| "↵ changes it · ⌘⌫ deletes it".to_string())
                    },
                    selected: self.sel == row,
                    new_line: false,
                }
            })
            .collect();
        let adding = self.editing.is_none() && !self.line.trim().is_empty();
        rows.push(NoteRow {
            id: "note-line:new".to_string(),
            title: if adding {
                self.line.clone()
            } else {
                "Write a line…".to_string()
            },
            description: self
                .at_new_line()
                .then(|| "↵ adds it · ⌘↵ asks the model about this note · ⌘P finds a note".into()),
            selected: self.at_new_line(),
            new_line: true,
        });
        rows
    }

    pub(crate) fn enter(&self) -> EnterTarget {
        if self.editing.is_some() {
            return EnterTarget::new("note:save", "Save the line");
        }
        if !self.at_new_line() {
            return EnterTarget::new(format!("note:edit:{}", self.sel), "Change this line");
        }
        if self.line.trim().is_empty() {
            EnterTarget::new("note:add", "Write a line first")
        } else {
            EnterTarget::new("note:add", format!("Add it to “{}”", self.title))
        }
    }

    /// The note's words, for the model.
    fn text(&self) -> String {
        self.raw.join("\n")
    }

    /// Words typed while the note is open are the line being added or changed.
    pub(crate) fn typed(&mut self, text: String) {
        self.line = text;
    }
}

/// The notes listed by name, to open one.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NotesList {
    pub(crate) query: String,
    pub(crate) found: Vec<Found>,
    pub(crate) sel: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Found {
    pub(crate) doc: NoteDoc,
    pub(crate) title: String,
    pub(crate) sub: String,
}

const LISTED_NOTES: usize = 30;

impl NotesList {
    fn new(query: String) -> Self {
        let mut list = Self {
            query,
            found: Vec::new(),
            sel: 0,
        };
        list.search();
        list
    }

    fn search(&mut self) {
        let q = self.query.trim().to_lowercase();
        let named = |title: &str| q.is_empty() || title.to_lowercase().contains(&q);
        let mut found = Vec::new();
        let days = crate::notes::day_switcher::load_day_note_switcher_entries(
            &notes::notes_brain_days_dir(),
        );
        let mut days: Vec<_> = days.into_iter().collect();
        days.sort_by_key(|day| std::cmp::Reverse(day.date));
        for day in days {
            let title = day_title(day.date);
            if named(&title) || named(&day.title) {
                found.push(Found {
                    doc: NoteDoc::Day(day.date),
                    title,
                    sub: "Day note".to_string(),
                });
            }
        }
        if notes::init_notes_db().is_ok() {
            for note in notes::get_all_notes().unwrap_or_default() {
                if note.deleted_at.is_none() && named(&note.title) {
                    found.push(Found {
                        doc: NoteDoc::Note(note.id),
                        sub: format!("Note · changed {}", note.updated_at.format("%b %-d")),
                        title: note.title,
                    });
                }
            }
        }
        found.truncate(LISTED_NOTES);
        self.found = found;
        self.sel = 0;
    }

    pub(crate) fn enter(&self) -> EnterTarget {
        match self.found.get(self.sel) {
            Some(found) => EnterTarget::new(
                format!("notes:open:{}", found.title),
                format!("Open “{}”", found.title),
            ),
            None => EnterTarget::new("notes:none", "No note by that name"),
        }
    }

    pub(crate) fn typed(&mut self, text: String) {
        if text != self.query {
            self.query = text;
            self.search();
        }
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
    /// Whether a note is the view open under the input.
    pub(crate) fn one_window_note_open(&self) -> bool {
        matches!(self.one_window_hosted_view(), Some(HostedView::Note(_)))
    }

    /// Opens `target` under the input, in place of the list.
    pub(crate) fn one_window_open_note_target(
        &mut self,
        target: NoteTarget,
        cx: &mut Context<Self>,
    ) -> Result<()> {
        if !matches!(self.current_view, crate::AppView::ScriptList) {
            self.reset_to_script_list(cx);
        }
        let pending = &self.one_window_host.note_deletes;
        let (view, line) = match target {
            NoteTarget::Search => {
                let list = NotesList::new(String::new());
                self.one_window_open_hosted(HostedView::Notes(list), Some(String::new()), cx);
                return Ok(());
            }
            NoteTarget::Today => (
                NoteView::load(NoteDoc::Day(today()), pending)?,
                String::new(),
            ),
            NoteTarget::Append(line) => (NoteView::load(NoteDoc::Day(today()), pending)?, line),
            NoteTarget::Day(date) => (NoteView::load(NoteDoc::Day(date), pending)?, String::new()),
            NoteTarget::Note(id) => (NoteView::load(NoteDoc::Note(id), pending)?, String::new()),
            NoteTarget::New => (NoteView::load(NoteDoc::New, pending)?, String::new()),
        };
        let view = NoteView {
            line: line.clone(),
            ..view
        };
        self.one_window_open_hosted(HostedView::Note(view), Some(line), cx);
        Ok(())
    }

    /// A note someone asked for while the window was hidden opens once it's shown.
    pub(crate) fn one_window_open_waiting_note(&mut self, cx: &mut Context<Self>) {
        if let Some(target) = self.one_window_host.note_waiting.take() {
            if let Err(error) = self.one_window_open_note_target(target, cx) {
                self.one_window_notice(crate::line_notice::FAILED, error.to_string(), cx);
            }
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
            return self.one_window_open_note_target(target, cx);
        }
        self.one_window_host.note_waiting = Some(target);
        self.dispatch_window_event(
            crate::window_orchestrator::WindowEvent::ShowMain { activate_app: true },
            cx,
        );
        Ok(())
    }

    /// A key the reducer routed to an open note.
    pub(crate) fn one_window_note_key(&mut self, key: Key, cx: &mut Context<Self>) -> Vec<Event> {
        let Some(HostedView::Note(view)) = self.one_window_host.view.as_mut() else {
            return Vec::new();
        };
        let before = view.enter();
        let len = view.row_count();
        if move_sel(&mut view.sel, len, &key) {
            let mut events = Vec::new();
            if let Some(row) = view.editing.take() {
                if row != view.sel {
                    let draft = std::mem::take(&mut view.draft);
                    view.line = draft.clone();
                    events.extend(self.one_window_note_words(draft, cx));
                }
            }
            return self.one_window_note_enter_changed(before, events);
        }
        match key {
            Key::Enter => self.one_window_note_enter(cx),
            Key::CmdEnter => {
                let note = crate::ai::message_parts::AiContextPart::TextBlock {
                    label: format!("Note · {}", view.title),
                    source: NOTE_SOURCE.to_string(),
                    text: view.text(),
                    mime_type: None,
                };
                let asked = view.line.trim();
                let entry = if asked.is_empty() {
                    crate::one_window::chat_entry::ChatEntry::quick()
                } else {
                    crate::one_window::chat_entry::ChatEntry::sending(asked)
                };
                self.one_window_open_chat(entry.with(note), cx);
                Vec::new()
            }
            Key::Other(k) if k == DELETE_LINE_KEY => {
                self.one_window_note_delete(cx);
                Vec::new()
            }
            Key::Other(k) if k == FIND_NOTE_KEY => {
                let list = NotesList::new(String::new());
                self.one_window_open_hosted(HostedView::Notes(list), Some(String::new()), cx);
                Vec::new()
            }
            Key::CmdZ => {
                self.one_window_note_stop_delete(cx);
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    fn one_window_note_enter_changed(
        &self,
        before: EnterTarget,
        mut events: Vec<Event>,
    ) -> Vec<Event> {
        if let Some(HostedView::Note(view)) = self.one_window_host.view.as_ref() {
            let after = view.enter();
            if after != before {
                events.push(Event::ViewEnter(after));
            }
        }
        events
    }

    /// Puts `words` in the input, the way the note moved them there rather than typing.
    fn one_window_note_words(&mut self, words: String, cx: &mut Context<Self>) -> Vec<Event> {
        self.one_window_host.view_text = Some(words.clone());
        self.one_window_set_input(words.clone(), cx);
        vec![Event::Text(words)]
    }

    fn one_window_note_enter(&mut self, cx: &mut Context<Self>) -> Vec<Event> {
        let Some(HostedView::Note(view)) = self.one_window_host.view.as_mut() else {
            return Vec::new();
        };
        let before = view.enter();
        if let Some(row) = view.editing {
            let Some(old) = view.raw_at(row).map(str::to_string) else {
                return Vec::new();
            };
            let (markup, _) = split_line(&old);
            let new = format!("{markup}{}", view.line.trim());
            let doc = view.doc.clone();
            let draft = std::mem::take(&mut view.draft);
            return match write_change(&doc, Change::Replace { old, new }) {
                Ok(_) => self.one_window_note_reload(doc, Some(row), draft, cx),
                Err(error) => {
                    self.one_window_notice(crate::line_notice::FAILED, error.to_string(), cx);
                    Vec::new()
                }
            };
        }
        if !view.at_new_line() {
            let row = view.sel;
            let words = view
                .raw_at(row)
                .map(|raw| split_line(raw).1.to_string())
                .unwrap_or_default();
            view.draft = std::mem::replace(&mut view.line, words.clone());
            view.editing = Some(row);
            let events = self.one_window_note_words(words, cx);
            return self.one_window_note_enter_changed(before, events);
        }
        let line = view.line.trim().to_string();
        if line.is_empty() {
            return Vec::new();
        }
        let doc = view.doc.clone();
        let title = view.title.clone();
        match write_change(&doc, Change::Append(line.clone())) {
            Ok(doc) => {
                let mut events = self.one_window_note_reload(doc, None, String::new(), cx);
                events.push(Event::Done(Done {
                    icon: "note".into(),
                    title: format!("Added to “{title}” · {line}"),
                    stay: true,
                    ..Done::default()
                }));
                events
            }
            Err(error) => {
                self.one_window_notice(crate::line_notice::FAILED, error.to_string(), cx);
                Vec::new()
            }
        }
    }

    /// Reads the note again after it changed, keeping the highlight where it was, with `words`
    /// in the input.
    fn one_window_note_reload(
        &mut self,
        doc: NoteDoc,
        sel: Option<usize>,
        words: String,
        cx: &mut Context<Self>,
    ) -> Vec<Event> {
        let before = match self.one_window_host.view.as_ref() {
            Some(HostedView::Note(view)) => view.enter(),
            _ => return Vec::new(),
        };
        let view = match NoteView::load(doc, &self.one_window_host.note_deletes) {
            Ok(view) => view,
            Err(error) => {
                self.one_window_notice(crate::line_notice::FAILED, error.to_string(), cx);
                return Vec::new();
            }
        };
        let sel = sel.unwrap_or(view.shown.len()).min(view.shown.len());
        let view = NoteView {
            sel,
            line: words.clone(),
            ..view
        };
        let after = view.enter();
        self.one_window_host.view = Some(HostedView::Note(view));
        let mut events = self.one_window_note_words(words, cx);
        if after != before {
            events.push(Event::ViewEnter(after));
        }
        events
    }

    /// ⌘⌫ on a line: it leaves the note now and is deleted when the wait in the line above the
    /// input runs out; ⌘Z stops it.
    fn one_window_note_delete(&mut self, cx: &mut Context<Self>) {
        let Some(HostedView::Note(view)) = self.one_window_host.view.as_ref() else {
            return;
        };
        if view.editing.is_some() || view.at_new_line() {
            return;
        }
        let Some(raw) = view.raw_at(view.sel).map(str::to_string) else {
            return;
        };
        let reopen = view.clone();
        let pending = PendingDelete {
            doc: view.doc.clone(),
            raw: raw.clone(),
        };
        let title = format!("Delete “{}”", split_line(&raw).1.trim());
        self.one_window_host.note_deletes.push(pending.clone());
        let ran = pending.clone();
        let stopped = pending;
        self.one_window_hold_or_stop(
            "trash",
            title,
            Box::new(move |app, cx| {
                app.one_window_host.note_deletes.retain(|d| *d != ran);
                if let Err(error) = write_change(&ran.doc, Change::Remove(ran.raw.clone())) {
                    app.one_window_notice(crate::line_notice::FAILED, error.to_string(), cx);
                }
            }),
            Some(Box::new(move |app, cx| {
                app.one_window_host.note_deletes.retain(|d| *d != stopped);
                app.one_window_note_refresh(&stopped.doc, cx);
            })),
            cx,
        );
        self.one_window_note_reopen(reopen, cx);
    }

    /// ⌘Z in a note stops the delete waiting in the line; the note stays open.
    fn one_window_note_stop_delete(&mut self, cx: &mut Context<Self>) {
        let Some(HostedView::Note(view)) = self.one_window_host.view.clone() else {
            return;
        };
        let Some(pending) = self
            .one_window_host
            .note_deletes
            .iter()
            .rev()
            .find(|d| d.doc == view.doc)
            .cloned()
        else {
            return;
        };
        let target = format!("Delete “{}”", split_line(&pending.raw).1.trim());
        self.one_window_handle(
            Event::Unsend {
                target: Some(target),
            },
            cx,
        );
        self.one_window_note_reopen(view, cx);
    }

    /// The wait in the line takes the window home; the note comes straight back, reread.
    fn one_window_note_reopen(&mut self, view: NoteView, cx: &mut Context<Self>) {
        let doc = view.doc.clone();
        match NoteView::load(doc, &self.one_window_host.note_deletes) {
            Ok(fresh) => {
                let sel = view.sel.min(fresh.shown.len());
                let fresh = NoteView { sel, ..fresh };
                self.one_window_open_hosted(HostedView::Note(fresh), Some(String::new()), cx);
            }
            Err(error) => self.one_window_notice(crate::line_notice::FAILED, error.to_string(), cx),
        }
    }

    /// The open note shows `doc` again as stored, if it's that note.
    fn one_window_note_refresh(&mut self, doc: &NoteDoc, cx: &mut Context<Self>) {
        let open = match self.one_window_host.view.as_ref() {
            Some(HostedView::Note(view)) if view.doc == *doc => view.clone(),
            _ => return,
        };
        if let Ok(fresh) = NoteView::load(open.doc.clone(), &self.one_window_host.note_deletes) {
            let sel = open.sel.min(fresh.shown.len());
            self.one_window_host.view = Some(HostedView::Note(NoteView {
                sel,
                line: open.line,
                editing: None,
                ..fresh
            }));
            cx.notify();
        }
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
        let Some(found) = list.found.get(list.sel).cloned() else {
            return Vec::new();
        };
        let target = match found.doc {
            NoteDoc::Day(date) => NoteTarget::Day(date),
            NoteDoc::Note(id) => NoteTarget::Note(id),
            NoteDoc::New => NoteTarget::New,
        };
        if let Err(error) = self.one_window_open_note_target(target, cx) {
            self.one_window_notice(crate::line_notice::FAILED, error.to_string(), cx);
        }
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::split_line;

    #[test]
    fn a_line_keeps_its_markup_apart_from_its_words() {
        assert_eq!(split_line("- 10:30 call mom"), ("- 10:30 ", "call mom"));
        assert_eq!(split_line("- [ ] buy milk"), ("- [ ] ", "buy milk"));
        assert_eq!(split_line("## Launch"), ("## ", "Launch"));
        assert_eq!(split_line("plain words"), ("", "plain words"));
        assert_eq!(split_line("- 2026 plan"), ("- ", "2026 plan"));
        assert_eq!(
            split_line("07:56 call the venue"),
            ("07:56 ", "call the venue")
        );
        assert_eq!(
            split_line("07:56 - [ ] ship it"),
            ("07:56 - [ ] ", "ship it")
        );
    }
}
