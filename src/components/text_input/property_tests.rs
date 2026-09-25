//! Rules for the single-line text field, checked against generated text.
//!
//! Text is built from pieces that mix plain letters with characters made of
//! several code points (a skin-toned emoji, a family emoji, a flag, a keycap,
//! a letter with a combining accent). Those are where cursor arithmetic that
//! counts code points instead of visible characters goes wrong.

use gpui::proptest::prelude::*;
use unicode_segmentation::UnicodeSegmentation;

use super::core::TextInputState;
use crate::test_utils::property::assert_rule;

const PIECES: &[&str] = &[
    "a",
    "b",
    "z",
    " ",
    "\u{e9}",
    "e\u{301}",
    "\u{4eac}",
    "\u{1f44d}",
    "\u{1f44d}\u{1f3fd}",
    "\u{1f468}\u{200d}\u{1f469}\u{200d}\u{1f467}",
    "\u{1f1ef}\u{1f1f5}",
    "1\u{fe0f}\u{20e3}",
];

fn text() -> impl Strategy<Value = String> {
    prop::collection::vec(prop::sample::select(PIECES), 0..8).prop_map(|pieces| pieces.concat())
}

fn byte_offset(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map_or(text.len(), |(byte, _)| byte)
}

fn between_whole_characters(text: &str, char_index: usize) -> bool {
    let byte = byte_offset(text, char_index);
    byte == text.len() || text.grapheme_indices(true).any(|(start, _)| start == byte)
}

#[derive(Clone, Debug)]
enum Key {
    Left,
    Right,
    ShiftLeft,
    ShiftRight,
    OptionLeft,
    OptionRight,
    Home,
    End,
    Backspace,
    Delete,
    OptionBackspace,
    OptionDelete,
    Type(&'static str),
}

fn key() -> impl Strategy<Value = Key> {
    prop_oneof![
        Just(Key::Left),
        Just(Key::Right),
        Just(Key::ShiftLeft),
        Just(Key::ShiftRight),
        Just(Key::OptionLeft),
        Just(Key::OptionRight),
        Just(Key::Home),
        Just(Key::End),
        Just(Key::Backspace),
        Just(Key::Delete),
        Just(Key::OptionBackspace),
        Just(Key::OptionDelete),
        prop::sample::select(PIECES).prop_map(Key::Type),
    ]
}

fn press(state: &mut TextInputState, key: &Key) {
    match key {
        Key::Left => state.move_left(false),
        Key::Right => state.move_right(false),
        Key::ShiftLeft => state.move_left(true),
        Key::ShiftRight => state.move_right(true),
        Key::OptionLeft => state.move_word_left(false),
        Key::OptionRight => state.move_word_right(false),
        Key::Home => state.move_to_start(false),
        Key::End => state.move_to_end(false),
        Key::Backspace => state.handle_backspace_shortcut(false, false),
        Key::Delete => state.handle_delete_shortcut(false),
        Key::OptionBackspace => state.handle_backspace_shortcut(false, true),
        Key::OptionDelete => state.handle_delete_shortcut(true),
        Key::Type(piece) => state.insert_str(piece),
    }
}

#[test]
fn backspace_removes_exactly_one_whole_character() {
    assert_rule(
        "Backspace at the end removes exactly one visible character",
        512,
        text(),
        |text| {
            let mut state = TextInputState::with_text(text.clone());
            state.backspace();
            let mut characters: Vec<&str> = text.graphemes(true).collect();
            characters.pop();
            prop_assert_eq!(state.text(), characters.concat());
            Ok(())
        },
    );
}

#[test]
fn delete_removes_exactly_one_whole_character() {
    assert_rule(
        "Delete at the start removes exactly one visible character",
        512,
        text(),
        |text| {
            let mut state = TextInputState::with_text(text.clone());
            state.move_to_start(false);
            state.delete();
            let characters: Vec<&str> = text.graphemes(true).skip(1).collect();
            prop_assert_eq!(state.text(), characters.concat());
            Ok(())
        },
    );
}

#[test]
fn caret_and_selection_always_sit_between_whole_characters() {
    assert_rule(
        "After any key, the caret and selection edges sit between visible characters",
        512,
        (text(), prop::collection::vec(key(), 1..24)),
        |(text, keys)| {
            let mut state = TextInputState::with_text(text);
            for key in &keys {
                press(&mut state, key);
                let selection = state.selection();
                prop_assert!(
                    between_whole_characters(state.text(), selection.cursor)
                        && between_whole_characters(state.text(), selection.anchor),
                    "{:?} left the caret inside a character: text={:?} anchor={} cursor={}",
                    key,
                    state.text(),
                    selection.anchor,
                    selection.cursor
                );
            }
            Ok(())
        },
    );
}
