    #[test]
    fn cat19_clipboard_unpinned_last_three_destructive() {
        let entry = ClipboardEntryInfo {
            id: "u1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "txt".into(),
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
    fn cat19_clipboard_pinned_last_three_destructive() {
        let entry = ClipboardEntryInfo {
            id: "p1".into(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "txt".into(),
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
    fn cat19_clipboard_paste_always_first() {
        for pinned in [true, false] {
            for ct in [ContentType::Text, ContentType::Image] {
                let entry = ClipboardEntryInfo {
                    id: "x".into(),
                    content_type: ct,
                    pinned,
                    preview: "p".into(),
                    image_dimensions: if ct == ContentType::Image {
                        Some((1, 1))
                    } else {
                        None
                    },
                    frontmost_app_name: None,
                };
                let actions = get_clipboard_history_context_actions(&entry);
                assert_eq!(
                    actions[0].id, "clipboard_paste",
                    "Paste should be first for pinned={} type={:?}",
                    pinned, ct
                );
            }
        }
    }

    #[test]
    fn cat19_clipboard_copy_always_second() {
        let entry = ClipboardEntryInfo {
            id: "c".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "t".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions[1].id, "clipboard_copy");
    }

    // =========================================================================
    // 20. Global actions always empty
    // =========================================================================

    #[test]
    fn cat20_global_actions_empty() {
        let actions = get_global_actions();
        assert!(actions.is_empty());
    }

    // =========================================================================
    // 21. New chat action structure and ID patterns
    // =========================================================================

    #[test]
    fn cat21_new_chat_empty_inputs_empty_actions() {
        let actions = get_new_chat_actions(&[], &[], &[]);
        assert!(actions.is_empty());
    }

    #[test]
    fn cat21_new_chat_last_used_id_pattern() {
        let last_used = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "Claude".into(),
            provider: "anthropic".into(),
            provider_display_name: "Anthropic".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions[0].id, "last_used_0");
    }

    #[test]
    fn cat21_new_chat_preset_id_pattern() {
        let presets = vec![NewChatPresetInfo {
            id: "general".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(actions[0].id, "preset_general");
    }

    #[test]
    fn cat21_new_chat_model_id_pattern() {
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
    fn cat21_new_chat_section_ordering() {
        let last_used = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "Claude".into(),
            provider: "anthropic".into(),
            provider_display_name: "Anthropic".into(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "gen".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "m2".into(),
            display_name: "GPT-4".into(),
            provider: "openai".into(),
            provider_display_name: "OpenAI".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &presets, &models);
        assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
        assert_eq!(actions[1].section.as_deref(), Some("Presets"));
        assert_eq!(actions[2].section.as_deref(), Some("Models"));
    }

    #[test]
    fn cat21_new_chat_last_used_icon_bolt() {
        let last_used = vec![NewChatModelInfo {
            model_id: "x".into(),
            display_name: "X".into(),
            provider: "p".into(),
            provider_display_name: "P".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
    }

    #[test]
    fn cat21_new_chat_model_icon_settings() {
        let models = vec![NewChatModelInfo {
            model_id: "x".into(),
            display_name: "X".into(),
            provider: "p".into(),
            provider_display_name: "P".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions[0].icon, Some(IconName::Settings));
    }

    // =========================================================================
    // 22. format_shortcut_hint additional cases
    // =========================================================================

    #[test]
    fn cat22_format_shortcut_opt_maps_option() {
        let result = ActionsDialog::format_shortcut_hint("opt+v");
        assert_eq!(result, "⌥V");
    }

    #[test]
    fn cat22_format_shortcut_triple_modifier() {
        let result = ActionsDialog::format_shortcut_hint("cmd+shift+ctrl+a");
        assert_eq!(result, "⌘⇧⌃A");
    }

    #[test]
    fn cat22_format_shortcut_space() {
        let result = ActionsDialog::format_shortcut_hint("space");
        assert_eq!(result, "␣");
    }

    #[test]
    fn cat22_format_shortcut_tab() {
        let result = ActionsDialog::format_shortcut_hint("tab");
        assert_eq!(result, "⇥");
    }

    #[test]
    fn cat22_format_shortcut_arrowup() {
        let result = ActionsDialog::format_shortcut_hint("arrowup");
        assert_eq!(result, "↑");
    }

    #[test]
    fn cat22_format_shortcut_arrowdown() {
        let result = ActionsDialog::format_shortcut_hint("arrowdown");
        assert_eq!(result, "↓");
    }

    // =========================================================================
    // 23. Agent context actions
    // =========================================================================

    #[test]
    fn cat23_agent_has_edit_agent_title() {
        let mut script = ScriptInfo::new("My Agent", "/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit.title, "Edit Agent");
    }

    #[test]
    fn cat23_agent_has_no_view_logs() {
        let mut script = ScriptInfo::new("My Agent", "/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let ids = action_ids(&get_script_context_actions(&script));
        assert!(!ids.contains(&"view_logs".to_string()));
    }

    #[test]
    fn cat23_agent_has_copy_content() {
        let mut script = ScriptInfo::new("My Agent", "/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let ids = action_ids(&get_script_context_actions(&script));
        assert!(ids.contains(&"copy_content".to_string()));
    }

    #[test]
    fn cat23_agent_has_reveal_and_copy_path() {
        let mut script = ScriptInfo::new("My Agent", "/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let ids = action_ids(&get_script_context_actions(&script));
        assert!(ids.contains(&"reveal_in_finder".to_string()));
        assert!(ids.contains(&"copy_path".to_string()));
    }

    #[test]
    fn cat23_agent_description_mentions_agent() {
        let mut script = ScriptInfo::new("My Agent", "/agent.md");
        script.is_agent = true;
        script.is_script = false;
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert!(edit
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("agent"));
    }

    // =========================================================================
    // 24. Cross-context ID uniqueness
    // =========================================================================

    #[test]
    fn cat24_script_context_ids_unique() {
        let script = ScriptInfo::new("test", "/test.ts");
        let ids = action_ids(&get_script_context_actions(&script));
        let set: HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), set.len());
    }

    #[test]
    fn cat24_clipboard_context_ids_unique() {
        let entry = ClipboardEntryInfo {
            id: "c".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "t".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let ids = action_ids(&get_clipboard_history_context_actions(&entry));
        let set: HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), set.len());
    }

    #[test]
    fn cat24_ai_command_bar_ids_unique() {
        let ids = action_ids(&get_ai_command_bar_actions());
        let set: HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), set.len());
    }

    #[test]
    fn cat24_path_context_ids_unique() {
        let info = PathInfo {
            path: "/test".into(),
            name: "test".into(),
            is_dir: false,
        };
        let ids = action_ids(&get_path_context_actions(&info));
        let set: HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), set.len());
    }

    #[test]
    fn cat24_notes_context_ids_unique() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let ids = action_ids(&get_notes_command_bar_actions(&info));
        let set: HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), set.len());
    }

    #[test]
    fn cat24_file_context_ids_unique() {
        let info = FileInfo {
            path: "/test/f.txt".into(),
            name: "f.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let ids = action_ids(&get_file_context_actions(&info));
        let set: HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), set.len());
    }

    // =========================================================================
    // 25. has_action=false for all built-ins
    // =========================================================================

    #[test]
    fn cat25_script_all_has_action_false() {
        let script = ScriptInfo::new("test", "/test.ts");
        for action in &get_script_context_actions(&script) {
            assert!(
                !action.has_action,
                "Action {} has_action should be false",
                action.id
            );
        }
    }

    #[test]
    fn cat25_clipboard_all_has_action_false() {
        let entry = ClipboardEntryInfo {
            id: "c".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "t".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        for action in &get_clipboard_history_context_actions(&entry) {
            assert!(
                !action.has_action,
                "Action {} has_action should be false",
                action.id
            );
        }
    }

    #[test]
    fn cat25_ai_all_has_action_false() {
        for action in &get_ai_command_bar_actions() {
            assert!(
                !action.has_action,
                "Action {} has_action should be false",
                action.id
            );
        }
    }

    #[test]
    fn cat25_path_all_has_action_false() {
        let info = PathInfo {
            path: "/test".into(),
            name: "test".into(),
            is_dir: false,
        };
        for action in &get_path_context_actions(&info) {
            assert!(
                !action.has_action,
                "Action {} has_action should be false",
                action.id
            );
        }
    }

    #[test]
    fn cat25_file_all_has_action_false() {
        let info = FileInfo {
            path: "/test/f.txt".into(),
            name: "f.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        for action in &get_file_context_actions(&info) {
            assert!(
                !action.has_action,
                "Action {} has_action should be false",
                action.id
            );
        }
    }

    #[test]
    fn cat25_notes_all_has_action_false() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        for action in &get_notes_command_bar_actions(&info) {
            assert!(
                !action.has_action,
                "Action {} has_action should be false",
                action.id
            );
        }
    }

    // =========================================================================
    // 26. Ordering determinism
    // =========================================================================

