//! Views under the input: pasted subjects, forms and answers.

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
                    let mut slot = Slot::new(SLOTS[name], format!("v{name}"), Source::YourWords);
                    slot.at = at;
                    for a in 0..alts {
                        slot = slot.with_alt(format!("alt{a}"), Source::Known("calendar".into()));
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
            prop_assert_eq!(win.chip().map(|c| c.title), Some(staged.subject.title.clone()));
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
