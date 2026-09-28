//! The highlighted row is the user's. While one sentence sits in the input, nothing that arrives
//! on its own (another answer from Jev, Jev going quiet, names searched again, time passing)
//! replaces it or moves the highlight off it, and Return runs exactly it.

use proptest::prelude::*;
use sk_one_window::Row;
use sk_one_window::window::{Effect, Event, Key, OneWindow};

const SAID: &str = "quit gmgui";
const READINGS: &[&str] = &[
    "Quit GPUI",
    "Focus the “Quit gmgui” window",
    "Quit Gmail",
    "Search the web for gmgui",
];
const NAMES: &[&str] = &["Quit Script Kit", "Quick Terminal", "Quit All Apps"];

#[derive(Clone, Debug)]
enum Arrival {
    /// Jev's readings of the sentence (by index, with how likely each is), late or again.
    Answer(Vec<(usize, u8)>),
    /// Jev gave up on the sentence.
    Quiet,
    /// The search listed names for the sentence again.
    Names(Vec<(usize, u8)>),
    Wait(u16),
    /// The user moved the highlight: the one thing here that may change it.
    Move(bool),
}

fn arrival() -> impl Strategy<Value = Arrival> {
    prop_oneof![
        4 => prop::collection::vec((0..READINGS.len(), 30u8..=100), 0..4).prop_map(Arrival::Answer),
        1 => Just(Arrival::Quiet),
        2 => prop::collection::vec((0..NAMES.len(), 0u8..=100), 0..3).prop_map(Arrival::Names),
        3 => (1u16..1_500).prop_map(Arrival::Wait),
        2 => any::<bool>().prop_map(Arrival::Move),
    ]
}

fn reading(i: usize, p: u8) -> Row {
    Row {
        p: Some(f32::from(p) / 100.0),
        ..Row::new(format!("jev:{}", READINGS[i]), READINGS[i])
    }
}

/// Each index once, first mention first.
fn once(picks: Vec<(usize, u8)>) -> Vec<(usize, u8)> {
    let mut seen = Vec::new();
    picks
        .into_iter()
        .filter(|(i, _)| {
            let new = !seen.contains(i);
            seen.push(*i);
            new
        })
        .collect()
}

fn event(a: &Arrival) -> Event {
    match a {
        Arrival::Answer(picks) => Event::Jev {
            text: SAID.into(),
            rows: once(picks.clone())
                .into_iter()
                .map(|(i, p)| reading(i, p))
                .collect(),
        },
        Arrival::Quiet => Event::JevDown { text: SAID.into() },
        Arrival::Names(picks) => Event::Names(
            once(picks.clone())
                .into_iter()
                .map(|(i, score)| {
                    Row::new(format!("name:{}", NAMES[i]), NAMES[i]).with_score(score)
                })
                .collect(),
        ),
        Arrival::Wait(_) => Event::Tick,
        Arrival::Move(down) => Event::Key(if *down { Key::Down } else { Key::Up }),
    }
}

/// What Return would run right now, without running it.
fn return_runs(win: &OneWindow, now: u64) -> Result<Option<String>, &'static str> {
    let out = win.clone().handle(now, Event::Key(Key::Enter));
    if out.effects.contains(&Effect::EnterIgnored) {
        return Err("ignored");
    }
    Ok(out.picked.map(|(row, _)| row.key))
}

fn slot_titles(win: &OneWindow) -> Vec<String> {
    let list = win.list();
    (0..list.jev.len())
        .filter_map(|i| list.jev.get(i).and_then(|s| s.row.as_ref()))
        .map(|row| row.title.clone())
        .collect()
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 1024, ..ProptestConfig::default() })]

    #[test]
    fn return_runs_what_the_highlight_shows(arrivals in prop::collection::vec(arrival(), 1..40)) {
        let mut win = OneWindow::default();
        let mut now = 0u64;
        win.handle(now, Event::Text(SAID.into()));
        for a in arrivals {
            now += match a {
                Arrival::Wait(ms) => u64::from(ms),
                _ => 1 + (now % 7) * 40,
            };
            let was = win.list().focused().map(|row| row.key.clone());
            let held_back = return_runs(&win, now).is_err();
            win.handle(now, event(&a));
            if matches!(a, Arrival::Move(_)) {
                continue;
            }
            let Some(key) = was else {
                continue;
            };
            let is = win.list().focused().map(|row| row.key.clone());
            prop_assert_eq!(is.as_deref(), Some(key.as_str()), "{:?} took the highlight off “{}”", a, key);
            match return_runs(&win, now) {
                Ok(ran) => prop_assert_eq!(ran.as_deref(), Some(key.as_str()), "Return ran something else after {:?}", a),
                Err(_) => prop_assert!(held_back, "nothing under the highlight changed, yet {:?} held Return back", a),
            }
        }
    }
}

/// The session in the log: Jev gave up on “quit gmgui” after five seconds, its answer came two
/// seconds later, and Return three seconds after that ran the reading that had replaced the row
/// under the highlight.
#[test]
fn a_late_answer_lands_below_what_return_runs() {
    let mut win = OneWindow::default();
    win.handle(0, Event::Text(SAID.into()));
    win.handle(5_000, Event::JevDown { text: SAID.into() });
    let before = win.enter_target(5_000);
    assert_eq!(before.label, "Ask the model: “quit gmgui”");
    win.handle(
        6_850,
        Event::Jev {
            text: SAID.into(),
            rows: vec![reading(1, 90), reading(0, 60)],
        },
    );
    assert_eq!(win.enter_target(6_850), before);
    assert_eq!(
        slot_titles(&win),
        [
            "Ask the model: “quit gmgui”",
            "Focus the “Quit gmgui” window",
            "Quit GPUI"
        ]
    );
    assert_eq!(return_runs(&win, 9_750), Ok(Some("down:model".into())));
}

#[test]
fn a_second_answer_for_the_same_words_never_swaps_the_highlighted_reading() {
    let mut win = OneWindow::default();
    win.handle(0, Event::Text(SAID.into()));
    win.handle(
        700,
        Event::Jev {
            text: SAID.into(),
            rows: vec![reading(2, 70)],
        },
    );
    win.handle(
        2_000,
        Event::Jev {
            text: SAID.into(),
            rows: vec![reading(0, 95)],
        },
    );
    assert_eq!(slot_titles(&win), ["Quit Gmail", "Quit GPUI"]);
    assert_eq!(return_runs(&win, 2_010), Ok(Some("jev:Quit Gmail".into())));
    win.handle(2_020, Event::Key(Key::Down));
    assert_eq!(return_runs(&win, 2_030), Ok(Some("jev:Quit GPUI".into())));
}

#[test]
fn going_quiet_after_an_answer_keeps_the_answer() {
    let mut win = OneWindow::default();
    win.handle(0, Event::Text(SAID.into()));
    win.handle(
        700,
        Event::Jev {
            text: SAID.into(),
            rows: vec![reading(0, 95)],
        },
    );
    win.handle(5_700, Event::JevDown { text: SAID.into() });
    assert_eq!(slot_titles(&win), ["Quit GPUI"]);
    assert_eq!(return_runs(&win, 5_710), Ok(Some("jev:Quit GPUI".into())));
}

#[test]
fn a_name_searched_away_stays_under_the_highlight() {
    let mut win = OneWindow::default();
    win.handle(0, Event::Text("quit".into()));
    win.handle(
        0,
        Event::Names(vec![
            Row::new("name:Quit Script Kit", "Quit Script Kit").with_score(90),
            Row::new("name:Quick Terminal", "Quick Terminal").with_score(80),
        ]),
    );
    win.handle(100, Event::Key(Key::Down));
    assert_eq!(
        return_runs(&win, 110),
        Ok(Some("name:Quick Terminal".into()))
    );
    win.handle(
        900,
        Event::Names(vec![
            Row::new("name:Quit Script Kit", "Quit Script Kit").with_score(90),
            Row::new("name:Quit All Apps", "Quit All Apps").with_score(85),
        ]),
    );
    assert_eq!(
        return_runs(&win, 910),
        Ok(Some("name:Quick Terminal".into()))
    );
}
