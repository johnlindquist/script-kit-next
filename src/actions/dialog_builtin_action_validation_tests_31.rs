//! Batch 31: Builtin action validation tests
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
use crate::actions::types::{Action, ActionCategory, SearchPosition};
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;

// ---------------------------------------------------------------------------
// 1. Script context: shortcut and alias combined state produces correct counts
// ---------------------------------------------------------------------------
#[test]
fn batch31_script_with_both_shortcut_and_alias_has_update_remove_pairs() {
    let script = crate::actions::types::ScriptInfo::with_all(
        "test",
        "/p/test.ts",
        true,
        "Run",
        Some("⌘M".into()),
        Some("ts".into()),
    );
    let actions = get_script_context_actions(&script);
    assert!(actions.iter().any(|a| a.id == "update_shortcut"));
    assert!(actions.iter().any(|a| a.id == "remove_shortcut"));
    assert!(actions.iter().any(|a| a.id == "update_alias"));
    assert!(actions.iter().any(|a| a.id == "remove_alias"));
}

#[test]
fn batch31_script_with_both_shortcut_and_alias_no_add_actions() {
    let script = crate::actions::types::ScriptInfo::with_all(
        "test",
        "/p/test.ts",
        true,
        "Run",
        Some("⌘M".into()),
        Some("ts".into()),
    );
    let actions = get_script_context_actions(&script);
    assert!(!actions.iter().any(|a| a.id == "add_shortcut"));
    assert!(!actions.iter().any(|a| a.id == "add_alias"));
}

#[test]
fn batch31_script_neither_shortcut_nor_alias_has_add_only() {
    let script = crate::actions::types::ScriptInfo::new("test", "/p/test.ts");
    let actions = get_script_context_actions(&script);
    assert!(actions.iter().any(|a| a.id == "add_shortcut"));
    assert!(actions.iter().any(|a| a.id == "add_alias"));
    assert!(!actions.iter().any(|a| a.id == "update_shortcut"));
    assert!(!actions.iter().any(|a| a.id == "remove_shortcut"));
    assert!(!actions.iter().any(|a| a.id == "update_alias"));
    assert!(!actions.iter().any(|a| a.id == "remove_alias"));
}

#[test]
fn batch31_script_with_shortcut_and_alias_total_action_count() {
    let script = crate::actions::types::ScriptInfo::with_all(
        "test",
        "/p/test.ts",
        true,
        "Run",
        Some("⌘M".into()),
        Some("ts".into()),
    );
    let actions = get_script_context_actions(&script);
    // run + update_shortcut + remove_shortcut + update_alias + remove_alias
    //   + edit + view_logs + reveal + copy_path + copy_content + copy_deeplink = 11
    assert_eq!(actions.len(), 11);
}

// ---------------------------------------------------------------------------
// 2. Clipboard: paste_keep_open shortcut and description
// ---------------------------------------------------------------------------
#[test]
fn batch31_clipboard_paste_keep_open_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
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
fn batch31_clipboard_paste_keep_open_desc_mentions_keep() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
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
fn batch31_clipboard_paste_keep_open_title() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
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

// ---------------------------------------------------------------------------
// 3. Clipboard: save_file shortcut and description
// ---------------------------------------------------------------------------
#[test]
fn batch31_clipboard_save_file_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let sf = actions
        .iter()
        .find(|a| a.id == "clipboard_save_file")
        .unwrap();
    assert_eq!(sf.shortcut.as_deref(), Some("⌥⇧⌘S"));
}

#[test]
fn batch31_clipboard_save_file_desc_mentions_file() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let sf = actions
        .iter()
        .find(|a| a.id == "clipboard_save_file")
        .unwrap();
    assert!(sf.description.as_ref().unwrap().contains("file"));
}

#[test]
fn batch31_clipboard_save_file_title() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
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

// ---------------------------------------------------------------------------
// 4. Clipboard: save_snippet shortcut and description
// ---------------------------------------------------------------------------
#[test]
fn batch31_clipboard_save_snippet_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ss = actions
        .iter()
        .find(|a| a.id == "clipboard_save_snippet")
        .unwrap();
    assert_eq!(ss.shortcut.as_deref(), Some("⇧⌘S"));
}

#[test]
fn batch31_clipboard_save_snippet_desc_mentions_scriptlet() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ss = actions
        .iter()
        .find(|a| a.id == "clipboard_save_snippet")
        .unwrap();
    assert!(ss.description.as_ref().unwrap().contains("scriptlet"));
}

#[test]
fn batch31_clipboard_save_snippet_title() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
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

// ---------------------------------------------------------------------------
// 5. Clipboard: delete_multiple shortcut and description
// ---------------------------------------------------------------------------
#[test]
fn batch31_clipboard_delete_multiple_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let dm = actions
        .iter()
        .find(|a| a.id == "clipboard_delete_multiple")
        .unwrap();
    assert_eq!(dm.shortcut.as_deref(), Some("⇧⌘X"));
}

#[test]
fn batch31_clipboard_delete_multiple_desc_mentions_filter() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let dm = actions
        .iter()
        .find(|a| a.id == "clipboard_delete_multiple")
        .unwrap();
    assert!(dm.description.as_ref().unwrap().contains("filter"));
}

#[test]
fn batch31_clipboard_delete_all_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let da = actions
        .iter()
        .find(|a| a.id == "clipboard_delete_all")
        .unwrap();
    assert_eq!(da.shortcut.as_deref(), Some("⌃⇧X"));
}

#[test]
fn batch31_clipboard_delete_all_desc_mentions_pinned() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let da = actions
        .iter()
        .find(|a| a.id == "clipboard_delete_all")
        .unwrap();
    assert!(da.description.as_ref().unwrap().contains("pinned"));
}

// ---------------------------------------------------------------------------
// 6. File context: macOS-only actions present on macOS
// ---------------------------------------------------------------------------
#[test]
fn batch31_file_context_has_open_with_on_macos() {
    let fi = FileInfo {
        path: "/tmp/test.txt".into(),
        name: "test.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&fi);
    #[cfg(target_os = "macos")]
    assert!(actions.iter().any(|a| a.id == "open_with"));
    #[cfg(not(target_os = "macos"))]
    assert!(!actions.iter().any(|a| a.id == "open_with"));
}

#[test]
fn batch31_file_context_open_with_shortcut_cmd_o() {
    let fi = FileInfo {
        path: "/tmp/test.txt".into(),
        name: "test.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&fi);
    if let Some(ow) = actions.iter().find(|a| a.id == "open_with") {
        assert_eq!(ow.shortcut.as_deref(), Some("⌘O"));
    }
}

#[test]
fn batch31_file_context_show_info_on_macos() {
    let fi = FileInfo {
        path: "/tmp/test.txt".into(),
        name: "test.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&fi);
    #[cfg(target_os = "macos")]
    assert!(actions.iter().any(|a| a.id == "show_info"));
    #[cfg(not(target_os = "macos"))]
    assert!(!actions.iter().any(|a| a.id == "show_info"));
}

#[test]
fn batch31_file_context_show_info_shortcut_cmd_i() {
    let fi = FileInfo {
        path: "/tmp/test.txt".into(),
        name: "test.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&fi);
    if let Some(si) = actions.iter().find(|a| a.id == "show_info") {
        assert_eq!(si.shortcut.as_deref(), Some("⌘I"));
    }
}

// ---------------------------------------------------------------------------
// 7. File context: directory also gets open_with and show_info on macOS
// ---------------------------------------------------------------------------
#[test]
fn batch31_file_dir_has_open_with_on_macos() {
    let fi = FileInfo {
        path: "/tmp/mydir".into(),
        name: "mydir".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&fi);
    #[cfg(target_os = "macos")]
    assert!(actions.iter().any(|a| a.id == "open_with"));
    #[cfg(not(target_os = "macos"))]
    assert!(!actions.iter().any(|a| a.id == "open_with"));
}

#[test]
fn batch31_file_dir_has_show_info_on_macos() {
    let fi = FileInfo {
        path: "/tmp/mydir".into(),
        name: "mydir".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&fi);
    #[cfg(target_os = "macos")]
    assert!(actions.iter().any(|a| a.id == "show_info"));
    #[cfg(not(target_os = "macos"))]
    assert!(!actions.iter().any(|a| a.id == "show_info"));
}

#[test]
fn batch31_file_dir_show_info_desc_mentions_finder() {
    let fi = FileInfo {
        path: "/tmp/mydir".into(),
        name: "mydir".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&fi);
    if let Some(si) = actions.iter().find(|a| a.id == "show_info") {
        assert!(si.description.as_ref().unwrap().contains("Finder"));
    }
}

#[test]
fn batch31_file_dir_show_info_title_get_info() {
    let fi = FileInfo {
        path: "/tmp/mydir".into(),
        name: "mydir".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&fi);
    if let Some(si) = actions.iter().find(|a| a.id == "show_info") {
        assert_eq!(si.title, "Get Info");
    }
}

// ---------------------------------------------------------------------------
// 8. Path context: open_in_editor description mentions $EDITOR
// ---------------------------------------------------------------------------
#[test]
fn batch31_path_open_in_editor_desc_mentions_editor() {
    let pi = PathInfo::new("test.txt", "/tmp/test.txt", false);
    let actions = get_path_context_actions(&pi);
    let editor = actions.iter().find(|a| a.id == "open_in_editor").unwrap();
    assert!(editor.description.as_ref().unwrap().contains("$EDITOR"));
}

#[test]
fn batch31_path_open_in_terminal_desc_mentions_terminal() {
    let pi = PathInfo::new("mydir", "/tmp/mydir", true);
    let actions = get_path_context_actions(&pi);
    let term = actions.iter().find(|a| a.id == "open_in_terminal").unwrap();
    assert!(term.description.as_ref().unwrap().contains("terminal"));
}

#[test]
fn batch31_path_open_in_finder_desc_mentions_finder() {
    let pi = PathInfo::new("test.txt", "/tmp/test.txt", false);
    let actions = get_path_context_actions(&pi);
    let finder = actions.iter().find(|a| a.id == "open_in_finder").unwrap();
    assert!(finder.description.as_ref().unwrap().contains("Finder"));
}

#[test]
fn batch31_path_copy_path_desc_mentions_clipboard() {
    let pi = PathInfo::new("test.txt", "/tmp/test.txt", false);
    let actions = get_path_context_actions(&pi);
    let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
    assert!(cp.description.as_ref().unwrap().contains("clipboard"));
}

// ---------------------------------------------------------------------------
// 9. Path context: copy_filename has no shortcut vs file context copy_filename
// ---------------------------------------------------------------------------
#[test]
fn batch31_path_copy_filename_no_shortcut() {
    let pi = PathInfo::new("test.txt", "/tmp/test.txt", false);
    let actions = get_path_context_actions(&pi);
    let cf = actions.iter().find(|a| a.id == "copy_filename").unwrap();
    assert!(cf.shortcut.is_none());
}

#[test]
fn batch31_file_copy_filename_has_shortcut() {
    let fi = FileInfo {
        path: "/tmp/test.txt".into(),
        name: "test.txt".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&fi);
    let cf = actions.iter().find(|a| a.id == "copy_filename").unwrap();
    assert_eq!(cf.shortcut.as_deref(), Some("⌘C"));
}

#[test]
fn batch31_path_copy_filename_desc_mentions_filename() {
    let pi = PathInfo::new("test.txt", "/tmp/test.txt", false);
    let actions = get_path_context_actions(&pi);
    let cf = actions.iter().find(|a| a.id == "copy_filename").unwrap();
    assert!(cf.description.as_ref().unwrap().contains("filename"));
}

// ---------------------------------------------------------------------------
// 10. Script context: builtin has exactly 4 actions
// ---------------------------------------------------------------------------
#[test]
fn batch31_builtin_has_four_actions() {
    let script = crate::actions::types::ScriptInfo::builtin("Settings");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions.len(), 4);
}

#[test]
fn batch31_builtin_action_ids() {
    let script = crate::actions::types::ScriptInfo::builtin("Settings");
    let actions = get_script_context_actions(&script);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(
        ids,
        vec!["run_script", "add_shortcut", "add_alias", "copy_deeplink"]
    );
}

#[test]
fn batch31_builtin_run_script_title_uses_verb_and_name() {
    let script = crate::actions::types::ScriptInfo::builtin("Clipboard History");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].title, "Run \"Clipboard History\"");
}

#[test]
fn batch31_builtin_copy_deeplink_desc_has_url() {
    let script = crate::actions::types::ScriptInfo::builtin("Clipboard History");
    let actions = get_script_context_actions(&script);
    let cdl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert!(cdl
        .description
        .as_ref()
        .unwrap()
        .contains("scriptkit://run/clipboard-history"));
}

// ---------------------------------------------------------------------------
// 11. Script context: is_script=true action count without suggestion
// ---------------------------------------------------------------------------
#[test]
fn batch31_script_is_script_true_count() {
    let script = crate::actions::types::ScriptInfo::new("my-script", "/p/my-script.ts");
    let actions = get_script_context_actions(&script);
    // run + add_shortcut + add_alias + edit + view_logs + reveal + copy_path + copy_content + copy_deeplink = 9
    assert_eq!(actions.len(), 9);
}

#[test]
fn batch31_script_with_suggestion_has_reset_ranking() {
    let script = crate::actions::types::ScriptInfo::new("my-script", "/p/my-script.ts")
        .with_frecency(true, Some("/p/frecency".into()));
    let actions = get_script_context_actions(&script);
    assert!(actions.iter().any(|a| a.id == "reset_ranking"));
    assert_eq!(actions.len(), 10);
}

#[test]
fn batch31_script_without_suggestion_no_reset_ranking() {
    let script = crate::actions::types::ScriptInfo::new("my-script", "/p/my-script.ts");
    let actions = get_script_context_actions(&script);
    assert!(!actions.iter().any(|a| a.id == "reset_ranking"));
}

#[test]
fn batch31_script_reset_ranking_desc_mentions_suggested() {
    let script = crate::actions::types::ScriptInfo::new("my-script", "/p/my-script.ts")
        .with_frecency(true, Some("/p/frecency".into()));
    let actions = get_script_context_actions(&script);
    let rr = actions.iter().find(|a| a.id == "reset_ranking").unwrap();
    assert!(rr.description.as_ref().unwrap().contains("Suggested"));
}

// ---------------------------------------------------------------------------
// 12. Scriptlet context: action ordering invariant
// ---------------------------------------------------------------------------
#[test]
fn batch31_scriptlet_first_action_is_run_script() {
    let script = crate::actions::types::ScriptInfo::scriptlet("hello", "/p/hello.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert_eq!(actions[0].id, "run_script");
}

#[test]
fn batch31_scriptlet_last_action_is_copy_deeplink_or_reset() {
    let script = crate::actions::types::ScriptInfo::scriptlet("hello", "/p/hello.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert_eq!(actions.last().unwrap().id, "copy_deeplink");
}

#[test]
fn batch31_scriptlet_with_suggestion_last_is_reset_ranking() {
    let mut script =
        crate::actions::types::ScriptInfo::scriptlet("hello", "/p/hello.md", None, None);
    script.is_suggested = true;
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert_eq!(actions.last().unwrap().id, "reset_ranking");
}

#[test]
fn batch31_scriptlet_edit_scriptlet_desc_mentions_editor() {
    let script = crate::actions::types::ScriptInfo::scriptlet("hello", "/p/hello.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    let edit = actions.iter().find(|a| a.id == "edit_scriptlet").unwrap();
    assert!(edit.description.as_ref().unwrap().contains("$EDITOR"));
}

// ---------------------------------------------------------------------------
// 13. Chat context: model action title format with and without current
// ---------------------------------------------------------------------------
#[test]
fn batch31_chat_current_model_title_has_checkmark() {
    let info = ChatPromptInfo {
        current_model: Some("GPT-4".into()),
        available_models: vec![ChatModelInfo {
            id: "gpt4".into(),
            display_name: "GPT-4".into(),
            provider: "OpenAI".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let model = actions
        .iter()
        .find(|a| a.id == "select_model_gpt4")
        .unwrap();
    assert_eq!(model.title, "GPT-4 ✓");
}

#[test]
fn batch31_chat_non_current_model_title_no_checkmark() {
    let info = ChatPromptInfo {
        current_model: Some("Claude".into()),
        available_models: vec![ChatModelInfo {
            id: "gpt4".into(),
            display_name: "GPT-4".into(),
            provider: "OpenAI".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let model = actions
        .iter()
        .find(|a| a.id == "select_model_gpt4")
        .unwrap();
    assert_eq!(model.title, "GPT-4");
}

#[test]
fn batch31_chat_model_desc_via_provider() {
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
    let model = actions
        .iter()
        .find(|a| a.id == "select_model_claude")
        .unwrap();
    assert_eq!(model.description.as_deref(), Some("via Anthropic"));
}

#[test]
fn batch31_chat_no_current_model_no_checkmark() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "gpt4".into(),
            display_name: "GPT-4".into(),
            provider: "OpenAI".into(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let model = actions
        .iter()
        .find(|a| a.id == "select_model_gpt4")
        .unwrap();
    assert!(!model.title.contains('✓'));
}

// ---------------------------------------------------------------------------
// 14. Chat context: combined flags produce exact action counts
// ---------------------------------------------------------------------------
#[test]
fn batch31_chat_no_models_no_flags_one_action() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "continue_in_chat");
}

#[test]
fn batch31_chat_one_model_both_flags_four_actions() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "m".into(),
            display_name: "M".into(),
            provider: "P".into(),
        }],
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    // 1 model + continue_in_chat + copy_response + clear_conversation = 4
    assert_eq!(actions.len(), 4);
}

#[test]
fn batch31_chat_has_response_adds_copy_response() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "copy_response"));
}

#[test]
fn batch31_chat_no_response_no_copy_response() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "copy_response"));
}

// ---------------------------------------------------------------------------
// 15. AI command bar: section distribution
// ---------------------------------------------------------------------------
#[test]
fn batch31_ai_bar_response_section_count() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Response"))
        .count();
    assert_eq!(count, 3);
}

#[test]
fn batch31_ai_bar_actions_section_count() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Actions"))
        .count();
    assert_eq!(count, 4);
}

#[test]
fn batch31_ai_bar_attachments_section_count() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Attachments"))
        .count();
    assert_eq!(count, 2);
}

#[test]
fn batch31_ai_bar_export_section_count() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Export"))
        .count();
    assert_eq!(count, 1);
}

// ---------------------------------------------------------------------------
// 16. AI command bar: add_attachment details
// ---------------------------------------------------------------------------
#[test]
fn batch31_ai_bar_add_attachment_shortcut() {
    let actions = get_ai_command_bar_actions();
    let aa = actions.iter().find(|a| a.id == "add_attachment").unwrap();
    assert_eq!(aa.shortcut.as_deref(), Some("⇧⌘A"));
}

#[test]
fn batch31_ai_bar_add_attachment_icon() {
    let actions = get_ai_command_bar_actions();
    let aa = actions.iter().find(|a| a.id == "add_attachment").unwrap();
    assert_eq!(aa.icon, Some(IconName::Plus));
}

#[test]
fn batch31_ai_bar_add_attachment_section() {
    let actions = get_ai_command_bar_actions();
    let aa = actions.iter().find(|a| a.id == "add_attachment").unwrap();
    assert_eq!(aa.section.as_deref(), Some("Attachments"));
}

#[test]
fn batch31_ai_bar_add_attachment_desc_mentions_attach() {
    let actions = get_ai_command_bar_actions();
    let aa = actions.iter().find(|a| a.id == "add_attachment").unwrap();
    assert!(aa.description.as_ref().unwrap().contains("Attach"));
}

// ---------------------------------------------------------------------------
// 17. Notes command bar: duplicate_note conditionality
// ---------------------------------------------------------------------------
#[test]
fn batch31_notes_duplicate_note_present_with_selection_not_trash() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "duplicate_note"));
}

#[test]
fn batch31_notes_duplicate_note_absent_in_trash() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "duplicate_note"));
}

#[test]
fn batch31_notes_duplicate_note_absent_without_selection() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "duplicate_note"));
}

#[test]
fn batch31_notes_duplicate_note_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let dup = actions.iter().find(|a| a.id == "duplicate_note").unwrap();
    assert_eq!(dup.shortcut.as_deref(), Some("⌘D"));
}

// ---------------------------------------------------------------------------
// 18. Notes command bar: copy_note_as details
// ---------------------------------------------------------------------------
#[test]
fn batch31_notes_copy_note_as_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let cna = actions.iter().find(|a| a.id == "copy_note_as").unwrap();
    assert_eq!(cna.shortcut.as_deref(), Some("⇧⌘C"));
}

#[test]
fn batch31_notes_copy_note_as_icon() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let cna = actions.iter().find(|a| a.id == "copy_note_as").unwrap();
    assert_eq!(cna.icon, Some(IconName::Copy));
}

#[test]
fn batch31_notes_copy_note_as_section() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let cna = actions.iter().find(|a| a.id == "copy_note_as").unwrap();
    assert_eq!(cna.section.as_deref(), Some("Copy"));
}

#[test]
fn batch31_notes_copy_note_as_absent_without_selection() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "copy_note_as"));
}

// ---------------------------------------------------------------------------
// 19. Notes command bar: copy_deeplink details in notes context
// ---------------------------------------------------------------------------
#[test]
fn batch31_notes_copy_deeplink_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let cdl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert_eq!(cdl.shortcut.as_deref(), Some("⇧⌘D"));
}

#[test]
fn batch31_notes_copy_deeplink_icon() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let cdl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert_eq!(cdl.icon, Some(IconName::ArrowRight));
}

#[test]
fn batch31_notes_copy_deeplink_section() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let cdl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert_eq!(cdl.section.as_deref(), Some("Copy"));
}

#[test]
fn batch31_notes_copy_deeplink_absent_in_trash() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "copy_deeplink"));
}

// ---------------------------------------------------------------------------
// 20. Notes command bar: create_quicklink details
// ---------------------------------------------------------------------------
#[test]
fn batch31_notes_create_quicklink_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ql = actions.iter().find(|a| a.id == "create_quicklink").unwrap();
    assert_eq!(ql.shortcut.as_deref(), Some("⇧⌘L"));
}

#[test]
fn batch31_notes_create_quicklink_icon() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ql = actions.iter().find(|a| a.id == "create_quicklink").unwrap();
    assert_eq!(ql.icon, Some(IconName::Star));
}

#[test]
fn batch31_notes_create_quicklink_section() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let ql = actions.iter().find(|a| a.id == "create_quicklink").unwrap();
    assert_eq!(ql.section.as_deref(), Some("Copy"));
}

// ---------------------------------------------------------------------------
// 21. Notes command bar: enable_auto_sizing details
// ---------------------------------------------------------------------------
#[test]
fn batch31_notes_enable_auto_sizing_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let eas = actions
        .iter()
        .find(|a| a.id == "enable_auto_sizing")
        .unwrap();
    assert_eq!(eas.shortcut.as_deref(), Some("⌘A"));
}

#[test]
fn batch31_notes_enable_auto_sizing_icon() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let eas = actions
        .iter()
        .find(|a| a.id == "enable_auto_sizing")
        .unwrap();
    assert_eq!(eas.icon, Some(IconName::Settings));
}

#[test]
fn batch31_notes_enable_auto_sizing_section() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let eas = actions
        .iter()
        .find(|a| a.id == "enable_auto_sizing")
        .unwrap();
    assert_eq!(eas.section.as_deref(), Some("Settings"));
}

#[test]
fn batch31_notes_enable_auto_sizing_desc_mentions_window() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let eas = actions
        .iter()
        .find(|a| a.id == "enable_auto_sizing")
        .unwrap();
    assert!(eas.description.as_ref().unwrap().contains("Window"));
}

// ---------------------------------------------------------------------------
// 22. Note switcher: current note has bullet prefix, non-current does not
// ---------------------------------------------------------------------------
#[test]
fn batch31_note_switcher_current_has_bullet() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc-123".into(),
        title: "My Note".into(),
        char_count: 100,
        is_current: true,
        is_pinned: false,
        preview: "Some preview".into(),
        relative_time: "1m ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(actions[0].title.starts_with("• "));
}

#[test]
fn batch31_note_switcher_non_current_no_bullet() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc-123".into(),
        title: "My Note".into(),
        char_count: 100,
        is_current: false,
        is_pinned: false,
        preview: "Some preview".into(),
        relative_time: "1m ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(!actions[0].title.starts_with("• "));
    assert_eq!(actions[0].title, "My Note");
}

#[test]
fn batch31_note_switcher_current_icon_check() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc-123".into(),
        title: "My Note".into(),
        char_count: 100,
        is_current: true,
        is_pinned: false,
        preview: "preview".into(),
        relative_time: "1m".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::Check));
}

#[test]
fn batch31_note_switcher_regular_icon_file() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc-123".into(),
        title: "My Note".into(),
        char_count: 100,
        is_current: false,
        is_pinned: false,
        preview: "preview".into(),
        relative_time: "1m".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::File));
}

// ---------------------------------------------------------------------------
// 23. Note switcher: preview description formatting variants
// ---------------------------------------------------------------------------
#[test]
fn batch31_note_switcher_preview_with_time_has_separator() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "id1".into(),
        title: "T".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "Hello world".into(),
        relative_time: "2m ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(actions[0].description.as_ref().unwrap().contains(" · "));
}

#[test]
fn batch31_note_switcher_preview_no_time_no_separator() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "id1".into(),
        title: "T".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "Hello world".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(!desc.contains(" · "));
    assert_eq!(desc, "Hello world");
}

#[test]
fn batch31_note_switcher_no_preview_with_time_shows_time() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "id1".into(),
        title: "T".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "5m ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("5m ago"));
}

#[test]
fn batch31_note_switcher_no_preview_no_time_shows_chars() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "id1".into(),
        title: "T".into(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_deref(), Some("42 chars"));
}

// ---------------------------------------------------------------------------
// 24. New chat: model ID patterns and sections
// ---------------------------------------------------------------------------
#[test]
fn batch31_new_chat_model_id_format() {
    let models = vec![NewChatModelInfo {
        model_id: "gpt4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].id, "model_0");
}

#[test]
fn batch31_new_chat_last_used_id_format() {
    let last_used = vec![NewChatModelInfo {
        model_id: "claude".into(),
        display_name: "Claude".into(),
        provider: "anthropic".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].id, "last_used_0");
}

#[test]
fn batch31_new_chat_preset_id_format() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].id, "preset_general");
}

#[test]
fn batch31_new_chat_last_used_icon_bolt() {
    let last_used = vec![NewChatModelInfo {
        model_id: "claude".into(),
        display_name: "Claude".into(),
        provider: "anthropic".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
}

// ---------------------------------------------------------------------------
// 25. to_deeplink_name: additional edge cases
// ---------------------------------------------------------------------------
#[test]
fn batch31_deeplink_name_trailing_leading_hyphens_stripped() {
    assert_eq!(to_deeplink_name("  hello  "), "hello");
}

#[test]
fn batch31_deeplink_name_consecutive_special_chars_collapse() {
    assert_eq!(to_deeplink_name("a---b"), "a-b");
}

#[test]
fn batch31_deeplink_name_unicode_cjk_preserved() {
    let result = to_deeplink_name("日本語Script");
    assert!(result.contains("日本語"));
}

#[test]
fn batch31_deeplink_name_mixed_case_lowered() {
    assert_eq!(to_deeplink_name("Hello World"), "hello-world");
}

// ---------------------------------------------------------------------------
// 26. Action builder: default field values
// ---------------------------------------------------------------------------
#[test]
fn batch31_action_new_has_action_false() {
    let a = Action::new("test", "Test", None, ActionCategory::ScriptContext);
    assert!(!a.has_action);
}

#[test]
fn batch31_action_new_value_none() {
    let a = Action::new("test", "Test", None, ActionCategory::ScriptContext);
    assert!(a.value.is_none());
}

#[test]
fn batch31_action_new_icon_none() {
    let a = Action::new("test", "Test", None, ActionCategory::ScriptContext);
    assert!(a.icon.is_none());
}

#[test]
fn batch31_action_new_section_none() {
    let a = Action::new("test", "Test", None, ActionCategory::ScriptContext);
    assert!(a.section.is_none());
}

// ---------------------------------------------------------------------------
// 27. Action builder: with_shortcut sets both shortcut and shortcut_lower
// ---------------------------------------------------------------------------
#[test]
fn batch31_action_with_shortcut_sets_shortcut() {
    let a = Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
    assert_eq!(a.shortcut.as_deref(), Some("⌘E"));
}

#[test]
fn batch31_action_with_shortcut_sets_shortcut_lower() {
    let a = Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
    assert!(a.shortcut_lower.is_some());
}

#[test]
fn batch31_action_no_shortcut_shortcut_lower_none() {
    let a = Action::new("test", "Test", None, ActionCategory::ScriptContext);
    assert!(a.shortcut_lower.is_none());
}

#[test]
fn batch31_action_title_lower_precomputed() {
    let a = Action::new("test", "Hello World", None, ActionCategory::ScriptContext);
    assert_eq!(a.title_lower, "hello world");
}

// ---------------------------------------------------------------------------
// 28. CommandBarConfig: preset field combinations
// ---------------------------------------------------------------------------
#[test]
fn batch31_config_ai_style_search_position_top() {
    let config = CommandBarConfig::ai_style();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Top);
}

#[test]
fn batch31_config_main_menu_search_position_bottom() {
    let config = CommandBarConfig::main_menu_style();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Bottom);
}

#[test]
fn batch31_config_no_search_search_position_hidden() {
    let config = CommandBarConfig::no_search();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Hidden);
}

#[test]
fn batch31_config_notes_search_position_top() {
    let config = CommandBarConfig::notes_style();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Top);
}

// ---------------------------------------------------------------------------
// 29. CommandBarConfig: close flag defaults
// ---------------------------------------------------------------------------
#[test]
fn batch31_config_default_close_on_select_true() {
    let config = CommandBarConfig::main_menu_style();
    assert!(config.close_on_select);
}

#[test]
fn batch31_config_default_close_on_escape_true() {
    let config = CommandBarConfig::main_menu_style();
    assert!(config.close_on_escape);
}

#[test]
fn batch31_config_ai_close_on_click_outside_true() {
    let config = CommandBarConfig::ai_style();
    assert!(config.close_on_click_outside);
}

#[test]
fn batch31_config_notes_close_on_select_true() {
    let config = CommandBarConfig::notes_style();
    assert!(config.close_on_select);
}

// ---------------------------------------------------------------------------
// 30. Cross-context: last action ordering and deeplink invariants
// ---------------------------------------------------------------------------
#[test]
fn batch31_script_copy_deeplink_is_last_non_suggested() {
    let script = crate::actions::types::ScriptInfo::new("test", "/p/test.ts");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions.last().unwrap().id, "copy_deeplink");
}

#[test]
fn batch31_scriptlet_copy_deeplink_is_last_non_suggested() {
    let script = crate::actions::types::ScriptInfo::scriptlet("test", "/p/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert_eq!(actions.last().unwrap().id, "copy_deeplink");
}

#[test]
fn batch31_builtin_copy_deeplink_is_last() {
    let script = crate::actions::types::ScriptInfo::builtin("Settings");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions.last().unwrap().id, "copy_deeplink");
}

#[test]
fn batch31_clipboard_last_three_are_destructive() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
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
fn batch31_all_ai_bar_actions_have_icon() {
    let actions = get_ai_command_bar_actions();
    for action in &actions {
        assert!(
            action.icon.is_some(),
            "AI bar action {} should have an icon",
            action.id
        );
    }
}

#[test]
fn batch31_all_ai_bar_actions_have_section() {
    let actions = get_ai_command_bar_actions();
    for action in &actions {
        assert!(
            action.section.is_some(),
            "AI bar action {} should have a section",
            action.id
        );
    }
}
