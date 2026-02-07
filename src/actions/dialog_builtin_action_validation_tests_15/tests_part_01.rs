    use super::super::builders::*;
    use super::super::command_bar::CommandBarConfig;
    use super::super::types::*;
    use crate::clipboard_history::ContentType;
    use crate::designs::icon_variations::IconName;
    use crate::file_search::FileInfo;
    use crate::prompts::PathInfo;
    use std::collections::HashSet;

    fn action_ids(actions: &[Action]) -> Vec<String> {
        actions.iter().map(|a| a.id.clone()).collect()
    }

    // =========================================================================
    // cat01: to_deeplink_name with non-Latin Unicode scripts
    // =========================================================================

    #[test]
    fn cat01_deeplink_arabic_preserved() {
        // Arabic alphanumeric chars should pass is_alphanumeric()
        let result = to_deeplink_name("مرحبا");
        assert!(!result.is_empty(), "Arabic should be preserved");
        assert!(
            result.chars().all(|c| c.is_alphanumeric() || c == '-'),
            "Result should only have alphanumeric or hyphens: {}",
            result
        );
    }

    #[test]
    fn cat01_deeplink_thai_preserved() {
        let result = to_deeplink_name("สวัสดี");
        assert!(!result.is_empty(), "Thai should be preserved");
    }

    #[test]
    fn cat01_deeplink_devanagari_preserved() {
        let result = to_deeplink_name("नमस्ते");
        assert!(!result.is_empty(), "Devanagari should be preserved");
    }

    #[test]
    fn cat01_deeplink_mixed_scripts() {
        // "Hello-مرحبا" — mixed Latin and Arabic
        let result = to_deeplink_name("Hello مرحبا");
        assert!(result.contains("hello"), "Latin part lowercased");
        // Arabic and Latin separated by space → hyphen
        assert!(result.contains('-'), "Space becomes hyphen");
    }

    #[test]
    fn cat01_deeplink_empty_string() {
        assert_eq!(to_deeplink_name(""), "");
    }

    #[test]
    fn cat01_deeplink_only_specials() {
        assert_eq!(to_deeplink_name("!@#$%^&*()"), "");
    }

    #[test]
    fn cat01_deeplink_single_char() {
        assert_eq!(to_deeplink_name("a"), "a");
    }

    // =========================================================================
    // cat02: Clipboard image macOS-exclusive action set
    // =========================================================================

    #[cfg(target_os = "macos")]
    #[test]
    fn cat02_clipboard_image_macos_has_open_with() {
        let entry = ClipboardEntryInfo {
            id: "img-1".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"clipboard_open_with".to_string()));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat02_clipboard_image_macos_has_annotate_cleanshot() {
        let entry = ClipboardEntryInfo {
            id: "img-2".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: Some((200, 200)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"clipboard_annotate_cleanshot".to_string()));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat02_clipboard_image_macos_has_upload_cleanshot() {
        let entry = ClipboardEntryInfo {
            id: "img-3".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"clipboard_upload_cleanshot".to_string()));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat02_clipboard_text_macos_no_annotate() {
        let entry = ClipboardEntryInfo {
            id: "txt-1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hello".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ids = action_ids(&actions);
        assert!(!ids.contains(&"clipboard_annotate_cleanshot".to_string()));
        assert!(!ids.contains(&"clipboard_upload_cleanshot".to_string()));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat02_clipboard_image_has_ocr_text_does_not() {
        let img = ClipboardEntryInfo {
            id: "i1".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let txt = ClipboardEntryInfo {
            id: "t1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let img_ids = action_ids(&get_clipboard_history_context_actions(&img));
        let txt_ids = action_ids(&get_clipboard_history_context_actions(&txt));
        assert!(img_ids.contains(&"clipboard_ocr".to_string()));
        assert!(!txt_ids.contains(&"clipboard_ocr".to_string()));
    }

    // =========================================================================
    // cat03: Notes combined-flag interactions
    // =========================================================================

    #[test]
    fn cat03_notes_all_true_trash_blocks_selection() {
        // has_selection=true + is_trash_view=true → selection-dependent actions hidden
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        // Trash view blocks: duplicate, find, format, copy_note_as, copy_deeplink, create_quicklink, export
        assert!(!ids.contains(&"duplicate_note".to_string()));
        assert!(!ids.contains(&"find_in_note".to_string()));
        assert!(!ids.contains(&"export".to_string()));
    }

    #[test]
    fn cat03_notes_no_selection_no_trash() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        // Always present: new_note, browse_notes
        assert!(ids.contains(&"new_note".to_string()));
        assert!(ids.contains(&"browse_notes".to_string()));
        // No selection → no duplicate, find, format, etc.
        assert!(!ids.contains(&"duplicate_note".to_string()));
        // auto_sizing disabled → enable_auto_sizing present
        assert!(ids.contains(&"enable_auto_sizing".to_string()));
    }

    #[test]
    fn cat03_notes_full_feature_set() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        // Should have max actions: new_note, duplicate, browse, find, format,
        // copy_note_as, copy_deeplink, create_quicklink, export, enable_auto_sizing
        assert_eq!(actions.len(), 10, "Full feature set should be 10 actions");
    }

    #[test]
    fn cat03_notes_auto_sizing_enabled_hides_action() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        assert!(!ids.contains(&"enable_auto_sizing".to_string()));
        // Full set minus enable_auto_sizing = 9
        assert_eq!(actions.len(), 9);
    }

    #[test]
    fn cat03_notes_trash_view_minimal() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: true,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        // Only new_note and browse_notes (auto_sizing enabled, so no enable_auto_sizing)
        assert_eq!(actions.len(), 2);
    }

    // =========================================================================
    // cat04: Chat context boundary states
    // =========================================================================

    #[test]
    fn cat04_chat_zero_models_both_flags_false() {
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
    fn cat04_chat_zero_models_both_flags_true() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        // continue_in_chat + copy_response + clear_conversation = 3
        assert_eq!(actions.len(), 3);
    }

    #[test]
    fn cat04_chat_model_id_format() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "gpt-4o".into(),
                display_name: "GPT-4o".into(),
                provider: "OpenAI".into(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert_eq!(actions[0].id, "select_model_gpt-4o");
    }

    #[test]
    fn cat04_chat_current_model_checkmark() {
        let info = ChatPromptInfo {
            current_model: Some("Claude 3.5".into()),
            available_models: vec![
                ChatModelInfo {
                    id: "claude-35".into(),
                    display_name: "Claude 3.5".into(),
                    provider: "Anthropic".into(),
                },
                ChatModelInfo {
                    id: "gpt-4".into(),
                    display_name: "GPT-4".into(),
                    provider: "OpenAI".into(),
                },
            ],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(
            actions[0].title.contains("✓"),
            "Current model gets checkmark"
        );
        assert!(
            !actions[1].title.contains("✓"),
            "Non-current model no checkmark"
        );
    }

    #[test]
    fn cat04_chat_continue_shortcut() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert_eq!(actions[0].shortcut.as_deref(), Some("⌘↵"));
    }

    #[test]
    fn cat04_chat_model_description_via_provider() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "m1".into(),
                display_name: "Model One".into(),
                provider: "Acme Corp".into(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert_eq!(actions[0].description.as_deref(), Some("via Acme Corp"));
    }

    // =========================================================================
    // cat05: New chat section ordering guarantees
    // =========================================================================

    #[test]
    fn cat05_new_chat_empty_all_sections() {
        let actions = get_new_chat_actions(&[], &[], &[]);
        assert!(actions.is_empty());
    }

    #[test]
    fn cat05_new_chat_section_ordering() {
        let last_used = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "Model 1".into(),
            provider: "P".into(),
            provider_display_name: "Provider".into(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "general".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "m2".into(),
            display_name: "Model 2".into(),
            provider: "P".into(),
            provider_display_name: "Provider".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &presets, &models);
        assert_eq!(actions.len(), 3);
        assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
        assert_eq!(actions[1].section.as_deref(), Some("Presets"));
        assert_eq!(actions[2].section.as_deref(), Some("Models"));
    }

    #[test]
    fn cat05_new_chat_preset_no_description() {
        let presets = vec![NewChatPresetInfo {
            id: "code".into(),
            name: "Code".into(),
            icon: IconName::Code,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(actions[0].description, None);
    }

    #[test]
    fn cat05_new_chat_model_has_provider_description() {
        let models = vec![NewChatModelInfo {
            model_id: "gpt4".into(),
            display_name: "GPT-4".into(),
            provider: "openai".into(),
            provider_display_name: "OpenAI".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].description.as_deref(), Some("OpenAI"));
    }

    #[test]
    fn cat05_new_chat_last_used_icon_bolt() {
        let last_used = vec![NewChatModelInfo {
            model_id: "x".into(),
            display_name: "X".into(),
            provider: "P".into(),
            provider_display_name: "PP".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
    }

    #[test]
    fn cat05_new_chat_model_icon_settings() {
        let models = vec![NewChatModelInfo {
            model_id: "m".into(),
            display_name: "M".into(),
            provider: "P".into(),
            provider_display_name: "PP".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].icon, Some(IconName::Settings));
    }

    // =========================================================================
    // cat06: Note switcher description fallback hierarchy
    // =========================================================================

    #[test]
    fn cat06_note_switcher_preview_and_time() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n1".into(),
            title: "Test".into(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: "Hello world".into(),
            relative_time: "5m ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_deref().unwrap();
        assert!(desc.contains("Hello world"));
        assert!(desc.contains("5m ago"));
        assert!(desc.contains("·"), "Separator between preview and time");
    }

