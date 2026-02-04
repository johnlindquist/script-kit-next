//! Batch 33: Dialog built-in action validation tests
//!
//! 115 tests across 30 categories validating random behaviors from
//! built-in action window dialogs.

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
    Action, ActionCategory, ActionsDialogConfig, AnchorPosition, ScriptInfo, SearchPosition,
    SectionStyle,
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

// =====================================================================
// 13. CommandBarConfig: show_icons and show_footer presets
// =====================================================================

#[test]
fn command_bar_ai_shows_icons_and_footer() {
    let config = CommandBarConfig::ai_style();
    assert!(config.dialog_config.show_icons);
    assert!(config.dialog_config.show_footer);
}

#[test]
fn command_bar_main_menu_hides_icons_and_footer() {
    let config = CommandBarConfig::main_menu_style();
    assert!(!config.dialog_config.show_icons);
    assert!(!config.dialog_config.show_footer);
}

#[test]
fn command_bar_notes_shows_icons_and_footer() {
    let config = CommandBarConfig::notes_style();
    assert!(config.dialog_config.show_icons);
    assert!(config.dialog_config.show_footer);
}

#[test]
fn command_bar_no_search_hides_icons_and_footer() {
    let config = CommandBarConfig::no_search();
    assert!(!config.dialog_config.show_icons);
    assert!(!config.dialog_config.show_footer);
}

// =====================================================================
// 14. CommandBarConfig: close flag defaults
// =====================================================================

#[test]
fn command_bar_default_close_flags_all_true() {
    let config = CommandBarConfig::default();
    assert!(config.close_on_select);
    assert!(config.close_on_click_outside);
    assert!(config.close_on_escape);
}

#[test]
fn command_bar_ai_close_flags_inherited() {
    let config = CommandBarConfig::ai_style();
    assert!(config.close_on_select);
    assert!(config.close_on_click_outside);
    assert!(config.close_on_escape);
}

#[test]
fn command_bar_main_menu_close_flags_inherited() {
    let config = CommandBarConfig::main_menu_style();
    assert!(config.close_on_select);
    assert!(config.close_on_click_outside);
    assert!(config.close_on_escape);
}

#[test]
fn command_bar_notes_close_flags_inherited() {
    let config = CommandBarConfig::notes_style();
    assert!(config.close_on_select);
    assert!(config.close_on_click_outside);
    assert!(config.close_on_escape);
}

// =====================================================================
// 15. AI command bar: paste_image details
// =====================================================================

#[test]
fn ai_command_bar_paste_image_shortcut() {
    let actions = get_ai_command_bar_actions();
    let action = actions.iter().find(|a| a.id == "paste_image").unwrap();
    assert_eq!(action.shortcut.as_ref().unwrap(), "⌘V");
}

#[test]
fn ai_command_bar_paste_image_icon() {
    let actions = get_ai_command_bar_actions();
    let action = actions.iter().find(|a| a.id == "paste_image").unwrap();
    assert_eq!(action.icon, Some(IconName::File));
}

#[test]
fn ai_command_bar_paste_image_section() {
    let actions = get_ai_command_bar_actions();
    let action = actions.iter().find(|a| a.id == "paste_image").unwrap();
    assert_eq!(action.section.as_deref(), Some("Attachments"));
}

#[test]
fn ai_command_bar_paste_image_desc_mentions_clipboard() {
    let actions = get_ai_command_bar_actions();
    let action = actions.iter().find(|a| a.id == "paste_image").unwrap();
    assert!(action
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("clipboard"));
}

// =====================================================================
// 16. AI command bar: section distribution (count per section)
// =====================================================================

#[test]
fn ai_command_bar_response_section_has_3_actions() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Response"))
        .count();
    assert_eq!(count, 3);
}

#[test]
fn ai_command_bar_actions_section_has_4_actions() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Actions"))
        .count();
    assert_eq!(count, 4);
}

#[test]
fn ai_command_bar_attachments_section_has_2_actions() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Attachments"))
        .count();
    assert_eq!(count, 2);
}

#[test]
fn ai_command_bar_total_is_12() {
    let actions = get_ai_command_bar_actions();
    assert_eq!(actions.len(), 12);
}

// =====================================================================
// 17. to_deeplink_name: edge cases with unicode and special chars
// =====================================================================

#[test]
fn to_deeplink_name_with_parentheses_and_ampersand() {
    assert_eq!(to_deeplink_name("Copy & Paste (v2)"), "copy-paste-v2");
}

#[test]
fn to_deeplink_name_with_dots_and_slashes() {
    assert_eq!(to_deeplink_name("file.txt/path"), "file-txt-path");
}

#[test]
fn to_deeplink_name_only_special_chars() {
    assert_eq!(to_deeplink_name("!@#$%^&*()"), "");
}

#[test]
fn to_deeplink_name_already_hyphenated() {
    assert_eq!(to_deeplink_name("my-script"), "my-script");
}

// =====================================================================
// 18. Script context: exact action ordering for plain script
// =====================================================================

#[test]
fn script_context_first_action_is_run_script() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].id, "run_script");
}

#[test]
fn script_context_last_action_is_copy_deeplink_without_suggestion() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions.last().unwrap().id, "copy_deeplink");
}

#[test]
fn script_context_last_action_is_reset_ranking_with_suggestion() {
    let script = ScriptInfo::new("test", "/path/test.ts")
        .with_frecency(true, Some("/path/test.ts".to_string()));
    let actions = get_script_context_actions(&script);
    assert_eq!(actions.last().unwrap().id, "reset_ranking");
}

#[test]
fn script_context_action_count_no_shortcut_no_alias() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    // run + add_shortcut + add_alias + edit + view_logs + reveal + copy_path + copy_content + copy_deeplink = 9
    assert_eq!(actions.len(), 9);
}

// =====================================================================
// 19. Script context: agent-specific descriptions mention "agent"
// =====================================================================

#[test]
fn agent_edit_title_is_edit_agent() {
    let mut agent = ScriptInfo::new("My Agent", "/path/agent.ts");
    agent.is_script = false;
    agent.is_agent = true;
    let actions = get_script_context_actions(&agent);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert_eq!(edit.title, "Edit Agent");
}

#[test]
fn agent_edit_desc_mentions_agent_file() {
    let mut agent = ScriptInfo::new("My Agent", "/path/agent.ts");
    agent.is_script = false;
    agent.is_agent = true;
    let actions = get_script_context_actions(&agent);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert!(edit
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("agent"));
}

#[test]
fn agent_reveal_desc_mentions_agent() {
    let mut agent = ScriptInfo::new("My Agent", "/path/agent.ts");
    agent.is_script = false;
    agent.is_agent = true;
    let actions = get_script_context_actions(&agent);
    let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
    assert!(reveal
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("agent"));
}

#[test]
fn agent_has_no_view_logs() {
    let mut agent = ScriptInfo::new("My Agent", "/path/agent.ts");
    agent.is_script = false;
    agent.is_agent = true;
    let actions = get_script_context_actions(&agent);
    assert!(!actions.iter().any(|a| a.id == "view_logs"));
}

// =====================================================================
// 20. Clipboard: share shortcut and section for both text and image
// =====================================================================

#[test]
fn clipboard_share_shortcut_is_shift_cmd_e() {
    let entry = ClipboardEntryInfo {
        id: "t".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let share = actions.iter().find(|a| a.id == "clipboard_share").unwrap();
    assert_eq!(share.shortcut.as_ref().unwrap(), "⇧⌘E");
}

#[test]
fn clipboard_share_title_is_share() {
    let entry = ClipboardEntryInfo {
        id: "t".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let share = actions.iter().find(|a| a.id == "clipboard_share").unwrap();
    assert_eq!(share.title, "Share...");
}

#[test]
fn clipboard_share_present_for_image() {
    let entry = ClipboardEntryInfo {
        id: "i".to_string(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".to_string(),
        image_dimensions: Some((800, 600)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_share"));
}

#[test]
fn clipboard_share_desc_mentions_share() {
    let entry = ClipboardEntryInfo {
        id: "t".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let share = actions.iter().find(|a| a.id == "clipboard_share").unwrap();
    assert!(share
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("share"));
}

// =====================================================================
// 21. Note switcher: char count singular vs plural
// =====================================================================

#[test]
fn note_switcher_zero_chars_plural() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "id1".to_string(),
        title: "Note".to_string(),
        char_count: 0,
        is_current: false,
        is_pinned: false,
        preview: "".to_string(),
        relative_time: "".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("0 chars"));
}

#[test]
fn note_switcher_one_char_singular() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "id1".to_string(),
        title: "Note".to_string(),
        char_count: 1,
        is_current: false,
        is_pinned: false,
        preview: "".to_string(),
        relative_time: "".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("1 char"));
}

#[test]
fn note_switcher_many_chars_plural() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "id1".to_string(),
        title: "Note".to_string(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: "".to_string(),
        relative_time: "".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("42 chars"));
}

// =====================================================================
// 22. Note switcher: preview with relative time separator
// =====================================================================

#[test]
fn note_switcher_preview_with_time_has_dot_separator() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "id1".to_string(),
        title: "Note".to_string(),
        char_count: 100,
        is_current: false,
        is_pinned: false,
        preview: "Hello world".to_string(),
        relative_time: "2m ago".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(
        actions[0].description.as_deref(),
        Some("Hello world · 2m ago")
    );
}

#[test]
fn note_switcher_preview_without_time_no_separator() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "id1".to_string(),
        title: "Note".to_string(),
        char_count: 100,
        is_current: false,
        is_pinned: false,
        preview: "Hello world".to_string(),
        relative_time: "".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("Hello world"));
}

#[test]
fn note_switcher_no_preview_with_time_shows_time() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "id1".to_string(),
        title: "Note".to_string(),
        char_count: 100,
        is_current: false,
        is_pinned: false,
        preview: "".to_string(),
        relative_time: "5d ago".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("5d ago"));
}

// =====================================================================
// 23. Notes command bar: conditional action presence (selection + trash)
// =====================================================================

#[test]
fn notes_cmd_bar_no_selection_has_only_3_actions() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note + browse_notes + enable_auto_sizing = 3
    assert_eq!(actions.len(), 3);
}

#[test]
fn notes_cmd_bar_trash_view_has_3_actions() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note + browse_notes + enable_auto_sizing = 3 (trash blocks selection-dependent)
    assert_eq!(actions.len(), 3);
}

#[test]
fn notes_cmd_bar_full_mode_has_10_actions() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert_eq!(actions.len(), 10);
}

#[test]
fn notes_cmd_bar_auto_sizing_enabled_has_9_actions() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert_eq!(actions.len(), 9);
}

// =====================================================================
// 24. Path context: exact action count for file vs dir
// =====================================================================

#[test]
fn path_context_file_has_7_actions() {
    let path_info = PathInfo {
        path: "/tmp/test.txt".to_string(),
        name: "test.txt".to_string(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    assert_eq!(actions.len(), 7);
}

#[test]
fn path_context_dir_has_7_actions() {
    let path_info = PathInfo {
        path: "/tmp/mydir".to_string(),
        name: "mydir".to_string(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path_info);
    assert_eq!(actions.len(), 7);
}

#[test]
fn path_context_file_first_is_select_file() {
    let path_info = PathInfo {
        path: "/tmp/test.txt".to_string(),
        name: "test.txt".to_string(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    assert_eq!(actions[0].id, "select_file");
}

#[test]
fn path_context_dir_first_is_open_directory() {
    let path_info = PathInfo {
        path: "/tmp/mydir".to_string(),
        name: "mydir".to_string(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path_info);
    assert_eq!(actions[0].id, "open_directory");
}

// =====================================================================
// 25. File context: macOS action count for file vs dir
// =====================================================================

#[cfg(target_os = "macos")]
#[test]
fn file_context_file_macos_has_7_actions() {
    let file_info = FileInfo {
        path: "/tmp/test.txt".to_string(),
        name: "test.txt".to_string(),
        file_type: crate::file_search::FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file_info);
    // open_file + reveal + quick_look + open_with + show_info + copy_path + copy_filename = 7
    assert_eq!(actions.len(), 7);
}

#[cfg(target_os = "macos")]
#[test]
fn file_context_dir_macos_has_6_actions() {
    let file_info = FileInfo {
        path: "/tmp/mydir".to_string(),
        name: "mydir".to_string(),
        file_type: crate::file_search::FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&file_info);
    // open_directory + reveal + open_with + show_info + copy_path + copy_filename = 6 (no quick_look)
    assert_eq!(actions.len(), 6);
}

#[test]
fn file_context_file_title_quoted() {
    let file_info = FileInfo {
        path: "/tmp/doc.pdf".to_string(),
        name: "doc.pdf".to_string(),
        file_type: crate::file_search::FileType::Document,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file_info);
    assert_eq!(actions[0].title, "Open \"doc.pdf\"");
}

#[test]
fn file_context_dir_title_quoted() {
    let file_info = FileInfo {
        path: "/tmp/docs".to_string(),
        name: "docs".to_string(),
        file_type: crate::file_search::FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&file_info);
    assert_eq!(actions[0].title, "Open \"docs\"");
}

// =====================================================================
// 26. Scriptlet context with H3 custom: ordering invariant
// =====================================================================

#[test]
fn scriptlet_with_custom_run_before_custom_actions() {
    use crate::scriptlets::{Scriptlet, ScriptletAction};

    let script = ScriptInfo::scriptlet("Test", "/test.md", None, None);
    let mut scriptlet = Scriptlet::new("Test".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![ScriptletAction {
        name: "Custom".to_string(),
        command: "custom".to_string(),
        tool: "bash".to_string(),
        code: "echo custom".to_string(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];

    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let run_idx = actions.iter().position(|a| a.id == "run_script").unwrap();
    let custom_idx = actions
        .iter()
        .position(|a| a.id == "scriptlet_action:custom")
        .unwrap();
    assert!(run_idx < custom_idx);
}

#[test]
fn scriptlet_with_custom_builtins_after_custom() {
    use crate::scriptlets::{Scriptlet, ScriptletAction};

    let script = ScriptInfo::scriptlet("Test", "/test.md", None, None);
    let mut scriptlet = Scriptlet::new("Test".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![ScriptletAction {
        name: "Custom".to_string(),
        command: "custom".to_string(),
        tool: "bash".to_string(),
        code: "echo custom".to_string(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];

    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let custom_idx = actions
        .iter()
        .position(|a| a.id == "scriptlet_action:custom")
        .unwrap();
    let edit_idx = actions
        .iter()
        .position(|a| a.id == "edit_scriptlet")
        .unwrap();
    assert!(custom_idx < edit_idx);
}

#[test]
fn scriptlet_custom_action_has_action_true() {
    use crate::scriptlets::{Scriptlet, ScriptletAction};

    let script = ScriptInfo::scriptlet("Test", "/test.md", None, None);
    let mut scriptlet = Scriptlet::new("Test".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![ScriptletAction {
        name: "Custom".to_string(),
        command: "custom".to_string(),
        tool: "bash".to_string(),
        code: "echo custom".to_string(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];

    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let custom = actions
        .iter()
        .find(|a| a.id == "scriptlet_action:custom")
        .unwrap();
    assert!(custom.has_action);
}

// =====================================================================
// 27. New chat: section ordering and ID format
// =====================================================================

#[test]
fn new_chat_last_used_section_name() {
    let last_used = vec![NewChatModelInfo {
        model_id: "claude-3".to_string(),
        display_name: "Claude 3".to_string(),
        provider: "anthropic".to_string(),
        provider_display_name: "Anthropic".to_string(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
}

#[test]
fn new_chat_model_id_format() {
    let models = vec![NewChatModelInfo {
        model_id: "gpt-4".to_string(),
        display_name: "GPT-4".to_string(),
        provider: "openai".to_string(),
        provider_display_name: "OpenAI".to_string(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].id, "model_0");
}

#[test]
fn new_chat_preset_id_format() {
    let presets = vec![NewChatPresetInfo {
        id: "general".to_string(),
        name: "General".to_string(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].id, "preset_general");
}

#[test]
fn new_chat_preset_description_is_none() {
    let presets = vec![NewChatPresetInfo {
        id: "code".to_string(),
        name: "Code".to_string(),
        icon: IconName::Code,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert!(actions[0].description.is_none());
}

// =====================================================================
// 28. Action builder: with_shortcut_opt(None) vs with_shortcut_opt(Some)
// =====================================================================

#[test]
fn action_with_shortcut_opt_none_leaves_shortcut_none() {
    let action =
        Action::new("id", "Title", None, ActionCategory::ScriptContext).with_shortcut_opt(None);
    assert!(action.shortcut.is_none());
    assert!(action.shortcut_lower.is_none());
}

#[test]
fn action_with_shortcut_opt_some_sets_shortcut() {
    let action = Action::new("id", "Title", None, ActionCategory::ScriptContext)
        .with_shortcut_opt(Some("⌘K".to_string()));
    assert_eq!(action.shortcut.as_deref(), Some("⌘K"));
    assert_eq!(action.shortcut_lower.as_deref(), Some("⌘k"));
}

#[test]
fn action_with_icon_sets_icon() {
    let action =
        Action::new("id", "Title", None, ActionCategory::ScriptContext).with_icon(IconName::Copy);
    assert_eq!(action.icon, Some(IconName::Copy));
}

#[test]
fn action_with_section_sets_section() {
    let action =
        Action::new("id", "Title", None, ActionCategory::ScriptContext).with_section("Response");
    assert_eq!(action.section.as_deref(), Some("Response"));
}

// =====================================================================
// 29. Cross-context: all built-in actions have has_action=false
// =====================================================================

#[test]
fn all_script_actions_have_has_action_false() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    for action in get_script_context_actions(&script) {
        assert!(
            !action.has_action,
            "Action {} should have has_action=false",
            action.id
        );
    }
}

#[test]
fn all_clipboard_actions_have_has_action_false() {
    let entry = ClipboardEntryInfo {
        id: "t".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    for action in get_clipboard_history_context_actions(&entry) {
        assert!(
            !action.has_action,
            "Action {} should have has_action=false",
            action.id
        );
    }
}

#[test]
fn all_ai_bar_actions_have_has_action_false() {
    for action in get_ai_command_bar_actions() {
        assert!(
            !action.has_action,
            "Action {} should have has_action=false",
            action.id
        );
    }
}

#[test]
fn all_notes_actions_have_has_action_false() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    for action in get_notes_command_bar_actions(&info) {
        assert!(
            !action.has_action,
            "Action {} should have has_action=false",
            action.id
        );
    }
}

#[test]
fn all_path_actions_have_has_action_false() {
    let path_info = PathInfo {
        path: "/tmp/test.txt".to_string(),
        name: "test.txt".to_string(),
        is_dir: false,
    };
    for action in get_path_context_actions(&path_info) {
        assert!(
            !action.has_action,
            "Action {} should have has_action=false",
            action.id
        );
    }
}

#[test]
fn all_file_actions_have_has_action_false() {
    let file_info = FileInfo {
        path: "/tmp/test.txt".to_string(),
        name: "test.txt".to_string(),
        file_type: crate::file_search::FileType::File,
        is_dir: false,
    };
    for action in get_file_context_actions(&file_info) {
        assert!(
            !action.has_action,
            "Action {} should have has_action=false",
            action.id
        );
    }
}

// =====================================================================
// 30. Cross-context: all actions have non-empty title and id
// =====================================================================

#[test]
fn all_ai_bar_actions_have_nonempty_title_and_id() {
    for action in get_ai_command_bar_actions() {
        assert!(!action.id.is_empty(), "Action should have non-empty id");
        assert!(
            !action.title.is_empty(),
            "Action {} should have non-empty title",
            action.id
        );
    }
}

#[test]
fn all_notes_actions_have_nonempty_title_and_id() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    for action in get_notes_command_bar_actions(&info) {
        assert!(!action.id.is_empty());
        assert!(
            !action.title.is_empty(),
            "Action {} should have non-empty title",
            action.id
        );
    }
}

#[test]
fn all_new_chat_actions_have_nonempty_title_and_id() {
    let models = vec![NewChatModelInfo {
        model_id: "m1".to_string(),
        display_name: "Model 1".to_string(),
        provider: "p1".to_string(),
        provider_display_name: "Provider 1".to_string(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "general".to_string(),
        name: "General".to_string(),
        icon: IconName::Star,
    }];
    for action in get_new_chat_actions(&models, &presets, &models) {
        assert!(!action.id.is_empty());
        assert!(
            !action.title.is_empty(),
            "Action {} should have non-empty title",
            action.id
        );
    }
}
