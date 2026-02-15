// --- merged from part_01.rs ---
//! Batch 32: Builtin action validation tests
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
use crate::actions::dialog::{
    build_grouped_items_static, coerce_action_selection, GroupedActionItem,
};
use crate::actions::types::{Action, ActionCategory, SectionStyle};
use crate::actions::ActionsDialog;
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;

// ---------------------------------------------------------------------------
// 1. Script context: agent has no view_logs but has copy_content desc about file
// ---------------------------------------------------------------------------

#[test]
fn batch32_agent_copy_content_desc_mentions_entire_file() {
    let mut script = crate::actions::types::ScriptInfo::new("my-agent", "/p/my-agent.md");
    script.is_agent = true;
    script.is_script = false;
    let actions = get_script_context_actions(&script);
    let cc = actions.iter().find(|a| a.id == "copy_content").unwrap();
    assert!(
        cc.description.as_ref().unwrap().contains("entire file"),
        "agent copy_content desc should mention 'entire file', got: {:?}",
        cc.description
    );
}

#[test]
fn batch32_agent_edit_script_desc_mentions_agent_file() {
    let mut script = crate::actions::types::ScriptInfo::new("my-agent", "/p/my-agent.md");
    script.is_agent = true;
    script.is_script = false;
    let actions = get_script_context_actions(&script);
    let es = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert!(
        es.description.as_ref().unwrap().contains("agent"),
        "agent edit desc should mention 'agent', got: {:?}",
        es.description
    );
}

#[test]
fn batch32_agent_reveal_desc_mentions_agent_file() {
    let mut script = crate::actions::types::ScriptInfo::new("my-agent", "/p/my-agent.md");
    script.is_agent = true;
    script.is_script = false;
    let actions = get_script_context_actions(&script);
    let r = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
    assert!(
        r.description.as_ref().unwrap().contains("agent"),
        "agent reveal desc should mention 'agent', got: {:?}",
        r.description
    );
}

#[test]
fn batch32_agent_copy_path_desc_mentions_agent() {
    let mut script = crate::actions::types::ScriptInfo::new("my-agent", "/p/my-agent.md");
    script.is_agent = true;
    script.is_script = false;
    let actions = get_script_context_actions(&script);
    let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
    assert!(
        cp.description.as_ref().unwrap().contains("agent"),
        "agent copy_path desc should mention 'agent', got: {:?}",
        cp.description
    );
}

// ---------------------------------------------------------------------------
// 2. Scriptlet context with_custom: None scriptlet produces only built-in actions
// ---------------------------------------------------------------------------

#[test]
fn batch32_scriptlet_context_none_scriptlet_no_custom_actions() {
    let script = crate::actions::types::ScriptInfo::scriptlet("Test", "/p/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    // All actions should have has_action=false (built-in)
    for a in &actions {
        assert!(
            !a.has_action,
            "built-in action {} should have has_action=false",
            a.id
        );
    }
}

#[test]
fn batch32_scriptlet_context_none_scriptlet_has_edit_scriptlet() {
    let script = crate::actions::types::ScriptInfo::scriptlet("Test", "/p/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(actions.iter().any(|a| a.id == "edit_scriptlet"));
}

#[test]
fn batch32_scriptlet_context_none_scriptlet_has_reveal_scriptlet() {
    let script = crate::actions::types::ScriptInfo::scriptlet("Test", "/p/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(actions.iter().any(|a| a.id == "reveal_scriptlet_in_finder"));
}

#[test]
fn batch32_scriptlet_context_none_scriptlet_has_copy_scriptlet_path() {
    let script = crate::actions::types::ScriptInfo::scriptlet("Test", "/p/test.md", None, None);
    let actions = get_scriptlet_context_actions_with_custom(&script, None);
    assert!(actions.iter().any(|a| a.id == "copy_scriptlet_path"));
}

// ---------------------------------------------------------------------------
// 3. Clipboard: frontmost_app_name empty string edge case
// ---------------------------------------------------------------------------

#[test]
fn batch32_clipboard_empty_app_name_paste_title() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: Some("".to_string()),
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = actions.iter().find(|a| a.id == "clip:clipboard_paste").unwrap();
    // Empty string still goes through format!("Paste to {}", name) path
    assert_eq!(paste.title, "Paste to ");
}

#[test]
fn batch32_clipboard_long_app_name_paste_title() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: Some("Very Long Application Name Here".to_string()),
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = actions.iter().find(|a| a.id == "clip:clipboard_paste").unwrap();
    assert_eq!(paste.title, "Paste to Very Long Application Name Here");
}

#[test]
fn batch32_clipboard_none_app_name_paste_active_app() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let paste = actions.iter().find(|a| a.id == "clip:clipboard_paste").unwrap();
    assert_eq!(paste.title, "Paste to Active App");
}

// ---------------------------------------------------------------------------
// 4. Clipboard: text entry has no image-specific actions
// ---------------------------------------------------------------------------

#[test]
fn batch32_clipboard_text_no_clipboard_open_with() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clip:clipboard_open_with"));
}

#[test]
fn batch32_clipboard_text_no_clipboard_annotate_cleanshot() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions
        .iter()
        .any(|a| a.id == "clip:clipboard_annotate_cleanshot"));
}

#[test]
fn batch32_clipboard_text_no_clipboard_upload_cleanshot() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clip:clipboard_upload_cleanshot"));
}

#[test]
fn batch32_clipboard_text_no_clipboard_ocr() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clip:clipboard_ocr"));
}

// ---------------------------------------------------------------------------
// 5. File context: reveal_in_finder always present for both file and dir
// ---------------------------------------------------------------------------

#[test]
fn batch32_file_reveal_in_finder_present_for_file() {
    let info = FileInfo {
        path: "/p/test.txt".into(),
        name: "test.txt".into(),
        file_type: FileType::Document,
        is_dir: false,
    };
    let actions = get_file_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "file:reveal_in_finder"));
}

#[test]
fn batch32_file_reveal_in_finder_present_for_dir() {
    let info = FileInfo {
        path: "/p/mydir".into(),
        name: "mydir".into(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "file:reveal_in_finder"));
}

#[test]
fn batch32_file_reveal_in_finder_shortcut_is_cmd_enter() {
    let info = FileInfo {
        path: "/p/test.txt".into(),
        name: "test.txt".into(),
        file_type: FileType::Document,
        is_dir: false,
    };
    let actions = get_file_context_actions(&info);
    let reveal = actions.iter().find(|a| a.id == "file:reveal_in_finder").unwrap();
    assert_eq!(reveal.shortcut.as_deref(), Some("⌘↵"));
}

#[test]
fn batch32_file_reveal_desc_says_reveal_in_finder() {
    let info = FileInfo {
        path: "/p/test.txt".into(),
        name: "test.txt".into(),
        file_type: FileType::Document,
        is_dir: false,
    };
    let actions = get_file_context_actions(&info);
    let reveal = actions.iter().find(|a| a.id == "file:reveal_in_finder").unwrap();
    assert!(reveal.description.as_ref().unwrap().contains("Finder"));
}

// ---------------------------------------------------------------------------
// 6. Path context: action ordering after primary action
// ---------------------------------------------------------------------------

#[test]
fn batch32_path_file_second_action_is_copy_path() {
    let info = PathInfo::new("test.txt", "/p/test.txt", false);
    let actions = get_path_context_actions(&info);
    assert_eq!(actions[1].id, "file:copy_path");
}

#[test]
fn batch32_path_dir_second_action_is_copy_path() {
    let info = PathInfo::new("mydir", "/p/mydir", true);
    let actions = get_path_context_actions(&info);
    assert_eq!(actions[1].id, "file:copy_path");
}

#[test]
fn batch32_path_file_third_action_is_open_in_finder() {
    let info = PathInfo::new("test.txt", "/p/test.txt", false);
    let actions = get_path_context_actions(&info);
    assert_eq!(actions[2].id, "file:open_in_finder");
}

#[test]
fn batch32_path_last_action_is_move_to_trash() {
    let info = PathInfo::new("test.txt", "/p/test.txt", false);
    let actions = get_path_context_actions(&info);
    assert_eq!(actions.last().unwrap().id, "file:move_to_trash");
}

// ---------------------------------------------------------------------------
// 7. Path context: name in primary action title is quoted
// ---------------------------------------------------------------------------

#[test]
fn batch32_path_file_primary_title_quotes_name() {
    let info = PathInfo::new("report.pdf", "/p/report.pdf", false);
    let actions = get_path_context_actions(&info);
    assert_eq!(actions[0].title, "Select \"report.pdf\"");
}

#[test]
fn batch32_path_dir_primary_title_quotes_name() {
    let info = PathInfo::new("Documents", "/p/Documents", true);
    let actions = get_path_context_actions(&info);
    assert_eq!(actions[0].title, "Open \"Documents\"");
}

#[test]
fn batch32_path_file_select_desc_submit() {
    let info = PathInfo::new("file.txt", "/p/file.txt", false);
    let actions = get_path_context_actions(&info);
    assert!(actions[0]
        .description
        .as_ref()
        .unwrap()
        .contains("Selects this file"));
}

#[test]
fn batch32_path_dir_open_desc_navigate() {
    let info = PathInfo::new("dir", "/p/dir", true);
    let actions = get_path_context_actions(&info);
    assert!(actions[0]
        .description
        .as_ref()
        .unwrap()
        .contains("Opens this directory"));
}

// ---------------------------------------------------------------------------
// 8. AI command bar: export_markdown details
// ---------------------------------------------------------------------------

#[test]
fn batch32_ai_export_markdown_section_is_export() {
    let actions = get_ai_command_bar_actions();
    let em = actions.iter().find(|a| a.id == "chat:export_markdown").unwrap();
    assert_eq!(em.section.as_deref(), Some("Export"));
}

#[test]
fn batch32_ai_export_markdown_icon_is_file_code() {
    let actions = get_ai_command_bar_actions();
    let em = actions.iter().find(|a| a.id == "chat:export_markdown").unwrap();
    assert_eq!(em.icon, Some(IconName::FileCode));
}

#[test]
fn batch32_ai_export_markdown_shortcut() {
    let actions = get_ai_command_bar_actions();
    let em = actions.iter().find(|a| a.id == "chat:export_markdown").unwrap();
    assert_eq!(em.shortcut.as_deref(), Some("⇧⌘E"));
}

#[test]
fn batch32_ai_export_markdown_desc_mentions_markdown() {
    let actions = get_ai_command_bar_actions();
    let em = actions.iter().find(|a| a.id == "chat:export_markdown").unwrap();
    assert!(em.description.as_ref().unwrap().contains("Markdown"));
}

// ---------------------------------------------------------------------------
// 9. AI command bar: submit action details
// ---------------------------------------------------------------------------

#[test]
fn batch32_ai_submit_icon_is_arrow_up() {
    let actions = get_ai_command_bar_actions();
    let s = actions.iter().find(|a| a.id == "chat:submit").unwrap();
    assert_eq!(s.icon, Some(IconName::ArrowUp));
}

#[test]
fn batch32_ai_submit_section_is_actions() {
    let actions = get_ai_command_bar_actions();
    let s = actions.iter().find(|a| a.id == "chat:submit").unwrap();
    assert_eq!(s.section.as_deref(), Some("Actions"));
}

#[test]
fn batch32_ai_submit_shortcut_is_enter() {
    let actions = get_ai_command_bar_actions();
    let s = actions.iter().find(|a| a.id == "chat:submit").unwrap();
    assert_eq!(s.shortcut.as_deref(), Some("↵"));
}

#[test]
fn batch32_ai_submit_desc_mentions_send() {
    let actions = get_ai_command_bar_actions();
    let s = actions.iter().find(|a| a.id == "chat:submit").unwrap();
    assert!(
        s.description.as_ref().unwrap().contains("Send"),
        "submit desc should mention 'Send', got: {:?}",
        s.description
    );
}

// ---------------------------------------------------------------------------
// 10. Chat context: single model produces 2 actions minimum
// ---------------------------------------------------------------------------

#[test]
fn batch32_chat_single_model_no_flags_produces_2_actions() {
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
    assert_eq!(actions.len(), 2, "1 model + continue_in_chat = 2");
}

// --- merged from part_02.rs ---

#[test]
fn batch32_chat_single_model_both_flags_produces_4_actions() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "gpt4".into(),
            display_name: "GPT-4".into(),
            provider: "OpenAI".into(),
        }],
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions.len(), 4, "1 model + continue + copy + clear = 4");
}

#[test]
fn batch32_chat_single_model_title_matches_display_name() {
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
    assert_eq!(actions[0].title, "GPT-4");
}

// ---------------------------------------------------------------------------
// 11. Chat context: has_response=true without has_messages
// ---------------------------------------------------------------------------

#[test]
fn batch32_chat_has_response_no_messages_has_copy_no_clear() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    assert!(actions.iter().any(|a| a.id == "chat:copy_response"));
    assert!(!actions.iter().any(|a| a.id == "chat:clear_conversation"));
}

#[test]
fn batch32_chat_has_messages_no_response_has_clear_no_copy() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "chat:copy_response"));
    assert!(actions.iter().any(|a| a.id == "chat:clear_conversation"));
}

#[test]
fn batch32_chat_no_flags_only_continue() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "chat:continue_in_chat");
}

// ---------------------------------------------------------------------------
// 12. Notes command bar: find_in_note details
// ---------------------------------------------------------------------------

#[test]
fn batch32_notes_find_in_note_shortcut() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let fin = actions.iter().find(|a| a.id == "find_in_note").unwrap();
    assert_eq!(fin.shortcut.as_deref(), Some("⌘F"));
}

#[test]
fn batch32_notes_find_in_note_icon() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let fin = actions.iter().find(|a| a.id == "find_in_note").unwrap();
    assert_eq!(fin.icon, Some(IconName::MagnifyingGlass));
}

#[test]
fn batch32_notes_find_in_note_section() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    let fin = actions.iter().find(|a| a.id == "find_in_note").unwrap();
    assert_eq!(fin.section.as_deref(), Some("Edit"));
}

#[test]
fn batch32_notes_find_in_note_absent_without_selection() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "find_in_note"));
}

// ---------------------------------------------------------------------------
// 13. Notes: trash view blocks all selection-dependent actions
// ---------------------------------------------------------------------------

#[test]
fn batch32_notes_trash_no_duplicate_note() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "duplicate_note"));
}

#[test]
fn batch32_notes_trash_no_find_in_note() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "find_in_note"));
}

#[test]
fn batch32_notes_trash_no_format() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "format"));
}

#[test]
fn batch32_notes_trash_no_export() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "export"));
}

// ---------------------------------------------------------------------------
// 14. Note switcher: preview truncation with trim_end on > 60 chars
// ---------------------------------------------------------------------------

#[test]
fn batch32_note_switcher_61_char_preview_truncated_with_ellipsis() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "T".into(),
        char_count: 100,
        is_current: false,
        is_pinned: false,
        preview: "a".repeat(61),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(
        desc.ends_with('…'),
        "61-char preview should end with …, got: {}",
        desc
    );
}

#[test]
fn batch32_note_switcher_60_char_preview_no_truncation() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "T".into(),
        char_count: 100,
        is_current: false,
        is_pinned: false,
        preview: "b".repeat(60),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert!(
        !desc.ends_with('…'),
        "60-char preview should not be truncated, got: {}",
        desc
    );
    assert_eq!(desc.len(), 60);
}

#[test]
fn batch32_note_switcher_short_preview_no_truncation() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "T".into(),
        char_count: 100,
        is_current: false,
        is_pinned: false,
        preview: "hello world".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    let desc = actions[0].description.as_ref().unwrap();
    assert_eq!(desc, "hello world");
}

// ---------------------------------------------------------------------------
// 15. Note switcher: title without current indicator has no bullet
// ---------------------------------------------------------------------------

#[test]
fn batch32_note_switcher_non_current_no_bullet() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "My Note".into(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].title, "My Note");
}

#[test]
fn batch32_note_switcher_current_has_bullet() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "My Note".into(),
        char_count: 10,
        is_current: true,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].title, "• My Note");
}

#[test]
fn batch32_note_switcher_current_pinned_icon_star_filled() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "My Note".into(),
        char_count: 10,
        is_current: true,
        is_pinned: true,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

// ---------------------------------------------------------------------------
// 16. New chat: last_used icon is always BoltFilled
// ---------------------------------------------------------------------------

#[test]
fn batch32_new_chat_last_used_icon_bolt_filled() {
    let last_used = vec![NewChatModelInfo {
        model_id: "gpt4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
}

#[test]
fn batch32_new_chat_last_used_section_is_last_used_settings() {
    let last_used = vec![NewChatModelInfo {
        model_id: "gpt4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
}

#[test]
fn batch32_new_chat_last_used_desc_is_provider_display_name() {
    let last_used = vec![NewChatModelInfo {
        model_id: "gpt4".into(),
        display_name: "GPT-4".into(),
        provider: "openai".into(),
        provider_display_name: "OpenAI".into(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].description.as_deref(), Some("Uses OpenAI"));
}

// ---------------------------------------------------------------------------
// 17. New chat: model section always "Models" with Settings icon
// ---------------------------------------------------------------------------

#[test]
fn batch32_new_chat_model_section_is_models() {
    let models = vec![NewChatModelInfo {
        model_id: "claude".into(),
        display_name: "Claude".into(),
        provider: "anthropic".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].section.as_deref(), Some("Models"));
}

#[test]
fn batch32_new_chat_model_icon_is_settings() {
    let models = vec![NewChatModelInfo {
        model_id: "claude".into(),
        display_name: "Claude".into(),
        provider: "anthropic".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].icon, Some(IconName::Settings));
}

#[test]
fn batch32_new_chat_model_id_format() {
    let models = vec![NewChatModelInfo {
        model_id: "claude".into(),
        display_name: "Claude".into(),
        provider: "anthropic".into(),
        provider_display_name: "Anthropic".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].id, "model_anthropic::claude");
}

#[test]
fn batch32_new_chat_preset_section_is_presets() {
    let presets = vec![NewChatPresetInfo {
        id: "general".into(),
        name: "General".into(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].section.as_deref(), Some("Presets"));
}

// ---------------------------------------------------------------------------
// 18. to_deeplink_name: tab, newline, and numbers-only input
// ---------------------------------------------------------------------------

#[test]
fn batch32_to_deeplink_name_tab_and_newline() {
    assert_eq!(to_deeplink_name("test\ttab\nnewline"), "test-tab-newline");
}

#[test]
fn batch32_to_deeplink_name_numbers_only() {
    assert_eq!(to_deeplink_name("12345"), "12345");
}

#[test]
fn batch32_to_deeplink_name_leading_trailing_hyphens() {
    assert_eq!(to_deeplink_name("--hello--"), "hello");
}

#[test]
fn batch32_to_deeplink_name_single_word() {
    assert_eq!(to_deeplink_name("hello"), "hello");
}

// ---------------------------------------------------------------------------
// 19. format_shortcut_hint (on ActionsDialog): key conversions
// ---------------------------------------------------------------------------

#[test]
fn batch32_format_shortcut_hint_cmd_e() {
    let result = ActionsDialog::format_shortcut_hint("cmd+e");
    assert_eq!(result, "⌘E");
}

#[test]
fn batch32_format_shortcut_hint_all_modifiers() {
    let result = ActionsDialog::format_shortcut_hint("cmd+shift+ctrl+alt+k");
    assert_eq!(result, "⌘⇧⌃⌥K");
}

#[test]
fn batch32_format_shortcut_hint_enter_alone() {
    let result = ActionsDialog::format_shortcut_hint("enter");
    assert_eq!(result, "↵");
}

#[test]
fn batch32_format_shortcut_hint_meta_alias() {
    let result = ActionsDialog::format_shortcut_hint("meta+c");
    assert_eq!(result, "⌘C");
}

// ---------------------------------------------------------------------------
// 20. parse_shortcut_keycaps: various inputs
// ---------------------------------------------------------------------------

#[test]
fn batch32_parse_shortcut_keycaps_single_letter() {
    let caps = ActionsDialog::parse_shortcut_keycaps("E");
    assert_eq!(caps, vec!["E"]);
}

#[test]
fn batch32_parse_shortcut_keycaps_cmd_enter() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘↵");
    assert_eq!(caps, vec!["⌘", "↵"]);
}

#[test]
fn batch32_parse_shortcut_keycaps_slash() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘/");
    assert_eq!(caps, vec!["⌘", "/"]);
}

// --- merged from part_03.rs ---

#[test]
fn batch32_parse_shortcut_keycaps_space_symbol() {
    let caps = ActionsDialog::parse_shortcut_keycaps("␣");
    assert_eq!(caps, vec!["␣"]);
}

// ---------------------------------------------------------------------------
// 21. score_action: empty search returns zero
// ---------------------------------------------------------------------------

#[test]
fn batch32_score_action_empty_search_returns_zero() {
    let action = Action::new("test", "Test Action", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "");
    // Empty string is a prefix of everything, so prefix match gives 100
    assert!(
        score >= 100,
        "Empty search should prefix-match, got {}",
        score
    );
}

#[test]
fn batch32_score_action_prefix_match_100_plus() {
    let action = Action::new(
        "script:edit",
        "Edit Script",
        Some("Open in editor".into()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘E");
    let score = ActionsDialog::score_action(&action, "edit");
    // prefix (100) + description contains "edit" (15) = 115
    assert!(score >= 100, "Prefix match should be 100+, got {}", score);
}

#[test]
fn batch32_score_action_no_match_returns_zero() {
    let action = Action::new("test", "Test Action", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "xyz");
    assert_eq!(score, 0);
}

#[test]
fn batch32_score_action_desc_bonus_stacks() {
    let action = Action::new(
        "open",
        "Open File",
        Some("Open in editor".into()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "open");
    // prefix (100) + desc contains "open" (15) = 115
    assert_eq!(score, 115);
}

// ---------------------------------------------------------------------------
// 22. fuzzy_match: edge cases
// ---------------------------------------------------------------------------

#[test]
fn batch32_fuzzy_match_empty_needle_matches() {
    assert!(ActionsDialog::fuzzy_match("anything", ""));
}

#[test]
fn batch32_fuzzy_match_empty_haystack_empty_needle() {
    assert!(ActionsDialog::fuzzy_match("", ""));
}

#[test]
fn batch32_fuzzy_match_empty_haystack_nonempty_needle() {
    assert!(!ActionsDialog::fuzzy_match("", "a"));
}

#[test]
fn batch32_fuzzy_match_subsequence() {
    assert!(ActionsDialog::fuzzy_match("hello world", "hlo"));
}

#[test]
fn batch32_fuzzy_match_no_subsequence() {
    assert!(!ActionsDialog::fuzzy_match("hello", "ba"));
}

// ---------------------------------------------------------------------------
// 23. build_grouped_items_static: Headers style adds section headers
// ---------------------------------------------------------------------------

#[test]
fn batch32_grouped_items_headers_style_adds_header() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Sec1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Sec1"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // Should be: Header("Sec1"), Item(0), Item(1)
    assert_eq!(grouped.len(), 3);
    assert!(matches!(&grouped[0], GroupedActionItem::SectionHeader(s) if s == "Sec1"));
}

#[test]
fn batch32_grouped_items_separators_no_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Sec1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Sec2"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
    // No headers, just items
    assert_eq!(grouped.len(), 2);
    assert!(matches!(&grouped[0], GroupedActionItem::Item(_)));
}

#[test]
fn batch32_grouped_items_headers_two_sections() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // Header("S1"), Item(0), Header("S2"), Item(1)
    assert_eq!(grouped.len(), 4);
    assert!(matches!(&grouped[0], GroupedActionItem::SectionHeader(s) if s == "S1"));
    assert!(matches!(&grouped[2], GroupedActionItem::SectionHeader(s) if s == "S2"));
}

#[test]
fn batch32_grouped_items_empty_returns_empty() {
    let actions: Vec<Action> = vec![];
    let filtered: Vec<usize> = vec![];
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    assert!(grouped.is_empty());
}

// ---------------------------------------------------------------------------
// 24. coerce_action_selection: various patterns
// ---------------------------------------------------------------------------

#[test]
fn batch32_coerce_on_item_stays() {
    let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
    assert_eq!(coerce_action_selection(&rows, 0), Some(0));
}

#[test]
fn batch32_coerce_on_header_jumps_down_to_item() {
    let rows = vec![
        GroupedActionItem::SectionHeader("H".into()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

#[test]
fn batch32_coerce_trailing_header_jumps_up() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("H".into()),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(0));
}

#[test]
fn batch32_coerce_all_headers_returns_none() {
    let rows = vec![
        GroupedActionItem::SectionHeader("H1".into()),
        GroupedActionItem::SectionHeader("H2".into()),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

#[test]
fn batch32_coerce_empty_returns_none() {
    let rows: Vec<GroupedActionItem> = vec![];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

// ---------------------------------------------------------------------------
// 25. CommandBarConfig: ai_style vs main_menu_style differences
// ---------------------------------------------------------------------------

#[test]
fn batch32_config_ai_style_show_icons_true() {
    let config = CommandBarConfig::ai_style();
    assert!(config.dialog_config.show_icons);
}

#[test]
fn batch32_config_main_menu_show_icons_false() {
    let config = CommandBarConfig::main_menu_style();
    assert!(!config.dialog_config.show_icons);
}

#[test]
fn batch32_config_ai_style_show_footer_true() {
    let config = CommandBarConfig::ai_style();
    assert!(config.dialog_config.show_footer);
}

#[test]
fn batch32_config_main_menu_show_footer_false() {
    let config = CommandBarConfig::main_menu_style();
    assert!(!config.dialog_config.show_footer);
}

// ---------------------------------------------------------------------------
// 26. Script context: with_action_verb propagates to run_script title
// ---------------------------------------------------------------------------

#[test]
fn batch32_script_custom_verb_launch() {
    let script =
        crate::actions::types::ScriptInfo::with_action_verb("App", "/p/app", true, "Launch");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].title, "Launch");
}

#[test]
fn batch32_script_custom_verb_switch_to() {
    let script =
        crate::actions::types::ScriptInfo::with_action_verb("Window", "/p/w", false, "Switch to");
    let actions = get_script_context_actions(&script);
    assert_eq!(actions[0].title, "Switch To");
}

#[test]
fn batch32_script_custom_verb_desc_uses_verb() {
    let script =
        crate::actions::types::ScriptInfo::with_action_verb("Foo", "/p/foo", true, "Execute");
    let actions = get_script_context_actions(&script);
    assert!(
        actions[0].description.as_ref().unwrap().contains("Execute"),
        "run desc should use verb, got: {:?}",
        actions[0].description
    );
}

// ---------------------------------------------------------------------------
// 27. Script context: deeplink URL format in copy_deeplink description
// ---------------------------------------------------------------------------

#[test]
fn batch32_script_deeplink_url_format() {
    let script = crate::actions::types::ScriptInfo::new("My Script", "/p/my-script.ts");
    let actions = get_script_context_actions(&script);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert!(
        dl.description
            .as_ref()
            .unwrap()
            .contains("scriptkit://run/my-script"),
        "deeplink should contain URL, got: {:?}",
        dl.description
    );
}

#[test]
fn batch32_script_deeplink_shortcut() {
    let script = crate::actions::types::ScriptInfo::new("Test", "/p/test.ts");
    let actions = get_script_context_actions(&script);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert_eq!(dl.shortcut.as_deref(), Some("⌘⇧D"));
}

#[test]
fn batch32_builtin_deeplink_url_format() {
    let builtin = crate::actions::types::ScriptInfo::builtin("Clipboard History");
    let actions = get_script_context_actions(&builtin);
    let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
    assert!(dl
        .description
        .as_ref()
        .unwrap()
        .contains("scriptkit://run/clipboard-history"),);
}

// ---------------------------------------------------------------------------
// 28. Clipboard: save_snippet and save_file always present
// ---------------------------------------------------------------------------

#[test]
fn batch32_clipboard_text_has_save_snippet() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clip:clipboard_save_snippet"));
}

#[test]
fn batch32_clipboard_image_has_save_snippet() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "".into(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clip:clipboard_save_snippet"));
}

#[test]
fn batch32_clipboard_text_has_save_file() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clip:clipboard_save_file"));
}

// ---------------------------------------------------------------------------
// 29. Action builder: cached lowercase fields
// ---------------------------------------------------------------------------

#[test]
fn batch32_action_title_lower_is_precomputed() {
    let action = Action::new("id", "Hello World", None, ActionCategory::ScriptContext);
    assert_eq!(action.title_lower, "hello world");
}

#[test]
fn batch32_action_description_lower_is_precomputed() {
    let action = Action::new(
        "id",
        "T",
        Some("Open In EDITOR".into()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(action.description_lower.as_deref(), Some("open in editor"));
}

#[test]
fn batch32_action_no_description_lower_is_none() {
    let action = Action::new("id", "T", None, ActionCategory::ScriptContext);
    assert!(action.description_lower.is_none());
}

#[test]
fn batch32_action_shortcut_lower_after_with_shortcut() {
    let action = Action::new("id", "T", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧C");
    assert_eq!(action.shortcut_lower.as_deref(), Some("⌘⇧c"));
}

// ---------------------------------------------------------------------------
// 30. Cross-context: all clipboard actions have ActionCategory::ScriptContext
// ---------------------------------------------------------------------------

#[test]
fn batch32_all_clipboard_actions_are_script_context() {
    let entry = ClipboardEntryInfo {
        id: "1".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "hello".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    for a in &actions {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "clipboard action {} should be ScriptContext",
            a.id
        );
    }
}

#[test]
fn batch32_all_file_actions_are_script_context() {
    let info = FileInfo {
        path: "/p/test.txt".into(),
        name: "test.txt".into(),
        file_type: FileType::Document,
        is_dir: false,
    };
    let actions = get_file_context_actions(&info);
    for a in &actions {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "file action {} should be ScriptContext",
            a.id
        );
    }
}

#[test]
fn batch32_all_path_actions_are_script_context() {
    let info = PathInfo::new("test.txt", "/p/test.txt", false);
    let actions = get_path_context_actions(&info);
    for a in &actions {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "path action {} should be ScriptContext",
            a.id
        );
    }
}

#[test]
fn batch32_all_notes_actions_are_script_context() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    for a in &actions {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "notes action {} should be ScriptContext",
            a.id
        );
    }
}

#[test]
fn batch32_all_ai_bar_actions_are_script_context() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "AI action {} should be ScriptContext",
            a.id
        );
    }
}

// --- merged from part_04.rs ---

#[test]
fn batch32_all_new_chat_actions_are_script_context() {
    let models = vec![NewChatModelInfo {
        model_id: "m".into(),
        display_name: "M".into(),
        provider: "p".into(),
        provider_display_name: "P".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    for a in &actions {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "new chat action {} should be ScriptContext",
            a.id
        );
    }
}
