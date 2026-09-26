//! Golden traces: what the prototype was fed and what it showed, as JSON lines.
//!
//! Input lines (`{"t":..,"ev":"text",...}`) become [`Event`]s; output lines (`{"t":..,"out":"slot",...}`)
//! are what the reducer must reproduce. The [`Observer`] writes the reducer's own output lines in the
//! same shape so the two can be compared.

use serde::{Deserialize, Serialize};

use crate::Ms;
use crate::enter::EnterTarget;
use crate::height::HeightMode;
use crate::ledger::{Entry, EntryState, Risk};
use crate::row::Row;
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

pub fn slot_kind_words(kind: crate::slot::SlotKind) -> &'static str {
    use crate::slot::SlotKind::*;
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
                Effect::Held { op, label, .. } => Out::Held {
                    op: held_words(*op).into(),
                    label: label.clone(),
                },
                _ => continue,
            };
            self.push(now, out);
        }
        let snap = win.snapshot(now);
        let prev = self.last.take();
        if prev
            .as_ref()
            .is_none_or(|p| !p.slot.same_words(&snap.slot) || p.slot.dimmed() != snap.slot.dimmed())
        {
            self.push(
                now,
                Out::Slot {
                    kind: slot_kind_words(snap.slot.kind).into(),
                    label: snap.slot.label.clone(),
                    title: snap.slot.title.clone(),
                    sub: snap.slot.sub.clone(),
                    dimmed: snap.slot.dimmed(),
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
}

impl From<RowIn> for Row {
    fn from(r: RowIn) -> Self {
        let mut row = Row::new(r.key, r.title);
        row.enter_label = r.enter_label;
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
        takes_input: bool,
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
    Content {
        px: f32,
    },
    Dismiss {
        #[serde(default)]
        app: Option<String>,
    },
    Summon,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EvLine {
    pub t: Ms,
    #[serde(flatten)]
    pub ev: Ev,
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
        Ev::Names { rows, .. } => vec![Event::Names(rows.into_iter().map(Row::from).collect())],
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
        } => {
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
    }
}
