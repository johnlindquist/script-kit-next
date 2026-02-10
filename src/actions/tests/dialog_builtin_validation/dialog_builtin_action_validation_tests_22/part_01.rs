// Batch 22: Built-in action validation tests
//
// ~140 tests across 30 categories validating built-in dialog actions.

use super::builders::{
    get_ai_command_bar_actions, get_chat_context_actions, get_clipboard_history_context_actions,
    get_file_context_actions, get_new_chat_actions, get_note_switcher_actions,
    get_notes_command_bar_actions, get_path_context_actions, get_script_context_actions,
    get_scriptlet_context_actions_with_custom, ChatModelInfo, ChatPromptInfo, ClipboardEntryInfo,
    NewChatModelInfo, NewChatPresetInfo, NoteSwitcherNoteInfo, NotesInfo,
};
use super::command_bar::CommandBarConfig;
use super::dialog::{
    build_grouped_items_static, coerce_action_selection, ActionsDialog, GroupedActionItem,
};
use super::types::{
    Action, ActionCategory, ActionsDialogConfig, AnchorPosition, ScriptInfo, SearchPosition,
    SectionStyle,
};
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;

// ============================================================
// 1. format_shortcut_hint edge cases: empty, single char, trailing +
// ============================================================

#[test]
fn batch22_format_shortcut_hint_empty_string() {
    let result = ActionsDialog::format_shortcut_hint("");
    assert_eq!(result, "");
}

#[test]
fn batch22_format_shortcut_hint_single_letter() {
    // Single letter without '+' means it is both first and last part
    let result = ActionsDialog::format_shortcut_hint("c");
    assert_eq!(result, "C");
}

#[test]
fn batch22_format_shortcut_hint_return_key() {
    let result = ActionsDialog::format_shortcut_hint("return");
    assert_eq!(result, "↵");
}

#[test]
fn batch22_format_shortcut_hint_mixed_case_modifiers() {
    let result = ActionsDialog::format_shortcut_hint("Cmd+Shift+c");
    assert_eq!(result, "⌘⇧C");
}

#[test]
fn batch22_format_shortcut_hint_opt_alias() {
    let result = ActionsDialog::format_shortcut_hint("opt+s");
    assert_eq!(result, "⌥S");
}

// ============================================================
// 2. parse_shortcut_keycaps: single modifier, all modifiers, mixed
// ============================================================

#[test]
fn batch22_parse_keycaps_single_modifier() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘");
    assert_eq!(caps, vec!["⌘"]);
}

#[test]
fn batch22_parse_keycaps_all_four_modifiers() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘⌃⌥⇧");
    assert_eq!(caps, vec!["⌘", "⌃", "⌥", "⇧"]);
}

#[test]
fn batch22_parse_keycaps_lowercase_letter_uppercased() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘a");
    assert_eq!(caps, vec!["⌘", "A"]);
}

#[test]
fn batch22_parse_keycaps_number_stays() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘1");
    assert_eq!(caps, vec!["⌘", "1"]);
}

#[test]
fn batch22_parse_keycaps_empty_string() {
    let caps = ActionsDialog::parse_shortcut_keycaps("");
    assert!(caps.is_empty());
}

// ============================================================
// 3. score_action: combined bonus from all fields
// ============================================================

#[test]
fn batch22_score_prefix_plus_desc_plus_shortcut() {
    let action = Action::new(
        "id",
        "Edit Script",
        Some("Edit in editor".to_string()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘E");
    let score = ActionsDialog::score_action(&action, "edit");
    // prefix=100 + desc(edit)=15 = 115 (shortcut ⌘e doesn't contain "edit")
    assert!(score >= 115, "Expected >=115, got {}", score);
}

#[test]
fn batch22_score_no_match_zero() {
    let action = Action::new("id", "Copy Path", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "zzzzz");
    assert_eq!(score, 0);
}

#[test]
fn batch22_score_desc_only_match() {
    let action = Action::new(
        "id",
        "Open File",
        Some("Launch the editor".to_string()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "editor");
    // No title match, desc match=15
    assert_eq!(score, 15);
}

#[test]
fn batch22_score_shortcut_only_match() {
    let action =
        Action::new("id", "Run Script", None, ActionCategory::ScriptContext).with_shortcut("↵");
    let score = ActionsDialog::score_action(&action, "↵");
    // shortcut match=10
    assert!(score >= 10, "Expected >=10, got {}", score);
}

#[test]
fn batch22_score_empty_search_is_prefix() {
    let action = Action::new("id", "Anything", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "");
    // Empty string is prefix of everything
    assert!(score >= 100, "Expected >=100, got {}", score);
}

// ============================================================
// 4. fuzzy_match: Unicode, emoji, repeated chars
// ============================================================

#[test]
fn batch22_fuzzy_match_unicode_subsequence() {
    assert!(ActionsDialog::fuzzy_match("café latte", "cfl"));
}

#[test]
fn batch22_fuzzy_match_exact() {
    assert!(ActionsDialog::fuzzy_match("hello", "hello"));
}

#[test]
fn batch22_fuzzy_match_both_empty() {
    assert!(ActionsDialog::fuzzy_match("", ""));
}

#[test]
fn batch22_fuzzy_match_needle_longer() {
    assert!(!ActionsDialog::fuzzy_match("ab", "abc"));
}

#[test]
fn batch22_fuzzy_match_repeated_chars() {
    assert!(ActionsDialog::fuzzy_match("banana", "aaa"));
}

// ============================================================
// 5. to_deeplink_name: emoji stripped, single char, numeric
// ============================================================

#[test]
fn batch22_deeplink_single_char() {
    assert_eq!(super::builders::to_deeplink_name("a"), "a");
}

#[test]
fn batch22_deeplink_all_special_chars_returns_empty() {
    assert_eq!(super::builders::to_deeplink_name("@#$%^&*"), "");
}

#[test]
fn batch22_deeplink_numeric_only() {
    assert_eq!(super::builders::to_deeplink_name("42"), "42");
}

#[test]
fn batch22_deeplink_underscores_to_hyphens() {
    assert_eq!(
        super::builders::to_deeplink_name("hello_world"),
        "hello-world"
    );
}

#[test]
fn batch22_deeplink_empty_string() {
    assert_eq!(super::builders::to_deeplink_name(""), "");
}

// ============================================================
// 6. coerce_action_selection: item at end, headers at beginning
// ============================================================

#[test]
fn batch22_coerce_item_at_end_headers_at_start() {
    let rows = vec![
        GroupedActionItem::SectionHeader("A".into()),
        GroupedActionItem::SectionHeader("B".into()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(2));
}

#[test]
fn batch22_coerce_single_item() {
    let rows = vec![GroupedActionItem::Item(0)];
    assert_eq!(coerce_action_selection(&rows, 0), Some(0));
}

#[test]
fn batch22_coerce_single_header() {
    let rows = vec![GroupedActionItem::SectionHeader("H".into())];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

#[test]
fn batch22_coerce_ix_beyond_bounds_clamped() {
    let rows = vec![
        GroupedActionItem::SectionHeader("H".into()),
        GroupedActionItem::Item(0),
    ];
    // ix=999 should be clamped to len-1=1 which is an Item
    assert_eq!(coerce_action_selection(&rows, 999), Some(1));
}

#[test]
fn batch22_coerce_empty_returns_none() {
    let rows: Vec<GroupedActionItem> = vec![];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

// ============================================================
// 7. build_grouped_items_static: section transitions from None to Some
// ============================================================

#[test]
fn batch22_grouped_none_to_some_section() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Sec"),
    ];
    let filtered = vec![0, 1];
    let items = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // First action has no section → no header; second has section → header + item
    assert_eq!(items.len(), 3); // Item(0), SectionHeader("Sec"), Item(1)
}

#[test]
fn batch22_grouped_rapid_alternation() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("X"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Y"),
        Action::new("c", "C", None, ActionCategory::ScriptContext).with_section("X"),
    ];
    let filtered = vec![0, 1, 2];
    let items = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // Each section change adds a header: X, item, Y, item, X, item = 6
    assert_eq!(items.len(), 6);
}

#[test]
fn batch22_grouped_separators_no_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("X"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Y"),
    ];
    let filtered = vec![0, 1];
    let items = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
    assert_eq!(items.len(), 2); // No headers, just items
}

#[test]
fn batch22_grouped_none_style_no_headers() {
    let actions =
        vec![Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("X")];
    let filtered = vec![0];
    let items = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
    assert_eq!(items.len(), 1);
}

// ============================================================
// 8. Script context: agent description mentions "agent"
// ============================================================

#[test]
fn batch22_agent_edit_desc_mentions_agent() {
    let mut s = ScriptInfo::new("my-agent", "/p");
    s.is_agent = true;
    s.is_script = false;
    let actions = get_script_context_actions(&s);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert!(edit
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("agent"));
}

#[test]
fn batch22_agent_reveal_desc_mentions_agent() {
    let mut s = ScriptInfo::new("my-agent", "/p");
    s.is_agent = true;
    s.is_script = false;
    let actions = get_script_context_actions(&s);
    let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
    assert!(reveal
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("agent"));
}

#[test]
fn batch22_agent_copy_path_desc_mentions_agent() {
    let mut s = ScriptInfo::new("my-agent", "/p");
    s.is_agent = true;
    s.is_script = false;
    let actions = get_script_context_actions(&s);
    let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
    assert!(cp
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("agent"));
}

#[test]
fn batch22_script_edit_desc_mentions_editor() {
    let s = ScriptInfo::new("my-script", "/p");
    let actions = get_script_context_actions(&s);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert!(edit.description.as_ref().unwrap().contains("$EDITOR"));
}

// ============================================================
// 9. Script context: add/update/remove shortcut shortcuts are consistent
// ============================================================

#[test]
fn batch22_add_shortcut_has_cmd_shift_k() {
    let s = ScriptInfo::new("s", "/p");
    let actions = get_script_context_actions(&s);
    let add = actions.iter().find(|a| a.id == "add_shortcut").unwrap();
    assert_eq!(add.shortcut.as_deref(), Some("⌘⇧K"));
}

#[test]
fn batch22_update_shortcut_has_cmd_shift_k() {
    let s = ScriptInfo::with_shortcut("s", "/p", Some("cmd+x".into()));
    let actions = get_script_context_actions(&s);
    let upd = actions.iter().find(|a| a.id == "update_shortcut").unwrap();
    assert_eq!(upd.shortcut.as_deref(), Some("⌘⇧K"));
}

#[test]
fn batch22_remove_shortcut_has_cmd_opt_k() {
    let s = ScriptInfo::with_shortcut("s", "/p", Some("cmd+x".into()));
    let actions = get_script_context_actions(&s);
    let rem = actions.iter().find(|a| a.id == "remove_shortcut").unwrap();
    assert_eq!(rem.shortcut.as_deref(), Some("⌘⌥K"));
}

#[test]
fn batch22_add_alias_has_cmd_shift_a() {
    let s = ScriptInfo::new("s", "/p");
    let actions = get_script_context_actions(&s);
    let add = actions.iter().find(|a| a.id == "add_alias").unwrap();
    assert_eq!(add.shortcut.as_deref(), Some("⌘⇧A"));
}

#[test]
fn batch22_remove_alias_has_cmd_opt_a() {
    let s = ScriptInfo::with_shortcut_and_alias("s", "/p", None, Some("a".into()));
    let actions = get_script_context_actions(&s);
    let rem = actions.iter().find(|a| a.id == "remove_alias").unwrap();
    assert_eq!(rem.shortcut.as_deref(), Some("⌘⌥A"));
}

// ============================================================
// 10. Clipboard context: text vs image action count difference
// ============================================================

#[test]
fn batch22_clipboard_text_action_count() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    // Text has no OCR, no open_with, no annotate/upload cleanshot
    let text_count = actions.len();
    assert!(
        text_count >= 10,
        "Text should have >=10 actions, got {}",
        text_count
    );
}

#[test]
fn batch22_clipboard_image_has_ocr() {
    let entry = ClipboardEntryInfo {
        id: "2".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: String::new(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_ocr"));
}
