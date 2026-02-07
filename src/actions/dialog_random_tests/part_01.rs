//! Random action/dialog/window behavior validation tests
//!
//! Tests validate various edge cases and behaviors across action builders,
//! dialog helpers, window utilities, and configuration presets.

use super::builders::{
    get_ai_command_bar_actions, get_chat_context_actions, get_clipboard_history_context_actions,
    get_file_context_actions, get_new_chat_actions, get_note_switcher_actions,
    get_notes_command_bar_actions, get_path_context_actions, get_script_context_actions,
    get_scriptlet_context_actions_with_custom, to_deeplink_name, ChatModelInfo, ChatPromptInfo,
    ClipboardEntryInfo, NewChatModelInfo, NewChatPresetInfo, NoteSwitcherNoteInfo, NotesInfo,
};
use super::command_bar::CommandBarConfig;
use super::dialog::{
    build_grouped_items_static, coerce_action_selection, ActionsDialog, GroupedActionItem,
};
use super::types::{
    Action, ActionCategory, ActionsDialogConfig, AnchorPosition, ScriptInfo, SearchPosition,
    SectionStyle,
};
use super::window::{count_section_headers, WindowPosition};
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;
use crate::protocol::ProtocolAction;
use crate::scriptlets::{Scriptlet, ScriptletAction};

// =========================================================================
// Helper functions
// =========================================================================

fn action_ids(actions: &[Action]) -> Vec<&str> {
    actions.iter().map(|a| a.id.as_str()).collect()
}

fn find_action<'a>(actions: &'a [Action], id: &str) -> Option<&'a Action> {
    actions.iter().find(|a| a.id == id)
}

fn make_action(id: &str, title: &str, section: Option<&str>) -> Action {
    let mut a = Action::new(id, title, None, ActionCategory::ScriptContext);
    if let Some(s) = section {
        a = a.with_section(s);
    }
    a
}

// =========================================================================
// 1. Window count_section_headers — edge cases
// =========================================================================

#[test]
fn window_count_section_headers_empty_indices() {
    let actions = vec![make_action("a", "A", Some("S1"))];
    assert_eq!(count_section_headers(&actions, &[]), 0);
}

#[test]
fn window_count_section_headers_all_none_sections() {
    let actions = vec![
        make_action("a", "A", None),
        make_action("b", "B", None),
        make_action("c", "C", None),
    ];
    assert_eq!(count_section_headers(&actions, &[0, 1, 2]), 0);
}

#[test]
fn window_count_section_headers_single_section() {
    let actions = vec![
        make_action("a", "A", Some("S1")),
        make_action("b", "B", Some("S1")),
    ];
    // Only 1 header for "S1" (first item introduces the section)
    assert_eq!(count_section_headers(&actions, &[0, 1]), 1);
}

#[test]
fn window_count_section_headers_alternating_sections() {
    let actions = vec![
        make_action("a", "A", Some("S1")),
        make_action("b", "B", Some("S2")),
        make_action("c", "C", Some("S1")),
        make_action("d", "D", Some("S2")),
    ];
    // S1 -> S2 -> S1 -> S2 = 4 section changes
    assert_eq!(count_section_headers(&actions, &[0, 1, 2, 3]), 4);
}

#[test]
fn window_count_section_headers_none_then_some() {
    let actions = vec![
        make_action("a", "A", None),
        make_action("b", "B", Some("S1")),
        make_action("c", "C", Some("S1")),
    ];
    // None does not count; S1 introduced once
    assert_eq!(count_section_headers(&actions, &[0, 1, 2]), 1);
}

#[test]
fn window_count_section_headers_some_then_none_then_some() {
    let actions = vec![
        make_action("a", "A", Some("S1")),
        make_action("b", "B", None),
        make_action("c", "C", Some("S1")),
    ];
    // S1(count) -> None(skip) -> S1(prev was None, which is != Some("S1"), count)
    assert_eq!(count_section_headers(&actions, &[0, 1, 2]), 2);
}

#[test]
fn window_count_section_headers_filtered_subset() {
    let actions = vec![
        make_action("a", "A", Some("S1")),
        make_action("b", "B", Some("S2")),
        make_action("c", "C", Some("S3")),
    ];
    // Only show S1 and S3 — skipping S2
    assert_eq!(count_section_headers(&actions, &[0, 2]), 2);
}

#[test]
fn window_count_section_headers_out_of_bounds_index() {
    let actions = vec![make_action("a", "A", Some("S1"))];
    // Index 99 is out of bounds; .get(99) returns None so it's skipped
    assert_eq!(count_section_headers(&actions, &[0, 99]), 1);
}

// =========================================================================
// 2. WindowPosition enum
// =========================================================================

#[test]
fn window_position_default_is_bottom_right() {
    let pos = WindowPosition::default();
    assert_eq!(pos, WindowPosition::BottomRight);
}

#[test]
fn window_position_variants_are_distinct() {
    assert_ne!(WindowPosition::BottomRight, WindowPosition::TopRight);
    assert_ne!(WindowPosition::TopRight, WindowPosition::TopCenter);
    assert_ne!(WindowPosition::BottomRight, WindowPosition::TopCenter);
}

// =========================================================================
// 3. format_shortcut_hint — edge cases
// =========================================================================

#[test]
fn format_shortcut_hint_empty_string() {
    let result = ActionsDialog::format_shortcut_hint("");
    assert_eq!(result, "");
}

#[test]
fn format_shortcut_hint_single_key_no_modifier() {
    let result = ActionsDialog::format_shortcut_hint("c");
    assert_eq!(result, "C");
}

#[test]
fn format_shortcut_hint_all_modifier_aliases() {
    // command, meta, super all map to ⌘
    assert!(ActionsDialog::format_shortcut_hint("command+a").contains('⌘'));
    assert!(ActionsDialog::format_shortcut_hint("meta+a").contains('⌘'));
    assert!(ActionsDialog::format_shortcut_hint("super+a").contains('⌘'));
    // control maps to ⌃
    assert!(ActionsDialog::format_shortcut_hint("control+a").contains('⌃'));
    // opt, option map to ⌥
    assert!(ActionsDialog::format_shortcut_hint("opt+a").contains('⌥'));
    assert!(ActionsDialog::format_shortcut_hint("option+a").contains('⌥'));
}

#[test]
fn format_shortcut_hint_mixed_case_modifiers() {
    let result = ActionsDialog::format_shortcut_hint("CMD+SHIFT+c");
    assert!(result.contains('⌘'));
    assert!(result.contains('⇧'));
    assert!(result.contains('C'));
}

#[test]
fn format_shortcut_hint_special_keys_as_last_part() {
    assert_eq!(ActionsDialog::format_shortcut_hint("cmd+enter"), "⌘↵");
    assert_eq!(ActionsDialog::format_shortcut_hint("cmd+escape"), "⌘⎋");
    assert_eq!(ActionsDialog::format_shortcut_hint("cmd+tab"), "⌘⇥");
    assert_eq!(ActionsDialog::format_shortcut_hint("cmd+backspace"), "⌘⌫");
    assert_eq!(ActionsDialog::format_shortcut_hint("cmd+space"), "⌘␣");
}

#[test]
fn format_shortcut_hint_arrow_keys() {
    assert_eq!(ActionsDialog::format_shortcut_hint("cmd+up"), "⌘↑");
    assert_eq!(ActionsDialog::format_shortcut_hint("cmd+arrowdown"), "⌘↓");
    assert_eq!(ActionsDialog::format_shortcut_hint("cmd+arrowleft"), "⌘←");
    assert_eq!(ActionsDialog::format_shortcut_hint("cmd+arrowright"), "⌘→");
}

#[test]
fn format_shortcut_hint_return_and_esc_aliases() {
    assert_eq!(ActionsDialog::format_shortcut_hint("return"), "↵");
    assert_eq!(ActionsDialog::format_shortcut_hint("esc"), "⎋");
}

#[test]
fn format_shortcut_hint_whitespace_around_parts() {
    let result = ActionsDialog::format_shortcut_hint(" cmd + shift + c ");
    assert!(result.contains('⌘'));
    assert!(result.contains('⇧'));
    assert!(result.contains('C'));
}

#[test]
fn format_shortcut_hint_multi_char_key() {
    // A multi-char key like "f12" should be uppercased
    let result = ActionsDialog::format_shortcut_hint("cmd+f12");
    assert!(result.contains('⌘'));
    assert!(result.contains("F12"));
}

// =========================================================================
// 4. parse_shortcut_keycaps — edge cases
// =========================================================================

#[test]
fn parse_shortcut_keycaps_empty() {
    assert!(ActionsDialog::parse_shortcut_keycaps("").is_empty());
}

#[test]
fn parse_shortcut_keycaps_all_modifiers() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘⌃⌥⇧");
    assert_eq!(caps, vec!["⌘", "⌃", "⌥", "⇧"]);
}

#[test]
fn parse_shortcut_keycaps_arrow_symbols() {
    let caps = ActionsDialog::parse_shortcut_keycaps("↑↓←→");
    assert_eq!(caps, vec!["↑", "↓", "←", "→"]);
}

#[test]
fn parse_shortcut_keycaps_special_symbols() {
    let caps = ActionsDialog::parse_shortcut_keycaps("↵⎋⇥⌫␣");
    assert_eq!(caps, vec!["↵", "⎋", "⇥", "⌫", "␣"]);
}

#[test]
fn parse_shortcut_keycaps_lowercase_uppercased() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘a");
    assert_eq!(caps, vec!["⌘", "A"]);
}

#[test]
fn parse_shortcut_keycaps_number_key() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘1");
    assert_eq!(caps, vec!["⌘", "1"]);
}

// =========================================================================
// 5. score_action — cumulative scoring edge cases
// =========================================================================

#[test]
fn score_action_empty_search_matches_all_with_prefix() {
    let action = Action::new("test", "Run Script", None, ActionCategory::ScriptContext);
    // Empty search matches as prefix of anything
    let score = ActionsDialog::score_action(&action, "");
    assert!(
        score >= 100,
        "Empty search should be prefix match: {}",
        score
    );
}

#[test]
fn score_action_all_fields_match() {
    let action = Action::new(
        "edit_script",
        "Edit Script",
        Some("Edit the script in your editor".to_string()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘E");
    // "edit" matches title as prefix (+100), description contains "edit" (+15), shortcut doesn't contain "edit"
    let score = ActionsDialog::score_action(&action, "edit");
    assert_eq!(score, 115, "Prefix(100) + description(15) = 115");
}

#[test]
fn score_action_description_only_match() {
    let action = Action::new(
        "open_file",
        "Open File",
        Some("Launch with default application".to_string()),
        ActionCategory::ScriptContext,
    );
    // "launch" doesn't match title at all, but matches description
    let score = ActionsDialog::score_action(&action, "launch");
    assert_eq!(score, 15, "Description-only match should be 15");
}

#[test]
fn score_action_shortcut_only_match() {
    let action = Action::new("run", "Run Script", None, ActionCategory::ScriptContext)
        .with_shortcut("⌘enter");
    // "enter" doesn't match title prefix/contains/fuzzy but matches shortcut (lowercase)
    let score = ActionsDialog::score_action(&action, "enter");
    // "enter" doesn't appear in "run script", no description, but shortcut_lower = "⌘enter" contains "enter"
    assert_eq!(score, 10, "Shortcut-only match should be 10");
}

#[test]
fn score_action_fuzzy_plus_description() {
    let action = Action::new(
        "exit_dialog",
        "Exit Dialog",
        Some("Close the exit dialog panel".to_string()),
        ActionCategory::ScriptContext,
    );
    // "edg" — fuzzy matches "exit dialog" (e-x-i-t- -d-i-a-l-o-g: e...d...g not found)
    // Actually let's use "eid" which fuzzy matches: e(xit d)i(alog) — but "d" doesn't come after "i" in remaining
    // Use "xit" which fuzzy matches and title contains "xit"
    let score = ActionsDialog::score_action(&action, "xit");
    // "xit" is contained in "exit dialog" → +50 (contains), description "close the exit dialog panel" also contains "xit" → +15
    assert_eq!(score, 65, "Contains(50) + description(15) = 65");
}

#[test]
fn score_action_no_match_at_all() {
    let action = Action::new(
        "run_script",
        "Run Script",
        Some("Execute the script".to_string()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘R");
    let score = ActionsDialog::score_action(&action, "zzz");
    assert_eq!(score, 0, "No match should score 0");
}

// =========================================================================
// 6. fuzzy_match — edge cases
// =========================================================================

#[test]
fn fuzzy_match_empty_needle_returns_true() {
    assert!(ActionsDialog::fuzzy_match("anything", ""));
}

#[test]
fn fuzzy_match_empty_haystack_non_empty_needle() {
    assert!(!ActionsDialog::fuzzy_match("", "x"));
}

#[test]
fn fuzzy_match_exact_match() {
    assert!(ActionsDialog::fuzzy_match("hello", "hello"));
}

#[test]
fn fuzzy_match_subsequence() {
    assert!(ActionsDialog::fuzzy_match("hello world", "hlw"));
}

#[test]
fn fuzzy_match_needle_longer_than_haystack() {
    assert!(!ActionsDialog::fuzzy_match("ab", "abc"));
}

#[test]
fn fuzzy_match_repeated_chars_in_needle() {
    // "aab" in "abracadabra" — first 'a' at 0, second 'a' at 2, 'b' at 4
    assert!(ActionsDialog::fuzzy_match("abracadabra", "aab"));
}

#[test]
fn fuzzy_match_same_char_repeated() {
    // "aaa" in "banana" — b-a-n-a-n-a: 'a' at 1, 'a' at 3, 'a' at 5
    assert!(ActionsDialog::fuzzy_match("banana", "aaa"));
}

#[test]
fn fuzzy_match_case_sensitive() {
    // Fuzzy match is case-sensitive
    assert!(!ActionsDialog::fuzzy_match("hello", "H"));
    assert!(ActionsDialog::fuzzy_match("Hello", "H"));
}

// =========================================================================
// 7. coerce_action_selection — more patterns
// =========================================================================

#[test]
fn coerce_all_headers_returns_none() {
    let rows = vec![
        GroupedActionItem::SectionHeader("S1".to_string()),
        GroupedActionItem::SectionHeader("S2".to_string()),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

#[test]
fn coerce_header_then_item_selects_item() {
    let rows = vec![
        GroupedActionItem::SectionHeader("S1".to_string()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

#[test]
fn coerce_item_then_header_at_end() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("S1".to_string()),
    ];
    // Index 1 is header; search down finds nothing, search up finds Item at 0
    assert_eq!(coerce_action_selection(&rows, 1), Some(0));
}

#[test]
fn coerce_multiple_headers_before_item() {
    let rows = vec![
        GroupedActionItem::SectionHeader("S1".to_string()),
        GroupedActionItem::SectionHeader("S2".to_string()),
        GroupedActionItem::SectionHeader("S3".to_string()),
        GroupedActionItem::Item(0),
    ];
    // Index 0 is header; searching down finds Item at 3
    assert_eq!(coerce_action_selection(&rows, 0), Some(3));
}

#[test]
fn coerce_beyond_bounds_clamps() {
    let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
    // Index 999 clamps to len-1 = 1, which is an Item
    assert_eq!(coerce_action_selection(&rows, 999), Some(1));
}
