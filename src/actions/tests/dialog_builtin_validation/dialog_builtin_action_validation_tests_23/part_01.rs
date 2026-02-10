// Batch 23: Dialog builtin action validation tests
//
// 30 categories of tests validating random built-in action behaviors.

use super::builders::*;
use super::dialog::*;
use super::types::*;
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;
use crate::scriptlets::{Scriptlet, ScriptletAction};

// ============================================================
// 1. Script context: action_verb propagation in run_script title
// ============================================================

#[test]
fn batch23_action_verb_run_default() {
    let script = ScriptInfo::new("my-script", "/path/to/script.ts");
    let actions = get_script_context_actions(&script);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert!(run.title.starts_with("Run"));
}

#[test]
fn batch23_action_verb_launch() {
    let script =
        ScriptInfo::with_action_verb("Safari", "/Applications/Safari.app", false, "Launch");
    let actions = get_script_context_actions(&script);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert!(run.title.starts_with("Launch"));
    assert!(run.title.contains("Safari"));
}

#[test]
fn batch23_action_verb_switch_to() {
    let script = ScriptInfo::with_action_verb("My Window", "window:123", false, "Switch to");
    let actions = get_script_context_actions(&script);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert!(run.title.starts_with("Switch to"));
}

#[test]
fn batch23_action_verb_open() {
    let script = ScriptInfo::with_action_verb("Clipboard History", "builtin:ch", false, "Open");
    let actions = get_script_context_actions(&script);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert!(run.title.starts_with("Open"));
    assert!(run.description.as_ref().unwrap().contains("Open"));
}

#[test]
fn batch23_action_verb_description_uses_verb() {
    let script = ScriptInfo::with_action_verb("Timer", "/path/timer.ts", true, "Start");
    let actions = get_script_context_actions(&script);
    let run = actions.iter().find(|a| a.id == "run_script").unwrap();
    assert_eq!(run.description.as_ref().unwrap(), "Start this item");
}

// ============================================================
// 2. Script context: action count varies by type flags
// ============================================================

#[test]
fn batch23_script_action_count_full() {
    // is_script=true, no shortcut, no alias, not suggested
    let script = ScriptInfo::new("test", "/test.ts");
    let actions = get_script_context_actions(&script);
    // run_script + add_shortcut + add_alias + edit_script + view_logs + reveal_in_finder + copy_path + copy_content + copy_deeplink = 9
    assert_eq!(actions.len(), 9);
}

#[test]
fn batch23_builtin_action_count() {
    let builtin = ScriptInfo::builtin("Test Built-in");
    let actions = get_script_context_actions(&builtin);
    // run_script + add_shortcut + add_alias + copy_deeplink = 4
    assert_eq!(actions.len(), 4);
}

#[test]
fn batch23_scriptlet_action_count() {
    let scriptlet = ScriptInfo::scriptlet("Test", "/test.md", None, None);
    let actions = get_script_context_actions(&scriptlet);
    // run_script + add_shortcut + add_alias + edit_scriptlet + reveal_scriptlet + copy_scriptlet_path + copy_content + copy_deeplink = 8
    assert_eq!(actions.len(), 8);
}

#[test]
fn batch23_script_with_shortcut_adds_two() {
    let script = ScriptInfo::with_shortcut("test", "/test.ts", Some("cmd+t".to_string()));
    let actions = get_script_context_actions(&script);
    // Same as full script but shortcut adds one extra (update+remove instead of add = +1)
    assert_eq!(actions.len(), 10);
}

#[test]
fn batch23_script_with_shortcut_and_alias_adds_two_more() {
    let script = ScriptInfo::with_shortcut_and_alias(
        "test",
        "/test.ts",
        Some("cmd+t".to_string()),
        Some("ts".to_string()),
    );
    let actions = get_script_context_actions(&script);
    // script(9) + 1 extra shortcut + 1 extra alias = 11
    assert_eq!(actions.len(), 11);
}

// ============================================================
// 3. Clipboard context: exact action ordering
// ============================================================

#[test]
fn batch23_clipboard_text_first_three_actions() {
    let entry = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert_eq!(actions[0].id, "clipboard_paste");
    assert_eq!(actions[1].id, "clipboard_copy");
    assert_eq!(actions[2].id, "clipboard_paste_keep_open");
}

#[test]
fn batch23_clipboard_share_and_attach_after_paste_keep_open() {
    let entry = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert_eq!(actions[3].id, "clipboard_share");
    assert_eq!(actions[4].id, "clipboard_attach_to_ai");
}

#[test]
fn batch23_clipboard_destructive_actions_at_end() {
    let entry = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let len = actions.len();
    assert_eq!(actions[len - 3].id, "clipboard_delete");
    assert_eq!(actions[len - 2].id, "clipboard_delete_multiple");
    assert_eq!(actions[len - 1].id, "clipboard_delete_all");
}

#[test]
fn batch23_clipboard_save_before_delete() {
    let entry = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let snippet_idx = actions
        .iter()
        .position(|a| a.id == "clipboard_save_snippet")
        .unwrap();
    let file_idx = actions
        .iter()
        .position(|a| a.id == "clipboard_save_file")
        .unwrap();
    let delete_idx = actions
        .iter()
        .position(|a| a.id == "clipboard_delete")
        .unwrap();
    assert!(snippet_idx < delete_idx);
    assert!(file_idx < delete_idx);
    assert!(file_idx == snippet_idx + 1);
}

// ============================================================
// 4. Clipboard context: attach_to_ai shortcut
// ============================================================

#[test]
fn batch23_clipboard_attach_to_ai_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let attach = actions
        .iter()
        .find(|a| a.id == "clipboard_attach_to_ai")
        .unwrap();
    assert_eq!(attach.shortcut.as_ref().unwrap(), "⌃⌘A");
}

#[test]
fn batch23_clipboard_attach_to_ai_description() {
    let entry = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "test".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let attach = actions
        .iter()
        .find(|a| a.id == "clipboard_attach_to_ai")
        .unwrap();
    assert!(attach
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("ai"));
}

#[test]
fn batch23_clipboard_attach_present_for_image() {
    let entry = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".to_string(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_attach_to_ai"));
}

// ============================================================
// 5. Path context: exact action IDs in order for directory
// ============================================================

#[test]
fn batch23_path_dir_action_ids_in_order() {
    let path = PathInfo::new("Documents", "/Users/test/Documents", true);
    let actions = get_path_context_actions(&path);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(ids[0], "open_directory");
    assert_eq!(ids[1], "copy_path");
    assert_eq!(ids[2], "open_in_finder");
    assert_eq!(ids[3], "open_in_editor");
    assert_eq!(ids[4], "open_in_terminal");
    assert_eq!(ids[5], "copy_filename");
    assert_eq!(ids[6], "move_to_trash");
}

#[test]
fn batch23_path_file_action_ids_in_order() {
    let path = PathInfo::new("readme.md", "/Users/test/readme.md", false);
    let actions = get_path_context_actions(&path);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(ids[0], "select_file");
    assert_eq!(ids[1], "copy_path");
    assert_eq!(ids[2], "open_in_finder");
    assert_eq!(ids[3], "open_in_editor");
    assert_eq!(ids[4], "open_in_terminal");
    assert_eq!(ids[5], "copy_filename");
    assert_eq!(ids[6], "move_to_trash");
}

#[test]
fn batch23_path_always_7_actions() {
    let dir = PathInfo::new("dir", "/dir", true);
    let file = PathInfo::new("file.txt", "/file.txt", false);
    assert_eq!(get_path_context_actions(&dir).len(), 7);
    assert_eq!(get_path_context_actions(&file).len(), 7);
}

// ============================================================
// 6. Path context: open_in_editor description mentions $EDITOR
// ============================================================

#[test]
fn batch23_path_open_in_editor_desc() {
    let path = PathInfo::new("test.txt", "/test.txt", false);
    let actions = get_path_context_actions(&path);
    let editor = actions.iter().find(|a| a.id == "open_in_editor").unwrap();
    assert!(editor.description.as_ref().unwrap().contains("$EDITOR"));
}

#[test]
fn batch23_path_open_in_terminal_desc() {
    let path = PathInfo::new("src", "/src", true);
    let actions = get_path_context_actions(&path);
    let terminal = actions.iter().find(|a| a.id == "open_in_terminal").unwrap();
    assert!(terminal
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("terminal"));
}

#[test]
fn batch23_path_copy_path_shortcut() {
    let path = PathInfo::new("test", "/test", false);
    let actions = get_path_context_actions(&path);
    let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
    assert_eq!(cp.shortcut.as_ref().unwrap(), "⌘⇧C");
}

// ============================================================
// 7. File context: shortcut matrix
// ============================================================

#[test]
fn batch23_file_open_shortcut_enter() {
    let file = FileInfo {
        path: "/test.txt".to_string(),
        name: "test.txt".to_string(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    let open = actions.iter().find(|a| a.id == "open_file").unwrap();
    assert_eq!(open.shortcut.as_ref().unwrap(), "↵");
}

#[test]
fn batch23_file_reveal_shortcut() {
    let file = FileInfo {
        path: "/test.txt".to_string(),
        name: "test.txt".to_string(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
    assert_eq!(reveal.shortcut.as_ref().unwrap(), "⌘↵");
}

#[test]
fn batch23_file_copy_path_shortcut() {
    let file = FileInfo {
        path: "/test.txt".to_string(),
        name: "test.txt".to_string(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
    assert_eq!(cp.shortcut.as_ref().unwrap(), "⌘⇧C");
}

#[test]
fn batch23_file_copy_filename_shortcut() {
    let file = FileInfo {
        path: "/test.txt".to_string(),
        name: "test.txt".to_string(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    let cf = actions.iter().find(|a| a.id == "copy_filename").unwrap();
    assert_eq!(cf.shortcut.as_ref().unwrap(), "⌘C");
}

// ============================================================
// 8. File context: title includes quoted file name
// ============================================================

#[test]
fn batch23_file_open_title_quotes_name() {
    let file = FileInfo {
        path: "/test/readme.md".to_string(),
        name: "readme.md".to_string(),
        file_type: FileType::Document,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    let open = actions.iter().find(|a| a.id == "open_file").unwrap();
    assert!(open.title.contains("\"readme.md\""));
}

#[test]
fn batch23_file_dir_open_title_quotes_name() {
    let dir = FileInfo {
        path: "/test/src".to_string(),
        name: "src".to_string(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&dir);
    let open = actions.iter().find(|a| a.id == "open_directory").unwrap();
    assert!(open.title.contains("\"src\""));
}

#[test]
fn batch23_file_open_dir_description() {
    let dir = FileInfo {
        path: "/test/docs".to_string(),
        name: "docs".to_string(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&dir);
    let open = actions.iter().find(|a| a.id == "open_directory").unwrap();
    assert!(open
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("folder"));
}

#[test]
fn batch23_file_open_file_description() {
    let file = FileInfo {
        path: "/test/notes.txt".to_string(),
        name: "notes.txt".to_string(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    let open = actions.iter().find(|a| a.id == "open_file").unwrap();
    assert!(open
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("application"));
}
