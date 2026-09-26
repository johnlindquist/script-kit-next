//! Golden traces: what the prototype was fed and what it showed, as JSON lines.
//!
//! Input lines (`{"t":..,"ev":"text",...}`) become [`Event`]s; output lines (`{"t":..,"out":"slot",...}`)
//! are what the reducer must reproduce. The [`Observer`] writes the reducer's own output lines in the
//! same shape so the two can be compared.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Ms;
use crate::enter::EnterTarget;
use crate::height::HeightMode;
use crate::ledger::{Entry, EntryState, Risk};
use crate::row::Row;
use crate::slot::{SlotKind, SlotLine};
use crate::stage::{Subject, SubjectKind};
use crate::window::{
    Done, Effect, Event, HeldOp, Key, OneWindow, RunHow, Snapshot, ViewOp, ViewSpec,
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
    Height {
        px: f32,
        mode: String,
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
    /// Outputs the reducer doesn't produce (toasts, the page setting the input).
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
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OutLine {
    pub t: Ms,
    #[serde(flatten)]
    pub out: Out,
}

fn mode_words(mode: HeightMode) -> &'static str {
    match mode {
        HeightMode::Grow => "grow",
        HeightMode::Hold => "hold",
        HeightMode::Shrink => "shrink",
    }
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
        Typing => "typing",
    }
}

/// Turns effects and changes in what's shown into trace lines.
#[derive(Clone, Debug, Default)]
pub struct Observer {
    last: Option<Snapshot>,
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

    pub fn observe(&mut self, now: Ms, win: &OneWindow, effects: &[Effect]) {
        for fx in effects {
            let out = match fx {
                Effect::Run { row, how } => Out::Run {
                    key: row.key.clone(),
                    title: row.title.clone(),
                    how: match how {
                        RunHow::Match => "match",
                        RunHow::Slot => "slot",
                    }
                    .into(),
                },
                Effect::EnterIgnored => Out::Ignored {
                    key: "Enter".into(),
                },
                Effect::Height(d) => Out::Height {
                    px: d.px,
                    mode: mode_words(d.mode).into(),
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
        let (words, view) = match &snap.view {
            Some(v) => (snap.crumb.clone(), Some(v.clone())),
            None => (snap.header.clone(), None),
        };
        let header_changed = prev.as_ref().is_none_or(|p| {
            let (pw, pv) = match &p.view {
                Some(v) => (p.crumb.as_str(), Some(v)),
                None => (p.header.as_str(), None),
            };
            pw != words || pv != view.as_ref()
        });
        if header_changed {
            self.push(now, Out::Header { words, view });
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
    View {
        op: String,
        #[serde(default)]
        name: String,
        #[serde(default)]
        label: String,
        #[serde(default)]
        enter: Option<EnterIn>,
        #[serde(default)]
        text: Option<String>,
        #[serde(default)]
        takes_input: Option<bool>,
        #[serde(default)]
        leave: Option<String>,
    },
    ViewEnter {
        #[serde(default)]
        id: String,
        #[serde(default)]
        label: String,
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
/// consequence the reducer produces itself from an earlier input.
pub fn replayable(line: &EvLine) -> bool {
    let via = line.via.as_deref().unwrap_or("");
    let by_key = via.starts_with("key:");
    match &line.ev {
        Ev::View { op, .. } => match op.as_str() {
            "home" => {
                via.is_empty()
                    || matches!(&line.ev, Ev::View { text: Some(t), .. } if !t.is_empty())
            }
            "back" => !by_key,
            _ => true,
        },
        Ev::Done { .. } => via != "unsend",
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

/// Which of the prototype's views read typing themselves (the rest close when you type), for
/// traces recorded before views carried `takesInput`.
fn prototype_view_takes_input(name: &str) -> bool {
    matches!(
        name,
        "form"
            | "agent"
            | "note"
            | "note-open"
            | "dictation"
            | "steps"
            | "set"
            | "save"
            | "list"
            | "chat"
    )
}

/// Which of the prototype's views clear the input on the way back rather than restore what was typed.
fn prototype_view_leaves_empty(name: &str) -> bool {
    matches!(name, "agent" | "dictation" | "shape")
}

/// The reducer events one input line stands for. `last_run` is the last row the reducer ran, for
/// dismissals that follow launching it.
pub fn events_of(ev: Ev, last_run: Option<&Row>) -> Vec<Event> {
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
            if pinned && text.trim().is_empty() {
                vec![Event::Suggested(rows)]
            } else {
                vec![Event::Names(rows)]
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
            let takes_input = takes_input.unwrap_or_else(|| prototype_view_takes_input(&name));
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
                "home" => ViewOp::Home {
                    text: spec.text.unwrap_or_default(),
                },
                "park" => ViewOp::Park,
                "reopen" => ViewOp::Reopen(spec),
                _ => return Vec::new(),
            };
            vec![Event::View(op)]
        }
        Ev::ViewEnter { id, label } => vec![Event::ViewEnter(EnterTarget::new(id, label))],
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
        Ev::Content { px } => vec![Event::Content { px }],
        Ev::Dismiss { app } => {
            let row = last_run.cloned().unwrap_or_else(|| {
                Row::new("dismiss", app.clone().unwrap_or_default()).with_icon("app")
            });
            vec![Event::Ran {
                row,
                app: Some(app.unwrap_or_default()),
            }]
        }
        Ev::Summon => vec![Event::Summon],
        Ev::Focused(f) => vec![Event::Focused(subject_of(f))],
        Ev::Line(l) => vec![Event::Line(line_of(l))],
        Ev::ViewLabel { label } => vec![Event::View(ViewOp::Label(label))],
    }
}

fn line_of(l: LineIn) -> Option<SlotLine> {
    if l.clear {
        return None;
    }
    let kind = match l.kind.as_str() {
        "focused" => SlotKind::Focused,
        "recent" => SlotKind::Recent,
        "holding" => SlotKind::Holding,
        "voice" | "typing" => SlotKind::Typing,
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
            for ev in events_of(line.ev, None) {
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
    fn a_line_shows_until_cleared() {
        let win = fed(&[
            r#"{"t":0,"ev":"line","kind":"voice","label":"Typing into","title":"Mail","sub":"To · Re: beta","icon":"type","hints":[["esc","drop"]]}"#,
        ]);
        let slot = win.slot_line(1);
        assert_eq!(slot.kind, SlotKind::Typing);
        assert_eq!(slot.label, "Typing into");
        assert_eq!(slot.hints, vec![("esc".to_string(), "drop".to_string())]);
        let win = fed(&[
            r#"{"t":0,"ev":"line","kind":"voice","label":"Typing into","title":"Mail"}"#,
            r#"{"t":1,"ev":"line","clear":true}"#,
        ]);
        assert_eq!(win.slot_line(2).kind, SlotKind::Focused);
    }

    #[test]
    fn focus_changes_what_the_line_calls_focused() {
        let win = fed(&[
            r#"{"t":0,"ev":"focused","kind":"selection","label":"Selected","title":"“the footer kept the old label”","sub":"in Chrome","icon":"type"}"#,
        ]);
        let slot = win.slot_line(1);
        assert_eq!(slot.title, "“the footer kept the old label”");
        assert_eq!(slot.sub, "in Chrome");
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
        obs.observe(0, &win, &out.effects);
        let out = win.handle(
            10,
            Event::Rewrite {
                target: "To Sarah: “It'll be friday”".into(),
                text: "It'll be thursday".into(),
            },
        );
        obs.observe(10, &win, &out.effects);
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
