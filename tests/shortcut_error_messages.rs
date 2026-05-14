use script_kit_gpui::shortcuts::ShortcutParseError;

#[test]
fn test_shortcut_parse_error_messages_describe_recovery_when_input_is_invalid() {
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
