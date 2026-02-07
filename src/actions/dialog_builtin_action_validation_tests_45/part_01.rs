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
