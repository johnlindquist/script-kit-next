//! Dialog behavior tests
//!
//! Validates random built-in action dialog behaviors end-to-end:
//! - Primary action verb propagation
//! - Action ordering and deduplication
//! - Complex ScriptInfo combos (shortcut+alias+frecency)
//! - CommandBarConfig presets
//! - Clipboard context title truncation
//! - GroupedActionItem coercion edge cases
//! - Action builder chain methods
//! - ProtocolAction constructors

use super::builders::{
    get_clipboard_history_context_actions, get_file_context_actions, get_global_actions,
    get_notes_command_bar_actions, get_path_context_actions, get_script_context_actions,
    ChatPromptInfo, ClipboardEntryInfo, NotesInfo,
};
use super::command_bar::CommandBarConfig;
use super::dialog::{build_grouped_items_static, coerce_action_selection, GroupedActionItem};
use super::types::{
    Action, ActionCategory, ActionsDialogConfig, AnchorPosition, ScriptInfo, SearchPosition,
    SectionStyle,
};
use crate::clipboard_history::ContentType;
use crate::file_search::FileInfo;
use crate::prompts::PathInfo;
use crate::protocol::ProtocolAction;

// =========================================================================
// Script context: primary action verb propagation
// =========================================================================

#[test]
fn script_primary_action_title_uses_action_verb() {
    let script = ScriptInfo::with_action_verb("Window Manager", "builtin:wm", false, "Switch to");
    let actions = get_script_context_actions(&script);
    let primary = &actions[0];
    assert_eq!(primary.id, "run_script");
    assert!(
        primary.title.starts_with("Switch to"),
        "Primary action title should start with verb, got: {}",
        primary.title
    );
    assert!(
        primary.title.contains("Window Manager"),
        "Primary action title should contain script name, got: {}",
        primary.title
    );
}

#[test]
fn script_primary_action_description_uses_action_verb() {
    let script = ScriptInfo::with_action_verb("App Launcher", "builtin:apps", false, "Launch");
    let actions = get_script_context_actions(&script);
    let primary = &actions[0];
    assert_eq!(
        primary.description,
        Some("Launch this item".to_string()),
        "Description should use the action verb"
    );
}

#[test]
fn script_primary_action_always_first() {
    // Regular script
    let script = ScriptInfo::new("my-script", "/path/to/script.ts");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].id, "run_script", "run_script must be first");
    assert_eq!(
        actions[0].shortcut,
        Some("↵".to_string()),
        "Primary should have Enter shortcut"
    );

    // Built-in
    let builtin = ScriptInfo::builtin("Clipboard History");
    let builtin_actions = get_script_context_actions(&builtin);
    assert_eq!(
        builtin_actions[0].id, "run_script",
        "run_script must be first for builtins too"
    );

    // Agent
    let mut agent = ScriptInfo::new("my-agent", "/path/agent.md");
    agent.is_agent = true;
    agent.is_script = false;
    let agent_actions = get_script_context_actions(&agent);
    assert_eq!(
        agent_actions[0].id, "run_script",
        "run_script must be first for agents"
    );
}

// =========================================================================
// Script context: both shortcut AND alias set
// =========================================================================

#[test]
fn script_with_both_shortcut_and_alias_shows_all_management_actions() {
    let script = ScriptInfo::with_shortcut_and_alias(
        "test-script",
        "/path/test.ts",
        Some("cmd+t".to_string()),
        Some("ts".to_string()),
    );
    let actions = get_script_context_actions(&script);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();

    // Should have update+remove for BOTH shortcut and alias
    assert!(
        ids.contains(&"update_shortcut"),
        "Should have update_shortcut"
    );
    assert!(
        ids.contains(&"remove_shortcut"),
        "Should have remove_shortcut"
    );
    assert!(ids.contains(&"update_alias"), "Should have update_alias");
    assert!(ids.contains(&"remove_alias"), "Should have remove_alias");

    // Should NOT have add variants
    assert!(
        !ids.contains(&"add_shortcut"),
        "Should NOT have add_shortcut"
    );
    assert!(!ids.contains(&"add_alias"), "Should NOT have add_alias");
}

#[test]
fn script_with_frecency_shortcut_alias_has_all_actions() {
    let script = ScriptInfo::with_shortcut_and_alias(
        "power-script",
        "/path/power.ts",
        Some("cmd+p".to_string()),
        Some("ps".to_string()),
    )
    .with_frecency(true, Some("/path/power.ts".to_string()));

    let actions = get_script_context_actions(&script);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();

    // All management actions
    assert!(ids.contains(&"update_shortcut"));
    assert!(ids.contains(&"remove_shortcut"));
    assert!(ids.contains(&"update_alias"));
    assert!(ids.contains(&"remove_alias"));
    assert!(ids.contains(&"reset_ranking"), "Should have reset_ranking");

    // Script-specific actions
    assert!(ids.contains(&"edit_script"));
    assert!(ids.contains(&"view_logs"));
    assert!(ids.contains(&"copy_deeplink"));
}

// =========================================================================
// Agent-specific action behaviors
// =========================================================================

#[test]
fn agent_edit_title_says_edit_agent() {
    let mut agent = ScriptInfo::new("code-reviewer", "/agents/code-reviewer.md");
    agent.is_agent = true;
    agent.is_script = false;
    let actions = get_script_context_actions(&agent);

    let edit = actions.iter().find(|a| a.id == "edit_script");
    assert!(edit.is_some(), "Agent should have edit_script action");
    assert_eq!(
        edit.unwrap().title,
        "Edit Agent",
        "Agent edit title should say 'Edit Agent'"
    );
}

#[test]
fn agent_has_no_view_logs() {
    let mut agent = ScriptInfo::new("helper", "/agents/helper.md");
    agent.is_agent = true;
    agent.is_script = false;
    let actions = get_script_context_actions(&agent);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(
        !ids.contains(&"view_logs"),
        "Agent should NOT have view_logs"
    );
}

#[test]
fn agent_has_reveal_copy_path_copy_content() {
    let mut agent = ScriptInfo::new("helper", "/agents/helper.md");
    agent.is_agent = true;
    agent.is_script = false;
    let actions = get_script_context_actions(&agent);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"reveal_in_finder"));
    assert!(ids.contains(&"copy_path"));
    assert!(ids.contains(&"copy_content"));
}

// =========================================================================
// Deeplink action is always present
// =========================================================================

#[test]
fn copy_deeplink_present_for_all_script_types() {
    // Script
    let script = ScriptInfo::new("test", "/path/test.ts");
    let ids: Vec<String> = get_script_context_actions(&script)
        .iter()
        .map(|a| a.id.clone())
        .collect();
    assert!(ids.contains(&"copy_deeplink".to_string()));

    // Builtin
    let builtin = ScriptInfo::builtin("Clipboard History");
    let ids: Vec<String> = get_script_context_actions(&builtin)
        .iter()
        .map(|a| a.id.clone())
        .collect();
    assert!(ids.contains(&"copy_deeplink".to_string()));

    // Scriptlet
    let scriptlet = ScriptInfo::scriptlet("Open URL", "/path/url.md", None, None);
    let ids: Vec<String> = get_script_context_actions(&scriptlet)
        .iter()
        .map(|a| a.id.clone())
        .collect();
    assert!(ids.contains(&"copy_deeplink".to_string()));

    // Agent
    let mut agent = ScriptInfo::new("agent", "/agents/a.md");
    agent.is_agent = true;
    let ids: Vec<String> = get_script_context_actions(&agent)
        .iter()
        .map(|a| a.id.clone())
        .collect();
    assert!(ids.contains(&"copy_deeplink".to_string()));
}

#[test]
fn copy_deeplink_description_contains_formatted_name() {
    let script = ScriptInfo::new("My Cool Script", "/path/cool.ts");
    let actions = get_script_context_actions(&script);
    let deeplink = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();

    assert!(
        deeplink
            .description
            .as_ref()
            .unwrap()
            .contains("my-cool-script"),
        "Deeplink description should contain formatted deeplink name, got: {:?}",
        deeplink.description
    );
}

// =========================================================================
// No duplicate action IDs within a context
// =========================================================================

#[test]
fn no_duplicate_ids_in_script_context() {
    let script =
        ScriptInfo::new("test", "/path/test.ts").with_frecency(true, Some("/path".to_string()));
    let actions = get_script_context_actions(&script);
    let mut ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    let total = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(
        total,
        ids.len(),
        "Script context should have no duplicate IDs"
    );
}

#[test]
fn no_duplicate_ids_in_clipboard_context() {
    let entry = ClipboardEntryInfo {
        id: "test-1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test text".to_string(),
        image_dimensions: None,
        frontmost_app_name: Some("VS Code".to_string()),
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let mut ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    let total = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(
        total,
        ids.len(),
        "Clipboard context should have no duplicate IDs"
    );
}

// =========================================================================
// CommandBarConfig::notes_style
// =========================================================================

#[test]
fn command_bar_config_notes_style() {
    let config = CommandBarConfig::notes_style();
    assert!(matches!(
        config.dialog_config.search_position,
        SearchPosition::Top
    ));
    assert!(matches!(
        config.dialog_config.section_style,
        SectionStyle::Separators
    ));
    assert!(matches!(config.dialog_config.anchor, AnchorPosition::Top));
    assert!(config.dialog_config.show_icons);
    assert!(config.dialog_config.show_footer);
    // Inherits default close behaviors
    assert!(config.close_on_select);
    assert!(config.close_on_escape);
    assert!(config.close_on_click_outside);
}

// =========================================================================
// ActionsDialogConfig defaults
// =========================================================================

#[test]
fn actions_dialog_config_default_values() {
    let config = ActionsDialogConfig::default();
    assert!(matches!(config.search_position, SearchPosition::Bottom));
    assert!(matches!(config.section_style, SectionStyle::Separators));
    assert!(matches!(config.anchor, AnchorPosition::Bottom));
    assert!(!config.show_icons);
    assert!(!config.show_footer);
}

// =========================================================================
// GroupedActionItem coercion edge cases
// =========================================================================

#[test]
fn coerce_alternating_headers_and_items() {
    // H, I, H, I, H, I pattern
    let rows = vec![
        GroupedActionItem::SectionHeader("A".to_string()),
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("B".to_string()),
        GroupedActionItem::Item(1),
        GroupedActionItem::SectionHeader("C".to_string()),
        GroupedActionItem::Item(2),
    ];
    // Landing on header at index 0 should skip to item at 1
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
    // Landing on header at index 2 should skip to item at 3
    assert_eq!(coerce_action_selection(&rows, 2), Some(3));
    // Landing on header at index 4 should skip to item at 5
    assert_eq!(coerce_action_selection(&rows, 4), Some(5));
    // Items stay as-is
    assert_eq!(coerce_action_selection(&rows, 1), Some(1));
    assert_eq!(coerce_action_selection(&rows, 3), Some(3));
    assert_eq!(coerce_action_selection(&rows, 5), Some(5));
}

#[test]
fn coerce_consecutive_headers_at_start() {
    // H, H, H, I pattern - three headers before first item
    let rows = vec![
        GroupedActionItem::SectionHeader("A".to_string()),
        GroupedActionItem::SectionHeader("B".to_string()),
        GroupedActionItem::SectionHeader("C".to_string()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(3));
    assert_eq!(coerce_action_selection(&rows, 1), Some(3));
    assert_eq!(coerce_action_selection(&rows, 2), Some(3));
}

#[test]
fn coerce_single_item() {
    let rows = vec![GroupedActionItem::Item(0)];
    assert_eq!(coerce_action_selection(&rows, 0), Some(0));
    // Beyond bounds clamps
    assert_eq!(coerce_action_selection(&rows, 100), Some(0));
}

// =========================================================================
// build_grouped_items_static with section style variations
// =========================================================================

#[test]
fn grouped_items_headers_style_inserts_headers_between_sections() {
    let actions = vec![
        Action::new("a1", "Action 1", None, ActionCategory::ScriptContext)
            .with_section("Section A"),
        Action::new("a2", "Action 2", None, ActionCategory::ScriptContext)
            .with_section("Section A"),
        Action::new("b1", "Action 3", None, ActionCategory::ScriptContext)
            .with_section("Section B"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);

    // Should be: Header("Section A"), Item(0), Item(1), Header("Section B"), Item(2)
    assert_eq!(grouped.len(), 5);
    assert!(matches!(&grouped[0], GroupedActionItem::SectionHeader(s) if s == "Section A"));
    assert!(matches!(&grouped[1], GroupedActionItem::Item(0)));
    assert!(matches!(&grouped[2], GroupedActionItem::Item(1)));
    assert!(matches!(&grouped[3], GroupedActionItem::SectionHeader(s) if s == "Section B"));
    assert!(matches!(&grouped[4], GroupedActionItem::Item(2)));
}

#[test]
fn grouped_items_separators_style_no_headers_inserted() {
    let actions = vec![
        Action::new("a1", "Action 1", None, ActionCategory::ScriptContext)
            .with_section("Section A"),
        Action::new("b1", "Action 2", None, ActionCategory::ScriptContext)
            .with_section("Section B"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);

    // No headers inserted for Separators style
    assert_eq!(grouped.len(), 2);
    assert!(matches!(&grouped[0], GroupedActionItem::Item(0)));
    assert!(matches!(&grouped[1], GroupedActionItem::Item(1)));
}

#[test]
fn grouped_items_none_style_no_headers() {
    let actions = vec![
        Action::new("a1", "Action 1", None, ActionCategory::ScriptContext)
            .with_section("Section A"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);

    assert_eq!(grouped.len(), 1);
    assert!(matches!(&grouped[0], GroupedActionItem::Item(0)));
}
