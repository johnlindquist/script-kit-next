// --- merged from part_01.rs ---
//! Batch 23: Dialog builtin action validation tests
//!
//! 30 categories of tests validating random built-in action behaviors.

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
    assert_eq!(actions[0].id, "clip:clipboard_paste");
    assert_eq!(actions[1].id, "clip:clipboard_copy");
    assert_eq!(actions[2].id, "clip:clipboard_paste_keep_open");
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
    assert_eq!(actions[3].id, "clip:clipboard_share");
    assert_eq!(actions[4].id, "clip:clipboard_attach_to_ai");
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
    assert_eq!(actions[len - 3].id, "clip:clipboard_delete");
    assert_eq!(actions[len - 2].id, "clip:clipboard_delete_multiple");
    assert_eq!(actions[len - 1].id, "clip:clipboard_delete_all");
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
        .position(|a| a.id == "clip:clipboard_save_snippet")
        .unwrap();
    let file_idx = actions
        .iter()
        .position(|a| a.id == "clip:clipboard_save_file")
        .unwrap();
    let delete_idx = actions
        .iter()
        .position(|a| a.id == "clip:clipboard_delete")
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
        .find(|a| a.id == "clip:clipboard_attach_to_ai")
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
        .find(|a| a.id == "clip:clipboard_attach_to_ai")
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
    assert!(actions.iter().any(|a| a.id == "clip:clipboard_attach_to_ai"));
}

// ============================================================
// 5. Path context: exact action IDs in order for directory
// ============================================================

#[test]
fn batch23_path_dir_action_ids_in_order() {
    let path = PathInfo::new("Documents", "/Users/test/Documents", true);
    let actions = get_path_context_actions(&path);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(ids[0], "file:open_directory");
    assert_eq!(ids[1], "file:copy_path");
    assert_eq!(ids[2], "file:open_in_finder");
    assert_eq!(ids[3], "file:open_in_editor");
    assert_eq!(ids[4], "file:open_in_terminal");
    assert_eq!(ids[5], "file:copy_filename");
    assert_eq!(ids[6], "file:move_to_trash");
}

#[test]
fn batch23_path_file_action_ids_in_order() {
    let path = PathInfo::new("readme.md", "/Users/test/readme.md", false);
    let actions = get_path_context_actions(&path);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(ids[0], "file:select_file");
    assert_eq!(ids[1], "file:copy_path");
    assert_eq!(ids[2], "file:open_in_finder");
    assert_eq!(ids[3], "file:open_in_editor");
    assert_eq!(ids[4], "file:open_in_terminal");
    assert_eq!(ids[5], "file:copy_filename");
    assert_eq!(ids[6], "file:move_to_trash");
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
    let editor = actions.iter().find(|a| a.id == "file:open_in_editor").unwrap();
    assert!(editor.description.as_ref().unwrap().contains("$EDITOR"));
}

#[test]
fn batch23_path_open_in_terminal_desc() {
    let path = PathInfo::new("src", "/src", true);
    let actions = get_path_context_actions(&path);
    let terminal = actions.iter().find(|a| a.id == "file:open_in_terminal").unwrap();
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
    let cp = actions.iter().find(|a| a.id == "file:copy_path").unwrap();
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
    let open = actions.iter().find(|a| a.id == "file:open_file").unwrap();
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
    let reveal = actions.iter().find(|a| a.id == "file:reveal_in_finder").unwrap();
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
    let cp = actions.iter().find(|a| a.id == "file:copy_path").unwrap();
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
    let cf = actions.iter().find(|a| a.id == "file:copy_filename").unwrap();
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
    let open = actions.iter().find(|a| a.id == "file:open_file").unwrap();
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
    let open = actions.iter().find(|a| a.id == "file:open_directory").unwrap();
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
    let open = actions.iter().find(|a| a.id == "file:open_directory").unwrap();
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
    let open = actions.iter().find(|a| a.id == "file:open_file").unwrap();
    assert!(open
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("application"));
}

// --- merged from part_02.rs ---

// ============================================================
// 9. AI command bar: action shortcut presence/absence matrix
// ============================================================

#[test]
fn batch23_ai_copy_response_has_shortcut() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "chat:copy_response").unwrap();
    assert_eq!(a.shortcut.as_ref().unwrap(), "⇧⌘C");
}

#[test]
fn batch23_ai_copy_chat_has_shortcut() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "chat:copy_chat").unwrap();
    assert_eq!(a.shortcut.as_ref().unwrap(), "⌥⇧⌘C");
}

#[test]
fn batch23_ai_copy_last_code_has_shortcut() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "chat:copy_last_code").unwrap();
    assert_eq!(a.shortcut.as_ref().unwrap(), "⌥⌘C");
}

#[test]
fn batch23_ai_branch_from_last_no_shortcut() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "chat:branch_from_last").unwrap();
    assert!(a.shortcut.is_none());
}

#[test]
fn batch23_ai_change_model_no_shortcut() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "chat:change_model").unwrap();
    assert!(a.shortcut.is_none());
}

// ============================================================
// 10. AI command bar: description content validation
// ============================================================

#[test]
fn batch23_ai_submit_description() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "chat:submit").unwrap();
    assert!(a
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("send"));
}

#[test]
fn batch23_ai_new_chat_description() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "chat:new_chat").unwrap();
    assert!(a
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("new"));
}

#[test]
fn batch23_ai_delete_chat_description() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "chat:delete_chat").unwrap();
    assert!(a
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("delete"));
}

#[test]
fn batch23_ai_export_markdown_description() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "chat:export_markdown").unwrap();
    assert!(a
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("markdown"));
}

#[test]
fn batch23_ai_paste_image_description() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "chat:paste_image").unwrap();
    assert!(a
        .description
        .as_ref()
        .unwrap()
        .to_lowercase()
        .contains("clipboard"));
}

// ============================================================
// 11. Chat context: model IDs use select_model_ prefix
// ============================================================

#[test]
fn batch23_chat_model_id_prefix() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "gpt-4".to_string(),
            display_name: "GPT-4".to_string(),
            provider: "OpenAI".to_string(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions[0].id.starts_with("select_model_"));
    assert_eq!(actions[0].id, "select_model_gpt-4");
}

#[test]
fn batch23_chat_multiple_models_sequential_ids() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![
            ChatModelInfo {
                id: "claude-3".to_string(),
                display_name: "Claude 3".to_string(),
                provider: "Anthropic".to_string(),
            },
            ChatModelInfo {
                id: "gpt-4".to_string(),
                display_name: "GPT-4".to_string(),
                provider: "OpenAI".to_string(),
            },
        ],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions[0].id, "select_model_claude-3");
    assert_eq!(actions[1].id, "select_model_gpt-4");
}

#[test]
fn batch23_chat_model_descriptions_via_provider() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "opus".to_string(),
            display_name: "Claude Opus".to_string(),
            provider: "Anthropic".to_string(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions[0].description.as_ref().unwrap(), "via Anthropic");
}

#[test]
fn batch23_chat_current_model_checkmark() {
    let info = ChatPromptInfo {
        current_model: Some("GPT-4".to_string()),
        available_models: vec![
            ChatModelInfo {
                id: "gpt-4".to_string(),
                display_name: "GPT-4".to_string(),
                provider: "OpenAI".to_string(),
            },
            ChatModelInfo {
                id: "claude-3".to_string(),
                display_name: "Claude 3".to_string(),
                provider: "Anthropic".to_string(),
            },
        ],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions[0].title.contains("✓"));
    assert!(!actions[1].title.contains("✓"));
}

// ============================================================
// 12. Chat context: continue_in_chat is always present
// ============================================================

#[test]
fn batch23_chat_continue_in_chat_always_present() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "chat:continue_in_chat"));
}

#[test]
fn batch23_chat_continue_in_chat_shortcut() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let c = actions.iter().find(|a| a.id == "chat:continue_in_chat").unwrap();
    assert_eq!(c.shortcut.as_ref().unwrap(), "⌘↵");
}

#[test]
fn batch23_chat_continue_after_models() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "m1".to_string(),
            display_name: "M1".to_string(),
            provider: "P".to_string(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let model_idx = actions
        .iter()
        .position(|a| a.id.starts_with("select_model_"))
        .unwrap();
    let continue_idx = actions
        .iter()
        .position(|a| a.id == "chat:continue_in_chat")
        .unwrap();
    assert!(continue_idx > model_idx);
}

// ============================================================
// 13. Notes command bar: section icon assignments
// ============================================================

#[test]
fn batch23_notes_new_note_icon() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let note = actions.iter().find(|a| a.id == "notes:new_note").unwrap();
    assert_eq!(note.icon, Some(IconName::Plus));
}

#[test]
fn batch23_notes_browse_notes_icon() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let browse = actions.iter().find(|a| a.id == "browse_notes").unwrap();
    assert_eq!(browse.icon, Some(IconName::FolderOpen));
}

#[test]
fn batch23_notes_find_in_note_icon() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let find = actions.iter().find(|a| a.id == "find_in_note").unwrap();
    assert_eq!(find.icon, Some(IconName::MagnifyingGlass));
}

#[test]
fn batch23_notes_format_icon() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let fmt = actions.iter().find(|a| a.id == "format").unwrap();
    assert_eq!(fmt.icon, Some(IconName::Code));
}

#[test]
fn batch23_notes_enable_auto_sizing_icon() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let auto = actions
        .iter()
        .find(|a| a.id == "enable_auto_sizing")
        .unwrap();
    assert_eq!(auto.icon, Some(IconName::Settings));
}

// ============================================================
// 14. Notes command bar: shortcut assignments
// ============================================================

#[test]
fn batch23_notes_new_note_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let note = actions.iter().find(|a| a.id == "notes:new_note").unwrap();
    assert_eq!(note.shortcut.as_ref().unwrap(), "⌘N");
}

#[test]
fn batch23_notes_browse_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let browse = actions.iter().find(|a| a.id == "browse_notes").unwrap();
    assert_eq!(browse.shortcut.as_ref().unwrap(), "⌘P");
}

#[test]
fn batch23_notes_duplicate_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let dup = actions.iter().find(|a| a.id == "duplicate_note").unwrap();
    assert_eq!(dup.shortcut.as_ref().unwrap(), "⌘D");
}

#[test]
fn batch23_notes_format_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let fmt = actions.iter().find(|a| a.id == "format").unwrap();
    assert_eq!(fmt.shortcut.as_ref().unwrap(), "⇧⌘T");
}

// ============================================================
// 15. New chat actions: empty inputs produce empty output
// ============================================================

#[test]
fn batch23_new_chat_all_empty() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert!(actions.is_empty());
}

#[test]
fn batch23_new_chat_only_last_used() {
    let last_used = vec![NewChatModelInfo {
        model_id: "claude".to_string(),
        display_name: "Claude".to_string(),
        provider: "anthropic".to_string(),
        provider_display_name: "Anthropic".to_string(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_ref().unwrap(), "Last Used Settings");
}

#[test]
fn batch23_new_chat_only_presets() {
    let presets = vec![NewChatPresetInfo {
        id: "general".to_string(),
        name: "General".to_string(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_ref().unwrap(), "Presets");
}

#[test]
fn batch23_new_chat_only_models() {
    let models = vec![NewChatModelInfo {
        model_id: "gpt4".to_string(),
        display_name: "GPT-4".to_string(),
        provider: "openai".to_string(),
        provider_display_name: "OpenAI".to_string(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_ref().unwrap(), "Models");
}

#[test]
fn batch23_new_chat_mixed_sections_count() {
    let last = vec![NewChatModelInfo {
        model_id: "c".to_string(),
        display_name: "C".to_string(),
        provider: "p".to_string(),
        provider_display_name: "P".to_string(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "g".to_string(),
        name: "G".to_string(),
        icon: IconName::Star,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "m".to_string(),
        display_name: "M".to_string(),
        provider: "p".to_string(),
        provider_display_name: "P".to_string(),
    }];
    let actions = get_new_chat_actions(&last, &presets, &models);
    assert_eq!(actions.len(), 3);
}

// ============================================================
// 16. New chat actions: icon assignments
// ============================================================

#[test]
fn batch23_new_chat_last_used_icon() {
    let last = vec![NewChatModelInfo {
        model_id: "c".to_string(),
        display_name: "C".to_string(),
        provider: "p".to_string(),
        provider_display_name: "P".to_string(),
    }];
    let actions = get_new_chat_actions(&last, &[], &[]);
    assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
}

// --- merged from part_03.rs ---

#[test]
fn batch23_new_chat_model_icon() {
    let models = vec![NewChatModelInfo {
        model_id: "m".to_string(),
        display_name: "M".to_string(),
        provider: "p".to_string(),
        provider_display_name: "P".to_string(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].icon, Some(IconName::Settings));
}

#[test]
fn batch23_new_chat_preset_icon_preserved() {
    let presets = vec![NewChatPresetInfo {
        id: "code".to_string(),
        name: "Code".to_string(),
        icon: IconName::Code,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].icon, Some(IconName::Code));
}

#[test]
fn batch23_new_chat_preset_no_description() {
    let presets = vec![NewChatPresetInfo {
        id: "g".to_string(),
        name: "General".to_string(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert!(actions[0].description.is_none());
}

// ============================================================
// 17. Note switcher: empty list produces placeholder
// ============================================================

#[test]
fn batch23_note_switcher_empty_placeholder_id() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "no_notes");
}

#[test]
fn batch23_note_switcher_empty_placeholder_title() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions[0].title, "No notes yet");
}

#[test]
fn batch23_note_switcher_empty_placeholder_icon() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions[0].icon, Some(IconName::Plus));
}

#[test]
fn batch23_note_switcher_empty_placeholder_section() {
    let actions = get_note_switcher_actions(&[]);
    assert_eq!(actions[0].section.as_ref().unwrap(), "Notes");
}

#[test]
fn batch23_note_switcher_empty_placeholder_description() {
    let actions = get_note_switcher_actions(&[]);
    assert!(actions[0].description.as_ref().unwrap().contains("⌘N"));
}

// ============================================================
// 18. Note switcher: multi-note section assignment
// ============================================================

#[test]
fn batch23_note_switcher_pinned_and_recent_sections() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "1".to_string(),
            title: "Pinned Note".to_string(),
            char_count: 50,
            is_current: false,
            is_pinned: true,
            preview: "pinned content".to_string(),
            relative_time: "1h ago".to_string(),
        },
        NoteSwitcherNoteInfo {
            id: "2".to_string(),
            title: "Recent Note".to_string(),
            char_count: 30,
            is_current: false,
            is_pinned: false,
            preview: "recent content".to_string(),
            relative_time: "5m ago".to_string(),
        },
    ];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].section.as_ref().unwrap(), "Pinned");
    assert_eq!(actions[1].section.as_ref().unwrap(), "Recent");
}

#[test]
fn batch23_note_switcher_all_pinned() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "1".to_string(),
            title: "A".to_string(),
            char_count: 10,
            is_current: false,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "2".to_string(),
            title: "B".to_string(),
            char_count: 20,
            is_current: false,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        },
    ];
    let actions = get_note_switcher_actions(&notes);
    assert!(actions
        .iter()
        .all(|a| a.section.as_ref().unwrap() == "Pinned"));
}

#[test]
fn batch23_note_switcher_all_recent() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".to_string(),
        title: "A".to_string(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].section.as_ref().unwrap(), "Recent");
}

// ============================================================
// 19. Note switcher: id format uses note_{uuid}
// ============================================================

#[test]
fn batch23_note_switcher_id_format() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc-123".to_string(),
        title: "Test".to_string(),
        char_count: 5,
        is_current: false,
        is_pinned: false,
        preview: String::new(),
        relative_time: String::new(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].id, "note_abc-123");
}

#[test]
fn batch23_note_switcher_multiple_ids_unique() {
    let notes = vec![
        NoteSwitcherNoteInfo {
            id: "a".to_string(),
            title: "A".to_string(),
            char_count: 1,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
        NoteSwitcherNoteInfo {
            id: "b".to_string(),
            title: "B".to_string(),
            char_count: 2,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        },
    ];
    let actions = get_note_switcher_actions(&notes);
    assert_ne!(actions[0].id, actions[1].id);
}

// ============================================================
// 20. Scriptlet defined actions: has_action and value
// ============================================================

#[test]
fn batch23_scriptlet_defined_has_action_true() {
    let mut scriptlet = Scriptlet::new("T".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![ScriptletAction {
        name: "Act".to_string(),
        command: "act-cmd".to_string(),
        tool: "bash".to_string(),
        code: "echo act".to_string(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert!(actions[0].has_action);
}

#[test]
fn batch23_scriptlet_defined_value_is_command() {
    let mut scriptlet = Scriptlet::new("T".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![ScriptletAction {
        name: "Copy".to_string(),
        command: "copy-cmd".to_string(),
        tool: "bash".to_string(),
        code: "pbcopy".to_string(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert_eq!(actions[0].value.as_ref().unwrap(), "copy-cmd");
}

#[test]
fn batch23_scriptlet_defined_id_prefix() {
    let mut scriptlet = Scriptlet::new("T".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![ScriptletAction {
        name: "My Action".to_string(),
        command: "my-action".to_string(),
        tool: "bash".to_string(),
        code: "echo".to_string(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert!(actions[0].id.starts_with("scriptlet_action:"));
    assert_eq!(actions[0].id, "scriptlet_action:my-action");
}

#[test]
fn batch23_scriptlet_defined_shortcut_formatted() {
    let mut scriptlet = Scriptlet::new("T".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![ScriptletAction {
        name: "Test".to_string(),
        command: "test".to_string(),
        tool: "bash".to_string(),
        code: "echo".to_string(),
        inputs: vec![],
        shortcut: Some("cmd+shift+x".to_string()),
        description: None,
    }];
    let actions = get_scriptlet_defined_actions(&scriptlet);
    assert_eq!(actions[0].shortcut.as_ref().unwrap(), "⌘⇧X");
}

// ============================================================
// 21. Scriptlet context with custom: ordering of custom vs built-in
// ============================================================

#[test]
fn batch23_scriptlet_custom_between_run_and_shortcut() {
    let script = ScriptInfo::scriptlet("Test", "/test.md", None, None);
    let mut scriptlet = Scriptlet::new("Test".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![ScriptletAction {
        name: "Custom".to_string(),
        command: "custom".to_string(),
        tool: "bash".to_string(),
        code: "echo".to_string(),
        inputs: vec![],
        shortcut: None,
        description: None,
    }];
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let run_idx = actions.iter().position(|a| a.id == "run_script").unwrap();
    let custom_idx = actions
        .iter()
        .position(|a| a.id == "scriptlet_action:custom")
        .unwrap();
    let shortcut_idx = actions.iter().position(|a| a.id == "add_shortcut").unwrap();
    assert_eq!(run_idx, 0);
    assert_eq!(custom_idx, 1);
    assert!(shortcut_idx > custom_idx);
}

#[test]
fn batch23_scriptlet_custom_multiple_preserve_order() {
    let script = ScriptInfo::scriptlet("Test", "/test.md", None, None);
    let mut scriptlet = Scriptlet::new("Test".to_string(), "bash".to_string(), "echo".to_string());
    scriptlet.actions = vec![
        ScriptletAction {
            name: "First".to_string(),
            command: "first".to_string(),
            tool: "bash".to_string(),
            code: "echo 1".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        },
        ScriptletAction {
            name: "Second".to_string(),
            command: "second".to_string(),
            tool: "bash".to_string(),
            code: "echo 2".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        },
    ];
    let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
    let first_idx = actions
        .iter()
        .position(|a| a.id == "scriptlet_action:first")
        .unwrap();
    let second_idx = actions
        .iter()
        .position(|a| a.id == "scriptlet_action:second")
        .unwrap();
    assert!(first_idx < second_idx);
    assert_eq!(first_idx, 1); // right after run_script
    assert_eq!(second_idx, 2);
}

// ============================================================
// 22. to_deeplink_name: whitespace and mixed input
// ============================================================

#[test]
fn batch23_deeplink_tabs_and_newlines() {
    assert_eq!(to_deeplink_name("hello\tworld\ntest"), "hello-world-test");
}

#[test]
fn batch23_deeplink_multiple_spaces() {
    assert_eq!(to_deeplink_name("a   b"), "a-b");
}

#[test]
fn batch23_deeplink_leading_trailing_specials() {
    assert_eq!(to_deeplink_name("--hello--"), "hello");
}

#[test]
fn batch23_deeplink_mixed_alpha_numeric_special() {
    assert_eq!(to_deeplink_name("Script #1 (beta)"), "script-1-beta");
}

#[test]
fn batch23_deeplink_unicode_preserved() {
    let result = to_deeplink_name("日本語スクリプト");
    assert!(result.contains("日本語スクリプト"));
}

// ============================================================
// 23. format_shortcut_hint (ActionsDialog): modifier ordering
// ============================================================

#[test]
fn batch23_format_cmd_c() {
    assert_eq!(ActionsDialog::format_shortcut_hint("cmd+c"), "⌘C");
}

#[test]
fn batch23_format_ctrl_shift_delete() {
    assert_eq!(
        ActionsDialog::format_shortcut_hint("ctrl+shift+delete"),
        "⌃⇧⌫"
    );
}

#[test]
fn batch23_format_alt_enter() {
    assert_eq!(ActionsDialog::format_shortcut_hint("alt+enter"), "⌥↵");
}

#[test]
fn batch23_format_meta_is_cmd() {
    assert_eq!(ActionsDialog::format_shortcut_hint("meta+a"), "⌘A");
}

#[test]
fn batch23_format_super_is_cmd() {
    assert_eq!(ActionsDialog::format_shortcut_hint("super+k"), "⌘K");
}

// ============================================================
// 24. parse_shortcut_keycaps: multi-char and edge cases
// ============================================================

#[test]
fn batch23_parse_keycaps_cmd_enter() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘↵");
    assert_eq!(caps, vec!["⌘", "↵"]);
}

#[test]
fn batch23_parse_keycaps_single_letter() {
    let caps = ActionsDialog::parse_shortcut_keycaps("a");
    assert_eq!(caps, vec!["A"]);
}

#[test]
fn batch23_parse_keycaps_arrows() {
    let caps = ActionsDialog::parse_shortcut_keycaps("↑↓←→");
    assert_eq!(caps, vec!["↑", "↓", "←", "→"]);
}

#[test]
fn batch23_parse_keycaps_all_modifiers() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘⌃⌥⇧");
    assert_eq!(caps, vec!["⌘", "⌃", "⌥", "⇧"]);
}

#[test]
fn batch23_parse_keycaps_lowercase_uppercased() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘c");
    assert_eq!(caps, vec!["⌘", "C"]);
}

// ============================================================
// 25. score_action: fuzzy vs prefix vs contains
// ============================================================

#[test]
fn batch23_score_prefix_highest() {
    let action = Action::new("a", "Copy Path", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "copy");
    assert!(score >= 100);
}

#[test]
fn batch23_score_contains_medium() {
    let action = Action::new("a", "Full Copy Path", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "copy");
    assert!((50..100).contains(&score));
}

// --- merged from part_04.rs ---

#[test]
fn batch23_score_fuzzy_low() {
    let action = Action::new(
        "a",
        "Configure Options Pretty",
        None,
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "cop");
    // "cop" is a subsequence of "configure options pretty"
    assert!(score >= 25);
}

#[test]
fn batch23_score_no_match_zero() {
    let action = Action::new("a", "Delete", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "xyz");
    assert_eq!(score, 0);
}

#[test]
fn batch23_score_empty_search_prefix() {
    let action = Action::new("a", "Anything", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "");
    assert!(score >= 100);
}

// ============================================================
// 26. fuzzy_match: edge cases
// ============================================================

#[test]
fn batch23_fuzzy_exact_match() {
    assert!(ActionsDialog::fuzzy_match("hello", "hello"));
}

#[test]
fn batch23_fuzzy_subsequence() {
    assert!(ActionsDialog::fuzzy_match("hello world", "hwd"));
}

#[test]
fn batch23_fuzzy_no_match() {
    assert!(!ActionsDialog::fuzzy_match("abc", "xyz"));
}

#[test]
fn batch23_fuzzy_empty_needle_matches() {
    assert!(ActionsDialog::fuzzy_match("anything", ""));
}

#[test]
fn batch23_fuzzy_needle_longer_fails() {
    assert!(!ActionsDialog::fuzzy_match("ab", "abc"));
}

// ============================================================
// 27. build_grouped_items_static: headers style adds section labels
// ============================================================

#[test]
fn batch23_grouped_headers_two_sections() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // S1 header + item A + S2 header + item B = 4
    assert_eq!(grouped.len(), 4);
    assert!(matches!(&grouped[0], GroupedActionItem::SectionHeader(s) if s == "S1"));
    assert!(matches!(&grouped[2], GroupedActionItem::SectionHeader(s) if s == "S2"));
}

#[test]
fn batch23_grouped_headers_same_section_no_dup() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S1"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // S1 header + item A + item B = 3 (no second header)
    assert_eq!(grouped.len(), 3);
}

#[test]
fn batch23_grouped_separators_no_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
    // No headers with Separators, just items
    assert_eq!(grouped.len(), 2);
    assert!(matches!(&grouped[0], GroupedActionItem::Item(_)));
    assert!(matches!(&grouped[1], GroupedActionItem::Item(_)));
}

#[test]
fn batch23_grouped_empty_filtered() {
    let actions = vec![Action::new("a", "A", None, ActionCategory::ScriptContext)];
    let filtered: Vec<usize> = vec![];
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    assert!(grouped.is_empty());
}

// ============================================================
// 28. coerce_action_selection: various scenarios
// ============================================================

#[test]
fn batch23_coerce_empty_returns_none() {
    assert_eq!(coerce_action_selection(&[], 0), None);
}

#[test]
fn batch23_coerce_item_returns_same() {
    let rows = vec![GroupedActionItem::Item(0)];
    assert_eq!(coerce_action_selection(&rows, 0), Some(0));
}

#[test]
fn batch23_coerce_header_skips_to_item() {
    let rows = vec![
        GroupedActionItem::SectionHeader("H".to_string()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

#[test]
fn batch23_coerce_header_at_end_searches_up() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("H".to_string()),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(0));
}

#[test]
fn batch23_coerce_all_headers_returns_none() {
    let rows = vec![
        GroupedActionItem::SectionHeader("A".to_string()),
        GroupedActionItem::SectionHeader("B".to_string()),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

// ============================================================
// 29. Action builder: has_action defaults to false
// ============================================================

#[test]
fn batch23_action_default_has_action_false() {
    let action = Action::new("id", "title", None, ActionCategory::ScriptContext);
    assert!(!action.has_action);
}

#[test]
fn batch23_action_default_value_none() {
    let action = Action::new("id", "title", None, ActionCategory::ScriptContext);
    assert!(action.value.is_none());
}

#[test]
fn batch23_action_default_icon_none() {
    let action = Action::new("id", "title", None, ActionCategory::ScriptContext);
    assert!(action.icon.is_none());
}

#[test]
fn batch23_action_default_section_none() {
    let action = Action::new("id", "title", None, ActionCategory::ScriptContext);
    assert!(action.section.is_none());
}

#[test]
fn batch23_action_with_all_builders() {
    let action = Action::new(
        "id",
        "Title",
        Some("Desc".to_string()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘A")
    .with_icon(IconName::Star)
    .with_section("S1");
    assert_eq!(action.shortcut.as_ref().unwrap(), "⌘A");
    assert_eq!(action.icon, Some(IconName::Star));
    assert_eq!(action.section.as_ref().unwrap(), "S1");
    assert_eq!(action.title_lower, "title");
    assert_eq!(action.description_lower.as_ref().unwrap(), "desc");
    assert_eq!(action.shortcut_lower.as_ref().unwrap(), "⌘a");
}

// ============================================================
// 30. Cross-context: all contexts produce at least one action
// ============================================================

#[test]
fn batch23_cross_script_has_actions() {
    let script = ScriptInfo::new("t", "/t.ts");
    assert!(!get_script_context_actions(&script).is_empty());
}

#[test]
fn batch23_cross_builtin_has_actions() {
    let b = ScriptInfo::builtin("B");
    assert!(!get_script_context_actions(&b).is_empty());
}

#[test]
fn batch23_cross_clipboard_text_has_actions() {
    let e = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "t".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    assert!(!get_clipboard_history_context_actions(&e).is_empty());
}

#[test]
fn batch23_cross_clipboard_image_has_actions() {
    let e = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".to_string(),
        image_dimensions: Some((1, 1)),
        frontmost_app_name: None,
    };
    assert!(!get_clipboard_history_context_actions(&e).is_empty());
}

#[test]
fn batch23_cross_path_has_actions() {
    let p = PathInfo::new("t", "/t", false);
    assert!(!get_path_context_actions(&p).is_empty());
}

#[test]
fn batch23_cross_file_has_actions() {
    let f = FileInfo {
        path: "/t".to_string(),
        name: "t".to_string(),
        file_type: FileType::File,
        is_dir: false,
    };
    assert!(!get_file_context_actions(&f).is_empty());
}

#[test]
fn batch23_cross_ai_has_actions() {
    assert!(!get_ai_command_bar_actions().is_empty());
}

#[test]
fn batch23_cross_notes_has_actions() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    assert!(!get_notes_command_bar_actions(&info).is_empty());
}

#[test]
fn batch23_cross_chat_has_actions() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    assert!(!get_chat_context_actions(&info).is_empty());
}

#[test]
fn batch23_cross_note_switcher_empty_has_placeholder() {
    assert!(!get_note_switcher_actions(&[]).is_empty());
}
