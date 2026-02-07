use super::*;

#[test]
fn test_parse_shortcut_keycaps() {
    let keycaps = TerminalCommandBar::parse_shortcut_keycaps("⌘C");
    assert_eq!(keycaps, vec!["⌘", "C"]);

    let keycaps = TerminalCommandBar::parse_shortcut_keycaps("⌃⇧T");
    assert_eq!(keycaps, vec!["⌃", "⇧", "T"]);
}

#[test]
fn test_should_accept_search_char_rejects_control_chars() {
    assert!(TerminalCommandBar::should_accept_search_char('a'));
    assert!(TerminalCommandBar::should_accept_search_char('⌘'));
    assert!(!TerminalCommandBar::should_accept_search_char('\n'));
    assert!(!TerminalCommandBar::should_accept_search_char('\t'));
}

#[test]
fn test_command_list_height_uses_minimum_row_when_empty() {
    let height = TerminalCommandBar::command_list_height(0);
    assert_eq!(height, COMMAND_ITEM_HEIGHT);
}

#[test]
fn test_command_list_height_caps_at_max() {
    let height = TerminalCommandBar::command_list_height(1_000);
    let expected_cap = COMMAND_BAR_MAX_HEIGHT - SEARCH_INPUT_HEIGHT;
    assert_eq!(height, expected_cap);
}
