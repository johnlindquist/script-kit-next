    #[test]
    fn chat_context_has_messages_no_response() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let ids = action_ids(&actions);
        assert!(!ids.contains(&"copy_response"));
        assert!(ids.contains(&"clear_conversation"));
    }

    // =========================================================================
    // 6. Notes info systematic boolean combos with section labels
    // =========================================================================

    #[test]
    fn notes_all_false_has_new_note_browse_and_auto_sizing() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"new_note"));
        assert!(ids.contains(&"browse_notes"));
        assert!(ids.contains(&"enable_auto_sizing"));
        assert!(!ids.contains(&"duplicate_note"));
        assert!(!ids.contains(&"find_in_note"));
    }

    #[test]
    fn notes_selection_no_trash_no_auto_has_full_set() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"new_note"));
        assert!(ids.contains(&"duplicate_note"));
        assert!(ids.contains(&"browse_notes"));
        assert!(ids.contains(&"find_in_note"));
        assert!(ids.contains(&"format"));
        assert!(ids.contains(&"copy_note_as"));
        assert!(ids.contains(&"copy_deeplink"));
        assert!(ids.contains(&"create_quicklink"));
        assert!(ids.contains(&"export"));
        assert!(ids.contains(&"enable_auto_sizing"));
    }

    #[test]
    fn notes_selection_no_trash_auto_enabled_hides_auto_sizing() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        assert!(!ids.contains(&"enable_auto_sizing"));
        // Everything else present
        assert!(ids.contains(&"duplicate_note"));
        assert!(ids.contains(&"export"));
    }

    #[test]
    fn notes_selection_trash_hides_conditional_actions() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids = action_ids(&actions);
        // Trash view hides selection-dependent actions
        assert!(!ids.contains(&"duplicate_note"));
        assert!(!ids.contains(&"find_in_note"));
        assert!(!ids.contains(&"format"));
        assert!(!ids.contains(&"copy_note_as"));
        assert!(!ids.contains(&"export"));
        // These are always present
        assert!(ids.contains(&"new_note"));
        assert!(ids.contains(&"browse_notes"));
    }

    #[test]
    fn notes_no_selection_trash_minimal_actions() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: true,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        // Only new_note and browse_notes (auto_sizing_enabled=true hides that)
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].id, "new_note");
        assert_eq!(actions[1].id, "browse_notes");
    }

    #[test]
    fn notes_section_labels_present_for_full_set() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        // Verify section labels
        let sections: Vec<&str> = actions
            .iter()
            .filter_map(|a| a.section.as_deref())
            .collect();
        assert!(sections.contains(&"Notes"));
        assert!(sections.contains(&"Edit"));
        assert!(sections.contains(&"Copy"));
        assert!(sections.contains(&"Export"));
        assert!(sections.contains(&"Settings"));
    }

    #[test]
    fn notes_icons_present_for_all_actions() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        for action in &actions {
            assert!(
                action.icon.is_some(),
                "Action '{}' should have an icon",
                action.id
            );
        }
    }

    // =========================================================================
    // 7. Note switcher mixed pinned/unpinned section assignment
    // =========================================================================

    #[test]
    fn note_switcher_pinned_notes_in_pinned_section() {
        let notes = vec![
            NoteSwitcherNoteInfo {
                id: "p1".into(),
                title: "Pinned Note".into(),
                char_count: 50,
                is_current: false,
                is_pinned: true,
                preview: "pinned content".into(),
                relative_time: "1h ago".into(),
            },
            NoteSwitcherNoteInfo {
                id: "r1".into(),
                title: "Recent Note".into(),
                char_count: 30,
                is_current: false,
                is_pinned: false,
                preview: "recent content".into(),
                relative_time: "5m ago".into(),
            },
        ];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
        assert_eq!(actions[1].section.as_deref(), Some("Recent"));
    }

    #[test]
    fn note_switcher_current_pinned_gets_star_icon() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "cp".into(),
            title: "Current Pinned".into(),
            char_count: 10,
            is_current: true,
            is_pinned: true,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        // Pinned takes precedence over current for icon
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
        // But current still gets bullet prefix
        assert!(actions[0].title.starts_with("• "));
    }

    #[test]
    fn note_switcher_current_not_pinned_gets_check_icon() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "cn".into(),
            title: "Current Note".into(),
            char_count: 10,
            is_current: true,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::Check));
    }

    #[test]
    fn note_switcher_regular_note_gets_file_icon() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "rn".into(),
            title: "Regular Note".into(),
            char_count: 10,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::File));
    }

    #[test]
    fn note_switcher_id_format_is_note_prefix() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc-123".into(),
            title: "Test".into(),
            char_count: 5,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].id, "note_abc-123");
    }

    #[test]
    fn note_switcher_empty_shows_no_notes_message() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].id, "no_notes");
        assert_eq!(actions[0].title, "No notes yet");
        assert_eq!(actions[0].section.as_deref(), Some("Notes"));
    }

    #[test]
    fn note_switcher_char_count_singular() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "s1".into(),
            title: "One Char".into(),
            char_count: 1,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "1 char");
    }

    #[test]
    fn note_switcher_char_count_plural() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "s2".into(),
            title: "Many Chars".into(),
            char_count: 42,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "42 chars");
    }

    #[test]
    fn note_switcher_char_count_zero() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "s0".into(),
            title: "Empty Note".into(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "0 chars");
    }

    #[test]
    fn note_switcher_preview_exactly_60_chars_not_truncated() {
        let preview = "a".repeat(60);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "t60".into(),
            title: "Exact 60".into(),
            char_count: 60,
            is_current: false,
            is_pinned: false,
            preview: preview.clone(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, &preview);
        assert!(!desc.contains("…"), "60 chars should not be truncated");
    }

    #[test]
    fn note_switcher_preview_61_chars_is_truncated() {
        let preview = "a".repeat(61);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "t61".into(),
            title: "Over 60".into(),
            char_count: 61,
            is_current: false,
            is_pinned: false,
            preview,
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.ends_with("…"), "61 chars should be truncated with …");
    }

    #[test]
    fn note_switcher_relative_time_only_no_preview() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "to".into(),
            title: "Time Only".into(),
            char_count: 10,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "3d ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "3d ago");
    }

    // =========================================================================
    // 8. New chat with partial sections
    // =========================================================================

    #[test]
    fn new_chat_no_last_used_only_presets_and_models() {
        let presets = vec![NewChatPresetInfo {
            id: "general".into(),
            name: "General".into(),
            icon: IconName::Settings,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "gpt4".into(),
            display_name: "GPT-4".into(),
            provider: "openai".into(),
            provider_display_name: "OpenAI".into(),
        }];
        let actions = get_new_chat_actions(&[], &presets, &models);
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].section.as_deref(), Some("Presets"));
        assert_eq!(actions[1].section.as_deref(), Some("Models"));
    }

    #[test]
    fn new_chat_only_models_no_presets_no_last_used() {
        let models = vec![
            NewChatModelInfo {
                model_id: "m1".into(),
                display_name: "Model 1".into(),
                provider: "p1".into(),
                provider_display_name: "Provider 1".into(),
            },
            NewChatModelInfo {
                model_id: "m2".into(),
                display_name: "Model 2".into(),
                provider: "p2".into(),
                provider_display_name: "Provider 2".into(),
            },
        ];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].id, "model_0");
        assert_eq!(actions[1].id, "model_1");
    }

    #[test]
    fn new_chat_only_last_used() {
        let last_used = vec![NewChatModelInfo {
            model_id: "lu1".into(),
            display_name: "Last Used".into(),
            provider: "p".into(),
            provider_display_name: "Provider".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].id, "last_used_0");
        assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
        assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
    }

    #[test]
    fn new_chat_all_three_sections_have_correct_section_labels() {
        let last_used = vec![NewChatModelInfo {
            model_id: "lu".into(),
            display_name: "LU".into(),
            provider: "p".into(),
            provider_display_name: "P".into(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "pr".into(),
            name: "Preset".into(),
            icon: IconName::Code,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "m".into(),
            display_name: "Model".into(),
            provider: "p".into(),
            provider_display_name: "P".into(),
        }];
        let actions = get_new_chat_actions(&last_used, &presets, &models);
        assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
        assert_eq!(actions[1].section.as_deref(), Some("Presets"));
        assert_eq!(actions[2].section.as_deref(), Some("Models"));
    }

    #[test]
    fn new_chat_preset_has_no_description() {
        let presets = vec![NewChatPresetInfo {
            id: "gen".into(),
            name: "General".into(),
            icon: IconName::Settings,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert!(actions[0].description.is_none());
    }

