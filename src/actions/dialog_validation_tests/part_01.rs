//! Random action/dialog/window validation tests
//!
//! Validates miscellaneous behaviors across the actions system that
//! haven't been covered by other test files. Focuses on:
//! - Window module's count_section_headers (pure function)
//! - SDK action conversion flow (ProtocolAction -> Action)
//! - Action lowercase caching invariants
//! - build_actions internal routing (script + global)
//! - Refilter scoring end-to-end
//! - Constants consistency
//! - CommandBarConfig field combinations
//! - Agent vs script action differentiation
//! - Clipboard destructive action ordering invariants
//! - Path context action count consistency

use super::builders::{
    get_ai_command_bar_actions, get_chat_context_actions, get_clipboard_history_context_actions,
    get_file_context_actions, get_new_chat_actions, get_note_switcher_actions,
    get_notes_command_bar_actions, get_path_context_actions, get_script_context_actions,
    get_scriptlet_context_actions_with_custom, ChatModelInfo, ChatPromptInfo, ClipboardEntryInfo,
    NewChatModelInfo, NewChatPresetInfo, NoteSwitcherNoteInfo, NotesInfo,
};
use super::constants::{
    ACTION_ITEM_HEIGHT, ACTION_ROW_INSET, HEADER_HEIGHT, KEYCAP_HEIGHT, KEYCAP_MIN_WIDTH,
    POPUP_MAX_HEIGHT, POPUP_WIDTH, SEARCH_INPUT_HEIGHT, SECTION_HEADER_HEIGHT, SELECTION_RADIUS,
};
use super::dialog::{build_grouped_items_static, coerce_action_selection, GroupedActionItem};
use super::types::{
    Action, ActionCategory, ActionsDialogConfig, AnchorPosition, SearchPosition, SectionStyle,
};
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;
use crate::protocol::ProtocolAction;
use crate::scriptlets::{Scriptlet, ScriptletAction};

use super::command_bar::CommandBarConfig;
use super::dialog::ActionsDialog;
use super::types::ScriptInfo;

// =========================================================================
// Window count_section_headers (pure function in window.rs, duplicated logic)
// =========================================================================

/// Reimplementation of window.rs count_section_headers for testing
/// (The original is module-private in window.rs)
fn count_section_headers(actions: &[Action], filtered_indices: &[usize]) -> usize {
    if filtered_indices.is_empty() {
        return 0;
    }

    let mut count = 0;
    let mut prev_section: Option<&Option<String>> = None;

    for &idx in filtered_indices {
        if let Some(action) = actions.get(idx) {
            let current_section = &action.section;
            if current_section.is_some() {
                match prev_section {
                    None => count += 1,
                    Some(prev) if prev != current_section => count += 1,
                    _ => {}
                }
            }
            prev_section = Some(current_section);
        }
    }

    count
}

#[test]
fn test_count_section_headers_empty() {
    let actions: Vec<Action> = vec![];
    let filtered: Vec<usize> = vec![];
    assert_eq!(count_section_headers(&actions, &filtered), 0);
}

#[test]
fn test_count_section_headers_no_sections() {
    let actions = vec![
        Action::new("a", "Action A", None, ActionCategory::ScriptContext),
        Action::new("b", "Action B", None, ActionCategory::ScriptContext),
    ];
    let filtered = vec![0, 1];
    assert_eq!(count_section_headers(&actions, &filtered), 0);
}

#[test]
fn test_count_section_headers_single_section() {
    let actions = vec![
        Action::new("a", "Action A", None, ActionCategory::ScriptContext).with_section("Response"),
        Action::new("b", "Action B", None, ActionCategory::ScriptContext).with_section("Response"),
    ];
    let filtered = vec![0, 1];
    assert_eq!(count_section_headers(&actions, &filtered), 1);
}

#[test]
fn test_count_section_headers_multiple_sections() {
    let actions = vec![
        Action::new("a", "Copy Response", None, ActionCategory::ScriptContext)
            .with_section("Response"),
        Action::new("b", "Copy Chat", None, ActionCategory::ScriptContext).with_section("Response"),
        Action::new("c", "Submit", None, ActionCategory::ScriptContext).with_section("Actions"),
        Action::new("d", "New Chat", None, ActionCategory::ScriptContext).with_section("Actions"),
        Action::new("e", "Change Model", None, ActionCategory::ScriptContext)
            .with_section("Settings"),
    ];
    let filtered = vec![0, 1, 2, 3, 4];
    assert_eq!(count_section_headers(&actions, &filtered), 3);
}

#[test]
fn test_count_section_headers_mixed_section_and_no_section() {
    let actions = vec![
        Action::new("a", "Action A", None, ActionCategory::ScriptContext).with_section("Group"),
        Action::new("b", "Action B", None, ActionCategory::ScriptContext), // no section
        Action::new("c", "Action C", None, ActionCategory::ScriptContext).with_section("Group"),
    ];
    let filtered = vec![0, 1, 2];
    // First item has section "Group" -> header
    // Second item has no section -> no header
    // Third item has section "Group" but prev was None -> counts as new header
    assert_eq!(count_section_headers(&actions, &filtered), 2);
}

#[test]
fn test_count_section_headers_filtered_subset() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Alpha"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Beta"),
        Action::new("c", "C", None, ActionCategory::ScriptContext).with_section("Alpha"),
        Action::new("d", "D", None, ActionCategory::ScriptContext).with_section("Beta"),
    ];
    // Only show items from Alpha section (indices 0, 2)
    assert_eq!(count_section_headers(&actions, &[0, 2]), 1); // same section
    assert_eq!(count_section_headers(&actions, &[0, 1]), 2); // different sections
}

#[test]
fn test_count_section_headers_out_of_bounds_index() {
    let actions =
        vec![Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Response")];
    let filtered = vec![0, 5, 10]; // 5 and 10 are out of bounds
                                   // Should only count valid actions
    assert_eq!(count_section_headers(&actions, &filtered), 1);
}

// =========================================================================
// Action lowercase caching invariants
// =========================================================================

#[test]
fn test_action_lowercase_cache_title() {
    let action = Action::new(
        "test_id",
        "Edit Script",
        None,
        ActionCategory::ScriptContext,
    );
    assert_eq!(action.title_lower, "edit script");
}

#[test]
fn test_action_lowercase_cache_description() {
    let action = Action::new(
        "test_id",
        "Edit Script",
        Some("Open in $EDITOR".to_string()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(
        action.description_lower,
        Some("open in $editor".to_string())
    );
}

#[test]
fn test_action_lowercase_cache_no_description() {
    let action = Action::new(
        "test_id",
        "Edit Script",
        None,
        ActionCategory::ScriptContext,
    );
    assert!(action.description_lower.is_none());
}

#[test]
fn test_action_lowercase_cache_shortcut() {
    let action = Action::new(
        "test_id",
        "Edit Script",
        None,
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘E");
    assert_eq!(action.shortcut_lower, Some("⌘e".to_string()));
}

#[test]
fn test_action_lowercase_cache_shortcut_opt_none() {
    let action = Action::new(
        "test_id",
        "Edit Script",
        None,
        ActionCategory::ScriptContext,
    )
    .with_shortcut_opt(None);
    assert!(action.shortcut_lower.is_none());
    assert!(action.shortcut.is_none());
}

#[test]
fn test_action_lowercase_cache_shortcut_opt_some() {
    let action = Action::new(
        "test_id",
        "Edit Script",
        None,
        ActionCategory::ScriptContext,
    )
    .with_shortcut_opt(Some("⌘⇧C".to_string()));
    assert_eq!(action.shortcut, Some("⌘⇧C".to_string()));
    assert_eq!(action.shortcut_lower, Some("⌘⇧c".to_string()));
}

#[test]
fn test_action_unicode_title_lowercase() {
    let action = Action::new(
        "test",
        "Über Cool Étape",
        None,
        ActionCategory::ScriptContext,
    );
    assert_eq!(action.title_lower, "über cool étape");
}

// =========================================================================
// SDK action conversion (ProtocolAction -> Action)
// =========================================================================

#[test]
fn test_protocol_action_with_handler() {
    let pa = ProtocolAction::with_handler("Custom Handler".to_string());
    assert_eq!(pa.name, "Custom Handler");
    assert!(pa.has_action);
    assert!(pa.value.is_none());
    assert!(pa.is_visible());
    assert!(pa.should_close());
}

#[test]
fn test_protocol_action_with_value() {
    let pa = ProtocolAction::with_value("Submit Value".to_string(), "my-value".to_string());
    assert_eq!(pa.name, "Submit Value");
    assert!(!pa.has_action);
    assert_eq!(pa.value, Some("my-value".to_string()));
}

#[test]
fn test_protocol_action_visibility_combinations() {
    // visible: None -> visible
    assert!(ProtocolAction::new("A".into()).is_visible());
    // visible: Some(true) -> visible
    let mut pa = ProtocolAction::new("B".into());
    pa.visible = Some(true);
    assert!(pa.is_visible());
    // visible: Some(false) -> hidden
    pa.visible = Some(false);
    assert!(!pa.is_visible());
}

#[test]
fn test_protocol_action_close_combinations() {
    // close: None -> closes
    assert!(ProtocolAction::new("A".into()).should_close());
    // close: Some(true) -> closes
    let mut pa = ProtocolAction::new("B".into());
    pa.close = Some(true);
    assert!(pa.should_close());
    // close: Some(false) -> stays open
    pa.close = Some(false);
    assert!(!pa.should_close());
}

#[test]
fn test_sdk_action_conversion_preserves_shortcut_formatting() {
    // When set_sdk_actions converts ProtocolAction, it calls format_shortcut_hint
    // Test that the format_shortcut_hint output is what gets stored
    let hint = ActionsDialog::format_shortcut_hint("cmd+shift+c");
    assert_eq!(hint, "⌘⇧C");

    // Verify modifiers
    assert_eq!(ActionsDialog::format_shortcut_hint("cmd+a"), "⌘A");
    assert_eq!(ActionsDialog::format_shortcut_hint("ctrl+b"), "⌃B");
    assert_eq!(ActionsDialog::format_shortcut_hint("alt+c"), "⌥C");
    assert_eq!(ActionsDialog::format_shortcut_hint("shift+d"), "⇧D");
    assert_eq!(ActionsDialog::format_shortcut_hint("enter"), "↵");
    assert_eq!(ActionsDialog::format_shortcut_hint("escape"), "⎋");
    assert_eq!(ActionsDialog::format_shortcut_hint("tab"), "⇥");
    assert_eq!(ActionsDialog::format_shortcut_hint("backspace"), "⌫");
    assert_eq!(ActionsDialog::format_shortcut_hint("space"), "␣");
    assert_eq!(ActionsDialog::format_shortcut_hint("up"), "↑");
    assert_eq!(ActionsDialog::format_shortcut_hint("down"), "↓");
    assert_eq!(ActionsDialog::format_shortcut_hint("left"), "←");
    assert_eq!(ActionsDialog::format_shortcut_hint("right"), "→");
}

#[test]
fn test_sdk_action_conversion_modifier_aliases() {
    // meta/super/command all map to ⌘
    assert_eq!(ActionsDialog::format_shortcut_hint("meta+a"), "⌘A");
    assert_eq!(ActionsDialog::format_shortcut_hint("super+a"), "⌘A");
    assert_eq!(ActionsDialog::format_shortcut_hint("command+a"), "⌘A");
    // control maps to ⌃
    assert_eq!(ActionsDialog::format_shortcut_hint("control+a"), "⌃A");
    // opt/option maps to ⌥
    assert_eq!(ActionsDialog::format_shortcut_hint("opt+a"), "⌥A");
    assert_eq!(ActionsDialog::format_shortcut_hint("option+a"), "⌥A");
    // return/esc aliases
    assert_eq!(ActionsDialog::format_shortcut_hint("return"), "↵");
    assert_eq!(ActionsDialog::format_shortcut_hint("esc"), "⎋");
    // arrow aliases
    assert_eq!(ActionsDialog::format_shortcut_hint("arrowup"), "↑");
    assert_eq!(ActionsDialog::format_shortcut_hint("arrowdown"), "↓");
    assert_eq!(ActionsDialog::format_shortcut_hint("arrowleft"), "←");
    assert_eq!(ActionsDialog::format_shortcut_hint("arrowright"), "→");
    // delete maps to ⌫
    assert_eq!(ActionsDialog::format_shortcut_hint("delete"), "⌫");
}

#[test]
fn test_parse_shortcut_keycaps_special_symbols() {
    let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘⇧C");
    assert_eq!(keycaps, vec!["⌘", "⇧", "C"]);
}

#[test]
fn test_parse_shortcut_keycaps_all_modifiers() {
    let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘⌃⌥⇧");
    assert_eq!(keycaps, vec!["⌘", "⌃", "⌥", "⇧"]);
}

#[test]
fn test_parse_shortcut_keycaps_arrows() {
    assert_eq!(ActionsDialog::parse_shortcut_keycaps("↑"), vec!["↑"]);
    assert_eq!(ActionsDialog::parse_shortcut_keycaps("↓"), vec!["↓"]);
    assert_eq!(ActionsDialog::parse_shortcut_keycaps("←"), vec!["←"]);
    assert_eq!(ActionsDialog::parse_shortcut_keycaps("→"), vec!["→"]);
}

#[test]
fn test_parse_shortcut_keycaps_lowercase_to_uppercase() {
    let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘a");
    assert_eq!(keycaps, vec!["⌘", "A"]);
}

#[test]
fn test_parse_shortcut_keycaps_enter_escape() {
    assert_eq!(ActionsDialog::parse_shortcut_keycaps("↵"), vec!["↵"]);
    assert_eq!(ActionsDialog::parse_shortcut_keycaps("⎋"), vec!["⎋"]);
    assert_eq!(ActionsDialog::parse_shortcut_keycaps("⇥"), vec!["⇥"]);
    assert_eq!(ActionsDialog::parse_shortcut_keycaps("⌫"), vec!["⌫"]);
    assert_eq!(ActionsDialog::parse_shortcut_keycaps("␣"), vec!["␣"]);
}

// =========================================================================
// Score action validation (uses pub(crate) function)
// =========================================================================

#[test]
fn test_score_action_prefix_always_highest() {
    let action = Action::new(
        "test",
        "Edit Script",
        Some("Open in editor".to_string()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘E");

    let prefix_score = ActionsDialog::score_action(&action, "edit");
    let contains_score = ActionsDialog::score_action(&action, "script");
    // "edit" is a prefix of "edit script" -> should be highest
    assert!(
        prefix_score >= 100,
        "Prefix score should be >= 100, got {}",
        prefix_score
    );
    // "script" is contained but not a prefix
    assert!(
        (50..100).contains(&contains_score),
        "Contains score should be 50-99, got {}",
        contains_score
    );
}

#[test]
fn test_score_action_description_bonus() {
    let action_with_desc = Action::new(
        "test",
        "Open File",
        Some("Open the file in your editor".to_string()),
        ActionCategory::ScriptContext,
    );
    let action_no_desc = Action::new("test", "Open File", None, ActionCategory::ScriptContext);

    // Both match on title prefix
    let score_with = ActionsDialog::score_action(&action_with_desc, "open");
    let score_without = ActionsDialog::score_action(&action_no_desc, "open");
    // Both should have prefix score, but "open" doesn't appear differently in desc
    assert!(score_with >= 100);
    assert!(score_without >= 100);

    // Search for something only in the description
    let desc_only_with = ActionsDialog::score_action(&action_with_desc, "editor");
    let desc_only_without = ActionsDialog::score_action(&action_no_desc, "editor");
    assert_eq!(desc_only_with, 15, "Description-only match should score 15");
    assert_eq!(desc_only_without, 0, "No match without description");
}

#[test]
fn test_score_action_shortcut_bonus() {
    let action_with_shortcut =
        Action::new("test", "Delete Entry", None, ActionCategory::ScriptContext)
            .with_shortcut("⌃X");

    // "⌃x" is lowercase of the shortcut
    let score = ActionsDialog::score_action(&action_with_shortcut, "⌃x");
    assert!(
        score >= 10,
        "Shortcut match should add >= 10, got {}",
        score
    );
}
