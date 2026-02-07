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
