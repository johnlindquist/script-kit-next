//! Replays the prototype's golden traces through the reducer and compares what it shows.
//!
//! Traces are recorded on the page's clock with a 200 ms hold ticker and a real layout engine, so
//! the comparison is per kind of output, in order, after dropping consecutive repeats. What the
//! window shows (the line above the input, the first header, what ↵ says, the list) is compared as
//! each page turn left it; what happened (what ran, ignored Enters, held messages) is compared
//! event by event. The page's window height isn't compared: the app's window keeps one height
//! while it shows.

use std::path::{Path, PathBuf};

use sk_one_window::contract::HOLD_TICK_MS;
use sk_one_window::trace::{Ev, EvLine, Line, Observer, Out, OutLine, events_of, parse_line};
use sk_one_window::window::{Config, Effect, Event, OneWindow, Outcome};
use sk_one_window::{Row, Subject};

fn traces() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/traces");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map(|d| d.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default();
    files.retain(|p| p.extension().is_some_and(|e| e == "jsonl"));
    files.sort();
    files
}

/// The rules the prototype's page recorded these traces with: its line kept the last run until
/// something else ran and never named the chat's reply.
fn prototype() -> Config {
    Config {
        subject_chip: false,
        ..Config::default()
    }
}

#[derive(Default)]
struct Replay {
    /// What the page showed, each with the turn that showed it.
    expected: Vec<(OutLine, usize)>,
    got: Vec<OutLine>,
    turns: Vec<Turn>,
    /// The last time the trace records.
    end: u64,
    /// Lines this replay can't read, by their `ev`/`out` kind.
    unknown: Vec<String>,
}

/// Feeds a trace to the reducer one page turn at a time: an input with no `via` starts a turn,
/// inputs it caused join it and are fed at the turn's time (so the reducer sees them as part of
/// the same keypress), and what's shown is observed when the turn ends.
struct Player {
    win: OneWindow,
    obs: Observer,
    /// The host row the reducer ran in this turn.
    turn_run: Option<Row>,
    /// What was staged when this turn began.
    turn_staged: Option<Subject>,
    wake: Option<u64>,
    turn: Option<u64>,
    listed: bool,
    outcomes: Vec<Outcome>,
    turns: Vec<Turn>,
}

/// A page turn the replay observed: one started by an input, or by one of the reducer's timers.
#[derive(Clone, Copy)]
struct Turn {
    at: u64,
    tick: bool,
}

impl Player {
    fn new(win: OneWindow) -> Self {
        Self {
            win,
            obs: Observer::new(),
            turn_run: None,
            turn_staged: None,
            wake: None,
            turn: None,
            listed: false,
            outcomes: Vec::new(),
            turns: Vec::new(),
        }
    }

    fn feed(&mut self, t: u64, ev: Event) {
        let out = self.win.handle(t, ev);
        for fx in &out.effects {
            match fx {
                Effect::WakeAt(at) => self.wake = Some(*at),
                Effect::Run { row, .. } => self.turn_run = Some(row.clone()),
                _ => {}
            }
        }
        self.outcomes.push(out);
    }

    fn close_turn(&mut self) {
        if let Some(t) = self.turn.take() {
            let outcomes = std::mem::take(&mut self.outcomes);
            self.obs.observe(t, &self.win, &outcomes);
            self.turn_run = None;
        }
    }

    /// Delivers the reducer's timers that fall before `t`, each in its own turn.
    fn ticks_before(&mut self, t: u64) {
        while let Some(at) = self.wake.filter(|at| *at < t) {
            self.close_turn();
            self.wake = None;
            self.turn = Some(at);
            self.turns.push(Turn { at, tick: true });
            self.feed(at, Event::Tick);
            self.close_turn();
        }
    }

    fn staged(&self) -> Option<Subject> {
        self.win.staged().map(|s| s.subject.clone())
    }

    /// The page's startup is one turn up to its first list (a `names` line is written after the
    /// page drew it). `render` rows changed on their own and are a turn of their own, unless this
    /// turn staged or let go of something: those are the rows it offers, drawn in the same keypress.
    fn input(&mut self, line: EvLine) {
        let starts_turn = match (line.via.as_deref(), &line.ev) {
            _ if self.turn.is_none() => true,
            (Some("render"), _) => self.staged() == self.turn_staged,
            _ if !self.listed => false,
            (via, _) => via.is_none(),
        };
        if matches!(line.ev, Ev::Names { .. }) {
            self.listed = true;
        }
        if starts_turn {
            self.ticks_before(line.t);
            self.close_turn();
            self.turn = Some(line.t);
            self.turn_staged = self.staged();
            self.turns.push(Turn {
                at: line.t,
                tick: false,
            });
        }
        let t = self.turn.unwrap_or(line.t);
        for e in events_of(line, self.turn_run.as_ref(), &self.win) {
            self.feed(t, e);
        }
    }
}

fn replay(path: &Path) -> Replay {
    let text = std::fs::read_to_string(path).expect("trace reads");
    let mut r = Replay::default();
    let mut player: Option<Player> = None;
    let mut end = 0;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        if let Some(t) = serde_json::from_str::<serde_json::Value>(line)
            .ok()
            .and_then(|v| v.get("t").and_then(serde_json::Value::as_u64))
        {
            end = end.max(t);
        }
        match parse_line(line).expect("trace line is JSON") {
            Line::Meta(_) => {}
            Line::Unknown(v) => r.unknown.push(
                ["ev", "out"]
                    .iter()
                    .find_map(|k| {
                        v.get(*k)
                            .and_then(|x| x.as_str())
                            .map(|x| format!("{k}:{x}"))
                    })
                    .unwrap_or_else(|| "?".into()),
            ),
            Line::Out(out) => {
                let turn = player
                    .as_ref()
                    .map_or(0, |p| p.turns.len().saturating_sub(1));
                r.expected.push((out, turn));
            }
            Line::Ev(ev) => {
                if let Ev::Init {
                    hold_ms, history, ..
                } = &ev.ev
                {
                    let cfg = Config {
                        hold_ms: hold_ms.unwrap_or(Config::default().hold_ms),
                        ..prototype()
                    };
                    let history = history
                        .iter()
                        .cloned()
                        .map(sk_one_window::trace::history_entry)
                        .collect();
                    player = Some(Player::new(OneWindow::new(cfg).with_history(history)));
                }
                player
                    .get_or_insert_with(|| Player::new(OneWindow::new(prototype())))
                    .input(ev);
            }
        }
    }
    if let Some(p) = player.as_mut() {
        // The page notices a pause or a resume on its next hold tick, so a wait it ends the
        // recording on can finish up to a couple of ticks later here.
        p.ticks_before(end + 2 * HOLD_TICK_MS + 1);
        p.close_turn();
        r.got = p.obs.drain();
        r.turns = std::mem::take(&mut p.turns);
    }
    r.end = end;
    // An output is written after the input that caused it, so it belongs to the last input turn
    // before it in the file, or to a timer turn that followed that input and began before it.
    for (out, turn) in &mut r.expected {
        while r
            .turns
            .get(*turn + 1)
            .is_some_and(|next| next.tick && next.at <= out.t)
        {
            *turn += 1;
        }
    }
    if let Some(dir) = std::env::var_os("OW_REPLAY_DUMP") {
        let name = path.file_stem().unwrap().to_string_lossy();
        let out: String = r
            .got
            .iter()
            .map(|l| serde_json::to_string(l).unwrap() + "\n")
            .collect();
        std::fs::write(Path::new(&dir).join(format!("{name}.got.jsonl")), out).expect("dump");
        let want: String = r
            .expected
            .iter()
            .map(|(l, turn)| {
                let mut v = serde_json::to_value(l).unwrap();
                v["turn"] = (*turn).into();
                v.to_string() + "\n"
            })
            .collect();
        std::fs::write(Path::new(&dir).join(format!("{name}.want.jsonl")), want).expect("dump");
    }
    r
}

fn kind(out: &Out) -> &'static str {
    match out {
        Out::Slot { .. } => "slot",
        Out::Header { .. } => "header",
        Out::Enter { .. } => "enter",
        Out::Run { .. } => "run",
        Out::Ignored { .. } => "ignored",
        Out::Held { .. } => "held",
        Out::List { .. } => "list",
        Out::Other => "other",
    }
}

/// The comparable words of an output: counters and pixel values are left out.
fn key(out: &Out) -> String {
    match out {
        Out::Slot {
            kind,
            label,
            title,
            dimmed,
            ..
        } => {
            let label = if label.starts_with("Sending in") {
                "Sending in"
            } else {
                label
            };
            format!("{kind}|{label}|{title}|{dimmed}")
        }
        Out::Header { words, view } => format!("{words}|{}", view.as_deref().unwrap_or("")),
        Out::Enter { label, .. } => label.clone(),
        Out::Run { key, .. } => key.clone(),
        Out::Ignored { key } => key.clone(),
        Out::Held { op, label } => format!("{op}|{label}"),
        Out::List { rows, sel } => {
            // The page keeps all three of Jev's places; the app draws the first and the ones
            // holding a reading, so an empty later place is left out of both sides.
            let mut first_place = true;
            let mut at = *sel;
            let mut shown = Vec::new();
            for (i, r) in rows.iter().enumerate() {
                let first = r.jev && std::mem::take(&mut first_place);
                if r.jev && r.key.is_none() && !first {
                    if i < *sel {
                        at -= 1;
                    }
                    continue;
                }
                shown.push(match (&r.key, r.jev) {
                    (Some(k), _) => k.clone(),
                    (None, true) => "jev:·".into(),
                    (None, false) => r.title.clone(),
                });
            }
            format!("{}@{at}", shown.join(","))
        }
        Out::Other => String::new(),
    }
}

/// Kinds that are a state of the window rather than something that happened: the page can draw
/// several within one turn (a list painted for the old words before the input clears), and the
/// reducer shows what the turn settled on.
const STATES: [&str; 4] = ["slot", "header", "enter", "list"];

fn seq<'a>(lines: impl Iterator<Item = &'a OutLine>, k: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for l in lines.filter(|l| kind(&l.out) == k) {
        let s = key(&l.out);
        if out.last() != Some(&s) {
            out.push(s);
        }
    }
    out
}

/// The page's outputs of kind `k`, keeping only the last of a state kind in each turn.
fn settled(expected: &[(OutLine, usize)], k: &str) -> Vec<String> {
    let of_kind: Vec<&(OutLine, usize)> =
        expected.iter().filter(|(l, _)| kind(&l.out) == k).collect();
    let keep = of_kind.iter().enumerate().filter(|(i, (_, turn))| {
        !STATES.contains(&k) || of_kind.get(i + 1).is_none_or(|(_, next)| next != turn)
    });
    seq(keep.map(|(_, (l, _))| l), k)
}

/// Differences these traces can't settle, and why. Each one must still differ: once a trace
/// records what's missing, its line comes off this list.
const KNOWN: &[(&str, &str, &str)] = &[
    (
        "s06-clipboard",
        "slot",
        "the page stages the clipboard on ⌘V (win.stage) and no input records it",
    ),
    (
        "s10-steps",
        "slot",
        "s15's page keeps a saved run last when its messages go out; s10's page lets the message \
         become last; the reducer follows s15",
    ),
    (
        "s13-history",
        "slot",
        "the history view shares an entry (win.share) and no input records it",
    ),
    (
        "s13-history",
        "run",
        "the history view shares an entry (win.share) and no input records it",
    ),
    (
        "s15-scripts",
        "slot",
        "“Share my last action” calls win.share and no input records it",
    ),
    (
        "s15-scripts",
        "header",
        "“Share my last action” calls win.share and no input records it",
    ),
    (
        "s15-scripts",
        "enter",
        "“Share my last action” calls win.share and no input records it",
    ),
    (
        "s15-scripts",
        "list",
        "“Share my last action” calls win.share and no input records it",
    ),
    (
        "s16-automations",
        "slot",
        "the automations view records each toggle (win.record) and no input records it",
    ),
    (
        "s17-dictation",
        "slot",
        "dictation records what it typed or set (win.record) and no input records it",
    ),
    (
        "keys-s18-steady",
        "enter",
        "the page keeps Jev's empty places, so ↓ can land on one before Jev fills it; the app \
         draws only places that hold a reading, so the highlight stays on the last row drawn",
    ),
    (
        "keys-s18-steady",
        "list",
        "the page keeps Jev's empty places, so ↓ can land on one before Jev fills it; the app \
         draws only places that hold a reading, so the highlight stays on the last row drawn",
    ),
    (
        "keys-s19-fast-hands",
        "enter",
        "the page keeps Jev's empty places, so ↓ can land on one before Jev fills it; the app \
         draws only places that hold a reading, so the highlight stays on the last row drawn",
    ),
    (
        "keys-s19-fast-hands",
        "list",
        "the page keeps Jev's empty places, so ↓ can land on one before Jev fills it; the app \
         draws only places that hold a reading, so the highlight stays on the last row drawn",
    ),
];

#[test]
fn golden_traces_replay() {
    let files = traces();
    if files.is_empty() {
        eprintln!("no traces in fixtures/traces yet");
        return;
    }
    let strict = ["slot", "header", "run", "ignored", "held", "enter", "list"];
    let mut failures = Vec::new();
    let mut known_seen = Vec::new();
    for path in &files {
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let r = replay(path);
        let mut differences: Vec<(&str, String)> = Vec::new();
        for k in &strict {
            let (want, got) = (settled(&r.expected, k), seq(r.got.iter(), k));
            if want != got {
                let first = want
                    .iter()
                    .zip(&got)
                    .position(|(a, b)| a != b)
                    .unwrap_or(want.len().min(got.len()));
                differences.push((
                    k,
                    format!(
                        "first difference at #{first}\n  want: {:?}\n  got:  {:?}",
                        want.get(first..(first + 4).min(want.len())).unwrap_or(&[]),
                        got.get(first..(first + 4).min(got.len())).unwrap_or(&[]),
                    ),
                ));
            }
        }
        for (k, msg) in differences {
            let msg = format!("{name} {k}: {msg}");
            if let Some(known) = KNOWN.iter().find(|(t, kk, _)| *t == name && *kk == k) {
                eprintln!("(known: {}) {msg}", known.2);
                known_seen.push(known);
            } else {
                failures.push(msg);
            }
        }
        if !r.unknown.is_empty() {
            failures.push(format!(
                "{name}: {} lines this replay doesn't read: {}",
                r.unknown.len(),
                r.unknown.join(", ")
            ));
        }
    }
    for known in KNOWN {
        if !known_seen.contains(&known) {
            failures.push(format!(
                "{} {} matches now; take it off KNOWN",
                known.0, known.1
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} differences:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
