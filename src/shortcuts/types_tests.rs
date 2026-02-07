//! Tests for shortcut types

use super::types::*;

// ========================
// Parse Tests
// ========================

#[test]
fn parse_empty_string_returns_error() {
    assert_eq!(Shortcut::parse(""), Err(ShortcutParseError::Empty));
    assert_eq!(Shortcut::parse("   "), Err(ShortcutParseError::Empty));
}

#[test]
fn parse_modifiers_only_returns_error() {
    assert_eq!(Shortcut::parse("cmd"), Err(ShortcutParseError::MissingKey));
    assert_eq!(
        Shortcut::parse("cmd+shift"),
        Err(ShortcutParseError::MissingKey)
    );
}

#[test]
fn parse_unknown_key_returns_error() {
    assert_eq!(
        Shortcut::parse("cmd+madeup"),
        Err(ShortcutParseError::UnknownKey("madeup".to_string()))
    );
}

#[test]
fn parse_error_messages_include_recovery_guidance() {
    assert_eq!(
        ShortcutParseError::Empty.to_string(),
        "Shortcut is empty. Enter one key, for example 'cmd+k' or 'ctrl+k'."
    );
    assert_eq!(
        ShortcutParseError::MissingKey.to_string(),
        "Shortcut is missing a key. Add one key after modifiers, for example 'cmd+k'."
    );
    assert_eq!(
        ShortcutParseError::UnknownToken("extra".to_string()).to_string(),
        "Unexpected token 'extra' in shortcut. Use optional modifiers plus one key, for example 'cmd+shift+k'."
    );
    assert_eq!(
        ShortcutParseError::UnknownKey("madeup".to_string()).to_string(),
        "Unknown key 'madeup'. Use a letter, number, function key (f1-f12), or named key like 'enter' or 'escape'."
    );
}

#[test]
fn parse_simple_key() {
    let s = Shortcut::parse("k").unwrap();
    assert_eq!(s.key, "k");
    assert!(!s.modifiers.cmd);
    assert!(!s.modifiers.shift);
}

#[test]
fn parse_cmd_plus_key() {
    let s = Shortcut::parse("cmd+k").unwrap();
    assert_eq!(s.key, "k");
    assert!(s.modifiers.cmd);
    assert!(!s.modifiers.shift);
}

#[test]
fn parse_multiple_modifiers() {
    let s = Shortcut::parse("cmd+shift+k").unwrap();
    assert_eq!(s.key, "k");
    assert!(s.modifiers.cmd);
    assert!(s.modifiers.shift);
    assert!(!s.modifiers.ctrl);
    assert!(!s.modifiers.alt);
}

#[test]
fn parse_all_modifiers() {
    let s = Shortcut::parse("ctrl+alt+shift+cmd+k").unwrap();
    assert!(s.modifiers.cmd);
    assert!(s.modifiers.ctrl);
    assert!(s.modifiers.alt);
    assert!(s.modifiers.shift);
}

#[test]
fn parse_space_separated() {
    let s = Shortcut::parse("cmd shift k").unwrap();
    assert_eq!(s.key, "k");
    assert!(s.modifiers.cmd);
    assert!(s.modifiers.shift);
}

#[test]
fn parse_mixed_separators() {
    let s = Shortcut::parse("cmd + shift + k").unwrap();
    assert_eq!(s.key, "k");
    assert!(s.modifiers.cmd);
    assert!(s.modifiers.shift);
}

#[test]
fn parse_modifier_aliases() {
    // Command aliases
    assert!(Shortcut::parse("command+k").unwrap().modifiers.cmd);
    assert!(Shortcut::parse("meta+k").unwrap().modifiers.cmd);
    assert!(Shortcut::parse("super+k").unwrap().modifiers.cmd);
    assert!(Shortcut::parse("win+k").unwrap().modifiers.cmd);
    assert!(Shortcut::parse("⌘+k").unwrap().modifiers.cmd);
    assert!(Shortcut::parse("mod+k").unwrap().modifiers.cmd);

    // Control aliases
    assert!(Shortcut::parse("control+k").unwrap().modifiers.ctrl);
    assert!(Shortcut::parse("ctl+k").unwrap().modifiers.ctrl);

    // Alt aliases
    assert!(Shortcut::parse("opt+k").unwrap().modifiers.alt);
    assert!(Shortcut::parse("option+k").unwrap().modifiers.alt);
    assert!(Shortcut::parse("⌥+k").unwrap().modifiers.alt);

    // Shift aliases
    assert!(Shortcut::parse("shft+k").unwrap().modifiers.shift);
    assert!(Shortcut::parse("⇧+k").unwrap().modifiers.shift);
}

#[test]
fn parse_arrow_keys() {
    assert_eq!(Shortcut::parse("up").unwrap().key, "up");
    assert_eq!(Shortcut::parse("arrowup").unwrap().key, "up");
    assert_eq!(Shortcut::parse("ArrowUp").unwrap().key, "up");
    assert_eq!(Shortcut::parse("down").unwrap().key, "down");
    assert_eq!(Shortcut::parse("arrowdown").unwrap().key, "down");
    assert_eq!(Shortcut::parse("left").unwrap().key, "left");
    assert_eq!(Shortcut::parse("right").unwrap().key, "right");
}

#[test]
fn parse_special_keys() {
    assert_eq!(Shortcut::parse("enter").unwrap().key, "enter");
    assert_eq!(Shortcut::parse("return").unwrap().key, "enter");
    assert_eq!(Shortcut::parse("escape").unwrap().key, "escape");
    assert_eq!(Shortcut::parse("esc").unwrap().key, "escape");
    assert_eq!(Shortcut::parse("tab").unwrap().key, "tab");
    assert_eq!(Shortcut::parse("space").unwrap().key, "space");
    assert_eq!(Shortcut::parse("backspace").unwrap().key, "backspace");
    assert_eq!(Shortcut::parse("delete").unwrap().key, "delete");
}

#[test]
fn parse_symbol_keys() {
    assert_eq!(Shortcut::parse("cmd+;").unwrap().key, "semicolon");
    assert_eq!(Shortcut::parse("cmd+semicolon").unwrap().key, "semicolon");
    assert_eq!(Shortcut::parse("cmd+/").unwrap().key, "slash");
    assert_eq!(Shortcut::parse("cmd+slash").unwrap().key, "slash");
    assert_eq!(Shortcut::parse("cmd+,").unwrap().key, "comma");
    assert_eq!(Shortcut::parse("cmd+.").unwrap().key, "period");
    assert_eq!(Shortcut::parse("cmd+[").unwrap().key, "bracketleft");
    assert_eq!(Shortcut::parse("cmd+]").unwrap().key, "bracketright");
    assert_eq!(Shortcut::parse("cmd+-").unwrap().key, "minus");
    assert_eq!(Shortcut::parse("cmd+=").unwrap().key, "equal");
}

#[test]
fn parse_function_keys() {
    for i in 1..=12 {
        let s = Shortcut::parse(&format!("f{}", i)).unwrap();
        assert_eq!(s.key, format!("f{}", i));
    }
}

#[test]
fn parse_case_insensitive() {
    let s1 = Shortcut::parse("CMD+SHIFT+K").unwrap();
    let s2 = Shortcut::parse("cmd+shift+k").unwrap();
    assert_eq!(s1, s2);
}

// ========================
// Display Tests
// ========================

#[test]
fn display_macos_simple() {
    let s = Shortcut::parse("cmd+k").unwrap();
    assert_eq!(s.display_for_platform(Platform::MacOS), "⌘K");
}

#[test]
fn display_macos_multiple_modifiers() {
    let s = Shortcut::parse("ctrl+alt+shift+cmd+k").unwrap();
    assert_eq!(s.display_for_platform(Platform::MacOS), "⌃⌥⇧⌘K");
}

#[test]
fn display_macos_special_keys() {
    assert_eq!(
        Shortcut::parse("cmd+enter")
            .unwrap()
            .display_for_platform(Platform::MacOS),
        "⌘↵"
    );
    assert_eq!(
        Shortcut::parse("escape")
            .unwrap()
            .display_for_platform(Platform::MacOS),
        "⎋"
    );
    assert_eq!(
        Shortcut::parse("shift+up")
            .unwrap()
            .display_for_platform(Platform::MacOS),
        "⇧↑"
    );
}

#[test]
fn display_windows_simple() {
    let s = Shortcut::parse("cmd+k").unwrap();
    assert_eq!(s.display_for_platform(Platform::Windows), "Super+K");
}

#[test]
fn display_windows_multiple_modifiers() {
    let s = Shortcut::parse("ctrl+alt+shift+cmd+k").unwrap();
    assert_eq!(
        s.display_for_platform(Platform::Windows),
        "Ctrl+Alt+Shift+Super+K"
    );
}

#[test]
fn display_windows_special_keys() {
    assert_eq!(
        Shortcut::parse("cmd+enter")
            .unwrap()
            .display_for_platform(Platform::Windows),
        "Super+Enter"
    );
    assert_eq!(
        Shortcut::parse("escape")
            .unwrap()
            .display_for_platform(Platform::Windows),
        "Esc"
    );
}

#[test]
fn display_normalizes_arrow_alias_key_names() {
    let s = Shortcut {
        key: "arrowup".to_string(),
        modifiers: Modifiers::shift(),
    };
    assert_eq!(s.display_for_platform(Platform::MacOS), "⇧↑");
    assert_eq!(s.display_for_platform(Platform::Windows), "Shift+Up");
}

// ========================
// Canonical String Tests
// ========================

#[test]
fn to_canonical_string_sorted() {
    let s1 = Shortcut::parse("shift+cmd+k").unwrap();
    let s2 = Shortcut::parse("cmd+shift+k").unwrap();
    assert_eq!(s1.to_canonical_string(), "cmd+shift+k");
    assert_eq!(s2.to_canonical_string(), "cmd+shift+k");

    let s3 = Shortcut::parse("shift+alt+ctrl+cmd+k").unwrap();
    assert_eq!(s3.to_canonical_string(), "alt+cmd+ctrl+shift+k");
}

#[test]
fn to_canonical_string_roundtrip() {
    let original = "cmd+shift+k";
    let s = Shortcut::parse(original).unwrap();
    let canonical = s.to_canonical_string();
    let reparsed = Shortcut::parse(&canonical).unwrap();
    assert_eq!(s, reparsed);
}

#[test]
fn to_canonical_string_normalizes_arrow_alias_key_names() {
    let s = Shortcut {
        key: "arrowdown".to_string(),
        modifiers: Modifiers::cmd(),
    };
    assert_eq!(s.to_canonical_string(), "cmd+down");
}

// ========================
// Modifiers Helper Tests
// ========================

#[test]
fn modifiers_any_and_none() {
    assert!(Modifiers::default().none());
    assert!(!Modifiers::default().any());
    assert!(Modifiers::cmd().any());
    assert!(!Modifiers::cmd().none());
    assert!(Modifiers::shift().any());
}

// ========================
// Equality Tests
// ========================

#[test]
fn shortcuts_equal_regardless_of_parse_format() {
    let s1 = Shortcut::parse("cmd+shift+k").unwrap();
    let s2 = Shortcut::parse("shift+cmd+k").unwrap();
    let s3 = Shortcut::parse("command shift k").unwrap();
    let s4 = Shortcut::parse("⌘+⇧+k").unwrap();

    assert_eq!(s1, s2);
    assert_eq!(s1, s3);
    assert_eq!(s1, s4);
}

#[test]
fn shortcuts_not_equal_with_different_modifiers() {
    let s1 = Shortcut::parse("cmd+k").unwrap();
    let s2 = Shortcut::parse("ctrl+k").unwrap();
    assert_ne!(s1, s2);
}

#[test]
fn shortcuts_not_equal_with_different_keys() {
    let s1 = Shortcut::parse("cmd+k").unwrap();
    let s2 = Shortcut::parse("cmd+j").unwrap();
    assert_ne!(s1, s2);
}
