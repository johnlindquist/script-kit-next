// --- merged from part_01.rs ---
//! Batch 45: Dialog Built-in Action Validation Tests
//!
//! 120 tests across 30 categories validating action behaviors
//! in various built-in action window dialogs.

use crate::actions::builders::*;
use crate::actions::dialog::ActionsDialog;
use crate::actions::types::{Action, ActionCategory, ScriptInfo};
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;
use crate::scriptlets::{Scriptlet, ScriptletAction};

// =========== 1. ScriptInfo::with_shortcut_and_alias: both populated ===========

#[test]
fn with_shortcut_and_alias_sets_shortcut() {
    let s =
        ScriptInfo::with_shortcut_and_alias("test", "/p", Some("cmd+k".into()), Some("tk".into()));
    assert_eq!(s.shortcut, Some("cmd+k".to_string()));
}

#[test]
fn with_shortcut_and_alias_sets_alias() {
    let s =
        ScriptInfo::with_shortcut_and_alias("test", "/p", Some("cmd+k".into()), Some("tk".into()));
    assert_eq!(s.alias, Some("tk".to_string()));
}

#[test]
fn with_shortcut_and_alias_is_script_true() {
    let s =
        ScriptInfo::with_shortcut_and_alias("test", "/p", Some("cmd+k".into()), Some("tk".into()));
    assert!(s.is_script);
}

#[test]
fn with_shortcut_and_alias_is_scriptlet_false() {
    let s =
        ScriptInfo::with_shortcut_and_alias("test", "/p", Some("cmd+k".into()), Some("tk".into()));
    assert!(!s.is_scriptlet);
}

// =========== 2. ScriptInfo: with_frecency on scriptlet preserves type ===========

#[test]
fn scriptlet_with_frecency_preserves_is_scriptlet() {
    let s = ScriptInfo::scriptlet("Open URL", "/urls.md", None, None)
        .with_frecency(true, Some("/f".into()));
    assert!(s.is_scriptlet);
}

#[test]
fn scriptlet_with_frecency_is_script_stays_false() {
    let s = ScriptInfo::scriptlet("Open URL", "/urls.md", None, None)
        .with_frecency(true, Some("/f".into()));
    assert!(!s.is_script);
}

#[test]
fn scriptlet_with_frecency_preserves_name() {
    let s = ScriptInfo::scriptlet("Open URL", "/urls.md", None, None)
        .with_frecency(true, Some("/f".into()));
    assert_eq!(s.name, "Open URL");
}

#[test]
fn scriptlet_with_frecency_sets_is_suggested() {
    let s = ScriptInfo::scriptlet("Open URL", "/urls.md", None, None)
        .with_frecency(true, Some("/f".into()));
    assert!(s.is_suggested);
}

// =========== 3. Action: category preserved through builder chaining ===========

#[test]
fn action_category_preserved_after_with_shortcut() {
    let a = Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut("⌘T");
    assert_eq!(a.category, ActionCategory::ScriptContext);
}

#[test]
fn action_category_preserved_after_with_icon() {
    let a =
        Action::new("test", "Test", None, ActionCategory::ScriptContext).with_icon(IconName::Star);
    assert_eq!(a.category, ActionCategory::ScriptContext);
}

#[test]
fn action_category_preserved_after_with_section() {
    let a =
        Action::new("test", "Test", None, ActionCategory::ScriptContext).with_section("Actions");
    assert_eq!(a.category, ActionCategory::ScriptContext);
}

#[test]
fn action_category_preserved_after_full_chain() {
    let a = Action::new("test", "Test", None, ActionCategory::ScriptContext)
        .with_shortcut("⌘T")
        .with_icon(IconName::Star)
        .with_section("Actions");
    assert_eq!(a.category, ActionCategory::ScriptContext);
}

// =========== 4. Action: with_icon returns expected icon value ===========

#[test]
fn action_with_icon_star_filled() {
    let a = Action::new("test", "Test", None, ActionCategory::ScriptContext)
        .with_icon(IconName::StarFilled);
    assert_eq!(a.icon, Some(IconName::StarFilled));
}

#[test]
fn action_with_icon_plus() {
    let a =
        Action::new("test", "Test", None, ActionCategory::ScriptContext).with_icon(IconName::Plus);
    assert_eq!(a.icon, Some(IconName::Plus));
}

#[test]
fn action_with_icon_settings() {
    let a = Action::new("test", "Test", None, ActionCategory::ScriptContext)
        .with_icon(IconName::Settings);
    assert_eq!(a.icon, Some(IconName::Settings));
}

#[test]
fn action_without_icon_is_none() {
    let a = Action::new("test", "Test", None, ActionCategory::ScriptContext);
    assert!(a.icon.is_none());
}

// =========== 5. Clipboard: first 4 text action IDs in order ===========

#[test]
fn clipboard_text_first_action_is_paste() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert_eq!(actions[0].id, "clipboard_paste");
}

#[test]
fn clipboard_text_second_action_is_copy() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert_eq!(actions[1].id, "clipboard_copy");
}

#[test]
fn clipboard_text_third_action_is_paste_keep_open() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert_eq!(actions[2].id, "clipboard_paste_keep_open");
}

#[test]
fn clipboard_text_fourth_action_is_share() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert_eq!(actions[3].id, "clipboard_share");
}

// =========== 6. Clipboard: save_snippet desc mentions "scriptlet" ===========

#[test]
fn clipboard_save_snippet_desc_mentions_scriptlet() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let action = actions
        .iter()
        .find(|a| a.id == "clipboard_save_snippet")
        .unwrap();
    assert!(action.description.as_ref().unwrap().contains("scriptlet"));
}

#[test]
fn clipboard_save_file_desc_mentions_file() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let action = actions
        .iter()
        .find(|a| a.id == "clipboard_save_file")
        .unwrap();
    assert!(action.description.as_ref().unwrap().contains("file"));
}

#[test]
fn clipboard_save_snippet_shortcut_differs_from_save_file() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let snippet = actions
        .iter()
        .find(|a| a.id == "clipboard_save_snippet")
        .unwrap();
    let file = actions
        .iter()
        .find(|a| a.id == "clipboard_save_file")
        .unwrap();
    assert_ne!(snippet.shortcut, file.shortcut);
}

#[test]
fn clipboard_save_snippet_and_save_file_both_present() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_save_snippet"));
    assert!(actions.iter().any(|a| a.id == "clipboard_save_file"));
}

// =========== 7. Clipboard: destructive actions in last 3 positions ===========

#[test]
fn clipboard_delete_is_third_from_last() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let len = actions.len();
    assert_eq!(actions[len - 3].id, "clipboard_delete");
}

#[test]
fn clipboard_delete_multiple_is_second_from_last() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let len = actions.len();
    assert_eq!(actions[len - 2].id, "clipboard_delete_multiple");
}

#[test]
fn clipboard_delete_all_is_last() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let last = actions.last().unwrap();
    assert_eq!(last.id, "clipboard_delete_all");
}

#[test]
fn clipboard_all_three_destructive_actions_present() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_delete"));
    assert!(actions.iter().any(|a| a.id == "clipboard_delete_multiple"));
    assert!(actions.iter().any(|a| a.id == "clipboard_delete_all"));
}

// =========== 8. Clipboard: annotate_cleanshot image-only ===========

#[test]
fn clipboard_annotate_present_for_image() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: String::new(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions
        .iter()
        .any(|a| a.id == "clipboard_annotate_cleanshot"));
}

#[test]
fn clipboard_annotate_absent_for_text() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions
        .iter()
        .any(|a| a.id == "clipboard_annotate_cleanshot"));
}

#[test]
fn clipboard_annotate_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: String::new(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let action = actions
        .iter()
        .find(|a| a.id == "clipboard_annotate_cleanshot")
        .unwrap();
    assert_eq!(action.shortcut.as_deref(), Some("⇧⌘A"));
}

#[test]
fn clipboard_annotate_desc_mentions_cleanshot() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: String::new(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let action = actions
        .iter()
        .find(|a| a.id == "clipboard_annotate_cleanshot")
        .unwrap();
    assert!(action.description.as_ref().unwrap().contains("CleanShot X"));
}

// =========== 9. File context: macOS file=7 dir=6 action count ===========

#[test]
fn file_context_file_has_7_actions() {
    let file_info = FileInfo {
        name: "test.txt".into(),
        path: "/tmp/test.txt".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&file_info);
    assert_eq!(actions.len(), 7);
}

#[test]
fn file_context_dir_has_6_actions() {
    let file_info = FileInfo {
        name: "mydir".into(),
        path: "/tmp/mydir".into(),
        is_dir: true,
        file_type: FileType::Directory,
    };
    let actions = get_file_context_actions(&file_info);
    assert_eq!(actions.len(), 6);
}

#[test]
fn file_context_file_has_quick_look() {
    let file_info = FileInfo {
        name: "test.txt".into(),
        path: "/tmp/test.txt".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&file_info);
    assert!(actions.iter().any(|a| a.id == "quick_look"));
}

#[test]
fn file_context_dir_has_no_quick_look() {
    let file_info = FileInfo {
        name: "mydir".into(),
        path: "/tmp/mydir".into(),
        is_dir: true,
        file_type: FileType::Directory,
    };
    let actions = get_file_context_actions(&file_info);
    assert!(!actions.iter().any(|a| a.id == "quick_look"));
}

// --- merged from part_02.rs ---

// =========== 10. File context: all ScriptContext category ===========

#[test]
fn file_context_file_all_script_context() {
    let file_info = FileInfo {
        name: "test.txt".into(),
        path: "/tmp/test.txt".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&file_info);
    assert!(actions
        .iter()
        .all(|a| a.category == ActionCategory::ScriptContext));
}

#[test]
fn file_context_dir_all_script_context() {
    let file_info = FileInfo {
        name: "mydir".into(),
        path: "/tmp/mydir".into(),
        is_dir: true,
        file_type: FileType::Directory,
    };
    let actions = get_file_context_actions(&file_info);
    assert!(actions
        .iter()
        .all(|a| a.category == ActionCategory::ScriptContext));
}

#[test]
fn file_context_no_script_ops() {
    let file_info = FileInfo {
        name: "test.txt".into(),
        path: "/tmp/test.txt".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&file_info);
    assert!(!actions
        .iter()
        .any(|a| a.category == ActionCategory::ScriptOps));
}

#[test]
fn file_context_no_global_ops() {
    let file_info = FileInfo {
        name: "test.txt".into(),
        path: "/tmp/test.txt".into(),
        is_dir: false,
        file_type: FileType::File,
    };
    let actions = get_file_context_actions(&file_info);
    assert!(!actions
        .iter()
        .any(|a| a.category == ActionCategory::GlobalOps));
}

// =========== 11. Path context: primary at index 0 ===========

#[test]
fn path_file_primary_at_index_0() {
    let path_info = PathInfo {
        name: "file.txt".into(),
        path: "/tmp/file.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    assert_eq!(actions[0].id, "select_file");
}

#[test]
fn path_dir_primary_at_index_0() {
    let path_info = PathInfo {
        name: "mydir".into(),
        path: "/tmp/mydir".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path_info);
    assert_eq!(actions[0].id, "open_directory");
}

#[test]
fn path_file_copy_path_at_index_1() {
    let path_info = PathInfo {
        name: "file.txt".into(),
        path: "/tmp/file.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    assert_eq!(actions[1].id, "copy_path");
}

#[test]
fn path_dir_copy_path_at_index_1() {
    let path_info = PathInfo {
        name: "mydir".into(),
        path: "/tmp/mydir".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path_info);
    assert_eq!(actions[1].id, "copy_path");
}

// =========== 12. Path context: dir has all 7 IDs ===========

#[test]
fn path_dir_has_open_directory() {
    let path_info = PathInfo {
        name: "d".into(),
        path: "/d".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path_info);
    assert!(actions.iter().any(|a| a.id == "open_directory"));
}

#[test]
fn path_dir_has_copy_path_and_copy_filename() {
    let path_info = PathInfo {
        name: "d".into(),
        path: "/d".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path_info);
    assert!(actions.iter().any(|a| a.id == "copy_path"));
    assert!(actions.iter().any(|a| a.id == "copy_filename"));
}

#[test]
fn path_dir_has_open_in_finder_and_editor() {
    let path_info = PathInfo {
        name: "d".into(),
        path: "/d".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path_info);
    assert!(actions.iter().any(|a| a.id == "open_in_finder"));
    assert!(actions.iter().any(|a| a.id == "open_in_editor"));
}

#[test]
fn path_dir_has_terminal_and_trash() {
    let path_info = PathInfo {
        name: "d".into(),
        path: "/d".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path_info);
    assert!(actions.iter().any(|a| a.id == "open_in_terminal"));
    assert!(actions.iter().any(|a| a.id == "move_to_trash"));
}

// =========== 13. Script: shortcut+alias yields update+remove for both ===========

#[test]
fn script_shortcut_alias_has_update_shortcut() {
    let s =
        ScriptInfo::with_shortcut_and_alias("test", "/p", Some("cmd+t".into()), Some("ts".into()));
    let actions = get_script_context_actions(&s);
    assert!(actions.iter().any(|a| a.id == "update_shortcut"));
}

#[test]
fn script_shortcut_alias_has_update_alias() {
    let s =
        ScriptInfo::with_shortcut_and_alias("test", "/p", Some("cmd+t".into()), Some("ts".into()));
    let actions = get_script_context_actions(&s);
    assert!(actions.iter().any(|a| a.id == "update_alias"));
}

#[test]
fn script_shortcut_alias_has_remove_shortcut() {
    let s =
        ScriptInfo::with_shortcut_and_alias("test", "/p", Some("cmd+t".into()), Some("ts".into()));
    let actions = get_script_context_actions(&s);
    assert!(actions.iter().any(|a| a.id == "remove_shortcut"));
}

#[test]
fn script_shortcut_alias_has_remove_alias() {
    let s =
        ScriptInfo::with_shortcut_and_alias("test", "/p", Some("cmd+t".into()), Some("ts".into()));
    let actions = get_script_context_actions(&s);
    assert!(actions.iter().any(|a| a.id == "remove_alias"));
}

// =========== 14. Script: agent exactly 8 actions ===========

#[test]
fn agent_has_8_actions() {
    let mut s = ScriptInfo::new("my-agent", "/agents/my-agent.md");
    s.is_agent = true;
    s.is_script = false;
    let actions = get_script_context_actions(&s);
    assert_eq!(actions.len(), 8);
}

#[test]
fn agent_has_edit_script() {
    let mut s = ScriptInfo::new("my-agent", "/agents/my-agent.md");
    s.is_agent = true;
    s.is_script = false;
    let actions = get_script_context_actions(&s);
    assert!(actions.iter().any(|a| a.id == "edit_script"));
}

#[test]
fn agent_has_copy_content() {
    let mut s = ScriptInfo::new("my-agent", "/agents/my-agent.md");
    s.is_agent = true;
    s.is_script = false;
    let actions = get_script_context_actions(&s);
    assert!(actions.iter().any(|a| a.id == "copy_content"));
}

#[test]
fn agent_has_copy_deeplink() {
    let mut s = ScriptInfo::new("my-agent", "/agents/my-agent.md");
    s.is_agent = true;
    s.is_script = false;
    let actions = get_script_context_actions(&s);
    assert!(actions.iter().any(|a| a.id == "copy_deeplink"));
}

// =========== 15. Script: get_global_actions empty ===========

#[test]
fn global_actions_returns_empty() {
    let actions = get_global_actions();
    assert!(actions.is_empty());
}

#[test]
fn global_actions_len_zero() {
    let actions = get_global_actions();
    assert_eq!(actions.len(), 0);
}

#[test]
fn global_actions_no_script_context() {
    let actions = get_global_actions();
    assert!(!actions
        .iter()
        .any(|a| a.category == ActionCategory::ScriptContext));
}

#[test]
fn global_actions_no_global_ops() {
    let actions = get_global_actions();
    assert!(!actions
        .iter()
        .any(|a| a.category == ActionCategory::GlobalOps));
}

// =========== 16. Scriptlet with_custom: None scriptlet → no has_action=true ===========

#[test]
fn scriptlet_with_custom_none_first_is_run() {
    let s = ScriptInfo::scriptlet("My Scriptlet", "/s.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    assert_eq!(actions[0].id, "run_script");
}

#[test]
fn scriptlet_with_custom_none_all_has_action_false() {
    let s = ScriptInfo::scriptlet("My Scriptlet", "/s.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    assert!(actions.iter().all(|a| !a.has_action));
}

#[test]
fn scriptlet_with_custom_none_no_scriptlet_action_ids() {
    let s = ScriptInfo::scriptlet("My Scriptlet", "/s.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    assert!(!actions
        .iter()
        .any(|a| a.id.starts_with("scriptlet_action:")));
}

#[test]
fn scriptlet_with_custom_none_has_edit_scriptlet() {
    let s = ScriptInfo::scriptlet("My Scriptlet", "/s.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    assert!(actions.iter().any(|a| a.id == "edit_scriptlet"));
}

// =========== 17. Scriptlet with_custom: copy_content desc ===========

#[test]
fn scriptlet_with_custom_copy_content_desc() {
    let s = ScriptInfo::scriptlet("My Scriptlet", "/s.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    let action = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert!(action
        .description
        .as_ref()
        .unwrap()
        .contains("entire file content"));
}

#[test]
fn scriptlet_with_custom_copy_content_shortcut() {
    let s = ScriptInfo::scriptlet("My Scriptlet", "/s.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    let action = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert_eq!(action.shortcut.as_deref(), Some("⌘⌥C"));
}

#[test]
fn scriptlet_with_custom_copy_content_title() {
    let s = ScriptInfo::scriptlet("My Scriptlet", "/s.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    let action = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert_eq!(action.title, "Copy Content");
}

#[test]
fn scriptlet_with_custom_copy_content_present() {
    let s = ScriptInfo::scriptlet("My Scriptlet", "/s.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    assert!(actions.iter().any(|a| a.id == "copy_content"));
}

// =========== 18. Scriptlet defined: empty actions → empty result ===========

#[test]
fn scriptlet_defined_empty_returns_empty() {
    let scriptlet = Scriptlet::new(
        "test".to_string(),
        "bash".to_string(),
        "echo hi".to_string(),
    );
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert!(actions.is_empty());
}

#[test]
fn scriptlet_defined_empty_len_zero() {
    let scriptlet = Scriptlet::new(
        "test".to_string(),
        "bash".to_string(),
        "echo hi".to_string(),
    );
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert_eq!(actions.len(), 0);
}

#[test]
fn scriptlet_defined_empty_no_has_action() {
    let scriptlet = Scriptlet::new(
        "test".to_string(),
        "bash".to_string(),
        "echo hi".to_string(),
    );
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert!(!actions.iter().any(|a| a.has_action));
}

#[test]
fn scriptlet_defined_empty_no_ids() {
    let scriptlet = Scriptlet::new(
        "test".to_string(),
        "bash".to_string(),
        "echo hi".to_string(),
    );
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert!(!actions
        .iter()
        .any(|a| a.id.starts_with("scriptlet_action:")));
}

// =========== 19. Scriptlet defined: action with description preserved ===========

#[test]
fn scriptlet_defined_preserves_description() {
    let mut scriptlet = Scriptlet::new(
        "test".to_string(),
        "bash".to_string(),
        "echo hi".to_string(),
    );
    scriptlet.actions = vec![ScriptletAction {
        name: "Copy".to_string(),
        command: "pbcopy".to_string(),
        tool: "bash".to_string(),
        code: "echo hi | pbcopy".to_string(),
        description: Some("Copy to clipboard".to_string()),
        shortcut: None,
        inputs: vec![],
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert_eq!(
        actions[0].description,
        Some("Copy to clipboard".to_string())
    );
}

#[test]
fn scriptlet_defined_has_action_true() {
    let mut scriptlet = Scriptlet::new(
        "test".to_string(),
        "bash".to_string(),
        "echo hi".to_string(),
    );
    scriptlet.actions = vec![ScriptletAction {
        name: "Copy".to_string(),
        command: "pbcopy".to_string(),
        tool: "bash".to_string(),
        code: "echo hi | pbcopy".to_string(),
        description: None,
        shortcut: None,
        inputs: vec![],
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert!(actions[0].has_action);
}

#[test]
fn scriptlet_defined_value_is_command() {
    let mut scriptlet = Scriptlet::new(
        "test".to_string(),
        "bash".to_string(),
        "echo hi".to_string(),
    );
    scriptlet.actions = vec![ScriptletAction {
        name: "Copy".to_string(),
        command: "pbcopy".to_string(),
        tool: "bash".to_string(),
        code: "echo hi | pbcopy".to_string(),
        description: None,
        shortcut: None,
        inputs: vec![],
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert_eq!(actions[0].value, Some("pbcopy".to_string()));
}

// --- merged from part_03.rs ---

#[test]
fn scriptlet_defined_id_uses_action_id() {
    let mut scriptlet = Scriptlet::new(
        "test".to_string(),
        "bash".to_string(),
        "echo hi".to_string(),
    );
    scriptlet.actions = vec![ScriptletAction {
        name: "Copy".to_string(),
        command: "pbcopy".to_string(),
        tool: "bash".to_string(),
        code: "echo hi | pbcopy".to_string(),
        description: None,
        shortcut: None,
        inputs: vec![],
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert!(actions[0].id.contains("pbcopy") || actions[0].id.starts_with("scriptlet_action:"));
}

// =========== 20. AI bar: all 12 have descriptions ===========

#[test]
fn ai_bar_all_have_descriptions() {
    let actions = get_ai_command_bar_actions();
    assert!(actions.iter().all(|a| a.description.is_some()));
}

#[test]
fn ai_bar_all_descriptions_non_empty() {
    let actions = get_ai_command_bar_actions();
    assert!(actions
        .iter()
        .all(|a| !a.description.as_ref().unwrap().is_empty()));
}

#[test]
fn ai_bar_count_is_12() {
    let actions = get_ai_command_bar_actions();
    assert_eq!(actions.len(), 12);
}

#[test]
fn ai_bar_all_have_icons() {
    let actions = get_ai_command_bar_actions();
    assert!(actions.iter().all(|a| a.icon.is_some()));
}

// =========== 21. AI bar: Response section has 3 actions ===========

#[test]
fn ai_bar_response_section_count() {
    let actions = get_ai_command_bar_actions();
    let response_count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Response"))
        .count();
    assert_eq!(response_count, 3);
}

#[test]
fn ai_bar_response_has_copy_response() {
    let actions = get_ai_command_bar_actions();
    let response: Vec<_> = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Response"))
        .collect();
    assert!(response.iter().any(|a| a.id == "copy_response"));
}

#[test]
fn ai_bar_response_has_copy_chat() {
    let actions = get_ai_command_bar_actions();
    let response: Vec<_> = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Response"))
        .collect();
    assert!(response.iter().any(|a| a.id == "copy_chat"));
}

#[test]
fn ai_bar_response_has_copy_last_code() {
    let actions = get_ai_command_bar_actions();
    let response: Vec<_> = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Response"))
        .collect();
    assert!(response.iter().any(|a| a.id == "copy_last_code"));
}

// =========== 22. Notes: untested boolean combos ===========

#[test]
fn notes_no_selection_trash_no_auto_count() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert_eq!(actions.len(), 3);
}

#[test]
fn notes_no_selection_trash_auto_count() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert_eq!(actions.len(), 2);
}

#[test]
fn notes_selection_trash_auto_count() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert_eq!(actions.len(), 2);
}

#[test]
fn notes_no_selection_no_trash_auto_count() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert_eq!(actions.len(), 2);
}

// =========== 23. Notes: trash+selection suppresses selection-dependent ===========

#[test]
fn notes_trash_selection_no_duplicate() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "duplicate_note"));
}

#[test]
fn notes_trash_selection_no_find() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "find_in_note"));
}

#[test]
fn notes_trash_selection_no_format() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "format"));
}

#[test]
fn notes_trash_selection_no_export() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "export"));
}

// =========== 24. Chat: 2 models + response + messages = 5 actions ===========

#[test]
fn chat_2_models_response_messages_count() {
    let info = ChatPromptInfo {
        current_model: Some("Claude".into()),
        available_models: vec![
            ChatModelInfo {
                id: "claude".into(),
                display_name: "Claude".into(),
                provider: "Anthropic".into(),
            },
            ChatModelInfo {
                id: "gpt4".into(),
                display_name: "GPT-4".into(),
                provider: "OpenAI".into(),
            },
        ],
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions.len(), 5);
}

#[test]
fn chat_2_models_response_messages_has_continue() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![
            ChatModelInfo {
                id: "a".into(),
                display_name: "A".into(),
                provider: "P".into(),
            },
            ChatModelInfo {
                id: "b".into(),
                display_name: "B".into(),
                provider: "Q".into(),
            },
        ],
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "continue_in_chat"));
}

#[test]
fn chat_2_models_response_messages_has_copy_response() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![
            ChatModelInfo {
                id: "a".into(),
                display_name: "A".into(),
                provider: "P".into(),
            },
            ChatModelInfo {
                id: "b".into(),
                display_name: "B".into(),
                provider: "Q".into(),
            },
        ],
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "copy_response"));
}

#[test]
fn chat_2_models_response_messages_has_clear() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![
            ChatModelInfo {
                id: "a".into(),
                display_name: "A".into(),
                provider: "P".into(),
            },
            ChatModelInfo {
                id: "b".into(),
                display_name: "B".into(),
                provider: "Q".into(),
            },
        ],
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "clear_conversation"));
}

// =========== 25. Chat: models before continue_in_chat in ordering ===========

#[test]
fn chat_model_at_index_0() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "claude".into(),
            display_name: "Claude".into(),
            provider: "Anthropic".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions[0].id.starts_with("select_model_"));
}

#[test]
fn chat_continue_after_models() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![
            ChatModelInfo {
                id: "a".into(),
                display_name: "A".into(),
                provider: "P".into(),
            },
            ChatModelInfo {
                id: "b".into(),
                display_name: "B".into(),
                provider: "Q".into(),
            },
        ],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions[2].id, "continue_in_chat");
}

#[test]
fn chat_models_preserve_insertion_order() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![
            ChatModelInfo {
                id: "first".into(),
                display_name: "First".into(),
                provider: "P".into(),
            },
            ChatModelInfo {
                id: "second".into(),
                display_name: "Second".into(),
                provider: "Q".into(),
            },
        ],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions[0].id, "select_model_first");
    assert_eq!(actions[1].id, "select_model_second");
}

#[test]
fn chat_single_model_continue_at_index_1() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "only".into(),
            display_name: "Only".into(),
            provider: "P".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions[1].id, "continue_in_chat");
}

// =========== 26. New chat: section assignment per type ===========

#[test]
fn new_chat_last_used_section() {
    let last_used = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "Model 1".into(),
        provider: "p".into(),
        provider_display_name: "Provider".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
}

#[test]
fn new_chat_preset_section() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].section.as_deref(), Some("Presets"));
}

#[test]
fn new_chat_model_section() {
    let models = vec![NewChatModelInfo {
        model_id: "gpt4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].section.as_deref(), Some("Models"));
}

#[test]
fn new_chat_all_three_sections_present() {
    let last_used = vec![NewChatModelInfo {
        model_id: "m1".into(),
        display_name: "M1".into(),
        provider: "p".into(),
        provider_display_name: "P".into(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "gen".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "gpt4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &presets, &models);
    let sections: Vec<_> = actions
        .iter()
        .filter_map(|a| a.section.as_deref())
        .collect();
    assert!(sections.contains(&"Last Used Settings"));
    assert!(sections.contains(&"Presets"));
    assert!(sections.contains(&"Models"));
}

// =========== 27. Clipboard: text has no image-specific actions ===========

#[test]
fn clipboard_text_no_ocr() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clipboard_ocr"));
}

// --- merged from part_04.rs ---

#[test]
fn clipboard_text_no_open_with() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clipboard_open_with"));
}

#[test]
fn clipboard_text_no_annotate_cleanshot() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions
        .iter()
        .any(|a| a.id == "clipboard_annotate_cleanshot"));
}

#[test]
fn clipboard_text_no_upload_cleanshot() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clipboard_upload_cleanshot"));
}

// =========== 28. Script: scriptlet vs with_custom share common actions ===========

#[test]
fn scriptlet_both_contexts_have_run_script() {
    let s = ScriptInfo::scriptlet("My Script", "/s.md", None, None);
    let script_actions = get_script_context_actions(&s);
    let custom_actions = get_scriptlet_context_actions_with_custom(&s, None);
    assert!(script_actions.iter().any(|a| a.id == "run_script"));
    assert!(custom_actions.iter().any(|a| a.id == "run_script"));
}

#[test]
fn scriptlet_both_contexts_have_copy_content() {
    let s = ScriptInfo::scriptlet("My Script", "/s.md", None, None);
    let script_actions = get_script_context_actions(&s);
    let custom_actions = get_scriptlet_context_actions_with_custom(&s, None);
    assert!(script_actions.iter().any(|a| a.id == "copy_content"));
    assert!(custom_actions.iter().any(|a| a.id == "copy_content"));
}

#[test]
fn scriptlet_both_contexts_have_edit_scriptlet() {
    let s = ScriptInfo::scriptlet("My Script", "/s.md", None, None);
    let script_actions = get_script_context_actions(&s);
    let custom_actions = get_scriptlet_context_actions_with_custom(&s, None);
    assert!(script_actions.iter().any(|a| a.id == "edit_scriptlet"));
    assert!(custom_actions.iter().any(|a| a.id == "edit_scriptlet"));
}

#[test]
fn scriptlet_both_contexts_have_copy_deeplink() {
    let s = ScriptInfo::scriptlet("My Script", "/s.md", None, None);
    let script_actions = get_script_context_actions(&s);
    let custom_actions = get_scriptlet_context_actions_with_custom(&s, None);
    assert!(script_actions.iter().any(|a| a.id == "copy_deeplink"));
    assert!(custom_actions.iter().any(|a| a.id == "copy_deeplink"));
}

// =========== 29. Dialog format_shortcut_hint: arrow key variants ===========

#[test]
fn dialog_format_hint_up() {
    assert_eq!(ActionsDialog::format_shortcut_hint("up"), "↑");
}

#[test]
fn dialog_format_hint_arrowup() {
    assert_eq!(ActionsDialog::format_shortcut_hint("arrowup"), "↑");
}

#[test]
fn dialog_format_hint_down() {
    assert_eq!(ActionsDialog::format_shortcut_hint("down"), "↓");
}

#[test]
fn dialog_format_hint_arrowdown() {
    assert_eq!(ActionsDialog::format_shortcut_hint("arrowdown"), "↓");
}

// =========== 30. Dialog format_shortcut_hint: control and opt aliases ===========

#[test]
fn dialog_format_hint_control() {
    assert_eq!(ActionsDialog::format_shortcut_hint("control+k"), "⌃K");
}

#[test]
fn dialog_format_hint_opt() {
    assert_eq!(ActionsDialog::format_shortcut_hint("opt+k"), "⌥K");
}

#[test]
fn dialog_format_hint_command() {
    assert_eq!(ActionsDialog::format_shortcut_hint("command+k"), "⌘K");
}

#[test]
fn dialog_format_hint_arrowleft() {
    assert_eq!(ActionsDialog::format_shortcut_hint("arrowleft"), "←");
}
