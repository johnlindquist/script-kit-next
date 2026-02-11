// --- merged from part_01.rs ---
// Built-in action behavioral validation tests — batch 2
//
// Validates randomly-selected built-in actions across window dialogs and
// contexts that were NOT covered in batch 1. Focuses on:
// - Action ID uniqueness across contexts
// - Conditional action presence (notes, chat, clipboard)
// - description_lower / shortcut_lower cache correctness
// - AI command bar icon & section presence
// - Clipboard content-type-specific action sets
// - Pin/unpin mutual exclusivity
// - Path & file is_dir primary action variations
// - Agent-specific action invariants
// - Note switcher edge cases (empty, singular char count, icon hierarchy)
// - Score bonuses for description and shortcut matches
// - New chat action ID format and empty section handling
// - CommandBarConfig preset dialog_config field specifics
// - Grouped items with SectionStyle::None

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
use super::types::{Action, ActionCategory, ScriptInfo, SearchPosition, SectionStyle};
use super::window::count_section_headers;
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;
use crate::scriptlets::{Scriptlet, ScriptletAction};

// =========================================================================
// Helpers
// =========================================================================

fn action_ids(actions: &[Action]) -> Vec<&str> {
    actions.iter().map(|a| a.id.as_str()).collect()
}

fn find_action<'a>(actions: &'a [Action], id: &str) -> Option<&'a Action> {
    actions.iter().find(|a| a.id == id)
}

fn has_duplicates(ids: &[&str]) -> Option<String> {
    let mut seen = std::collections::HashSet::new();
    for id in ids {
        if !seen.insert(id) {
            return Some(id.to_string());
        }
    }
    None
}

// =========================================================================
// 1. Action ID uniqueness within each context
// =========================================================================

#[test]
fn script_context_action_ids_are_unique() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let ids = action_ids(&actions);
    assert!(
        has_duplicates(&ids).is_none(),
        "Duplicate action ID in script context: {:?}",
        has_duplicates(&ids)
    );
}

#[test]
fn file_context_action_ids_are_unique() {
    let file = FileInfo {
        path: "/test.txt".into(),
        name: "test.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    let ids = action_ids(&actions);
    assert!(
        has_duplicates(&ids).is_none(),
        "Duplicate action ID in file context: {:?}",
        has_duplicates(&ids)
    );
}

#[test]
fn clipboard_context_action_ids_are_unique() {
    let entry = ClipboardEntryInfo {
        id: "e1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions);
    assert!(
        has_duplicates(&ids).is_none(),
        "Duplicate action ID in clipboard context: {:?}",
        has_duplicates(&ids)
    );
}

#[test]
fn path_context_action_ids_are_unique() {
    let path = PathInfo::new("test", "/test", false);
    let actions = get_path_context_actions(&path);
    let ids = action_ids(&actions);
    assert!(
        has_duplicates(&ids).is_none(),
        "Duplicate action ID in path context: {:?}",
        has_duplicates(&ids)
    );
}

#[test]
fn ai_command_bar_action_ids_are_unique() {
    let actions = get_ai_command_bar_actions();
    let ids = action_ids(&actions);
    assert!(
        has_duplicates(&ids).is_none(),
        "Duplicate action ID in AI command bar: {:?}",
        has_duplicates(&ids)
    );
}

#[test]
fn notes_command_bar_action_ids_are_unique() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ids = action_ids(&actions);
    assert!(
        has_duplicates(&ids).is_none(),
        "Duplicate action ID in notes command bar: {:?}",
        has_duplicates(&ids)
    );
}

#[test]
fn chat_context_action_ids_are_unique() {
    let info = ChatPromptInfo {
        current_model: Some("A".into()),
        available_models: vec![
            ChatModelInfo {
                id: "a".into(),
                display_name: "A".into(),
                provider: "PA".into(),
            },
            ChatModelInfo {
                id: "b".into(),
                display_name: "B".into(),
                provider: "PB".into(),
            },
        ],
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    let ids = action_ids(&actions);
    assert!(
        has_duplicates(&ids).is_none(),
        "Duplicate action ID in chat context: {:?}",
        has_duplicates(&ids)
    );
}

// =========================================================================
// 2. All actions have non-empty title and ID
// =========================================================================

#[test]
fn all_script_actions_have_nonempty_title_and_id() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    for action in &get_script_context_actions(&script) {
        assert!(!action.id.is_empty(), "Action has empty ID");
        assert!(
            !action.title.is_empty(),
            "Action '{}' has empty title",
            action.id
        );
    }
}

#[test]
fn all_ai_actions_have_nonempty_title_and_id() {
    for action in &get_ai_command_bar_actions() {
        assert!(!action.id.is_empty(), "Action has empty ID");
        assert!(
            !action.title.is_empty(),
            "Action '{}' has empty title",
            action.id
        );
    }
}

// =========================================================================
// 3. AI command bar — every action has an icon
// =========================================================================

#[test]
fn ai_command_bar_all_actions_have_icons() {
    let actions = get_ai_command_bar_actions();
    for action in &actions {
        assert!(
            action.icon.is_some(),
            "AI command bar action '{}' should have an icon",
            action.id
        );
    }
}

// =========================================================================
// 4. AI command bar — every action has a section
// =========================================================================

#[test]
fn ai_command_bar_all_actions_have_sections() {
    let actions = get_ai_command_bar_actions();
    for action in &actions {
        assert!(
            action.section.is_some(),
            "AI command bar action '{}' should have a section",
            action.id
        );
    }
}

// =========================================================================
// 5. description_lower matches description across contexts
// =========================================================================

#[test]
fn description_lower_matches_description_for_script_actions() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    for action in &get_script_context_actions(&script) {
        match (&action.description, &action.description_lower) {
            (Some(desc), Some(lower)) => {
                assert_eq!(
                    lower,
                    &desc.to_lowercase(),
                    "description_lower mismatch for script action '{}'",
                    action.id
                );
            }
            (None, None) => {} // Both absent, fine
            (Some(_), None) => {
                panic!(
                    "Action '{}' has description but no description_lower",
                    action.id
                );
            }
            (None, Some(_)) => {
                panic!(
                    "Action '{}' has description_lower but no description",
                    action.id
                );
            }
        }
    }
}

#[test]
fn description_lower_matches_description_for_ai_actions() {
    for action in &get_ai_command_bar_actions() {
        match (&action.description, &action.description_lower) {
            (Some(desc), Some(lower)) => {
                assert_eq!(
                    lower,
                    &desc.to_lowercase(),
                    "description_lower mismatch for AI action '{}'",
                    action.id
                );
            }
            (None, None) => {}
            (Some(_), None) => {
                panic!(
                    "AI action '{}' has description but no description_lower",
                    action.id
                );
            }
            (None, Some(_)) => {
                panic!(
                    "AI action '{}' has description_lower but no description",
                    action.id
                );
            }
        }
    }
}

#[test]
fn description_lower_matches_description_for_path_actions() {
    let path = PathInfo::new("test", "/test", false);
    for action in &get_path_context_actions(&path) {
        match (&action.description, &action.description_lower) {
            (Some(desc), Some(lower)) => {
                assert_eq!(
                    lower,
                    &desc.to_lowercase(),
                    "description_lower mismatch for path action '{}'",
                    action.id
                );
            }
            (None, None) => {}
            _ => panic!(
                "Action '{}' has mismatched description/description_lower presence",
                action.id
            ),
        }
    }
}

// =========================================================================
// 6. shortcut_lower matches shortcut across contexts
// =========================================================================

#[test]
fn shortcut_lower_matches_shortcut_for_script_actions() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    for action in &get_script_context_actions(&script) {
        match (&action.shortcut, &action.shortcut_lower) {
            (Some(sc), Some(lower)) => {
                assert_eq!(
                    lower,
                    &sc.to_lowercase(),
                    "shortcut_lower mismatch for script action '{}'",
                    action.id
                );
            }
            (None, None) => {}
            (Some(_), None) => {
                panic!("Action '{}' has shortcut but no shortcut_lower", action.id);
            }
            (None, Some(_)) => {
                panic!("Action '{}' has shortcut_lower but no shortcut", action.id);
            }
        }
    }
}

#[test]
fn shortcut_lower_matches_shortcut_for_clipboard_actions() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    for action in &get_clipboard_history_context_actions(&entry) {
        match (&action.shortcut, &action.shortcut_lower) {
            (Some(sc), Some(lower)) => {
                assert_eq!(
                    lower,
                    &sc.to_lowercase(),
                    "shortcut_lower mismatch for clipboard action '{}'",
                    action.id
                );
            }
            (None, None) => {}
            _ => panic!(
                "Action '{}' has mismatched shortcut/shortcut_lower presence",
                action.id
            ),
        }
    }
}

// =========================================================================
// 7. Notes conditional actions — selection + non-trash required
// =========================================================================

#[test]
fn notes_duplicate_only_when_selected_and_not_trash() {
    // has_selection=true, is_trash_view=false → duplicate present
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions_1 = get_notes_command_bar_actions(&info);
    let ids = action_ids(&actions_1);
    assert!(
        ids.contains(&"duplicate_note"),
        "Should have duplicate_note with selection + non-trash"
    );

    // has_selection=false → no duplicate
    let info_no_sel = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions_2 = get_notes_command_bar_actions(&info_no_sel);
    let ids_no_sel = action_ids(&actions_2);
    assert!(
        !ids_no_sel.contains(&"duplicate_note"),
        "Should NOT have duplicate_note without selection"
    );

    // is_trash_view=true → no duplicate
    let info_trash = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions_3 = get_notes_command_bar_actions(&info_trash);
    let ids_trash = action_ids(&actions_3);
    assert!(
        !ids_trash.contains(&"duplicate_note"),
        "Should NOT have duplicate_note in trash view"
    );
}

#[test]
fn notes_find_in_note_only_when_selected_and_not_trash() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions_1 = get_notes_command_bar_actions(&info);
    let ids = action_ids(&actions_1);
    assert!(ids.contains(&"find_in_note"));

    let info_no_sel = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions_2 = get_notes_command_bar_actions(&info_no_sel);
    let ids_no_sel = action_ids(&actions_2);
    assert!(!ids_no_sel.contains(&"find_in_note"));
}

// --- merged from part_02.rs ---

#[test]
fn notes_format_only_when_selected_and_not_trash() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions_1 = get_notes_command_bar_actions(&info);
    let ids = action_ids(&actions_1);
    assert!(ids.contains(&"format"));

    let info_trash = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions_2 = get_notes_command_bar_actions(&info_trash);
    let ids_trash = action_ids(&actions_2);
    assert!(!ids_trash.contains(&"format"));
}

#[test]
fn notes_copy_section_only_when_selected_and_not_trash() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions_1 = get_notes_command_bar_actions(&info);
    let ids = action_ids(&actions_1);
    assert!(ids.contains(&"copy_note_as"));
    assert!(ids.contains(&"copy_deeplink"));
    assert!(ids.contains(&"create_quicklink"));

    let info_no_sel = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions_2 = get_notes_command_bar_actions(&info_no_sel);
    let ids_no_sel = action_ids(&actions_2);
    assert!(!ids_no_sel.contains(&"copy_note_as"));
    assert!(!ids_no_sel.contains(&"copy_deeplink"));
    assert!(!ids_no_sel.contains(&"create_quicklink"));
}

#[test]
fn notes_export_only_when_selected_and_not_trash() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions_1 = get_notes_command_bar_actions(&info);
    let ids = action_ids(&actions_1);
    assert!(ids.contains(&"export"));

    let info_no_sel = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions_2 = get_notes_command_bar_actions(&info_no_sel);
    let ids_no_sel = action_ids(&actions_2);
    assert!(!ids_no_sel.contains(&"export"));
}

// =========================================================================
// 8. Notes auto-sizing toggle — only when disabled
// =========================================================================

#[test]
fn notes_auto_sizing_only_when_disabled() {
    let info_disabled = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions_1 = get_notes_command_bar_actions(&info_disabled);
    let ids_disabled = action_ids(&actions_1);
    assert!(
        ids_disabled.contains(&"enable_auto_sizing"),
        "Should show enable_auto_sizing when disabled"
    );

    let info_enabled = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions_2 = get_notes_command_bar_actions(&info_enabled);
    let ids_enabled = action_ids(&actions_2);
    assert!(
        !ids_enabled.contains(&"enable_auto_sizing"),
        "Should NOT show enable_auto_sizing when already enabled"
    );
}

// =========================================================================
// 9. Chat conditional actions — copy_response / clear_conversation
// =========================================================================

#[test]
fn chat_copy_response_only_when_has_response() {
    let with_response = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions_1 = get_chat_context_actions(&with_response);
    let ids = action_ids(&actions_1);
    assert!(ids.contains(&"copy_response"));

    let without_response = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions_2 = get_chat_context_actions(&without_response);
    let ids_no = action_ids(&actions_2);
    assert!(!ids_no.contains(&"copy_response"));
}

#[test]
fn chat_clear_conversation_only_when_has_messages() {
    let with_messages = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions_1 = get_chat_context_actions(&with_messages);
    let ids = action_ids(&actions_1);
    assert!(ids.contains(&"clear_conversation"));

    let without_messages = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions_2 = get_chat_context_actions(&without_messages);
    let ids_no = action_ids(&actions_2);
    assert!(!ids_no.contains(&"clear_conversation"));
}

#[test]
fn chat_empty_models_still_has_continue_in_chat() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(
        actions.len(),
        1,
        "Empty chat should have only continue_in_chat"
    );
    assert_eq!(actions[0].id, "continue_in_chat");
}

#[test]
fn chat_full_context_has_all_actions() {
    let info = ChatPromptInfo {
        current_model: Some("GPT-4".into()),
        available_models: vec![ChatModelInfo {
            id: "gpt4".into(),
            display_name: "GPT-4".into(),
            provider: "OpenAI".into(),
        }],
        has_messages: true,
        has_response: true,
    };
    let actions_tmp = get_chat_context_actions(&info);
    let ids = action_ids(&actions_tmp);
    assert!(ids.contains(&"select_model_gpt4"));
    assert!(ids.contains(&"continue_in_chat"));
    assert!(ids.contains(&"copy_response"));
    assert!(ids.contains(&"clear_conversation"));
}

// =========================================================================
// 10. Clipboard content-type-specific actions
// =========================================================================

#[test]
fn clipboard_image_has_ocr_action() {
    let entry = ClipboardEntryInfo {
        id: "img1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions_tmp = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions_tmp);
    assert!(
        ids.contains(&"clipboard_ocr"),
        "Image entry should have OCR action"
    );
}

#[test]
fn clipboard_text_has_no_ocr_action() {
    let entry = ClipboardEntryInfo {
        id: "txt1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions_tmp = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions_tmp);
    assert!(
        !ids.contains(&"clipboard_ocr"),
        "Text entry should NOT have OCR action"
    );
}

#[test]
fn clipboard_image_has_more_actions_than_text() {
    let text_entry = ClipboardEntryInfo {
        id: "t".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let image_entry = ClipboardEntryInfo {
        id: "i".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "".into(),
        image_dimensions: Some((10, 10)),
        frontmost_app_name: None,
    };
    let text_count = get_clipboard_history_context_actions(&text_entry).len();
    let image_count = get_clipboard_history_context_actions(&image_entry).len();
    assert!(
        image_count > text_count,
        "Image ({}) should have more actions than text ({})",
        image_count,
        text_count
    );
}

// =========================================================================
// 11. Clipboard pin/unpin mutual exclusivity
// =========================================================================

#[test]
fn clipboard_pinned_has_unpin_not_pin() {
    let entry = ClipboardEntryInfo {
        id: "p1".into(),
        content_type: ContentType::Text,
        pinned: true,
        preview: "pinned".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions_tmp = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions_tmp);
    assert!(
        ids.contains(&"clipboard_unpin"),
        "Pinned entry should have unpin"
    );
    assert!(
        !ids.contains(&"clipboard_pin"),
        "Pinned entry should NOT have pin"
    );
}

#[test]
fn clipboard_unpinned_has_pin_not_unpin() {
    let entry = ClipboardEntryInfo {
        id: "u1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "unpinned".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions_tmp = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions_tmp);
    assert!(
        ids.contains(&"clipboard_pin"),
        "Unpinned entry should have pin"
    );
    assert!(
        !ids.contains(&"clipboard_unpin"),
        "Unpinned entry should NOT have unpin"
    );
}

// =========================================================================
// 12. Clipboard frontmost app name in paste title
// =========================================================================

#[test]
fn clipboard_paste_title_includes_app_name() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: Some("VS Code".into()),
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = find_action(&actions, "clipboard_paste").unwrap();
    assert!(
        paste.title.contains("VS Code"),
        "Paste title should include app name, got '{}'",
        paste.title
    );
}

#[test]
fn clipboard_paste_title_fallback_when_no_app() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = find_action(&actions, "clipboard_paste").unwrap();
    assert!(
        paste.title.contains("Active App"),
        "Paste title should say 'Active App' as fallback, got '{}'",
        paste.title
    );
}

// =========================================================================
// 13. Path context — is_dir differences
// =========================================================================

#[test]
fn path_directory_primary_is_open_directory() {
    let path = PathInfo::new("my-dir", "/my-dir", true);
    let actions = get_path_context_actions(&path);
    assert_eq!(actions[0].id, "open_directory");
    assert!(actions[0].title.contains("my-dir"));
}

#[test]
fn path_file_primary_is_select_file() {
    let path = PathInfo::new("file.txt", "/file.txt", false);
    let actions = get_path_context_actions(&path);
    assert_eq!(actions[0].id, "select_file");
    assert!(actions[0].title.contains("file.txt"));
}

#[test]
fn path_trash_description_varies_by_is_dir() {
    let dir_path = PathInfo::new("dir", "/dir", true);
    let dir_actions = get_path_context_actions(&dir_path);
    let dir_trash = find_action(&dir_actions, "move_to_trash").unwrap();
    assert!(
        dir_trash.description.as_ref().unwrap().contains("folder"),
        "Directory trash description should mention 'folder', got '{:?}'",
        dir_trash.description
    );

    let file_path = PathInfo::new("file.txt", "/file.txt", false);
    let file_actions = get_path_context_actions(&file_path);
    let file_trash = find_action(&file_actions, "move_to_trash").unwrap();
    assert!(
        file_trash.description.as_ref().unwrap().contains("file"),
        "File trash description should mention 'file', got '{:?}'",
        file_trash.description
    );
}

// =========================================================================
// 14. File context — is_dir differences
// =========================================================================

#[test]
fn file_directory_primary_is_open_directory() {
    let dir = FileInfo {
        path: "/my-dir".into(),
        name: "my-dir".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&dir);
    assert_eq!(actions[0].id, "open_directory");
}

#[test]
fn file_non_directory_primary_is_open_file() {
    let file = FileInfo {
        path: "/test.rs".into(),
        name: "test.rs".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    assert_eq!(actions[0].id, "open_file");
}

#[test]
fn file_directory_has_no_quick_look() {
    let dir = FileInfo {
        path: "/my-dir".into(),
        name: "my-dir".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions_tmp = get_file_context_actions(&dir);
    let ids = action_ids(&actions_tmp);
    assert!(
        !ids.contains(&"quick_look"),
        "Directories should not have quick_look"
    );
}

// =========================================================================
// 15. Agent-specific action invariants
// =========================================================================

#[test]
fn agent_has_edit_with_agent_title() {
    let mut agent = ScriptInfo::new("my-agent", "/path/agent.md");
    agent.is_agent = true;
    agent.is_script = false;
    let actions = get_script_context_actions(&agent);
    let edit = find_action(&actions, "edit_script").unwrap();
    assert!(
        edit.title.contains("Agent"),
        "Agent edit action should say 'Agent', got '{}'",
        edit.title
    );
}

// --- merged from part_03.rs ---

#[test]
fn agent_has_reveal_and_copy_path() {
    let mut agent = ScriptInfo::new("my-agent", "/path/agent.md");
    agent.is_agent = true;
    agent.is_script = false;
    let actions_tmp = get_script_context_actions(&agent);
    let ids = action_ids(&actions_tmp);
    assert!(ids.contains(&"reveal_in_finder"));
    assert!(ids.contains(&"copy_path"));
    assert!(ids.contains(&"copy_content"));
}

#[test]
fn agent_lacks_view_logs() {
    let mut agent = ScriptInfo::new("my-agent", "/path/agent.md");
    agent.is_agent = true;
    agent.is_script = false;
    let actions_tmp = get_script_context_actions(&agent);
    let ids = action_ids(&actions_tmp);
    assert!(
        !ids.contains(&"view_logs"),
        "Agent should not have view_logs"
    );
}

// =========================================================================
// 16. Builtin lacks file-specific actions
// =========================================================================

#[test]
fn builtin_lacks_edit_view_logs_reveal_copy_path_copy_content() {
    let builtin = ScriptInfo::builtin("Clipboard History");
    let actions_tmp = get_script_context_actions(&builtin);
    let ids = action_ids(&actions_tmp);
    assert!(!ids.contains(&"edit_script"));
    assert!(!ids.contains(&"view_logs"));
    assert!(!ids.contains(&"reveal_in_finder"));
    assert!(!ids.contains(&"copy_path"));
    assert!(!ids.contains(&"copy_content"));
}

#[test]
fn builtin_has_run_shortcut_alias_deeplink() {
    let builtin = ScriptInfo::builtin("Clipboard History");
    let actions_tmp = get_script_context_actions(&builtin);
    let ids = action_ids(&actions_tmp);
    assert!(ids.contains(&"run_script"));
    assert!(ids.contains(&"add_shortcut"));
    assert!(ids.contains(&"add_alias"));
    assert!(ids.contains(&"copy_deeplink"));
}

// =========================================================================
// 17. Note switcher edge cases
// =========================================================================

#[test]
fn note_switcher_empty_shows_no_notes_placeholder() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "no_notes");
    assert!(actions[0].title.contains("No notes"));
    assert_eq!(actions[0].icon, Some(IconName::Plus));
}

#[test]
fn note_switcher_singular_character_count() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "one".into(),
        title: "One Char".into(),
        char_count: 1,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(
        actions[0].description.as_ref().unwrap().contains("1 char"),
        "Singular should be '1 char', got '{:?}'",
        actions[0].description
    );
    assert!(
        !actions[0].description.as_ref().unwrap().contains("chars"),
        "Singular should NOT contain 'chars'"
    );
}

#[test]
fn note_switcher_plural_character_count() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "many".into(),
        title: "Many Chars".into(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(
        actions[0]
            .description
            .as_ref()
            .unwrap()
            .contains("42 chars"),
        "Plural should be '42 chars', got '{:?}'",
        actions[0].description
    );
}

#[test]
fn note_switcher_zero_characters_plural() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "empty".into(),
        title: "Empty Note".into(),
        char_count: 0,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(
        actions[0].description.as_ref().unwrap().contains("0 chars"),
        "Zero should be '0 chars', got '{:?}'",
        actions[0].description
    );
}

#[test]
fn note_switcher_icon_hierarchy_pinned_over_current() {
    // Pinned + current = StarFilled (pinned wins)
    let notes = vec![NoteSwitcherNoteInfo {
        id: "both".into(),
        title: "Both".into(),
        char_count: 5,
        is_current: true,
        is_pinned: true,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

#[test]
fn note_switcher_icon_current_only() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "cur".into(),
        title: "Current".into(),
        char_count: 5,
        is_current: true,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::Check));
}

#[test]
fn note_switcher_icon_default() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "plain".into(),
        title: "Plain".into(),
        char_count: 5,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::File));
}

#[test]
fn note_switcher_current_has_bullet_prefix() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "cur".into(),
            title: "Current Note".into(),
            char_count: 5,
            is_current: true,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "other".into(),
            title: "Other Note".into(),
            char_count: 3,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
    ];
    let actions = get_note_switcher_actions(&notes);
    assert!(
        actions[0].title.starts_with("• "),
        "Current note should have '• ' prefix, got '{}'",
        actions[0].title
    );
    assert!(
        !actions[1].title.starts_with("• "),
        "Non-current note should NOT have '• ' prefix"
    );
}

#[test]
fn note_switcher_all_have_notes_section() {
    let notes: Vec<NoteSwitcherNoteInfo> = (0..5)
        .map(|i| NoteSwitcherNoteInfo {
            id: format!("n{}", i),
            title: format!("Note {}", i),
            char_count: i * 10,
            is_current: i == 0,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        })
        .collect();
    for action in &get_note_switcher_actions(&notes) {
        assert!(
            action.section.as_deref() == Some("Recent")
                || action.section.as_deref() == Some("Pinned"),
            "Note switcher action '{}' should be in 'Recent' or 'Pinned' section, got {:?}",
            action.id,
            action.section
        );
    }
}

// =========================================================================
// 18. Score bonuses for description and shortcut matches
// =========================================================================

#[test]
fn score_description_only_match_returns_nonzero() {
    // Title doesn't match, but description contains the query
    let action = Action::new(
        "test",
        "Something Unrelated",
        Some("Opens the editor for you".into()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "editor");
    assert!(
        score > 0,
        "Description-only match should return nonzero score, got {}",
        score
    );
}

#[test]
fn score_shortcut_only_match_returns_nonzero() {
    let action = Action::new(
        "test",
        "Something Unrelated",
        None,
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘E");
    let score = ActionsDialog::score_action(&action, "⌘e");
    assert!(
        score > 0,
        "Shortcut-only match should return nonzero score, got {}",
        score
    );
}

#[test]
fn score_no_match_returns_zero() {
    let action = Action::new(
        "test",
        "Edit Script",
        Some("Open in editor".into()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘E");
    let score = ActionsDialog::score_action(&action, "zzzznotfound");
    assert_eq!(score, 0, "No match should return 0");
}

#[test]
fn score_title_plus_description_bonus_stacks() {
    let action = Action::new(
        "edit",
        "Edit Script",
        Some("Edit the script file".into()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "edit");
    // Should get prefix bonus (100) + description bonus (15) = 115
    assert!(
        score > 100,
        "Title + description match should stack bonuses, got {}",
        score
    );
}

// =========================================================================
// 19. New chat action ID format and empty sections
// =========================================================================

#[test]
fn new_chat_last_used_ids_are_indexed() {
    let last = vec![
        NewChatModelInfo {
            model_id: "a".into(),
            display_name: "A".into(),
            provider: "p".into(),
            provider_display_name: "P".into(),
        },
        NewChatModelInfo {
            model_id: "b".into(),
            display_name: "B".into(),
            provider: "p".into(),
            provider_display_name: "P".into(),
        },
    ];
    let actions = get_new_chat_actions(&last, &[], &[]);
    assert_eq!(actions[0].id, "last_used_0");
    assert_eq!(actions[1].id, "last_used_1");
}

#[test]
fn new_chat_preset_ids_use_preset_id() {
    let presets = vec![NewChatPresetInfo {
        id: "code-review".into(),
        name: "Code Review".into(),
        icon: IconName::Code,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].id, "preset_code-review");
}

#[test]
fn new_chat_model_ids_are_indexed() {
    let models = vec![
        NewChatModelInfo {
            model_id: "claude".into(),
            display_name: "Claude".into(),
            provider: "anthropic".into(),
            provider_display_name: "Anthropic".into(),
        },
        NewChatModelInfo {
            model_id: "gpt4".into(),
            display_name: "GPT-4".into(),
            provider: "openai".into(),
            provider_display_name: "OpenAI".into(),
        },
    ];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].id, "model_0");
    assert_eq!(actions[1].id, "model_1");
}

#[test]
fn new_chat_empty_all_sections_returns_empty() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert!(
        actions.is_empty(),
        "All empty sections should return empty actions"
    );
}

#[test]
fn new_chat_model_descriptions_have_provider() {
    let models = vec![NewChatModelInfo {
        model_id: "claude".into(),
        display_name: "Claude".into(),
        provider: "anthropic".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(
        actions[0].description.as_deref(),
        Some("Anthropic"),
        "Model description should be provider_display_name"
    );
}

#[test]
fn new_chat_last_used_descriptions_have_provider() {
    let last = vec![NewChatModelInfo {
        model_id: "gpt4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&last, &[], &[]);
    assert_eq!(
        actions[0].description.as_deref(),
        Some("OpenAI"),
        "Last used description should be provider_display_name"
    );
}

#[test]
fn new_chat_presets_have_no_description() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert!(
        actions[0].description.is_none(),
        "Presets should have no description"
    );
}

// =========================================================================
// 20. Global actions always empty
// =========================================================================

#[test]
fn global_actions_always_returns_empty() {
    use super::builders::get_global_actions;
    let actions = get_global_actions();
    assert!(actions.is_empty(), "Global actions should always be empty");
}

// =========================================================================
// 21. Deeplink name edge cases
// =========================================================================

#[test]
fn deeplink_name_multiple_spaces_collapsed() {
    assert_eq!(to_deeplink_name("My   Script   Name"), "my-script-name");
}

#[test]
fn deeplink_name_underscores_become_hyphens() {
    assert_eq!(to_deeplink_name("my_script_name"), "my-script-name");
}

#[test]
fn deeplink_name_mixed_case_special_chars() {
    assert_eq!(to_deeplink_name("Hello (World) #1!"), "hello-world-1");
}

#[test]
fn deeplink_name_leading_trailing_special_chars() {
    assert_eq!(to_deeplink_name("---hello---"), "hello");
}

// --- merged from part_04.rs ---

#[test]
fn deeplink_name_single_word() {
    assert_eq!(to_deeplink_name("script"), "script");
}

// =========================================================================
// 22. CommandBarConfig preset dialog_config specifics
// =========================================================================

#[test]
fn command_bar_ai_style_has_search_top_and_headers() {
    let config = CommandBarConfig::ai_style();
    assert!(matches!(
        config.dialog_config.search_position,
        SearchPosition::Top
    ));
    assert!(matches!(
        config.dialog_config.section_style,
        SectionStyle::Headers
    ));
    assert!(config.dialog_config.show_icons);
    assert!(config.dialog_config.show_footer);
}

#[test]
fn command_bar_main_menu_has_search_bottom_and_separators() {
    let config = CommandBarConfig::main_menu_style();
    assert!(matches!(
        config.dialog_config.search_position,
        SearchPosition::Bottom
    ));
    assert!(matches!(
        config.dialog_config.section_style,
        SectionStyle::Separators
    ));
    assert!(!config.dialog_config.show_icons);
    assert!(!config.dialog_config.show_footer);
}

#[test]
fn command_bar_notes_has_search_top_and_separators() {
    let config = CommandBarConfig::notes_style();
    assert!(matches!(
        config.dialog_config.search_position,
        SearchPosition::Top
    ));
    assert!(matches!(
        config.dialog_config.section_style,
        SectionStyle::Separators
    ));
    assert!(config.dialog_config.show_icons);
    assert!(config.dialog_config.show_footer);
}

#[test]
fn command_bar_no_search_has_hidden_search() {
    let config = CommandBarConfig::no_search();
    assert!(matches!(
        config.dialog_config.search_position,
        SearchPosition::Hidden
    ));
}

// =========================================================================
// 23. Grouped items with SectionStyle::None
// =========================================================================

#[test]
fn grouped_items_none_style_has_no_headers_or_separators() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
    for item in &grouped {
        assert!(
            matches!(item, GroupedActionItem::Item(_)),
            "SectionStyle::None should produce only Items, not headers/separators"
        );
    }
}

#[test]
fn grouped_items_none_style_count_matches_filtered() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
    assert_eq!(grouped.len(), filtered.len());
}

// =========================================================================
// 24. Coerce selection on notes grouped actions
// =========================================================================

#[test]
fn coerce_selection_on_notes_grouped_finds_valid_item() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    let result = coerce_action_selection(&grouped, 0);
    assert!(
        result.is_some(),
        "Should find valid item in notes grouped actions"
    );
    if let Some(idx) = result {
        assert!(matches!(grouped[idx], GroupedActionItem::Item(_)));
    }
}

// =========================================================================
// 25. title_lower correctness for AI and notes contexts
// =========================================================================

#[test]
fn title_lower_matches_title_for_all_ai_actions() {
    for action in &get_ai_command_bar_actions() {
        assert_eq!(
            action.title_lower,
            action.title.to_lowercase(),
            "title_lower mismatch for AI action '{}'",
            action.id
        );
    }
}

#[test]
fn title_lower_matches_title_for_all_notes_actions() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    for action in &get_notes_command_bar_actions(&info) {
        assert_eq!(
            action.title_lower,
            action.title.to_lowercase(),
            "title_lower mismatch for notes action '{}'",
            action.id
        );
    }
}

#[test]
fn title_lower_matches_title_for_note_switcher_actions() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "a".into(),
            title: "Capital Title".into(),
            char_count: 10,
            is_current: true,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "b".into(),
            title: "ALL CAPS NOTE".into(),
            char_count: 20,
            is_current: false,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        },
    ];
    for action in &get_note_switcher_actions(&notes) {
        assert_eq!(
            action.title_lower,
            action.title.to_lowercase(),
            "title_lower mismatch for note switcher action '{}'",
            action.id
        );
    }
}

// =========================================================================
// 26. Scriptlet custom action with shortcut and description
// =========================================================================

#[test]
fn scriptlet_custom_action_shortcut_is_formatted() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo hi".into());
    scriptlet.actions.push(ScriptletAction {
        name: "Copy Output".into(),
        command: "copy-output".into(),
        tool: "bash".into(),
        code: "echo | pbcopy".into(),
        inputs: vec![],
        shortcut: Some("cmd+shift+c".into()),
        description: None,
    });
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let custom = actions
        .iter()
        .find(|a| a.id.starts_with("scriptlet_action:"))
        .unwrap();
    let sc = custom.shortcut.as_ref().unwrap();
    assert!(
        sc.contains('⌘') && sc.contains('⇧'),
        "Scriptlet shortcut should be formatted with symbols, got '{}'",
        sc
    );
}

#[test]
fn scriptlet_custom_action_description_propagated() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo hi".into());
    scriptlet.actions.push(ScriptletAction {
        name: "Explained".into(),
        command: "explained".into(),
        tool: "bash".into(),
        code: "echo".into(),
        inputs: vec![],
        shortcut: None,
        description: Some("A detailed description".into()),
    });
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let custom = actions
        .iter()
        .find(|a| a.id.starts_with("scriptlet_action:"))
        .unwrap();
    assert_eq!(
        custom.description.as_deref(),
        Some("A detailed description")
    );
}

// =========================================================================
// 27. All actions have ActionCategory::ScriptContext
// =========================================================================

#[test]
fn all_script_actions_are_script_context_category() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    for action in &get_script_context_actions(&script) {
        assert!(
            matches!(action.category, ActionCategory::ScriptContext),
            "Action '{}' should be ScriptContext category",
            action.id
        );
    }
}

#[test]
fn all_clipboard_actions_are_script_context_category() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    for action in &get_clipboard_history_context_actions(&entry) {
        assert!(
            matches!(action.category, ActionCategory::ScriptContext),
            "Clipboard action '{}' should be ScriptContext category",
            action.id
        );
    }
}

#[test]
fn all_file_actions_are_script_context_category() {
    let file = FileInfo {
        path: "/test.txt".into(),
        name: "test.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    for action in &get_file_context_actions(&file) {
        assert!(
            matches!(action.category, ActionCategory::ScriptContext),
            "File action '{}' should be ScriptContext category",
            action.id
        );
    }
}

#[test]
fn all_ai_actions_are_script_context_category() {
    for action in &get_ai_command_bar_actions() {
        assert!(
            matches!(action.category, ActionCategory::ScriptContext),
            "AI action '{}' should be ScriptContext category",
            action.id
        );
    }
}

// =========================================================================
// 28. File context primary action title includes filename
// =========================================================================

#[test]
fn file_primary_action_title_includes_filename() {
    let file = FileInfo {
        path: "/docs/readme.md".into(),
        name: "readme.md".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    assert!(
        actions[0].title.contains("readme.md"),
        "Primary action title should include filename, got '{}'",
        actions[0].title
    );
}

#[test]
fn file_directory_primary_title_includes_dirname() {
    let dir = FileInfo {
        path: "/projects/my-app".into(),
        name: "my-app".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&dir);
    assert!(
        actions[0].title.contains("my-app"),
        "Directory primary action title should include dirname, got '{}'",
        actions[0].title
    );
}

// =========================================================================
// 29. Frecency reset ranking conditional
// =========================================================================

#[test]
fn frecency_not_suggested_lacks_reset_ranking() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions_tmp = get_script_context_actions(&script);
    let ids = action_ids(&actions_tmp);
    assert!(!ids.contains(&"reset_ranking"));
}

#[test]
fn frecency_suggested_has_reset_ranking() {
    let script = ScriptInfo::new("test", "/path/test.ts").with_frecency(true, Some("/path".into()));
    let actions_tmp = get_script_context_actions(&script);
    let ids = action_ids(&actions_tmp);
    assert!(ids.contains(&"reset_ranking"));
}

#[test]
fn frecency_suggested_reset_ranking_is_last() {
    let script = ScriptInfo::new("test", "/path/test.ts").with_frecency(true, Some("/path".into()));
    let actions = get_script_context_actions(&script);
    let last = actions.last().unwrap();
    assert_eq!(
        last.id, "reset_ranking",
        "reset_ranking should be the last action"
    );
}

// =========================================================================
// 30. All actions have descriptions (broad check)
// =========================================================================

#[test]
fn all_script_context_actions_have_descriptions() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    for action in &get_script_context_actions(&script) {
        assert!(
            action.description.is_some(),
            "Script action '{}' should have a description",
            action.id
        );
    }
}

#[test]
fn all_ai_command_bar_actions_have_descriptions() {
    for action in &get_ai_command_bar_actions() {
        assert!(
            action.description.is_some(),
            "AI action '{}' should have a description",
            action.id
        );
    }
}

#[test]
fn all_path_actions_have_descriptions() {
    let path = PathInfo::new("test", "/test", false);
    for action in &get_path_context_actions(&path) {
        assert!(
            action.description.is_some(),
            "Path action '{}' should have a description",
            action.id
        );
    }
}

#[test]
fn all_file_actions_have_descriptions() {
    let file = FileInfo {
        path: "/test.txt".into(),
        name: "test.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    for action in &get_file_context_actions(&file) {
        assert!(
            action.description.is_some(),
            "File action '{}' should have a description",
            action.id
        );
    }
}

// =========================================================================
// 31. Clipboard has_action=false and no value for all entries
// =========================================================================

#[test]
fn clipboard_all_actions_have_no_value() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    for action in &get_clipboard_history_context_actions(&entry) {
        assert!(
            action.value.is_none(),
            "Clipboard action '{}' should have no value",
            action.id
        );
    }
}

// --- merged from part_05.rs ---

#[test]
fn path_all_actions_have_no_value() {
    let path = PathInfo::new("test", "/test", false);
    for action in &get_path_context_actions(&path) {
        assert!(
            action.value.is_none(),
            "Path action '{}' should have no value",
            action.id
        );
    }
}

// =========================================================================
// 32. Section header count consistency for AI with headers
// =========================================================================

#[test]
fn ai_section_header_count_is_seven() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let header_count = count_section_headers(&actions, &filtered);
    assert_eq!(
        header_count, 7,
        "AI command bar should have 7 section headers (Response, Actions, Attachments, Export, Actions, Help, Settings)"
    );
}

// =========================================================================
// 33. Scriptlet context actions from get_scriptlet_context_actions_with_custom
//     have all the same universal actions as script context
// =========================================================================

#[test]
fn scriptlet_context_has_shortcut_alias_deeplink() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions_tmp = get_scriptlet_context_actions_with_custom(&script, None);
    let ids = action_ids(&actions_tmp);
    assert!(ids.contains(&"add_shortcut"));
    assert!(ids.contains(&"add_alias"));
    assert!(ids.contains(&"copy_deeplink"));
}

#[test]
fn scriptlet_context_has_edit_reveal_copy() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions_tmp = get_scriptlet_context_actions_with_custom(&script, None);
    let ids = action_ids(&actions_tmp);
    assert!(ids.contains(&"edit_scriptlet"));
    assert!(ids.contains(&"reveal_scriptlet_in_finder"));
    assert!(ids.contains(&"copy_scriptlet_path"));
    assert!(ids.contains(&"copy_content"));
}

// =========================================================================
// 34. Notes new_note always present across all permutations
// =========================================================================

#[test]
fn notes_new_note_always_present() {
    for sel in [false, true] {
        for trash in [false, true] {
            for auto in [false, true] {
                let info = NotesInfo {
                    has_selection: sel,
                    is_trash_view: trash,
                    auto_sizing_enabled: auto,
                };
                let actions_tmp = get_notes_command_bar_actions(&info);
                let ids = action_ids(&actions_tmp);
                assert!(
                    ids.contains(&"new_note"),
                    "new_note should always be present (sel={}, trash={}, auto={})",
                    sel,
                    trash,
                    auto
                );
            }
        }
    }
}

#[test]
fn notes_browse_notes_always_present() {
    for sel in [false, true] {
        for trash in [false, true] {
            for auto in [false, true] {
                let info = NotesInfo {
                    has_selection: sel,
                    is_trash_view: trash,
                    auto_sizing_enabled: auto,
                };
                let actions_tmp = get_notes_command_bar_actions(&info);
                let ids = action_ids(&actions_tmp);
                assert!(
                    ids.contains(&"browse_notes"),
                    "browse_notes should always be present (sel={}, trash={}, auto={})",
                    sel,
                    trash,
                    auto
                );
            }
        }
    }
}

// =========================================================================
// 35. Fuzzy match on real action IDs across contexts
// =========================================================================

#[test]
fn fuzzy_match_on_clipboard_action_titles() {
    let entry = ClipboardEntryInfo {
        id: "e".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    // "pke" should fuzzy match "Paste and Keep Window Open" (p-a-s-t-e... k-e-e-p)
    let paste_keep = actions
        .iter()
        .find(|a| a.id == "clipboard_paste_keep_open")
        .unwrap();
    assert!(ActionsDialog::fuzzy_match(&paste_keep.title_lower, "pke"));
}

#[test]
fn fuzzy_match_on_notes_action_titles() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let new_note = actions.iter().find(|a| a.id == "new_note").unwrap();
    // "nn" matches "new note" → n at 0, n at 4
    assert!(ActionsDialog::fuzzy_match(&new_note.title_lower, "nn"));
}

// =========================================================================
// 36. Grouped items headers style produces section headers
// =========================================================================

#[test]
fn grouped_items_headers_style_has_section_headers() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    let header_count = grouped
        .iter()
        .filter(|i| matches!(i, GroupedActionItem::SectionHeader(_)))
        .count();
    assert!(
        header_count > 0,
        "Headers style should produce at least one section header"
    );
}

#[test]
fn grouped_items_separators_style_has_separator_items() {
    let actions = get_ai_command_bar_actions();
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
    // Should have separator items but no header items
    let headers = grouped
        .iter()
        .filter(|i| matches!(i, GroupedActionItem::SectionHeader(_)))
        .count();
    assert_eq!(headers, 0, "Separators style should have no headers");
}
