// Tests for action builder functions.
//
// Validates that built-in action builders produce correct actions with
// expected IDs, titles, shortcuts, and conditional behavior across
// multiple contexts: file search, path prompt, clipboard history,
// chat prompt, notes command bar, and AI command bar.

use super::*;
use crate::clipboard_history::ContentType;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;
use std::collections::HashSet;

// ============================================================
// Helper: extract action IDs from a Vec<Action>
// ============================================================

fn action_ids(actions: &[Action]) -> Vec<&str> {
    actions.iter().map(|a| a.id.as_str()).collect()
}

fn find_action<'a>(actions: &'a [Action], id: &str) -> Option<&'a Action> {
    actions.iter().find(|a| a.id == id)
}

// ============================================================
// 1. File context actions: files vs directories
// ============================================================

#[test]
fn file_context_directory_primary_action_is_open_directory() {
    let dir = FileInfo {
        path: "/Users/test/Documents".into(),
        name: "Documents".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&dir);
    let ids = action_ids(&actions);

    assert_eq!(
        actions[0].id, "open_directory",
        "First action for a directory must be open_directory"
    );
    assert!(
        actions[0].title.contains("Documents"),
        "Title should include directory name"
    );
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));

    // Directories should NOT have quick_look
    assert!(
        !ids.contains(&"quick_look"),
        "Directories should not have Quick Look"
    );
}

#[test]
fn file_context_file_primary_action_is_open_file() {
    let file = FileInfo {
        path: "/Users/test/photo.png".into(),
        name: "photo.png".into(),
        file_type: FileType::Image,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    let ids = action_ids(&actions);

    assert_eq!(
        actions[0].id, "open_file",
        "First action for a file must be open_file"
    );
    assert!(
        actions[0].title.contains("photo.png"),
        "Title should include filename"
    );

    // Files SHOULD have quick_look on macOS
    #[cfg(target_os = "macos")]
    assert!(
        ids.contains(&"quick_look"),
        "Files should have Quick Look on macOS"
    );
}

#[test]
fn file_context_common_actions_present_for_both() {
    let file = FileInfo {
        path: "/Users/test/readme.md".into(),
        name: "readme.md".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let dir = FileInfo {
        path: "/Users/test/src".into(),
        name: "src".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };

    for info in &[&file, &dir] {
        let actions = get_file_context_actions(info);
        let ids = action_ids(&actions);
        assert!(
            ids.contains(&"reveal_in_finder"),
            "{} should have reveal_in_finder",
            info.name
        );
        assert!(
            ids.contains(&"copy_path"),
            "{} should have copy_path",
            info.name
        );
        assert!(
            ids.contains(&"copy_filename"),
            "{} should have copy_filename",
            info.name
        );
    }
}

#[test]
fn file_context_macos_specific_actions() {
    let file = FileInfo {
        path: "/Users/test/data.csv".into(),
        name: "data.csv".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);
    let ids = action_ids(&actions);

    #[cfg(target_os = "macos")]
    {
        assert!(ids.contains(&"open_with"), "macOS should have Open With...");
        assert!(ids.contains(&"show_info"), "macOS should have Show Info");
    }

    #[cfg(not(target_os = "macos"))]
    {
        assert!(
            !ids.contains(&"open_with"),
            "Non-macOS should not have Open With"
        );
        assert!(
            !ids.contains(&"show_info"),
            "Non-macOS should not have Show Info"
        );
    }
}

#[test]
fn file_context_finder_labels_use_reveal_consistently() {
    let file = FileInfo {
        path: "/Users/test/data.csv".into(),
        name: "data.csv".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file);

    let reveal = find_action(&actions, "reveal_in_finder").expect("missing reveal_in_finder");
    assert_eq!(reveal.title, "Reveal in Finder");

    #[cfg(target_os = "macos")]
    {
        let show_info = find_action(&actions, "show_info").expect("missing show_info");
        assert_eq!(show_info.title, "Show Info");
    }
}

// ============================================================
// 2. Path context actions: files vs directories
// ============================================================

#[test]
fn path_context_directory_has_open_directory_as_primary() {
    let path = PathInfo::new("Projects", "/Users/test/Projects", true);
    let actions = get_path_context_actions(&path);

    assert_eq!(actions[0].id, "open_directory");
    assert!(actions[0].title.contains("Projects"));
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn path_context_file_has_select_file_as_primary() {
    let path = PathInfo::new("config.toml", "/Users/test/config.toml", false);
    let actions = get_path_context_actions(&path);

    assert_eq!(actions[0].id, "select_file");
    assert!(actions[0].title.contains("config.toml"));
    assert_eq!(actions[0].shortcut.as_deref(), Some("↵"));
}

#[test]
fn path_context_all_common_actions_present() {
    let path = PathInfo::new("file.txt", "/tmp/file.txt", false);
    let actions = get_path_context_actions(&path);
    let ids = action_ids(&actions);

    let expected = [
        "copy_path",
        "open_in_finder",
        "open_in_editor",
        "open_in_terminal",
        "copy_filename",
        "move_to_trash",
    ];
    for &expected_id in &expected {
        assert!(
            ids.contains(&expected_id),
            "Path context should have {}",
            expected_id
        );
    }
}

#[test]
fn path_context_finder_label_uses_reveal() {
    let path = PathInfo::new("file.txt", "/tmp/file.txt", false);
    let actions = get_path_context_actions(&path);
    let reveal = find_action(&actions, "open_in_finder").expect("missing open_in_finder");
    assert_eq!(reveal.title, "Reveal in Finder");
}

#[test]
fn path_context_move_to_trash_description_matches_type() {
    let dir_path = PathInfo::new("build", "/tmp/build", true);
    let file_path = PathInfo::new("temp.log", "/tmp/temp.log", false);

    let dir_actions = get_path_context_actions(&dir_path);
    let file_actions = get_path_context_actions(&file_path);

    let dir_trash = find_action(&dir_actions, "move_to_trash").unwrap();
    let file_trash = find_action(&file_actions, "move_to_trash").unwrap();

    assert!(
        dir_trash.description.as_ref().unwrap().contains("folder"),
        "Directory trash description should mention 'folder'"
    );
    assert!(
        file_trash.description.as_ref().unwrap().contains("file"),
        "File trash description should mention 'file'"
    );
}

// ============================================================
// 3. Clipboard history actions: text vs image, pinned vs unpinned
// ============================================================

fn make_text_entry(pinned: bool) -> ClipboardEntryInfo {
    ClipboardEntryInfo {
        id: "entry-1".into(),
        content_type: ContentType::Text,
        pinned,
        preview: "Hello world".into(),
        image_dimensions: None,
        frontmost_app_name: Some("VS Code".into()),
    }
}

fn make_image_entry(pinned: bool) -> ClipboardEntryInfo {
    ClipboardEntryInfo {
        id: "entry-2".into(),
        content_type: ContentType::Image,
        pinned,
        preview: String::new(),
        image_dimensions: Some((800, 600)),
        frontmost_app_name: None,
    }
}

#[test]
fn clipboard_text_entry_has_core_actions() {
    let entry = make_text_entry(false);
    let actions = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions);

    let expected = [
        "clipboard_paste",
        "clipboard_copy",
        "clipboard_paste_keep_open",
        "clipboard_share",
        "clipboard_attach_to_ai",
        "clipboard_save_snippet",
        "clipboard_save_file",
        "clipboard_delete",
        "clipboard_delete_multiple",
        "clipboard_delete_all",
    ];
    for &expected_id in &expected {
        assert!(
            ids.contains(&expected_id),
            "Text entry should have {}",
            expected_id
        );
    }
}

#[test]
fn clipboard_paste_title_includes_app_name() {
    let entry = make_text_entry(false);
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = find_action(&actions, "clipboard_paste").unwrap();

    assert_eq!(paste.title, "Paste to VS Code");
}

#[test]
fn clipboard_paste_title_fallback_when_no_app() {
    let entry = make_image_entry(false);
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = find_action(&actions, "clipboard_paste").unwrap();

    assert_eq!(paste.title, "Paste to Active App");
}

#[test]
fn clipboard_image_entry_has_ocr_action() {
    let entry = make_image_entry(false);
    let actions = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions);

    assert!(
        ids.contains(&"clipboard_ocr"),
        "Image entries should have OCR action"
    );

    // macOS image-specific actions
    #[cfg(target_os = "macos")]
    {
        assert!(
            ids.contains(&"clipboard_open_with"),
            "macOS image should have Open With"
        );
        assert!(
            ids.contains(&"clipboard_annotate_cleanshot"),
            "macOS image should have CleanShot annotate"
        );
        assert!(
            ids.contains(&"clipboard_upload_cleanshot"),
            "macOS image should have CleanShot upload"
        );
    }
}

#[test]
fn clipboard_text_entry_lacks_image_specific_actions() {
    let entry = make_text_entry(false);
    let actions = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions);

    assert!(
        !ids.contains(&"clipboard_ocr"),
        "Text entries should NOT have OCR"
    );
    assert!(
        !ids.contains(&"clipboard_open_with"),
        "Text entries should NOT have Open With"
    );
    assert!(
        !ids.contains(&"clipboard_annotate_cleanshot"),
        "Text entries should NOT have CleanShot annotate"
    );
}

#[test]
fn clipboard_pinned_entry_shows_unpin() {
    let entry = make_text_entry(true);
    let actions = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions);

    assert!(
        ids.contains(&"clipboard_unpin"),
        "Pinned entry should show Unpin"
    );
    assert!(
        !ids.contains(&"clipboard_pin"),
        "Pinned entry should NOT show Pin"
    );
}

#[test]
fn clipboard_unpinned_entry_shows_pin() {
    let entry = make_text_entry(false);
    let actions = get_clipboard_history_context_actions(&entry);
    let ids = action_ids(&actions);

    assert!(
        ids.contains(&"clipboard_pin"),
        "Unpinned entry should show Pin"
    );
    assert!(
        !ids.contains(&"clipboard_unpin"),
        "Unpinned entry should NOT show Unpin"
    );
}

