//! Batch 44: Dialog Built-in Action Validation Tests
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

// =========== 1. ScriptInfo::with_is_script: is_script true sets correct defaults ===========

#[test]
fn with_is_script_true_sets_is_script() {
    let s = ScriptInfo::with_is_script("my-script", "/path", true);
    assert!(s.is_script);
}

#[test]
fn with_is_script_true_is_scriptlet_false() {
    let s = ScriptInfo::with_is_script("my-script", "/path", true);
    assert!(!s.is_scriptlet);
}

#[test]
fn with_is_script_false_sets_is_script_false() {
    let s = ScriptInfo::with_is_script("builtin", "", false);
    assert!(!s.is_script);
}

#[test]
fn with_is_script_defaults_action_verb_run() {
    let s = ScriptInfo::with_is_script("test", "/p", true);
    assert_eq!(s.action_verb, "Run");
}

// =========== 2. ScriptInfo::with_action_verb: custom verb preserved ===========

#[test]
fn with_action_verb_sets_verb() {
    let s = ScriptInfo::with_action_verb("Safari", "/Applications/Safari.app", false, "Launch");
    assert_eq!(s.action_verb, "Launch");
}

#[test]
fn with_action_verb_is_script_param() {
    let s = ScriptInfo::with_action_verb("test", "/p", true, "Execute");
    assert!(s.is_script);
}

#[test]
fn with_action_verb_false_is_script() {
    let s = ScriptInfo::with_action_verb("test", "/p", false, "Open");
    assert!(!s.is_script);
}

#[test]
fn with_action_verb_shortcut_none() {
    let s = ScriptInfo::with_action_verb("test", "/p", true, "Run");
    assert!(s.shortcut.is_none());
}

// =========== 3. Clipboard: paste title with frontmost_app_name ===========

#[test]
fn clipboard_paste_title_with_app_name() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: Some("Safari".to_string()),
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
    assert_eq!(paste.title, "Paste to Safari");
}

#[test]
fn clipboard_paste_title_without_app_name() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
    assert_eq!(paste.title, "Paste to Active App");
}

#[test]
fn clipboard_paste_shortcut_is_enter() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
    assert_eq!(paste.shortcut, Some("↵".to_string()));
}

#[test]
fn clipboard_paste_desc_mentions_paste() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
    assert!(paste.description.as_ref().unwrap().contains("paste"));
}

// =========== 4. Clipboard: save_snippet and save_file details ===========

#[test]
fn clipboard_save_snippet_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "code".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ss = actions
        .iter()
        .find(|a| a.id == "clipboard_save_snippet")
        .unwrap();
    assert_eq!(ss.shortcut, Some("⇧⌘S".to_string()));
}

#[test]
fn clipboard_save_snippet_title() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "code".into(),
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
fn clipboard_save_file_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "code".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let sf = actions
        .iter()
        .find(|a| a.id == "clipboard_save_file")
        .unwrap();
    assert_eq!(sf.shortcut, Some("⌥⇧⌘S".to_string()));
}

#[test]
fn clipboard_save_file_title() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "code".into(),
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

// =========== 5. Clipboard: image upload_cleanshot details (macOS) ===========

#[cfg(target_os = "macos")]
#[test]
fn clipboard_upload_cleanshot_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let uc = actions
        .iter()
        .find(|a| a.id == "clipboard_upload_cleanshot")
        .unwrap();
    assert_eq!(uc.shortcut, Some("⇧⌘U".to_string()));
}

#[cfg(target_os = "macos")]
#[test]
fn clipboard_upload_cleanshot_title() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let uc = actions
        .iter()
        .find(|a| a.id == "clipboard_upload_cleanshot")
        .unwrap();
    assert_eq!(uc.title, "Upload to CleanShot X");
}

#[cfg(target_os = "macos")]
#[test]
fn clipboard_upload_cleanshot_not_present_for_text() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "txt".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clipboard_upload_cleanshot"));
}

#[cfg(target_os = "macos")]
#[test]
fn clipboard_upload_cleanshot_desc_mentions_cloud() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((200, 200)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let uc = actions
        .iter()
        .find(|a| a.id == "clipboard_upload_cleanshot")
        .unwrap();
    assert!(uc.description.as_ref().unwrap().contains("Cloud"));
}

// =========== 6. Clipboard: OCR shortcut and desc ===========

#[test]
fn clipboard_ocr_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ocr = actions.iter().find(|a| a.id == "clipboard_ocr").unwrap();
    assert_eq!(ocr.shortcut, Some("⇧⌘C".to_string()));
}

#[test]
fn clipboard_ocr_title() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ocr = actions.iter().find(|a| a.id == "clipboard_ocr").unwrap();
    assert_eq!(ocr.title, "Copy Text from Image");
}

#[test]
fn clipboard_ocr_desc_mentions_ocr() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ocr = actions.iter().find(|a| a.id == "clipboard_ocr").unwrap();
    assert!(ocr.description.as_ref().unwrap().contains("OCR"));
}

#[test]
fn clipboard_ocr_not_present_for_text() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "txt".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clipboard_ocr"));
}

// =========== 7. File context: quick_look only for files (macOS) ===========

#[cfg(target_os = "macos")]
#[test]
fn file_quick_look_present_for_file() {
    let file = FileInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    assert!(actions.iter().any(|a| a.id == "quick_look"));
}

#[cfg(target_os = "macos")]
#[test]
fn file_quick_look_shortcut() {
    let file = FileInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    let ql = actions.iter().find(|a| a.id == "quick_look").unwrap();
    assert_eq!(ql.shortcut, Some("⌘Y".to_string()));
}

#[cfg(target_os = "macos")]
#[test]
fn file_quick_look_absent_for_dir() {
    let dir = FileInfo {
        path: "/test/dir".into(),
        name: "dir".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&dir);
    assert!(!actions.iter().any(|a| a.id == "quick_look"));
}

#[cfg(target_os = "macos")]
#[test]
fn file_quick_look_desc_mentions_preview() {
    let file = FileInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    let ql = actions.iter().find(|a| a.id == "quick_look").unwrap();
    assert!(ql
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("preview"));
}

// =========== 8. File context: copy_path shortcut is ⌘⇧C ===========

#[test]
fn file_copy_path_shortcut() {
    let file = FileInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
    assert_eq!(cp.shortcut, Some("⌘⇧C".to_string()));
}

#[test]
fn file_copy_path_title() {
    let file = FileInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
    assert_eq!(cp.title, "Copy Path");
}

#[test]
fn file_copy_path_desc_mentions_clipboard() {
    let file = FileInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
    assert!(cp.description.as_ref().unwrap().contains("clipboard"));
}

#[test]
fn file_copy_path_present_for_dir() {
    let dir = FileInfo {
        path: "/test/dir".into(),
        name: "dir".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&dir);
    assert!(actions.iter().any(|a| a.id == "copy_path"));
}
