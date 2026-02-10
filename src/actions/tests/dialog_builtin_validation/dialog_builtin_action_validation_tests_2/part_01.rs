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
