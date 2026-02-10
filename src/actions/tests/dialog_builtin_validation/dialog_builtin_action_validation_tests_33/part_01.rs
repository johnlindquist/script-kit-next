// Batch 33: Dialog built-in action validation tests
//
// 115 tests across 30 categories validating random behaviors from
// built-in action window dialogs.

use crate::actions::builders::{
    get_ai_command_bar_actions, get_clipboard_history_context_actions, get_file_context_actions,
    get_new_chat_actions, get_note_switcher_actions, get_notes_command_bar_actions,
    get_path_context_actions, get_script_context_actions,
    get_scriptlet_context_actions_with_custom, to_deeplink_name, ClipboardEntryInfo,
    NewChatModelInfo, NewChatPresetInfo, NoteSwitcherNoteInfo, NotesInfo,
};
use crate::actions::command_bar::CommandBarConfig;
use crate::actions::dialog::{build_grouped_items_static, coerce_action_selection, ActionsDialog};
use crate::actions::types::{
    Action, ActionCategory, AnchorPosition, ScriptInfo, SearchPosition, SectionStyle,
};
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::FileInfo;
use crate::prompts::PathInfo;

// =====================================================================
// 1. ActionsDialog::format_shortcut_hint: alias handling for meta/super
// =====================================================================

#[test]
fn format_shortcut_hint_meta_maps_to_cmd_symbol() {
    let result = ActionsDialog::format_shortcut_hint("meta+c");
    assert_eq!(result, "⌘C");
}

#[test]
fn format_shortcut_hint_super_maps_to_cmd_symbol() {
    let result = ActionsDialog::format_shortcut_hint("super+x");
    assert_eq!(result, "⌘X");
}

#[test]
fn format_shortcut_hint_command_alias() {
    let result = ActionsDialog::format_shortcut_hint("command+z");
    assert_eq!(result, "⌘Z");
}

#[test]
fn format_shortcut_hint_opt_maps_to_option_symbol() {
    let result = ActionsDialog::format_shortcut_hint("opt+a");
    assert_eq!(result, "⌥A");
}

// =====================================================================
// 2. ActionsDialog::format_shortcut_hint: special keys
// =====================================================================

#[test]
fn format_shortcut_hint_return_maps_to_enter_symbol() {
    let result = ActionsDialog::format_shortcut_hint("return");
    assert_eq!(result, "↵");
}

#[test]
fn format_shortcut_hint_esc_maps_to_escape_symbol() {
    let result = ActionsDialog::format_shortcut_hint("esc");
    assert_eq!(result, "⎋");
}

#[test]
fn format_shortcut_hint_tab_maps_to_tab_symbol() {
    let result = ActionsDialog::format_shortcut_hint("tab");
    assert_eq!(result, "⇥");
}

#[test]
fn format_shortcut_hint_space_maps_to_space_symbol() {
    let result = ActionsDialog::format_shortcut_hint("space");
    assert_eq!(result, "␣");
}

// =====================================================================
// 3. ActionsDialog::format_shortcut_hint: arrow key variants
// =====================================================================

#[test]
fn format_shortcut_hint_arrowup_maps_to_up_arrow() {
    let result = ActionsDialog::format_shortcut_hint("arrowup");
    assert_eq!(result, "↑");
}

#[test]
fn format_shortcut_hint_arrowdown_maps_to_down_arrow() {
    let result = ActionsDialog::format_shortcut_hint("arrowdown");
    assert_eq!(result, "↓");
}

#[test]
fn format_shortcut_hint_arrowleft_maps_to_left_arrow() {
    let result = ActionsDialog::format_shortcut_hint("arrowleft");
    assert_eq!(result, "←");
}

#[test]
fn format_shortcut_hint_arrowright_maps_to_right_arrow() {
    let result = ActionsDialog::format_shortcut_hint("arrowright");
    assert_eq!(result, "→");
}

// =====================================================================
// 4. ActionsDialog::format_shortcut_hint: combined modifier+special key
// =====================================================================

#[test]
fn format_shortcut_hint_cmd_enter() {
    let result = ActionsDialog::format_shortcut_hint("cmd+enter");
    assert_eq!(result, "⌘↵");
}

#[test]
fn format_shortcut_hint_ctrl_backspace() {
    let result = ActionsDialog::format_shortcut_hint("ctrl+backspace");
    assert_eq!(result, "⌃⌫");
}

#[test]
fn format_shortcut_hint_option_space() {
    let result = ActionsDialog::format_shortcut_hint("option+space");
    assert_eq!(result, "⌥␣");
}

#[test]
fn format_shortcut_hint_all_modifiers_plus_key() {
    let result = ActionsDialog::format_shortcut_hint("cmd+shift+ctrl+alt+k");
    assert_eq!(result, "⌘⇧⌃⌥K");
}

// =====================================================================
// 5. ActionsDialog::parse_shortcut_keycaps: multi-symbol strings
// =====================================================================

#[test]
fn parse_shortcut_keycaps_cmd_return() {
    let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘↵");
    assert_eq!(keycaps, vec!["⌘", "↵"]);
}

#[test]
fn parse_shortcut_keycaps_all_modifiers_key() {
    let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘⇧⌃⌥K");
    assert_eq!(keycaps, vec!["⌘", "⇧", "⌃", "⌥", "K"]);
}

#[test]
fn parse_shortcut_keycaps_space_symbol() {
    let keycaps = ActionsDialog::parse_shortcut_keycaps("␣");
    assert_eq!(keycaps, vec!["␣"]);
}

#[test]
fn parse_shortcut_keycaps_lowercase_uppercased() {
    let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘a");
    assert_eq!(keycaps, vec!["⌘", "A"]);
}

// =====================================================================
// 6. ActionsDialog::score_action: prefix vs contains vs fuzzy vs none
// =====================================================================

#[test]
fn score_action_prefix_match_at_least_100() {
    let action = Action::new("edit", "Edit Script", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "edit");
    assert!(score >= 100, "Prefix match should be >= 100, got {}", score);
}

#[test]
fn score_action_contains_match_between_50_and_99() {
    let action = Action::new(
        "copy",
        "Copy Edit Path",
        None,
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "edit");
    assert!(
        (50..100).contains(&score),
        "Contains match should be 50-99, got {}",
        score
    );
}

#[test]
fn score_action_no_match_returns_zero() {
    let action = Action::new("run", "Run Script", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "zzznotfound");
    assert_eq!(score, 0, "No match should return 0");
}

#[test]
fn score_action_description_bonus_adds_15() {
    let action = Action::new(
        "open",
        "Open File",
        Some("Edit the file in your editor".to_string()),
        ActionCategory::ScriptContext,
    );
    // "editor" matches description but not title
    let score = ActionsDialog::score_action(&action, "editor");
    assert!(
        score >= 15,
        "Description match should add >= 15 points, got {}",
        score
    );
}

// =====================================================================
// 7. ActionsDialog::score_action: shortcut bonus and empty search
// =====================================================================

#[test]
fn score_action_shortcut_bonus_adds_10() {
    let action =
        Action::new("test", "Test Action", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
    // Searching for "⌘" matches the shortcut_lower
    let score = ActionsDialog::score_action(&action, "⌘");
    assert!(
        score >= 10,
        "Shortcut match should add >= 10 points, got {}",
        score
    );
}

#[test]
fn score_action_empty_search_gives_prefix_match() {
    let action = Action::new("test", "Anything", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "");
    assert!(
        score >= 100,
        "Empty search should prefix-match everything, got {}",
        score
    );
}

#[test]
fn score_action_prefix_plus_description_bonus_stacks() {
    let action = Action::new(
        "edit",
        "Edit Script",
        Some("Edit the script file".to_string()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "edit");
    assert!(
        score >= 115,
        "Prefix (100) + desc bonus (15) should be >= 115, got {}",
        score
    );
}

// =====================================================================
// 8. ActionsDialog::fuzzy_match: edge cases
// =====================================================================

#[test]
fn fuzzy_match_exact_match_returns_true() {
    assert!(ActionsDialog::fuzzy_match("edit script", "edit script"));
}

#[test]
fn fuzzy_match_subsequence_returns_true() {
    assert!(ActionsDialog::fuzzy_match("edit script", "eds"));
}

#[test]
fn fuzzy_match_no_match_returns_false() {
    assert!(!ActionsDialog::fuzzy_match("edit script", "xyz"));
}

#[test]
fn fuzzy_match_empty_needle_returns_true() {
    assert!(ActionsDialog::fuzzy_match("anything", ""));
}

#[test]
fn fuzzy_match_needle_longer_returns_false() {
    assert!(!ActionsDialog::fuzzy_match("ab", "abcdef"));
}

// =====================================================================
// 9. build_grouped_items_static: Headers vs Separators behavior
// =====================================================================

#[test]
fn build_grouped_headers_style_adds_section_headers() {
    let actions = vec![
        Action::new("a", "Action A", None, ActionCategory::ScriptContext).with_section("Response"),
        Action::new("b", "Action B", None, ActionCategory::ScriptContext).with_section("Actions"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // Should have 2 headers + 2 items = 4
    assert_eq!(grouped.len(), 4, "Should have 2 headers + 2 items");
}

#[test]
fn build_grouped_separators_style_no_headers() {
    let actions = vec![
        Action::new("a", "Action A", None, ActionCategory::ScriptContext).with_section("Response"),
        Action::new("b", "Action B", None, ActionCategory::ScriptContext).with_section("Actions"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
    // Should have just 2 items, no headers
    assert_eq!(grouped.len(), 2, "Separators style should have no headers");
}

#[test]
fn build_grouped_same_section_one_header() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Same"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Same"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // 1 header + 2 items = 3
    assert_eq!(
        grouped.len(),
        3,
        "Same section should produce single header + items"
    );
}

#[test]
fn build_grouped_empty_returns_empty() {
    let actions: Vec<Action> = vec![];
    let filtered: Vec<usize> = vec![];
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    assert!(grouped.is_empty());
}

// =====================================================================
// 10. coerce_action_selection: header skipping behavior
// =====================================================================

#[test]
fn coerce_on_item_stays_put() {
    use crate::actions::dialog::GroupedActionItem;
    let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
    assert_eq!(coerce_action_selection(&rows, 0), Some(0));
}

#[test]
fn coerce_on_header_jumps_to_next_item() {
    use crate::actions::dialog::GroupedActionItem;
    let rows = vec![
        GroupedActionItem::SectionHeader("Header".to_string()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

#[test]
fn coerce_trailing_header_jumps_up() {
    use crate::actions::dialog::GroupedActionItem;
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("Header".to_string()),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(0));
}

#[test]
fn coerce_all_headers_returns_none() {
    use crate::actions::dialog::GroupedActionItem;
    let rows = vec![
        GroupedActionItem::SectionHeader("A".to_string()),
        GroupedActionItem::SectionHeader("B".to_string()),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

#[test]
fn coerce_empty_returns_none() {
    let rows: Vec<crate::actions::dialog::GroupedActionItem> = vec![];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

// =====================================================================
// 11. CommandBarConfig: preset dialog_config fields
// =====================================================================

#[test]
fn command_bar_main_menu_search_bottom() {
    let config = CommandBarConfig::main_menu_style();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Bottom);
}

#[test]
fn command_bar_ai_search_top() {
    let config = CommandBarConfig::ai_style();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Top);
}

#[test]
fn command_bar_no_search_hidden() {
    let config = CommandBarConfig::no_search();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Hidden);
}

#[test]
fn command_bar_notes_search_top() {
    let config = CommandBarConfig::notes_style();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Top);
}

// =====================================================================
// 12. CommandBarConfig: section_style and anchor presets
// =====================================================================

#[test]
fn command_bar_ai_section_headers() {
    let config = CommandBarConfig::ai_style();
    assert_eq!(config.dialog_config.section_style, SectionStyle::Headers);
}

#[test]
fn command_bar_main_menu_section_separators() {
    let config = CommandBarConfig::main_menu_style();
    assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
}

#[test]
fn command_bar_notes_section_separators() {
    let config = CommandBarConfig::notes_style();
    assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
}

#[test]
fn command_bar_ai_anchor_top() {
    let config = CommandBarConfig::ai_style();
    assert_eq!(config.dialog_config.anchor, AnchorPosition::Top);
}
