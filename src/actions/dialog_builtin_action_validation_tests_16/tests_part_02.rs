    #[test]
    fn cat05_file_copy_filename_shortcut_cmd_c() {
        let file = FileInfo {
            path: "/doc.pdf".into(),
            name: "doc.pdf".into(),
            file_type: crate::file_search::FileType::Document,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        let cf = actions.iter().find(|a| a.id == "copy_filename").unwrap();
        assert_eq!(cf.shortcut.as_deref(), Some("⌘C"));
    }

    // =========================================================================
    // cat06: Notes command bar section count per flag combination
    // =========================================================================

    #[test]
    fn cat06_notes_full_feature_section_count() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let sections: HashSet<&str> = actions
            .iter()
            .filter_map(|a| a.section.as_deref())
            .collect();
        // Notes, Edit, Copy, Export, Settings
        assert_eq!(
            sections.len(),
            5,
            "full feature has 5 sections: {:?}",
            sections
        );
    }

    #[test]
    fn cat06_notes_trash_view_minimal_sections() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let sections: HashSet<&str> = actions
            .iter()
            .filter_map(|a| a.section.as_deref())
            .collect();
        // Only Notes and Settings
        assert!(sections.contains("Notes"));
        assert!(sections.contains("Settings"));
    }

    #[test]
    fn cat06_notes_no_selection_sections() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let sections: HashSet<&str> = actions
            .iter()
            .filter_map(|a| a.section.as_deref())
            .collect();
        assert!(sections.contains("Notes"));
        assert!(sections.contains("Settings"));
        assert!(!sections.contains("Edit"), "no Edit without selection");
        assert!(!sections.contains("Copy"), "no Copy without selection");
    }

    #[test]
    fn cat06_notes_auto_sizing_enabled_hides_setting() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        assert!(!ids.contains(&"enable_auto_sizing".to_string()));
    }

    #[test]
    fn cat06_notes_auto_sizing_disabled_shows_setting() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"enable_auto_sizing".to_string()));
    }

    // =========================================================================
    // cat07: Chat context continue_in_chat shortcut
    // =========================================================================

    #[test]
    fn cat07_chat_continue_shortcut_cmd_enter() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let cont = actions.iter().find(|a| a.id == "continue_in_chat").unwrap();
        assert_eq!(cont.shortcut.as_deref(), Some("⌘↵"));
    }

    #[test]
    fn cat07_chat_continue_always_present() {
        // Even with no models, continue_in_chat should be present
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let ids = action_ids(&get_chat_context_actions(&info));
        assert!(ids.contains(&"continue_in_chat".to_string()));
    }

    #[test]
    fn cat07_chat_copy_response_conditional_true() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: true,
        };
        let ids = action_ids(&get_chat_context_actions(&info));
        assert!(ids.contains(&"copy_response".to_string()));
    }

    #[test]
    fn cat07_chat_copy_response_conditional_false() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let ids = action_ids(&get_chat_context_actions(&info));
        assert!(!ids.contains(&"copy_response".to_string()));
    }

    #[test]
    fn cat07_chat_clear_conditional_true() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: false,
        };
        let ids = action_ids(&get_chat_context_actions(&info));
        assert!(ids.contains(&"clear_conversation".to_string()));
    }

    #[test]
    fn cat07_chat_clear_conditional_false() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let ids = action_ids(&get_chat_context_actions(&info));
        assert!(!ids.contains(&"clear_conversation".to_string()));
    }

    // =========================================================================
    // cat08: AI command bar export section has exactly one action
    // =========================================================================

    #[test]
    fn cat08_ai_export_section_count() {
        let actions = get_ai_command_bar_actions();
        let export_count = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Export"))
            .count();
        assert_eq!(export_count, 1, "Export section has exactly 1 action");
    }

    #[test]
    fn cat08_ai_export_action_is_export_markdown() {
        let actions = get_ai_command_bar_actions();
        let export = actions
            .iter()
            .find(|a| a.section.as_deref() == Some("Export"))
            .unwrap();
        assert_eq!(export.id, "export_markdown");
    }

    #[test]
    fn cat08_ai_export_markdown_shortcut() {
        let actions = get_ai_command_bar_actions();
        let export = actions.iter().find(|a| a.id == "export_markdown").unwrap();
        assert_eq!(export.shortcut.as_deref(), Some("⇧⌘E"));
    }

    #[test]
    fn cat08_ai_export_markdown_icon() {
        let actions = get_ai_command_bar_actions();
        let export = actions.iter().find(|a| a.id == "export_markdown").unwrap();
        assert_eq!(export.icon, Some(IconName::FileCode));
    }

    // =========================================================================
    // cat09: New chat preset icon propagation
    // =========================================================================

    #[test]
    fn cat09_new_chat_preset_icon_preserved() {
        let presets = vec![NewChatPresetInfo {
            id: "general".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        let preset_action = actions.iter().find(|a| a.id == "preset_general").unwrap();
        assert_eq!(preset_action.icon, Some(IconName::Star));
    }

    #[test]
    fn cat09_new_chat_preset_no_description() {
        let presets = vec![NewChatPresetInfo {
            id: "code".into(),
            name: "Code".into(),
            icon: IconName::Code,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        let preset_action = actions.iter().find(|a| a.id == "preset_code").unwrap();
        assert!(preset_action.description.is_none());
    }

    #[test]
    fn cat09_new_chat_model_has_provider_description() {
        let models = vec![NewChatModelInfo {
            model_id: "gpt-4".into(),
            display_name: "GPT-4".into(),
            provider: "openai".into(),
            provider_display_name: "OpenAI".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        let model_action = actions.iter().find(|a| a.id == "model_0").unwrap();
        assert_eq!(model_action.description.as_deref(), Some("OpenAI"));
    }

    #[test]
    fn cat09_new_chat_model_icon_settings() {
        let models = vec![NewChatModelInfo {
            model_id: "claude".into(),
            display_name: "Claude".into(),
            provider: "anthropic".into(),
            provider_display_name: "Anthropic".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        let model = actions.iter().find(|a| a.id == "model_0").unwrap();
        assert_eq!(model.icon, Some(IconName::Settings));
    }

    #[test]
    fn cat09_new_chat_last_used_bolt_icon() {
        let last_used = vec![NewChatModelInfo {
            model_id: "gpt-4o".into(),
            display_name: "GPT-4o".into(),
            provider: "openai".into(),
            provider_display_name: "OpenAI".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        let lu = actions.iter().find(|a| a.id == "last_used_0").unwrap();
        assert_eq!(lu.icon, Some(IconName::BoltFilled));
    }

    // =========================================================================
    // cat10: Note switcher pinned+current combined state
    // =========================================================================

    #[test]
    fn cat10_pinned_current_icon_is_star() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n1".into(),
            title: "My Note".into(),
            char_count: 100,
            is_current: true,
            is_pinned: true,
            preview: "content".into(),
            relative_time: "1m ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    }

    #[test]
    fn cat10_pinned_current_has_bullet_prefix() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n2".into(),
            title: "Pinned Current".into(),
            char_count: 50,
            is_current: true,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert!(
            actions[0].title.starts_with("• "),
            "current note should have bullet prefix"
        );
    }

    #[test]
    fn cat10_pinned_not_current_no_bullet() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n3".into(),
            title: "Pinned Only".into(),
            char_count: 20,
            is_current: false,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].title, "Pinned Only");
    }

    #[test]
    fn cat10_pinned_section_is_pinned() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n4".into(),
            title: "Pin".into(),
            char_count: 5,
            is_current: false,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
    }

    #[test]
    fn cat10_unpinned_section_is_recent() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n5".into(),
            title: "Regular".into(),
            char_count: 5,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Recent"));
    }

    // =========================================================================
    // cat11: format_shortcut_hint modifier keyword normalization
    // =========================================================================

    #[test]
    fn cat11_format_shortcut_cmd_c() {
        // Using the builders-private fn via ActionsDialog
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘C");
        assert_eq!(keycaps, vec!["⌘", "C"]);
    }

    #[test]
    fn cat11_format_shortcut_ctrl_alt_del() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌃⌥⌫");
        assert_eq!(keycaps, vec!["⌃", "⌥", "⌫"]);
    }

    #[test]
    fn cat11_format_shortcut_enter() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("↵");
        assert_eq!(keycaps, vec!["↵"]);
    }

    #[test]
    fn cat11_format_shortcut_arrows() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("↑↓←→");
        assert_eq!(keycaps, vec!["↑", "↓", "←", "→"]);
    }

    #[test]
    fn cat11_format_shortcut_escape() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⎋");
        assert_eq!(keycaps, vec!["⎋"]);
    }

    #[test]
    fn cat11_format_shortcut_tab() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⇥");
        assert_eq!(keycaps, vec!["⇥"]);
    }

    // =========================================================================
    // cat12: to_deeplink_name numeric and underscore handling
    // =========================================================================

    #[test]
    fn cat12_deeplink_numeric_only() {
        assert_eq!(to_deeplink_name("12345"), "12345");
    }

    #[test]
    fn cat12_deeplink_underscores_to_hyphens() {
        assert_eq!(to_deeplink_name("hello_world"), "hello-world");
    }

    #[test]
    fn cat12_deeplink_mixed_case_lowered() {
        assert_eq!(to_deeplink_name("MyScript"), "myscript");
    }

    #[test]
    fn cat12_deeplink_consecutive_specials_collapsed() {
        assert_eq!(to_deeplink_name("a!!b"), "a-b");
    }

    #[test]
    fn cat12_deeplink_leading_trailing_stripped() {
        assert_eq!(to_deeplink_name("--hello--"), "hello");
    }

    // =========================================================================
    // cat13: score_action empty query behaviour
    // =========================================================================

