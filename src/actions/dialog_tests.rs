//! Tests for ActionsDialog static helper functions.
//!
//! Validates dialog-internal logic: selection coercion, grouped item building,
//! shortcut formatting, shortcut keycap parsing, fuzzy matching, and scoring.

use super::dialog::{build_grouped_items_static, coerce_action_selection, GroupedActionItem};
use super::types::{Action, ActionCategory, SectionStyle};

// ============================================================
// Helper
// ============================================================

fn make_action(id: &str, title: &str, section: Option<&str>) -> Action {
    let mut a = Action::new(id, title, None, ActionCategory::ScriptContext);
    if let Some(s) = section {
        a = a.with_section(s);
    }
    a
}

// ============================================================
// 1. coerce_action_selection
// ============================================================

#[test]
fn coerce_empty_list_returns_none() {
    let rows: Vec<GroupedActionItem> = vec![];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

#[test]
fn coerce_item_at_zero() {
    let rows = vec![GroupedActionItem::Item(0)];
    assert_eq!(coerce_action_selection(&rows, 0), Some(0));
}

#[test]
fn coerce_header_at_zero_skips_to_first_item() {
    let rows = vec![
        GroupedActionItem::SectionHeader("Section".into()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

#[test]
fn coerce_header_only_returns_none() {
    let rows = vec![GroupedActionItem::SectionHeader("Section".into())];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

#[test]
fn coerce_beyond_end_clamps_to_last_item() {
    let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
    // Index 99 should clamp to last element (index 1)
    assert_eq!(coerce_action_selection(&rows, 99), Some(1));
}

#[test]
fn coerce_header_at_end_searches_backwards() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("Section".into()),
    ];
    // Landing on the header at index 1 should search down first (nothing),
    // then search up and find Item at index 0
    assert_eq!(coerce_action_selection(&rows, 1), Some(0));
}

#[test]
fn coerce_multiple_headers_before_items() {
    let rows = vec![
        GroupedActionItem::SectionHeader("A".into()),
        GroupedActionItem::SectionHeader("B".into()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(2));
    assert_eq!(coerce_action_selection(&rows, 1), Some(2));
}

// ============================================================
// 2. build_grouped_items_static
// ============================================================

#[test]
fn grouped_items_empty_actions() {
    let actions: Vec<Action> = vec![];
    let filtered: Vec<usize> = vec![];
    let result = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    assert!(result.is_empty());
}

#[test]
fn grouped_items_no_sections_produces_items_only() {
    let actions = vec![
        make_action("a", "Alpha", None),
        make_action("b", "Beta", None),
    ];
    let filtered: Vec<usize> = vec![0, 1];
    let result = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // No sections => no headers
    assert_eq!(result.len(), 2);
    assert!(matches!(result[0], GroupedActionItem::Item(0)));
    assert!(matches!(result[1], GroupedActionItem::Item(1)));
}

#[test]
fn grouped_items_with_section_headers() {
    let actions = vec![
        make_action("a", "Alpha", Some("Group 1")),
        make_action("b", "Beta", Some("Group 1")),
        make_action("c", "Charlie", Some("Group 2")),
    ];
    let filtered: Vec<usize> = vec![0, 1, 2];
    let result = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // Group 1 header, item 0, item 1, Group 2 header, item 2
    assert_eq!(result.len(), 5);
    assert!(matches!(&result[0], GroupedActionItem::SectionHeader(s) if s == "Group 1"));
    assert!(matches!(result[1], GroupedActionItem::Item(0)));
    assert!(matches!(result[2], GroupedActionItem::Item(1)));
    assert!(matches!(&result[3], GroupedActionItem::SectionHeader(s) if s == "Group 2"));
    assert!(matches!(result[4], GroupedActionItem::Item(2)));
}

#[test]
fn grouped_items_separators_style_no_headers() {
    let actions = vec![
        make_action("a", "Alpha", Some("Group 1")),
        make_action("b", "Beta", Some("Group 2")),
    ];
    let filtered: Vec<usize> = vec![0, 1];
    let result = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
    // Separators style: no section headers inserted
    assert_eq!(result.len(), 2);
    assert!(matches!(result[0], GroupedActionItem::Item(0)));
    assert!(matches!(result[1], GroupedActionItem::Item(1)));
}

#[test]
fn grouped_items_respects_filtered_order() {
    let actions = vec![
        make_action("a", "Alpha", None),
        make_action("b", "Beta", None),
        make_action("c", "Charlie", None),
    ];
    // Reversed filter order
    let filtered: Vec<usize> = vec![2, 0, 1];
    let result = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
    assert_eq!(result.len(), 3);
    assert!(matches!(result[0], GroupedActionItem::Item(0))); // filter_idx 0 = actions[2]
    assert!(matches!(result[1], GroupedActionItem::Item(1))); // filter_idx 1 = actions[0]
    assert!(matches!(result[2], GroupedActionItem::Item(2))); // filter_idx 2 = actions[1]
}

// ============================================================
// 3. ActionsDialog::format_shortcut_hint (via dialog module)
// ============================================================

#[test]
fn format_shortcut_hint_cmd_c() {
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("cmd+c"),
        "⌘C"
    );
}

#[test]
fn format_shortcut_hint_ctrl_alt_delete() {
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("ctrl+alt+delete"),
        "⌃⌥⌫"
    );
}

#[test]
fn format_shortcut_hint_shift_enter() {
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("shift+enter"),
        "⇧↵"
    );
}

#[test]
fn format_shortcut_hint_cmd_shift_arrow() {
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("cmd+shift+up"),
        "⌘⇧↑"
    );
}

#[test]
fn format_shortcut_hint_space_and_tab() {
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("cmd+space"),
        "⌘␣"
    );
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("cmd+tab"),
        "⌘⇥"
    );
}

#[test]
fn format_shortcut_hint_all_arrow_variants() {
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("arrowup"),
        "↑"
    );
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("arrowdown"),
        "↓"
    );
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("arrowleft"),
        "←"
    );
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("arrowright"),
        "→"
    );
}

#[test]
fn format_shortcut_hint_meta_and_super() {
    // "meta" and "super" should both map to ⌘
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("meta+a"),
        "⌘A"
    );
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("super+a"),
        "⌘A"
    );
}

#[test]
fn format_shortcut_hint_option_alias() {
    // "opt" and "option" should both map to ⌥
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("opt+x"),
        "⌥X"
    );
    assert_eq!(
        super::dialog::ActionsDialog::format_shortcut_hint("option+x"),
        "⌥X"
    );
}

// ============================================================
// 4. ActionsDialog::parse_shortcut_keycaps
// ============================================================

#[test]
fn parse_keycaps_simple_symbol() {
    let keycaps = super::dialog::ActionsDialog::parse_shortcut_keycaps("⌘↵");
    assert_eq!(keycaps, vec!["⌘", "↵"]);
}

#[test]
fn parse_keycaps_letter() {
    let keycaps = super::dialog::ActionsDialog::parse_shortcut_keycaps("⌘E");
    assert_eq!(keycaps, vec!["⌘", "E"]);
}

#[test]
fn parse_keycaps_multi_modifier() {
    let keycaps = super::dialog::ActionsDialog::parse_shortcut_keycaps("⌘⇧C");
    assert_eq!(keycaps, vec!["⌘", "⇧", "C"]);
}

#[test]
fn parse_keycaps_lowercase_uppercased() {
    // Lowercase letter should be uppercased
    let keycaps = super::dialog::ActionsDialog::parse_shortcut_keycaps("⌘e");
    assert_eq!(keycaps, vec!["⌘", "E"]);
}

#[test]
fn parse_keycaps_special_keys() {
    let keycaps = super::dialog::ActionsDialog::parse_shortcut_keycaps("⎋");
    assert_eq!(keycaps, vec!["⎋"]);

    let keycaps = super::dialog::ActionsDialog::parse_shortcut_keycaps("⌫");
    assert_eq!(keycaps, vec!["⌫"]);

    let keycaps = super::dialog::ActionsDialog::parse_shortcut_keycaps("⇥");
    assert_eq!(keycaps, vec!["⇥"]);

    let keycaps = super::dialog::ActionsDialog::parse_shortcut_keycaps("␣");
    assert_eq!(keycaps, vec!["␣"]);
}

#[test]
fn parse_keycaps_arrows() {
    let keycaps = super::dialog::ActionsDialog::parse_shortcut_keycaps("↑↓←→");
    assert_eq!(keycaps, vec!["↑", "↓", "←", "→"]);
}

// ============================================================
// 5. ActionsDialog::score_action
// ============================================================

#[test]
fn score_prefix_match_highest() {
    let action = Action::new("id", "Edit Script", None, ActionCategory::ScriptContext);
    let score = super::dialog::ActionsDialog::score_action(&action, "edit");
    assert_eq!(score, 100, "Prefix match should be 100");
}

#[test]
fn score_contains_match() {
    let action = Action::new("id", "Quick Edit", None, ActionCategory::ScriptContext);
    let score = super::dialog::ActionsDialog::score_action(&action, "edit");
    assert_eq!(score, 50, "Contains match should be 50");
}

#[test]
fn score_fuzzy_match() {
    // "esc" fuzzy matches "edit script" (e...s...c)
    let action = Action::new("id", "Edit Script", None, ActionCategory::ScriptContext);
    let score = super::dialog::ActionsDialog::score_action(&action, "esc");
    assert_eq!(score, 25, "Fuzzy match should be 25");
}

#[test]
fn score_description_bonus() {
    let action = Action::new(
        "id",
        "Open File",
        Some("Edit the file in your editor".to_string()),
        ActionCategory::ScriptContext,
    );
    // "editor" doesn't match title but matches description
    let score = super::dialog::ActionsDialog::score_action(&action, "editor");
    assert_eq!(score, 15, "Description-only match should be 15");
}

#[test]
fn score_shortcut_bonus() {
    let action = Action::new("id", "Run", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
    // "⌘e" matches shortcut (lowercased)
    let score = super::dialog::ActionsDialog::score_action(&action, "⌘e");
    assert_eq!(score, 10, "Shortcut-only match should be 10");
}

#[test]
fn score_no_match() {
    let action = Action::new("id", "Edit", None, ActionCategory::ScriptContext);
    let score = super::dialog::ActionsDialog::score_action(&action, "zzz");
    assert_eq!(score, 0);
}

#[test]
fn score_combined_title_and_description() {
    let action = Action::new(
        "id",
        "Edit Script",
        Some("Edit the script file".to_string()),
        ActionCategory::ScriptContext,
    );
    // "edit" matches title as prefix (+100) and description contains (+15)
    let score = super::dialog::ActionsDialog::score_action(&action, "edit");
    assert_eq!(score, 115, "Prefix + description bonus = 115");
}

// ============================================================
// 6. ActionsDialog::fuzzy_match
// ============================================================

#[test]
fn fuzzy_match_basic() {
    assert!(super::dialog::ActionsDialog::fuzzy_match(
        "edit script",
        "esc"
    ));
    assert!(super::dialog::ActionsDialog::fuzzy_match(
        "hello world",
        "hwd"
    ));
}

#[test]
fn fuzzy_match_exact() {
    assert!(super::dialog::ActionsDialog::fuzzy_match("abc", "abc"));
}

#[test]
fn fuzzy_match_empty_needle() {
    assert!(super::dialog::ActionsDialog::fuzzy_match("anything", ""));
}

#[test]
fn fuzzy_match_no_match() {
    assert!(!super::dialog::ActionsDialog::fuzzy_match("abc", "xyz"));
}

#[test]
fn fuzzy_match_needle_longer_than_haystack() {
    assert!(!super::dialog::ActionsDialog::fuzzy_match("ab", "abc"));
}

#[test]
fn fuzzy_match_repeated_chars() {
    assert!(super::dialog::ActionsDialog::fuzzy_match("aabbc", "abc"));
}

// ============================================================
// 7. selected_action_should_close defaults
// ============================================================

#[test]
fn action_config_default_values() {
    let config = super::types::ActionsDialogConfig::default();
    assert_eq!(config.search_position, super::types::SearchPosition::Bottom);
    assert_eq!(config.section_style, SectionStyle::Separators);
    assert_eq!(config.anchor, super::types::AnchorPosition::Bottom);
    assert!(!config.show_icons);
    assert!(!config.show_footer);
}
