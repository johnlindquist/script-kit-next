// --- merged from part_01.rs ---
//! Batch 24: Dialog builtin action validation tests
//!
//! 131 tests across 30 categories validating random built-in action behaviors.

use super::builders::*;
use super::command_bar::CommandBarConfig;
use super::dialog::*;
use super::types::*;
use crate::clipboard_history::ContentType;
use crate::designs::icon_variations::IconName;
use crate::file_search::{FileInfo, FileType};
use crate::prompts::PathInfo;

// ============================================================
// 1. Agent context: is_agent flag enables agent-specific actions
// ============================================================

#[test]
fn batch24_agent_has_edit_agent_title() {
    let mut script = ScriptInfo::new("My Agent", "/path/to/agent.md");
    script.is_agent = true;
    script.is_script = false;
    let actions = get_script_context_actions(&script);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert_eq!(edit.title, "Edit Agent");
}

#[test]
fn batch24_agent_has_copy_content() {
    let mut script = ScriptInfo::new("My Agent", "/path/to/agent.md");
    script.is_agent = true;
    script.is_script = false;
    let actions = get_script_context_actions(&script);
    assert!(actions.iter().any(|a| a.id == "copy_content"));
}

#[test]
fn batch24_agent_lacks_view_logs() {
    let mut script = ScriptInfo::new("My Agent", "/path/to/agent.md");
    script.is_agent = true;
    script.is_script = false;
    let actions = get_script_context_actions(&script);
    assert!(!actions.iter().any(|a| a.id == "view_logs"));
}

#[test]
fn batch24_agent_has_reveal_in_finder() {
    let mut script = ScriptInfo::new("My Agent", "/path/to/agent.md");
    script.is_agent = true;
    script.is_script = false;
    let actions = get_script_context_actions(&script);
    assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
}

// ============================================================
// 2. Agent edit description mentions agent file
// ============================================================

#[test]
fn batch24_agent_edit_desc_mentions_agent_file() {
    let mut script = ScriptInfo::new("Agent X", "/path/to/agent");
    script.is_agent = true;
    script.is_script = false;
    let actions = get_script_context_actions(&script);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert!(edit.description.as_ref().unwrap().contains("agent"));
}

#[test]
fn batch24_agent_reveal_desc_mentions_agent() {
    let mut script = ScriptInfo::new("Agent X", "/path/to/agent");
    script.is_agent = true;
    script.is_script = false;
    let actions = get_script_context_actions(&script);
    let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
    assert!(reveal.description.as_ref().unwrap().contains("agent"));
}

#[test]
fn batch24_agent_copy_path_desc_mentions_agent() {
    let mut script = ScriptInfo::new("Agent X", "/path/to/agent");
    script.is_agent = true;
    script.is_script = false;
    let actions = get_script_context_actions(&script);
    let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
    assert!(cp.description.as_ref().unwrap().contains("agent"));
}

#[test]
fn batch24_script_edit_desc_mentions_editor() {
    let script = ScriptInfo::new("My Script", "/path/to/script.ts");
    let actions = get_script_context_actions(&script);
    let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
    assert!(edit.description.as_ref().unwrap().contains("$EDITOR"));
}

// ============================================================
// 3. ScriptInfo constructors: is_agent defaults to false
// ============================================================

#[test]
fn batch24_new_is_agent_false() {
    let s = ScriptInfo::new("test", "/path");
    assert!(!s.is_agent);
}

#[test]
fn batch24_builtin_is_agent_false() {
    let s = ScriptInfo::builtin("Clipboard");
    assert!(!s.is_agent);
}

#[test]
fn batch24_scriptlet_is_agent_false() {
    let s = ScriptInfo::scriptlet("Open URL", "/path.md", None, None);
    assert!(!s.is_agent);
}

#[test]
fn batch24_with_shortcut_is_agent_false() {
    let s = ScriptInfo::with_shortcut("test", "/path", Some("cmd+t".to_string()));
    assert!(!s.is_agent);
}

#[test]
fn batch24_with_all_is_agent_false() {
    let s = ScriptInfo::with_all("test", "/path", true, "Run", None, None);
    assert!(!s.is_agent);
}

// ============================================================
// 4. Chat context: has_response/has_messages flag combinations
// ============================================================

#[test]
fn batch24_chat_no_response_no_messages() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    // Only continue_in_chat
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "continue_in_chat");
}

#[test]
fn batch24_chat_response_only() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: false,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions.len(), 2);
    assert!(actions.iter().any(|a| a.id == "copy_response"));
    assert!(!actions.iter().any(|a| a.id == "clear_conversation"));
}

#[test]
fn batch24_chat_messages_only() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions.len(), 2);
    assert!(!actions.iter().any(|a| a.id == "copy_response"));
    assert!(actions.iter().any(|a| a.id == "clear_conversation"));
}

#[test]
fn batch24_chat_both_flags() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![],
        has_messages: true,
        has_response: true,
    };
    let actions = get_chat_context_actions(&info);
    assert_eq!(actions.len(), 3);
    assert!(actions.iter().any(|a| a.id == "copy_response"));
    assert!(actions.iter().any(|a| a.id == "clear_conversation"));
}

// ============================================================
// 5. Chat context: model checkmark only for current model
// ============================================================

#[test]
fn batch24_chat_current_model_checkmark() {
    let info = ChatPromptInfo {
        current_model: Some("GPT-4".to_string()),
        available_models: vec![
            ChatModelInfo {
                id: "gpt4".to_string(),
                display_name: "GPT-4".to_string(),
                provider: "OpenAI".to_string(),
            },
            ChatModelInfo {
                id: "claude".to_string(),
                display_name: "Claude".to_string(),
                provider: "Anthropic".to_string(),
            },
        ],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let gpt4 = actions
        .iter()
        .find(|a| a.id == "select_model_gpt4")
        .unwrap();
    assert!(gpt4.title.contains("✓"));
    let claude = actions
        .iter()
        .find(|a| a.id == "select_model_claude")
        .unwrap();
    assert!(!claude.title.contains("✓"));
}

#[test]
fn batch24_chat_no_current_model_no_checkmark() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "gpt4".to_string(),
            display_name: "GPT-4".to_string(),
            provider: "OpenAI".to_string(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let gpt4 = actions
        .iter()
        .find(|a| a.id == "select_model_gpt4")
        .unwrap();
    assert!(!gpt4.title.contains("✓"));
}

#[test]
fn batch24_chat_model_description_via_provider() {
    let info = ChatPromptInfo {
        current_model: None,
        available_models: vec![ChatModelInfo {
            id: "m1".to_string(),
            display_name: "Model One".to_string(),
            provider: "TestProvider".to_string(),
        }],
        has_messages: false,
        has_response: false,
    };
    let actions = get_chat_context_actions(&info);
    let m1 = actions.iter().find(|a| a.id == "select_model_m1").unwrap();
    assert_eq!(m1.description.as_ref().unwrap(), "via TestProvider");
}

// ============================================================
// 6. Clipboard macOS-specific image actions (cfg(target_os = "macos"))
// ============================================================

#[cfg(target_os = "macos")]
#[test]
fn batch24_clipboard_image_has_open_with() {
    let entry = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".to_string(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_open_with"));
}

#[cfg(target_os = "macos")]
#[test]
fn batch24_clipboard_image_has_annotate_cleanshot() {
    let entry = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".to_string(),
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
fn batch24_clipboard_image_has_upload_cleanshot() {
    let entry = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".to_string(),
        image_dimensions: Some((100, 100)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_upload_cleanshot"));
}

#[cfg(target_os = "macos")]
#[test]
fn batch24_clipboard_text_no_open_with() {
    let entry = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clipboard_open_with"));
}

// ============================================================
// 7. Clipboard: OCR only for image, not text
// ============================================================

#[test]
fn batch24_clipboard_image_has_ocr() {
    let entry = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".to_string(),
        image_dimensions: Some((800, 600)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_ocr"));
}

#[test]
fn batch24_clipboard_text_no_ocr() {
    let entry = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(!actions.iter().any(|a| a.id == "clipboard_ocr"));
}

#[test]
fn batch24_clipboard_ocr_shortcut() {
    let entry = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".to_string(),
        image_dimensions: Some((800, 600)),
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    let ocr = actions.iter().find(|a| a.id == "clipboard_ocr").unwrap();
    assert_eq!(ocr.shortcut.as_ref().unwrap(), "⇧⌘C");
}

// ============================================================
// 8. Clipboard: image with None dimensions still gets image actions
// ============================================================

#[test]
fn batch24_clipboard_image_no_dimensions_still_has_ocr() {
    let entry = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_ocr"));
}

#[test]
fn batch24_clipboard_image_no_dimensions_has_paste() {
    let entry = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Image,
        pinned: false,
        preview: "img".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let actions = get_clipboard_history_context_actions(&entry);
    assert!(actions.iter().any(|a| a.id == "clipboard_paste"));
}

// ============================================================
// 9. Notes: trash mode minimal actions
// ============================================================

#[test]
fn batch24_notes_trash_minimal_actions() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // Trash: new_note, browse_notes, enable_auto_sizing (3)
    assert_eq!(actions.len(), 3);
}

#[test]
fn batch24_notes_trash_has_new_note() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "new_note"));
}

#[test]
fn batch24_notes_trash_has_browse() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(actions.iter().any(|a| a.id == "browse_notes"));
}

// --- merged from part_02.rs ---

#[test]
fn batch24_notes_trash_no_duplicate() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "duplicate_note"));
}

#[test]
fn batch24_notes_trash_no_find() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: true,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    assert!(!actions.iter().any(|a| a.id == "find_in_note"));
}

// ============================================================
// 10. Notes full mode with selection: maximum actions
// ============================================================

#[test]
fn batch24_notes_full_mode_count() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note, duplicate_note, browse_notes, find_in_note, format,
    // copy_note_as, copy_deeplink, create_quicklink, export, enable_auto_sizing = 10
    assert_eq!(actions.len(), 10);
}

#[test]
fn batch24_notes_full_auto_sizing_enabled_count() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: true,
    };
    let actions = get_notes_command_bar_actions(&info);
    // Same minus enable_auto_sizing = 9
    assert_eq!(actions.len(), 9);
}

#[test]
fn batch24_notes_no_selection_count() {
    let info = NotesInfo {
        has_selection: false,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    // new_note, browse_notes, enable_auto_sizing = 3
    assert_eq!(actions.len(), 3);
}

// ============================================================
// 11. Notes icon assignments
// ============================================================

#[test]
fn batch24_notes_new_note_icon_plus() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    let actions = get_notes_command_bar_actions(&info);
    let new_note = actions.iter().find(|a| a.id == "new_note").unwrap();
    assert_eq!(new_note.icon, Some(IconName::Plus));
}

#[test]
fn batch24_notes_browse_icon_folder_open() {
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
fn batch24_notes_find_icon_magnifying() {
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
fn batch24_notes_auto_sizing_icon_settings() {
    let info = NotesInfo {
        has_selection: false,
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
// 12. Note switcher: notes with empty preview fall back to char count
// ============================================================

#[test]
fn batch24_note_switcher_empty_preview_zero_chars() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".to_string(),
        title: "Empty".to_string(),
        char_count: 0,
        is_current: false,
        is_pinned: false,
        preview: "".to_string(),
        relative_time: "".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_ref().unwrap(), "0 chars");
}

#[test]
fn batch24_note_switcher_empty_preview_one_char() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".to_string(),
        title: "One".to_string(),
        char_count: 1,
        is_current: false,
        is_pinned: false,
        preview: "".to_string(),
        relative_time: "".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_ref().unwrap(), "1 char");
}

#[test]
fn batch24_note_switcher_empty_preview_with_time() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".to_string(),
        title: "T".to_string(),
        char_count: 50,
        is_current: false,
        is_pinned: false,
        preview: "".to_string(),
        relative_time: "5m ago".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description.as_ref().unwrap(), "5m ago");
}

#[test]
fn batch24_note_switcher_preview_with_time() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".to_string(),
        title: "T".to_string(),
        char_count: 50,
        is_current: false,
        is_pinned: false,
        preview: "Some content".to_string(),
        relative_time: "2h ago".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert!(actions[0].description.as_ref().unwrap().contains(" · "));
}

// ============================================================
// 13. Note switcher: pinned + current icon priority
// ============================================================

#[test]
fn batch24_note_switcher_pinned_icon() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "a".to_string(),
        title: "Pinned".to_string(),
        char_count: 10,
        is_current: false,
        is_pinned: true,
        preview: "".to_string(),
        relative_time: "".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

#[test]
fn batch24_note_switcher_current_icon() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "a".to_string(),
        title: "Current".to_string(),
        char_count: 10,
        is_current: true,
        is_pinned: false,
        preview: "".to_string(),
        relative_time: "".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::Check));
}

#[test]
fn batch24_note_switcher_pinned_trumps_current() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "a".to_string(),
        title: "Both".to_string(),
        char_count: 10,
        is_current: true,
        is_pinned: true,
        preview: "".to_string(),
        relative_time: "".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::StarFilled));
}

#[test]
fn batch24_note_switcher_regular_icon() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "a".to_string(),
        title: "Regular".to_string(),
        char_count: 10,
        is_current: false,
        is_pinned: false,
        preview: "".to_string(),
        relative_time: "".to_string(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].icon, Some(IconName::File));
}

// ============================================================
// 14. AI command bar: all 12 actions present
// ============================================================

#[test]
fn batch24_ai_command_bar_total_12() {
    let actions = get_ai_command_bar_actions();
    assert_eq!(actions.len(), 12);
}

#[test]
fn batch24_ai_command_bar_all_have_icons() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert!(a.icon.is_some(), "Action {} missing icon", a.id);
    }
}

#[test]
fn batch24_ai_command_bar_all_have_sections() {
    let actions = get_ai_command_bar_actions();
    for a in &actions {
        assert!(a.section.is_some(), "Action {} missing section", a.id);
    }
}

#[test]
fn batch24_ai_command_bar_response_section_count() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Response"))
        .count();
    assert_eq!(count, 3);
}

#[test]
fn batch24_ai_command_bar_actions_section_count() {
    let actions = get_ai_command_bar_actions();
    let count = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Actions"))
        .count();
    assert_eq!(count, 4);
}

// ============================================================
// 15. AI command bar: specific shortcut and icon pairs
// ============================================================

#[test]
fn batch24_ai_export_markdown_shortcut_icon() {
    let actions = get_ai_command_bar_actions();
    let export = actions.iter().find(|a| a.id == "export_markdown").unwrap();
    assert_eq!(export.shortcut.as_ref().unwrap(), "⇧⌘E");
    assert_eq!(export.icon, Some(IconName::FileCode));
}

#[test]
fn batch24_ai_branch_from_last_no_shortcut() {
    let actions = get_ai_command_bar_actions();
    let branch = actions.iter().find(|a| a.id == "branch_from_last").unwrap();
    assert!(branch.shortcut.is_none());
    assert_eq!(branch.icon, Some(IconName::ArrowRight));
}

#[test]
fn batch24_ai_change_model_no_shortcut() {
    let actions = get_ai_command_bar_actions();
    let model = actions.iter().find(|a| a.id == "change_model").unwrap();
    assert!(model.shortcut.is_none());
    assert_eq!(model.icon, Some(IconName::Settings));
}

#[test]
fn batch24_ai_toggle_shortcuts_help_shortcut() {
    let actions = get_ai_command_bar_actions();
    let help = actions
        .iter()
        .find(|a| a.id == "toggle_shortcuts_help")
        .unwrap();
    assert_eq!(help.shortcut.as_ref().unwrap(), "⌘/");
}

// ============================================================
// 16. New chat actions: empty inputs
// ============================================================

#[test]
fn batch24_new_chat_all_empty() {
    let actions = get_new_chat_actions(&[], &[], &[]);
    assert!(actions.is_empty());
}

#[test]
fn batch24_new_chat_only_last_used() {
    let last = vec![NewChatModelInfo {
        model_id: "m1".to_string(),
        display_name: "Model 1".to_string(),
        provider: "p".to_string(),
        provider_display_name: "Provider".to_string(),
    }];
    let actions = get_new_chat_actions(&last, &[], &[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
}

#[test]
fn batch24_new_chat_only_presets() {
    let presets = vec![NewChatPresetInfo {
        id: "general".to_string(),
        name: "General".to_string(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_deref(), Some("Presets"));
}

#[test]
fn batch24_new_chat_only_models() {
    let models = vec![NewChatModelInfo {
        model_id: "m1".to_string(),
        display_name: "Model 1".to_string(),
        provider: "p".to_string(),
        provider_display_name: "Provider".to_string(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].section.as_deref(), Some("Models"));
}

#[test]
fn batch24_new_chat_mixed() {
    let last = vec![NewChatModelInfo {
        model_id: "m1".to_string(),
        display_name: "M1".to_string(),
        provider: "p".to_string(),
        provider_display_name: "P".to_string(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "g".to_string(),
        name: "G".to_string(),
        icon: IconName::Star,
    }];
    let models = vec![NewChatModelInfo {
        model_id: "m2".to_string(),
        display_name: "M2".to_string(),
        provider: "p".to_string(),
        provider_display_name: "P".to_string(),
    }];
    let actions = get_new_chat_actions(&last, &presets, &models);
    assert_eq!(actions.len(), 3);
}

// ============================================================
// 17. New chat actions: icon assignments
// ============================================================

#[test]
fn batch24_new_chat_last_used_icon_bolt() {
    let last = vec![NewChatModelInfo {
        model_id: "m1".to_string(),
        display_name: "M1".to_string(),
        provider: "p".to_string(),
        provider_display_name: "P".to_string(),
    }];
    let actions = get_new_chat_actions(&last, &[], &[]);
    assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
}

#[test]
fn batch24_new_chat_model_icon_settings() {
    let models = vec![NewChatModelInfo {
        model_id: "m1".to_string(),
        display_name: "M1".to_string(),
        provider: "p".to_string(),
        provider_display_name: "P".to_string(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].icon, Some(IconName::Settings));
}

#[test]
fn batch24_new_chat_preset_icon_preserved() {
    let presets = vec![NewChatPresetInfo {
        id: "g".to_string(),
        name: "General".to_string(),
        icon: IconName::Code,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].icon, Some(IconName::Code));
}

// --- merged from part_03.rs ---

#[test]
fn batch24_new_chat_preset_no_description() {
    let presets = vec![NewChatPresetInfo {
        id: "g".to_string(),
        name: "General".to_string(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert!(actions[0].description.is_none());
}

// ============================================================
// 18. Path context: exact action IDs in order for directory
// ============================================================

#[test]
fn batch24_path_dir_action_ids_ordered() {
    let p = PathInfo::new("Documents", "/Users/test/Documents", true);
    let actions = get_path_context_actions(&p);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(
        ids,
        vec![
            "open_directory",
            "copy_path",
            "open_in_finder",
            "open_in_editor",
            "open_in_terminal",
            "copy_filename",
            "move_to_trash",
        ]
    );
}

#[test]
fn batch24_path_file_action_ids_ordered() {
    let p = PathInfo::new("file.txt", "/Users/test/file.txt", false);
    let actions = get_path_context_actions(&p);
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(
        ids,
        vec![
            "select_file",
            "copy_path",
            "open_in_finder",
            "open_in_editor",
            "open_in_terminal",
            "copy_filename",
            "move_to_trash",
        ]
    );
}

#[test]
fn batch24_path_always_7_actions() {
    let dir = PathInfo::new("d", "/d", true);
    let file = PathInfo::new("f", "/f", false);
    assert_eq!(get_path_context_actions(&dir).len(), 7);
    assert_eq!(get_path_context_actions(&file).len(), 7);
}

// ============================================================
// 19. Path context: shortcut assignments
// ============================================================

#[test]
fn batch24_path_copy_path_shortcut() {
    let p = PathInfo::new("f", "/f", false);
    let actions = get_path_context_actions(&p);
    let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
    assert_eq!(cp.shortcut.as_ref().unwrap(), "⌘⇧C");
}

#[test]
fn batch24_path_open_in_finder_shortcut() {
    let p = PathInfo::new("f", "/f", false);
    let actions = get_path_context_actions(&p);
    let f = actions.iter().find(|a| a.id == "open_in_finder").unwrap();
    assert_eq!(f.shortcut.as_ref().unwrap(), "⌘⇧F");
}

#[test]
fn batch24_path_open_in_terminal_shortcut() {
    let p = PathInfo::new("f", "/f", false);
    let actions = get_path_context_actions(&p);
    let t = actions.iter().find(|a| a.id == "open_in_terminal").unwrap();
    assert_eq!(t.shortcut.as_ref().unwrap(), "⌘T");
}

#[test]
fn batch24_path_copy_filename_no_shortcut() {
    let p = PathInfo::new("f", "/f", false);
    let actions = get_path_context_actions(&p);
    let cf = actions.iter().find(|a| a.id == "copy_filename").unwrap();
    assert!(cf.shortcut.is_none());
}

#[test]
fn batch24_path_move_to_trash_shortcut() {
    let p = PathInfo::new("f", "/f", false);
    let actions = get_path_context_actions(&p);
    let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
    assert_eq!(trash.shortcut.as_ref().unwrap(), "⌘⌫");
}

// ============================================================
// 20. File context: macOS action count difference
// ============================================================

#[cfg(target_os = "macos")]
#[test]
fn batch24_file_context_macos_file_count() {
    let f = FileInfo {
        path: "/test/f.txt".to_string(),
        name: "f.txt".to_string(),
        file_type: FileType::File,
        is_dir: false,
    };
    let actions = get_file_context_actions(&f);
    // open_file, reveal, quick_look, open_with, show_info, copy_path, copy_filename = 7
    assert_eq!(actions.len(), 7);
}

#[cfg(target_os = "macos")]
#[test]
fn batch24_file_context_macos_dir_count() {
    let f = FileInfo {
        path: "/test/d".to_string(),
        name: "d".to_string(),
        file_type: FileType::Directory,
        is_dir: true,
    };
    let actions = get_file_context_actions(&f);
    // open_directory, reveal, open_with, show_info, copy_path, copy_filename = 6
    // (no quick_look for dirs)
    assert_eq!(actions.len(), 6);
}

// ============================================================
// 21. to_deeplink_name: additional edge cases
// ============================================================

#[test]
fn batch24_deeplink_numeric_only() {
    assert_eq!(to_deeplink_name("123"), "123");
}

#[test]
fn batch24_deeplink_single_char() {
    assert_eq!(to_deeplink_name("a"), "a");
}

#[test]
fn batch24_deeplink_all_special_empty() {
    assert_eq!(to_deeplink_name("!@#$%"), "");
}

#[test]
fn batch24_deeplink_mixed_unicode() {
    let result = to_deeplink_name("Café Script");
    assert!(result.contains("caf"));
    assert!(result.contains("script"));
}

#[test]
fn batch24_deeplink_underscores_to_hyphens() {
    assert_eq!(to_deeplink_name("hello_world"), "hello-world");
}

// ============================================================
// 22. format_shortcut_hint (dialog.rs version): alias coverage
// ============================================================

#[test]
fn batch24_format_hint_command_alias() {
    assert_eq!(ActionsDialog::format_shortcut_hint("command+c"), "⌘C");
}

#[test]
fn batch24_format_hint_meta_alias() {
    assert_eq!(ActionsDialog::format_shortcut_hint("meta+c"), "⌘C");
}

#[test]
fn batch24_format_hint_super_alias() {
    assert_eq!(ActionsDialog::format_shortcut_hint("super+c"), "⌘C");
}

#[test]
fn batch24_format_hint_control_alias() {
    assert_eq!(ActionsDialog::format_shortcut_hint("control+c"), "⌃C");
}

#[test]
fn batch24_format_hint_opt_alias() {
    assert_eq!(ActionsDialog::format_shortcut_hint("opt+c"), "⌥C");
}

#[test]
fn batch24_format_hint_option_alias() {
    assert_eq!(ActionsDialog::format_shortcut_hint("option+c"), "⌥C");
}

#[test]
fn batch24_format_hint_return_alias() {
    assert_eq!(ActionsDialog::format_shortcut_hint("cmd+return"), "⌘↵");
}

#[test]
fn batch24_format_hint_esc_alias() {
    assert_eq!(ActionsDialog::format_shortcut_hint("esc"), "⎋");
}

// ============================================================
// 23. parse_shortcut_keycaps: modifiers and special keys
// ============================================================

#[test]
fn batch24_keycaps_single_modifier() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘");
    assert_eq!(caps, vec!["⌘"]);
}

#[test]
fn batch24_keycaps_modifier_and_letter() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘C");
    assert_eq!(caps, vec!["⌘", "C"]);
}

#[test]
fn batch24_keycaps_all_modifiers() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌃⌥⇧⌘C");
    assert_eq!(caps, vec!["⌃", "⌥", "⇧", "⌘", "C"]);
}

#[test]
fn batch24_keycaps_arrows() {
    let caps = ActionsDialog::parse_shortcut_keycaps("↑↓←→");
    assert_eq!(caps, vec!["↑", "↓", "←", "→"]);
}

#[test]
fn batch24_keycaps_lowercase_uppercased() {
    let caps = ActionsDialog::parse_shortcut_keycaps("⌘e");
    assert_eq!(caps, vec!["⌘", "E"]);
}

// ============================================================
// 24. score_action: scoring tiers with cached lowercase
// ============================================================

#[test]
fn batch24_score_prefix_match() {
    let action = Action::new(
        "id",
        "Edit Script",
        Some("Open editor".to_string()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "edit");
    assert!(score >= 100);
}

#[test]
fn batch24_score_contains_match() {
    let action = Action::new("id", "Copy Edit Path", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "edit");
    assert!(score >= 50);
    assert!(score < 100);
}

#[test]
fn batch24_score_no_match() {
    let action = Action::new("id", "Copy Path", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "xyz");
    assert_eq!(score, 0);
}

#[test]
fn batch24_score_description_bonus() {
    let action = Action::new(
        "id",
        "Open File",
        Some("Edit in editor".to_string()),
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "editor");
    assert!(score >= 15);
}

#[test]
fn batch24_score_shortcut_bonus() {
    let action =
        Action::new("id", "Open File", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
    let score = ActionsDialog::score_action(&action, "⌘e");
    assert!(score >= 10);
}

// ============================================================
// 25. fuzzy_match: edge cases
// ============================================================

#[test]
fn batch24_fuzzy_exact() {
    assert!(ActionsDialog::fuzzy_match("edit", "edit"));
}

#[test]
fn batch24_fuzzy_subsequence() {
    assert!(ActionsDialog::fuzzy_match("edit script", "esc"));
}

#[test]
fn batch24_fuzzy_no_match() {
    assert!(!ActionsDialog::fuzzy_match("abc", "xyz"));
}

#[test]
fn batch24_fuzzy_empty_needle() {
    assert!(ActionsDialog::fuzzy_match("abc", ""));
}

#[test]
fn batch24_fuzzy_needle_longer() {
    assert!(!ActionsDialog::fuzzy_match("ab", "abc"));
}

// ============================================================
// 26. build_grouped_items_static: section style effects
// ============================================================

#[test]
fn batch24_grouped_headers_adds_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // S1 header, A item, S2 header, B item = 4
    assert_eq!(grouped.len(), 4);
}

#[test]
fn batch24_grouped_same_section_one_header() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S1"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    // S1 header, A, B = 3
    assert_eq!(grouped.len(), 3);
}

#[test]
fn batch24_grouped_separators_no_headers() {
    let actions = vec![
        Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
        Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
    ];
    let filtered: Vec<usize> = (0..actions.len()).collect();
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
    // Just items, no headers
    assert_eq!(grouped.len(), 2);
}

#[test]
fn batch24_grouped_empty_filtered() {
    let actions = vec![Action::new("a", "A", None, ActionCategory::ScriptContext)];
    let filtered: Vec<usize> = vec![];
    let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
    assert!(grouped.is_empty());
}

// ============================================================
// 27. coerce_action_selection: header skipping
// ============================================================

#[test]
fn batch24_coerce_on_item_stays() {
    let rows = vec![
        GroupedActionItem::SectionHeader("S".to_string()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(1));
}

#[test]
fn batch24_coerce_header_skips_down() {
    let rows = vec![
        GroupedActionItem::SectionHeader("S".to_string()),
        GroupedActionItem::Item(0),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

#[test]
fn batch24_coerce_trailing_header_searches_up() {
    let rows = vec![
        GroupedActionItem::Item(0),
        GroupedActionItem::SectionHeader("S".to_string()),
    ];
    assert_eq!(coerce_action_selection(&rows, 1), Some(0));
}

#[test]
fn batch24_coerce_all_headers_none() {
    let rows = vec![
        GroupedActionItem::SectionHeader("A".to_string()),
        GroupedActionItem::SectionHeader("B".to_string()),
    ];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

#[test]
fn batch24_coerce_empty_none() {
    let rows: Vec<GroupedActionItem> = vec![];
    assert_eq!(coerce_action_selection(&rows, 0), None);
}

// ============================================================
// 28. CommandBarConfig preset field values
// ============================================================

#[test]
fn batch24_cmdbar_default_close_flags() {
    let config = CommandBarConfig::default();
    assert!(config.close_on_select);
    assert!(config.close_on_escape);
    assert!(config.close_on_click_outside);
}

#[test]
fn batch24_cmdbar_ai_style_search_top() {
    let config = CommandBarConfig::ai_style();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Top);
    assert!(config.dialog_config.show_icons);
    assert!(config.dialog_config.show_footer);
}

#[test]
fn batch24_cmdbar_main_menu_search_bottom() {
    let config = CommandBarConfig::main_menu_style();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Bottom);
    assert!(!config.dialog_config.show_icons);
    assert!(!config.dialog_config.show_footer);
}

// --- merged from part_04.rs ---

#[test]
fn batch24_cmdbar_no_search_hidden() {
    let config = CommandBarConfig::no_search();
    assert_eq!(config.dialog_config.search_position, SearchPosition::Hidden);
}

#[test]
fn batch24_cmdbar_notes_style_separators() {
    let config = CommandBarConfig::notes_style();
    assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
    assert!(config.dialog_config.show_icons);
    assert!(config.dialog_config.show_footer);
}

// ============================================================
// 29. Action builder: defaults and chaining
// ============================================================

#[test]
fn batch24_action_default_has_action_false() {
    let a = Action::new("id", "Title", None, ActionCategory::ScriptContext);
    assert!(!a.has_action);
}

#[test]
fn batch24_action_default_value_none() {
    let a = Action::new("id", "Title", None, ActionCategory::ScriptContext);
    assert!(a.value.is_none());
}

#[test]
fn batch24_action_default_icon_none() {
    let a = Action::new("id", "Title", None, ActionCategory::ScriptContext);
    assert!(a.icon.is_none());
}

#[test]
fn batch24_action_default_section_none() {
    let a = Action::new("id", "Title", None, ActionCategory::ScriptContext);
    assert!(a.section.is_none());
}

#[test]
fn batch24_action_chain_preserves_all() {
    let a = Action::new(
        "id",
        "Title",
        Some("Desc".to_string()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘C")
    .with_icon(IconName::Copy)
    .with_section("Section");
    assert_eq!(a.shortcut.as_deref(), Some("⌘C"));
    assert_eq!(a.icon, Some(IconName::Copy));
    assert_eq!(a.section.as_deref(), Some("Section"));
    assert_eq!(a.description.as_deref(), Some("Desc"));
}

// ============================================================
// 30. Cross-context: all actions have ScriptContext category
// ============================================================

#[test]
fn batch24_cross_script_all_script_context() {
    let script = ScriptInfo::new("test", "/path");
    for a in get_script_context_actions(&script) {
        assert_eq!(a.category, ActionCategory::ScriptContext);
    }
}

#[test]
fn batch24_cross_clipboard_all_script_context() {
    let entry = ClipboardEntryInfo {
        id: "1".to_string(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".to_string(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    for a in get_clipboard_history_context_actions(&entry) {
        assert_eq!(a.category, ActionCategory::ScriptContext);
    }
}

#[test]
fn batch24_cross_ai_all_script_context() {
    for a in get_ai_command_bar_actions() {
        assert_eq!(a.category, ActionCategory::ScriptContext);
    }
}

#[test]
fn batch24_cross_notes_all_script_context() {
    let info = NotesInfo {
        has_selection: true,
        is_trash_view: false,
        auto_sizing_enabled: false,
    };
    for a in get_notes_command_bar_actions(&info) {
        assert_eq!(a.category, ActionCategory::ScriptContext);
    }
}

#[test]
fn batch24_cross_path_all_script_context() {
    let p = PathInfo::new("f", "/f", false);
    for a in get_path_context_actions(&p) {
        assert_eq!(a.category, ActionCategory::ScriptContext);
    }
}

#[test]
fn batch24_cross_file_all_script_context() {
    let f = FileInfo {
        path: "/f.txt".to_string(),
        name: "f.txt".to_string(),
        file_type: FileType::File,
        is_dir: false,
    };
    for a in get_file_context_actions(&f) {
        assert_eq!(a.category, ActionCategory::ScriptContext);
    }
}
