//! Batch 38: Dialog builtin action validation tests
//!
//! Focuses on constructor variations, under-tested builder paths, and integration edges:
//! - ScriptInfo constructor variants (with_action_verb, with_all, with_is_script)
//! - Clipboard save_snippet, save_file, and frontmost_app_name dynamic title
//! - Notes duplicate_note, find_in_note, copy_note_as details
//! - AI command bar export_markdown and submit action specifics
//! - Chat context empty models and model ID format
//! - New chat last_used icon BoltFilled, model icon Settings
//! - Note switcher current note "• " prefix and preview trimming
//! - count_section_headers edge cases
//! - WindowPosition enum variants and defaults
//! - ProtocolAction::with_value constructor
//! - score_action with whitespace and special character searches
//! - Cross-context description keyword validation

#[cfg(test)]
mod tests {
    // --- merged from tests_part_01.rs ---
    use crate::actions::builders::*;
    use crate::actions::command_bar::CommandBarConfig;
    use crate::actions::dialog::ActionsDialog;
    use crate::actions::types::{Action, ActionCategory, AnchorPosition, ScriptInfo, SectionStyle};
    use crate::actions::window::{count_section_headers, WindowPosition};
    use crate::clipboard_history::ContentType;
    use crate::designs::icon_variations::IconName;
    use crate::file_search::{FileInfo, FileType};
    use crate::prompts::PathInfo;
    use crate::protocol::ProtocolAction;

    // =========================================================================
    // 1. ScriptInfo::with_action_verb: custom verb in primary action title
    // =========================================================================

    #[test]
    fn script_info_with_action_verb_sets_verb() {
        let info = ScriptInfo::with_action_verb("App Launcher", "/bin/launch", false, "Launch");
        assert_eq!(info.action_verb, "Launch");
    }

    #[test]
    fn script_info_with_action_verb_sets_is_script() {
        let info = ScriptInfo::with_action_verb("App Launcher", "/bin/launch", false, "Launch");
        assert!(!info.is_script);
    }

    #[test]
    fn script_info_with_action_verb_run_script_title_uses_verb() {
        let info = ScriptInfo::with_action_verb("Spotify", "/bin/spotify", false, "Switch to");
        let actions = get_script_context_actions(&info);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert!(run.title.starts_with("Switch to"));
    }

    #[test]
    fn script_info_with_action_verb_desc_uses_verb() {
        let info = ScriptInfo::with_action_verb("Calc", "/bin/calc", true, "Open");
        let actions = get_script_context_actions(&info);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert!(run.description.as_ref().unwrap().contains("Open"));
    }

    // =========================================================================
    // 2. ScriptInfo::with_all: preserves all fields
    // =========================================================================

    #[test]
    fn script_info_with_all_preserves_name_and_path() {
        let info = ScriptInfo::with_all(
            "Test",
            "/test.ts",
            true,
            "Run",
            Some("cmd+t".into()),
            Some("tt".into()),
        );
        assert_eq!(info.name, "Test");
        assert_eq!(info.path, "/test.ts");
    }

    #[test]
    fn script_info_with_all_preserves_shortcut_and_alias() {
        let info = ScriptInfo::with_all(
            "Test",
            "/test.ts",
            true,
            "Run",
            Some("cmd+t".into()),
            Some("tt".into()),
        );
        assert_eq!(info.shortcut, Some("cmd+t".to_string()));
        assert_eq!(info.alias, Some("tt".to_string()));
    }

    #[test]
    fn script_info_with_all_defaults_agent_false() {
        let info = ScriptInfo::with_all("Test", "/test.ts", true, "Run", None, None);
        assert!(!info.is_agent);
    }

    #[test]
    fn script_info_with_all_defaults_suggested_false() {
        let info = ScriptInfo::with_all("Test", "/test.ts", true, "Run", None, None);
        assert!(!info.is_suggested);
        assert!(info.frecency_path.is_none());
    }

    // =========================================================================
    // 3. ScriptInfo::with_is_script: explicit is_script flag
    // =========================================================================

    #[test]
    fn script_info_with_is_script_true_has_edit_actions() {
        let info = ScriptInfo::with_is_script("my-script", "/path.ts", true);
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "edit_script"));
        assert!(actions.iter().any(|a| a.id == "view_logs"));
    }

    #[test]
    fn script_info_with_is_script_false_no_edit_actions() {
        let info = ScriptInfo::with_is_script("built-in", "", false);
        let actions = get_script_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "edit_script"));
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
    }

    #[test]
    fn script_info_with_is_script_defaults_verb_to_run() {
        let info = ScriptInfo::with_is_script("test", "/p", true);
        assert_eq!(info.action_verb, "Run");
    }

    #[test]
    fn script_info_with_is_script_defaults_scriptlet_false() {
        let info = ScriptInfo::with_is_script("test", "/p", true);
        assert!(!info.is_scriptlet);
    }

    // =========================================================================
    // 4. Clipboard: save_snippet and save_file details
    // =========================================================================

    #[test]
    fn clipboard_save_snippet_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let save = actions
            .iter()
            .find(|a| a.id == "clip:clipboard_save_snippet")
            .unwrap();
        assert_eq!(save.shortcut.as_deref(), Some("⇧⌘S"));
    }

    #[test]
    fn clipboard_save_snippet_title() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let save = actions
            .iter()
            .find(|a| a.id == "clip:clipboard_save_snippet")
            .unwrap();
        assert_eq!(save.title, "Save Text as Snippet");
    }

    #[test]
    fn clipboard_save_file_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let save = actions
            .iter()
            .find(|a| a.id == "clip:clipboard_save_file")
            .unwrap();
        assert_eq!(save.shortcut.as_deref(), Some("⌥⇧⌘S"));
    }

    #[test]
    fn clipboard_save_file_desc_mentions_file() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let save = actions
            .iter()
            .find(|a| a.id == "clip:clipboard_save_file")
            .unwrap();
        assert!(save.description.as_ref().unwrap().contains("file"));
    }

    // =========================================================================
    // 5. Clipboard: frontmost_app_name dynamic paste title
    // =========================================================================

    #[test]
    fn clipboard_paste_title_with_app_name() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: Some("Safari".into()),
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let paste = actions.iter().find(|a| a.id == "clip:clipboard_paste").unwrap();
        assert_eq!(paste.title, "Paste to Safari");
    }

    #[test]
    fn clipboard_paste_title_without_app_name() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let paste = actions.iter().find(|a| a.id == "clip:clipboard_paste").unwrap();
        assert_eq!(paste.title, "Paste to Active App");
    }

    #[test]
    fn clipboard_paste_title_with_special_chars_in_app_name() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: Some("Visual Studio Code".into()),
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let paste = actions.iter().find(|a| a.id == "clip:clipboard_paste").unwrap();
        assert_eq!(paste.title, "Paste to Visual Studio Code");
    }

    #[test]
    fn clipboard_paste_shortcut_always_return() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: Some("Terminal".into()),
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let paste = actions.iter().find(|a| a.id == "clip:clipboard_paste").unwrap();
        assert_eq!(paste.shortcut.as_deref(), Some("↵"));
    }

    // =========================================================================
    // 6. Notes: duplicate_note and find_in_note details
    // =========================================================================

    #[test]
    fn notes_duplicate_note_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let dup = actions.iter().find(|a| a.id == "duplicate_note").unwrap();
        assert_eq!(dup.shortcut.as_deref(), Some("⌘D"));
    }

    #[test]
    fn notes_duplicate_note_icon_is_copy() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let dup = actions.iter().find(|a| a.id == "duplicate_note").unwrap();
        assert_eq!(dup.icon, Some(IconName::Copy));
    }

    #[test]
    fn notes_duplicate_note_absent_without_selection() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "duplicate_note"));
    }

    #[test]
    fn notes_duplicate_note_absent_in_trash() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "duplicate_note"));
    }

    // =========================================================================
    // 7. Notes: find_in_note details
    // =========================================================================

    #[test]
    fn notes_find_in_note_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let find = actions.iter().find(|a| a.id == "find_in_note").unwrap();
        assert_eq!(find.shortcut.as_deref(), Some("⌘F"));
    }

    #[test]
    fn notes_find_in_note_icon() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let find = actions.iter().find(|a| a.id == "find_in_note").unwrap();
        assert_eq!(find.icon, Some(IconName::MagnifyingGlass));
    }

    #[test]
    fn notes_find_in_note_section_is_edit() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let find = actions.iter().find(|a| a.id == "find_in_note").unwrap();
        assert_eq!(find.section.as_deref(), Some("Edit"));
    }

    #[test]
    fn notes_find_in_note_absent_without_selection() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "find_in_note"));
    }

    // =========================================================================
    // 8. Notes: copy_note_as details
    // =========================================================================

    #[test]
    fn notes_copy_note_as_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let copy = actions.iter().find(|a| a.id == "copy_note_as").unwrap();
        assert_eq!(copy.shortcut.as_deref(), Some("⇧⌘C"));
    }

    #[test]
    fn notes_copy_note_as_icon_is_copy() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let copy = actions.iter().find(|a| a.id == "copy_note_as").unwrap();
        assert_eq!(copy.icon, Some(IconName::Copy));
    }

    #[test]
    fn notes_copy_note_as_section_is_copy() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let copy = actions.iter().find(|a| a.id == "copy_note_as").unwrap();
        assert_eq!(copy.section.as_deref(), Some("Copy"));
    }

    #[test]
    fn notes_copy_note_as_absent_in_trash() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "copy_note_as"));
    }

    // =========================================================================
    // 9. AI command bar: export_markdown details
    // =========================================================================

    #[test]
    fn ai_bar_export_markdown_shortcut() {
        let actions = get_ai_command_bar_actions();
        let exp = actions.iter().find(|a| a.id == "chat:export_markdown").unwrap();
        assert_eq!(exp.shortcut.as_deref(), Some("⇧⌘E"));
    }

    #[test]
    fn ai_bar_export_markdown_icon() {
        let actions = get_ai_command_bar_actions();
        let exp = actions.iter().find(|a| a.id == "chat:export_markdown").unwrap();
        assert_eq!(exp.icon, Some(IconName::FileCode));
    }

    #[test]
    fn ai_bar_export_markdown_section() {
        let actions = get_ai_command_bar_actions();
        let exp = actions.iter().find(|a| a.id == "chat:export_markdown").unwrap();
        assert_eq!(exp.section.as_deref(), Some("Export"));
    }

    #[test]
    fn ai_bar_export_markdown_desc_mentions_markdown() {
        let actions = get_ai_command_bar_actions();
        let exp = actions.iter().find(|a| a.id == "chat:export_markdown").unwrap();
        assert!(exp.description.as_ref().unwrap().contains("Markdown"));
    }

    // =========================================================================
    // 10. AI command bar: submit action details
    // =========================================================================


    // --- merged from tests_part_02.rs ---
    #[test]
    fn ai_bar_submit_shortcut() {
        let actions = get_ai_command_bar_actions();
        let submit = actions.iter().find(|a| a.id == "chat:submit").unwrap();
        assert_eq!(submit.shortcut.as_deref(), Some("↵"));
    }

    #[test]
    fn ai_bar_submit_icon() {
        let actions = get_ai_command_bar_actions();
        let submit = actions.iter().find(|a| a.id == "chat:submit").unwrap();
        assert_eq!(submit.icon, Some(IconName::ArrowUp));
    }

    #[test]
    fn ai_bar_submit_section_is_actions() {
        let actions = get_ai_command_bar_actions();
        let submit = actions.iter().find(|a| a.id == "chat:submit").unwrap();
        assert_eq!(submit.section.as_deref(), Some("Actions"));
    }

    #[test]
    fn ai_bar_submit_desc_mentions_send() {
        let actions = get_ai_command_bar_actions();
        let submit = actions.iter().find(|a| a.id == "chat:submit").unwrap();
        assert!(submit.description.as_ref().unwrap().contains("Send"));
    }

    // =========================================================================
    // 11. Chat context: empty available_models
    // =========================================================================

    #[test]
    fn chat_empty_models_still_has_continue() {
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
    fn chat_empty_models_no_response_count_is_1() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert_eq!(actions.len(), 1);
    }

    #[test]
    fn chat_model_id_uses_model_id_field() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "claude-3".into(),
                display_name: "Claude 3".into(),
                provider: "Anthropic".into(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "chat:select_model_claude-3"));
    }

    #[test]
    fn chat_current_model_gets_check_mark() {
        let info = ChatPromptInfo {
            current_model: Some("Claude 3".into()),
            available_models: vec![ChatModelInfo {
                id: "claude-3".into(),
                display_name: "Claude 3".into(),
                provider: "Anthropic".into(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model = actions
            .iter()
            .find(|a| a.id == "chat:select_model_claude-3")
            .unwrap();
        assert!(model.title.contains('✓'));
    }

    // =========================================================================
    // 12. New chat: last_used icon is BoltFilled
    // =========================================================================

    #[test]
    fn new_chat_last_used_icon_bolt_filled() {
        let last_used = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "Model 1".into(),
            provider: "p1".into(),
            provider_display_name: "Provider 1".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        let action = actions.iter().find(|a| a.id == "last_used_0").unwrap();
        assert_eq!(action.icon, Some(IconName::BoltFilled));
    }

    #[test]
    fn new_chat_last_used_section() {
        let last_used = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "Model 1".into(),
            provider: "p1".into(),
            provider_display_name: "Provider 1".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        let action = actions.iter().find(|a| a.id == "last_used_0").unwrap();
        assert_eq!(action.section.as_deref(), Some("Last Used Settings"));
    }

    #[test]
    fn new_chat_last_used_desc_is_provider_display() {
        let last_used = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "Model 1".into(),
            provider: "p1".into(),
            provider_display_name: "Anthropic".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        let action = actions.iter().find(|a| a.id == "last_used_0").unwrap();
        assert_eq!(action.description.as_deref(), Some("Anthropic"));
    }

    #[test]
    fn new_chat_model_icon_settings() {
        let models = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "Model 1".into(),
            provider: "p1".into(),
            provider_display_name: "Provider 1".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        let action = actions.iter().find(|a| a.id == "model_0").unwrap();
        assert_eq!(action.icon, Some(IconName::Settings));
    }

    // =========================================================================
    // 13. New chat: preset section and desc
    // =========================================================================

    #[test]
    fn new_chat_preset_section_is_presets() {
        let presets = vec![NewChatPresetInfo {
            id: "general".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        let action = actions.iter().find(|a| a.id == "preset_general").unwrap();
        assert_eq!(action.section.as_deref(), Some("Presets"));
    }

    #[test]
    fn new_chat_preset_desc_is_none() {
        let presets = vec![NewChatPresetInfo {
            id: "general".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        let action = actions.iter().find(|a| a.id == "preset_general").unwrap();
        assert!(action.description.is_none());
    }

    #[test]
    fn new_chat_preset_icon_preserved() {
        let presets = vec![NewChatPresetInfo {
            id: "code".into(),
            name: "Code".into(),
            icon: IconName::Code,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        let action = actions.iter().find(|a| a.id == "preset_code").unwrap();
        assert_eq!(action.icon, Some(IconName::Code));
    }

    #[test]
    fn new_chat_model_section_is_models() {
        let models = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "M1".into(),
            provider: "p".into(),
            provider_display_name: "P".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        let action = actions.iter().find(|a| a.id == "model_0").unwrap();
        assert_eq!(action.section.as_deref(), Some("Models"));
    }

    // =========================================================================
    // 14. Note switcher: current note gets "• " prefix
    // =========================================================================

    #[test]
    fn note_switcher_current_note_has_bullet_prefix() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "My Note".into(),
            char_count: 42,
            is_current: true,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].title, "• My Note");
    }

    #[test]
    fn note_switcher_non_current_note_no_prefix() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "My Note".into(),
            char_count: 42,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].title, "My Note");
    }

    #[test]
    fn note_switcher_current_icon_is_check_when_not_pinned() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "Test".into(),
            char_count: 10,
            is_current: true,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::Check));
    }

    #[test]
    fn note_switcher_pinned_takes_priority_over_current() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "Test".into(),
            char_count: 10,
            is_current: true,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    }

    // =========================================================================
    // 15. Note switcher: preview trimming at 60 chars
    // =========================================================================

    #[test]
    fn note_switcher_preview_exactly_60_not_truncated() {
        let preview: String = "A".repeat(60);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "T".into(),
            char_count: 60,
            is_current: false,
            is_pinned: false,
            preview,
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(!desc.contains('…'));
    }

    #[test]
    fn note_switcher_preview_61_chars_truncated() {
        let preview: String = "B".repeat(61);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "T".into(),
            char_count: 61,
            is_current: false,
            is_pinned: false,
            preview,
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.contains('…'));
    }

    #[test]
    fn note_switcher_preview_with_time_has_separator() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "T".into(),
            char_count: 10,
            is_current: false,
            is_pinned: false,
            preview: "Hello world".into(),
            relative_time: "2m ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.contains(" · "));
    }

    #[test]
    fn note_switcher_no_preview_no_time_shows_chars() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc".into(),
            title: "T".into(),
            char_count: 99,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "99 chars");
    }

    // =========================================================================
    // 16. count_section_headers: edge cases
    // =========================================================================

    #[test]
    fn count_section_headers_empty_filtered() {
        let actions: Vec<Action> = vec![];
        let filtered: Vec<usize> = vec![];
        assert_eq!(count_section_headers(&actions, &filtered), 0);
    }

    #[test]
    fn count_section_headers_no_sections() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext),
            Action::new("b", "B", None, ActionCategory::ScriptContext),
        ];
        let filtered = vec![0, 1];
        assert_eq!(count_section_headers(&actions, &filtered), 0);
    }

    #[test]
    fn count_section_headers_all_same_section() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S"),
        ];
        let filtered = vec![0, 1];
        assert_eq!(count_section_headers(&actions, &filtered), 1);
    }

    #[test]
    fn count_section_headers_different_sections() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("X"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Y"),
            Action::new("c", "C", None, ActionCategory::ScriptContext).with_section("Z"),
        ];
        let filtered = vec![0, 1, 2];
        assert_eq!(count_section_headers(&actions, &filtered), 3);
    }

    // =========================================================================
    // 17. count_section_headers: mixed with and without sections
    // =========================================================================

    #[test]
    fn count_section_headers_mixed_some_none() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S"),
            Action::new("b", "B", None, ActionCategory::ScriptContext), // no section
            Action::new("c", "C", None, ActionCategory::ScriptContext).with_section("S"),
        ];
        let filtered = vec![0, 1, 2];
        // First S = 1 header; no section in b = skip; second S after None = new header
        assert_eq!(count_section_headers(&actions, &filtered), 2);
    }

    #[test]
    fn count_section_headers_filtered_subset() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("X"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Y"),
            Action::new("c", "C", None, ActionCategory::ScriptContext).with_section("X"),
        ];
        // Only index 0 and 2 (same section X, but separated by skipped Y)
        let filtered = vec![0, 2];
        assert_eq!(count_section_headers(&actions, &filtered), 1);
    }

    #[test]
    fn count_section_headers_single_action_with_section() {
        let actions =
            vec![Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S")];
        let filtered = vec![0];
        assert_eq!(count_section_headers(&actions, &filtered), 1);
    }

    #[test]
    fn count_section_headers_single_action_no_section() {
        let actions = vec![Action::new("a", "A", None, ActionCategory::ScriptContext)];
        let filtered = vec![0];
        assert_eq!(count_section_headers(&actions, &filtered), 0);
    }

    // =========================================================================
    // 18. WindowPosition enum variants and Default
    // =========================================================================

    #[test]
    fn window_position_default_is_bottom_right() {
        assert_eq!(WindowPosition::default(), WindowPosition::BottomRight);
    }

    #[test]
    fn window_position_bottom_right_variant_exists() {
        let _pos = WindowPosition::BottomRight;
    }

    #[test]
    fn window_position_top_right_variant_exists() {
        let _pos = WindowPosition::TopRight;
    }

    #[test]
    fn window_position_top_center_variant_exists() {
        let _pos = WindowPosition::TopCenter;
    }

    // =========================================================================
    // 19. ProtocolAction::with_value constructor
    // =========================================================================


    // --- merged from tests_part_03.rs ---
    #[test]
    fn protocol_action_with_value_sets_name() {
        let pa = ProtocolAction::with_value("Test Action".into(), "test-val".into());
        assert_eq!(pa.name, "Test Action");
    }

    #[test]
    fn protocol_action_with_value_sets_value() {
        let pa = ProtocolAction::with_value("Test".into(), "my-value".into());
        assert_eq!(pa.value, Some("my-value".to_string()));
    }

    #[test]
    fn protocol_action_with_value_has_action_false() {
        let pa = ProtocolAction::with_value("Test".into(), "val".into());
        assert!(!pa.has_action);
    }

    #[test]
    fn protocol_action_with_value_defaults_visible_and_close_none() {
        let pa = ProtocolAction::with_value("Test".into(), "val".into());
        assert!(pa.visible.is_none());
        assert!(pa.close.is_none());
        // But is_visible() and should_close() default to true
        assert!(pa.is_visible());
        assert!(pa.should_close());
    }

    // =========================================================================
    // 20. score_action: whitespace and special character searches
    // =========================================================================

    #[test]
    fn score_action_whitespace_search_no_match() {
        let action = Action::new(
            "test",
            "Run Script",
            Some("Execute script".into()),
            ActionCategory::ScriptContext,
        );
        // Whitespace in search - "run script" is lowered and contains space
        let score = ActionsDialog::score_action(&action, "run script");
        // "run script" is a prefix of "run script" (title_lower)
        assert!(score >= 100);
    }

    #[test]
    fn score_action_dash_in_search() {
        let action = Action::new(
            "test",
            "copy-path",
            Some("Copy the path".into()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "copy-");
        assert!(score >= 100); // prefix match
    }

    #[test]
    fn score_action_single_char_search() {
        let action = Action::new("test", "Run", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "r");
        assert!(score >= 100); // "r" is prefix of "run"
    }

    #[test]
    fn score_action_no_match_returns_zero() {
        let action = Action::new("test", "Run Script", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "zzz");
        assert_eq!(score, 0);
    }

    // =========================================================================
    // 21. fuzzy_match: repeated and edge characters
    // =========================================================================

    #[test]
    fn fuzzy_match_repeated_chars_in_needle() {
        // "aab" should match "a_a_b_" since both a's and b are found in order
        assert!(ActionsDialog::fuzzy_match("a_a_b_", "aab"));
    }

    #[test]
    fn fuzzy_match_needle_equals_haystack() {
        assert!(ActionsDialog::fuzzy_match("exact", "exact"));
    }

    #[test]
    fn fuzzy_match_reverse_order_fails() {
        // "ba" should not match "ab" because b comes after a
        assert!(!ActionsDialog::fuzzy_match("ab", "ba"));
    }

    #[test]
    fn fuzzy_match_single_char() {
        assert!(ActionsDialog::fuzzy_match("hello", "h"));
    }

    // =========================================================================
    // 22. to_deeplink_name: numeric and edge cases
    // =========================================================================

    #[test]
    fn to_deeplink_name_numeric_only() {
        assert_eq!(to_deeplink_name("123"), "123");
    }

    #[test]
    fn to_deeplink_name_leading_trailing_spaces() {
        assert_eq!(to_deeplink_name("  hello  "), "hello");
    }

    #[test]
    fn to_deeplink_name_mixed_case() {
        assert_eq!(to_deeplink_name("Hello World"), "hello-world");
    }

    #[test]
    fn to_deeplink_name_underscores_to_hyphens() {
        assert_eq!(to_deeplink_name("my_cool_script"), "my-cool-script");
    }

    // =========================================================================
    // 23. CommandBarConfig: all styles have expected close defaults
    // =========================================================================

    #[test]
    fn command_bar_config_default_close_flags() {
        let cfg = CommandBarConfig::default();
        assert!(cfg.close_on_select);
        assert!(cfg.close_on_click_outside);
        assert!(cfg.close_on_escape);
    }

    #[test]
    fn command_bar_config_ai_style_inherits_close_flags() {
        let cfg = CommandBarConfig::ai_style();
        assert!(cfg.close_on_select);
        assert!(cfg.close_on_escape);
    }

    #[test]
    fn command_bar_config_main_menu_inherits_close_flags() {
        let cfg = CommandBarConfig::main_menu_style();
        assert!(cfg.close_on_select);
        assert!(cfg.close_on_escape);
    }

    #[test]
    fn command_bar_config_notes_inherits_close_flags() {
        let cfg = CommandBarConfig::notes_style();
        assert!(cfg.close_on_select);
        assert!(cfg.close_on_escape);
    }

    // =========================================================================
    // 24. ActionsDialogConfig Default trait
    // =========================================================================

    #[test]
    fn actions_dialog_config_default_search_bottom() {
        let cfg = crate::actions::types::ActionsDialogConfig::default();
        assert_eq!(
            cfg.search_position,
            crate::actions::types::SearchPosition::Bottom
        );
    }

    #[test]
    fn actions_dialog_config_default_section_style_separators() {
        let cfg = crate::actions::types::ActionsDialogConfig::default();
        assert_eq!(cfg.section_style, SectionStyle::Separators);
    }

    #[test]
    fn actions_dialog_config_default_anchor_bottom() {
        let cfg = crate::actions::types::ActionsDialogConfig::default();
        assert_eq!(cfg.anchor, AnchorPosition::Bottom);
    }

    #[test]
    fn actions_dialog_config_default_show_icons_false() {
        let cfg = crate::actions::types::ActionsDialogConfig::default();
        assert!(!cfg.show_icons);
        assert!(!cfg.show_footer);
    }

    // =========================================================================
    // 25. File context: reveal_in_finder always present
    // =========================================================================

    #[cfg(target_os = "macos")]
    #[test]
    fn file_context_reveal_in_finder_present_for_file() {
        let file = FileInfo {
            path: "/test.txt".into(),
            name: "test.txt".into(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        assert!(actions.iter().any(|a| a.id == "file:reveal_in_finder"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn file_context_reveal_in_finder_present_for_dir() {
        let dir = FileInfo {
            path: "/docs".into(),
            name: "docs".into(),
            file_type: FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&dir);
        assert!(actions.iter().any(|a| a.id == "file:reveal_in_finder"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn file_context_reveal_shortcut() {
        let file = FileInfo {
            path: "/test.txt".into(),
            name: "test.txt".into(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        let reveal = actions.iter().find(|a| a.id == "file:reveal_in_finder").unwrap();
        assert_eq!(reveal.shortcut.as_deref(), Some("⌘↵"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn file_context_reveal_desc_mentions_finder() {
        let file = FileInfo {
            path: "/test.txt".into(),
            name: "test.txt".into(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        let reveal = actions.iter().find(|a| a.id == "file:reveal_in_finder").unwrap();
        assert!(reveal.description.as_ref().unwrap().contains("Finder"));
    }

    // =========================================================================
    // 26. Path context: primary action differences for file vs dir
    // =========================================================================

    #[test]
    fn path_context_file_primary_is_select() {
        let info = PathInfo::new("readme.md", "/home/readme.md", false);
        let actions = get_path_context_actions(&info);
        assert_eq!(actions[0].id, "file:select_file");
    }

    #[test]
    fn path_context_dir_primary_is_open_directory() {
        let info = PathInfo::new("docs", "/home/docs", true);
        let actions = get_path_context_actions(&info);
        assert_eq!(actions[0].id, "file:open_directory");
    }

    #[test]
    fn path_context_file_primary_title_quotes_name() {
        let info = PathInfo::new("data.csv", "/home/data.csv", false);
        let actions = get_path_context_actions(&info);
        assert!(actions[0].title.contains("\"data.csv\""));
    }

    #[test]
    fn path_context_dir_primary_title_quotes_name() {
        let info = PathInfo::new("src", "/home/src", true);
        let actions = get_path_context_actions(&info);
        assert!(actions[0].title.contains("\"src\""));
    }

    // =========================================================================
    // 27. Script context: builtin has exactly 4 actions (no shortcut/alias)
    // =========================================================================

    #[test]
    fn builtin_script_action_count_no_extras() {
        let builtin = ScriptInfo::builtin("Clipboard History");
        let actions = get_script_context_actions(&builtin);
        // run_script + add_shortcut + add_alias + copy_deeplink = 4
        assert_eq!(actions.len(), 4);
    }

    #[test]
    fn builtin_script_no_edit_actions() {
        let builtin = ScriptInfo::builtin("App Launcher");
        let actions = get_script_context_actions(&builtin);
        assert!(!actions.iter().any(|a| a.id == "edit_script"));
        assert!(!actions.iter().any(|a| a.id == "edit_scriptlet"));
    }

    #[test]
    fn builtin_script_no_reveal_or_copy_path() {
        let builtin = ScriptInfo::builtin("App Launcher");
        let actions = get_script_context_actions(&builtin);
        assert!(!actions.iter().any(|a| a.id == "file:reveal_in_finder"));
        assert!(!actions.iter().any(|a| a.id == "file:copy_path"));
    }

    #[test]
    fn builtin_script_has_copy_deeplink() {
        let builtin = ScriptInfo::builtin("App Launcher");
        let actions = get_script_context_actions(&builtin);
        assert!(actions.iter().any(|a| a.id == "script:copy_deeplink"));
    }

    // =========================================================================
    // 28. Script context: agent actions include specific IDs
    // =========================================================================

    #[test]
    fn agent_script_has_edit_agent_title() {
        let mut info = ScriptInfo::new("my-agent", "/agents/my-agent.md");
        info.is_script = false;
        info.is_agent = true;
        let actions = get_script_context_actions(&info);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit.title, "Edit Agent");
    }

    #[test]
    fn agent_script_desc_mentions_agent() {
        let mut info = ScriptInfo::new("my-agent", "/agents/my-agent.md");
        info.is_script = false;
        info.is_agent = true;
        let actions = get_script_context_actions(&info);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert!(edit.description.as_ref().unwrap().contains("agent"));
    }

    #[test]
    fn agent_script_has_copy_content() {
        let mut info = ScriptInfo::new("my-agent", "/agents/my-agent.md");
        info.is_script = false;
        info.is_agent = true;
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "copy_content"));
    }

    #[test]
    fn agent_script_no_view_logs() {
        let mut info = ScriptInfo::new("my-agent", "/agents/my-agent.md");
        info.is_script = false;
        info.is_agent = true;
        let actions = get_script_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
    }

    // =========================================================================
    // 29. Scriptlet context: copy_scriptlet_path details
    // =========================================================================

    #[test]
    fn scriptlet_copy_path_id() {
        let info = ScriptInfo::scriptlet("Test", "/path.md#test", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        assert!(actions.iter().any(|a| a.id == "copy_scriptlet_path"));
    }

    #[test]
    fn scriptlet_copy_path_shortcut() {
        let info = ScriptInfo::scriptlet("Test", "/path.md#test", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        let cp = actions
            .iter()
            .find(|a| a.id == "copy_scriptlet_path")
            .unwrap();
        assert_eq!(cp.shortcut.as_deref(), Some("⌘⇧C"));
    }

    #[test]
    fn scriptlet_copy_path_desc_mentions_path() {
        let info = ScriptInfo::scriptlet("Test", "/path.md#test", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        let cp = actions
            .iter()
            .find(|a| a.id == "copy_scriptlet_path")
            .unwrap();
        assert!(cp.description.as_ref().unwrap().contains("path"));
    }

    #[test]
    fn scriptlet_edit_scriptlet_desc_mentions_editor() {
        let info = ScriptInfo::scriptlet("Test", "/path.md#test", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        let edit = actions.iter().find(|a| a.id == "edit_scriptlet").unwrap();
        assert!(edit.description.as_ref().unwrap().contains("$EDITOR"));
    }

    // =========================================================================
    // 30. Cross-context: all action titles are non-empty and IDs are non-empty
    // =========================================================================

    #[test]
    fn cross_context_script_all_titles_and_ids_nonempty() {
        let script = ScriptInfo::new("test", "/test.ts");
        let actions = get_script_context_actions(&script);
        for action in &actions {
            assert!(
                !action.title.is_empty(),
                "Empty title for id: {}",
                action.id
            );
            assert!(!action.id.is_empty(), "Empty id found");
        }
    }

    #[test]
    fn cross_context_clipboard_all_titles_and_ids_nonempty() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        for action in &actions {
            assert!(
                !action.title.is_empty(),
                "Empty title for id: {}",
                action.id
            );
            assert!(!action.id.is_empty(), "Empty id found");
        }
    }

    #[test]
    fn cross_context_ai_bar_all_titles_and_ids_nonempty() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            assert!(
                !action.title.is_empty(),
                "Empty title for id: {}",
                action.id
            );
            assert!(!action.id.is_empty(), "Empty id found");
        }
    }


    // --- merged from tests_part_04.rs ---
    #[test]
    fn cross_context_notes_all_titles_and_ids_nonempty() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        for action in &actions {
            assert!(
                !action.title.is_empty(),
                "Empty title for id: {}",
                action.id
            );
            assert!(!action.id.is_empty(), "Empty id found");
        }
    }

}
