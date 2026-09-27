//! Learning between queries, through the reducer; and s14's trace replayed through it.

use std::path::Path;

use sk_one_window::trace::{Ev, Line, events_of, parse_line};
use sk_one_window::{Effect, Event, Key, OneWindow, Row};

fn reading(title: &str) -> Row {
    Row::new(format!("jev:{title}"), title).with_icon("window")
}

/// Types `text`, lets Jev read it as `row`, and presses Enter.
fn run_sentence(win: &mut OneWindow, t: u64, text: &str, row: &Row) -> Vec<Effect> {
    win.handle(t, Event::Text(text.into()));
    win.handle(
        t + 100,
        Event::Jev {
            text: text.into(),
            rows: vec![row.clone()],
        },
    );
    win.handle(t + 600, Event::Key(Key::Enter)).effects
}

#[test]
fn a_sentence_that_ran_is_a_learned_row_next_time() {
    let mut win = OneWindow::default();
    let chrome = reading("Move Google Chrome to Next Display");
    let fx = run_sentence(&mut win, 0, "put the pr on my second screen", &chrome);
    assert!(fx.iter().any(|f| matches!(f, Effect::Run { .. })));
    assert!(
        win.learned_rows("put").is_empty(),
        "nothing is learned until it finished"
    );
    let out = win.handle(
        1_000,
        Event::Ran {
            row: chrome,
            app: None,
        },
    );
    assert!(out.effects.contains(&Effect::Learned), "the host keeps it");
    let rows = win.learned_rows("put t");
    assert_eq!(rows[0].key, "learned:Move Google Chrome to Next Display");
    win.handle(2_000, Event::Text("put t".into()));
    win.handle(2_000, Event::Names(rows));
    assert_eq!(
        win.enter_target(2_000).label,
        "Move Google Chrome to Next Display",
        "the learned row owns Enter before Jev answers"
    );
}

#[test]
fn a_row_that_opened_something_teaches_nothing() {
    let mut win = OneWindow::default();
    let triage = reading("Go through everything waiting on me");
    run_sentence(&mut win, 0, "clear my plate for today", &triage);
    win.handle(
        700,
        Event::Done(sk_one_window::Done {
            icon: "triage".into(),
            title: "Went through 4 things".into(),
            ..sk_one_window::Done::default()
        }),
    );
    assert!(win.learned_rows("clear my").is_empty());
}

/// The page learned each sentence that ran and listed it as a learned name the next time its
/// start was typed. Replaying the trace through the reducer, every learned row the page listed
/// is the reducer's learned row for the same words.
#[test]
fn s14_replays_through_the_lessons_rule() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/traces/s14-learns.jsonl");
    let text = std::fs::read_to_string(path).expect("trace");
    let mut win = OneWindow::default();
    let mut turn_run: Option<Row> = None;
    let mut compared = 0;
    for raw in text.lines().filter(|l| !l.trim().is_empty()) {
        let Ok(Line::Ev(line)) = parse_line(raw) else {
            continue;
        };
        let t = line.t;
        if line.via.is_none() && !matches!(line.ev, Ev::Names { .. } | Ev::Content { .. }) {
            turn_run = None;
        }
        if let Ev::Names { text, rows } = &line.ev {
            let want: Vec<String> = rows
                .iter()
                .filter(|r| r.learned)
                .map(|r| r.key.clone())
                .collect();
            let got: Vec<String> = win.learned_rows(text).into_iter().map(|r| r.key).collect();
            assert_eq!(got, want, "learned rows for {text:?} at {t}");
            compared += usize::from(!want.is_empty());
        }
        for event in events_of(line, turn_run.as_ref(), &win) {
            for fx in win.handle(t, event).effects {
                if let Effect::Run { row, .. } = fx {
                    turn_run = Some(row);
                }
            }
        }
    }
    assert!(compared >= 5, "the trace lists learned rows ({compared})");
}
