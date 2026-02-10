    #[test]
    fn cat06_note_switcher_current_gets_bullet_prefix() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n1".into(),
            title: "My Note".into(),
            char_count: 100,
            is_current: true,
            is_pinned: false,
            preview: "content".into(),
            relative_time: "1m ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert!(actions[0].title.starts_with("• "));
    }

    #[test]
    fn cat06_note_switcher_non_current_no_bullet() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n2".into(),
            title: "Other Note".into(),
            char_count: 50,
            is_current: false,
            is_pinned: false,
            preview: "stuff".into(),
            relative_time: "5m ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert!(!actions[0].title.starts_with("• "));
        assert_eq!(actions[0].title, "Other Note");
    }

    #[test]
    fn cat06_note_switcher_pinned_overrides_current_icon() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n3".into(),
            title: "Both".into(),
            char_count: 10,
            is_current: true,
            is_pinned: true,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        // Pinned takes priority over current for icon
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    }

    #[test]
    fn cat06_note_switcher_regular_icon_is_file() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n4".into(),
            title: "Regular".into(),
            char_count: 5,
            is_current: false,
            is_pinned: false,
            preview: "abc".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::File));
    }

    #[test]
    fn cat06_note_switcher_empty_shows_placeholder() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].id, "no_notes");
        assert!(actions[0].title.contains("No notes"));
    }

    #[test]
    fn cat06_note_switcher_id_format() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc-123".into(),
            title: "Test".into(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].id, "note_abc-123");
    }

    // =========================================================================
    // 7. Chat context: zero models, both flags false
    // =========================================================================

    #[test]
    fn cat07_chat_zero_models_still_has_continue() {
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
    fn cat07_chat_no_response_no_copy() {
        let info = ChatPromptInfo {
            current_model: Some("GPT-4".into()),
            available_models: vec![],
            has_messages: true,
            has_response: false,
        };
        let ids = action_ids(&get_chat_context_actions(&info));
        assert!(!ids.contains(&"copy_response".to_string()));
    }

    #[test]
    fn cat07_chat_no_messages_no_clear() {
        let info = ChatPromptInfo {
            current_model: Some("GPT-4".into()),
            available_models: vec![],
            has_messages: false,
            has_response: true,
        };
        let ids = action_ids(&get_chat_context_actions(&info));
        assert!(!ids.contains(&"clear_conversation".to_string()));
    }

    #[test]
    fn cat07_chat_both_flags_true_gives_max_actions() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".into()),
            available_models: vec![ChatModelInfo {
                id: "m1".into(),
                display_name: "Claude".into(),
                provider: "Anthropic".into(),
            }],
            has_messages: true,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        // 1 model + continue + copy_response + clear = 4
        assert_eq!(actions.len(), 4);
    }

    #[test]
    fn cat07_chat_current_model_gets_checkmark() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".into()),
            available_models: vec![
                ChatModelInfo {
                    id: "m1".into(),
                    display_name: "Claude".into(),
                    provider: "Anthropic".into(),
                },
                ChatModelInfo {
                    id: "m2".into(),
                    display_name: "GPT-4".into(),
                    provider: "OpenAI".into(),
                },
            ],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let claude = actions.iter().find(|a| a.id == "select_model_m1").unwrap();
        assert!(claude.title.contains("✓"));
        let gpt = actions.iter().find(|a| a.id == "select_model_m2").unwrap();
        assert!(!gpt.title.contains("✓"));
    }

    // =========================================================================
    // 8. Scriptlet context custom action ordering with multiple H3 actions
    // =========================================================================

    #[test]
    fn cat08_scriptlet_custom_actions_appear_after_run() {
        let script = ScriptInfo::scriptlet("Test", "/test.md", None, None);
        let mut scriptlet = Scriptlet::new(
            "Test".to_string(),
            "bash".to_string(),
            "echo main".to_string(),
        );
        scriptlet.actions = vec![
            ScriptletAction {
                name: "Alpha".into(),
                command: "alpha".into(),
                tool: "bash".into(),
                code: "echo a".into(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
            ScriptletAction {
                name: "Beta".into(),
                command: "beta".into(),
                tool: "bash".into(),
                code: "echo b".into(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
        ];
        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        let ids = action_ids(&actions);
        let run_idx = ids.iter().position(|id| id == "run_script").unwrap();
        let alpha_idx = ids
            .iter()
            .position(|id| id == "scriptlet_action:alpha")
            .unwrap();
        let beta_idx = ids
            .iter()
            .position(|id| id == "scriptlet_action:beta")
            .unwrap();
        assert!(run_idx < alpha_idx);
        assert!(alpha_idx < beta_idx);
    }

    #[test]
    fn cat08_scriptlet_custom_actions_before_builtins() {
        let script = ScriptInfo::scriptlet("Test", "/test.md", None, None);
        let mut scriptlet = Scriptlet::new(
            "Test".to_string(),
            "bash".to_string(),
            "echo main".to_string(),
        );
        scriptlet.actions = vec![ScriptletAction {
            name: "Custom".into(),
            command: "custom".into(),
            tool: "bash".into(),
            code: "echo c".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];
        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        let ids = action_ids(&actions);
        let custom_idx = ids
            .iter()
            .position(|id| id == "scriptlet_action:custom")
            .unwrap();
        let edit_idx = ids.iter().position(|id| id == "edit_scriptlet").unwrap();
        assert!(custom_idx < edit_idx);
    }

    #[test]
    fn cat08_scriptlet_custom_has_action_true() {
        let mut scriptlet =
            Scriptlet::new("Test".to_string(), "bash".to_string(), "echo".to_string());
        scriptlet.actions = vec![ScriptletAction {
            name: "Do Thing".into(),
            command: "do-thing".into(),
            tool: "bash".into(),
            code: "echo do".into(),
            inputs: vec![],
            shortcut: Some("cmd+d".into()),
            description: Some("Does the thing".into()),
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert!(actions[0].has_action);
        assert_eq!(actions[0].value, Some("do-thing".to_string()));
    }

    #[test]
    fn cat08_scriptlet_custom_shortcut_formatted() {
        let mut scriptlet =
            Scriptlet::new("Test".to_string(), "bash".to_string(), "echo".to_string());
        scriptlet.actions = vec![ScriptletAction {
            name: "Copy".into(),
            command: "copy".into(),
            tool: "bash".into(),
            code: "echo cp".into(),
            inputs: vec![],
            shortcut: Some("cmd+shift+c".into()),
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert_eq!(actions[0].shortcut, Some("⌘⇧C".to_string()));
    }

    #[test]
    fn cat08_scriptlet_no_actions_returns_empty() {
        let scriptlet = Scriptlet::new("Empty".to_string(), "bash".to_string(), "echo".to_string());
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert!(actions.is_empty());
    }

    // =========================================================================
    // 9. File context macOS-only action count delta
    // =========================================================================

    #[cfg(target_os = "macos")]
    #[test]
    fn cat09_file_macos_has_quick_look_open_with_show_info() {
        let info = FileInfo {
            path: "/test/file.txt".into(),
            name: "file.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let ids = action_ids(&get_file_context_actions(&info));
        assert!(ids.contains(&"quick_look".to_string()));
        assert!(ids.contains(&"open_with".to_string()));
        assert!(ids.contains(&"show_info".to_string()));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat09_file_dir_no_quick_look_macos() {
        let info = FileInfo {
            path: "/test/dir".into(),
            name: "dir".into(),
            file_type: crate::file_search::FileType::Directory,
            is_dir: true,
        };
        let ids = action_ids(&get_file_context_actions(&info));
        assert!(!ids.contains(&"quick_look".to_string()));
        // But still has open_with and show_info
        assert!(ids.contains(&"open_with".to_string()));
        assert!(ids.contains(&"show_info".to_string()));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat09_file_macos_file_more_actions_than_dir() {
        let file_info = FileInfo {
            path: "/test/f.txt".into(),
            name: "f.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let dir_info = FileInfo {
            path: "/test/d".into(),
            name: "d".into(),
            file_type: crate::file_search::FileType::Directory,
            is_dir: true,
        };
        let file_count = get_file_context_actions(&file_info).len();
        let dir_count = get_file_context_actions(&dir_info).len();
        assert!(
            file_count > dir_count,
            "File ({}) should have more actions than dir ({}) on macOS",
            file_count,
            dir_count
        );
    }

    // =========================================================================
    // 10. Cross-context description non-emptiness
    // =========================================================================

    #[test]
    fn cat10_script_context_all_have_descriptions() {
        let script = ScriptInfo::new("test", "/test.ts");
        for action in &get_script_context_actions(&script) {
            assert!(
                action.description.is_some(),
                "Script action '{}' should have a description",
                action.id
            );
        }
    }

    #[test]
    fn cat10_ai_command_bar_all_have_descriptions() {
        for action in &get_ai_command_bar_actions() {
            assert!(
                action.description.is_some(),
                "AI action '{}' should have a description",
                action.id
            );
        }
    }

    #[test]
    fn cat10_path_context_all_have_descriptions() {
        let info = PathInfo {
            path: "/test".into(),
            name: "test".into(),
            is_dir: false,
        };
        for action in &get_path_context_actions(&info) {
            assert!(
                action.description.is_some(),
                "Path action '{}' should have a description",
                action.id
            );
        }
    }

    #[test]
    fn cat10_file_context_all_have_descriptions() {
        let info = FileInfo {
            path: "/test/f.txt".into(),
            name: "f.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        for action in &get_file_context_actions(&info) {
            assert!(
                action.description.is_some(),
                "File action '{}' should have a description",
                action.id
            );
        }
    }

    #[test]
    fn cat10_clipboard_context_all_have_descriptions() {
        let entry = ClipboardEntryInfo {
            id: "c1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        for action in &get_clipboard_history_context_actions(&entry) {
            assert!(
                action.description.is_some(),
                "Clipboard action '{}' should have a description",
                action.id
            );
        }
    }

    #[test]
    fn cat10_notes_context_all_have_descriptions() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        for action in &get_notes_command_bar_actions(&info) {
            assert!(
                action.description.is_some(),
                "Notes action '{}' should have a description",
                action.id
            );
        }
    }

    // =========================================================================
    // 11. build_grouped_items_static with mixed sections
    // =========================================================================

