    #[test]
    fn notes_cmd_bar_new_note_section_notes() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "new_note").unwrap();
        assert_eq!(a.section.as_deref(), Some("Notes"));
    }

    #[test]
    fn notes_cmd_bar_find_in_note_section_edit() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "find_in_note").unwrap();
        assert_eq!(a.section.as_deref(), Some("Edit"));
    }

    #[test]
    fn notes_cmd_bar_copy_note_as_section_copy() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "copy_note_as").unwrap();
        assert_eq!(a.section.as_deref(), Some("Copy"));
    }

    #[test]
    fn notes_cmd_bar_export_section_export() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "export").unwrap();
        assert_eq!(a.section.as_deref(), Some("Export"));
    }

    // ========================================
    // 29. ID uniqueness and non-empty invariants (6 tests)
    // ========================================

    #[test]
    fn notes_cmd_bar_ids_unique() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids: HashSet<_> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn chat_context_ids_unique() {
        let info = ChatPromptInfo {
            current_model: Some("M1".to_string()),
            available_models: vec![
                ChatModelInfo {
                    id: "m1".to_string(),
                    display_name: "M1".to_string(),
                    provider: "P1".to_string(),
                },
                ChatModelInfo {
                    id: "m2".to_string(),
                    display_name: "M2".to_string(),
                    provider: "P2".to_string(),
                },
            ],
            has_messages: true,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        let ids: HashSet<_> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn new_chat_ids_unique() {
        let actions = get_new_chat_actions(
            &[NewChatModelInfo {
                model_id: "l1".to_string(),
                display_name: "L1".to_string(),
                provider: "p".to_string(),
                provider_display_name: "P".to_string(),
            }],
            &[NewChatPresetInfo {
                id: "gen".to_string(),
                name: "Gen".to_string(),
                icon: IconName::File,
            }],
            &[NewChatModelInfo {
                model_id: "m1".to_string(),
                display_name: "M1".to_string(),
                provider: "p".to_string(),
                provider_display_name: "P".to_string(),
            }],
        );
        let ids: HashSet<_> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn note_switcher_ids_unique() {
        let notes = vec![
            make_note("uuid-1", "Note 1", 10, false, false, "", ""),
            make_note("uuid-2", "Note 2", 20, true, false, "", ""),
            make_note("uuid-3", "Note 3", 30, false, true, "", ""),
        ];
        let actions = get_note_switcher_actions(&notes);
        let ids: HashSet<_> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn all_note_switcher_actions_nonempty_title() {
        let notes = vec![
            make_note("1", "A", 1, false, false, "", ""),
            make_note("2", "B", 2, true, true, "preview", "1m ago"),
        ];
        let actions = get_note_switcher_actions(&notes);
        for a in &actions {
            assert!(!a.title.is_empty(), "Action {} has empty title", a.id);
            assert!(!a.id.is_empty(), "Action has empty id");
        }
    }

    #[test]
    fn all_path_actions_nonempty_title_and_id() {
        let info = PathInfo {
            name: "test".to_string(),
            path: "/test".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        for a in &actions {
            assert!(!a.title.is_empty());
            assert!(!a.id.is_empty());
        }
    }

    // ========================================
    // 30. Ordering determinism (4 tests)
    // ========================================

    #[test]
    fn notes_cmd_bar_ordering_deterministic() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let a = get_notes_command_bar_actions(&info);
        let b = get_notes_command_bar_actions(&info);
        assert_eq!(action_ids(&a), action_ids(&b));
    }

    #[test]
    fn chat_context_ordering_deterministic() {
        let info = ChatPromptInfo {
            current_model: Some("X".to_string()),
            available_models: vec![
                ChatModelInfo {
                    id: "a".to_string(),
                    display_name: "A".to_string(),
                    provider: "P".to_string(),
                },
                ChatModelInfo {
                    id: "b".to_string(),
                    display_name: "B".to_string(),
                    provider: "P".to_string(),
                },
            ],
            has_messages: true,
            has_response: true,
        };
        let a = get_chat_context_actions(&info);
        let b = get_chat_context_actions(&info);
        assert_eq!(action_ids(&a), action_ids(&b));
    }

    #[test]
    fn new_chat_ordering_deterministic() {
        let last = vec![NewChatModelInfo {
            model_id: "l".to_string(),
            display_name: "L".to_string(),
            provider: "p".to_string(),
            provider_display_name: "P".to_string(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "g".to_string(),
            name: "G".to_string(),
            icon: IconName::File,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "m".to_string(),
            display_name: "M".to_string(),
            provider: "p".to_string(),
            provider_display_name: "P".to_string(),
        }];
        let a = get_new_chat_actions(&last, &presets, &models);
        let b = get_new_chat_actions(&last, &presets, &models);
        assert_eq!(action_ids(&a), action_ids(&b));
    }

    #[test]
    fn path_context_ordering_deterministic() {
        let info = PathInfo {
            name: "f".to_string(),
            path: "/f".to_string(),
            is_dir: false,
        };
        let a = get_path_context_actions(&info);
        let b = get_path_context_actions(&info);
        assert_eq!(action_ids(&a), action_ids(&b));
    }
