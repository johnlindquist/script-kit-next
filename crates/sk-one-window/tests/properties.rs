//! Rules that hold for any sequence of keys, words, answers and time.

use std::collections::{HashMap, HashSet};

use proptest::prelude::*;
use sk_one_window::contract::words;
use sk_one_window::contract::{ENTER_GUARD_MS, READING_SETTLE_MS, SHRINK_HOLD_MS};
use sk_one_window::list::Item;
use sk_one_window::window::{
    Done, Effect, Event, HeldOp, Key, OneWindow, RunHow, ViewOp, ViewSpec,
};
use sk_one_window::{EnterTarget, HeightMode, Row, SlotKind};

/// What ↵ says while Enter runs Jev's reading, whichever it turns out to be.
const JEV_RUNS: [&str; 3] = [
    words::ENTER_JEV_FIRST,
    words::ENTER_WAIT,
    words::ENTER_WAITING,
];

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
    "when does lebron james play next",
];
const NAMES: &[&str] = &[
    "Slack", "Safari", "Launcher", "Linear", "Sarah", "WhatsApp", "Weather", "Webex",
];
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
                Effect::Run { row, how } => {
                    if let Some(target) = &enter_before {
                        assert!(
                            now >= self.guard_until,
                            "Enter runs nothing while guarded ({now} < {})",
                            self.guard_until
                        );
                        let says = target.label.trim_end_matches(" — ↵ again");
                        let runs_jev = *how == RunHow::Slot && JEV_RUNS.contains(&says);
                        assert!(
                            says == row.enter_words() || says == row.title || runs_jev,
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

/// One key of a sentence typed without a pause: how long after the last key it came (under
/// [`READING_SETTLE_MS`]), whether a name that owned Enter earlier in the sentence is listed again
/// (a keyword or alias hit), and whether Jev answered before the next key (for these words, or for
/// older ones). Jev answers the whole sentence once, after its last key.
#[derive(Clone, Debug)]
struct Keystroke {
    gap: u64,
    echo: bool,
    jev: Option<(bool, u64, Vec<usize>)>,
}

/// The file's words that no other word extends, typed one key at a time.
fn sentences() -> Vec<&'static str> {
    WORDS
        .iter()
        .copied()
        .filter(|w| !w.is_empty() && !WORDS.iter().any(|o| o.len() > w.len() && o.starts_with(w)))
        .collect()
}

/// The names a search lists for `text`, strongest first: a whole name, its start, the start of
/// one of its words, anywhere in it, or its letters in order (a fuzzy near-miss, too weak to own
/// Enter).
fn search(text: &str) -> Vec<Row> {
    let t = text.to_lowercase();
    let mut found: Vec<(u8, &str)> = NAMES
        .iter()
        .filter_map(|name| {
            let n = name.to_lowercase();
            let mut letters = n.chars();
            let score = if n == t {
                100
            } else if n.starts_with(&t) {
                95
            } else if n.split_whitespace().any(|w| w.starts_with(&t)) {
                90
            } else if n.contains(&t) {
                80
            } else if t
                .chars()
                .filter(|c| !c.is_whitespace())
                .all(|c| letters.any(|l| l == c))
            {
                40
            } else {
                return None;
            };
            Some((score, *name))
        })
        .collect();
    found.sort_by(|a, b| b.0.cmp(&a.0));
    found
        .into_iter()
        .take(3)
        .map(|(score, name)| Row::new(format!("name:{name}"), name).with_score(score))
        .collect()
}

fn keystroke() -> impl Strategy<Value = Keystroke> {
    (
        1..READING_SETTLE_MS,
        prop::bool::weighted(0.2),
        prop::option::of((
            any::<bool>(),
            0u64..100,
            prop::collection::vec(0..JEV.len(), 0..3),
        )),
    )
        .prop_map(|(gap, echo, jev)| {
            let jev = jev.map(|(current, at, rows)| (current, at.min(gap - 1), rows));
            Keystroke { gap, echo, jev }
        })
}

/// What ↵ said after every page turn of a typed sentence, and why each change was allowed.
struct Typist {
    win: OneWindow,
    now: u64,
    wake: Option<u64>,
    last_key: u64,
    labels: Vec<String>,
    owner: Option<String>,
    /// Every name that owned Enter in the sentence.
    owners: Vec<String>,
}

impl Typist {
    fn feed(&mut self, ev: Event) {
        self.win.handle(self.now, ev);
        self.wake = self.win.next_wake();
    }

    fn until(&mut self, t: u64) {
        while let Some(at) = self.wake.filter(|at| *at <= t) {
            self.now = at;
            self.wake = None;
            self.feed(Event::Tick);
            self.look();
        }
        self.now = t;
    }

    /// The thing that owns Enter: a name by its key, or Jev's place.
    fn owner_now(&self) -> Option<String> {
        let list = self.win.list();
        match list.current()? {
            item @ Item::Name(_) => list.row(&item).map(|r| r.key.clone()),
            Item::Slot(_) => Some("jev".into()),
            _ => None,
        }
    }

    fn look(&mut self) {
        let label = self.win.enter_target(self.now).label;
        let owner = self.owner_now();
        if self.labels.last() != Some(&label) {
            let list = self.win.list();
            let settled_reading = matches!(list.current(), Some(Item::Slot(_)))
                && list.jev.settled_for(self.win.text())
                && self.now >= self.last_key + READING_SETTLE_MS
                && list.current_row(self.now).is_some_and(|r| r.title == label);
            assert!(
                self.labels.is_empty() || owner != self.owner || settled_reading,
                "↵ went from {:?} to {label:?} at {} without a stronger name or a settled reading",
                self.labels.last(),
                self.now
            );
            assert!(
                !self.labels.contains(&label),
                "↵ went back to {label:?} after {:?}",
                self.labels
            );
            self.labels.push(label);
        }
        if let Some(key) = owner.as_ref().filter(|k| k.starts_with("name:")) {
            if !self.owners.contains(key) {
                self.owners.push(key.clone());
            }
        }
        self.owner = owner;
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 512, ..ProptestConfig::default() })]

    #[test]
    fn the_enter_button_never_flickers_while_a_sentence_is_typed(
        pick in 0..sentences().len(),
        keys in prop::collection::vec(keystroke(), 40),
        final_wait in 0u64..600,
        final_rows in prop::collection::vec(0..JEV.len(), 1..3),
    ) {
        let sentence = sentences()[pick];
        let mut t = Typist {
            win: Run::new().win,
            now: 0,
            wake: None,
            last_key: 0,
            labels: Vec::new(),
            owner: None,
            owners: Vec::new(),
        };
        let prefixes: Vec<&str> = sentence
            .char_indices()
            .map(|(i, c)| &sentence[..i + c.len_utf8()])
            .collect();
        for (i, (text, key)) in prefixes.iter().zip(keys.iter().cycle()).enumerate() {
            t.until(t.last_key + key.gap);
            t.last_key = t.now;
            t.feed(Event::Text(text.to_string()));
            let mut names = search(text);
            if let Some(key) = t.owners.first().filter(|k| key.echo && !names.iter().any(|n| &n.key == *k)) {
                let name = key.trim_start_matches("name:");
                names.push(Row::new(key.clone(), name).with_score(70));
                names.sort_by(|a, b| b.score.cmp(&a.score));
            }
            t.feed(Event::Names(names));
            t.look();
            if let Some((current, at, rows)) = key.jev.as_ref().filter(|_| i + 1 < prefixes.len()) {
                t.until(t.last_key + at);
                let text = if *current || i == 0 { text.to_string() } else { prefixes[i - 1].to_string() };
                let rows = rows.iter().map(|j| Row::new(format!("jev:{}", JEV[*j]), JEV[*j])).collect();
                t.feed(Event::Jev { text, rows });
                t.look();
            }
        }
        t.until(t.now + final_wait);
        let rows: Vec<Row> = final_rows.iter().map(|j| Row::new(format!("jev:{}", JEV[*j]), JEV[*j])).collect();
        t.feed(Event::Jev { text: sentence.to_string(), rows: rows.clone() });
        t.look();
        t.until(t.now + 2_000);
        if let Some(row) = t.win.list().current_row(t.now).filter(|_| t.owner.as_deref() == Some("jev")) {
            prop_assert_eq!(
                t.labels.last(),
                Some(&row.title),
                "once the keys pause, ↵ says the reading Enter runs: {:?}",
                t.labels
            );
        }
    }
}
