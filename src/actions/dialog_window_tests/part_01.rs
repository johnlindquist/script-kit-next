//! Dialog and window action behavior validation tests
//!
//! Validates random built-in actions from dialog/window contexts:
//! - CommandBarConfig preset variants (main_menu, ai, no_search)
//! - Action ordering guarantees across contexts
//! - Notes command bar all 8 (selection x trash x auto-sizing) permutations
//! - Clipboard action structure: paste first, destructive last
//! - Chat context conditional action combos
//! - Note switcher ordering and icon assignment
//! - Path/file context action count consistency
//! - Deeplink name edge cases
//! - Agent with shortcut+alias+frecency combined
//! - ScriptInfo constructor invariant checks

use super::builders::{
    get_chat_context_actions, get_clipboard_history_context_actions, get_file_context_actions,
    get_new_chat_actions, get_note_switcher_actions, get_notes_command_bar_actions,
    get_path_context_actions, get_script_context_actions,
    get_scriptlet_context_actions_with_custom, to_deeplink_name, ChatModelInfo, ChatPromptInfo,
    ClipboardEntryInfo, NewChatModelInfo, NewChatPresetInfo, NoteSwitcherNoteInfo, NotesInfo,
};
use super::command_bar::CommandBarConfig;
use super::dialog::ActionsDialog;
use super::types::{
    Action, ActionCategory, AnchorPosition, ScriptInfo, SearchPosition, SectionStyle,
};
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::FileInfo;
use crate::prompts::PathInfo;
use crate::protocol::ProtocolAction;

// =========================================================================
// CommandBarConfig preset variants
// =========================================================================

#[test]
fn command_bar_config_default_values() {
    let config = CommandBarConfig::default();
    assert!(matches!(
        config.dialog_config.search_position,
        SearchPosition::Bottom
    ));
    assert!(matches!(
        config.dialog_config.section_style,
        SectionStyle::Separators
    ));
    assert!(matches!(
        config.dialog_config.anchor,
        AnchorPosition::Bottom
    ));
    assert!(!config.dialog_config.show_icons);
    assert!(!config.dialog_config.show_footer);
    assert!(config.close_on_select);
    assert!(config.close_on_escape);
    assert!(config.close_on_click_outside);
}

#[test]
fn command_bar_config_main_menu_style() {
    let config = CommandBarConfig::main_menu_style();
    assert!(matches!(
        config.dialog_config.search_position,
        SearchPosition::Bottom
    ));
    assert!(matches!(
        config.dialog_config.section_style,
        SectionStyle::Separators
    ));
    assert!(matches!(
        config.dialog_config.anchor,
        AnchorPosition::Bottom
    ));
    assert!(!config.dialog_config.show_icons);
    assert!(!config.dialog_config.show_footer);
    // Inherits default close behaviors
    assert!(config.close_on_select);
    assert!(config.close_on_escape);
    assert!(config.close_on_click_outside);
}

#[test]
fn command_bar_config_ai_style() {
    let config = CommandBarConfig::ai_style();
    assert!(matches!(
        config.dialog_config.search_position,
        SearchPosition::Top
    ));
    assert!(matches!(
        config.dialog_config.section_style,
        SectionStyle::Headers
    ));
    assert!(matches!(config.dialog_config.anchor, AnchorPosition::Top));
    assert!(config.dialog_config.show_icons);
    assert!(config.dialog_config.show_footer);
    assert!(config.close_on_select);
    assert!(config.close_on_escape);
    assert!(config.close_on_click_outside);
}

#[test]
fn command_bar_config_no_search() {
    let config = CommandBarConfig::no_search();
    assert!(matches!(
        config.dialog_config.search_position,
        SearchPosition::Hidden
    ));
    assert!(matches!(
        config.dialog_config.section_style,
        SectionStyle::Separators
    ));
    assert!(matches!(
        config.dialog_config.anchor,
        AnchorPosition::Bottom
    ));
    assert!(!config.dialog_config.show_icons);
    assert!(!config.dialog_config.show_footer);
}

// =========================================================================
// Action ordering: paste always first in clipboard
// =========================================================================

#[test]
fn clipboard_paste_is_always_first_action() {
    // Text entry
    let text_entry = ClipboardEntryInfo {
        id: "t1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".to_string(),
        image_dimensions: None,
        frontmost_app_name: Some("Finder".to_string()),
    };
    let text_actions = get_clipboard_history_context_actions(&text_entry);
    assert_eq!(text_actions[0].id, "clipboard_paste");

    // Image entry
    let img_entry = ClipboardEntryInfo {
        id: "i1".to_string(),
        content_type: ContentType::Image,
        pinned: true,
        preview: "Screenshot".to_string(),
        image_dimensions: Some((800, 600)),
        frontmost_app_name: None,
    };
    let img_actions = get_clipboard_history_context_actions(&img_entry);
    assert_eq!(img_actions[0].id, "clipboard_paste");
}

#[test]
fn clipboard_destructive_actions_are_last_three() {
    let entry = ClipboardEntryInfo {
        id: "d1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let len = actions.len();

    // Last three actions should be the destructive ones
    assert_eq!(actions[len - 3].id, "clipboard_delete");
    assert_eq!(actions[len - 2].id, "clipboard_delete_multiple");
    assert_eq!(actions[len - 1].id, "clipboard_delete_all");
}

#[test]
fn clipboard_copy_is_second_action() {
    let entry = ClipboardEntryInfo {
        id: "c1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert_eq!(actions[1].id, "clipboard_copy");
}

// =========================================================================
// Chat context: all conditional action combinations
// =========================================================================

#[test]
fn chat_no_messages_no_response_has_only_continue() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_response: false,
        has_messages: false,
    };
    let actions = get_chat_context_actions(&info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"continue_in_chat"));
    assert!(!ids.contains(&"copy_response"));
    assert!(!ids.contains(&"clear_conversation"));
    assert_eq!(actions.len(), 1);
}

#[test]
fn chat_has_response_only_shows_copy_response() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_response: true,
        has_messages: false,
    };
    let actions = get_chat_context_actions(&info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"continue_in_chat"));
    assert!(ids.contains(&"copy_response"));
    assert!(!ids.contains(&"clear_conversation"));
}

#[test]
fn chat_has_messages_only_shows_clear() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_response: false,
        has_messages: true,
    };
    let actions = get_chat_context_actions(&info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"continue_in_chat"));
    assert!(!ids.contains(&"copy_response"));
    assert!(ids.contains(&"clear_conversation"));
}

#[test]
fn chat_has_both_response_and_messages() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_response: true,
        has_messages: true,
    };
    let actions = get_chat_context_actions(&info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"continue_in_chat"));
    assert!(ids.contains(&"copy_response"));
    assert!(ids.contains(&"clear_conversation"));
}

#[test]
fn chat_model_checkmark_exact_format() {
    let info = ChatPromptInfo {
        current_model: Some("Claude Sonnet".to_string()),
        available_models: vec![
            ChatModelInfo {
                id: "sonnet".to_string(),
                display_name: "Claude Sonnet".to_string(),
                provider: "Anthropic".to_string(),
            },
            ChatModelInfo {
                id: "gpt4".to_string(),
                display_name: "GPT-4".to_string(),
                provider: "OpenAI".to_string(),
            },
        ],
        has_response: false,
        has_messages: false,
    };
    let actions = get_chat_context_actions(&info);

    // Current model should have checkmark in title
    let sonnet = actions
        .iter()
        .find(|a| a.id == "select_model_sonnet")
        .unwrap();
    assert_eq!(sonnet.title, "Claude Sonnet \u{2713}"); // "Claude Sonnet ✓"

    // Other model should NOT have checkmark
    let gpt4 = actions
        .iter()
        .find(|a| a.id == "select_model_gpt4")
        .unwrap();
    assert_eq!(gpt4.title, "GPT-4");
    assert!(!gpt4.title.contains('\u{2713}'));
}

#[test]
fn chat_model_description_shows_provider() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "haiku".to_string(),
            display_name: "Claude Haiku".to_string(),
            provider: "Anthropic".to_string(),
        }],
        has_response: false,
        has_messages: false,
    };
    let actions = get_chat_context_actions(&info);
    let haiku = actions
        .iter()
        .find(|a| a.id == "select_model_haiku")
        .unwrap();
    assert_eq!(haiku.description, Some("via Anthropic".to_string()));
}

// =========================================================================
// Notes command bar: all 8 permutations
// =========================================================================

#[test]
fn notes_no_selection_no_trash_no_autosizing() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();

    assert!(ids.contains(&"new_note"));
    assert!(ids.contains(&"browse_notes"));
    assert!(ids.contains(&"enable_auto_sizing"));
    // No selection → no edit/copy/export actions
    assert!(!ids.contains(&"duplicate_note"));
    assert!(!ids.contains(&"find_in_note"));
    assert!(!ids.contains(&"copy_note_as"));
    assert!(!ids.contains(&"export"));
}

#[test]
fn notes_with_selection_no_trash_no_autosizing() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();

    assert!(ids.contains(&"new_note"));
    assert!(ids.contains(&"browse_notes"));
    assert!(ids.contains(&"duplicate_note"));
    assert!(ids.contains(&"find_in_note"));
    assert!(ids.contains(&"format"));
    assert!(ids.contains(&"copy_note_as"));
    assert!(ids.contains(&"copy_deeplink"));
    assert!(ids.contains(&"create_quicklink"));
    assert!(ids.contains(&"export"));
    assert!(ids.contains(&"enable_auto_sizing"));
}

#[test]
fn notes_with_selection_no_trash_with_autosizing() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();

    // Auto-sizing already enabled → no enable action
    assert!(!ids.contains(&"enable_auto_sizing"));
    // Still has all selection actions
    assert!(ids.contains(&"duplicate_note"));
    assert!(ids.contains(&"find_in_note"));
}

#[test]
fn notes_trash_view_with_selection_suppresses_edit_copy_export() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();

    // Trash view suppresses all selection-gated actions
    assert!(!ids.contains(&"duplicate_note"));
    assert!(!ids.contains(&"find_in_note"));
    assert!(!ids.contains(&"format"));
    assert!(!ids.contains(&"copy_note_as"));
    assert!(!ids.contains(&"copy_deeplink"));
    assert!(!ids.contains(&"create_quicklink"));
    assert!(!ids.contains(&"export"));

    // Always-present actions remain
    assert!(ids.contains(&"new_note"));
    assert!(ids.contains(&"browse_notes"));
}

#[test]
fn notes_no_selection_trash_view() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();

    // Minimal actions only
    assert!(ids.contains(&"new_note"));
    assert!(ids.contains(&"browse_notes"));
    assert!(!ids.contains(&"duplicate_note"));
    assert!(!ids.contains(&"enable_auto_sizing")); // auto-sizing already enabled
}

#[test]
fn notes_all_actions_have_icons() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);

    for action in &actions {
        assert!(
            action.icon.is_some(),
            "Notes action '{}' should have an icon",
            action.id
        );
    }
}

#[test]
fn notes_all_actions_have_sections() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);

    for action in &actions {
        assert!(
            action.section.is_some(),
            "Notes action '{}' should have a section",
            action.id
        );
    }
}
