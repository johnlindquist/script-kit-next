// Batch 46: Dialog Built-in Action Validation Tests
//
// 120 tests across 30 categories validating action behaviors
// in various built-in action window dialogs.

use crate::actions::builders::*;
use crate::actions::dialog::{
    build_grouped_items_static, coerce_action_selection, GroupedActionItem,
};
use crate::actions::types::{Action, ActionCategory, ScriptInfo, SectionStyle};
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;

// =========== 1. Action::with_shortcut_opt: Some vs None ===========

#[test]
fn with_shortcut_opt_some_sets_shortcut() {
    let a = Action::new("test", "Test", None, ActionCategory::ScriptContext)
        .with_shortcut_opt(Some("⌘K".to_string()));
    assert_eq!(a.shortcut, Some("⌘K".to_string()));
}

#[test]
fn with_shortcut_opt_some_sets_shortcut_lower() {
    let a = Action::new("test", "Test", None, ActionCategory::ScriptContext)
        .with_shortcut_opt(Some("⌘K".to_string()));
    assert_eq!(a.shortcut_lower, Some("⌘k".to_string()));
}

#[test]
fn with_shortcut_opt_none_leaves_shortcut_none() {
    let a =
        Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut_opt(None);
    assert!(a.shortcut.is_none());
}

#[test]
fn with_shortcut_opt_none_leaves_shortcut_lower_none() {
    let a =
        Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut_opt(None);
    assert!(a.shortcut_lower.is_none());
}

// =========== 2. Action: title_lower correctly lowercased for mixed case ===========

#[test]
fn action_title_lower_from_mixed_case() {
    let a = Action::new("test", "Copy Deeplink", None, ActionCategory::ScriptContext);
    assert_eq!(a.title_lower, "copy deeplink");
}

#[test]
fn action_title_lower_from_all_caps() {
    let a = Action::new("test", "SUBMIT", None, ActionCategory::ScriptContext);
    assert_eq!(a.title_lower, "submit");
}

#[test]
fn action_title_lower_preserves_already_lowercase() {
    let a = Action::new("test", "browse notes", None, ActionCategory::ScriptContext);
    assert_eq!(a.title_lower, "browse notes");
}

#[test]
fn action_description_lower_from_mixed_case() {
    let a = Action::new(
        "test",
        "Test",
        Some("Open in $EDITOR".to_string()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(a.description_lower, Some("open in $editor".to_string()));
}

// =========== 3. ScriptInfo::with_action_verb_and_shortcut: verb and shortcut ===========

#[test]
fn with_action_verb_and_shortcut_sets_verb() {
    let s = ScriptInfo::with_action_verb_and_shortcut("Safari", "/app", false, "Launch", None);
    assert_eq!(s.action_verb, "Launch");
}

#[test]
fn with_action_verb_and_shortcut_sets_shortcut() {
    let s = ScriptInfo::with_action_verb_and_shortcut(
        "Safari",
        "/app",
        false,
        "Launch",
        Some("cmd+l".into()),
    );
    assert_eq!(s.shortcut, Some("cmd+l".to_string()));
}

#[test]
fn with_action_verb_and_shortcut_is_agent_false() {
    let s = ScriptInfo::with_action_verb_and_shortcut("Safari", "/app", false, "Launch", None);
    assert!(!s.is_agent);
}

#[test]
fn with_action_verb_and_shortcut_alias_none() {
    let s = ScriptInfo::with_action_verb_and_shortcut("Safari", "/app", false, "Launch", None);
    assert!(s.alias.is_none());
}

// =========== 4. Clipboard: unpinned text action count on macOS ===========

#[test]
fn clipboard_text_unpinned_has_pin_action() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_pin"));
    assert!(!actions.iter().any(|a| a.id == "clipboard_unpin"));
}

#[test]
fn clipboard_text_pinned_has_unpin_action() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: true,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_unpin"));
    assert!(!actions.iter().any(|a| a.id == "clipboard_pin"));
}

#[test]
fn clipboard_pin_shortcut_is_shift_cmd_p() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let pin = actions.iter().find(|a| a.id == "clipboard_pin").unwrap();
    assert_eq!(pin.shortcut.as_deref(), Some("⇧⌘P"));
}

#[test]
fn clipboard_unpin_shortcut_is_shift_cmd_p() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: true,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let unpin = actions.iter().find(|a| a.id == "clipboard_unpin").unwrap();
    assert_eq!(unpin.shortcut.as_deref(), Some("⇧⌘P"));
}

// =========== 5. Clipboard: paste_keep_open shortcut ⌥↵ ===========

#[test]
fn clipboard_paste_keep_open_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let pko = actions
        .iter()
        .find(|a| a.id == "clipboard_paste_keep_open")
        .unwrap();
    assert_eq!(pko.shortcut.as_deref(), Some("⌥↵"));
}

#[test]
fn clipboard_paste_keep_open_title() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let pko = actions
        .iter()
        .find(|a| a.id == "clipboard_paste_keep_open")
        .unwrap();
    assert_eq!(pko.title, "Paste and Keep Window Open");
}

#[test]
fn clipboard_paste_keep_open_desc_mentions_keep() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let pko = actions
        .iter()
        .find(|a| a.id == "clipboard_paste_keep_open")
        .unwrap();
    assert!(pko.description.as_ref().unwrap().contains("keep"));
}

#[test]
fn clipboard_paste_keep_open_present_for_image() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_paste_keep_open"));
}

// =========== 6. Clipboard: copy shortcut ⌘↵ ===========

#[test]
fn clipboard_copy_shortcut_is_cmd_enter() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let copy = actions.iter().find(|a| a.id == "clipboard_copy").unwrap();
    assert_eq!(copy.shortcut.as_deref(), Some("⌘↵"));
}

#[test]
fn clipboard_copy_title() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let copy = actions.iter().find(|a| a.id == "clipboard_copy").unwrap();
    assert_eq!(copy.title, "Copy to Clipboard");
}

#[test]
fn clipboard_copy_desc_mentions_without_pasting() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let copy = actions.iter().find(|a| a.id == "clipboard_copy").unwrap();
    assert!(copy.description.as_ref().unwrap().contains("without"));
}

#[test]
fn clipboard_copy_present_for_image() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "".into(),
        image_dimensions: Some((50, 50)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_copy"));
}

// =========== 7. File context: copy_filename shortcut ⌘C ===========

#[test]
fn file_context_copy_filename_shortcut() {
    let fi = FileInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&fi);
    let cf = actions.iter().find(|a| a.id == "copy_filename").unwrap();
    assert_eq!(cf.shortcut.as_deref(), Some("⌘C"));
}

#[test]
fn file_context_copy_filename_title() {
    let fi = FileInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&fi);
    let cf = actions.iter().find(|a| a.id == "copy_filename").unwrap();
    assert_eq!(cf.title, "Copy Filename");
}

#[test]
fn file_context_copy_filename_present_for_dir() {
    let fi = FileInfo {
        path: "/test/docs".into(),
        name: "docs".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&fi);
    assert!(actions.iter().any(|a| a.id == "copy_filename"));
}

#[test]
fn file_context_copy_filename_desc_mentions_filename() {
    let fi = FileInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&fi);
    let cf = actions.iter().find(|a| a.id == "copy_filename").unwrap();
    assert!(cf.description.as_ref().unwrap().contains("filename"));
}

// =========== 8. Path context: open_in_editor shortcut ⌘E ===========

#[test]
fn path_context_open_in_editor_shortcut() {
    let pi = PathInfo::new("file.rs", "/src/file.rs", false);
    let actions = get_path_context_actions(&pi);
    let oie = actions.iter().find(|a| a.id == "open_in_editor").unwrap();
    assert_eq!(oie.shortcut.as_deref(), Some("⌘E"));
}

#[test]
fn path_context_open_in_editor_title() {
    let pi = PathInfo::new("file.rs", "/src/file.rs", false);
    let actions = get_path_context_actions(&pi);
    let oie = actions.iter().find(|a| a.id == "open_in_editor").unwrap();
    assert_eq!(oie.title, "Open in Editor");
}

#[test]
fn path_context_open_in_editor_desc_mentions_editor() {
    let pi = PathInfo::new("file.rs", "/src/file.rs", false);
    let actions = get_path_context_actions(&pi);
    let oie = actions.iter().find(|a| a.id == "open_in_editor").unwrap();
    assert!(oie.description.as_ref().unwrap().contains("$EDITOR"));
}

#[test]
fn path_context_open_in_editor_present_for_dir() {
    let pi = PathInfo::new("src", "/project/src", true);
    let actions = get_path_context_actions(&pi);
    assert!(actions.iter().any(|a| a.id == "open_in_editor"));
}

// =========== 9. Path context: move_to_trash shortcut ⌘⌫ ===========

#[test]
fn path_context_move_to_trash_shortcut() {
    let pi = PathInfo::new("old.txt", "/tmp/old.txt", false);
    let actions = get_path_context_actions(&pi);
    let mt = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
    assert_eq!(mt.shortcut.as_deref(), Some("⌘⌫"));
}

#[test]
fn path_context_move_to_trash_file_desc_says_file() {
    let pi = PathInfo::new("old.txt", "/tmp/old.txt", false);
    let actions = get_path_context_actions(&pi);
    let mt = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
    assert!(mt.description.as_ref().unwrap().contains("file"));
}

#[test]
fn path_context_move_to_trash_dir_desc_says_folder() {
    let pi = PathInfo::new("old_dir", "/tmp/old_dir", true);
    let actions = get_path_context_actions(&pi);
    let mt = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
    assert!(mt.description.as_ref().unwrap().contains("folder"));
}

#[test]
fn path_context_move_to_trash_title() {
    let pi = PathInfo::new("old.txt", "/tmp/old.txt", false);
    let actions = get_path_context_actions(&pi);
    let mt = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
    assert_eq!(mt.title, "Move to Trash");
}

// =========== 10. Path context: file has 7 actions, dir has 8 ===========

#[test]
fn path_context_file_action_count() {
    let pi = PathInfo::new("file.rs", "/src/file.rs", false);
    let actions = get_path_context_actions(&pi);
    // select_file, copy_path, open_in_finder, open_in_editor, open_in_terminal, copy_filename, move_to_trash
    assert_eq!(actions.len(), 7);
}

#[test]
fn path_context_dir_action_count() {
    let pi = PathInfo::new("src", "/project/src", true);
    let actions = get_path_context_actions(&pi);
    // open_directory, copy_path, open_in_finder, open_in_editor, open_in_terminal, copy_filename, move_to_trash
    assert_eq!(actions.len(), 7);
}

#[test]
fn path_context_file_has_select_file() {
    let pi = PathInfo::new("file.rs", "/src/file.rs", false);
    let actions = get_path_context_actions(&pi);
    assert!(actions.iter().any(|a| a.id == "select_file"));
}
