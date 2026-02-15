// =============================================================================
// Dialog Built-in Action Validation Tests — Batch 18
//
// 30 categories of tests validating random built-in actions from dialog windows.
// Each category tests a specific behavior, field, or invariant.
//
// Run with:
//   cargo test --lib actions::dialog_builtin_action_validation_tests_18
// =============================================================================

#[cfg(test)]
mod tests {
    // --- merged from tests_part_01.rs ---
    use super::super::builders::*;
    use super::super::command_bar::CommandBarConfig;
    use super::super::dialog::{
        build_grouped_items_static, coerce_action_selection, ActionsDialog, GroupedActionItem,
    };
    use super::super::types::*;
    use crate::clipboard_history::ContentType;
    use crate::designs::icon_variations::IconName;
    use crate::file_search::FileInfo;
    use crate::prompts::PathInfo;
    use crate::scriptlets::{Scriptlet, ScriptletAction};
    use std::collections::HashSet;

    // =========================================================================
    // Category 01: Agent context is_agent via mutation — action set correctness
    // =========================================================================

    #[test]
    fn cat01_agent_via_mutation_has_edit_script() {
        let mut script = ScriptInfo::new("My Agent", "/path/to/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit.title, "Edit Agent");
    }

    #[test]
    fn cat01_agent_via_mutation_has_reveal_in_finder() {
        let mut script = ScriptInfo::new("My Agent", "/path/to/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        assert!(actions.iter().any(|a| a.id == "file:reveal_in_finder"));
    }

    #[test]
    fn cat01_agent_via_mutation_has_copy_path() {
        let mut script = ScriptInfo::new("My Agent", "/path/to/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        assert!(actions.iter().any(|a| a.id == "file:copy_path"));
    }

    #[test]
    fn cat01_agent_via_mutation_has_copy_content() {
        let mut script = ScriptInfo::new("My Agent", "/path/to/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        assert!(actions.iter().any(|a| a.id == "copy_content"));
    }

    #[test]
    fn cat01_agent_via_mutation_no_view_logs() {
        let mut script = ScriptInfo::new("My Agent", "/path/to/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
    }

    #[test]
    fn cat01_agent_edit_description_mentions_agent() {
        let mut script = ScriptInfo::new("My Agent", "/path/to/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert!(edit.description.as_ref().unwrap().contains("agent"));
    }

    // =========================================================================
    // Category 02: Scriptlet context with_custom — reset_ranking placement
    // =========================================================================

    #[test]
    fn cat02_scriptlet_custom_with_frecency_has_reset_ranking() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None)
            .with_frecency(true, Some("scriptlet:Test".to_string()));
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(actions.iter().any(|a| a.id == "reset_ranking"));
    }

    #[test]
    fn cat02_scriptlet_custom_reset_ranking_is_last() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None)
            .with_frecency(true, Some("scriptlet:Test".to_string()));
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let last = actions.last().unwrap();
        assert_eq!(last.id, "reset_ranking");
    }

    #[test]
    fn cat02_scriptlet_custom_no_frecency_no_reset() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(!actions.iter().any(|a| a.id == "reset_ranking"));
    }

    #[test]
    fn cat02_scriptlet_custom_with_custom_actions_and_frecency() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None)
            .with_frecency(true, Some("scriptlet:Test".to_string()));
        let mut scriptlet =
            Scriptlet::new("Test".to_string(), "bash".to_string(), "echo".to_string());
        scriptlet.actions = vec![ScriptletAction {
            name: "Custom".to_string(),
            command: "custom".to_string(),
            tool: "bash".to_string(),
            code: "echo custom".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];
        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        assert!(actions.iter().any(|a| a.id == "scriptlet_action:custom"));
        assert!(actions.iter().any(|a| a.id == "reset_ranking"));
    }

    // =========================================================================
    // Category 03: Clipboard OCR action details
    // =========================================================================

    #[test]
    fn cat03_ocr_shortcut_is_shift_cmd_c() {
        let entry = ClipboardEntryInfo {
            id: "img1".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "Image".to_string(),
            image_dimensions: Some((800, 600)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ocr = actions.iter().find(|a| a.id == "clip:clipboard_ocr").unwrap();
        assert_eq!(ocr.shortcut.as_ref().unwrap(), "⇧⌘C");
    }

    #[test]
    fn cat03_ocr_title_is_copy_text_from_image() {
        let entry = ClipboardEntryInfo {
            id: "img1".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "Image".to_string(),
            image_dimensions: Some((800, 600)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ocr = actions.iter().find(|a| a.id == "clip:clipboard_ocr").unwrap();
        assert_eq!(ocr.title, "Copy Text from Image");
    }

    #[test]
    fn cat03_ocr_description_mentions_ocr() {
        let entry = ClipboardEntryInfo {
            id: "img1".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "Image".to_string(),
            image_dimensions: Some((800, 600)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ocr = actions.iter().find(|a| a.id == "clip:clipboard_ocr").unwrap();
        assert!(ocr.description.as_ref().unwrap().contains("OCR"));
    }

    #[test]
    fn cat03_text_entry_has_no_ocr() {
        let entry = ClipboardEntryInfo {
            id: "txt1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "Hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(!actions.iter().any(|a| a.id == "clip:clipboard_ocr"));
    }

    // =========================================================================
    // Category 04: Clipboard CleanShot actions (macOS only)
    // =========================================================================

    #[cfg(target_os = "macos")]
    #[test]
    fn cat04_annotate_cleanshot_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "img1".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "Image".to_string(),
            image_dimensions: Some((800, 600)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let annotate = actions
            .iter()
            .find(|a| a.id == "clip:clipboard_annotate_cleanshot")
            .unwrap();
        assert_eq!(annotate.shortcut.as_ref().unwrap(), "⇧⌘A");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat04_upload_cleanshot_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "img1".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "Image".to_string(),
            image_dimensions: Some((800, 600)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let upload = actions
            .iter()
            .find(|a| a.id == "clip:clipboard_upload_cleanshot")
            .unwrap();
        assert_eq!(upload.shortcut.as_ref().unwrap(), "⇧⌘U");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat04_text_entry_no_cleanshot_actions() {
        let entry = ClipboardEntryInfo {
            id: "txt1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "Hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(!actions
            .iter()
            .any(|a| a.id == "clip:clipboard_annotate_cleanshot"));
        assert!(!actions.iter().any(|a| a.id == "clip:clipboard_upload_cleanshot"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat04_image_has_open_with_text_does_not() {
        let img = ClipboardEntryInfo {
            id: "img1".to_string(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "Image".to_string(),
            image_dimensions: Some((800, 600)),
            frontmost_app_name: None,
        };
        let txt = ClipboardEntryInfo {
            id: "txt1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "Hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let img_actions = get_clipboard_history_context_actions(&img);
        let txt_actions = get_clipboard_history_context_actions(&txt);
        assert!(img_actions.iter().any(|a| a.id == "clip:clipboard_open_with"));
        assert!(!txt_actions.iter().any(|a| a.id == "clip:clipboard_open_with"));
    }

    // =========================================================================
    // Category 05: Chat context both flags false — minimal action set
    // =========================================================================

    #[test]
    fn cat05_both_flags_false_zero_models_only_continue() {
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

    #[test]
    fn cat05_both_flags_true_adds_copy_and_clear() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        assert_eq!(actions.len(), 3);
        assert!(actions.iter().any(|a| a.id == "chat:continue_in_chat"));
        assert!(actions.iter().any(|a| a.id == "chat:copy_response"));
        assert!(actions.iter().any(|a| a.id == "chat:clear_conversation"));
    }

    #[test]
    fn cat05_has_response_only_adds_copy_response() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        assert_eq!(actions.len(), 2);
        assert!(actions.iter().any(|a| a.id == "chat:copy_response"));
        assert!(!actions.iter().any(|a| a.id == "chat:clear_conversation"));
    }

    #[test]
    fn cat05_has_messages_only_adds_clear() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert_eq!(actions.len(), 2);
        assert!(actions.iter().any(|a| a.id == "chat:clear_conversation"));
        assert!(!actions.iter().any(|a| a.id == "chat:copy_response"));
    }

    // =========================================================================
    // Category 06: Chat context model checkmark matching
    // =========================================================================

    #[test]
    fn cat06_current_model_gets_checkmark() {
        let info = ChatPromptInfo {
            current_model: Some("Claude 3.5 Sonnet".to_string()),
            available_models: vec![ChatModelInfo {
                id: "claude-3-5-sonnet".to_string(),
                display_name: "Claude 3.5 Sonnet".to_string(),
                provider: "Anthropic".to_string(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model_action = actions
            .iter()
            .find(|a| a.id == "select_model_claude-3-5-sonnet")
            .unwrap();
        assert!(model_action.title.contains('✓'));
    }

    #[test]
    fn cat06_non_current_model_no_checkmark() {
        let info = ChatPromptInfo {
            current_model: Some("GPT-4".to_string()),
            available_models: vec![ChatModelInfo {
                id: "claude-3-5-sonnet".to_string(),
                display_name: "Claude 3.5 Sonnet".to_string(),
                provider: "Anthropic".to_string(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model_action = actions
            .iter()
            .find(|a| a.id == "select_model_claude-3-5-sonnet")
            .unwrap();
        assert!(!model_action.title.contains('✓'));
    }

    #[test]
    fn cat06_model_description_has_via_provider() {
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
        let model_action = actions
            .iter()
            .find(|a| a.id == "select_model_gpt-4")
            .unwrap();
        assert_eq!(model_action.description.as_ref().unwrap(), "via OpenAI");
    }

    #[test]
    fn cat06_multiple_models_ordering_preserved() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![
                ChatModelInfo {
                    id: "m1".to_string(),
                    display_name: "Model A".to_string(),
                    provider: "P1".to_string(),
                },
                ChatModelInfo {
                    id: "m2".to_string(),
                    display_name: "Model B".to_string(),
                    provider: "P2".to_string(),
                },
            ],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let m1_idx = actions
            .iter()
            .position(|a| a.id == "select_model_m1")
            .unwrap();
        let m2_idx = actions
            .iter()
            .position(|a| a.id == "select_model_m2")
            .unwrap();
        assert!(m1_idx < m2_idx);
    }

    // =========================================================================
    // Category 07: New chat action provider_display_name propagation
    // =========================================================================

    #[test]
    fn cat07_last_used_description_uses_provider_display_name() {
        let last_used = vec![NewChatModelInfo {
            model_id: "claude-3".to_string(),
            display_name: "Claude 3".to_string(),
            provider: "anthropic".to_string(),
            provider_display_name: "Anthropic".to_string(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        let lu = actions.iter().find(|a| a.id == "last_used_0").unwrap();
        assert_eq!(lu.description.as_ref().unwrap(), "Anthropic");
    }


    // --- merged from tests_part_02.rs ---
    #[test]
    fn cat07_model_description_uses_provider_display_name() {
        let models = vec![NewChatModelInfo {
            model_id: "gpt-4".to_string(),
            display_name: "GPT-4".to_string(),
            provider: "openai".to_string(),
            provider_display_name: "OpenAI".to_string(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        let m = actions.iter().find(|a| a.id == "model_0").unwrap();
        assert_eq!(m.description.as_ref().unwrap(), "OpenAI");
    }

    #[test]
    fn cat07_preset_has_no_description() {
        let presets = vec![NewChatPresetInfo {
            id: "general".to_string(),
            name: "General".to_string(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        let p = actions.iter().find(|a| a.id == "preset_general").unwrap();
        assert!(p.description.is_none());
    }

    #[test]
    fn cat07_all_sections_present_when_all_inputs_provided() {
        let last_used = vec![NewChatModelInfo {
            model_id: "c3".to_string(),
            display_name: "Claude 3".to_string(),
            provider: "anthropic".to_string(),
            provider_display_name: "Anthropic".to_string(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "code".to_string(),
            name: "Code".to_string(),
            icon: IconName::Code,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "gpt-4".to_string(),
            display_name: "GPT-4".to_string(),
            provider: "openai".to_string(),
            provider_display_name: "OpenAI".to_string(),
        }];
        let actions = get_new_chat_actions(&last_used, &presets, &models);
        let sections: HashSet<_> = actions.iter().filter_map(|a| a.section.as_ref()).collect();
        assert!(sections.contains(&"Last Used Settings".to_string()));
        assert!(sections.contains(&"Presets".to_string()));
        assert!(sections.contains(&"Models".to_string()));
    }

    // =========================================================================
    // Category 08: Note switcher preview boundary — exactly 60 chars
    // =========================================================================

    #[test]
    fn cat08_preview_exactly_60_chars_no_ellipsis() {
        let note = NoteSwitcherNoteInfo {
            id: "n1".to_string(),
            title: "Note".to_string(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: "a".repeat(60),
            relative_time: String::new(),
        };
        let actions = get_note_switcher_actions(&[note]);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(!desc.contains('…'));
    }

    #[test]
    fn cat08_preview_61_chars_has_ellipsis() {
        let note = NoteSwitcherNoteInfo {
            id: "n1".to_string(),
            title: "Note".to_string(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: "a".repeat(61),
            relative_time: String::new(),
        };
        let actions = get_note_switcher_actions(&[note]);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.contains('…'));
    }

    #[test]
    fn cat08_preview_59_chars_no_ellipsis() {
        let note = NoteSwitcherNoteInfo {
            id: "n1".to_string(),
            title: "Note".to_string(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: "a".repeat(59),
            relative_time: String::new(),
        };
        let actions = get_note_switcher_actions(&[note]);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(!desc.contains('…'));
    }

    #[test]
    fn cat08_empty_preview_no_time_uses_char_count() {
        let note = NoteSwitcherNoteInfo {
            id: "n1".to_string(),
            title: "Note".to_string(),
            char_count: 42,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        };
        let actions = get_note_switcher_actions(&[note]);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "42 chars");
    }

    // =========================================================================
    // Category 09: Notes command bar find_in_note details
    // =========================================================================

    #[test]
    fn cat09_find_in_note_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let find = actions.iter().find(|a| a.id == "find_in_note").unwrap();
        assert_eq!(find.shortcut.as_ref().unwrap(), "⌘F");
    }

    #[test]
    fn cat09_find_in_note_icon() {
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
    fn cat09_find_in_note_section() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let find = actions.iter().find(|a| a.id == "find_in_note").unwrap();
        assert_eq!(find.section.as_ref().unwrap(), "Edit");
    }

    #[test]
    fn cat09_find_in_note_absent_in_trash() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "find_in_note"));
    }

    #[test]
    fn cat09_find_in_note_absent_no_selection() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "find_in_note"));
    }

    // =========================================================================
    // Category 10: Notes command bar export details
    // =========================================================================

    #[test]
    fn cat10_export_present_with_selection_no_trash() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions.iter().any(|a| a.id == "export"));
    }

    #[test]
    fn cat10_export_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let export = actions.iter().find(|a| a.id == "export").unwrap();
        assert_eq!(export.shortcut.as_ref().unwrap(), "⇧⌘E");
    }

    #[test]
    fn cat10_export_section_is_export() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let export = actions.iter().find(|a| a.id == "export").unwrap();
        assert_eq!(export.section.as_ref().unwrap(), "Export");
    }

    #[test]
    fn cat10_export_icon() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let export = actions.iter().find(|a| a.id == "export").unwrap();
        assert_eq!(export.icon, Some(IconName::ArrowRight));
    }

    #[test]
    fn cat10_export_absent_in_trash() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "export"));
    }

    // =========================================================================
    // Category 11: Path context open_in_finder description
    // =========================================================================

    #[test]
    fn cat11_open_in_finder_description() {
        let path = PathInfo {
            path: "/Users/test/Documents".to_string(),
            name: "Documents".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&path);
        let action = actions.iter().find(|a| a.id == "file:open_in_finder").unwrap();
        assert_eq!(action.description.as_ref().unwrap(), "Reveal in Finder");
    }

    #[test]
    fn cat11_open_in_finder_shortcut() {
        let path = PathInfo {
            path: "/Users/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path);
        let action = actions.iter().find(|a| a.id == "file:open_in_finder").unwrap();
        assert_eq!(action.shortcut.as_ref().unwrap(), "⌘⇧F");
    }

    #[test]
    fn cat11_open_in_editor_description_mentions_editor() {
        let path = PathInfo {
            path: "/Users/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path);
        let action = actions.iter().find(|a| a.id == "file:open_in_editor").unwrap();
        assert!(action.description.as_ref().unwrap().contains("$EDITOR"));
    }

    #[test]
    fn cat11_open_in_terminal_shortcut() {
        let path = PathInfo {
            path: "/Users/test/Documents".to_string(),
            name: "Documents".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&path);
        let action = actions.iter().find(|a| a.id == "file:open_in_terminal").unwrap();
        assert_eq!(action.shortcut.as_ref().unwrap(), "⌘T");
    }

    // =========================================================================
    // Category 12: File context exact descriptions
    // =========================================================================

    #[test]
    fn cat12_file_open_description() {
        let file = FileInfo {
            path: "/test/doc.pdf".to_string(),
            name: "doc.pdf".to_string(),
            file_type: crate::file_search::FileType::Document,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        let open = actions.iter().find(|a| a.id == "file:open_file").unwrap();
        assert_eq!(
            open.description.as_ref().unwrap(),
            "Open with default application"
        );
    }

    #[test]
    fn cat12_dir_open_description() {
        let dir = FileInfo {
            path: "/test/folder".to_string(),
            name: "folder".to_string(),
            file_type: crate::file_search::FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&dir);
        let open = actions.iter().find(|a| a.id == "file:open_directory").unwrap();
        assert_eq!(open.description.as_ref().unwrap(), "Open this folder");
    }

    #[test]
    fn cat12_reveal_in_finder_description() {
        let file = FileInfo {
            path: "/test/doc.pdf".to_string(),
            name: "doc.pdf".to_string(),
            file_type: crate::file_search::FileType::Document,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        let reveal = actions.iter().find(|a| a.id == "file:reveal_in_finder").unwrap();
        assert_eq!(reveal.description.as_ref().unwrap(), "Reveal in Finder");
    }

    #[test]
    fn cat12_copy_path_description() {
        let file = FileInfo {
            path: "/test/doc.pdf".to_string(),
            name: "doc.pdf".to_string(),
            file_type: crate::file_search::FileType::Document,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        let cp = actions.iter().find(|a| a.id == "file:copy_path").unwrap();
        assert_eq!(
            cp.description.as_ref().unwrap(),
            "Copy the full path to clipboard"
        );
    }

    #[test]
    fn cat12_copy_filename_description() {
        let file = FileInfo {
            path: "/test/doc.pdf".to_string(),
            name: "doc.pdf".to_string(),
            file_type: crate::file_search::FileType::Document,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        let cf = actions.iter().find(|a| a.id == "file:copy_filename").unwrap();
        assert_eq!(
            cf.description.as_ref().unwrap(),
            "Copy just the filename to clipboard"
        );
    }

    // =========================================================================
    // Category 13: format_shortcut_hint edge cases
    // =========================================================================

    #[test]
    fn cat13_control_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("control+c"), "⌃C");
    }

    #[test]
    fn cat13_super_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("super+c"), "⌘C");
    }

    #[test]
    fn cat13_esc_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("esc"), "⎋");
    }

    #[test]
    fn cat13_tab_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("tab"), "⇥");
    }

    #[test]
    fn cat13_backspace_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("backspace"), "⌫");
    }

    #[test]
    fn cat13_delete_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("delete"), "⌫");
    }

    #[test]
    fn cat13_space_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("space"), "␣");
    }

    #[test]
    fn cat13_arrowleft_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("arrowleft"), "←");
    }

    #[test]
    fn cat13_arrowright_key() {
        assert_eq!(ActionsDialog::format_shortcut_hint("arrowright"), "→");
    }

    // =========================================================================
    // Category 14: parse_shortcut_keycaps — all symbol types
    // =========================================================================

    #[test]
    fn cat14_space_symbol() {
        let caps = ActionsDialog::parse_shortcut_keycaps("␣");
        assert_eq!(caps, vec!["␣"]);
    }

    #[test]
    fn cat14_backspace_symbol() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌫");
        assert_eq!(caps, vec!["⌫"]);
    }

    #[test]
    fn cat14_tab_symbol() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⇥");
        assert_eq!(caps, vec!["⇥"]);
    }

    #[test]
    fn cat14_escape_symbol() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⎋");
        assert_eq!(caps, vec!["⎋"]);
    }


    // --- merged from tests_part_03.rs ---
    #[test]
    fn cat14_all_arrows() {
        let caps = ActionsDialog::parse_shortcut_keycaps("↑↓←→");
        assert_eq!(caps, vec!["↑", "↓", "←", "→"]);
    }

    #[test]
    fn cat14_cmd_shift_delete() {
        let caps = ActionsDialog::parse_shortcut_keycaps("⌘⇧⌫");
        assert_eq!(caps, vec!["⌘", "⇧", "⌫"]);
    }

    // =========================================================================
    // Category 15: score_action with empty search string
    // =========================================================================

    #[test]
    fn cat15_empty_search_matches_prefix() {
        let action = Action::new("test", "Test Action", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "");
        // Empty string is a prefix of everything
        assert_eq!(score, 100);
    }

    #[test]
    fn cat15_single_char_search() {
        let action = Action::new("test", "Test Action", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "t");
        assert_eq!(score, 100); // prefix match
    }

    #[test]
    fn cat15_no_match_returns_zero() {
        let action = Action::new("test", "Test Action", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "xyz");
        assert_eq!(score, 0);
    }

    #[test]
    fn cat15_description_bonus_stacking() {
        let action = Action::new(
            "test",
            "Test Action",
            Some("test description".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "test");
        // prefix(100) + description(15) = 115
        assert_eq!(score, 115);
    }

    #[test]
    fn cat15_shortcut_bonus_stacking() {
        let action = Action::new("test", "Test Action", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘T");
        let score = ActionsDialog::score_action(&action, "⌘t");
        // No title match for "⌘t", but shortcut match: 10
        assert_eq!(score, 10);
    }

    // =========================================================================
    // Category 16: fuzzy_match edge cases
    // =========================================================================

    #[test]
    fn cat16_repeated_chars_in_haystack() {
        // "aaa" in "banana" should match (b-a-n-a-n-a has three a's)
        assert!(ActionsDialog::fuzzy_match("banana", "aaa"));
    }

    #[test]
    fn cat16_repeated_chars_insufficient() {
        // "aaaa" in "banana" should fail (only 3 a's available)
        assert!(!ActionsDialog::fuzzy_match("banana", "aaaa"));
    }

    #[test]
    fn cat16_single_char_match() {
        assert!(ActionsDialog::fuzzy_match("hello", "h"));
    }

    #[test]
    fn cat16_single_char_no_match() {
        assert!(!ActionsDialog::fuzzy_match("hello", "z"));
    }

    #[test]
    fn cat16_full_string_match() {
        assert!(ActionsDialog::fuzzy_match("hello", "hello"));
    }

    // =========================================================================
    // Category 17: build_grouped_items_static — section change behavior
    // =========================================================================

    #[test]
    fn cat17_headers_style_adds_header_on_section_change() {
        let actions = vec![
            Action::new("a1", "Action 1", None, ActionCategory::ScriptContext)
                .with_section("Alpha"),
            Action::new("a2", "Action 2", None, ActionCategory::ScriptContext).with_section("Beta"),
        ];
        let filtered: Vec<usize> = (0..actions.len()).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Should be: Header("Alpha"), Item(0), Header("Beta"), Item(1)
        assert_eq!(grouped.len(), 4);
        assert!(matches!(grouped[0], GroupedActionItem::SectionHeader(ref s) if s == "Alpha"));
        assert!(matches!(grouped[1], GroupedActionItem::Item(0)));
        assert!(matches!(grouped[2], GroupedActionItem::SectionHeader(ref s) if s == "Beta"));
        assert!(matches!(grouped[3], GroupedActionItem::Item(1)));
    }

    #[test]
    fn cat17_headers_style_same_section_no_duplicate_header() {
        let actions = vec![
            Action::new("a1", "Action 1", None, ActionCategory::ScriptContext)
                .with_section("Alpha"),
            Action::new("a2", "Action 2", None, ActionCategory::ScriptContext)
                .with_section("Alpha"),
        ];
        let filtered: Vec<usize> = (0..actions.len()).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Should be: Header("Alpha"), Item(0), Item(1)
        assert_eq!(grouped.len(), 3);
        assert!(matches!(grouped[0], GroupedActionItem::SectionHeader(ref s) if s == "Alpha"));
    }

    #[test]
    fn cat17_separators_style_no_headers() {
        let actions = vec![
            Action::new("a1", "Action 1", None, ActionCategory::ScriptContext)
                .with_section("Alpha"),
            Action::new("a2", "Action 2", None, ActionCategory::ScriptContext).with_section("Beta"),
        ];
        let filtered: Vec<usize> = (0..actions.len()).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        // No headers for separators mode
        assert_eq!(grouped.len(), 2);
        assert!(matches!(grouped[0], GroupedActionItem::Item(0)));
        assert!(matches!(grouped[1], GroupedActionItem::Item(1)));
    }

    #[test]
    fn cat17_none_style_no_headers() {
        let actions = vec![
            Action::new("a1", "Action 1", None, ActionCategory::ScriptContext)
                .with_section("Alpha"),
        ];
        let filtered: Vec<usize> = (0..actions.len()).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
        assert_eq!(grouped.len(), 1);
        assert!(matches!(grouped[0], GroupedActionItem::Item(0)));
    }

    #[test]
    fn cat17_empty_filtered_returns_empty() {
        let actions = vec![Action::new(
            "a1",
            "Action 1",
            None,
            ActionCategory::ScriptContext,
        )];
        let filtered: Vec<usize> = vec![];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        assert!(grouped.is_empty());
    }

    // =========================================================================
    // Category 18: coerce_action_selection — consecutive headers
    // =========================================================================

    #[test]
    fn cat18_two_consecutive_headers_then_item() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".to_string()),
            GroupedActionItem::SectionHeader("B".to_string()),
            GroupedActionItem::Item(0),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), Some(2));
    }

    #[test]
    fn cat18_header_at_end_searches_up() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("A".to_string()),
        ];
        assert_eq!(coerce_action_selection(&rows, 1), Some(0));
    }

    #[test]
    fn cat18_single_item_returns_itself() {
        let rows = vec![GroupedActionItem::Item(0)];
        assert_eq!(coerce_action_selection(&rows, 0), Some(0));
    }

    #[test]
    fn cat18_index_beyond_bounds_clamped() {
        let rows = vec![GroupedActionItem::Item(0)];
        assert_eq!(coerce_action_selection(&rows, 99), Some(0));
    }

    #[test]
    fn cat18_all_headers_returns_none() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".to_string()),
            GroupedActionItem::SectionHeader("B".to_string()),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    // =========================================================================
    // Category 19: CommandBarConfig main_menu_style values
    // =========================================================================

    #[test]
    fn cat19_main_menu_search_bottom() {
        let config = CommandBarConfig::main_menu_style();
        assert_eq!(config.dialog_config.search_position, SearchPosition::Bottom);
    }

    #[test]
    fn cat19_main_menu_separators() {
        let config = CommandBarConfig::main_menu_style();
        assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
    }

    #[test]
    fn cat19_main_menu_anchor_bottom() {
        let config = CommandBarConfig::main_menu_style();
        assert_eq!(config.dialog_config.anchor, AnchorPosition::Bottom);
    }

    #[test]
    fn cat19_main_menu_no_icons() {
        let config = CommandBarConfig::main_menu_style();
        assert!(!config.dialog_config.show_icons);
    }

    #[test]
    fn cat19_main_menu_no_footer() {
        let config = CommandBarConfig::main_menu_style();
        assert!(!config.dialog_config.show_footer);
    }

    // =========================================================================
    // Category 20: CommandBarConfig ai_style values
    // =========================================================================

    #[test]
    fn cat20_ai_style_search_top() {
        let config = CommandBarConfig::ai_style();
        assert_eq!(config.dialog_config.search_position, SearchPosition::Top);
    }

    #[test]
    fn cat20_ai_style_headers() {
        let config = CommandBarConfig::ai_style();
        assert_eq!(config.dialog_config.section_style, SectionStyle::Headers);
    }

    #[test]
    fn cat20_ai_style_anchor_top() {
        let config = CommandBarConfig::ai_style();
        assert_eq!(config.dialog_config.anchor, AnchorPosition::Top);
    }

    #[test]
    fn cat20_ai_style_icons_enabled() {
        let config = CommandBarConfig::ai_style();
        assert!(config.dialog_config.show_icons);
    }

    #[test]
    fn cat20_ai_style_footer_enabled() {
        let config = CommandBarConfig::ai_style();
        assert!(config.dialog_config.show_footer);
    }

    // =========================================================================
    // Category 21: CommandBarConfig no_search values
    // =========================================================================

    #[test]
    fn cat21_no_search_hidden() {
        let config = CommandBarConfig::no_search();
        assert_eq!(config.dialog_config.search_position, SearchPosition::Hidden);
    }

    #[test]
    fn cat21_no_search_separators() {
        let config = CommandBarConfig::no_search();
        assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
    }

    #[test]
    fn cat21_no_search_close_defaults_true() {
        let config = CommandBarConfig::no_search();
        assert!(config.close_on_select);
        assert!(config.close_on_click_outside);
        assert!(config.close_on_escape);
    }

    // =========================================================================
    // Category 22: Action with_section sets section field
    // =========================================================================

    #[test]
    fn cat22_with_section_sets_field() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_section("MySection");
        assert_eq!(action.section, Some("MySection".to_string()));
    }

    #[test]
    fn cat22_no_section_by_default() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext);
        assert!(action.section.is_none());
    }

    #[test]
    fn cat22_with_section_preserves_shortcut() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘T")
            .with_section("MySection");
        assert_eq!(action.shortcut, Some("⌘T".to_string()));
        assert_eq!(action.section, Some("MySection".to_string()));
    }

    #[test]
    fn cat22_with_section_preserves_icon() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_icon(IconName::Star)
            .with_section("MySection");
        assert_eq!(action.icon, Some(IconName::Star));
        assert_eq!(action.section, Some("MySection".to_string()));
    }

    // =========================================================================
    // Category 23: ScriptInfo is_agent defaults and combined flags
    // =========================================================================

    #[test]
    fn cat23_new_is_agent_false() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        assert!(!script.is_agent);
    }

    #[test]
    fn cat23_builtin_is_agent_false() {
        let builtin = ScriptInfo::builtin("Clipboard History");
        assert!(!builtin.is_agent);
    }

    #[test]
    fn cat23_scriptlet_is_agent_false() {
        let scriptlet = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        assert!(!scriptlet.is_agent);
    }

    #[test]
    fn cat23_with_all_is_agent_false() {
        let script = ScriptInfo::with_all("Test", "/path", true, "Run", None, None);
        assert!(!script.is_agent);
    }

    #[test]
    fn cat23_agent_mutually_exclusive_with_script() {
        let mut script = ScriptInfo::new("Test", "/path");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        // Agent should NOT have view_logs (script-only)
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
        // Agent SHOULD have edit_script (with "Edit Agent" title)
        assert!(actions.iter().any(|a| a.id == "edit_script"));
    }

    // =========================================================================
    // Category 24: Clipboard save_snippet and save_file shortcuts
    // =========================================================================

    #[test]
    fn cat24_save_snippet_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "txt1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "Hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let snippet = actions
            .iter()
            .find(|a| a.id == "clip:clipboard_save_snippet")
            .unwrap();
        assert_eq!(snippet.shortcut.as_ref().unwrap(), "⇧⌘S");
    }

    #[test]
    fn cat24_save_file_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "txt1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "Hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let save = actions
            .iter()
            .find(|a| a.id == "clip:clipboard_save_file")
            .unwrap();
        assert_eq!(save.shortcut.as_ref().unwrap(), "⌥⇧⌘S");
    }

    #[test]
    fn cat24_save_snippet_title() {
        let entry = ClipboardEntryInfo {
            id: "txt1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "Hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let snippet = actions
            .iter()
            .find(|a| a.id == "clip:clipboard_save_snippet")
            .unwrap();
        assert_eq!(snippet.title, "Save Text as Snippet");
    }


    // --- merged from tests_part_04.rs ---
    #[test]
    fn cat24_save_file_title() {
        let entry = ClipboardEntryInfo {
            id: "txt1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "Hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let save = actions
            .iter()
            .find(|a| a.id == "clip:clipboard_save_file")
            .unwrap();
        assert_eq!(save.title, "Save as File...");
    }

    // =========================================================================
    // Category 25: Clipboard delete actions shortcuts
    // =========================================================================

    #[test]
    fn cat25_delete_entry_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "txt1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "Hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let del = actions.iter().find(|a| a.id == "clip:clipboard_delete").unwrap();
        assert_eq!(del.shortcut.as_ref().unwrap(), "⌃X");
    }

    #[test]
    fn cat25_delete_multiple_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "txt1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "Hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let del = actions
            .iter()
            .find(|a| a.id == "clip:clipboard_delete_multiple")
            .unwrap();
        assert_eq!(del.shortcut.as_ref().unwrap(), "⇧⌘X");
    }

    #[test]
    fn cat25_delete_all_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "txt1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "Hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let del = actions
            .iter()
            .find(|a| a.id == "clip:clipboard_delete_all")
            .unwrap();
        assert_eq!(del.shortcut.as_ref().unwrap(), "⌃⇧X");
    }

    #[test]
    fn cat25_delete_all_description_mentions_pinned() {
        let entry = ClipboardEntryInfo {
            id: "txt1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "Hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let del = actions
            .iter()
            .find(|a| a.id == "clip:clipboard_delete_all")
            .unwrap();
        assert!(del.description.as_ref().unwrap().contains("pinned"));
    }

    // =========================================================================
    // Category 26: to_deeplink_name edge cases
    // =========================================================================

    #[test]
    fn cat26_numbers_only() {
        assert_eq!(to_deeplink_name("12345"), "12345");
    }

    #[test]
    fn cat26_all_special_chars_becomes_empty() {
        assert_eq!(to_deeplink_name("!@#$%^&*()"), "");
    }

    #[test]
    fn cat26_mixed_case_lowered() {
        assert_eq!(to_deeplink_name("CamelCase"), "camelcase");
    }

    #[test]
    fn cat26_consecutive_specials_collapsed() {
        assert_eq!(to_deeplink_name("hello---world"), "hello-world");
    }

    #[test]
    fn cat26_underscores_become_hyphens() {
        assert_eq!(to_deeplink_name("hello_world"), "hello-world");
    }

    #[test]
    fn cat26_leading_trailing_specials_stripped() {
        assert_eq!(to_deeplink_name("--hello--"), "hello");
    }

    #[test]
    fn cat26_unicode_preserved() {
        // CJK characters are alphanumeric in Unicode
        assert_eq!(to_deeplink_name("日本語"), "日本語");
    }

    // =========================================================================
    // Category 27: AI command bar per-section action counts
    // =========================================================================

    #[test]
    fn cat27_response_section_has_3_actions() {
        let actions = get_ai_command_bar_actions();
        let response_count = actions
            .iter()
            .filter(|a| a.section.as_ref() == Some(&"Response".to_string()))
            .count();
        assert_eq!(response_count, 3);
    }

    #[test]
    fn cat27_actions_section_has_4_actions() {
        let actions = get_ai_command_bar_actions();
        let actions_count = actions
            .iter()
            .filter(|a| a.section.as_ref() == Some(&"Actions".to_string()))
            .count();
        assert_eq!(actions_count, 4);
    }

    #[test]
    fn cat27_attachments_section_has_2_actions() {
        let actions = get_ai_command_bar_actions();
        let count = actions
            .iter()
            .filter(|a| a.section.as_ref() == Some(&"Attachments".to_string()))
            .count();
        assert_eq!(count, 2);
    }

    #[test]
    fn cat27_export_section_has_1_action() {
        let actions = get_ai_command_bar_actions();
        let count = actions
            .iter()
            .filter(|a| a.section.as_ref() == Some(&"Export".to_string()))
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn cat27_help_section_has_1_action() {
        let actions = get_ai_command_bar_actions();
        let count = actions
            .iter()
            .filter(|a| a.section.as_ref() == Some(&"Help".to_string()))
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn cat27_settings_section_has_1_action() {
        let actions = get_ai_command_bar_actions();
        let count = actions
            .iter()
            .filter(|a| a.section.as_ref() == Some(&"Settings".to_string()))
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn cat27_total_ai_actions_is_12() {
        let actions = get_ai_command_bar_actions();
        assert_eq!(actions.len(), 12);
    }

    // =========================================================================
    // Category 28: Notes command bar all flag combinations
    // =========================================================================

    #[test]
    fn cat28_full_feature_count() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        // Notes: new_note, duplicate_note, browse_notes
        // Edit: find_in_note, format
        // Copy: copy_note_as, copy_deeplink, create_quicklink
        // Export: export
        // Settings: enable_auto_sizing
        assert_eq!(actions.len(), 10);
    }

    #[test]
    fn cat28_auto_sizing_enabled_hides_setting() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert_eq!(actions.len(), 9);
        assert!(!actions.iter().any(|a| a.id == "enable_auto_sizing"));
    }

    #[test]
    fn cat28_trash_view_minimal() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        // Only new_note, browse_notes, enable_auto_sizing
        assert_eq!(actions.len(), 3);
    }

    #[test]
    fn cat28_no_selection_minimal() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        // Only new_note, browse_notes, enable_auto_sizing
        assert_eq!(actions.len(), 3);
    }

    #[test]
    fn cat28_trash_no_selection_auto_sizing() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: true,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        // Only new_note, browse_notes (auto_sizing hidden)
        assert_eq!(actions.len(), 2);
    }

    // =========================================================================
    // Category 29: Path context move_to_trash description formatting
    // =========================================================================

    #[test]
    fn cat29_trash_dir_description_says_folder() {
        let path = PathInfo {
            path: "/Users/test/Documents".to_string(),
            name: "Documents".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&path);
        let trash = actions.iter().find(|a| a.id == "file:move_to_trash").unwrap();
        assert!(trash.description.as_ref().unwrap().contains("folder"));
    }

    #[test]
    fn cat29_trash_file_description_says_file() {
        let path = PathInfo {
            path: "/Users/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path);
        let trash = actions.iter().find(|a| a.id == "file:move_to_trash").unwrap();
        assert!(trash.description.as_ref().unwrap().contains("file"));
    }

    #[test]
    fn cat29_trash_shortcut() {
        let path = PathInfo {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path);
        let trash = actions.iter().find(|a| a.id == "file:move_to_trash").unwrap();
        assert_eq!(trash.shortcut.as_ref().unwrap(), "⌘⌫");
    }

    #[test]
    fn cat29_trash_always_last() {
        let path = PathInfo {
            path: "/test/Documents".to_string(),
            name: "Documents".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&path);
        let last = actions.last().unwrap();
        assert_eq!(last.id, "file:move_to_trash");
    }

    // =========================================================================
    // Category 30: Cross-context all actions have non-empty descriptions
    // =========================================================================

    #[test]
    fn cat30_script_actions_all_have_descriptions() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        for action in &actions {
            assert!(
                action.description.is_some(),
                "Script action '{}' should have description",
                action.id
            );
        }
    }

    #[test]
    fn cat30_clipboard_text_actions_all_have_descriptions() {
        let entry = ClipboardEntryInfo {
            id: "txt1".to_string(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "Hello".to_string(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        for action in &actions {
            assert!(
                action.description.is_some(),
                "Clipboard action '{}' should have description",
                action.id
            );
        }
    }

    #[test]
    fn cat30_ai_actions_all_have_descriptions() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            assert!(
                action.description.is_some(),
                "AI action '{}' should have description",
                action.id
            );
        }
    }

    #[test]
    fn cat30_path_actions_all_have_descriptions() {
        let path = PathInfo {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path);
        for action in &actions {
            assert!(
                action.description.is_some(),
                "Path action '{}' should have description",
                action.id
            );
        }
    }

    #[test]
    fn cat30_file_actions_all_have_descriptions() {
        let file = FileInfo {
            path: "/test/doc.pdf".to_string(),
            name: "doc.pdf".to_string(),
            file_type: crate::file_search::FileType::Document,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        for action in &actions {
            assert!(
                action.description.is_some(),
                "File action '{}' should have description",
                action.id
            );
        }
    }

    #[test]
    fn cat30_notes_actions_all_have_descriptions() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        for action in &actions {
            assert!(
                action.description.is_some(),
                "Notes action '{}' should have description",
                action.id
            );
        }
    }

}
