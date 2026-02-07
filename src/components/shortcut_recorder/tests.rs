use std::time::Duration;

use crate::theme::Theme;

use super::types::{
    compute_overlay_appear_style, overlay_color_with_alpha, OVERLAY_ANIMATION_DURATION_MS,
    OVERLAY_BACKDROP_ALPHA, OVERLAY_BACKDROP_HOVER_ALPHA,
};
use super::{RecordedShortcut, ShortcutRecorderColors};

#[test]
fn test_recorded_shortcut_to_display_string() {
    let mut shortcut = RecordedShortcut::new();
    shortcut.cmd = true;
    shortcut.shift = true;
    shortcut.key = Some("K".to_string());

    assert_eq!(shortcut.to_display_string(), "⇧⌘K");
}

#[test]
fn test_recorded_shortcut_to_config_string() {
    let mut shortcut = RecordedShortcut::new();
    shortcut.cmd = true;
    shortcut.shift = true;
    shortcut.key = Some("K".to_string());

    assert_eq!(shortcut.to_config_string(), "shift+cmd+k");
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
    assert_eq!(keycaps, vec!["⌃", "⌥", "⇧", "⌘", "K"]);
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
fn test_compute_overlay_appear_style_starts_hidden_offset_and_transparent() {
    let style = compute_overlay_appear_style(Duration::from_millis(0));
    assert_eq!(style.backdrop_opacity, 0.0);
    assert!(style.modal_offset_y > 0.0);
    assert!(style.modal_opacity < 1.0);
    assert!(!style.complete);
}

#[test]
fn test_compute_overlay_appear_style_reaches_full_visibility_after_duration() {
    let style = compute_overlay_appear_style(Duration::from_millis(OVERLAY_ANIMATION_DURATION_MS));
    assert!((style.backdrop_opacity - 1.0).abs() < 0.001);
    assert!((style.modal_offset_y - 0.0).abs() < 0.001);
    assert!((style.modal_opacity - 1.0).abs() < 0.001);
    assert!(style.complete);
}

#[test]
fn test_overlay_color_with_alpha_applies_requested_backdrop_alphas() {
    let base = 0x123456;
    let backdrop = overlay_color_with_alpha(base, OVERLAY_BACKDROP_ALPHA);
    let backdrop_hover = overlay_color_with_alpha(base, OVERLAY_BACKDROP_HOVER_ALPHA);

    assert_eq!(backdrop, 0x12345680);
    assert_eq!(backdrop_hover, 0x12345690);
    assert!(backdrop_hover > backdrop);
}
