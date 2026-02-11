// --- merged from part_01.rs ---
// Batch 21: Built-in action validation tests
//
// 146 tests across 30 categories validating built-in dialog actions.

use super::builders::{
    get_ai_command_bar_actions, get_chat_context_actions, get_clipboard_history_context_actions,
    get_file_context_actions, get_new_chat_actions, get_note_switcher_actions,
    get_notes_command_bar_actions, get_path_context_actions, get_script_context_actions,
    get_scriptlet_context_actions_with_custom, ChatModelInfo, ChatPromptInfo, ClipboardEntryInfo,
    NewChatModelInfo, NewChatPresetInfo, NoteSwitcherNoteInfo, NotesInfo,
};
use super::command_bar::CommandBarConfig;
use super::dialog::{build_grouped_items_static, coerce_action_selection, GroupedActionItem};
use super::types::{
    Action, ActionCategory, ActionsDialogConfig, AnchorPosition, ScriptInfo, SearchPosition,
    SectionStyle,
};
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;

// ============================================================
// 1. Script context: is_script / is_scriptlet / is_agent mutually
//    exclusive action sets
// ============================================================

#[test]
fn batch21_script_only_has_view_logs() {
    let s = ScriptInfo::new("s", "/p");
    let actions = get_script_context_actions(&s);
    assert!(actions.iter().any(|a| a.id == "view_logs"));
}

#[test]
fn batch21_scriptlet_no_view_logs() {
    let s = ScriptInfo::scriptlet("s", "/p", None, None);
    let actions = get_script_context_actions(&s);
    assert!(!actions.iter().any(|a| a.id == "view_logs"));
}

#[test]
fn batch21_agent_no_view_logs() {
    let mut s = ScriptInfo::new("a", "/p");
    s.is_agent = true;
    s.is_script = false;
    let actions = get_script_context_actions(&s);
    assert!(!actions.iter().any(|a| a.id == "view_logs"));
}

#[test]
fn batch21_builtin_no_view_logs() {
    let s = ScriptInfo::builtin("B");
    let actions = get_script_context_actions(&s);
    assert!(!actions.iter().any(|a| a.id == "view_logs"));
}

#[test]
fn batch21_script_has_edit_script_agent_has_edit_agent() {
    let script = ScriptInfo::new("s", "/p");
    let sa = get_script_context_actions(&script);
    let edit = sa.iter().find(|a| a.id == "edit_script").unwrap();
    assert_eq!(edit.title, "Edit Script");

    let mut agent = ScriptInfo::new("a", "/p");
    agent.is_agent = true;
    agent.is_script = false;
    let aa = get_script_context_actions(&agent);
    let edit2 = aa.iter().find(|a| a.id == "edit_script").unwrap();
    assert_eq!(edit2.title, "Edit Agent");
}

// ============================================================
// 2. Script context: run_script always first
// ============================================================

#[test]
fn batch21_run_script_is_first_for_script() {
    let s = ScriptInfo::new("s", "/p");
    let actions = get_script_context_actions(&s);
    assert_eq!(actions[0].id, "run_script");
}

#[test]
fn batch21_run_script_is_first_for_builtin() {
    let s = ScriptInfo::builtin("B");
    let actions = get_script_context_actions(&s);
    assert_eq!(actions[0].id, "run_script");
}

#[test]
fn batch21_run_script_is_first_for_scriptlet() {
    let s = ScriptInfo::scriptlet("s", "/p", None, None);
    let actions = get_script_context_actions(&s);
    assert_eq!(actions[0].id, "run_script");
}

#[test]
fn batch21_run_script_is_first_for_agent() {
    let mut s = ScriptInfo::new("a", "/p");
    s.is_agent = true;
    s.is_script = false;
    let actions = get_script_context_actions(&s);
    assert_eq!(actions[0].id, "run_script");
}

#[test]
fn batch21_run_script_is_first_for_scriptlet_with_custom() {
    let s = ScriptInfo::scriptlet("s", "/p", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    assert_eq!(actions[0].id, "run_script");
}

// ============================================================
// 3. File context: title format includes quoted name
// ============================================================

#[test]
fn batch21_file_open_title_contains_quoted_name() {
    let fi = FileInfo {
        path: "/tmp/readme.md".into(),
        name: "readme.md".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&fi);
    let open = actions.iter().find(|a| a.id == "open_file").unwrap();
    assert!(open.title.contains("\"readme.md\""));
}

#[test]
fn batch21_dir_open_title_contains_quoted_name() {
    let fi = FileInfo {
        path: "/tmp/docs".into(),
        name: "docs".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&fi);
    let open = actions.iter().find(|a| a.id == "open_directory").unwrap();
    assert!(open.title.contains("\"docs\""));
}

#[test]
fn batch21_file_open_title_starts_with_open() {
    let fi = FileInfo {
        path: "/a".into(),
        name: "a".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&fi);
    let open = actions.iter().find(|a| a.id == "open_file").unwrap();
    assert!(open.title.starts_with("Open"));
}

#[test]
fn batch21_dir_open_title_starts_with_open() {
    let fi = FileInfo {
        path: "/a".into(),
        name: "a".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&fi);
    let open = actions.iter().find(|a| a.id == "open_directory").unwrap();
    assert!(open.title.starts_with("Open"));
}

// ============================================================
// 4. Path context: dir primary=open_directory, file primary=select_file
// ============================================================

#[test]
fn batch21_path_dir_primary_is_open_directory() {
    let pi = PathInfo {
        path: "/d".into(),
        name: "d".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&pi);
    assert_eq!(actions[0].id, "open_directory");
}

#[test]
fn batch21_path_file_primary_is_select_file() {
    let pi = PathInfo {
        path: "/f".into(),
        name: "f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&pi);
    assert_eq!(actions[0].id, "select_file");
}

#[test]
fn batch21_path_dir_title_contains_name() {
    let pi = PathInfo {
        path: "/mydir".into(),
        name: "mydir".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&pi);
    assert!(actions[0].title.contains("\"mydir\""));
}

#[test]
fn batch21_path_file_title_contains_name() {
    let pi = PathInfo {
        path: "/f.txt".into(),
        name: "f.txt".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&pi);
    assert!(actions[0].title.contains("\"f.txt\""));
}

#[test]
fn batch21_path_dir_and_file_same_action_count() {
    let dir = PathInfo {
        path: "/d".into(),
        name: "d".into(),
        is_dir: true,
    };
    let file = PathInfo {
        path: "/f".into(),
        name: "f".into(),
        is_dir: false,
    };
    assert_eq!(
        get_path_context_actions(&dir).len(),
        get_path_context_actions(&file).len()
    );
}

// ============================================================
// 5. AI command bar: total action count and section counts
// ============================================================

#[test]
fn batch21_ai_command_bar_total_12_actions() {
    let actions = get_ai_command_bar_actions();
    assert_eq!(actions.len(), 12);
}

#[test]
fn batch21_ai_command_bar_response_section_3() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Response"))
        .count();
    assert_eq!(count, 3);
}

#[test]
fn batch21_ai_command_bar_actions_section_4() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Actions"))
        .count();
    assert_eq!(count, 4);
}

#[test]
fn batch21_ai_command_bar_attachments_section_2() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Attachments"))
        .count();
    assert_eq!(count, 2);
}

#[test]
fn batch21_ai_command_bar_export_section_1() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Export"))
        .count();
    assert_eq!(count, 1);
}

// ============================================================
// 6. AI command bar: copy_chat and copy_last_code details
// ============================================================

#[test]
fn batch21_ai_copy_chat_shortcut() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "copy_chat").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌥⇧⌘C"));
}

#[test]
fn batch21_ai_copy_chat_icon_copy() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "copy_chat").unwrap();
    assert_eq!(a.icon, Some(IconName::Copy));
}

#[test]
fn batch21_ai_copy_last_code_shortcut() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "copy_last_code").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌥⌘C"));
}

#[test]
fn batch21_ai_copy_last_code_icon_code() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "copy_last_code").unwrap();
    assert_eq!(a.icon, Some(IconName::Code));
}

#[test]
fn batch21_ai_copy_last_code_section_response() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "copy_last_code").unwrap();
    assert_eq!(a.section.as_deref(), Some("Response"));
}

// ============================================================
// 7. AI command bar: paste_image details
// ============================================================

#[test]
fn batch21_ai_paste_image_shortcut() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "paste_image").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌘V"));
}

#[test]
fn batch21_ai_paste_image_icon_file() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "paste_image").unwrap();
    assert_eq!(a.icon, Some(IconName::File));
}

#[test]
fn batch21_ai_paste_image_section_attachments() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "paste_image").unwrap();
    assert_eq!(a.section.as_deref(), Some("Attachments"));
}

#[test]
fn batch21_ai_add_attachment_shortcut() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "add_attachment").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⇧⌘A"));
}

// ============================================================
// 8. AI command bar: toggle_shortcuts_help details
// ============================================================

#[test]
fn batch21_ai_toggle_shortcuts_help_shortcut() {
    let actions = get_ai_command_bar_actions();
    let a = actions
        .iter()
        .find(|a| a.id == "toggle_shortcuts_help")
        .unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌘/"));
}

#[test]
fn batch21_ai_toggle_shortcuts_help_icon_star() {
    let actions = get_ai_command_bar_actions();
    let a = actions
        .iter()
        .find(|a| a.id == "toggle_shortcuts_help")
        .unwrap();
    assert_eq!(a.icon, Some(IconName::Star));
}

#[test]
fn batch21_ai_toggle_shortcuts_help_section_help() {
    let actions = get_ai_command_bar_actions();
    let a = actions
        .iter()
        .find(|a| a.id == "toggle_shortcuts_help")
        .unwrap();
    assert_eq!(a.section.as_deref(), Some("Help"));
}

#[test]
fn batch21_ai_change_model_no_shortcut() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "change_model").unwrap();
    assert!(a.shortcut.is_none());
}

#[test]
fn batch21_ai_branch_from_last_no_shortcut() {
    let actions = get_ai_command_bar_actions();
    let a = actions.iter().find(|a| a.id == "branch_from_last").unwrap();
    assert!(a.shortcut.is_none());
}

// ============================================================
// 9. Chat context: clear_conversation conditional
// ============================================================

#[test]
fn batch21_chat_clear_absent_no_messages() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "clear_conversation"));
}

#[test]
fn batch21_chat_clear_present_with_messages() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "clear_conversation"));
}

#[test]
fn batch21_chat_clear_shortcut() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let a = actions
        .iter()
        .find(|a| a.id == "clear_conversation")
        .unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌘⌫"));
}

// --- merged from part_02.rs ---

#[test]
fn batch21_chat_copy_response_shortcut() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    let a = actions.iter().find(|a| a.id == "copy_response").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌘C"));
}

// ============================================================
// 10. Chat context: continue_in_chat always after models
// ============================================================

#[test]
fn batch21_chat_continue_after_models() {
    let info = ChatPromptInfo {
        current_model: Some("gpt-4".into()),
        available_models: vec![
            ChatModelInfo {
                id: "gpt-4".into(),
                display_name: "GPT-4".into(),
                provider: "OpenAI".into(),
            },
            ChatModelInfo {
                id: "claude".into(),
                display_name: "Claude".into(),
                provider: "Anthropic".into(),
            },
        ],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let model_last_pos = actions
        .iter()
        .rposition(|a| a.id.starts_with("select_model_"))
        .unwrap();
    let continue_pos = actions
        .iter()
        .position(|a| a.id == "continue_in_chat")
        .unwrap();
    assert!(continue_pos > model_last_pos);
}

#[test]
fn batch21_chat_continue_present_zero_models() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "continue_in_chat"));
}

#[test]
fn batch21_chat_continue_shortcut() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let a = actions.iter().find(|a| a.id == "continue_in_chat").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌘↵"));
}

// ============================================================
// 11. Notes command bar: copy section actions
// ============================================================

#[test]
fn batch21_notes_copy_deeplink_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let a = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⇧⌘D"));
}

#[test]
fn batch21_notes_copy_deeplink_section_copy() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let a = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert_eq!(a.section.as_deref(), Some("Copy"));
}

#[test]
fn batch21_notes_create_quicklink_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let a = actions.iter().find(|a| a.id == "create_quicklink").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⇧⌘L"));
}

#[test]
fn batch21_notes_create_quicklink_icon_star() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let a = actions.iter().find(|a| a.id == "create_quicklink").unwrap();
    assert_eq!(a.icon, Some(IconName::Star));
}

#[test]
fn batch21_notes_copy_note_as_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let a = actions.iter().find(|a| a.id == "copy_note_as").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⇧⌘C"));
}

// ============================================================
// 12. Notes command bar: enable_auto_sizing conditional
// ============================================================

#[test]
fn batch21_notes_auto_sizing_absent_when_enabled() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "enable_auto_sizing"));
}

#[test]
fn batch21_notes_auto_sizing_present_when_disabled() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "enable_auto_sizing"));
}

#[test]
fn batch21_notes_auto_sizing_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let a = actions
        .iter()
        .find(|a| a.id == "enable_auto_sizing")
        .unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌘A"));
}

#[test]
fn batch21_notes_auto_sizing_section_settings() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let a = actions
        .iter()
        .find(|a| a.id == "enable_auto_sizing")
        .unwrap();
    assert_eq!(a.section.as_deref(), Some("Settings"));
}

// ============================================================
// 13. Note switcher: relative_time propagation
// ============================================================

#[test]
fn batch21_note_switcher_preview_and_time_joined() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 50,
        is_current: false,
        is_pinned: false,
        preview: "Hello world".into(),
        relative_time: "2m ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_deref().unwrap();
    assert!(desc.contains("Hello world"));
    assert!(desc.contains("2m ago"));
    assert!(desc.contains(" · "));
}

#[test]
fn batch21_note_switcher_no_time_no_separator() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "Some text".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_deref().unwrap();
    assert_eq!(desc, "Some text");
    assert!(!desc.contains(" · "));
}

#[test]
fn batch21_note_switcher_no_preview_with_time() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 5,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "1h ago".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_deref().unwrap();
    assert_eq!(desc, "1h ago");
}

#[test]
fn batch21_note_switcher_no_preview_no_time_char_count() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_deref().unwrap();
    assert_eq!(desc, "42 chars");
}

#[test]
fn batch21_note_switcher_singular_char() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "1".into(),
        title: "T".into(),
        char_count: 1,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_deref().unwrap();
    assert_eq!(desc, "1 char");
}

// ============================================================
// 14. New chat actions: ID format patterns
// ============================================================

#[test]
fn batch21_new_chat_last_used_id_format() {
    let last_used = vec![NewChatModelInfo {
        model_id: "gpt-4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].id, "last_used_0");
}

#[test]
fn batch21_new_chat_preset_id_format() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].id, "preset_general");
}

#[test]
fn batch21_new_chat_model_id_format() {
    let models = vec![NewChatModelInfo {
        model_id: "claude-3".into(),
        display_name: "Claude 3".into(),
        provider: "anthropic".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].id, "model_0");
}

#[test]
fn batch21_new_chat_multiple_last_used_sequential_ids() {
    let last_used = vec![
        NewChatModelInfo {
            model_id: "a".into(),
            display_name: "A".into(),
            provider: "p".into(),
            provider_display_name: "P".into(),
        },
        NewChatModelInfo {
            model_id: "b".into(),
            display_name: "B".into(),
            provider: "p".into(),
            provider_display_name: "P".into(),
        },
    ];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].id, "last_used_0");
    assert_eq!(actions[1].id, "last_used_1");
}

#[test]
fn batch21_new_chat_empty_all_empty_result() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert!(actions.is_empty());
}

// ============================================================
// 15. Clipboard context: clipboard_copy description
// ============================================================

#[test]
fn batch21_clipboard_copy_description_mentions_clipboard() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let a = actions.iter().find(|a| a.id == "clipboard_copy").unwrap();
    assert!(a
        .description
        .as_deref()
        .unwrap()
        .to_lowercase()
        .contains("clipboard"));
}

#[test]
fn batch21_clipboard_paste_description_mentions_clipboard() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let a = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
    assert!(a
        .description
        .as_deref()
        .unwrap()
        .to_lowercase()
        .contains("clipboard"));
}

#[test]
fn batch21_clipboard_paste_keep_open_desc_mentions_keep() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let a = actions
        .iter()
        .find(|a| a.id == "clipboard_paste_keep_open")
        .unwrap();
    assert!(a
        .description
        .as_deref()
        .unwrap()
        .to_lowercase()
        .contains("keep"));
}

#[test]
fn batch21_clipboard_delete_all_desc_mentions_pinned() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let a = actions
        .iter()
        .find(|a| a.id == "clipboard_delete_all")
        .unwrap();
    assert!(a
        .description
        .as_deref()
        .unwrap()
        .to_lowercase()
        .contains("pinned"));
}

// --- merged from part_03.rs ---

// ============================================================
// 16. Clipboard context: frontmost_app_name edge cases
// ============================================================

#[test]
fn batch21_clipboard_paste_empty_string_app() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: Some("".into()),
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let a = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
    // Empty string still triggers Some branch: "Paste to "
    assert_eq!(a.title, "Paste to ");
}

#[test]
fn batch21_clipboard_paste_unicode_app() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: Some("Xcode \u{2013} Beta".into()),
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let a = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
    assert_eq!(a.title, "Paste to Xcode \u{2013} Beta");
}

// ============================================================
// 17. CommandBarConfig preset field matrix
// ============================================================

#[test]
fn batch21_config_default_search_bottom() {
    let c = CommandBarConfig::default();
    assert!(matches!(
        c.dialog_config.search_position,
        SearchPosition::Bottom
    ));
}

#[test]
fn batch21_config_ai_style_anchor_top() {
    let c = CommandBarConfig::ai_style();
    assert!(matches!(c.dialog_config.anchor, AnchorPosition::Top));
}

#[test]
fn batch21_config_main_menu_anchor_bottom() {
    let c = CommandBarConfig::main_menu_style();
    assert!(matches!(c.dialog_config.anchor, AnchorPosition::Bottom));
}

#[test]
fn batch21_config_notes_style_icons_true() {
    let c = CommandBarConfig::notes_style();
    assert!(c.dialog_config.show_icons);
}

#[test]
fn batch21_config_notes_style_footer_true() {
    let c = CommandBarConfig::notes_style();
    assert!(c.dialog_config.show_footer);
}

// ============================================================
// 18. ActionsDialogConfig default values
// ============================================================

#[test]
fn batch21_dialog_config_default_search_bottom() {
    let c = ActionsDialogConfig::default();
    assert!(matches!(c.search_position, SearchPosition::Bottom));
}

#[test]
fn batch21_dialog_config_default_section_separators() {
    let c = ActionsDialogConfig::default();
    assert!(matches!(c.section_style, SectionStyle::Separators));
}

#[test]
fn batch21_dialog_config_default_anchor_bottom() {
    let c = ActionsDialogConfig::default();
    assert!(matches!(c.anchor, AnchorPosition::Bottom));
}

#[test]
fn batch21_dialog_config_default_no_icons() {
    let c = ActionsDialogConfig::default();
    assert!(!c.show_icons);
}

#[test]
fn batch21_dialog_config_default_no_footer() {
    let c = ActionsDialogConfig::default();
    assert!(!c.show_footer);
}

// ============================================================
// 19. Action with_shortcut caching behavior
// ============================================================

#[test]
fn batch21_action_with_shortcut_sets_shortcut_lower() {
    let a = Action::new("id", "T", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
    assert_eq!(a.shortcut_lower, Some("⌘e".into()));
}

#[test]
fn batch21_action_no_shortcut_lower_is_none() {
    let a = Action::new("id", "T", None, ActionCategory::ScriptContext);
    assert!(a.shortcut_lower.is_none());
}

#[test]
fn batch21_action_title_lower_precomputed() {
    let a = Action::new("id", "Edit Script", None, ActionCategory::ScriptContext);
    assert_eq!(a.title_lower, "edit script");
}

#[test]
fn batch21_action_description_lower_precomputed() {
    let a = Action::new(
        "id",
        "T",
        Some("Open in $EDITOR".into()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(a.description_lower, Some("open in $editor".into()));
}

#[test]
fn batch21_action_description_none_lower_none() {
    let a = Action::new("id", "T", None, ActionCategory::ScriptContext);
    assert!(a.description_lower.is_none());
}

// ============================================================
// 20. Action builder chaining: with_icon, with_section
// ============================================================

#[test]
fn batch21_action_with_icon_preserves_shortcut() {
    let a = Action::new("id", "T", None, ActionCategory::ScriptContext)
        .with_shortcut("⌘X")
        .with_icon(IconName::Copy);
    assert_eq!(a.shortcut.as_deref(), Some("⌘X"));
    assert_eq!(a.icon, Some(IconName::Copy));
}

#[test]
fn batch21_action_with_section_preserves_icon() {
    let a = Action::new("id", "T", None, ActionCategory::ScriptContext)
        .with_icon(IconName::Star)
        .with_section("MySection");
    assert_eq!(a.icon, Some(IconName::Star));
    assert_eq!(a.section.as_deref(), Some("MySection"));
}

#[test]
fn batch21_action_full_chain_all_fields() {
    let a = Action::new(
        "test_id",
        "Test Title",
        Some("Test Desc".into()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘T")
    .with_icon(IconName::Plus)
    .with_section("TestSection");

    assert_eq!(a.id, "test_id");
    assert_eq!(a.title, "Test Title");
    assert_eq!(a.description, Some("Test Desc".into()));
    assert_eq!(a.shortcut, Some("⌘T".into()));
    assert_eq!(a.icon, Some(IconName::Plus));
    assert_eq!(a.section, Some("TestSection".into()));
}

#[test]
fn batch21_action_with_shortcut_opt_none_preserves() {
    let a = Action::new("id", "T", None, ActionCategory::ScriptContext)
        .with_shortcut("⌘X")
        .with_shortcut_opt(None);
    assert_eq!(a.shortcut.as_deref(), Some("⌘X"));
}

#[test]
fn batch21_action_with_shortcut_opt_some_sets() {
    let a = Action::new("id", "T", None, ActionCategory::ScriptContext)
        .with_shortcut_opt(Some("⌘Y".into()));
    assert_eq!(a.shortcut.as_deref(), Some("⌘Y"));
}

// ============================================================
// 21. build_grouped_items_static: section transitions
// ============================================================

#[test]
fn batch21_grouped_items_headers_two_sections() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // S1 header, item0, S2 header, item1
    assert_eq!(grouped.len(), 4);
    assert!(matches!(grouped[0], GroupedActionItem::SectionHeader(_)));
    assert!(matches!(grouped[1], GroupedActionItem::Item(0)));
    assert!(matches!(grouped[2], GroupedActionItem::SectionHeader(_)));
    assert!(matches!(grouped[3], GroupedActionItem::Item(1)));
}

#[test]
fn batch21_grouped_items_headers_same_section_no_dup() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // S header, item0, item1
    assert_eq!(grouped.len(), 3);
    assert!(matches!(grouped[0], GroupedActionItem::SectionHeader(_)));
}

#[test]
fn batch21_grouped_items_separators_no_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
    // Just items, no headers
    assert_eq!(grouped.len(), 2);
    assert!(matches!(grouped[0], GroupedActionItem::Item(0)));
    assert!(matches!(grouped[1], GroupedActionItem::Item(1)));
}

#[test]
fn batch21_grouped_items_empty_filtered() {
    let actions = vec![Action::new("a", "A", None, ActionCategory::ScriptContext)];
    let filtered: Vec<usize> = vec![];
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    assert!(grouped.is_empty());
}

// ============================================================
// 22. coerce_action_selection: header skipping
// ============================================================

#[test]
fn batch21_coerce_on_item_stays() {
    let rows = vec![
        GroupedActionItem::SectionHeader("H".into()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(1));
}

#[test]
fn batch21_coerce_on_header_jumps_down() {
    let rows = vec![
        GroupedActionItem::SectionHeader("H".into()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

#[test]
fn batch21_coerce_trailing_header_jumps_up() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("H".into()),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(0));
}

#[test]
fn batch21_coerce_all_headers_none() {
    let rows = vec![
        GroupedActionItem::SectionHeader("A".into()),
        GroupedActionItem::SectionHeader("B".into()),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

#[test]
fn batch21_coerce_empty_none() {
    let rows: Vec<GroupedActionItem> = vec![];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

// ============================================================
// 23. ScriptInfo constructor defaults
// ============================================================

#[test]
fn batch21_scriptinfo_new_defaults() {
    let s = ScriptInfo::new("n", "/p");
    assert!(s.is_script);
    assert!(!s.is_scriptlet);
    assert!(!s.is_agent);
    assert_eq!(s.action_verb, "Run");
    assert!(!s.is_suggested);
    assert!(s.frecency_path.is_none());
}

#[test]
fn batch21_scriptinfo_builtin_path_empty() {
    let s = ScriptInfo::builtin("B");
    assert!(s.path.is_empty());
    assert!(!s.is_script);
}

#[test]
fn batch21_scriptinfo_scriptlet_flags() {
    let s = ScriptInfo::scriptlet("S", "/p", None, None);
    assert!(!s.is_script);
    assert!(s.is_scriptlet);
    assert!(!s.is_agent);
}

#[test]
fn batch21_scriptinfo_with_frecency_chaining() {
    let s = ScriptInfo::new("n", "/p").with_frecency(true, Some("fp".into()));
    assert!(s.is_suggested);
    assert_eq!(s.frecency_path, Some("fp".into()));
    // Original fields preserved
    assert!(s.is_script);
}

// ============================================================
// 24. Script context: copy_content shortcut consistent
// ============================================================

#[test]
fn batch21_script_copy_content_shortcut() {
    let s = ScriptInfo::new("s", "/p");
    let actions = get_script_context_actions(&s);
    let a = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌘⌥C"));
}

#[test]
fn batch21_scriptlet_copy_content_shortcut() {
    let s = ScriptInfo::scriptlet("s", "/p", None, None);
    let actions = get_script_context_actions(&s);
    let a = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌘⌥C"));
}

#[test]
fn batch21_agent_copy_content_shortcut() {
    let mut s = ScriptInfo::new("a", "/p");
    s.is_agent = true;
    s.is_script = false;
    let actions = get_script_context_actions(&s);
    let a = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌘⌥C"));
}

#[test]
fn batch21_scriptlet_with_custom_copy_content_shortcut() {
    let s = ScriptInfo::scriptlet("s", "/p", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&s, None);
    let a = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert_eq!(a.shortcut.as_deref(), Some("⌘⌥C"));
}

// ============================================================
// 25. File vs path context: primary action IDs differ
// ============================================================

#[test]
fn batch21_file_file_primary_is_open_file() {
    let fi = FileInfo {
        path: "/f".into(),
        name: "f".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&fi);
    assert_eq!(actions[0].id, "open_file");
}

#[test]
fn batch21_path_file_primary_is_select_file_in_file_vs_path() {
    let pi = PathInfo {
        path: "/f".into(),
        name: "f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&pi);
    assert_eq!(actions[0].id, "select_file");
}

#[test]
fn batch21_file_dir_and_path_dir_same_primary_id() {
    let fi = FileInfo {
        path: "/d".into(),
        name: "d".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let pi = PathInfo {
        path: "/d".into(),
        name: "d".into(),
        is_dir: true,
    };
    assert_eq!(
        get_file_context_actions(&fi)[0].id,
        get_path_context_actions(&pi)[0].id
    );
}

// ============================================================
// 26. Path context: move_to_trash always last
// ============================================================

#[test]
fn batch21_path_trash_last_for_dir() {
    let pi = PathInfo {
        path: "/d".into(),
        name: "d".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&pi);
    assert_eq!(actions.last().unwrap().id, "move_to_trash");
}

// --- merged from part_04.rs ---

#[test]
fn batch21_path_trash_last_for_file() {
    let pi = PathInfo {
        path: "/f".into(),
        name: "f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&pi);
    assert_eq!(actions.last().unwrap().id, "move_to_trash");
}

#[test]
fn batch21_path_trash_description_dir() {
    let pi = PathInfo {
        path: "/d".into(),
        name: "d".into(),
        is_dir: true,
    };
    let actions = get_path_context_actions(&pi);
    let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
    assert!(trash
        .description
        .as_deref()
        .unwrap()
        .to_lowercase()
        .contains("folder"));
}

#[test]
fn batch21_path_trash_description_file() {
    let pi = PathInfo {
        path: "/f".into(),
        name: "f".into(),
        is_dir: false,
    };
    let actions = get_path_context_actions(&pi);
    let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
    assert!(trash
        .description
        .as_deref()
        .unwrap()
        .to_lowercase()
        .contains("file"));
}

// ============================================================
// 27. Cross-context: all built-in IDs are snake_case
// ============================================================

fn assert_snake_case_ids(actions: &[Action], context: &str) {
    for a in actions {
        // Scriptlet-defined actions have "scriptlet_action:" prefix and are allowed colons
        if a.id.starts_with("scriptlet_action:") {
            continue;
        }
        assert!(
            !a.id.contains(' '),
            "{} action '{}' has spaces (not snake_case)",
            context,
            a.id
        );
        assert!(
            !a.id.contains('-'),
            "{} action '{}' has hyphens (not snake_case)",
            context,
            a.id
        );
    }
}

#[test]
fn batch21_snake_case_ids_script() {
    let s = ScriptInfo::new("s", "/p");
    assert_snake_case_ids(&get_script_context_actions(&s), "script");
}

#[test]
fn batch21_snake_case_ids_builtin() {
    let s = ScriptInfo::builtin("B");
    assert_snake_case_ids(&get_script_context_actions(&s), "builtin");
}

#[test]
fn batch21_snake_case_ids_scriptlet() {
    let s = ScriptInfo::scriptlet("S", "/p", None, None);
    assert_snake_case_ids(&get_script_context_actions(&s), "scriptlet");
}

#[test]
fn batch21_snake_case_ids_clipboard() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    assert_snake_case_ids(&get_clipboard_history_context_actions(&entry), "clipboard");
}

#[test]
fn batch21_snake_case_ids_ai() {
    assert_snake_case_ids(&get_ai_command_bar_actions(), "ai");
}

#[test]
fn batch21_snake_case_ids_notes() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    assert_snake_case_ids(&get_notes_command_bar_actions(&info), "notes");
}

// ============================================================
// 28. Cross-context: all actions have non-empty IDs and titles
// ============================================================

fn assert_nonempty_id_title(actions: &[Action], context: &str) {
    for a in actions {
        assert!(
            !a.id.is_empty(),
            "{}: action has empty ID (title={})",
            context,
            a.title
        );
        assert!(
            !a.title.is_empty(),
            "{}: action has empty title (id={})",
            context,
            a.id
        );
    }
}

#[test]
fn batch21_nonempty_id_title_script() {
    let s = ScriptInfo::new("s", "/p");
    assert_nonempty_id_title(&get_script_context_actions(&s), "script");
}

#[test]
fn batch21_nonempty_id_title_clipboard() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: true,
        preview: "".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    assert_nonempty_id_title(
        &get_clipboard_history_context_actions(&entry),
        "clipboard_image",
    );
}

#[test]
fn batch21_nonempty_id_title_path() {
    let pi = PathInfo {
        path: "/d".into(),
        name: "d".into(),
        is_dir: true,
    };
    assert_nonempty_id_title(&get_path_context_actions(&pi), "path");
}

#[test]
fn batch21_nonempty_id_title_file() {
    let fi = FileInfo {
        path: "/f".into(),
        name: "f".into(),
        file_type: FileType::File,
        is_dir: false,
    };
    assert_nonempty_id_title(&get_file_context_actions(&fi), "file");
}

// ============================================================
// 29. Script context: deeplink description URL format
// ============================================================

#[test]
fn batch21_deeplink_url_format_script() {
    let s = ScriptInfo::new("My Script", "/p");
    let actions = get_script_context_actions(&s);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    let desc = dl.description.as_deref().unwrap();
    assert!(desc.contains("scriptkit://run/my-script"));
}

#[test]
fn batch21_deeplink_url_format_builtin() {
    let s = ScriptInfo::builtin("Clipboard History");
    let actions = get_script_context_actions(&s);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    let desc = dl.description.as_deref().unwrap();
    assert!(desc.contains("scriptkit://run/clipboard-history"));
}

#[test]
fn batch21_deeplink_url_format_scriptlet() {
    let s = ScriptInfo::scriptlet("Open GitHub", "/p", None, None);
    let actions = get_script_context_actions(&s);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    let desc = dl.description.as_deref().unwrap();
    assert!(desc.contains("scriptkit://run/open-github"));
}

// ============================================================
// 30. Cross-context: ID uniqueness within each context
// ============================================================

fn assert_unique_ids(actions: &[Action], context: &str) {
    let mut seen = std::collections::HashSet::new();
    for a in actions {
        assert!(
            seen.insert(&a.id),
            "{}: duplicate action ID '{}'",
            context,
            a.id
        );
    }
}

#[test]
fn batch21_unique_ids_script() {
    let s = ScriptInfo::new("s", "/p");
    assert_unique_ids(&get_script_context_actions(&s), "script");
}

#[test]
fn batch21_unique_ids_script_with_shortcut_and_alias() {
    let s = ScriptInfo::with_shortcut_and_alias("s", "/p", Some("⌘T".into()), Some("t".into()));
    assert_unique_ids(&get_script_context_actions(&s), "script_full");
}

#[test]
fn batch21_unique_ids_clipboard_text() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "x".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    assert_unique_ids(
        &get_clipboard_history_context_actions(&entry),
        "clipboard_text",
    );
}

#[test]
fn batch21_unique_ids_clipboard_image() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: true,
        preview: "".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    assert_unique_ids(
        &get_clipboard_history_context_actions(&entry),
        "clipboard_image",
    );
}

#[test]
fn batch21_unique_ids_ai() {
    assert_unique_ids(&get_ai_command_bar_actions(), "ai");
}

#[test]
fn batch21_unique_ids_notes() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    assert_unique_ids(&get_notes_command_bar_actions(&info), "notes");
}

#[test]
fn batch21_unique_ids_path_dir() {
    let pi = PathInfo {
        path: "/d".into(),
        name: "d".into(),
        is_dir: true,
    };
    assert_unique_ids(&get_path_context_actions(&pi), "path_dir");
}

#[test]
fn batch21_unique_ids_path_file() {
    let pi = PathInfo {
        path: "/f".into(),
        name: "f".into(),
        is_dir: false,
    };
    assert_unique_ids(&get_path_context_actions(&pi), "path_file");
}
