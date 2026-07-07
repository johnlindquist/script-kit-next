use crate::theme::Theme;

use super::types::ShortcutRecorderFocusedAction;
use super::{RecordedShortcut, ShortcutRecorderColors};

#[test]
fn test_recorded_shortcut_to_display_string() {
    let mut shortcut = RecordedShortcut::new();
    shortcut.cmd = true;
    shortcut.shift = true;
    shortcut.key = Some("K".to_string());

    assert_eq!(shortcut.to_display_string(), "⌘⇧K");
}

#[test]
fn test_recorded_shortcut_to_config_string() {
    let mut shortcut = RecordedShortcut::new();
    shortcut.cmd = true;
    shortcut.shift = true;
    shortcut.key = Some("K".to_string());

    assert_eq!(shortcut.to_config_string(), "cmd+shift+k");
}

#[test]
fn test_recorded_shortcut_is_empty() {
    let shortcut = RecordedShortcut::new();
    assert!(shortcut.is_empty());

    let mut shortcut_with_mod = RecordedShortcut::new();
    shortcut_with_mod.cmd = true;
    assert!(!shortcut_with_mod.is_empty());
}

#[test]
fn test_recorded_shortcut_is_complete() {
    let mut shortcut = RecordedShortcut::new();
    shortcut.cmd = true;
    assert!(!shortcut.is_complete()); // No key yet

    shortcut.key = Some("K".to_string());
    assert!(shortcut.is_complete()); // Has modifier + key
}

#[test]
fn test_recorded_shortcut_to_keycaps() {
    let mut shortcut = RecordedShortcut::new();
    shortcut.ctrl = true;
    shortcut.alt = true;
    shortcut.shift = true;
    shortcut.cmd = true;
    shortcut.key = Some("K".to_string());

    let keycaps = shortcut.to_keycaps();
    assert_eq!(keycaps, vec!["⌃", "⌥", "⌘", "⇧", "K"]);
}

#[test]
fn test_format_key_display_special_keys() {
    assert_eq!(RecordedShortcut::format_key_display("enter"), "↵");
    assert_eq!(RecordedShortcut::format_key_display("escape"), "⎋");
    assert_eq!(RecordedShortcut::format_key_display("tab"), "⇥");
    assert_eq!(RecordedShortcut::format_key_display("backspace"), "⌫");
    assert_eq!(RecordedShortcut::format_key_display("space"), "␣");
    assert_eq!(RecordedShortcut::format_key_display("up"), "↑");
    assert_eq!(RecordedShortcut::format_key_display("arrowdown"), "↓");
}

#[test]
fn test_shortcut_recorder_colors_default() {
    let colors = ShortcutRecorderColors::default();
    assert_eq!(colors.accent, 0xfbbf24);
    assert_eq!(colors.warning, 0xf59e0b);
}

#[test]
fn test_shortcut_recorder_colors_from_theme_uses_theme_overlay_token() {
    let mut theme = Theme::default();
    theme.colors.background.main = 0x2b3c4d;

    let colors = ShortcutRecorderColors::from_theme(&theme);
    assert_eq!(colors.overlay_bg, 0x2b3c4d);
}

#[test]
fn test_shortcut_recorder_focus_cycles_forward_through_actions() {
    let mut focused = ShortcutRecorderFocusedAction::Save;

    focused = focused.next(true);
    assert_eq!(focused, ShortcutRecorderFocusedAction::Clear);

    focused = focused.next(true);
    assert_eq!(focused, ShortcutRecorderFocusedAction::Cancel);

    focused = focused.next(true);
    assert_eq!(focused, ShortcutRecorderFocusedAction::Save);
}

#[test]
fn test_shortcut_recorder_focus_cycles_backward_through_actions() {
    let mut focused = ShortcutRecorderFocusedAction::Save;

    focused = focused.previous(true);
    assert_eq!(focused, ShortcutRecorderFocusedAction::Cancel);

    focused = focused.previous(true);
    assert_eq!(focused, ShortcutRecorderFocusedAction::Clear);

    focused = focused.previous(true);
    assert_eq!(focused, ShortcutRecorderFocusedAction::Save);
}
