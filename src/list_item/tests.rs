// Tests for list_item module
//
// These are in a separate file because list_item.rs hits GPUI's
// macro recursion limit when #[test] attributes are added.

use crate::list_item::{
    format_shortcut_display, should_show_search_description, should_show_search_shortcut,
};

#[test]
fn test_format_shortcut_display_plus_delimited() {
    assert_eq!(format_shortcut_display("cmd+shift+k"), "⌘⇧K");
    assert_eq!(format_shortcut_display("ctrl+c"), "⌃C");
    assert_eq!(format_shortcut_display("alt+enter"), "⌥↩");
}

#[test]
fn test_format_shortcut_display_space_delimited() {
    // Script Kit metadata uses space-delimited format like "opt i"
    assert_eq!(format_shortcut_display("opt i"), "⌥I");
    assert_eq!(format_shortcut_display("cmd shift k"), "⌘⇧K");
    assert_eq!(format_shortcut_display("ctrl alt t"), "⌃⌥T");
    assert_eq!(format_shortcut_display("cmd g"), "⌘G");
    assert_eq!(format_shortcut_display("opt space"), "⌥␣");
    assert_eq!(format_shortcut_display("cmd shift m"), "⌘⇧M");
}

#[test]
fn test_format_shortcut_display_already_native() {
    assert_eq!(format_shortcut_display("⌘⇧K"), "⌘⇧K");
    assert_eq!(format_shortcut_display("⌥I"), "⌥I");
}

#[test]
fn test_format_shortcut_display_special_keys() {
    assert_eq!(format_shortcut_display("escape"), "⎋");
    assert_eq!(format_shortcut_display("tab"), "⇥");
    assert_eq!(format_shortcut_display("backspace"), "⌫");
    assert_eq!(format_shortcut_display("up"), "↑");
    assert_eq!(format_shortcut_display("down"), "↓");
}

#[test]
fn test_format_shortcut_display_mixed_format() {
    // Plus and space mixed (e.g., "cmd + shift + k")
    assert_eq!(format_shortcut_display("cmd + shift + k"), "⌘⇧K");
}

#[test]
fn test_should_show_search_shortcut_only_for_selected_or_hovered_rows() {
    assert!(!should_show_search_shortcut(
        true,  // is_filtering
        false, // selected
        false  // hovered
    ));
    assert!(should_show_search_shortcut(
        true, // is_filtering
        true, // selected
        false
    ));
    assert!(should_show_search_shortcut(
        true,  // is_filtering
        false, // selected
        true   // hovered
    ));
}

#[test]
fn test_should_show_search_description_only_when_selected_hovered_or_description_matches() {
    assert!(!should_show_search_description(
        false, // selected
        false, // hovered
        false  // has_description_match
    ));
    assert!(should_show_search_description(
        true,  // selected
        false, // hovered
        false
    ));
    assert!(should_show_search_description(
        false, // selected
        true,  // hovered
        false
    ));
    assert!(should_show_search_description(
        false, // selected
        false, // hovered
        true   // has_description_match
    ));
}
