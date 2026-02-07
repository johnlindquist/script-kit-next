//! Cross-context validation tests for actions/dialog/window behaviors
//!
//! These tests validate cross-cutting concerns:
//! - Primary action consistency across all builder contexts
//! - Action description content patterns
//! - Shortcut uniqueness within contexts
//! - Section grouping correctness for command bars
//! - Dynamic title formatting with special characters
//! - Clipboard action completeness for each entry type
//! - Note switcher icon assignment correctness
//! - Agent vs script flag interaction edge cases
//! - Action count determinism for all builder permutations

use super::builders::*;
use super::command_bar::CommandBarConfig;
use super::dialog::{build_grouped_items_static, coerce_action_selection, GroupedActionItem};
use super::types::*;
use super::window::{count_section_headers, WindowPosition};
use crate::clipboard_history::ContentType;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;
use crate::protocol::ProtocolAction;
use crate::scriptlets::{Scriptlet, ScriptletAction};

// ============================================================================
// Cross-context primary action consistency
// ============================================================================

#[test]
fn primary_action_is_always_first_in_script_context() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].id, "run_script");
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn primary_action_is_always_first_in_scriptlet_context() {
    let script = ScriptInfo::scriptlet("test", "/path/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert_eq!(actions[0].id, "run_script");
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn primary_action_is_always_first_in_file_context_file() {
    let info = FileInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&info);
    assert_eq!(actions[0].id, "open_file");
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn primary_action_is_always_first_in_file_context_dir() {
    let info = FileInfo {
        path: "/test/dir".into(),
        name: "dir".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&info);
    assert_eq!(actions[0].id, "open_directory");
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn primary_action_is_always_first_in_path_context_file() {
    let info = PathInfo::new("file.txt", "/test/file.txt", false);
    let actions = get_path_context_actions(&info);
    assert_eq!(actions[0].id, "select_file");
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn primary_action_is_always_first_in_path_context_dir() {
    let info = PathInfo::new("dir", "/test/dir", true);
    let actions = get_path_context_actions(&info);
    assert_eq!(actions[0].id, "open_directory");
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn primary_action_is_always_first_in_clipboard_context() {
    let info = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&info);
    assert_eq!(actions[0].id, "clipboard_paste");
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn primary_action_is_always_first_in_chat_context() {
    // Chat with no models has continue_in_chat as first
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions[0].id, "continue_in_chat");
}

// ============================================================================
// All actions have descriptions
// ============================================================================

#[test]
fn all_script_actions_have_descriptions() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    for action in &actions {
        assert!(
            action.description.is_some(),
            "Script action '{}' missing description",
            action.id
        );
    }
}

#[test]
fn all_file_actions_have_descriptions() {
    let info = FileInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&info);
    for action in &actions {
        assert!(
            action.description.is_some(),
            "File action '{}' missing description",
            action.id
        );
    }
}

#[test]
fn all_path_actions_have_descriptions() {
    let info = PathInfo::new("dir", "/test/dir", true);
    let actions = get_path_context_actions(&info);
    for action in &actions {
        assert!(
            action.description.is_some(),
            "Path action '{}' missing description",
            action.id
        );
    }
}

#[test]
fn all_clipboard_actions_have_descriptions() {
    let info = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: true,
        preview: "img".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: Some("Finder".into()),
    };
    let actions = get_clipboard_history_context_actions(&info);
    for action in &actions {
        assert!(
            action.description.is_some(),
            "Clipboard action '{}' missing description",
            action.id
        );
    }
}

#[test]
fn all_ai_command_bar_actions_have_descriptions() {
    let actions = get_ai_command_bar_actions();
    for action in &actions {
        assert!(
            action.description.is_some(),
            "AI action '{}' missing description",
            action.id
        );
    }
}

#[test]
fn all_notes_command_bar_actions_have_descriptions() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    for action in &actions {
        assert!(
            action.description.is_some(),
            "Notes action '{}' missing description",
            action.id
        );
    }
}

// ============================================================================
// Shortcut uniqueness within each context
// ============================================================================

#[test]
fn no_duplicate_shortcuts_in_script_context() {
    let script = ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let shortcuts: Vec<&str> = actions
        .iter()
        .filter_map(|a| a.shortcut.as_deref())
        .collect();
    for (i, s) in shortcuts.iter().enumerate() {
        for (j, other) in shortcuts.iter().enumerate() {
            if i != j {
                assert_ne!(s, other, "Duplicate shortcut '{}' in script context", s);
            }
        }
    }
}

#[test]
fn no_duplicate_shortcuts_in_ai_command_bar() {
    let actions = get_ai_command_bar_actions();
    let shortcuts: Vec<&str> = actions
        .iter()
        .filter_map(|a| a.shortcut.as_deref())
        .collect();
    for (i, s) in shortcuts.iter().enumerate() {
        for (j, other) in shortcuts.iter().enumerate() {
            if i != j {
                assert_ne!(s, other, "Duplicate shortcut '{}' in AI command bar", s);
            }
        }
    }
}

#[test]
fn no_duplicate_shortcuts_in_clipboard_context() {
    let info = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&info);
    let shortcuts: Vec<&str> = actions
        .iter()
        .filter_map(|a| a.shortcut.as_deref())
        .collect();
    for (i, s) in shortcuts.iter().enumerate() {
        for (j, other) in shortcuts.iter().enumerate() {
            if i != j {
                assert_ne!(s, other, "Duplicate shortcut '{}' in clipboard context", s);
            }
        }
    }
}

// ============================================================================
// Clipboard action completeness per entry type
// ============================================================================

#[test]
fn clipboard_text_unpinned_has_expected_action_set() {
    let info = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello world".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();

    // Must have these core actions
    assert!(ids.contains(&"clipboard_paste"));
    assert!(ids.contains(&"clipboard_copy"));
    assert!(ids.contains(&"clipboard_paste_keep_open"));
    assert!(ids.contains(&"clipboard_share"));
    assert!(ids.contains(&"clipboard_attach_to_ai"));
    assert!(ids.contains(&"clipboard_pin")); // not pinned → pin
    assert!(ids.contains(&"clipboard_save_snippet"));
    assert!(ids.contains(&"clipboard_save_file"));
    assert!(ids.contains(&"clipboard_delete"));
    assert!(ids.contains(&"clipboard_delete_multiple"));
    assert!(ids.contains(&"clipboard_delete_all"));

    // Must NOT have image-only actions
    assert!(!ids.contains(&"clipboard_ocr"));
    assert!(!ids.contains(&"clipboard_unpin"));
}

#[test]
fn clipboard_image_pinned_has_expected_action_set() {
    let info = ClipboardEntryInfo {
        id: "2".into(),
        content_type: ContentType::Image,
        pinned: true,
        preview: "Image 800x600".into(),
        image_dimensions: Some((800, 600)),
        frontmost_app_name: Some("Preview".into()),
    };
    let actions = get_clipboard_history_context_actions(&info);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();

    // Must have image-specific
    assert!(ids.contains(&"clipboard_ocr"));
    assert!(ids.contains(&"clipboard_unpin")); // pinned → unpin
    assert!(!ids.contains(&"clipboard_pin")); // should NOT have pin

    // Paste title should include app name
    let paste = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
    assert_eq!(paste.title, "Paste to Preview");
}

#[test]
fn clipboard_text_exact_action_count() {
    let info = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&info);
    // Text: paste, copy, paste_keep_open, share, attach_to_ai, quick_look (macOS),
    //        pin, save_snippet, save_file, delete, delete_multiple, delete_all
    #[cfg(target_os = "macos")]
    assert_eq!(
        actions.len(),
        12,
        "macOS text clipboard should have 12 actions"
    );
    #[cfg(not(target_os = "macos"))]
    assert_eq!(
        actions.len(),
        11,
        "non-macOS text clipboard should have 11 actions"
    );
}

#[test]
fn clipboard_image_exact_action_count() {
    let info = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&info);
    // Image adds: ocr + macOS: open_with, annotate_cleanshot, upload_cleanshot
    #[cfg(target_os = "macos")]
    assert_eq!(
        actions.len(),
        16,
        "macOS image clipboard should have 16 actions"
    );
    #[cfg(not(target_os = "macos"))]
    assert_eq!(
        actions.len(),
        12,
        "non-macOS image clipboard should have 12 actions"
    );
}

// ============================================================================
// Dynamic title formatting with names
// ============================================================================

#[test]
fn run_script_title_includes_script_name() {
    let script = ScriptInfo::new("Clipboard History", "/path/ch.ts");
    let actions = get_script_context_actions(&script);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert_eq!(run.title, "Run \"Clipboard History\"");
}

#[test]
fn run_script_title_uses_custom_verb() {
    let script = ScriptInfo::with_action_verb("Safari", "/app/safari", false, "Launch");
    let actions = get_script_context_actions(&script);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert_eq!(run.title, "Launch \"Safari\"");
    assert!(
        run.description.as_ref().unwrap().contains("Launch"),
        "Description should use verb"
    );
}

#[test]
fn open_file_title_includes_filename() {
    let info = FileInfo {
        path: "/test/my document.pdf".into(),
        name: "my document.pdf".into(),
        file_type: FileType::Document,
        is_dir: false,
    };
    let actions = get_file_context_actions(&info);
    let open = actions.iter().find(|a| a.id == "open_file").unwrap();
    assert_eq!(open.title, "Open \"my document.pdf\"");
}

#[test]
fn open_directory_title_includes_dirname() {
    let info = FileInfo {
        path: "/test/My Folder".into(),
        name: "My Folder".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&info);
    let open = actions.iter().find(|a| a.id == "open_directory").unwrap();
    assert_eq!(open.title, "Open \"My Folder\"");
}

#[test]
fn path_context_select_file_title_includes_name() {
    let info = PathInfo::new("report.csv", "/data/report.csv", false);
    let actions = get_path_context_actions(&info);
    let select = actions.iter().find(|a| a.id == "select_file").unwrap();
    assert_eq!(select.title, "Select \"report.csv\"");
}

#[test]
fn path_context_open_dir_title_includes_name() {
    let info = PathInfo::new("Documents", "/home/user/Documents", true);
    let actions = get_path_context_actions(&info);
    let open = actions.iter().find(|a| a.id == "open_directory").unwrap();
    assert_eq!(open.title, "Open \"Documents\"");
}
