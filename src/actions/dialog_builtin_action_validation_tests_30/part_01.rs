//! Batch 30: Builtin action validation tests
//!
//! 30 categories validating random built-in action behaviors across
//! script, clipboard, file, path, AI, notes, chat, and new-chat contexts.

use crate::actions::builders::{
    get_ai_command_bar_actions, get_chat_context_actions, get_clipboard_history_context_actions,
    get_file_context_actions, get_new_chat_actions, get_note_switcher_actions,
    get_notes_command_bar_actions, get_path_context_actions, get_script_context_actions,
    get_scriptlet_context_actions_with_custom, to_deeplink_name, ChatModelInfo, ChatPromptInfo,
    ClipboardEntryInfo, NewChatModelInfo, NewChatPresetInfo, NoteSwitcherNoteInfo, NotesInfo,
};
use crate::actions::command_bar::CommandBarConfig;
use crate::actions::dialog::{build_grouped_items_static, coerce_action_selection, ActionsDialog};
use crate::actions::types::{Action, ActionCategory, AnchorPosition, SearchPosition, SectionStyle};
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;

// ---------------------------------------------------------------------------
// 1. Script context: copy_content description wording is consistent
// ---------------------------------------------------------------------------
#[test]
fn batch30_script_copy_content_desc_says_entire_file() {
    let script = crate::actions::types::ScriptInfo::new("test", "/path/test.ts");
    let actions = get_script_context_actions(&script);
    let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert!(
        cc.description.as_ref().unwrap().contains("entire file"),
        "copy_content desc should mention 'entire file', got: {:?}",
        cc.description
    );
}

#[test]
fn batch30_scriptlet_copy_content_desc_says_entire_file() {
    let script = crate::actions::types::ScriptInfo::scriptlet("x", "/p.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert!(cc.description.as_ref().unwrap().contains("entire file"));
}

#[test]
fn batch30_agent_copy_content_desc_says_entire_file() {
    let mut script = crate::actions::types::ScriptInfo::new("agent", "/p/agent.ts");
    script.is_script = false;
    script.is_agent = true;
    let actions = get_script_context_actions(&script);
    let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert!(cc.description.as_ref().unwrap().contains("entire file"));
}

#[test]
fn batch30_builtin_has_no_copy_content() {
    let script = crate::actions::types::ScriptInfo::builtin("Settings");
    let actions = get_script_context_actions(&script);
    assert!(!actions.iter().any(|a| a.id == "copy_content"));
}

// ---------------------------------------------------------------------------
// 2. Clipboard: image-only actions absent for text entries
// ---------------------------------------------------------------------------
#[test]
fn batch30_clipboard_text_no_ocr() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hi".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clipboard_ocr"));
}

#[test]
fn batch30_clipboard_text_no_open_with() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clipboard_open_with"));
}

#[test]
fn batch30_clipboard_text_no_annotate_cleanshot() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions
        .iter()
        .any(|a| a.id == "clipboard_annotate_cleanshot"));
}

#[test]
fn batch30_clipboard_text_no_upload_cleanshot() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clipboard_upload_cleanshot"));
}

// ---------------------------------------------------------------------------
// 3. Clipboard: image entry has OCR and macOS image actions
// ---------------------------------------------------------------------------
#[test]
fn batch30_clipboard_image_has_ocr() {
    let entry = ClipboardEntryInfo {
        id: "i".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_ocr"));
}

#[cfg(target_os = "macos")]
#[test]
fn batch30_clipboard_image_has_open_with_macos() {
    let entry = ClipboardEntryInfo {
        id: "i".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_open_with"));
}

#[cfg(target_os = "macos")]
#[test]
fn batch30_clipboard_image_has_annotate_cleanshot_macos() {
    let entry = ClipboardEntryInfo {
        id: "i".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions
        .iter()
        .any(|a| a.id == "clipboard_annotate_cleanshot"));
}

#[cfg(target_os = "macos")]
#[test]
fn batch30_clipboard_image_annotate_cleanshot_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "i".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let a = actions
        .iter()
        .find(|a| a.id == "clipboard_annotate_cleanshot")
        .unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⇧⌘A"));
}

// ---------------------------------------------------------------------------
// 4. File context: directory has no quick_look on macOS
// ---------------------------------------------------------------------------
#[cfg(target_os = "macos")]
#[test]
fn batch30_file_dir_no_quick_look() {
    let info = FileInfo {
        path: "/tmp/dir".into(),
        name: "dir".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "quick_look"));
}

#[cfg(target_os = "macos")]
#[test]
fn batch30_file_regular_has_quick_look() {
    let info = FileInfo {
        path: "/tmp/f.txt".into(),
        name: "f.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "quick_look"));
}

#[cfg(target_os = "macos")]
#[test]
fn batch30_file_quick_look_shortcut_is_cmd_y() {
    let info = FileInfo {
        path: "/tmp/f.txt".into(),
        name: "f.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&info);
    let ql = actions.iter().find(|a| a.id == "quick_look").unwrap();
    assert_eq!(ql.shortcut.as_deref(), Some("⌘Y"));
}

#[test]
fn batch30_file_dir_has_open_directory() {
    let info = FileInfo {
        path: "/tmp/dir".into(),
        name: "dir".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "open_directory"));
    assert!(!actions.iter().any(|a| a.id == "open_file"));
}

// ---------------------------------------------------------------------------
// 5. Path context: total action count for file vs dir
// ---------------------------------------------------------------------------
#[test]
fn batch30_path_file_has_7_actions() {
    let info = PathInfo {
        path: "/tmp/f.txt".into(),
        name: "f.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&info);
    assert_eq!(
        actions.len(),
        7,
        "Path file should have 7 actions: select_file, copy_path, open_in_finder, open_in_editor, open_in_terminal, copy_filename, move_to_trash"
    );
}

#[test]
fn batch30_path_dir_has_7_actions() {
    let info = PathInfo {
        path: "/tmp/d".into(),
        name: "d".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&info);
    assert_eq!(
        actions.len(),
        7,
        "Path dir should have 7 actions: open_directory, copy_path, open_in_finder, open_in_editor, open_in_terminal, copy_filename, move_to_trash"
    );
}

#[test]
fn batch30_path_file_first_action_is_select_file() {
    let info = PathInfo {
        path: "/tmp/f.txt".into(),
        name: "f.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&info);
    assert_eq!(actions[0].id, "select_file");
}

#[test]
fn batch30_path_dir_first_action_is_open_directory() {
    let info = PathInfo {
        path: "/tmp/d".into(),
        name: "d".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&info);
    assert_eq!(actions[0].id, "open_directory");
}

// ---------------------------------------------------------------------------
// 6. Path context: open_in_terminal shortcut is ⌘T
// ---------------------------------------------------------------------------
#[test]
fn batch30_path_open_in_terminal_shortcut() {
    let info = PathInfo {
        path: "/tmp/f".into(),
        name: "f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&info);
    let t = actions.iter().find(|a| a.id == "open_in_terminal").unwrap();
    assert_eq!(t.shortcut.as_deref(), Some("⌘T"));
}

#[test]
fn batch30_path_open_in_terminal_desc_mentions_terminal() {
    let info = PathInfo {
        path: "/tmp/f".into(),
        name: "f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&info);
    let t = actions.iter().find(|a| a.id == "open_in_terminal").unwrap();
    assert!(t
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("terminal"));
}

#[test]
fn batch30_path_open_in_finder_shortcut() {
    let info = PathInfo {
        path: "/tmp/f".into(),
        name: "f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&info);
    let f = actions.iter().find(|a| a.id == "open_in_finder").unwrap();
    assert_eq!(f.shortcut.as_deref(), Some("⌘⇧F"));
}

#[test]
fn batch30_path_copy_path_shortcut() {
    let info = PathInfo {
        path: "/tmp/f".into(),
        name: "f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&info);
    let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
    assert_eq!(cp.shortcut.as_deref(), Some("⌘⇧C"));
}

// ---------------------------------------------------------------------------
// 7. AI command bar: all 12 actions have unique IDs
// ---------------------------------------------------------------------------
#[test]
fn batch30_ai_bar_12_actions() {
    let actions = get_ai_command_bar_actions();
    assert_eq!(actions.len(), 12);
}

#[test]
fn batch30_ai_bar_all_ids_unique() {
    let actions = get_ai_command_bar_actions();
    let mut ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 12, "All 12 AI bar action IDs must be unique");
}

#[test]
fn batch30_ai_bar_all_have_section() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert!(
            a.section.is_some(),
            "AI bar action '{}' should have a section",
            a.id
        );
    }
}

#[test]
fn batch30_ai_bar_all_have_icon() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert!(
            a.icon.is_some(),
            "AI bar action '{}' should have an icon",
            a.id
        );
    }
}

// ---------------------------------------------------------------------------
// 8. AI command bar: branch_from_last has no shortcut
// ---------------------------------------------------------------------------
#[test]
fn batch30_ai_bar_branch_from_last_no_shortcut() {
    let actions = get_ai_command_bar_actions();
    let b = actions.iter().find(|a| a.id == "branch_from_last").unwrap();
    assert!(b.shortcut.is_none());
}

#[test]
fn batch30_ai_bar_change_model_no_shortcut() {
    let actions = get_ai_command_bar_actions();
    let cm = actions.iter().find(|a| a.id == "change_model").unwrap();
    assert!(cm.shortcut.is_none());
}

#[test]
fn batch30_ai_bar_branch_from_last_icon_arrowright() {
    let actions = get_ai_command_bar_actions();
    let b = actions.iter().find(|a| a.id == "branch_from_last").unwrap();
    assert_eq!(b.icon, Some(IconName::ArrowRight));
}

#[test]
fn batch30_ai_bar_change_model_icon_settings() {
    let actions = get_ai_command_bar_actions();
    let cm = actions.iter().find(|a| a.id == "change_model").unwrap();
    assert_eq!(cm.icon, Some(IconName::Settings));
}
