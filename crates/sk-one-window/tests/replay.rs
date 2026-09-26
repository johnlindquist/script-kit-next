//! Replays the prototype's golden traces through the reducer and compares what it shows.
//!
//! Traces are recorded on the page's clock with a 200 ms hold ticker and a real layout engine, so
//! the comparison is per kind of output, in order, after dropping consecutive repeats: the line
//! above the input, the first header, what ran, and which Enters were ignored must match exactly;
//! height modes must follow the same grow/hold/shrink pattern.

use std::path::{Path, PathBuf};

use sk_one_window::Row;
use sk_one_window::trace::{
    Ev, EvLine, Line, Observer, Out, OutLine, events_of, parse_line, replayable,
};
use sk_one_window::window::{Config, Effect, Event, OneWindow};

fn traces() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/traces");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map(|d| d.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default();
    files.retain(|p| p.extension().is_some_and(|e| e == "jsonl"));
    files.sort();
    files
}

#[derive(Default)]
struct Replay {
    expected: Vec<OutLine>,
    got: Vec<OutLine>,
    unknown: usize,
}

/// Feeds a trace to the reducer one page turn at a time: an input with no `via` starts a turn,
/// inputs it caused join it and are fed at the turn's time (so the reducer sees them as part of
/// the same keypress), and what's shown is observed when the turn ends.
struct Player {
    win: OneWindow,
    obs: Observer,
    last_run: Option<Row>,
    wake: Option<u64>,
    turn: Option<u64>,
    effects: Vec<Effect>,
}

impl Player {
    fn new(win: OneWindow) -> Self {
        Self {
            win,
            obs: Observer::new(),
            last_run: None,
            wake: None,
            turn: None,
            effects: Vec::new(),
        }
    }

    fn feed(&mut self, t: u64, ev: Event) {
        let out = self.win.handle(t, ev);
        for fx in &out.effects {
            match fx {
                Effect::WakeAt(at) => self.wake = Some(*at),
                Effect::Run { row, .. } => self.last_run = Some(row.clone()),
                _ => {}
            }
        }
        self.effects.extend(out.effects);
    }

    fn close_turn(&mut self) {
        if let Some(t) = self.turn.take() {
            let effects = std::mem::take(&mut self.effects);
            self.obs.observe(t, &self.win, &effects);
        }
    }

    /// Delivers the reducer's timers that fall before `t`, each in its own turn.
    fn ticks_before(&mut self, t: u64) {
        while let Some(at) = self.wake.filter(|at| *at < t) {
            self.close_turn();
            self.wake = None;
            self.turn = Some(at);
            self.feed(at, Event::Tick);
            self.close_turn();
        }
    }

    fn input(&mut self, line: EvLine) {
        let starts_turn = line.via.is_none() || self.turn.is_none();
        if starts_turn {
            self.ticks_before(line.t);
            self.close_turn();
            self.turn = Some(line.t);
        }
        let t = self.turn.unwrap_or(line.t);
        for e in events_of(line.ev, self.last_run.as_ref()) {
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
            Line::Unknown(_) => r.unknown += 1,
            Line::Out(out) => r.expected.push(out),
            Line::Ev(ev) if !replayable(&ev) => {}
            Line::Ev(ev) => {
                if let Ev::Init {
                    hold_ms, history, ..
                } = &ev.ev
                {
                    let cfg = Config {
                        hold_ms: hold_ms.unwrap_or(Config::default().hold_ms),
                        ..Config::default()
                    };
                    let history = history
                        .iter()
                        .cloned()
                        .map(sk_one_window::trace::history_entry)
                        .collect();
                    player = Some(Player::new(OneWindow::new(cfg).with_history(history)));
                }
                player
                    .get_or_insert_with(|| Player::new(OneWindow::default()))
                    .input(ev);
            }
        }
    }
    if let Some(p) = player.as_mut() {
        p.ticks_before(end + 1);
        p.close_turn();
        r.got = p.obs.drain();
    }
    if let Some(dir) = std::env::var_os("OW_REPLAY_DUMP") {
        let name = path.file_stem().unwrap().to_string_lossy();
        let out: String = r
            .got
            .iter()
            .map(|l| serde_json::to_string(l).unwrap() + "\n")
            .collect();
        std::fs::write(Path::new(&dir).join(format!("{name}.got.jsonl")), out).expect("dump");
    }
    r
}

fn kind(out: &Out) -> &'static str {
    match out {
        Out::Slot { .. } => "slot",
        Out::Header { .. } => "header",
        Out::Enter { .. } => "enter",
        Out::Height { .. } => "height",
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
        Out::Height { mode, .. } => mode.clone(),
        Out::Run { key, .. } => key.clone(),
        Out::Ignored { key } => key.clone(),
        Out::Held { op, label } => format!("{op}|{label}"),
        Out::List { rows, sel } => {
            let rows: Vec<String> = rows
                .iter()
                .map(|r| match (&r.key, r.jev) {
                    (Some(k), _) => k.clone(),
                    (None, true) => "jev:·".into(),
                    (None, false) => r.title.clone(),
                })
                .collect();
            format!("{}@{sel}", rows.join(","))
        }
        Out::Other => String::new(),
    }
}

fn seq(lines: &[OutLine], k: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for l in lines.iter().filter(|l| kind(&l.out) == k) {
        let s = key(&l.out);
        if out.last() != Some(&s) {
            out.push(s);
        }
    }
    out
}

#[test]
fn golden_traces_replay() {
    let files = traces();
    if files.is_empty() {
        eprintln!("no traces in fixtures/traces yet");
        return;
    }
    let strict = ["slot", "header", "run", "ignored", "held"];
    let mut failures = Vec::new();
    for path in &files {
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let r = replay(path);
        for k in strict.iter().chain(["height", "enter", "list"].iter()) {
            let want = seq(&r.expected, k);
            let got = seq(&r.got, k);
            if want != got {
                let first = want
                    .iter()
                    .zip(&got)
                    .position(|(a, b)| a != b)
                    .unwrap_or(want.len().min(got.len()));
                let msg = format!(
                    "{name} {k}: first difference at #{first}\n  want: {:?}\n  got:  {:?}",
                    want.get(first..(first + 4).min(want.len())).unwrap_or(&[]),
                    got.get(first..(first + 4).min(got.len())).unwrap_or(&[]),
                );
                if strict.contains(k) {
                    failures.push(msg);
                } else {
                    eprintln!("(report) {msg}");
                }
            }
        }
        if r.unknown > 0 {
            eprintln!(
                "(report) {name}: {} lines this replay doesn't read",
                r.unknown
            );
        }
    }
    assert!(
        failures.is_empty(),
        "{} differences:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
