//! Golden traces: what the prototype was fed and what it showed, as JSON lines.
//!
//! Input lines (`{"t":..,"ev":"text",...}`) become [`Event`]s; output lines (`{"t":..,"out":"slot",...}`)
//! are what the reducer must reproduce. The [`Observer`] writes the reducer's own output lines in the
//! same shape so the two can be compared.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Ms;
use crate::enter::EnterTarget;
use crate::ledger::{Entry, EntryState, Risk};
use crate::list::Item;
use crate::row::Row;
use crate::slot::{SlotKind, SlotLine};
use crate::stage::{Subject, SubjectKind};
use crate::window::{
    Done, Effect, Event, HeldOp, Key, OneWindow, Outcome, RunHow, Snapshot, ViewOp, ViewSpec,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "out", rename_all = "lowercase")]
pub enum Out {
    Slot {
        kind: String,
        label: String,
        title: String,
        #[serde(default)]
        sub: String,
        dimmed: bool,
    },
    Header {
        words: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        view: Option<String>,
    },
    Enter {
        id: String,
        label: String,
        armed: bool,
    },
    Run {
        key: String,
        title: String,
        how: String,
    },
    Ignored {
        key: String,
    },
    Held {
        label: String,
        op: String,
    },
    List {
        rows: Vec<ListRowOut>,
        #[serde(default)]
        sel: usize,
    },
    /// Outputs the reducer doesn't produce (toasts, the page setting the input, the page's
    /// window height: the app's window keeps one height while it shows).
    #[serde(other)]
    Other,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ListRowOut {
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub jev: bool,
    #[serde(default)]
    pub reading: bool,
    #[serde(default)]
    pub stale: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OutLine {
    pub t: Ms,
    #[serde(flatten)]
    pub out: Out,
}

fn held_words(op: HeldOp) -> &'static str {
    match op {
        HeldOp::Start => "start",
        HeldOp::Sent => "sent",
        HeldOp::Unsent => "unsent",
        HeldOp::Paused => "pause",
        HeldOp::Resumed => "resume",
        HeldOp::Rewritten => "rewrite",
    }
}

pub fn slot_kind_words(kind: SlotKind) -> &'static str {
    use SlotKind::*;
    match kind {
        Focused => "focused",
        Recent => "recent",
        Staged => "staged",
        Holding => "holding",
        Typing => "voice",
    }
}

/// Turns effects and changes in what's shown into trace lines.
#[derive(Clone, Debug, Default)]
pub struct Observer {
    last: Option<Snapshot>,
    /// The header as the page reports it: the list's first header, or the open view's own crumb
    /// words without the trail.
    last_header: Option<(String, Option<String>)>,
    last_list: Option<Out>,
    lines: Vec<OutLine>,
    /// Each held message's words as last reported: the page names a rewrite by the words it had.
    held_labels: HashMap<u64, String>,
}

impl Observer {
    pub fn new() -> Self {
        Self::default()
    }

    fn push(&mut self, t: Ms, out: Out) {
        if self.lines.last().is_some_and(|l| l.out == out) {
            return;
        }
        self.lines.push(OutLine { t, out });
    }

    /// Records what one page turn showed: `outcomes` are the reducer's answers to that turn's events.
    pub fn observe(&mut self, now: Ms, win: &OneWindow, outcomes: &[Outcome]) {
        for outcome in outcomes {
            if let Some((row, how)) = &outcome.picked {
                self.push(
                    now,
                    Out::Run {
                        key: row.key.clone(),
                        title: row.title.clone(),
                        how: match how {
                            RunHow::Match => "match",
                            RunHow::Slot => "slot",
                        }
                        .into(),
                    },
                );
            }
            for fx in &outcome.effects {
                let out = match fx {
                    Effect::EnterIgnored => Out::Ignored {
                        key: "Enter".into(),
                    },
                    Effect::Held { op, id, label, .. } => {
                        let was = self.held_labels.insert(*id, label.clone());
                        Out::Held {
                            op: held_words(*op).into(),
                            label: match op {
                                HeldOp::Rewritten => was.unwrap_or_else(|| label.clone()),
                                _ => label.clone(),
                            },
                        }
                    }
                    _ => continue,
                };
                self.push(now, out);
            }
        }
        let snap = win.snapshot(now);
        let prev = self.last.take();
        if prev.as_ref().is_none_or(|p| {
            !p.slot.same_words(&snap.slot) || (p.slot_opacity < 1.0) != (snap.slot_opacity < 1.0)
        }) {
            self.push(
                now,
                Out::Slot {
                    kind: slot_kind_words(snap.slot.kind).into(),
                    label: snap.slot.label.clone(),
                    title: snap.slot.title.clone(),
                    sub: snap.slot.sub.clone(),
                    dimmed: snap.slot_opacity < 1.0,
                },
            );
        }
        let header = match (&snap.view, win.views().top()) {
            (Some(v), Some(top)) => (top.label.clone(), Some(v.clone())),
            _ => (snap.header.clone(), None),
        };
        if self.last_header.as_ref() != Some(&header) {
            self.last_header = Some(header.clone());
            let (words, view) = header;
            self.push(now, Out::Header { words, view });
        }
        if snap.view.is_none() {
            let list = list_out(win);
            if self.last_list.as_ref() != Some(&list) {
                self.last_list = Some(list.clone());
                self.push(now, list);
            }
        } else {
            self.last_list = None;
        }
        if prev
            .as_ref()
            .is_none_or(|p| p.enter != snap.enter || p.armed != snap.armed)
        {
            self.push(
                now,
                Out::Enter {
                    id: snap.enter.id.clone(),
                    label: snap.enter.label.clone(),
                    armed: snap.armed,
                },
            );
        }
        self.last = Some(snap);
    }

    pub fn lines(&self) -> &[OutLine] {
        &self.lines
    }

    pub fn drain(&mut self) -> Vec<OutLine> {
        std::mem::take(&mut self.lines)
    }
}

/// The main list's rows as the page reports them: every selectable row in order, Jev's places
/// marked (empty while reading), and which one is selected.
fn list_out(win: &OneWindow) -> Out {
    let list = win.list();
    let rows = list
        .visible()
        .iter()
        .map(|item| match item {
            Item::Slot(i) => {
                let slot = list.jev.get(*i);
                let row = slot.and_then(|s| s.row.as_ref());
                ListRowOut {
                    key: row.map(|r| r.key.clone()),
                    title: row.map(|r| r.title.clone()).unwrap_or_default(),
                    jev: true,
                    reading: row.is_none() && !list.jev.settled_for(&list.text),
                    stale: slot.is_some_and(|s| s.stale),
                }
            }
            item => {
                let row = list.row(item);
                ListRowOut {
                    key: row.map(|r| r.key.clone()),
                    title: row.map(|r| r.title.clone()).unwrap_or_default(),
                    jev: false,
                    reading: false,
                    stale: false,
                }
            }
        })
        .collect();
    Out::List {
        rows,
        sel: list.sel,
    }
}

// ---------------------------------------------------------------------- input lines

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RowIn {
    pub key: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub enter_label: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub score: Option<f64>,
    #[serde(default)]
    pub head: Option<String>,
    #[serde(default)]
    pub fallback: bool,
    #[serde(default)]
    pub learned: bool,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub sub: Option<String>,
    #[serde(default)]
    pub p: Option<f32>,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub verb: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub fill: Option<String>,
}

impl From<RowIn> for Row {
    fn from(r: RowIn) -> Self {
        let mut row = Row::new(r.key, r.title);
        row.enter_label = r.enter_label;
        row.sub = r.sub;
        row.p = r.p;
        row.target = r.target;
        row.verb = r.verb;
        row.source = r.source;
        row.fill = r.fill;
        row.icon = r.icon;
        row.score = r.score.map(|s| s.clamp(0.0, 255.0) as u8).unwrap_or(100);
        row.head = r.head;
        row.fallback = r.fallback;
        row.learned = r.learned;
        row
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct EnterIn {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub label: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct FocusedIn {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub sub: String,
    #[serde(default)]
    pub icon: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct HistoryIn {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub risk: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub said: String,
    #[serde(default)]
    pub at: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "ev", rename_all = "camelCase")]
pub enum Ev {
    #[serde(rename_all = "camelCase")]
    Init {
        #[serde(default)]
        hold_ms: Option<u64>,
        #[serde(default)]
        focused: Option<FocusedIn>,
        #[serde(default)]
        history: Vec<HistoryIn>,
    },
    Text {
        text: String,
    },
    Key {
        key: String,
    },
    Names {
        #[serde(default)]
        text: String,
        #[serde(default)]
        rows: Vec<RowIn>,
    },
    Jev {
        #[serde(default)]
        text: String,
        #[serde(default)]
        rows: Vec<RowIn>,
        #[serde(default)]
        down: bool,
    },
    #[serde(rename_all = "camelCase")]
    View {
        op: String,
        /// `null` on a `back` that leaves no view open.
        #[serde(default)]
        name: Option<String>,
        #[serde(default)]
        label: String,
        #[serde(default)]
        enter: Option<EnterIn>,
        #[serde(default)]
        text: Option<String>,
        #[serde(default)]
        takes_input: bool,
        #[serde(default)]
        leave: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    ViewEnter {
        #[serde(default)]
        id: String,
        #[serde(default)]
        label: String,
        /// Whether the view that set it was on top.
        #[serde(default = "on_top")]
        on_top: bool,
    },
    Done {
        #[serde(default)]
        title: String,
        #[serde(default)]
        icon: String,
        #[serde(default)]
        risk: Option<String>,
        #[serde(default)]
        form: Option<String>,
        #[serde(default)]
        stay: bool,
    },
    Hold {
        label: String,
        #[serde(default)]
        request: String,
    },
    Rewrite {
        label: String,
        text: String,
    },
    /// Stop a waiting message by its words, or `"all"`.
    Unsend {
        label: String,
    },
    /// The prototype's window measured its content; the app's height doesn't follow it.
    Content {
        px: f32,
    },
    Dismiss {
        #[serde(default)]
        app: Option<String>,
    },
    Summon,
    /// What ⌘K would act on changed without a key.
    Focused(FocusedIn),
    /// A view or page set (or cleared) the line above the input.
    Line(LineIn),
    /// The open view changed its crumb without opening anything.
    ViewLabel {
        label: String,
    },
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct LineIn {
    #[serde(default)]
    pub clear: bool,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub sub: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub hints: Vec<(String, String)>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EvLine {
    pub t: Ms,
    /// What caused this input on the page, when another input did ("key:Enter", "done").
    #[serde(default)]
    pub via: Option<String>,
    #[serde(flatten)]
    pub ev: Ev,
}

/// Whether a recorded input is something the host does (and a replay must feed), rather than a
/// consequence the reducer produces itself from an earlier input: Esc, ⌘K and ↑ go back or home on
/// their own, a finished form goes home and holds its message, a stopped message finishes.
fn replayable(line: &EvLine) -> bool {
    let via = line.via.as_deref().unwrap_or("");
    let by_key = via.starts_with("key:");
    match &line.ev {
        Ev::View { op, text, .. } => match op.as_str() {
            "home" => {
                via.is_empty()
                    || via == "key:Enter"
                    || text.as_deref().is_some_and(|t| !t.is_empty())
            }
            "back" => via != "key:Escape",
            _ => true,
        },
        Ev::Done { .. } => !matches!(via, "unsend" | "rewrite"),
        Ev::Hold { .. } => via != "done",
        Ev::Key { .. } => !by_key,
        _ => true,
    }
}

/// One parsed line of a trace file.
#[derive(Clone, Debug)]
pub enum Line {
    Meta(serde_json::Value),
    Ev(EvLine),
    Out(OutLine),
    /// A line this version doesn't know; kept so a replay can report it.
    Unknown(serde_json::Value),
}

pub fn parse_line(text: &str) -> Result<Line, serde_json::Error> {
    let v: serde_json::Value = serde_json::from_str(text)?;
    if let Some(meta) = v.get("meta") {
        return Ok(Line::Meta(meta.clone()));
    }
    if v.get("ev").is_some() {
        return Ok(match serde_json::from_value::<EvLine>(v.clone()) {
            Ok(ev) => Line::Ev(ev),
            Err(_) => Line::Unknown(v),
        });
    }
    if v.get("out").is_some() {
        return Ok(match serde_json::from_value::<OutLine>(v.clone()) {
            Ok(out) => Line::Out(out),
            Err(_) => Line::Unknown(v),
        });
    }
    Ok(Line::Unknown(v))
}

pub fn parse_risk(risk: Option<&str>) -> Option<Risk> {
    match risk? {
        "hold" => Some(Risk::Hold),
        "undo" => Some(Risk::Undo),
        "none" => Some(Risk::None),
        _ => None,
    }
}

pub fn parse_key(key: &str) -> Key {
    match key {
        "Enter" => Key::Enter,
        "CmdEnter" => Key::CmdEnter,
        "Escape" => Key::Escape,
        "ArrowUp" => Key::Up,
        "ArrowDown" => Key::Down,
        "ArrowRight" => Key::Right,
        "CmdK" => Key::CmdK,
        "CmdJ" => Key::CmdJ,
        "CmdZ" => Key::CmdZ,
        "Tab" => Key::Tab,
        k => match k.strip_prefix("Cmd").and_then(|d| d.parse::<u8>().ok()) {
            Some(d) => Key::CmdDigit(d),
            None => Key::Other(k.to_string()),
        },
    }
}

pub fn subject_of(f: FocusedIn) -> Subject {
    let kind = match f.kind.as_str() {
        "selection" => SubjectKind::Selection,
        "window" => SubjectKind::Window,
        "app" => SubjectKind::App,
        "message" => SubjectKind::Message,
        "entry" => SubjectKind::Entry,
        "script" => SubjectKind::Script,
        "answer" => SubjectKind::Answer,
        _ => SubjectKind::Thing,
    };
    Subject {
        kind,
        label: f.label,
        title: f.title,
        sub: f.sub,
        icon: f.icon,
        entry: None,
        row_key: None,
    }
}

pub fn history_entry(h: HistoryIn) -> Entry {
    Entry {
        id: 0,
        at: 0,
        icon: if h.icon.is_empty() {
            "check".into()
        } else {
            h.icon
        },
        title: h.title,
        said: h.said,
        risk: parse_risk(h.risk.as_deref()).unwrap_or(Risk::None),
        state: match h.state.as_deref() {
            Some("undone") => EntryState::Undone,
            Some("sent") => EntryState::Sent,
            _ => EntryState::Done,
        },
        app: None,
        script: None,
        held: None,
        seeded: true,
        when: h.at,
        undo_of: None,
    }
}

/// Which of the prototype's views clear the input on the way back rather than restore what was typed.
fn prototype_view_leaves_empty(name: &str) -> bool {
    matches!(name, "agent" | "dictation" | "shape")
}

/// The reducer events one input line stands for, given the host row the reducer ran earlier in
/// the same page turn (`turn_run`) and the window as it is now.
///
/// A launch that hides the window records the row that ran; a dismissal nothing ran is the user
/// going elsewhere. Going home right after a host row ran means the row finished on the spot. While
/// something is staged, the rows the host pins for the empty input are what it offers to do with it.
pub fn events_of(line: EvLine, turn_run: Option<&Row>, win: &OneWindow) -> Vec<Event> {
    if !replayable(&line) {
        return Vec::new();
    }
    let by_key = line.via.as_deref().is_some_and(|v| v.starts_with("key:"));
    let ev = line.ev;
    match ev {
        Ev::Init { focused, .. } => focused
            .map(|f| Event::Focused(subject_of(f)))
            .into_iter()
            .collect(),
        Ev::Text { text } => vec![Event::Text(text)],
        Ev::Key { key } => vec![Event::Key(parse_key(&key))],
        Ev::Names { text, rows } => {
            let pinned = !rows.is_empty() && rows.iter().all(|r| r.pinned);
            let rows: Vec<Row> = rows.into_iter().map(Row::from).collect();
            match (pinned && text.trim().is_empty(), win.staged().is_some()) {
                (true, true) => vec![Event::Actions(rows)],
                (true, false) => vec![Event::Suggested(rows)],
                (false, _) => vec![Event::Names(rows)],
            }
        }
        Ev::Jev { text, rows, down } => {
            if down {
                vec![Event::JevDown { text }]
            } else {
                vec![Event::Jev {
                    text,
                    rows: rows.into_iter().map(Row::from).collect(),
                }]
            }
        }
        Ev::View {
            op,
            name,
            label,
            enter,
            text,
            takes_input,
            leave,
        } => {
            let name = name.unwrap_or_default();
            let leave_text = leave.or_else(|| prototype_view_leaves_empty(&name).then(String::new));
            let spec = ViewSpec {
                label: if label.is_empty() {
                    name.clone()
                } else {
                    label
                },
                name,
                enter: enter.map(|e| EnterTarget::new(e.id, e.label)),
                takes_input,
                text,
                leave_text,
            };
            let op = match op.as_str() {
                "open" => ViewOp::Open(spec),
                "replace" => ViewOp::Replace(spec),
                "back" => ViewOp::Back,
                "home" => match (spec.text, turn_run) {
                    (Some(text), _) if !text.is_empty() => ViewOp::Home { text },
                    (_, Some(row)) if by_key => {
                        return vec![Event::Ran {
                            row: row.clone(),
                            app: None,
                        }];
                    }
                    (_, None) if by_key => return Vec::new(),
                    (text, _) => ViewOp::Home {
                        text: text.unwrap_or_default(),
                    },
                },
                "park" => ViewOp::Park,
                "reopen" => ViewOp::Reopen(spec),
                _ => return Vec::new(),
            };
            vec![Event::View(op)]
        }
        Ev::ViewEnter { id, label, on_top } => {
            if on_top {
                return vec![Event::ViewEnter(EnterTarget::new(id, label))];
            }
            // The prototype's Enter ids start with their view's name ("steps:all").
            let view = id.split(':').next().unwrap_or_default().to_string();
            vec![Event::ViewEnterBelow {
                view,
                target: EnterTarget::new(id, label),
            }]
        }
        Ev::Done {
            title,
            icon,
            risk,
            form,
            stay,
        } => vec![Event::Done(Done {
            icon: if icon.is_empty() {
                "check".into()
            } else {
                icon
            },
            title,
            risk: parse_risk(risk.as_deref()),
            form_kind: form,
            stay,
        })],
        Ev::Hold { label, request } => vec![Event::Hold { label, request }],
        Ev::Rewrite { label, text } => vec![Event::Rewrite {
            target: label,
            text,
        }],
        Ev::Unsend { label } => vec![Event::Unsend {
            target: (label != "all").then_some(label),
        }],
        Ev::Content { .. } => vec![],
        Ev::Dismiss { app } => match turn_run {
            Some(row) if by_key => vec![Event::Ran {
                row: row.clone(),
                app: Some(app.unwrap_or_default()),
            }],
            _ => vec![Event::Dismissed],
        },
        Ev::Summon => vec![Event::Summon],
        Ev::Focused(f) => vec![Event::Focused(subject_of(f))],
        Ev::Line(l) => vec![Event::Line(line_of(l))],
        Ev::ViewLabel { label } => vec![Event::View(ViewOp::Label(label))],
    }
}

fn on_top() -> bool {
    true
}

fn line_of(l: LineIn) -> Option<SlotLine> {
    if l.clear {
        return None;
    }
    let kind = match l.kind.as_str() {
        "focused" => SlotKind::Focused,
        "recent" => SlotKind::Recent,
        "holding" => SlotKind::Holding,
        "voice" => SlotKind::Typing,
        _ => SlotKind::Staged,
    };
    let mut line = SlotLine::new(kind, l.label, l.title)
        .with_icon(l.icon)
        .with_sub(l.sub);
    for (key, words) in l.hints {
        line = line.with_hint(key, words);
    }
    Some(line)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fed(lines: &[&str]) -> OneWindow {
        let mut win = OneWindow::default();
        for (t, text) in lines.iter().enumerate() {
            let Line::Ev(line) = parse_line(text).expect("json") else {
                panic!("not an input: {text}");
            };
            for ev in events_of(line, None, &win) {
                win.handle(t as Ms, ev);
            }
        }
        win
    }

    #[test]
    fn a_view_relabels_its_crumb_without_opening() {
        let win = fed(&[
            r#"{"t":0,"ev":"view","op":"open","name":"answer","label":"Answer","takesInput":false}"#,
            r#"{"t":1,"ev":"viewLabel","label":"How Jev reads it"}"#,
        ]);
        assert_eq!(win.snapshot(2).crumb, "How Jev reads it");
    }

    #[test]
    fn an_enter_set_under_the_top_view_waits_for_it() {
        let mut win = fed(&[
            r#"{"t":0,"ev":"view","op":"open","name":"steps","label":"Steps","takesInput":true,"enter":{"id":"steps:all","label":"Connect it first"}}"#,
            r#"{"t":1,"ev":"view","op":"open","name":"connect","label":"Connect Google Calendar","takesInput":false,"enter":{"id":"connect","label":"Connect"}}"#,
            r#"{"t":2,"ev":"viewEnter","id":"steps:all","label":"Do all 3","onTop":false}"#,
            r#"{"t":3,"ev":"viewEnter","id":"agent:running","label":"It's working","onTop":false}"#,
        ]);
        assert_eq!(win.enter_target(4).label, "Connect");
        win.handle(5, Event::View(ViewOp::Back));
        assert_eq!(win.enter_target(5).label, "Do all 3");
    }

    #[test]
    fn a_line_shows_until_cleared() {
        let win = fed(&[
            r#"{"t":0,"ev":"line","kind":"voice","label":"Typing into","title":"Mail","sub":"To · Re: beta","icon":"type","hints":[["esc","drop"]]}"#,
        ]);
        let slot = win.recent_line(1).unwrap();
        assert_eq!(slot.kind, SlotKind::Typing);
        assert_eq!(slot.label, "Typing into");
        assert_eq!(slot.hints, vec![("esc".to_string(), "drop".to_string())]);
        let win = fed(&[
            r#"{"t":0,"ev":"line","kind":"voice","label":"Typing into","title":"Mail"}"#,
            r#"{"t":1,"ev":"line","clear":true}"#,
        ]);
        assert!(win.recent_line(2).is_none());
        assert_eq!(
            win.slot_line(2).label,
            crate::contract::words::BEHIND,
            "the line is never empty"
        );
    }

    #[test]
    fn focus_changes_what_the_chip_offers() {
        let win = fed(&[
            r#"{"t":0,"ev":"focused","kind":"selection","label":"Selected","title":"“the footer kept the old label”","sub":"in Chrome","icon":"type"}"#,
        ]);
        let chip = win.chip().expect("offered");
        assert_eq!(chip.title, "“the footer kept the old label”");
        assert_eq!(chip.icon, "type");
    }

    #[test]
    fn an_open_answer_is_what_this_means() {
        let mut win = fed(&[
            r#"{"t":0,"ev":"focused","kind":"answer","label":"Focused","title":"Sarah, Mo, Priya","sub":"in Standup · the open answer","icon":"same-for"}"#,
        ]);
        assert_eq!(win.chip().unwrap().title, "Sarah, Mo, Priya");
        win.handle(2, Event::Key(Key::CmdK));
        let staged = win.staged().expect("⌘K stages the answer");
        assert_eq!(staged.subject.kind, SubjectKind::Answer);
        assert_eq!(staged.rows[0].title, "Open it");
    }

    #[test]
    fn a_rewrite_is_reported_by_the_words_it_had() {
        let mut win = OneWindow::default();
        let mut obs = Observer::new();
        let out = win.handle(
            0,
            Event::Hold {
                label: "To Sarah: “It'll be friday”".into(),
                request: "tell Sarah It'll be friday".into(),
            },
        );
        obs.observe(0, &win, &[out]);
        let out = win.handle(
            10,
            Event::Rewrite {
                target: "To Sarah: “It'll be friday”".into(),
                text: "It'll be thursday".into(),
            },
        );
        obs.observe(10, &win, &[out]);
        let held: Vec<String> = obs
            .drain()
            .into_iter()
            .filter_map(|l| match l.out {
                Out::Held { op, label } => Some(format!("{op}|{label}")),
                _ => None,
            })
            .collect();
        assert_eq!(
            held[..2],
            [
                "start|To Sarah: “It'll be friday”",
                "rewrite|To Sarah: “It'll be friday”"
            ]
        );
    }

    #[test]
    fn an_unsend_stops_the_named_message_once() {
        let win = fed(&[
            r#"{"t":0,"ev":"hold","label":"To #launcher: “The beta slipped a week”"}"#,
            r#"{"t":1,"ev":"unsend","label":"To #launcher: “The beta slipped a week”"}"#,
            r#"{"t":2,"ev":"unsend","label":"To #launcher: “The beta slipped a week”"}"#,
        ]);
        assert_eq!(win.holds().live_count(), 0);
    }
}
