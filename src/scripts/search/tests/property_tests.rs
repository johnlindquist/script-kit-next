//! Rules for launcher matching and Actions ranking, checked against
//! generated titles and queries.

use gpui::proptest::prelude::*;

use super::super::highlight::SearchHighlightMatchCtx;
use crate::actions::{Action, ActionCategory, ActionsDialog};
use crate::test_utils::property::assert_rule;

const WORDS: &[&str] = &["open", "notes", "clip", "board", "timer", "zap", "kit"];

/// Characters built from several code points, plus single wide ones.
const WIDE: &[&str] = &[
    "\u{1f44d}\u{1f3fd}",
    "\u{1f468}\u{200d}\u{1f469}\u{200d}\u{1f467}",
    "\u{1f1ef}\u{1f1f5}",
    "e\u{301}",
    "\u{1f680}",
    "\u{4eac}",
];

/// A title of space-separated parts with at least one plain word.
fn title() -> impl Strategy<Value = String> {
    let part = prop_oneof![
        3 => prop::sample::select(WORDS),
        2 => prop::sample::select(WIDE),
    ];
    (
        prop::sample::select(WORDS),
        prop::collection::vec(part, 0..5),
        any::<prop::sample::Index>(),
    )
        .prop_map(|(word, mut parts, at)| {
            let at = at.index(parts.len() + 1);
            parts.insert(at, word);
            parts.join(" ")
        })
}

/// A title plus a query of ASCII letters picked from it, in order.
fn title_and_query() -> impl Strategy<Value = (String, String)> {
    title().prop_flat_map(|title| {
        let letters: Vec<char> = title
            .chars()
            .filter(|character| character.is_ascii_lowercase())
            .collect();
        let most = letters.len().min(6);
        (
            Just(title),
            prop::sample::subsequence(letters, 1..=most)
                .prop_map(|picked| picked.into_iter().collect::<String>()),
        )
    })
}

#[test]
fn highlights_land_on_the_letters_the_query_typed() {
    assert_rule(
        "A matching title highlights exactly the characters the query typed",
        512,
        title_and_query(),
        |(title, query)| {
            let (matched, indices) = SearchHighlightMatchCtx::new(&query).indices_for(&title);
            prop_assert!(matched, "letters picked from the title must match it");
            let characters: Vec<char> = title.chars().collect();
            prop_assert!(
                indices.windows(2).all(|pair| pair[0] < pair[1]),
                "highlights out of order: {:?}",
                indices
            );
            prop_assert!(
                indices.iter().all(|&index| index < characters.len()),
                "highlight past the end of a {}-character title: {:?}",
                characters.len(),
                indices
            );
            let highlighted: String = indices.iter().map(|&index| characters[index]).collect();
            prop_assert_eq!(highlighted, query);
            Ok(())
        },
    );
}

const QUERY_LETTERS: &[char] = &['a', 'b', 'c', 'd', 'e', 'f'];
const FILLER_LETTERS: &[char] = &['u', 'v', 'w', 'x', 'y'];

/// Query letters and filler letters never overlap, so filler cannot make the
/// query appear contiguously.
fn query() -> impl Strategy<Value = String> {
    prop::collection::vec(prop::sample::select(QUERY_LETTERS), 2..5)
        .prop_map(|letters| letters.into_iter().collect())
}

fn filler() -> impl Strategy<Value = String> {
    prop::collection::vec(prop::sample::select(FILLER_LETTERS), 1..4)
        .prop_map(|letters| letters.into_iter().collect())
}

#[test]
fn a_title_prefix_match_outranks_a_scattered_match() {
    assert_rule(
        "In Actions, a title that starts with the query outranks one that only contains its letters scattered",
        512,
        (query(), filler(), prop::collection::vec(filler(), 4)),
        |(query, tail, gaps)| {
            let prefix_title = format!("{query}{tail}");
            let mut scattered_title = String::new();
            for (position, letter) in query.chars().enumerate() {
                if position > 0 {
                    scattered_title.push_str(&gaps[(position - 1) % gaps.len()]);
                }
                scattered_title.push(letter);
            }
            let prefix = Action::new("prefix", prefix_title, None, ActionCategory::ScriptContext);
            // The scattered row also earns every other bonus the query can buy.
            let scattered = Action::new(
                "scattered",
                scattered_title,
                Some(format!("about {query} here")),
                ActionCategory::ScriptContext,
            )
            .with_shortcut(query.clone());
            let prefix_score = ActionsDialog::score_action(&prefix, &query);
            let scattered_score = ActionsDialog::score_action(&scattered, &query);
            prop_assert!(
                prefix_score > scattered_score,
                "prefix scored {} but scattered scored {}",
                prefix_score,
                scattered_score
            );
            Ok(())
        },
    );
}
