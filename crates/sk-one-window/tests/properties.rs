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
        let live = match snap.slot.kind {
            SlotKind::Holding => true,
            SlotKind::Staged | SlotKind::Typing => snap.view.is_none(),
            SlotKind::Focused | SlotKind::Recent => false,
        };
        assert_eq!(
            snap.slot_opacity >= 1.0,
            live,
            "context and anything behind a view read as background: {:?} in {:?}",
            snap.slot.kind,
            snap.view
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
                    self.nav_at = Some(self.now);
                    self.guard_until = self.guard_until.max(self.now + ENTER_GUARD_MS);
                    self.nav_at = Some(self.now);
                }
            }
            Step::Content(px) => self.send(Event::Content { px: px as f32 }),
            Step::Open | Step::OpenUnprompted => {
                self.advance(self.now + 1);
                let target_before = self.win.enter_target(self.now).id;
                self.send(Event::View(ViewOp::Open(ViewSpec {
                    name: "form".into(),
                    label: "Remind me".into(),
                    enter: Some(EnterTarget::new("form", "Remind me")),
                    ..ViewSpec::default()
                })));
                let changed = self.win.enter_target(self.now).id != target_before;
                if matches!(s, Step::OpenUnprompted) && changed {
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

const SENTENCES: &[&str] = &[
    "put the pr on my second screen",
    "put chrome on the left",
    "mute slack for an hour",
];

#[derive(Clone, Debug)]
enum LearnStep {
    Type(usize),
    /// A sentence, Jev's reading of it, Enter, and the row finishing.
    Ran(usize),
    Clear,
}

fn learn_step() -> impl Strategy<Value = LearnStep> {
    prop_oneof![
        4 => (0..WORDS.len() + SENTENCES.len()).prop_map(LearnStep::Type),
        2 => (0..SENTENCES.len()).prop_map(LearnStep::Ran),
        1 => Just(LearnStep::Clear),
    ]
}

proptest! {
    /// Learning never changes the list under the cursor: within one query (from the first word
    /// until the input is empty again) the learned rows for the same words never change.
    #[test]
    fn learned_rows_change_only_between_queries(steps in prop::collection::vec(learn_step(), 1..40)) {
        let mut win = OneWindow::default();
        let mut seen: HashMap<String, Vec<String>> = HashMap::new();
        let mut now = 0;
        for step in steps {
            now += 500;
            match step {
                LearnStep::Type(i) => {
                    let text = WORDS.iter().chain(SENTENCES).nth(i).unwrap().to_string();
                    win.handle(now, Event::Text(text));
                }
                LearnStep::Ran(i) => {
                    let said = SENTENCES[i];
                    let row = Row::new(format!("jev:{i}"), format!("Reading {i}"));
                    win.handle(now, Event::Text(said.into()));
                    win.handle(now + 100, Event::Jev { text: said.into(), rows: vec![row.clone()] });
                    win.handle(now + 600, Event::Key(Key::Enter));
                    win.handle(now + 700, Event::Ran { row, app: None });
                    now += 700;
                }
                LearnStep::Clear => {
                    win.handle(now, Event::Text(String::new()));
                }
            }
            let text = win.text().to_string();
            if text.trim().is_empty() {
                seen.clear();
                continue;
            }
            let keys: Vec<String> = win.learned_rows(&text).into_iter().map(|r| r.key).collect();
            let first = seen.entry(text.clone()).or_insert_with(|| keys.clone());
            prop_assert_eq!(&*first, &keys, "learned rows for {:?} changed mid-query", text);
        }
    }
}

#[derive(Clone, Debug)]
enum TalkStep {
    Word,
    Read { words_back: usize, type_p: u8 },
    Pick(u8),
    Release,
    Escape,
    Wait(u16),
    Start,
}

fn talk_step() -> impl Strategy<Value = TalkStep> {
    prop_oneof![
        5 => Just(TalkStep::Word),
        4 => (0usize..3, 0u8..=100).prop_map(|(words_back, type_p)| TalkStep::Read { words_back, type_p }),
        1 => (1u8..=2).prop_map(TalkStep::Pick),
        2 => Just(TalkStep::Release),
        1 => Just(TalkStep::Escape),
        2 => (1u16..800).prop_map(TalkStep::Wait),
        1 => Just(TalkStep::Start),
    ]
}

const SPOKEN: &[&str] = &[
    "Hi",
    "Sarah,",
    "the",
    "review",
    "moved",
    "to",
    "Thursday.",
    "remind",
    "me",
    "at",
    "ten",
];

proptest! {
    /// While talking: the two rows never reorder; Jev's answers change the highlighted reading at
    /// most once per session, and never after it was picked; and the words paste only when the
    /// user picked typing or Jev's answer to exactly the final words agrees with typing and is at
    /// least 65 % sure.
    #[test]
    fn voice_readings_are_steady_and_letting_go_acts_only_when_sure(
        steps in prop::collection::vec(talk_step(), 1..60)
    ) {
        use sk_one_window::talk::{Reading, TalkEvent};
        use sk_one_window::contract::TALK_SURE_P;
        let mut win = OneWindow::default();
        let mut now = 0;
        let mut spoken: Vec<&str> = Vec::new();
        let mut changes = 0;
        for step in steps {
            now += 30;
            let before = win.talk().cloned();
            let event = match &step {
                TalkStep::Start => {
                    spoken.clear();
                    changes = 0;
                    Event::Talk(TalkEvent::Start { app: "Mail".into(), field: "Reply".into(), to: None })
                }
                TalkStep::Word => {
                    spoken.push(SPOKEN[spoken.len() % SPOKEN.len()]);
                    Event::Talk(TalkEvent::Heard(spoken.join(" ")))
                }
                TalkStep::Read { words_back, type_p } => {
                    let n = spoken.len().saturating_sub(*words_back);
                    let p = f32::from(*type_p) / 100.0;
                    Event::Talk(TalkEvent::Read { words: spoken[..n].join(" "), type_p: p, ask_p: 1.0 - p, ask: None })
                }
                TalkStep::Pick(d) => Event::Key(Key::CmdDigit(*d)),
                TalkStep::Release => Event::Talk(TalkEvent::Release),
                TalkStep::Escape => Event::Key(Key::Escape),
                TalkStep::Wait(ms) => {
                    now += u64::from(*ms);
                    Event::Tick
                }
            };
            let out = win.handle(now, event.clone());
            let pasted = out.effects.iter().any(|fx| matches!(fx, Effect::Paste { .. }));
            if let Some(before) = &before {
                if pasted {
                    let picked = matches!(step, TalkStep::Pick(1)) || before.locked == Some(Reading::Type);
                    let sure = |words: &str, type_p: f32, ask_p: f32| {
                        words.trim() == before.heard.trim() && type_p >= ask_p && type_p >= TALK_SURE_P
                    };
                    let jev_sure = match &event {
                        Event::Talk(TalkEvent::Read { words, type_p, ask_p, .. }) => sure(words, *type_p, *ask_p),
                        _ => before.answer.as_ref().is_some_and(|a| sure(&a.words, a.type_p, a.ask_p)),
                    };
                    prop_assert!(picked || jev_sure, "pasted without a pick or a sure answer: {:?}", before);
                    prop_assert!(!matches!(step, TalkStep::Escape), "Esc pastes nothing");
                }
            }
            if let Some(talk) = win.talk() {
                let rows = talk.rows();
                prop_assert_eq!(rows[0].key.as_str(), "talk:type");
                prop_assert_eq!(rows[1].key.as_str(), "talk:ask");
                let answered = matches!(step, TalkStep::Read { .. });
                if let Some(before) = &before {
                    if answered && before.reading.is_some() && before.reading != talk.reading {
                        prop_assert!(before.locked.is_none(), "a picked reading changed");
                        changes += 1;
                    }
                }
                prop_assert!(changes <= 1, "the reading changed {} times", changes);
            }
        }
    }
}
