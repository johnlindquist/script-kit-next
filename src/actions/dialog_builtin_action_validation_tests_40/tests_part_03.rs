    #[test]
    fn scriptlet_edit_scriptlet_shortcut() {
        let script = ScriptInfo::scriptlet("Test", "/path", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let edit = actions.iter().find(|a| a.id == "edit_scriptlet").unwrap();
        assert_eq!(edit.shortcut, Some("⌘E".to_string()));
    }

    // =========================================================================
    // 18. AI bar: toggle_shortcuts_help details
    // =========================================================================

    #[test]
    fn ai_bar_toggle_shortcuts_help_shortcut() {
        let actions = get_ai_command_bar_actions();
        let help = actions
            .iter()
            .find(|a| a.id == "toggle_shortcuts_help")
            .unwrap();
        assert_eq!(help.shortcut, Some("⌘/".to_string()));
    }

    #[test]
    fn ai_bar_toggle_shortcuts_help_icon() {
        let actions = get_ai_command_bar_actions();
        let help = actions
            .iter()
            .find(|a| a.id == "toggle_shortcuts_help")
            .unwrap();
        assert_eq!(help.icon, Some(IconName::Star));
    }

    #[test]
    fn ai_bar_toggle_shortcuts_help_section() {
        let actions = get_ai_command_bar_actions();
        let help = actions
            .iter()
            .find(|a| a.id == "toggle_shortcuts_help")
            .unwrap();
        assert_eq!(help.section, Some("Help".to_string()));
    }

    #[test]
    fn ai_bar_toggle_shortcuts_help_title() {
        let actions = get_ai_command_bar_actions();
        let help = actions
            .iter()
            .find(|a| a.id == "toggle_shortcuts_help")
            .unwrap();
        assert_eq!(help.title, "Keyboard Shortcuts");
    }

    // =========================================================================
    // 19. AI bar: change_model has no shortcut
    // =========================================================================

    #[test]
    fn ai_bar_change_model_no_shortcut() {
        let actions = get_ai_command_bar_actions();
        let model = actions.iter().find(|a| a.id == "change_model").unwrap();
        assert!(model.shortcut.is_none());
    }

    #[test]
    fn ai_bar_change_model_icon_settings() {
        let actions = get_ai_command_bar_actions();
        let model = actions.iter().find(|a| a.id == "change_model").unwrap();
        assert_eq!(model.icon, Some(IconName::Settings));
    }

    #[test]
    fn ai_bar_change_model_section_settings() {
        let actions = get_ai_command_bar_actions();
        let model = actions.iter().find(|a| a.id == "change_model").unwrap();
        assert_eq!(model.section, Some("Settings".to_string()));
    }

    #[test]
    fn ai_bar_branch_from_last_no_shortcut() {
        let actions = get_ai_command_bar_actions();
        let branch = actions.iter().find(|a| a.id == "branch_from_last").unwrap();
        assert!(branch.shortcut.is_none());
    }

    // =========================================================================
    // 20. AI bar: unique IDs across all 12 actions
    // =========================================================================

    #[test]
    fn ai_bar_has_12_actions() {
        let actions = get_ai_command_bar_actions();
        assert_eq!(actions.len(), 12);
    }

    #[test]
    fn ai_bar_all_ids_unique() {
        let actions = get_ai_command_bar_actions();
        let mut ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        let original_len = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), original_len);
    }

    #[test]
    fn ai_bar_all_have_icons() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            assert!(
                action.icon.is_some(),
                "AI bar action '{}' has no icon",
                action.id
            );
        }
    }

    #[test]
    fn ai_bar_all_have_sections() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            assert!(
                action.section.is_some(),
                "AI bar action '{}' has no section",
                action.id
            );
        }
    }

    // =========================================================================
    // 21. Notes: export action requires selection and not trash
    // =========================================================================

    #[test]
    fn notes_export_present_with_selection_no_trash() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions.iter().any(|a| a.id == "export"));
    }

    #[test]
    fn notes_export_absent_without_selection() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "export"));
    }

    #[test]
    fn notes_export_absent_in_trash() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "export"));
    }

    #[test]
    fn notes_export_shortcut_and_section() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let export = actions.iter().find(|a| a.id == "export").unwrap();
        assert_eq!(export.shortcut, Some("⇧⌘E".to_string()));
        assert_eq!(export.section, Some("Export".to_string()));
    }

    // =========================================================================
    // 22. Notes: browse_notes always present
    // =========================================================================

    #[test]
    fn notes_browse_notes_present_no_selection() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions.iter().any(|a| a.id == "browse_notes"));
    }

    #[test]
    fn notes_browse_notes_present_with_selection() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions.iter().any(|a| a.id == "browse_notes"));
    }

    #[test]
    fn notes_browse_notes_present_in_trash() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions.iter().any(|a| a.id == "browse_notes"));
    }

    #[test]
    fn notes_browse_notes_shortcut_and_icon() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let browse = actions.iter().find(|a| a.id == "browse_notes").unwrap();
        assert_eq!(browse.shortcut, Some("⌘P".to_string()));
        assert_eq!(browse.icon, Some(IconName::FolderOpen));
    }

    // =========================================================================
    // 23. Chat context: copy_response only when has_response
    // =========================================================================

    #[test]
    fn chat_copy_response_present_when_has_response() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".into()),
            available_models: vec![],
            has_messages: true,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "copy_response"));
    }

    #[test]
    fn chat_copy_response_absent_when_no_response() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".into()),
            available_models: vec![],
            has_messages: true,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "copy_response"));
    }

    #[test]
    fn chat_copy_response_shortcut() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        let copy = actions.iter().find(|a| a.id == "copy_response").unwrap();
        assert_eq!(copy.shortcut, Some("⌘C".to_string()));
    }

    #[test]
    fn chat_copy_response_title() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        let copy = actions.iter().find(|a| a.id == "copy_response").unwrap();
        assert_eq!(copy.title, "Copy Last Response");
    }

    // =========================================================================
    // 24. Chat context: clear_conversation only when has_messages
    // =========================================================================

    #[test]
    fn chat_clear_present_when_has_messages() {
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
    fn chat_clear_absent_when_no_messages() {
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
    fn chat_clear_shortcut() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let clear = actions
            .iter()
            .find(|a| a.id == "clear_conversation")
            .unwrap();
        assert_eq!(clear.shortcut, Some("⌘⌫".to_string()));
    }

    #[test]
    fn chat_continue_in_chat_always_present() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "continue_in_chat"));
    }

    // =========================================================================
    // 25. New chat: empty lists produce zero actions
    // =========================================================================

    #[test]
    fn new_chat_empty_inputs_zero_actions() {
        let actions = get_new_chat_actions(&[], &[], &[]);
        assert_eq!(actions.len(), 0);
    }

    #[test]
    fn new_chat_only_last_used_produces_correct_count() {
        let last_used = vec![NewChatModelInfo {
            model_id: "claude".into(),
            display_name: "Claude".into(),
            provider: "anthropic".into(),
            provider_display_name: "Anthropic".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions.len(), 1);
    }

    #[test]
    fn new_chat_only_models_produces_correct_count() {
        let models = vec![
            NewChatModelInfo {
                model_id: "gpt4".into(),
                display_name: "GPT-4".into(),
                provider: "openai".into(),
                provider_display_name: "OpenAI".into(),
            },
            NewChatModelInfo {
                model_id: "claude".into(),
                display_name: "Claude".into(),
                provider: "anthropic".into(),
                provider_display_name: "Anthropic".into(),
            },
        ];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions.len(), 2);
    }

    #[test]
    fn new_chat_all_three_sections_total() {
        let last_used = vec![NewChatModelInfo {
            model_id: "claude".into(),
            display_name: "Claude".into(),
            provider: "anthropic".into(),
            provider_display_name: "Anthropic".into(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "general".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "gpt4".into(),
            display_name: "GPT-4".into(),
            provider: "openai".into(),
            provider_display_name: "OpenAI".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &presets, &models);
        assert_eq!(actions.len(), 3);
    }

    // =========================================================================
    // 26. New chat: preset IDs use preset_{id} format
    // =========================================================================

    #[test]
    fn new_chat_preset_id_format() {
        let presets = vec![NewChatPresetInfo {
            id: "code-review".into(),
            name: "Code Review".into(),
            icon: IconName::Code,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(actions[0].id, "preset_code-review");
    }

    #[test]
    fn new_chat_preset_section_is_presets() {
        let presets = vec![NewChatPresetInfo {
            id: "general".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(actions[0].section, Some("Presets".to_string()));
    }

    #[test]
    fn new_chat_preset_icon_preserved() {
        let presets = vec![NewChatPresetInfo {
            id: "code".into(),
            name: "Code".into(),
            icon: IconName::Code,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(actions[0].icon, Some(IconName::Code));
    }

