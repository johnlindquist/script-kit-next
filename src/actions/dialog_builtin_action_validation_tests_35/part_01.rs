//! Batch 35: Dialog built-in action validation tests
//!
//! 116 tests across 30 categories validating random behaviors from
//! built-in action window dialogs.

use crate::actions::builders::{
    get_ai_command_bar_actions, get_chat_context_actions, get_clipboard_history_context_actions,
    get_file_context_actions, get_new_chat_actions, get_note_switcher_actions,
    get_notes_command_bar_actions, get_path_context_actions, get_script_context_actions,
    get_scriptlet_context_actions_with_custom, to_deeplink_name, ChatModelInfo, ChatPromptInfo,
    ClipboardEntryInfo, NewChatModelInfo, NewChatPresetInfo, NoteSwitcherNoteInfo, NotesInfo,
};
use crate::actions::command_bar::CommandBarConfig;
use crate::actions::constants::{
    ACCENT_BAR_WIDTH, ACTION_ROW_INSET, HEADER_HEIGHT, KEYCAP_HEIGHT, KEYCAP_MIN_WIDTH,
    SEARCH_INPUT_HEIGHT, SECTION_HEADER_HEIGHT, SELECTION_RADIUS,
};
use crate::actions::dialog::{build_grouped_items_static, coerce_action_selection, ActionsDialog};
use crate::actions::types::{Action, ActionCategory, ScriptInfo, SectionStyle};
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::FileInfo;
use crate::prompts::PathInfo;
use crate::scriptlets::{Scriptlet, ScriptletAction};

// =====================================================================
// 1. Clipboard: attach_to_ai shortcut and description
// =====================================================================

#[test]
fn clipboard_attach_to_ai_shortcut_is_ctrl_cmd_a() {
    let entry = ClipboardEntryInfo {
        id: "ai-1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "some text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let action = actions
        .iter()
        .find(|a| a.id == "clipboard_attach_to_ai")
        .unwrap();
    assert_eq!(action.shortcut.as_ref().unwrap(), "⌃⌘A");
}

#[test]
fn clipboard_attach_to_ai_title() {
    let entry = ClipboardEntryInfo {
        id: "ai-2".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let action = actions
        .iter()
        .find(|a| a.id == "clipboard_attach_to_ai")
        .unwrap();
    assert_eq!(action.title, "Attach to AI Chat");
}

#[test]
fn clipboard_attach_to_ai_desc_mentions_ai() {
    let entry = ClipboardEntryInfo {
        id: "ai-3".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let action = actions
        .iter()
        .find(|a| a.id == "clipboard_attach_to_ai")
        .unwrap();
    assert!(action
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("ai"));
}

#[test]
fn clipboard_attach_to_ai_present_for_image() {
    let entry = ClipboardEntryInfo {
        id: "ai-4".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((640, 480)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_attach_to_ai"));
}

// =====================================================================
// 2. Clipboard: total action count text vs image on macOS
// =====================================================================

#[cfg(target_os = "macos")]
#[test]
fn clipboard_text_action_count_macos() {
    let entry = ClipboardEntryInfo {
        id: "cnt-1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    // Text on macOS: paste, copy, paste_keep_open, share, attach_to_ai, quick_look, pin,
    // save_snippet, save_file, delete, delete_multiple, delete_all = 12
    assert_eq!(actions.len(), 12);
}

#[cfg(target_os = "macos")]
#[test]
fn clipboard_image_action_count_macos() {
    let entry = ClipboardEntryInfo {
        id: "cnt-2".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".into(),
        image_dimensions: Some((800, 600)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    // Image on macOS: paste, copy, paste_keep_open, share, attach_to_ai, quick_look,
    // open_with, annotate_cleanshot, upload_cleanshot, pin, ocr,
    // save_snippet, save_file, delete, delete_multiple, delete_all = 16
    assert_eq!(actions.len(), 16);
}

#[cfg(target_os = "macos")]
#[test]
fn clipboard_image_has_4_more_actions_than_text_on_macos() {
    let text_entry = ClipboardEntryInfo {
        id: "cnt-3".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let img_entry = ClipboardEntryInfo {
        id: "cnt-4".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "i".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let text_count = get_clipboard_history_context_actions(&text_entry).len();
    let img_count = get_clipboard_history_context_actions(&img_entry).len();
    // Image has 4 more: open_with, annotate_cleanshot, upload_cleanshot, ocr
    assert_eq!(img_count - text_count, 4);
}

#[test]
fn clipboard_pinned_vs_unpinned_same_count() {
    let pinned = ClipboardEntryInfo {
        id: "cnt-5".into(),
        content_type: ContentType::Text,
        pinned: true,
        preview: "p".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let unpinned = ClipboardEntryInfo {
        id: "cnt-6".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "u".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    // pin/unpin swapped but same count
    assert_eq!(
        get_clipboard_history_context_actions(&pinned).len(),
        get_clipboard_history_context_actions(&unpinned).len()
    );
}

// =====================================================================
// 3. File context: primary action title is quoted with file name
// =====================================================================

#[test]
fn file_context_file_title_quoted() {
    let file_info = FileInfo {
        path: "/Users/test/readme.md".into(),
        name: "readme.md".into(),
        file_type: crate::file_search::FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file_info);
    let primary = actions.first().unwrap();
    assert_eq!(primary.title, "Open \"readme.md\"");
}

#[test]
fn file_context_dir_title_quoted() {
    let file_info = FileInfo {
        path: "/Users/test/Documents".into(),
        name: "Documents".into(),
        file_type: crate::file_search::FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&file_info);
    let primary = actions.first().unwrap();
    assert_eq!(primary.title, "Open \"Documents\"");
}

#[test]
fn file_context_file_primary_shortcut_enter() {
    let file_info = FileInfo {
        path: "/test/file.txt".into(),
        name: "file.txt".into(),
        file_type: crate::file_search::FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&file_info);
    assert_eq!(actions[0].shortcut.as_ref().unwrap(), "↵");
}

#[test]
fn file_context_dir_primary_shortcut_enter() {
    let file_info = FileInfo {
        path: "/test/dir".into(),
        name: "dir".into(),
        file_type: crate::file_search::FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&file_info);
    assert_eq!(actions[0].shortcut.as_ref().unwrap(), "↵");
}

// =====================================================================
// 4. Path context: primary action description varies for file vs dir
// =====================================================================

#[test]
fn path_file_primary_desc_is_submit() {
    let path_info = PathInfo {
        path: "/Users/test/file.txt".into(),
        name: "file.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    let primary = actions.first().unwrap();
    assert!(primary.description.as_ref().unwrap().contains("Submit"));
}

#[test]
fn path_dir_primary_desc_is_navigate() {
    let path_info = PathInfo {
        path: "/Users/test/docs".into(),
        name: "docs".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path_info);
    let primary = actions.first().unwrap();
    assert!(primary.description.as_ref().unwrap().contains("Navigate"));
}

#[test]
fn path_file_primary_id_is_select_file() {
    let path_info = PathInfo {
        path: "/test/a.txt".into(),
        name: "a.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&path_info);
    assert_eq!(actions[0].id, "select_file");
}

#[test]
fn path_dir_primary_id_is_open_directory() {
    let path_info = PathInfo {
        path: "/test/dir".into(),
        name: "dir".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&path_info);
    assert_eq!(actions[0].id, "open_directory");
}

// =====================================================================
// 5. Script context: edit shortcut ⌘E across types
// =====================================================================

#[test]
fn script_edit_shortcut_cmd_e() {
    let script = ScriptInfo::new("my-script", "/path/to/script.ts");
    let actions = get_script_context_actions(&script);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert_eq!(edit.shortcut.as_ref().unwrap(), "⌘E");
}

#[test]
fn scriptlet_edit_shortcut_cmd_e() {
    let scriptlet = ScriptInfo::scriptlet("Open URL", "/path/url.md", None, None);
    let actions = get_script_context_actions(&scriptlet);
    let edit = actions.iter().find(|a| a.id == "edit_scriptlet").unwrap();
    assert_eq!(edit.shortcut.as_ref().unwrap(), "⌘E");
}

#[test]
fn agent_edit_shortcut_cmd_e() {
    let mut agent = ScriptInfo::new("My Agent", "/path/agent");
    agent.is_script = false;
    agent.is_agent = true;
    let actions = get_script_context_actions(&agent);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert_eq!(edit.shortcut.as_ref().unwrap(), "⌘E");
}

#[test]
fn script_copy_content_shortcut_cmd_opt_c() {
    let script = ScriptInfo::new("my-script", "/path/to/script.ts");
    let actions = get_script_context_actions(&script);
    let copy = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert_eq!(copy.shortcut.as_ref().unwrap(), "⌘⌥C");
}

// =====================================================================
// 6. Scriptlet with custom H3 actions: ID prefix, has_action, value
// =====================================================================

#[test]
fn scriptlet_custom_action_id_prefix() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut s = Scriptlet::new("Test".into(), "bash".into(), "echo hi".into());
    s.actions = vec![ScriptletAction {
        name: "My Custom".into(),
        command: "my-custom".into(),
        tool: "bash".into(),
        code: "echo custom".into(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&s));
    assert!(actions.iter().any(|a| a.id == "scriptlet_action:my-custom"));
}

#[test]
fn scriptlet_custom_action_has_action_true() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut s = Scriptlet::new("Test".into(), "bash".into(), "echo hi".into());
    s.actions = vec![ScriptletAction {
        name: "Custom".into(),
        command: "cmd".into(),
        tool: "bash".into(),
        code: "echo".into(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&s));
    let custom = actions
        .iter()
        .find(|a| a.id == "scriptlet_action:cmd")
        .unwrap();
    assert!(custom.has_action);
}

#[test]
fn scriptlet_custom_action_value_is_command() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut s = Scriptlet::new("Test".into(), "bash".into(), "echo hi".into());
    s.actions = vec![ScriptletAction {
        name: "Copy It".into(),
        command: "copy-it".into(),
        tool: "bash".into(),
        code: "pbcopy".into(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&s));
    let custom = actions
        .iter()
        .find(|a| a.id == "scriptlet_action:copy-it")
        .unwrap();
    assert_eq!(custom.value.as_ref().unwrap(), "copy-it");
}

#[test]
fn scriptlet_builtin_actions_has_action_false() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    for a in &actions {
        assert!(
            !a.has_action,
            "Built-in action {} should have has_action=false",
            a.id
        );
    }
}

// =====================================================================
// 7. Scriptlet custom action with shortcut gets format_shortcut_hint applied
// =====================================================================

#[test]
fn scriptlet_custom_action_shortcut_formatted() {
    let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
    let mut s = Scriptlet::new("Test".into(), "bash".into(), "echo".into());
    s.actions = vec![ScriptletAction {
        name: "Action".into(),
        command: "act".into(),
        tool: "bash".into(),
        code: "echo".into(),
        inputs: vec![],
        shortcut: Some("cmd+shift+x".into()),
        description: Some("Do something".into()),
    }];
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&s));
    let custom = actions
        .iter()
        .find(|a| a.id == "scriptlet_action:act")
        .unwrap();
    assert_eq!(custom.shortcut.as_ref().unwrap(), "⌘⇧X");
}
