use super::*;

#[test]
fn test_new_input() {
    let input = TextInputState::new();
    assert!(input.is_empty());
    assert_eq!(input.cursor(), 0);
    assert!(!input.has_selection());
}

#[test]
fn test_with_text() {
    let input = TextInputState::with_text("hello");
    assert_eq!(input.text(), "hello");
    assert_eq!(input.cursor(), 5); // At end
}

#[test]
fn test_insert_char() {
    let mut input = TextInputState::new();
    input.insert_char('a');
    input.insert_char('b');
    assert_eq!(input.text(), "ab");
    assert_eq!(input.cursor(), 2);
}

#[test]
fn test_backspace() {
    let mut input = TextInputState::with_text("abc");
    input.backspace();
    assert_eq!(input.text(), "ab");
    assert_eq!(input.cursor(), 2);
}

#[test]
fn test_selection() {
    let mut input = TextInputState::with_text("hello");
    input.move_to_start(false);
    input.move_right(true); // Select 'h'
    input.move_right(true); // Select 'he'
    assert_eq!(input.selected_text(), "he");
    assert!(input.has_selection());
}

#[test]
fn test_select_all() {
    let mut input = TextInputState::with_text("hello");
    input.select_all();
    assert_eq!(input.selected_text(), "hello");
}

#[test]
fn test_delete_selection() {
    let mut input = TextInputState::with_text("hello");
    input.select_all();
    input.backspace();
    assert!(input.is_empty());
}

#[test]
fn test_insert_replaces_selection() {
    let mut input = TextInputState::with_text("hello");
    input.select_all();
    input.insert_char('x');
    assert_eq!(input.text(), "x");
}

#[test]
fn test_display_text_secret() {
    let input = TextInputState::with_text("secret");
    assert_eq!(input.display_text(false), "secret");
    assert_eq!(input.display_text(true), "••••••");
}

#[test]
fn test_move_collapse_selection() {
    let mut input = TextInputState::with_text("hello");
    input.select_all();
    input.move_left(false); // Should collapse to start
    assert!(!input.has_selection());
    assert_eq!(input.cursor(), 0);
}

#[test]
fn test_word_boundary() {
    let mut input = TextInputState::with_text("hello world");
    input.move_to_end(false);
    input.move_word_left(false);
    assert_eq!(input.cursor(), 6); // At 'w'
    input.move_word_left(false);
    assert_eq!(input.cursor(), 0); // At start
}

#[test]
fn test_unicode() {
    let mut input = TextInputState::with_text("héllo");
    assert_eq!(input.text().chars().count(), 5);
    input.move_to_start(false);
    input.move_right(false);
    input.move_right(false);
    assert_eq!(input.cursor(), 2); // After 'hé'
}
