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
