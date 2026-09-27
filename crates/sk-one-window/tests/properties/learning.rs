//! Sentences that ran become learned rows, without moving the list you're typing into.

use std::collections::HashMap;

use proptest::prelude::*;
use sk_one_window::Row;
use sk_one_window::window::{Event, Key, OneWindow};

use super::WORDS;

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
