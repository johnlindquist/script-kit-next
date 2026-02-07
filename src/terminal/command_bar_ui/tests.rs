use super::*;

#[test]
fn test_parse_shortcut_keycaps() {
    let keycaps = TerminalCommandBar::parse_shortcut_keycaps("⌘C");
    assert_eq!(keycaps, vec!["⌘", "C"]);

    let keycaps = TerminalCommandBar::parse_shortcut_keycaps("⌃⇧T");
    assert_eq!(keycaps, vec!["⌃", "⇧", "T"]);
}
