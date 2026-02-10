// Batch 21: Built-in action validation tests
//
// 146 tests across 30 categories validating built-in dialog actions.

use super::builders::{
    get_ai_command_bar_actions, get_chat_context_actions, get_clipboard_history_context_actions,
    get_file_context_actions, get_new_chat_actions, get_note_switcher_actions,
    get_notes_command_bar_actions, get_path_context_actions, get_script_context_actions,
    get_scriptlet_context_actions_with_custom, ChatModelInfo, ChatPromptInfo, ClipboardEntryInfo,
    NewChatModelInfo, NewChatPresetInfo, NoteSwitcherNoteInfo, NotesInfo,
};
use super::command_bar::CommandBarConfig;
use super::dialog::{build_grouped_items_static, coerce_action_selection, GroupedActionItem};
use super::types::{
    Action, ActionCategory, ActionsDialogConfig, AnchorPosition, ScriptInfo, SearchPosition,
    SectionStyle,
};
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;

// ============================================================
// 1. Script context: is_script / is_scriptlet / is_agent mutually
//    exclusive action sets
// ============================================================

#[test]
fn batch21_script_only_has_view_logs() {
    let s = ScriptInfo::new("s", "/p");
    let actions = get_script_context_actions(&s);
    assert!(actions.iter().any(|a| a.id == "view_logs"));
}

#[test]
fn batch21_scriptlet_no_view_logs() {
    let s = ScriptInfo::scriptlet("s", "/p", None, None);
    let actions = get_script_context_actions(&s);
    assert!(!actions.iter().any(|a| a.id == "view_logs"));
}

#[test]
fn batch21_agent_no_view_logs() {
    let mut s = ScriptInfo::new("a", "/p");
    s.is_agent = true;
    s.is_script = false;
    let actions = get_script_context_actions(&s);
    assert!(!actions.iter().any(|a| a.id == "view_logs"));
}

#[test]
fn batch21_builtin_no_view_logs() {
    let s = ScriptInfo::builtin("B");
    let actions = get_script_context_actions(&s);
    assert!(!actions.iter().any(|a| a.id == "view_logs"));
}

#[test]
fn batch21_script_has_edit_script_agent_has_edit_agent() {
    let script = ScriptInfo::new("s", "/p");
    let sa = get_script_context_actions(&script);
    let edit = sa.iter().find(|a| a.id == "edit_script").unwrap();
    assert_eq!(edit.title, "Edit Script");

    let mut agent = ScriptInfo::new("a", "/p");
    agent.is_agent = true;
    agent.is_script = false;
    let aa = get_script_context_actions(&agent);
    let edit2 = aa.iter().find(|a| a.id == "edit_script").unwrap();
    assert_eq!(edit2.title, "Edit Agent");
}

// ============================================================
// 2. Script context: run_script always first
// ============================================================

#[test]
fn batch21_run_script_is_first_for_script() {
    let s = ScriptInfo::new("s", "/p");
    let actions = get_script_context_actions(&s);
    assert_eq!(actions[0].id, "run_script");
}

#[test]
fn batch21_run_script_is_first_for_builtin() {
    let s = ScriptInfo::builtin("B");
    let actions = get_script_context_actions(&s);
    assert_eq!(actions[0].id, "run_script");
}

#[test]
fn batch21_run_script_is_first_for_scriptlet() {
    let s = ScriptInfo::scriptlet("s", "/p", None, None);
    let actions = get_script_context_actions(&s);
    assert_eq!(actions[0].id, "run_script");
}

#[test]
fn batch21_run_script_is_first_for_agent() {
    let mut s = ScriptInfo::new("a", "/p");
    s.is_agent = true;
    s.is_script = false;
    let actions = get_script_context_actions(&s);
    assert_eq!(actions[0].id, "run_script");
}

#[test]
fn batch21_run_script_is_first_for_scriptlet_with_custom() {
    let s = ScriptInfo::scriptlet("s", "/p", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    assert_eq!(actions[0].id, "run_script");
}

// ============================================================
// 3. File context: title format includes quoted name
// ============================================================

#[test]
fn batch21_file_open_title_contains_quoted_name() {
    let fi = FileInfo {
        path: "/tmp/readme.md".into(),
        name: "readme.md".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&fi);
    let open = actions.iter().find(|a| a.id == "open_file").unwrap();
    assert!(open.title.contains("\"readme.md\""));
}

#[test]
fn batch21_dir_open_title_contains_quoted_name() {
    let fi = FileInfo {
        path: "/tmp/docs".into(),
        name: "docs".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&fi);
    let open = actions.iter().find(|a| a.id == "open_directory").unwrap();
    assert!(open.title.contains("\"docs\""));
}

#[test]
fn batch21_file_open_title_starts_with_open() {
    let fi = FileInfo {
        path: "/a".into(),
        name: "a".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&fi);
    let open = actions.iter().find(|a| a.id == "open_file").unwrap();
    assert!(open.title.starts_with("Open"));
}

#[test]
fn batch21_dir_open_title_starts_with_open() {
    let fi = FileInfo {
        path: "/a".into(),
        name: "a".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&fi);
    let open = actions.iter().find(|a| a.id == "open_directory").unwrap();
    assert!(open.title.starts_with("Open"));
}

// ============================================================
// 4. Path context: dir primary=open_directory, file primary=select_file
// ============================================================

#[test]
fn batch21_path_dir_primary_is_open_directory() {
    let pi = PathInfo {
        path: "/d".into(),
        name: "d".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&pi);
    assert_eq!(actions[0].id, "open_directory");
}

#[test]
fn batch21_path_file_primary_is_select_file() {
    let pi = PathInfo {
        path: "/f".into(),
        name: "f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&pi);
    assert_eq!(actions[0].id, "select_file");
}

#[test]
fn batch21_path_dir_title_contains_name() {
    let pi = PathInfo {
        path: "/mydir".into(),
        name: "mydir".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&pi);
    assert!(actions[0].title.contains("\"mydir\""));
}

#[test]
fn batch21_path_file_title_contains_name() {
    let pi = PathInfo {
        path: "/f.txt".into(),
        name: "f.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&pi);
    assert!(actions[0].title.contains("\"f.txt\""));
}

#[test]
fn batch21_path_dir_and_file_same_action_count() {
    let dir = PathInfo {
        path: "/d".into(),
        name: "d".into(),
        is_dir: true,
    };
    let file = PathInfo {
        path: "/f".into(),
        name: "f".into(),
        is_dir: false,
    };
    assert_eq!(
        get_path_context_actions(&dir).len(),
        get_path_context_actions(&file).len()
    );
}

// ============================================================
// 5. AI command bar: total action count and section counts
// ============================================================

#[test]
fn batch21_ai_command_bar_total_12_actions() {
    let actions = get_ai_command_bar_actions();
    assert_eq!(actions.len(), 12);
}

#[test]
fn batch21_ai_command_bar_response_section_3() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Response"))
        .count();
    assert_eq!(count, 3);
}

#[test]
fn batch21_ai_command_bar_actions_section_4() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Actions"))
        .count();
    assert_eq!(count, 4);
}

#[test]
fn batch21_ai_command_bar_attachments_section_2() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Attachments"))
        .count();
    assert_eq!(count, 2);
}

#[test]
fn batch21_ai_command_bar_export_section_1() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Export"))
        .count();
    assert_eq!(count, 1);
}

// ============================================================
// 6. AI command bar: copy_chat and copy_last_code details
// ============================================================

#[test]
fn batch21_ai_copy_chat_shortcut() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "copy_chat").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌥⇧⌘C"));
}

#[test]
fn batch21_ai_copy_chat_icon_copy() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "copy_chat").unwrap();
    assert_eq!(a.icon, Some(IconName::Copy));
}

#[test]
fn batch21_ai_copy_last_code_shortcut() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "copy_last_code").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌥⌘C"));
}

#[test]
fn batch21_ai_copy_last_code_icon_code() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "copy_last_code").unwrap();
    assert_eq!(a.icon, Some(IconName::Code));
}

#[test]
fn batch21_ai_copy_last_code_section_response() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "copy_last_code").unwrap();
    assert_eq!(a.section.as_deref(), Some("Response"));
}

// ============================================================
// 7. AI command bar: paste_image details
// ============================================================

#[test]
fn batch21_ai_paste_image_shortcut() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "paste_image").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌘V"));
}

#[test]
fn batch21_ai_paste_image_icon_file() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "paste_image").unwrap();
    assert_eq!(a.icon, Some(IconName::File));
}

#[test]
fn batch21_ai_paste_image_section_attachments() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "paste_image").unwrap();
    assert_eq!(a.section.as_deref(), Some("Attachments"));
}

#[test]
fn batch21_ai_add_attachment_shortcut() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "add_attachment").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⇧⌘A"));
}

// ============================================================
// 8. AI command bar: toggle_shortcuts_help details
// ============================================================

#[test]
fn batch21_ai_toggle_shortcuts_help_shortcut() {
    let actions = get_ai_command_bar_actions();
    let a = actions
        .iter()
        .find(|a| a.id == "toggle_shortcuts_help")
        .unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌘/"));
}

#[test]
fn batch21_ai_toggle_shortcuts_help_icon_star() {
    let actions = get_ai_command_bar_actions();
    let a = actions
        .iter()
        .find(|a| a.id == "toggle_shortcuts_help")
        .unwrap();
    assert_eq!(a.icon, Some(IconName::Star));
}

#[test]
fn batch21_ai_toggle_shortcuts_help_section_help() {
    let actions = get_ai_command_bar_actions();
    let a = actions
        .iter()
        .find(|a| a.id == "toggle_shortcuts_help")
        .unwrap();
    assert_eq!(a.section.as_deref(), Some("Help"));
}

#[test]
fn batch21_ai_change_model_no_shortcut() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "change_model").unwrap();
    assert!(a.shortcut.is_none());
}

#[test]
fn batch21_ai_branch_from_last_no_shortcut() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "branch_from_last").unwrap();
    assert!(a.shortcut.is_none());
}

// ============================================================
// 9. Chat context: clear_conversation conditional
// ============================================================

#[test]
fn batch21_chat_clear_absent_no_messages() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "clear_conversation"));
}

#[test]
fn batch21_chat_clear_present_with_messages() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "clear_conversation"));
}

#[test]
fn batch21_chat_clear_shortcut() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let a = actions
        .iter()
        .find(|a| a.id == "clear_conversation")
        .unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌘⌫"));
}
