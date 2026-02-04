//! Batch 28: Builtin action validation tests
//!
//! 118 tests across 30 categories validating built-in action window dialog behaviors.

use super::builders::*;
use super::command_bar::CommandBarConfig;
use super::types::*;
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;

// =============================================================================
// Category 1: Scriptlet context — copy_content shortcut and description
// =============================================================================

#[test]
fn cat28_01_scriptlet_copy_content_shortcut() {
    let script = ScriptInfo::scriptlet("My Snippet", "/p.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert_eq!(cc.shortcut.as_deref(), Some("⌘⌥C"));
}

#[test]
fn cat28_01_scriptlet_copy_content_description() {
    let script = ScriptInfo::scriptlet("My Snippet", "/p.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert!(cc
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("content"));
}

#[test]
fn cat28_01_scriptlet_copy_content_title() {
    let script = ScriptInfo::scriptlet("My Snippet", "/p.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert_eq!(cc.title, "Copy Content");
}

#[test]
fn cat28_01_scriptlet_copy_content_has_action_false() {
    let script = ScriptInfo::scriptlet("X", "/p.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert!(!cc.has_action);
}

// =============================================================================
// Category 2: Scriptlet context — alias actions share shortcut with script context
// =============================================================================

#[test]
fn cat28_02_scriptlet_add_alias_shortcut() {
    let script = ScriptInfo::scriptlet("S", "/p.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let aa = actions.iter().find(|a| a.id == "add_alias").unwrap();
    assert_eq!(aa.shortcut.as_deref(), Some("⌘⇧A"));
}

#[test]
fn cat28_02_scriptlet_update_alias_when_alias_present() {
    let script = ScriptInfo::scriptlet("S", "/p.md", None, Some("al".into()));
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(actions.iter().any(|a| a.id == "update_alias"));
    assert!(!actions.iter().any(|a| a.id == "add_alias"));
}

#[test]
fn cat28_02_scriptlet_remove_alias_shortcut() {
    let script = ScriptInfo::scriptlet("S", "/p.md", None, Some("al".into()));
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let ra = actions.iter().find(|a| a.id == "remove_alias").unwrap();
    assert_eq!(ra.shortcut.as_deref(), Some("⌘⌥A"));
}

#[test]
fn cat28_02_scriptlet_update_alias_shortcut() {
    let script = ScriptInfo::scriptlet("S", "/p.md", None, Some("al".into()));
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let ua = actions.iter().find(|a| a.id == "update_alias").unwrap();
    assert_eq!(ua.shortcut.as_deref(), Some("⌘⇧A"));
}

// =============================================================================
// Category 3: Script context — copy_content and copy_path shortcuts
// =============================================================================

#[test]
fn cat28_03_script_copy_content_shortcut() {
    let script = ScriptInfo::new("my-script", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert_eq!(cc.shortcut.as_deref(), Some("⌘⌥C"));
}

#[test]
fn cat28_03_script_copy_path_shortcut() {
    let script = ScriptInfo::new("my-script", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
    assert_eq!(cp.shortcut.as_deref(), Some("⌘⇧C"));
}

#[test]
fn cat28_03_script_edit_shortcut() {
    let script = ScriptInfo::new("my-script", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let es = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert_eq!(es.shortcut.as_deref(), Some("⌘E"));
}

#[test]
fn cat28_03_script_reveal_shortcut() {
    let script = ScriptInfo::new("my-script", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let rv = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
    assert_eq!(rv.shortcut.as_deref(), Some("⌘⇧F"));
}

// =============================================================================
// Category 4: Script context — view_logs details
// =============================================================================

#[test]
fn cat28_04_script_view_logs_shortcut() {
    let script = ScriptInfo::new("my-script", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let vl = actions.iter().find(|a| a.id == "view_logs").unwrap();
    assert_eq!(vl.shortcut.as_deref(), Some("⌘L"));
}

#[test]
fn cat28_04_script_view_logs_title() {
    let script = ScriptInfo::new("my-script", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let vl = actions.iter().find(|a| a.id == "view_logs").unwrap();
    assert_eq!(vl.title, "View Logs");
}

#[test]
fn cat28_04_script_view_logs_description() {
    let script = ScriptInfo::new("my-script", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let vl = actions.iter().find(|a| a.id == "view_logs").unwrap();
    assert!(vl
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("log"));
}

#[test]
fn cat28_04_view_logs_absent_for_builtin() {
    let builtin = ScriptInfo::builtin("App Launcher");
    let actions = get_script_context_actions(&builtin);
    assert!(!actions.iter().any(|a| a.id == "view_logs"));
}

// =============================================================================
// Category 5: Clipboard — pinned entry produces clipboard_unpin
// =============================================================================

#[test]
fn cat28_05_pinned_entry_has_unpin() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: true,
        preview: "test".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_unpin"));
    assert!(!actions.iter().any(|a| a.id == "clipboard_pin"));
}

#[test]
fn cat28_05_unpinned_entry_has_pin() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_pin"));
    assert!(!actions.iter().any(|a| a.id == "clipboard_unpin"));
}

#[test]
fn cat28_05_pin_unpin_same_shortcut() {
    let pinned = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: true,
        preview: "test".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let unpinned = ClipboardEntryInfo {
        id: "2".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let a1 = get_clipboard_history_context_actions(&pinned);
    let a2 = get_clipboard_history_context_actions(&unpinned);
    let s1 = a1
        .iter()
        .find(|a| a.id == "clipboard_unpin")
        .unwrap()
        .shortcut
        .as_deref();
    let s2 = a2
        .iter()
        .find(|a| a.id == "clipboard_pin")
        .unwrap()
        .shortcut
        .as_deref();
    assert_eq!(s1, s2);
    assert_eq!(s1, Some("⇧⌘P"));
}

// =============================================================================
// Category 6: Clipboard — save_snippet title and description
// =============================================================================

#[test]
fn cat28_06_save_snippet_title() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ss = actions
        .iter()
        .find(|a| a.id == "clipboard_save_snippet")
        .unwrap();
    assert_eq!(ss.title, "Save Text as Snippet");
}

#[test]
fn cat28_06_save_snippet_description_mentions_scriptlet() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ss = actions
        .iter()
        .find(|a| a.id == "clipboard_save_snippet")
        .unwrap();
    assert!(ss
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("scriptlet"));
}

#[test]
fn cat28_06_save_file_title() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let sf = actions
        .iter()
        .find(|a| a.id == "clipboard_save_file")
        .unwrap();
    assert_eq!(sf.title, "Save as File...");
}

// =============================================================================
// Category 7: Clipboard — clipboard_copy description mentions "without pasting"
// =============================================================================

#[test]
fn cat28_07_clipboard_copy_desc_without_pasting() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let cc = actions.iter().find(|a| a.id == "clipboard_copy").unwrap();
    assert!(cc
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("without pasting"));
}

#[test]
fn cat28_07_clipboard_paste_desc_mentions_paste() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let cp = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
    assert!(cp
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("paste"));
}

#[test]
fn cat28_07_clipboard_share_desc_mentions_share() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let cs = actions.iter().find(|a| a.id == "clipboard_share").unwrap();
    assert!(cs
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("share"));
}

// =============================================================================
// Category 8: File context — title includes quoted file name
// =============================================================================

#[test]
fn cat28_08_file_open_title_quotes_name() {
    let file_info = FileInfo {
        path: "/Users/test/readme.txt".into(),
        name: "readme.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file_info);
    let open = actions.iter().find(|a| a.id == "open_file").unwrap();
    assert!(open.title.contains("\"readme.txt\""));
}

#[test]
fn cat28_08_dir_open_title_quotes_name() {
    let file_info = FileInfo {
        path: "/Users/test/Documents".into(),
        name: "Documents".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&file_info);
    let open = actions.iter().find(|a| a.id == "open_directory").unwrap();
    assert!(open.title.contains("\"Documents\""));
}

#[test]
fn cat28_08_file_open_desc_says_application() {
    let file_info = FileInfo {
        path: "/Users/test/readme.txt".into(),
        name: "readme.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file_info);
    let open = actions.iter().find(|a| a.id == "open_file").unwrap();
    assert!(open
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("application"));
}

#[test]
fn cat28_08_dir_open_desc_says_folder() {
    let file_info = FileInfo {
        path: "/Users/test/Documents".into(),
        name: "Documents".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&file_info);
    let open = actions.iter().find(|a| a.id == "open_directory").unwrap();
    assert!(open
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("folder"));
}

// =============================================================================
// Category 9: File context — reveal_in_finder description
// =============================================================================

#[test]
fn cat28_09_file_reveal_desc_mentions_finder() {
    let file_info = FileInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file_info);
    let rv = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
    assert!(rv
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("finder"));
}

#[test]
fn cat28_09_file_copy_path_desc_mentions_clipboard() {
    let file_info = FileInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file_info);
    let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
    assert!(cp
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("clipboard"));
}

#[test]
fn cat28_09_file_copy_filename_desc_mentions_filename() {
    let file_info = FileInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file_info);
    let cf = actions.iter().find(|a| a.id == "copy_filename").unwrap();
    assert!(cf
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("filename"));
}

// =============================================================================
// Category 10: Path context — move_to_trash description dynamic for dir vs file
// =============================================================================

#[test]
fn cat28_10_path_trash_desc_dir_says_folder() {
    let path_info = PathInfo {
        name: "MyDir".into(),
        path: "/test/MyDir".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path_info);
    let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
    assert!(trash
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("folder"));
}

#[test]
fn cat28_10_path_trash_desc_file_says_file() {
    let path_info = PathInfo {
        name: "doc.txt".into(),
        path: "/test/doc.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
    assert!(trash
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("file"));
}

#[test]
fn cat28_10_path_trash_shortcut() {
    let path_info = PathInfo {
        name: "doc.txt".into(),
        path: "/test/doc.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
    assert_eq!(trash.shortcut.as_deref(), Some("⌘⌫"));
}

#[test]
fn cat28_10_path_trash_title() {
    let path_info = PathInfo {
        name: "doc.txt".into(),
        path: "/test/doc.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
    assert_eq!(trash.title, "Move to Trash");
}

// =============================================================================
// Category 11: Path context — select_file / open_directory title includes quoted name
// =============================================================================

#[test]
fn cat28_11_path_select_file_title_quotes_name() {
    let path_info = PathInfo {
        name: "report.pdf".into(),
        path: "/test/report.pdf".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    let sel = actions.iter().find(|a| a.id == "select_file").unwrap();
    assert!(sel.title.contains("\"report.pdf\""));
}

#[test]
fn cat28_11_path_open_dir_title_quotes_name() {
    let path_info = PathInfo {
        name: "Projects".into(),
        path: "/test/Projects".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path_info);
    let od = actions.iter().find(|a| a.id == "open_directory").unwrap();
    assert!(od.title.contains("\"Projects\""));
}

#[test]
fn cat28_11_path_select_desc_says_submit() {
    let path_info = PathInfo {
        name: "file.txt".into(),
        path: "/test/file.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    let sel = actions.iter().find(|a| a.id == "select_file").unwrap();
    assert!(sel
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("submit"));
}

#[test]
fn cat28_11_path_open_dir_desc_says_navigate() {
    let path_info = PathInfo {
        name: "Projects".into(),
        path: "/test/Projects".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path_info);
    let od = actions.iter().find(|a| a.id == "open_directory").unwrap();
    assert!(od
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("navigate"));
}

// =============================================================================
// Category 12: AI command bar — toggle_shortcuts_help details
// =============================================================================

#[test]
fn cat28_12_ai_toggle_shortcuts_shortcut() {
    let actions = get_ai_command_bar_actions();
    let tsh = actions
        .iter()
        .find(|a| a.id == "toggle_shortcuts_help")
        .unwrap();
    assert_eq!(tsh.shortcut.as_deref(), Some("⌘/"));
}

#[test]
fn cat28_12_ai_toggle_shortcuts_icon() {
    let actions = get_ai_command_bar_actions();
    let tsh = actions
        .iter()
        .find(|a| a.id == "toggle_shortcuts_help")
        .unwrap();
    assert_eq!(tsh.icon, Some(IconName::Star));
}

#[test]
fn cat28_12_ai_toggle_shortcuts_section() {
    let actions = get_ai_command_bar_actions();
    let tsh = actions
        .iter()
        .find(|a| a.id == "toggle_shortcuts_help")
        .unwrap();
    assert_eq!(tsh.section.as_deref(), Some("Help"));
}

#[test]
fn cat28_12_ai_toggle_shortcuts_title() {
    let actions = get_ai_command_bar_actions();
    let tsh = actions
        .iter()
        .find(|a| a.id == "toggle_shortcuts_help")
        .unwrap();
    assert_eq!(tsh.title, "Keyboard Shortcuts");
}

// =============================================================================
// Category 13: AI command bar — new_chat details
// =============================================================================

#[test]
fn cat28_13_ai_new_chat_shortcut() {
    let actions = get_ai_command_bar_actions();
    let nc = actions.iter().find(|a| a.id == "new_chat").unwrap();
    assert_eq!(nc.shortcut.as_deref(), Some("⌘N"));
}

#[test]
fn cat28_13_ai_new_chat_icon() {
    let actions = get_ai_command_bar_actions();
    let nc = actions.iter().find(|a| a.id == "new_chat").unwrap();
    assert_eq!(nc.icon, Some(IconName::Plus));
}

#[test]
fn cat28_13_ai_new_chat_section() {
    let actions = get_ai_command_bar_actions();
    let nc = actions.iter().find(|a| a.id == "new_chat").unwrap();
    assert_eq!(nc.section.as_deref(), Some("Actions"));
}

#[test]
fn cat28_13_ai_new_chat_desc_mentions_new() {
    let actions = get_ai_command_bar_actions();
    let nc = actions.iter().find(|a| a.id == "new_chat").unwrap();
    assert!(nc
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("new"));
}

// =============================================================================
// Category 14: AI command bar — delete_chat details
// =============================================================================

#[test]
fn cat28_14_ai_delete_chat_shortcut() {
    let actions = get_ai_command_bar_actions();
    let dc = actions.iter().find(|a| a.id == "delete_chat").unwrap();
    assert_eq!(dc.shortcut.as_deref(), Some("⌘⌫"));
}

#[test]
fn cat28_14_ai_delete_chat_icon() {
    let actions = get_ai_command_bar_actions();
    let dc = actions.iter().find(|a| a.id == "delete_chat").unwrap();
    assert_eq!(dc.icon, Some(IconName::Trash));
}

#[test]
fn cat28_14_ai_delete_chat_section() {
    let actions = get_ai_command_bar_actions();
    let dc = actions.iter().find(|a| a.id == "delete_chat").unwrap();
    assert_eq!(dc.section.as_deref(), Some("Actions"));
}

#[test]
fn cat28_14_ai_delete_chat_desc_mentions_delete() {
    let actions = get_ai_command_bar_actions();
    let dc = actions.iter().find(|a| a.id == "delete_chat").unwrap();
    assert!(dc
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("delete"));
}

// =============================================================================
// Category 15: Chat context — continue_in_chat shortcut
// =============================================================================

#[test]
fn cat28_15_chat_continue_in_chat_shortcut() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let cic = actions.iter().find(|a| a.id == "continue_in_chat").unwrap();
    assert_eq!(cic.shortcut.as_deref(), Some("⌘↵"));
}

#[test]
fn cat28_15_chat_continue_in_chat_desc_mentions_chat() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let cic = actions.iter().find(|a| a.id == "continue_in_chat").unwrap();
    assert!(cic
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("chat"));
}

#[test]
fn cat28_15_chat_continue_always_present() {
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
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "continue_in_chat"));
}

// =============================================================================
// Category 16: Chat context — clear_conversation details
// =============================================================================

#[test]
fn cat28_16_chat_clear_shortcut() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let clr = actions
        .iter()
        .find(|a| a.id == "clear_conversation")
        .unwrap();
    assert_eq!(clr.shortcut.as_deref(), Some("⌘⌫"));
}

#[test]
fn cat28_16_chat_clear_absent_when_no_messages() {
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
fn cat28_16_chat_clear_desc_mentions_clear() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let clr = actions
        .iter()
        .find(|a| a.id == "clear_conversation")
        .unwrap();
    assert!(clr
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("clear"));
}

#[test]
fn cat28_16_chat_copy_response_absent_when_no_response() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "copy_response"));
}

// =============================================================================
// Category 17: Notes — export section icon
// =============================================================================

#[test]
fn cat28_17_notes_export_icon() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let exp = actions.iter().find(|a| a.id == "export").unwrap();
    assert_eq!(exp.icon, Some(IconName::ArrowRight));
}

#[test]
fn cat28_17_notes_export_section() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let exp = actions.iter().find(|a| a.id == "export").unwrap();
    assert_eq!(exp.section.as_deref(), Some("Export"));
}

#[test]
fn cat28_17_notes_export_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let exp = actions.iter().find(|a| a.id == "export").unwrap();
    assert_eq!(exp.shortcut.as_deref(), Some("⇧⌘E"));
}

#[test]
fn cat28_17_notes_export_absent_without_selection() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "export"));
}

// =============================================================================
// Category 18: Notes — browse_notes details
// =============================================================================

#[test]
fn cat28_18_notes_browse_shortcut() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let bn = actions.iter().find(|a| a.id == "browse_notes").unwrap();
    assert_eq!(bn.shortcut.as_deref(), Some("⌘P"));
}

#[test]
fn cat28_18_notes_browse_icon() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let bn = actions.iter().find(|a| a.id == "browse_notes").unwrap();
    assert_eq!(bn.icon, Some(IconName::FolderOpen));
}

#[test]
fn cat28_18_notes_browse_section() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let bn = actions.iter().find(|a| a.id == "browse_notes").unwrap();
    assert_eq!(bn.section.as_deref(), Some("Notes"));
}

#[test]
fn cat28_18_notes_browse_always_present() {
    // Present even in trash view
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "browse_notes"));
}

// =============================================================================
// Category 19: Notes full mode action count
// =============================================================================

#[test]
fn cat28_19_notes_full_mode_count() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note, duplicate, browse, find, format, copy_note_as, copy_deeplink, create_quicklink, export, auto_sizing
    assert_eq!(actions.len(), 10);
}

#[test]
fn cat28_19_notes_full_auto_sizing_enabled_count() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    // Same minus auto_sizing = 9
    assert_eq!(actions.len(), 9);
}

#[test]
fn cat28_19_notes_no_selection_count() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note, browse, auto_sizing = 3
    assert_eq!(actions.len(), 3);
}

#[test]
fn cat28_19_notes_trash_view_count() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note, browse, auto_sizing = 3 (trash hides duplicate, edit, copy sections)
    assert_eq!(actions.len(), 3);
}

// =============================================================================
// Category 20: Note switcher — pinned note icon and section
// =============================================================================

#[test]
fn cat28_20_note_switcher_pinned_icon() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "Pinned Note".into(),
        char_count: 100,
        is_current: false,
        is_pinned: true,
        preview: "Some content".into(),
        relative_time: "1d ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

#[test]
fn cat28_20_note_switcher_pinned_section() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "Pinned Note".into(),
        char_count: 100,
        is_current: false,
        is_pinned: true,
        preview: "Some content".into(),
        relative_time: "1d ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
}

#[test]
fn cat28_20_note_switcher_regular_icon() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "Regular Note".into(),
        char_count: 50,
        is_current: false,
        is_pinned: false,
        preview: "test".into(),
        relative_time: "2h ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::File));
}

#[test]
fn cat28_20_note_switcher_regular_section() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "Regular Note".into(),
        char_count: 50,
        is_current: false,
        is_pinned: false,
        preview: "test".into(),
        relative_time: "2h ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].section.as_deref(), Some("Recent"));
}

// =============================================================================
// Category 21: Note switcher — current note icon and title prefix
// =============================================================================

#[test]
fn cat28_21_note_switcher_current_icon() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "xyz".into(),
        title: "Current Note".into(),
        char_count: 200,
        is_current: true,
        is_pinned: false,
        preview: "body".into(),
        relative_time: "5m ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::Check));
}

#[test]
fn cat28_21_note_switcher_current_bullet_prefix() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "xyz".into(),
        title: "Current Note".into(),
        char_count: 200,
        is_current: true,
        is_pinned: false,
        preview: "body".into(),
        relative_time: "5m ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(actions[0].title.starts_with("• "));
}

#[test]
fn cat28_21_note_switcher_non_current_no_bullet() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "xyz".into(),
        title: "Other Note".into(),
        char_count: 100,
        is_current: false,
        is_pinned: false,
        preview: "test".into(),
        relative_time: "1h ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(!actions[0].title.starts_with("• "));
}

#[test]
fn cat28_21_pinned_trumps_current_for_icon() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "xyz".into(),
        title: "Both".into(),
        char_count: 100,
        is_current: true,
        is_pinned: true,
        preview: "test".into(),
        relative_time: "1h ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

// =============================================================================
// Category 22: New chat — model description is provider_display_name
// =============================================================================

#[test]
fn cat28_22_new_chat_model_description() {
    let models = vec![NewChatModelInfo {
        model_id: "claude-3-opus".into(),
        display_name: "Claude 3 Opus".into(),
        provider: "anthropic".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    let model_action = actions.iter().find(|a| a.id == "model_0").unwrap();
    assert_eq!(model_action.description.as_deref(), Some("Anthropic"));
}

#[test]
fn cat28_22_new_chat_model_icon() {
    let models = vec![NewChatModelInfo {
        model_id: "gpt4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    let model_action = actions.iter().find(|a| a.id == "model_0").unwrap();
    assert_eq!(model_action.icon, Some(IconName::Settings));
}

#[test]
fn cat28_22_new_chat_model_section() {
    let models = vec![NewChatModelInfo {
        model_id: "gpt4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    let model_action = actions.iter().find(|a| a.id == "model_0").unwrap();
    assert_eq!(model_action.section.as_deref(), Some("Models"));
}

#[test]
fn cat28_22_new_chat_model_title_is_display_name() {
    let models = vec![NewChatModelInfo {
        model_id: "gpt4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    let model_action = actions.iter().find(|a| a.id == "model_0").unwrap();
    assert_eq!(model_action.title, "GPT-4");
}

// =============================================================================
// Category 23: New chat — preset description is None
// =============================================================================

#[test]
fn cat28_23_new_chat_preset_description_none() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    let preset = actions.iter().find(|a| a.id == "preset_general").unwrap();
    assert!(preset.description.is_none());
}

#[test]
fn cat28_23_new_chat_preset_icon_preserved() {
    let presets = vec![NewChatPresetInfo {
        id: "code".into(),
        name: "Code".into(),
        icon: IconName::Code,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    let preset = actions.iter().find(|a| a.id == "preset_code").unwrap();
    assert_eq!(preset.icon, Some(IconName::Code));
}

#[test]
fn cat28_23_new_chat_preset_section() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    let preset = actions.iter().find(|a| a.id == "preset_general").unwrap();
    assert_eq!(preset.section.as_deref(), Some("Presets"));
}

#[test]
fn cat28_23_new_chat_preset_title() {
    let presets = vec![NewChatPresetInfo {
        id: "writer".into(),
        name: "Writer".into(),
        icon: IconName::File,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    let preset = actions.iter().find(|a| a.id == "preset_writer").unwrap();
    assert_eq!(preset.title, "Writer");
}

// =============================================================================
// Category 24: format_shortcut_hint (builders.rs version) — simple transforms
// =============================================================================

#[test]
fn cat28_24_builders_format_hint_cmd_c() {
    assert_eq!(super::builders::to_deeplink_name("cmd+c"), "cmd-c");
}

#[test]
fn cat28_24_to_deeplink_name_basic() {
    assert_eq!(to_deeplink_name("My Script"), "my-script");
}

#[test]
fn cat28_24_to_deeplink_name_underscores() {
    assert_eq!(to_deeplink_name("hello_world"), "hello-world");
}

#[test]
fn cat28_24_to_deeplink_name_empty() {
    assert_eq!(to_deeplink_name(""), "");
}

// =============================================================================
// Category 25: Action with_shortcut_opt: None vs Some
// =============================================================================

#[test]
fn cat28_25_with_shortcut_opt_some() {
    let action = Action::new("a", "A", None, ActionCategory::ScriptContext)
        .with_shortcut_opt(Some("⌘A".into()));
    assert_eq!(action.shortcut.as_deref(), Some("⌘A"));
    assert_eq!(action.shortcut_lower.as_deref(), Some("⌘a"));
}

#[test]
fn cat28_25_with_shortcut_opt_none() {
    let action = Action::new("a", "A", None, ActionCategory::ScriptContext).with_shortcut_opt(None);
    assert!(action.shortcut.is_none());
    assert!(action.shortcut_lower.is_none());
}

#[test]
fn cat28_25_with_shortcut_sets_lower() {
    let action = Action::new("a", "A", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧C");
    assert_eq!(action.shortcut_lower.as_deref(), Some("⌘⇧c"));
}

#[test]
fn cat28_25_action_new_no_shortcut_lower() {
    let action = Action::new("a", "A", None, ActionCategory::ScriptContext);
    assert!(action.shortcut_lower.is_none());
}

// =============================================================================
// Category 26: Action with_icon and with_section
// =============================================================================

#[test]
fn cat28_26_with_icon_sets_field() {
    let action =
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_icon(IconName::Copy);
    assert_eq!(action.icon, Some(IconName::Copy));
}

#[test]
fn cat28_26_action_new_no_icon() {
    let action = Action::new("a", "A", None, ActionCategory::ScriptContext);
    assert!(action.icon.is_none());
}

#[test]
fn cat28_26_with_section_sets_field() {
    let action =
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("My Section");
    assert_eq!(action.section.as_deref(), Some("My Section"));
}

#[test]
fn cat28_26_action_new_no_section() {
    let action = Action::new("a", "A", None, ActionCategory::ScriptContext);
    assert!(action.section.is_none());
}

// =============================================================================
// Category 27: Action cached lowercase fields
// =============================================================================

#[test]
fn cat28_27_title_lower_computed() {
    let action = Action::new("a", "Hello World", None, ActionCategory::ScriptContext);
    assert_eq!(action.title_lower, "hello world");
}

#[test]
fn cat28_27_description_lower_computed() {
    let action = Action::new(
        "a",
        "A",
        Some("Some Description".into()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(
        action.description_lower.as_deref(),
        Some("some description")
    );
}

#[test]
fn cat28_27_description_lower_none_when_no_desc() {
    let action = Action::new("a", "A", None, ActionCategory::ScriptContext);
    assert!(action.description_lower.is_none());
}

#[test]
fn cat28_27_shortcut_lower_after_with_shortcut() {
    let action = Action::new("a", "A", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧D");
    assert_eq!(action.shortcut_lower.as_deref(), Some("⌘⇧d"));
}

// =============================================================================
// Category 28: CommandBarConfig presets — dialog_config fields
// =============================================================================

#[test]
fn cat28_28_main_menu_search_bottom() {
    let cfg = CommandBarConfig::main_menu_style();
    assert_eq!(cfg.dialog_config.search_position, SearchPosition::Bottom);
}

#[test]
fn cat28_28_ai_style_search_top() {
    let cfg = CommandBarConfig::ai_style();
    assert_eq!(cfg.dialog_config.search_position, SearchPosition::Top);
}

#[test]
fn cat28_28_no_search_hidden() {
    let cfg = CommandBarConfig::no_search();
    assert_eq!(cfg.dialog_config.search_position, SearchPosition::Hidden);
}

#[test]
fn cat28_28_notes_style_search_top() {
    let cfg = CommandBarConfig::notes_style();
    assert_eq!(cfg.dialog_config.search_position, SearchPosition::Top);
}

// =============================================================================
// Category 29: Cross-context — action ID uniqueness within each context
// =============================================================================

#[test]
fn cat28_29_script_ids_unique() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
    assert_eq!(ids.len(), unique.len(), "Duplicate IDs found: {:?}", ids);
}

#[test]
fn cat28_29_clipboard_ids_unique() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
    assert_eq!(ids.len(), unique.len(), "Duplicate IDs found: {:?}", ids);
}

#[test]
fn cat28_29_ai_ids_unique() {
    let actions = get_ai_command_bar_actions();
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
    assert_eq!(ids.len(), unique.len(), "Duplicate IDs found: {:?}", ids);
}

#[test]
fn cat28_29_path_ids_unique() {
    let path_info = PathInfo {
        name: "test".into(),
        path: "/test".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
    assert_eq!(ids.len(), unique.len(), "Duplicate IDs found: {:?}", ids);
}

// =============================================================================
// Category 30: Cross-context — all contexts produce non-empty title and id
// =============================================================================

#[test]
fn cat28_30_script_actions_non_empty_titles() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    for a in &actions {
        assert!(!a.id.is_empty(), "Action ID must not be empty");
        assert!(!a.title.is_empty(), "Action title must not be empty");
    }
}

#[test]
fn cat28_30_clipboard_actions_non_empty_titles() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    for a in &actions {
        assert!(!a.id.is_empty(), "Action ID must not be empty");
        assert!(!a.title.is_empty(), "Action title must not be empty");
    }
}

#[test]
fn cat28_30_notes_actions_non_empty_titles() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    for a in &actions {
        assert!(!a.id.is_empty(), "Action ID must not be empty");
        assert!(!a.title.is_empty(), "Action title must not be empty");
    }
}

#[test]
fn cat28_30_file_actions_non_empty_titles() {
    let file_info = FileInfo {
        path: "/test/f.txt".into(),
        name: "f.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file_info);
    for a in &actions {
        assert!(!a.id.is_empty(), "Action ID must not be empty");
        assert!(!a.title.is_empty(), "Action title must not be empty");
    }
}

#[test]
fn cat28_30_ai_actions_non_empty_titles() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert!(!a.id.is_empty(), "Action ID must not be empty");
        assert!(!a.title.is_empty(), "Action title must not be empty");
    }
}

#[test]
fn cat28_30_path_actions_non_empty_titles() {
    let path_info = PathInfo {
        name: "test".into(),
        path: "/test".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path_info);
    for a in &actions {
        assert!(!a.id.is_empty(), "Action ID must not be empty");
        assert!(!a.title.is_empty(), "Action title must not be empty");
    }
}
