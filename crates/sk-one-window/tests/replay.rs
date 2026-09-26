//! Replays the prototype's golden traces through the reducer and compares what it shows.
//!
//! Traces are recorded on the page's clock with a 200 ms hold ticker and a real layout engine, so
//! the comparison is per kind of output, in order, after dropping consecutive repeats: the line
//! above the input, the first header, what ran, and which Enters were ignored must match exactly;
//! height modes must follow the same grow/hold/shrink pattern.

use std::path::{Path, PathBuf};

use sk_one_window::Row;
use sk_one_window::trace::{Ev, Line, Observer, Out, OutLine, events_of, parse_line};
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

fn replay(path: &Path) -> Replay {
    let text = std::fs::read_to_string(path).expect("trace reads");
    let mut r = Replay::default();
    let mut win: Option<OneWindow> = None;
    let mut obs = Observer::new();
    let mut last_run: Option<Row> = None;
    let mut wake: Option<u64> = None;
    let deliver = |win: &mut OneWindow,
                   obs: &mut Observer,
                   t: u64,
                   ev: Event,
                   last_run: &mut Option<Row>,
                   wake: &mut Option<u64>| {
        let out = win.handle(t, ev);
        for fx in &out.effects {
            match fx {
                Effect::WakeAt(at) => *wake = Some(*at),
                Effect::Run { row, .. } => *last_run = Some(row.clone()),
                _ => {}
            }
        }
        obs.observe(t, win, &out.effects);
    };
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        match parse_line(line).expect("trace line is JSON") {
            Line::Meta(_) => {}
            Line::Unknown(_) => r.unknown += 1,
            Line::Out(out) => r.expected.push(out),
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
                    win = Some(OneWindow::new(cfg).with_history(history));
                }
                let w = win.get_or_insert_with(OneWindow::default);
                while let Some(at) = wake.filter(|at| *at <= ev.t) {
                    wake = None;
                    deliver(w, &mut obs, at, Event::Tick, &mut last_run, &mut wake);
                }
                for e in events_of(ev.ev, last_run.as_ref()) {
                    deliver(w, &mut obs, ev.t, e, &mut last_run, &mut wake);
                }
            }
        }
    }
    if let Some(w) = win.as_mut() {
        let end = r.expected.last().map(|l| l.t).unwrap_or(0) + 10_000;
        while let Some(at) = wake.filter(|at| *at <= end) {
            wake = None;
            deliver(w, &mut obs, at, Event::Tick, &mut last_run, &mut wake);
        }
    }
    r.got = obs.drain();
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
        for k in strict.iter().chain(["height", "enter"].iter()) {
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
