//! Rules that hold for any sequence of keys, words, answers and time.

use std::collections::{HashMap, HashSet};

use proptest::prelude::*;
use sk_one_window::contract::{ENTER_GUARD_MS, SHRINK_HOLD_MS};
use sk_one_window::list::Item;
use sk_one_window::window::{Done, Effect, Event, HeldOp, Key, OneWindow, ViewOp, ViewSpec};
use sk_one_window::{EnterTarget, HeightMode, Row, SlotKind};

#[derive(Clone, Debug)]
enum Step {
    Type(usize),
    Key(Key),
    Names(Vec<(usize, u8)>),
    Jev { current: bool, rows: Vec<usize> },
    JevDown,
    Done { message: bool, stay: bool },
    Content(u16),
    Open,
    OpenUnprompted,
    Wait(u16),
}

const WORDS: &[&str] = &[
    "",
    "s",
    "sl",
    "sla",
    "slack",
    "when does",
    "when does lebron",
    "tell sarah hi",
    "lunch",
];
const NAMES: &[&str] = &["Slack", "Safari", "Launcher", "Linear", "Sarah"];
const JEV: &[&str] = &[
    "open slack",
    "message sarah",
    "when does lebron play",
    "remind me",
    "add to note",
];

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        4 => (0..WORDS.len()).prop_map(Step::Type),
        4 => prop_oneof![
            Just(Key::Enter),
            Just(Key::Down),
            Just(Key::Up),
            Just(Key::Escape),
            Just(Key::CmdK),
            Just(Key::CmdZ),
            Just(Key::CmdEnter),
            (1u8..=3).prop_map(Key::CmdDigit),
        ]
        .prop_map(Step::Key),
        2 => prop::collection::vec((0..NAMES.len(), 0u8..=100), 0..3).prop_map(Step::Names),
        3 => (any::<bool>(), prop::collection::vec(0..JEV.len(), 0..4)).prop_map(|(current, rows)| Step::Jev { current, rows }),
        1 => Just(Step::JevDown),
        1 => (any::<bool>(), any::<bool>()).prop_map(|(message, stay)| Step::Done { message, stay }),
        2 => (120u16..600).prop_map(Step::Content),
        1 => Just(Step::Open),
        1 => Just(Step::OpenUnprompted),
        4 => (1u16..2_000).prop_map(Step::Wait),
    ]
}

struct Run {
    win: OneWindow,
    now: u64,
    last_text_at: Option<u64>,
    /// When the window last went somewhere (a view opened or closed, something staged, done):
    /// back on the list the floor clears, so only typing since then holds the height.
    nav_at: Option<u64>,
    guard_until: u64,
    sent: HashSet<u64>,
    unsent: HashSet<u64>,
    wake: Option<u64>,
}

impl Run {
    fn new() -> Self {
        let mut win = OneWindow::default();
        win.handle(
            0,
            Event::Suggested(vec![
                Row::new("s:slack", "Slack").with_enter_label("Open Slack"),
            ]),
        );
        Self {
            win,
            now: 0,
            last_text_at: None,
            nav_at: None,
            guard_until: 0,
            sent: HashSet::new(),
            unsent: HashSet::new(),
            wake: None,
        }
    }

    /// Delivers every wake the reducer asked for up to `until`, like the host's timer would.
    fn advance(&mut self, until: u64) {
        while let Some(at) = self.wake.filter(|at| *at <= until) {
            self.now = at;
            let out = self.win.handle(at, Event::Tick);
            self.check(&out.effects, None);
            if self.wake == Some(at) {
                self.wake = None;
            }
        }
        self.now = until;
    }

    fn send(&mut self, ev: Event) {
        let before = self.win.snapshot(self.now);
        let is_enter = matches!(ev, Event::Key(Key::Enter));
        let text_before = self.win.text().to_string();
        let staged_before = self.win.staged().is_some();
        let typed = matches!(ev, Event::Text(_));
        let command = matches!(&ev, Event::Key(k) if !matches!(k, Key::Up | Key::Down));
        let out = self.win.handle(self.now, ev);
        let went = command
            || before.view.is_some() != self.win.in_view()
            || staged_before != self.win.staged().is_some()
            || (!typed && text_before != self.win.text());
        if went {
            self.nav_at = Some(self.now);
        }
        self.check(&out.effects, is_enter.then_some(before.enter));
    }

    fn check(&mut self, effects: &[Effect], enter_before: Option<EnterTarget>) {
        let now = self.now;
        for fx in effects {
            match fx {
                Effect::WakeAt(t) => {
                    assert!(*t > now, "wakes are in the future");
                    self.wake = Some(*t);
                }
                Effect::Run { row, .. } => {
                    if let Some(target) = &enter_before {
                        assert!(
                            now >= self.guard_until,
                            "Enter runs nothing while guarded ({now} < {})",
                            self.guard_until
                        );
                        let says = target.label.trim_end_matches(" — ↵ again");
                        assert!(
                            says == row.enter_words() || says == row.title,
                            "Enter said “{}” but ran “{}”",
                            target.label,
                            row.title
                        );
                    }
                }
                Effect::Height(d) if d.mode == HeightMode::Shrink => {
                    if let Some(at) = self.last_text_at {
                        let went = self.nav_at.is_some_and(|n| n >= at);
                        assert!(
                            went || now >= at + SHRINK_HOLD_MS,
                            "shrank {}ms after a key",
                            now - at
                        );
                    }
                }
                Effect::Held {
                    op: HeldOp::Sent,
                    id,
                    ..
                } if *id != 0 => {
                    assert!(self.sent.insert(*id), "message {id} sent twice");
                    assert!(
                        !self.unsent.contains(id),
                        "message {id} sent after it was stopped"
                    );
                }
                Effect::Held {
                    op: HeldOp::Unsent,
                    id,
                    ..
                } => {
                    assert!(
                        !self.sent.contains(id),
                        "message {id} stopped after it went"
                    );
                    self.unsent.insert(*id);
                }
                _ => {}
            }
        }
        let snap = self.win.snapshot(now);
        if snap.view.is_none() {
            assert!(
                matches!(snap.items.first(), Some(Item::Header(w)) if !w.is_empty()),
                "every list starts with a header: {:?}",
                snap.items
            );
        }
        assert_eq!(
            snap.slot.dimmed(),
            matches!(snap.slot.kind, SlotKind::Focused | SlotKind::Recent)
        );
        assert!(
            !snap.slot.label.is_empty(),
            "the line above the input always says something"
        );
    }

    fn step(&mut self, s: Step) {
        match s {
            Step::Wait(ms) => self.advance(self.now + ms as u64),
            Step::Type(i) => {
                self.advance(self.now + 1);
                self.last_text_at = Some(self.now);
                self.send(Event::Text(WORDS[i].to_string()));
            }
            Step::Key(k) => {
                self.advance(self.now + 1);
                self.send(Event::Key(k));
            }
            Step::Names(rows) => {
                let text = self.win.text().to_lowercase();
                let rows = rows
                    .into_iter()
                    .filter(|_| !text.trim().is_empty())
                    .map(|(i, s)| {
                        Row::new(format!("name:{}", NAMES[i]), NAMES[i])
                            .with_score(s)
                            .with_icon("app")
                    })
                    .collect();
                self.send(Event::Names(rows));
            }
            Step::Jev { current, rows } => {
                self.advance(self.now + 1);
                let text = if current {
                    self.win.text().to_string()
                } else {
                    "older words".into()
                };
                let rows = rows
                    .into_iter()
                    .map(|i| Row::new(format!("jev:{}", JEV[i]), JEV[i]))
                    .collect();
                self.send(Event::Jev { text, rows });
            }
            Step::JevDown => {
                self.advance(self.now + 1);
                let text = self.win.text().to_string();
                self.send(Event::JevDown { text });
            }
            Step::Done { message, stay } => {
                self.advance(self.now + 1);
                self.send(Event::Done(Done {
                    icon: if message {
                        "message".into()
                    } else {
                        "move".into()
                    },
                    title: "Did it".into(),
                    form_kind: message.then(|| "message".into()),
                    stay,
                    ..Done::default()
                }));
                if !stay {
                    self.guard_until = self.guard_until.max(self.now + ENTER_GUARD_MS);
                    self.nav_at = Some(self.now);
                }
            }
            Step::Content(px) => self.send(Event::Content { px: px as f32 }),
            Step::Open | Step::OpenUnprompted => {
                self.advance(self.now + 1);
                self.send(Event::View(ViewOp::Open(ViewSpec {
                    name: "form".into(),
                    label: "Remind me".into(),
                    enter: Some(EnterTarget::new("form", "Remind me")),
                    ..ViewSpec::default()
                })));
                if matches!(s, Step::OpenUnprompted) {
                    self.guard_until = self.guard_until.max(self.now + ENTER_GUARD_MS);
                }
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 512, ..ProptestConfig::default() })]

    #[test]
    fn the_window_keeps_its_promises(steps in prop::collection::vec(step(), 1..80)) {
        let mut run = Run::new();
        for s in steps {
            run.step(s);
        }
        run.advance(run.now + 20_000);
        prop_assert!(run.win.holds().live_count() == 0 || run.win.holds().paused(), "every message goes out unless typing pauses it");
    }

    #[test]
    fn an_enter_inside_the_guard_is_ignored(gap in 0u64..ENTER_GUARD_MS) {
        let mut win = OneWindow::default();
        win.handle(0, Event::Suggested(vec![Row::new("s:slack", "Slack")]));
        win.handle(10, Event::Done(Done { icon: "move".into(), title: "Moved".into(), ..Done::default() }));
        let out = win.handle(10 + gap, Event::Key(Key::Enter));
        prop_assert!(out.effects.contains(&Effect::EnterIgnored));
        let ran = out.effects.iter().any(|e| matches!(e, Effect::Run { .. }));
        prop_assert!(!ran, "nothing ran");
    }
}

#[test]
fn a_stale_row_waits_and_runs_the_fresh_answer() {
    let mut win = OneWindow::default();
    win.handle(0, Event::Text("when does".into()));
    win.handle(
        80,
        Event::Jev {
            text: "when does".into(),
            rows: vec![Row::new("a", "When does the store close")],
        },
    );
    win.handle(500, Event::Text("when does lebron".into()));
    let out = win.handle(510, Event::Key(Key::Enter));
    assert!(
        !out.effects.iter().any(|e| matches!(e, Effect::Run { .. })),
        "a row read from older words never runs"
    );
    assert_eq!(
        win.snapshot(510).enter.label,
        sk_one_window::contract::words::ENTER_WAITING
    );
    let out = win.handle(
        700,
        Event::Jev {
            text: "when does lebron".into(),
            rows: vec![Row::new("b", "When LeBron plays next")],
        },
    );
    let ran: Vec<_> = out
        .effects
        .iter()
        .filter_map(|e| match e {
            Effect::Run { row, .. } => Some(row.key.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(ran, vec!["b".to_string()]);
}

#[test]
fn headers_never_come_and_go_while_typing() {
    let mut win = OneWindow::default();
    win.handle(0, Event::Suggested(vec![Row::new("s:slack", "Slack")]));
    let mut seen = HashMap::new();
    for (i, text) in ["s", "sl", "sla", "slac", "slack", "slack m", "slack me"]
        .iter()
        .enumerate()
    {
        let t = 100 * (i as u64 + 1);
        win.handle(t, Event::Text(text.to_string()));
        if text.len() < 6 {
            win.handle(
                t,
                Event::Names(vec![Row::new("name:Slack", "Slack").with_score(90)]),
            );
        }
        let snap = win.snapshot(t);
        assert!(matches!(snap.items.first(), Some(Item::Header(_))));
        seen.insert(text.to_string(), snap.header.clone());
    }
    assert_eq!(seen["s"], "Matches");
    assert_eq!(seen["slack me"], "Jev");
}
