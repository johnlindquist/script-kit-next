#[cfg(test)]
#[allow(unused_imports, dead_code)]
mod dialog_builtin_action_validation_tests_2 {
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

}

#[cfg(test)]
#[allow(unused_imports, dead_code)]
mod dialog_builtin_action_validation_tests_3 {
    // --- merged from part_01.rs ---
// Built-in action behavioral validation tests — batch 3
//
// Validates randomly-selected built-in actions across window dialogs and
// contexts that were NOT covered in batch 1 or batch 2. Focuses on:
// - Shortcut uniqueness within each context (no duplicate hotkeys)
// - Action ordering stability / determinism across repeated calls
// - Cross-context action exclusivity (clipboard IDs never in file context etc.)
// - with_shortcut_opt builder correctness
// - Section ordering in AI, Notes, and New Chat command bars
// - Scriptlet with multiple custom H3 actions: ordering and ID format
// - Action title formatting with varied action_verbs
// - Path context shortcut assignments completeness
// - Clipboard ordering invariant (paste first, deletes last)
// - Mixed flag combinations on ScriptInfo
// - Note switcher icon hierarchy for all is_current × is_pinned combos
// - to_deeplink_name with unicode / emoji edge cases
// - Score stacking (title + description bonuses accumulate)
// - File context primary title includes filename
// - Scriptlet context action order: run > custom > shortcut > built-in > deeplink
// - Chat model checkmark only on current model
// - Notes conditional section counts across all 8 permutations
// - CommandBarConfig notes_style specifics

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
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;
use crate::scriptlets::{Scriptlet, ScriptletAction};
use std::collections::HashSet;

// =========================================================================
// Helpers
// =========================================================================

fn action_ids(actions: &[Action]) -> Vec<&str> {
    actions.iter().map(|a| a.id.as_str()).collect()
}

fn find_action<'a>(actions: &'a [Action], id: &str) -> Option<&'a Action> {
    actions.iter().find(|a| a.id == id)
}

fn sections_in_order(actions: &[Action]) -> Vec<&str> {
    let mut sections = Vec::new();
    for a in actions {
        if let Some(ref s) = a.section {
            if sections
                .last()
                .map(|l: &&str| *l != s.as_str())
                .unwrap_or(true)
            {
                sections.push(s.as_str());
            }
        }
    }
    sections
}

// =========================================================================
// 1. Shortcut uniqueness within context — no two actions share a hotkey
// =========================================================================

#[test]
fn script_context_shortcuts_are_unique() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let mut seen = HashSet::new();
    for a in &actions {
        if let Some(ref s) = a.shortcut {
            assert!(
                seen.insert(s.as_str()),
                "Duplicate shortcut '{}' on action '{}' in script context",
                s,
                a.id
            );
        }
    }
}

#[test]
fn clipboard_context_text_shortcuts_are_unique() {
    let entry = ClipboardEntryInfo {
        id: "e1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let mut seen = HashSet::new();
    for a in &actions {
        if let Some(ref s) = a.shortcut {
            assert!(
                seen.insert(s.as_str()),
                "Duplicate shortcut '{}' on action '{}' in clipboard text context",
                s,
                a.id
            );
        }
    }
}

#[test]
fn file_context_shortcuts_are_unique() {
    let file = FileInfo {
        path: "/test.txt".into(),
        name: "test.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    let mut seen = HashSet::new();
    for a in &actions {
        if let Some(ref s) = a.shortcut {
            assert!(
                seen.insert(s.as_str()),
                "Duplicate shortcut '{}' on action '{}' in file context",
                s,
                a.id
            );
        }
    }
}

#[test]
fn path_context_shortcuts_are_unique() {
    let path = PathInfo {
        path: "/usr/local".into(),
        name: "local".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path);
    let mut seen = HashSet::new();
    for a in &actions {
        if let Some(ref s) = a.shortcut {
            assert!(
                seen.insert(s.as_str()),
                "Duplicate shortcut '{}' on action '{}' in path context",
                s,
                a.id
            );
        }
    }
}

#[test]
fn ai_command_bar_shortcuts_are_unique() {
    let actions = get_ai_command_bar_actions();
    let mut seen = HashSet::new();
    for a in &actions {
        if let Some(ref s) = a.shortcut {
            assert!(
                seen.insert(s.as_str()),
                "Duplicate shortcut '{}' on action '{}' in AI command bar",
                s,
                a.id
            );
        }
    }
}

// =========================================================================
// 2. Action ordering stability — same inputs always produce same output
// =========================================================================

#[test]
fn script_context_ordering_is_deterministic() {
    let script = ScriptInfo::with_shortcut_and_alias(
        "stable",
        "/path/stable.ts",
        Some("cmd+s".into()),
        Some("st".into()),
    )
    .with_frecency(true, Some("/path/stable.ts".into()));

    let a1 = get_script_context_actions(&script);
    let a2 = get_script_context_actions(&script);
    let a3 = get_script_context_actions(&script);
    let ids_1 = action_ids(&a1);
    let ids_2 = action_ids(&a2);
    let ids_3 = action_ids(&a3);

    assert_eq!(
        ids_1, ids_2,
        "Action ordering should be stable across calls"
    );
    assert_eq!(
        ids_2, ids_3,
        "Action ordering should be stable across calls"
    );
}

#[test]
fn clipboard_ordering_is_deterministic() {
    let entry = ClipboardEntryInfo {
        id: "det".into(),
        content_type: ContentType::Image,
        pinned: true,
        preview: "img".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: Some("Safari".into()),
    };
    let a1 = get_clipboard_history_context_actions(&entry);
    let a2 = get_clipboard_history_context_actions(&entry);
    let ids_1 = action_ids(&a1);
    let ids_2 = action_ids(&a2);
    assert_eq!(ids_1, ids_2);
}

#[test]
fn notes_command_bar_ordering_is_deterministic() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let a1 = get_notes_command_bar_actions(&info);
    let a2 = get_notes_command_bar_actions(&info);
    let ids_1 = action_ids(&a1);
    let ids_2 = action_ids(&a2);
    assert_eq!(ids_1, ids_2);
}

// =========================================================================
// 3. Cross-context action exclusivity
// =========================================================================

#[test]
fn clipboard_ids_never_appear_in_file_context() {
    let file = FileInfo {
        path: "/test.txt".into(),
        name: "test.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let file_actions = get_file_context_actions(&file);
    let file_ids: HashSet<&str> = action_ids(&file_actions).into_iter().collect();

    // Clipboard-specific action IDs
    let clipboard_only = [
        "clipboard_paste",
        "clipboard_copy",
        "clipboard_pin",
        "clipboard_unpin",
        "clipboard_ocr",
        "clipboard_delete",
        "clipboard_delete_all",
        "clipboard_save_snippet",
        "clipboard_share",
        "clipboard_attach_to_ai",
    ];

    for id in &clipboard_only {
        assert!(
            !file_ids.contains(id),
            "File context should not contain clipboard action '{}'",
            id
        );
    }
}

#[test]
fn file_ids_never_appear_in_clipboard_context() {
    let entry = ClipboardEntryInfo {
        id: "c1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let clip_actions = get_clipboard_history_context_actions(&entry);
    let clip_ids: HashSet<&str> = action_ids(&clip_actions).into_iter().collect();

    let file_only = [
        "open_file",
        "open_directory",
        "open_with",
        "show_info",
        "copy_filename",
    ];

    for id in &file_only {
        assert!(
            !clip_ids.contains(id),
            "Clipboard context should not contain file action '{}'",
            id
        );
    }
}

#[test]
fn script_ids_never_appear_in_path_context() {
    let path = PathInfo {
        path: "/usr/bin".into(),
        name: "bin".into(),
        is_dir: true,
    };
    let path_actions = get_path_context_actions(&path);
    let path_ids: HashSet<&str> = action_ids(&path_actions).into_iter().collect();

    let script_only = [
        "run_script",
        "edit_script",
        "view_logs",
        "add_shortcut",
        "add_alias",
        "copy_deeplink",
        "reset_ranking",
    ];

    for id in &script_only {
        assert!(
            !path_ids.contains(id),
            "Path context should not contain script action '{}'",
            id
        );
    }
}

// =========================================================================
// 4. with_shortcut_opt builder correctness
// =========================================================================

#[test]
fn with_shortcut_opt_some_sets_shortcut() {
    let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
        .with_shortcut_opt(Some("⌘T".to_string()));
    assert_eq!(action.shortcut, Some("⌘T".to_string()));
    assert_eq!(action.shortcut_lower, Some("⌘t".to_string()));
}

#[test]
fn with_shortcut_opt_none_leaves_shortcut_none() {
    let action =
        Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut_opt(None);
    assert!(action.shortcut.is_none());
    assert!(action.shortcut_lower.is_none());
}

// =========================================================================
// 5. AI command bar section ordering: Response > Actions > Attachments > Settings
// =========================================================================

#[test]
fn ai_command_bar_section_order_is_correct() {
    let actions = get_ai_command_bar_actions();
    let sections = sections_in_order(&actions);
    assert_eq!(
        sections,
        vec![
            "Response",
            "Actions",
            "Attachments",
            "Export",
            "Actions",
            "Help",
            "Settings"
        ],
        "AI command bar sections should be in correct order"
    );
}

#[test]
fn ai_command_bar_response_section_has_three_actions() {
    let actions = get_ai_command_bar_actions();
    let response_count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Response"))
        .count();
    assert_eq!(response_count, 3, "Response section should have 3 actions");
}

#[test]
fn ai_command_bar_actions_section_has_four_actions() {
    let actions = get_ai_command_bar_actions();
    let actions_count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Actions"))
        .count();
    assert_eq!(actions_count, 4, "Actions section should have 4 actions");
}

#[test]
fn ai_command_bar_attachments_section_has_two_actions() {
    let actions = get_ai_command_bar_actions();
    let attachments_count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Attachments"))
        .count();
    assert_eq!(
        attachments_count, 2,
        "Attachments section should have 2 actions"
    );
}

#[test]
fn ai_command_bar_settings_section_has_one_action() {
    let actions = get_ai_command_bar_actions();
    let settings_count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Settings"))
        .count();
    assert_eq!(settings_count, 1, "Settings section should have 1 action");
}

// =========================================================================
// 6. Notes command bar section ordering: Notes > Edit > Copy > Export > Settings
// =========================================================================

#[test]
fn notes_command_bar_section_order_full() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let sections = sections_in_order(&actions);
    assert_eq!(
        sections,
        vec!["Notes", "Edit", "Copy", "Export", "Settings"],
        "Notes command bar sections should be in correct order"
    );
}

}

#[cfg(test)]
#[allow(unused_imports, dead_code)]
mod dialog_builtin_action_validation_tests_4 {
    // --- merged from part_01.rs ---
// Built-in action behavioral validation tests — batch 4
//
// Validates randomly-selected built-in actions across window dialogs and
// contexts that were NOT covered in batches 1–3. Focuses on:
// - Agent flag interactions with shortcut/alias/frecency combinations
// - Custom action verbs propagating correctly into primary action titles
// - Scriptlet context vs script context systematic action set comparison
// - Clipboard text vs image action count differential (macOS)
// - Path context action IDs all snake_case
// - File context FileType variants produce consistent action set
// - Notes section label exhaustiveness for full-feature permutation
// - AI command bar icon-per-section coverage
// - New chat with all-empty inputs produces empty output
// - score_action edge cases (empty query, single char, unicode)
// - fuzzy_match boundary conditions (empty strings, longer needle, etc.)
// - parse_shortcut_keycaps for all modifier symbols
// - format_shortcut_hint roundtrips for unusual key names
// - to_deeplink_name with CJK, emoji, RTL characters
// - Grouped items with realistic AI command bar data
// - coerce_action_selection on all-headers edge case
// - Note switcher section assignment (Pinned vs Recent)
// - Clipboard frontmost app edge cases (empty string, unicode)
// - Chat with no models, no messages, no response
// - Multiple scriptlet custom actions preserve declaration order
// - Action constructor lowercase caching with unicode titles

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
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;
use crate::scriptlets::{Scriptlet, ScriptletAction};
use std::collections::HashSet;

// =========================================================================
// Helpers
// =========================================================================

fn action_ids(actions: &[Action]) -> Vec<&str> {
    actions.iter().map(|a| a.id.as_str()).collect()
}

fn find_action<'a>(actions: &'a [Action], id: &str) -> Option<&'a Action> {
    actions.iter().find(|a| a.id == id)
}

fn sections_in_order(actions: &[Action]) -> Vec<&str> {
    let mut sections = Vec::new();
    for a in actions {
        if let Some(ref s) = a.section {
            if sections
                .last()
                .map(|l: &&str| *l != s.as_str())
                .unwrap_or(true)
            {
                sections.push(s.as_str());
            }
        }
    }
    sections
}

// =========================================================================
// 1. Agent flag interactions with shortcut/alias/frecency
// =========================================================================

#[test]
fn agent_with_shortcut_has_update_and_remove_shortcut() {
    let mut agent = ScriptInfo::new("My Agent", "/path/to/agent.md");
    agent.is_script = false;
    agent.is_agent = true;
    agent.shortcut = Some("cmd+a".to_string());
    let actions = get_script_context_actions(&agent);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"update_shortcut"));
    assert!(ids.contains(&"remove_shortcut"));
    assert!(!ids.contains(&"add_shortcut"));
}

#[test]
fn agent_without_shortcut_has_add_shortcut() {
    let mut agent = ScriptInfo::new("My Agent", "/path/to/agent.md");
    agent.is_script = false;
    agent.is_agent = true;
    let actions = get_script_context_actions(&agent);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"add_shortcut"));
    assert!(!ids.contains(&"update_shortcut"));
    assert!(!ids.contains(&"remove_shortcut"));
}

#[test]
fn agent_with_alias_has_update_and_remove_alias() {
    let mut agent = ScriptInfo::new("My Agent", "/path/to/agent.md");
    agent.is_script = false;
    agent.is_agent = true;
    agent.alias = Some("ag".to_string());
    let actions = get_script_context_actions(&agent);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"update_alias"));
    assert!(ids.contains(&"remove_alias"));
    assert!(!ids.contains(&"add_alias"));
}

#[test]
fn agent_with_frecency_has_reset_ranking() {
    let mut agent = ScriptInfo::new("My Agent", "/path/to/agent.md");
    agent.is_script = false;
    agent.is_agent = true;
    agent.is_suggested = true;
    agent.frecency_path = Some("agent:/path".to_string());
    let actions = get_script_context_actions(&agent);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"reset_ranking"));
}

#[test]
fn agent_without_frecency_lacks_reset_ranking() {
    let mut agent = ScriptInfo::new("My Agent", "/path/to/agent.md");
    agent.is_script = false;
    agent.is_agent = true;
    let actions = get_script_context_actions(&agent);
    let ids = action_ids(&actions);
    assert!(!ids.contains(&"reset_ranking"));
}

#[test]
fn agent_has_edit_agent_title() {
    let mut agent = ScriptInfo::new("My Agent", "/path/to/agent.md");
    agent.is_script = false;
    agent.is_agent = true;
    let actions = get_script_context_actions(&agent);
    let edit = find_action(&actions, "edit_script").unwrap();
    assert_eq!(edit.title, "Edit Agent");
}

#[test]
fn agent_has_reveal_copy_path_copy_content() {
    let mut agent = ScriptInfo::new("My Agent", "/path/to/agent.md");
    agent.is_script = false;
    agent.is_agent = true;
    let actions = get_script_context_actions(&agent);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"reveal_in_finder"));
    assert!(ids.contains(&"copy_path"));
    assert!(ids.contains(&"copy_content"));
}

#[test]
fn agent_lacks_view_logs() {
    let mut agent = ScriptInfo::new("My Agent", "/path/to/agent.md");
    agent.is_script = false;
    agent.is_agent = true;
    let actions = get_script_context_actions(&agent);
    let ids = action_ids(&actions);
    assert!(!ids.contains(&"view_logs"));
}

// =========================================================================
// 2. Custom action verbs propagate into primary action title
// =========================================================================

#[test]
fn action_verb_run_in_primary_title() {
    let script = ScriptInfo::new("Test Script", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].title, "Run \"Test Script\"");
}

#[test]
fn action_verb_launch_in_primary_title() {
    let script =
        ScriptInfo::with_action_verb("Safari", "/Applications/Safari.app", false, "Launch");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].title, "Launch \"Safari\"");
}

#[test]
fn action_verb_switch_to_in_primary_title() {
    let script = ScriptInfo::with_action_verb("My Window", "window:123", false, "Switch to");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].title, "Switch to \"My Window\"");
}

#[test]
fn action_verb_open_in_primary_title() {
    let script = ScriptInfo::with_action_verb("Clipboard History", "builtin:ch", false, "Open");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].title, "Open \"Clipboard History\"");
}

#[test]
fn action_verb_execute_in_primary_title() {
    let script = ScriptInfo::with_all("My Task", "/path/task.ts", true, "Execute", None, None);
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].title, "Execute \"My Task\"");
}

// =========================================================================
// 3. Scriptlet context vs script context: systematic comparison
// =========================================================================

#[test]
fn scriptlet_context_has_edit_scriptlet_not_edit_script() {
    let scriptlet = ScriptInfo::scriptlet("My Snippet", "/path/to/bundle.md", None, None);
    let actions = get_script_context_actions(&scriptlet);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"edit_scriptlet"));
    assert!(!ids.contains(&"edit_script"));
}

#[test]
fn scriptlet_context_has_reveal_scriptlet_not_reveal() {
    let scriptlet = ScriptInfo::scriptlet("My Snippet", "/path/to/bundle.md", None, None);
    let actions = get_script_context_actions(&scriptlet);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"reveal_scriptlet_in_finder"));
    // The regular reveal_in_finder should NOT be present for scriptlets
    assert!(!ids.contains(&"reveal_in_finder"));
}

#[test]
fn scriptlet_context_has_copy_scriptlet_path_not_copy_path() {
    let scriptlet = ScriptInfo::scriptlet("My Snippet", "/path/to/bundle.md", None, None);
    let actions = get_script_context_actions(&scriptlet);
    let ids = action_ids(&actions);
    assert!(ids.contains(&"copy_scriptlet_path"));
    assert!(!ids.contains(&"copy_path"));
}

#[test]
fn scriptlet_and_script_both_have_copy_content() {
    let scriptlet = ScriptInfo::scriptlet("My Snippet", "/path/to/bundle.md", None, None);
    let script = ScriptInfo::new("My Script", "/path/script.ts");
    let scriptlet_actions = get_script_context_actions(&scriptlet);
    let script_actions = get_script_context_actions(&script);
    assert!(action_ids(&scriptlet_actions).contains(&"copy_content"));
    assert!(action_ids(&script_actions).contains(&"copy_content"));
}

#[test]
fn scriptlet_lacks_view_logs() {
    let scriptlet = ScriptInfo::scriptlet("My Snippet", "/path/to/bundle.md", None, None);
    let actions = get_script_context_actions(&scriptlet);
    assert!(!action_ids(&actions).contains(&"view_logs"));
}

// =========================================================================
// 4. Clipboard text vs image action count differential
// =========================================================================

#[test]
fn clipboard_image_has_strictly_more_actions_than_text() {
    let text_entry = ClipboardEntryInfo {
        id: "t1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let image_entry = ClipboardEntryInfo {
        id: "i1".to_string(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "Image (800x600)".to_string(),
        image_dimensions: Some((800, 600)),
        frontmost_app_name: None,
    };
    let text_actions = get_clipboard_history_context_actions(&text_entry);
    let image_actions = get_clipboard_history_context_actions(&image_entry);
    assert!(
        image_actions.len() > text_actions.len(),
        "Image should have more actions than text: {} > {}",
        image_actions.len(),
        text_actions.len()
    );
}

#[test]
fn clipboard_image_has_ocr_text_does_not() {
    let text_entry = ClipboardEntryInfo {
        id: "t1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let image_entry = ClipboardEntryInfo {
        id: "i1".to_string(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "Image".to_string(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let text_actions = get_clipboard_history_context_actions(&text_entry);
    let image_actions = get_clipboard_history_context_actions(&image_entry);
    let text_ids = action_ids(&text_actions);
    let image_ids = action_ids(&image_actions);
    assert!(!text_ids.contains(&"clipboard_ocr"));
    assert!(image_ids.contains(&"clipboard_ocr"));
}

#[test]
fn clipboard_pinned_shows_unpin_unpinned_shows_pin() {
    let pinned = ClipboardEntryInfo {
        id: "p1".to_string(),
        content_type: ContentType::Text,
        pinned: true,
        preview: "pinned".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let unpinned = ClipboardEntryInfo {
        id: "u1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "unpinned".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let pinned_actions = get_clipboard_history_context_actions(&pinned);
    let unpinned_actions = get_clipboard_history_context_actions(&unpinned);
    let pinned_ids = action_ids(&pinned_actions);
    let unpinned_ids = action_ids(&unpinned_actions);
    assert!(pinned_ids.contains(&"clipboard_unpin"));
    assert!(!pinned_ids.contains(&"clipboard_pin"));
    assert!(unpinned_ids.contains(&"clipboard_pin"));
    assert!(!unpinned_ids.contains(&"clipboard_unpin"));
}

// =========================================================================
// 5. Path context action IDs are all snake_case
// =========================================================================

#[test]
fn path_context_all_ids_are_snake_case() {
    let path = PathInfo {
        name: "test.txt".to_string(),
        path: "/home/user/test.txt".to_string(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path);
    for action in &actions {
        assert!(
            !action.id.contains(' ') && !action.id.contains('-'),
            "Action ID '{}' should be snake_case",
            action.id
        );
        assert_eq!(
            action.id,
            action.id.to_lowercase(),
            "Action ID '{}' should be lowercase",
            action.id
        );
    }
}

#[test]
fn path_context_dir_all_ids_are_snake_case() {
    let path = PathInfo {
        name: "Documents".to_string(),
        path: "/home/user/Documents".to_string(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path);
    for action in &actions {
        assert!(
            !action.id.contains(' '),
            "Action ID '{}' should not contain spaces",
            action.id
        );
    }
}

// =========================================================================
// 6. File context FileType variants produce consistent action set structure
// =========================================================================

#[test]
fn file_context_all_file_types_have_reveal_and_copy_path() {
    let file_types = vec![
        FileType::File,
        FileType::Document,
        FileType::Image,
        FileType::Application,
        FileType::Audio,
    ];
    for ft in file_types {
        let info = FileInfo {
            path: format!("/tmp/test.{:?}", ft),
            name: format!("test.{:?}", ft),
            file_type: ft,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        let ids = action_ids(&actions);
        assert!(
            ids.contains(&"reveal_in_finder"),
            "FileType {:?} should have reveal_in_finder",
            info.file_type
        );
        assert!(
            ids.contains(&"copy_path"),
            "FileType {:?} should have copy_path",
            info.file_type
        );
        assert!(
            ids.contains(&"copy_filename"),
            "FileType {:?} should have copy_filename",
            info.file_type
        );
    }
}

#[test]
fn file_context_file_has_open_file_dir_has_open_directory() {
    let file = FileInfo {
        path: "/tmp/readme.md".to_string(),
        name: "readme.md".to_string(),
        file_type: FileType::File,
        is_dir: false,
    };
    let dir = FileInfo {
        path: "/tmp/src".to_string(),
        name: "src".to_string(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let file_actions = get_file_context_actions(&file);
    let dir_actions = get_file_context_actions(&dir);
    assert!(action_ids(&file_actions).contains(&"open_file"));
    assert!(!action_ids(&file_actions).contains(&"open_directory"));
    assert!(action_ids(&dir_actions).contains(&"open_directory"));
    assert!(!action_ids(&dir_actions).contains(&"open_file"));
}

}
