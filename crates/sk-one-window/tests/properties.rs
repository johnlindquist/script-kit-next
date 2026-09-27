//! Rules that hold for any sequence of keys, words, answers and time.

use std::collections::{HashMap, HashSet};

use proptest::prelude::*;
use sk_one_window::contract::words;
use sk_one_window::contract::{ENTER_GUARD_MS, READING_SETTLE_MS, SHRINK_HOLD_MS};
use sk_one_window::list::Item;
use sk_one_window::notice::{Irreversible, Notice};
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
    Notice,
    Irreversible,
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
    "summarize @selection for >:dev",
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
        1 => Just(Step::Notice),
        1 => Just(Step::Irreversible),
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
    undone: HashSet<u64>,
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
            undone: HashSet::new(),
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
                Effect::Undo { entry } => {
                    assert!(self.undone.insert(*entry), "entry {entry} undone twice");
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
        if snap.view.is_none() && self.win.staged().is_none() && self.win.holds().live_count() > 0 {
            assert_eq!(
                snap.slot.kind,
                SlotKind::Holding,
                "nothing covers something waiting to go"
            );
        }
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
            Step::Notice => {
                self.advance(self.now + 1);
                self.send(Event::Notice(Notice::new("check", "Copied")));
            }
            Step::Irreversible => {
                self.advance(self.now + 1);
                self.send(Event::Irreversible(Irreversible::new(
                    "trash",
                    "Empty Trash",
                )));
                self.nav_at = Some(self.now);
                self.guard_until = self.guard_until.max(self.now + ENTER_GUARD_MS);
            }
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
    found.sort_by_key(|f| std::cmp::Reverse(f.0));
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
                names.sort_by_key(|n| std::cmp::Reverse(n.score));
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

proptest! {
    #![proptest_config(ProptestConfig { cases: 512, ..ProptestConfig::default() })]

    #[test]
    fn marks_never_guess_and_never_overlap(
        pick in 0..sentences().len(),
        typed in 0usize..48,
        reads in prop::collection::vec((0usize..5, 0usize..8, 1usize..4), 0..4),
    ) {
        use sk_one_window::marks::{Word, marks_for};
        let sentence = sentences()[pick];
        let words: Vec<&str> = sentence.split(' ').collect();
        let kinds = [Word::Subject, Word::Action, Word::Target, Word::Details, Word::Owner];
        let phrases: Vec<(Word, String)> = reads
            .iter()
            .filter(|(_, from, _)| *from < words.len())
            .map(|(k, from, n)| (kinds[*k], words[*from..(*from + n).min(words.len())].join(" ")))
            .collect();
        let read: Vec<(Word, &str)> = phrases.iter().map(|(w, p)| (*w, p.as_str())).collect();
        let end = sentence.char_indices().nth(typed).map_or(sentence.len(), |(i, _)| i);
        let text = &sentence[..end];
        let marks = marks_for(text, &read);
        for pair in marks.windows(2) {
            prop_assert!(pair[0].range.end <= pair[1].range.start, "marks overlap: {marks:?}");
        }
        for mark in &marks {
            let marked = &text[mark.range.clone()];
            let symbol = marked.starts_with('@') || marked.starts_with(">:") || marked.starts_with('/') || marked.ends_with(';');
            prop_assert!(
                symbol || read.iter().any(|(w, p)| *w == mark.word && p.eq_ignore_ascii_case(marked)),
                "{marked:?} in {text:?} is marked though Jev didn't read it as {:?}",
                mark.word
            );
        }
    }
}

// ---------------------------------------------------------------- work: chat, steps, connect, strip

mod work {
    use super::*;
    use sk_one_window::steps::StepSpec;
    use sk_one_window::strip::TaskState;
    use sk_one_window::work::{WorkEffect, WorkEvent};

    #[derive(Clone, Debug)]
    enum Move {
        Type(usize),
        Key(Key),
        Task(u8, u8),
        Wait(u16),
    }

    fn moves() -> impl Strategy<Value = Move> {
        prop_oneof![
            3 => (0..WORDS.len()).prop_map(Move::Type),
            3 => prop_oneof![Just(Key::Down), Just(Key::Up), Just(Key::Enter), Just(Key::CmdK)].prop_map(Move::Key),
            3 => (0u8..3, 0u8..4).prop_map(|(t, s)| Move::Task(t, s)),
            2 => (1u16..900).prop_map(Move::Wait),
        ]
    }

    fn state(s: u8) -> TaskState {
        [
            TaskState::Working,
            TaskState::NeedsYou,
            TaskState::Ready,
            TaskState::Failed,
        ][s as usize % 4]
    }

    /// The same list work as `Run::new`, fed to two windows; one also gets agent tasks.
    fn pair() -> (OneWindow, OneWindow) {
        let mut a = OneWindow::default();
        a.handle(0, Event::Suggested(vec![Row::new("s:slack", "Slack")]));
        let mut b = a.clone();
        b.handle(
            0,
            Event::Work(WorkEvent::Agent {
                task: "fix SK-431".into(),
            }),
        );
        b.handle(0, Event::Key(Key::Escape));
        (a, b)
    }

    fn view_key(win: &mut OneWindow, now: u64, key: Key) -> Vec<Effect> {
        let mut fx = win.handle(now, Event::Key(key)).effects;
        let handed: Vec<Key> = fx
            .iter()
            .filter_map(|e| match e {
                Effect::ViewKey(k) => Some(k.clone()),
                _ => None,
            })
            .collect();
        for k in handed {
            fx.extend(win.handle(now, Event::Work(WorkEvent::ViewKey(k))).effects);
        }
        fx
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

        /// A task starting, moving along, asking or finishing never changes the list you're in.
        #[test]
        fn tasks_never_move_the_list(moves in prop::collection::vec(moves(), 1..60)) {
            let (mut plain, mut busy) = pair();
            let id = busy.work().strip().tasks()[0].id.clone();
            // Past the guard the agent's unprompted view armed, so both windows start alike.
            let mut now = ENTER_GUARD_MS;
            for m in moves {
                now += 10;
                match m {
                    Move::Type(i) => {
                        plain.handle(now, Event::Text(WORDS[i].into()));
                        busy.handle(now, Event::Text(WORDS[i].into()));
                    }
                    Move::Key(k) => {
                        plain.handle(now, Event::Key(k.clone()));
                        busy.handle(now, Event::Key(k));
                    }
                    Move::Task(p, s) => {
                        busy.handle(now, Event::Work(WorkEvent::TaskChanged {
                            id: id.clone(),
                            status: format!("step {p}"),
                            progress: f32::from(p) / 3.0,
                            state: state(s),
                            approve: None,
                        }));
                        plain.handle(now, Event::Tick);
                    }
                    Move::Wait(ms) => {
                        now += u64::from(ms);
                        plain.handle(now, Event::Tick);
                        busy.handle(now, Event::Tick);
                    }
                }
                if busy.in_view() || plain.in_view() {
                    continue;
                }
                prop_assert_eq!(plain.list().layout(), busy.list().layout());
                prop_assert_eq!(plain.list().sel, busy.list().sel);
                prop_assert_eq!(plain.list().header(), busy.list().header());
            }
        }

        /// ⌘J opens whatever needs you first, then the newest task.
        #[test]
        fn cmd_j_opens_what_needs_you(states in prop::collection::vec(0u8..4, 1..5)) {
            let mut win = OneWindow::default();
            let mut ids = Vec::new();
            for (i, s) in states.iter().enumerate() {
                let now = i as u64 * 10;
                let fx = win.handle(now, Event::Work(WorkEvent::Agent { task: format!("task {i}") })).effects;
                if let Some(Effect::Work(WorkEffect::StartAgent { id, .. })) = fx.iter().find(|e| matches!(e, Effect::Work(WorkEffect::StartAgent { .. }))) {
                    ids.push(id.clone());
                }
                win.handle(now, Event::Key(Key::Escape));
                win.handle(now, Event::Work(WorkEvent::TaskChanged {
                    id: ids[i].clone(), status: String::new(), progress: 0.0, state: state(*s), approve: None,
                }));
            }
            let want = states.iter().position(|s| state(*s) == TaskState::NeedsYou).unwrap_or(states.len() - 1);
            let fx = win.handle(1_000, Event::Key(Key::CmdJ)).effects;
            let reopened = Effect::Work(WorkEffect::Reopened { id: ids[want].clone() });
            prop_assert!(fx.contains(&reopened));
        }

        /// The steps never offer "Do all" while a step waits on a service; ↵ names the first one.
        #[test]
        fn steps_never_run_past_a_missing_service(needs in prop::collection::vec(prop::option::of(0usize..3), 2..5), connected in prop::collection::vec(0usize..3, 0..3)) {
            const KEYS: [&str; 3] = ["calendar", "slack", "github"];
            let sentence = (0..needs.len()).map(|i| format!("open app{i}")).collect::<Vec<_>>().join(" and then ");
            let mut win = OneWindow::default();
            win.handle(0, Event::Work(WorkEvent::Connections(connected.iter().map(|k| KEYS[*k].to_string()).collect())));
            win.handle(0, Event::Work(WorkEvent::OpenSteps { sentence: sentence.clone() }));
            let texts = sk_one_window::steps::split(&sentence);
            win.handle(10, Event::Work(WorkEvent::StepsRead {
                sentence: sentence.clone(),
                steps: texts.iter().zip(&needs).map(|(t, n)| StepSpec { text: t.clone(), title: None, needs: n.map(|k| KEYS[k].to_string()) }).collect(),
            }));
            let first_missing = needs.iter().enumerate().find_map(|(i, n)| n.filter(|k| !connected.contains(k)).map(|k| (i, KEYS[k])));
            let label = win.enter_target(10).label;
            let fx = view_key(&mut win, 10 + ENTER_GUARD_MS, Key::Enter);
            let ran = fx.iter().any(|e| matches!(e, Effect::Work(WorkEffect::RunSteps { .. })));
            match first_missing {
                Some((i, key)) => {
                    let name = sk_one_window::connect::name_of(key);
                    let named = label.starts_with(&format!("Connect {name} for step {}", i + 1));
                    prop_assert!(named, "{label}");
                    prop_assert!(!ran);
                    prop_assert_eq!(win.views().top().map(|f| f.name.clone()), Some("connect".to_string()));
                }
                None => {
                    prop_assert!(label.starts_with("Do "), "{label}");
                    prop_assert!(ran);
                }
            }
        }

        /// Once a service connects the connect view hands back, the line says so, and an Enter
        /// inside the guard is ignored.
        #[test]
        fn connecting_hands_back_under_the_guard(gap in 0u64..ENTER_GUARD_MS, from_steps in any::<bool>()) {
            let mut win = OneWindow::default();
            if from_steps {
                let s = "move standup to 10 and then open linear";
                win.handle(0, Event::Text(s.into()));
                win.handle(0, Event::Work(WorkEvent::OpenSteps { sentence: s.into() }));
                let texts = sk_one_window::steps::split(s);
                win.handle(10, Event::Work(WorkEvent::StepsRead {
                    sentence: s.into(),
                    steps: vec![
                        StepSpec { text: texts[0].clone(), title: None, needs: Some("calendar".into()) },
                        StepSpec { text: texts[1].clone(), title: None, needs: None },
                    ],
                }));
            } else {
                win.handle(0, Event::Text("move standup to 10".into()));
                win.handle(10, Event::Jev {
                    text: "move standup to 10".into(),
                    rows: vec![Row { needs: Some("calendar".into()), ..Row::new("jev:move", "Move standup to 10") }],
                });
            }
            view_key(&mut win, 1_000, Key::Enter);
            prop_assert_eq!(win.views().top().map(|f| f.name.clone()), Some("connect".to_string()));
            view_key(&mut win, 1_100, Key::Enter);
            win.handle(2_000, Event::Work(WorkEvent::Connected { integration: "calendar".into() }));
            prop_assert!(win.views().top().is_none_or(|f| f.name != "connect"));
            prop_assert_eq!(win.work().connections().has("calendar"), true);
            if !from_steps {
                prop_assert_eq!(win.slot_line(2_000).title, "Google Calendar connected");
            }
            let out = win.handle(2_000 + gap, Event::Key(Key::Enter));
            prop_assert!(out.effects.contains(&Effect::EnterIgnored));
        }

        /// In the model's chat what you type owns ↵, and Esc always gives the sentence back.
        #[test]
        fn the_chat_takes_your_words_and_esc_restores_the_sentence(sentence in 0..WORDS.len(), typed in 1..WORDS.len(), replies in 0u8..3) {
            let sentence = WORDS[sentence];
            let mut win = OneWindow::default();
            win.handle(0, Event::Text(sentence.into()));
            win.handle(0, Event::Work(WorkEvent::AskModel { sentence: sentence.into() }));
            for r in 0..replies {
                win.handle(10 + u64::from(r), Event::Work(WorkEvent::ModelWrote { text: format!("reply {r}"), done: r + 1 == replies }));
            }
            win.handle(100, Event::Text(WORDS[typed].into()));
            let t = WORDS[typed].trim();
            if !t.is_empty() {
                prop_assert_eq!(win.enter_target(100).label, format!("Ask the model: “{t}”"));
            }
            let fx = win.handle(200, Event::Key(Key::Escape)).effects;
            prop_assert!(!win.in_view());
            prop_assert_eq!(win.text(), sentence);
            prop_assert!(fx.contains(&Effect::SetInput(sentence.into())) || sentence == WORDS[typed]);
        }
    }
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

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    /// Something that can't be taken back runs only once its whole wait has passed untouched,
    /// and never once it was stopped.
    #[test]
    fn an_irreversible_run_goes_only_after_its_wait(
        ticks in prop::collection::vec(1u64..3_000, 1..12),
        stop_at in prop::option::of(0usize..12),
    ) {
        let mut win = OneWindow::default();
        let out = win.handle(0, Event::Irreversible(Irreversible::new("trash", "Empty Trash")));
        let id = out.effects.iter().find_map(|e| match e {
            Effect::Held { op: HeldOp::Start, id, .. } => Some(*id),
            _ => None,
        }).expect("it waits");
        let (mut now, mut stopped, mut sent) = (0u64, false, 0);
        for (i, dt) in ticks.into_iter().enumerate() {
            now += dt;
            let ev = if stop_at == Some(i) { Event::Key(Key::CmdZ) } else { Event::Tick };
            stopped |= stop_at == Some(i) && now < sk_one_window::contract::MESSAGE_HOLD_MS;
            for fx in win.handle(now, ev).effects {
                if let Effect::Held { op: HeldOp::Sent, id: sent_id, .. } = fx {
                    prop_assert_eq!(sent_id, id);
                    prop_assert!(now >= sk_one_window::contract::MESSAGE_HOLD_MS, "ran early at {}", now);
                    prop_assert!(!stopped, "ran after it was stopped");
                    sent += 1;
                }
            }
        }
        prop_assert!(sent <= 1, "ran {} times", sent);
    }

    /// A notice is the line's last thing at once, whether the window shows or comes back later.
    #[test]
    fn a_notice_is_what_the_line_says_next(hidden in any::<bool>(), later in 0u64..60_000) {
        let mut win = OneWindow::default();
        if hidden {
            win.handle(0, Event::Dismissed);
        }
        win.handle(1, Event::Notice(Notice::new("check", "Backup finished")));
        if hidden {
            win.handle(1 + later, Event::Summon);
        }
        let line = win.slot_line(1 + later);
        prop_assert_eq!(line.kind, SlotKind::Recent);
        prop_assert_eq!(line.title.as_str(), "Backup finished");
    }
}

mod views {
    use proptest::prelude::*;
    use sk_one_window::contract::ENTER_GUARD_MS;
    use sk_one_window::form::{FieldState, Form, FormKey, Reading, Slot, SlotName, Source, Verb};
    use sk_one_window::window::{Effect, Event, Key, OneWindow, ViewOp, ViewSpec};
    use sk_one_window::{EnterTarget, Row, Subject, SubjectKind};

    const SLOTS: [SlotName; 5] = [
        SlotName::Who,
        SlotName::What,
        SlotName::When,
        SlotName::Where,
        SlotName::Event,
    ];

    fn verb() -> impl Strategy<Value = Verb> {
        prop_oneof![
            Just(Verb::Message),
            Just(Verb::Remind),
            Just(Verb::Schedule),
            Just(Verb::Move),
            Just(Verb::Note),
            Just(Verb::Other("rename".into())),
        ]
    }

    fn reading() -> impl Strategy<Value = Reading> {
        (
            verb(),
            prop::collection::vec(
                (0..SLOTS.len(), prop::option::of(0usize..60), 0..3usize),
                0..6,
            ),
            any::<bool>(),
        )
            .prop_map(|(verb, slots, staged)| Reading {
                verb,
                sentence: "tell sarah the beta slipped".into(),
                slots: slots
                    .into_iter()
                    .map(|(name, at, alts)| {
                        let mut slot =
                            Slot::new(SLOTS[name], format!("v{name}"), Source::YourWords);
                        slot.at = at;
                        for a in 0..alts {
                            slot =
                                slot.with_alt(format!("alt{a}"), Source::Known("calendar".into()));
                        }
                        slot
                    })
                    .collect(),
                subject: staged.then(|| Subject {
                    kind: SubjectKind::Clipboard,
                    title: "copied".into(),
                    ..Subject::default()
                }),
            })
    }

    /// Something pasted again starts at its first action, wherever the last paste was left.
    #[test]
    fn a_new_paste_starts_at_the_first_action() {
        let mut win = OneWindow::default();
        win.handle(0, Event::Paste("84".into()));
        win.handle(10, Event::Key(Key::Down));
        win.handle(20, Event::Key(Key::Down));
        assert_eq!(win.list().sel, 2);
        win.handle(30, Event::Key(Key::Escape));
        assert!(win.staged().is_none());
        win.handle(40, Event::Paste("84".into()));
        assert_eq!(win.list().sel, 0);
    }

    /// Running one of the staged subject's rows lets go of it, and the list it leaves starts
    /// at its first row, not at the action's place.
    #[test]
    fn a_staged_row_that_ran_leaves_the_list_at_its_top() {
        let mut win = OneWindow::default();
        win.handle(
            0,
            Event::Suggested(vec![
                Row::new("s:a", "A"),
                Row::new("s:b", "B"),
                Row::new("s:c", "C"),
                Row::new("s:d", "D"),
                Row::new("s:e", "E"),
            ]),
        );
        win.handle(10, Event::Paste("84".into()));
        for t in [20, 30, 40] {
            win.handle(t, Event::Key(Key::Down));
        }
        win.handle(50, Event::Key(Key::Enter));
        assert!(win.staged().is_none());
        assert_eq!(win.list().sel, 0);
    }

    /// What was copied lists the window's own actions for it; the host is never asked, so its
    /// actions for a highlighted row can't replace them.
    #[test]
    fn a_paste_lists_what_you_can_do_with_it_without_asking_the_host() {
        let mut win = OneWindow::default();
        let out = win.handle(0, Event::Paste("github.com/script-kit/app/pull/812".into()));
        assert!(
            !out.effects
                .iter()
                .any(|fx| matches!(fx, Effect::ActionsFor(_)))
        );
        let staged = win.staged().expect("staged");
        assert!(!staged.waiting_for_rows);
        let titles: Vec<&str> = staged.rows.iter().map(|r| r.title.as_str()).collect();
        assert_eq!(
            titles,
            [
                "Open it",
                "Send it to someone",
                "Remind me about it",
                "Add it to today's note",
                "Show all of it"
            ]
        );
    }

    proptest! {
        /// A form shows each slot once, in the order the sentence said them, and the caret starts
        /// on the first field nobody filled; Enter never submits while one is empty.
        #[test]
        fn a_form_asks_for_exactly_what_jev_could_not_fill(reading in reading()) {
            let mut form = Form::new(&reading);
            let mut seen = std::collections::HashSet::new();
            for f in &form.fields {
                prop_assert!(seen.insert(f.name), "{:?} twice", f.name);
                prop_assert_eq!(f.state == FieldState::Needed, f.from.is_none());
            }
            let positions: Vec<usize> = form
                .fields
                .iter()
                .filter_map(|f| reading.slots.iter().find(|s| s.name == f.name && f.from == Some(s.from.clone())).and_then(|s| s.at))
                .collect();
            prop_assert!(positions.windows(2).all(|w| w[0] <= w[1]), "sentence order {:?}", positions);
            let needed = form.fields.iter().position(|f| f.state == FieldState::Needed);
            prop_assert_eq!(form.caret, needed);
            if needed.is_some() {
                prop_assert_eq!(form.key(&Key::Enter), FormKey::Changed);
                prop_assert_eq!(form.caret, needed);
            }
        }

        /// Tab visits every field once and comes back where it started.
        #[test]
        fn tab_goes_round_the_fields(reading in reading(), start in 0usize..6) {
            let mut form = Form::new(&reading);
            let n = form.fields.len();
            prop_assume!(n > 0);
            form.caret = Some(start % n);
            let mut visited = std::collections::HashSet::new();
            for _ in 0..n {
                form.key(&Key::Tab);
                visited.insert(form.caret.expect("a caret"));
            }
            prop_assert_eq!(visited.len(), n);
            prop_assert_eq!(form.caret, Some(start % n));
        }

        /// ⌘V into the empty input with no view open stages what was copied; anywhere else the
        /// paste is the host's, and nothing is staged.
        #[test]
        fn pasting_stages_only_into_an_empty_input(typed in any::<bool>(), open in any::<bool>(), text in "[a-z ./:]{0,12}") {
            let mut win = OneWindow::default();
            if typed {
                win.handle(0, Event::Text("tell".into()));
            }
            if open {
                win.handle(10, Event::View(ViewOp::Open(ViewSpec { name: "form".into(), takes_input: true, ..ViewSpec::default() })));
            }
            let out = win.handle(20, Event::Paste(text.clone()));
            let stages = !typed && !open && !text.trim().is_empty();
            prop_assert_eq!(win.staged().is_some(), stages);
            prop_assert_eq!(out.consumed, stages);
            if stages {
                let staged = win.staged().expect("staged");
                prop_assert_eq!(staged.subject.kind, SubjectKind::Clipboard);
                prop_assert_eq!(win.slot_line(20).title, staged.subject.title.clone());
                prop_assert!(!win.enter_armed(20), "your own paste never guards Enter");
            } else {
                let passed = out.effects.iter().any(|e| matches!(e, Effect::PassKey(_)));
                prop_assert!(passed);
            }
        }

        /// An answer that arrives in an open view changes ↵ on its own, so an Enter pressed within
        /// the guard is ignored rather than taken by the new answer.
        #[test]
        fn an_arriving_answer_never_takes_an_enter_meant_for_before(wait in 1u64..2000, gap in 0u64..(2 * ENTER_GUARD_MS)) {
            let mut win = OneWindow::default();
            win.handle(0, Event::Text("12*7".into()));
            win.handle(0, Event::View(ViewOp::Open(ViewSpec {
                name: "answer".into(),
                enter: Some(EnterTarget::new("answer:wait", "Reading…")),
                takes_input: true,
                ..ViewSpec::default()
            })));
            win.handle(wait, Event::ViewEnter(EnterTarget::new("answer:number:84", "Copy 84")));
            let out = win.handle(wait + gap, Event::Key(Key::Enter));
            let ignored = out.effects.iter().any(|e| matches!(e, Effect::EnterIgnored));
            let to_view = out.effects.iter().any(|e| matches!(e, Effect::ViewKey(Key::Enter)));
            prop_assert_eq!(ignored, gap < ENTER_GUARD_MS);
            prop_assert_eq!(to_view, gap >= ENTER_GUARD_MS);
        }
    }
}
