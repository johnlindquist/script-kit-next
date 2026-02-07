    #[test]
    fn cat21_empty_inputs_empty_actions() {
        let actions = get_new_chat_actions(&[], &[], &[]);
        assert!(actions.is_empty());
    }

    #[test]
    fn cat21_last_used_section() {
        let lu = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "Claude".into(),
            provider: "anthropic".into(),
            provider_display_name: "Anthropic".into(),
        }];
        let actions = get_new_chat_actions(&lu, &[], &[]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
        assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
    }

    #[test]
    fn cat21_preset_section() {
        let presets = vec![NewChatPresetInfo {
            id: "general".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].section.as_deref(), Some("Presets"));
        assert!(
            actions[0].description.is_none(),
            "Presets have no description"
        );
    }

    #[test]
    fn cat21_models_section() {
        let models = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "GPT-4".into(),
            provider: "openai".into(),
            provider_display_name: "OpenAI".into(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].section.as_deref(), Some("Models"));
        assert_eq!(actions[0].icon, Some(IconName::Settings));
        assert_eq!(actions[0].description, Some("OpenAI".into()));
    }

    #[test]
    fn cat21_section_ordering() {
        let lu = vec![NewChatModelInfo {
            model_id: "m1".into(),
            display_name: "A".into(),
            provider: "p".into(),
            provider_display_name: "P".into(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "g".into(),
            name: "G".into(),
            icon: IconName::Star,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "m2".into(),
            display_name: "B".into(),
            provider: "p".into(),
            provider_display_name: "P".into(),
        }];
        let actions = get_new_chat_actions(&lu, &presets, &models);
        assert_eq!(actions.len(), 3);
        assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
        assert_eq!(actions[1].section.as_deref(), Some("Presets"));
        assert_eq!(actions[2].section.as_deref(), Some("Models"));
    }

    // =========================================================================
    // 22. Notes command bar auto_sizing toggle
    // =========================================================================

    #[test]
    fn cat22_auto_sizing_disabled_shows_enable() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(actions.iter().any(|a| a.id == "enable_auto_sizing"));
    }

    #[test]
    fn cat22_auto_sizing_enabled_hides_enable() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "enable_auto_sizing"));
    }

    #[test]
    fn cat22_auto_sizing_in_settings_section() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = actions
            .iter()
            .find(|a| a.id == "enable_auto_sizing")
            .unwrap();
        assert_eq!(a.section.as_deref(), Some("Settings"));
    }

    #[test]
    fn cat22_auto_sizing_icon_is_settings() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = actions
            .iter()
            .find(|a| a.id == "enable_auto_sizing")
            .unwrap();
        assert_eq!(a.icon, Some(IconName::Settings));
    }

    // =========================================================================
    // 23. File context FileType variants
    // =========================================================================

    #[test]
    fn cat23_document_and_image_same_file_actions() {
        let doc = FileInfo {
            path: "/t".into(),
            name: "t".into(),
            file_type: crate::file_search::FileType::Document,
            is_dir: false,
        };
        let img = FileInfo {
            path: "/t".into(),
            name: "t".into(),
            file_type: crate::file_search::FileType::Image,
            is_dir: false,
        };
        let doc_ids = action_ids(&get_file_context_actions(&doc));
        let img_ids = action_ids(&get_file_context_actions(&img));
        assert_eq!(doc_ids, img_ids, "FileType should not affect action list");
    }

    #[test]
    fn cat23_directory_different_from_file() {
        let file = FileInfo {
            path: "/t".into(),
            name: "t".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let dir = FileInfo {
            path: "/t".into(),
            name: "t".into(),
            file_type: crate::file_search::FileType::Directory,
            is_dir: true,
        };
        let file_ids = action_ids(&get_file_context_actions(&file));
        let dir_ids = action_ids(&get_file_context_actions(&dir));
        assert_ne!(
            file_ids, dir_ids,
            "Dir and file should have different actions"
        );
    }

    // =========================================================================
    // 24. Clipboard destructive actions always last three
    // =========================================================================

    #[test]
    fn cat24_text_last_three_destructive() {
        let actions = get_clipboard_history_context_actions(&make_text_entry());
        let ids = action_ids(&actions);
        let n = ids.len();
        assert_eq!(ids[n - 3], "clipboard_delete");
        assert_eq!(ids[n - 2], "clipboard_delete_multiple");
        assert_eq!(ids[n - 1], "clipboard_delete_all");
    }

    #[test]
    fn cat24_image_last_three_destructive() {
        let actions = get_clipboard_history_context_actions(&make_image_entry());
        let ids = action_ids(&actions);
        let n = ids.len();
        assert_eq!(ids[n - 3], "clipboard_delete");
        assert_eq!(ids[n - 2], "clipboard_delete_multiple");
        assert_eq!(ids[n - 1], "clipboard_delete_all");
    }

    #[test]
    fn cat24_paste_always_first() {
        let actions = get_clipboard_history_context_actions(&make_text_entry());
        assert_eq!(actions[0].id, "clipboard_paste");
    }

    #[test]
    fn cat24_copy_always_second() {
        let actions = get_clipboard_history_context_actions(&make_text_entry());
        assert_eq!(actions[1].id, "clipboard_copy");
    }

    // =========================================================================
    // 25. Note switcher icon hierarchy
    // =========================================================================

    #[test]
    fn cat25_pinned_gets_star_filled() {
        let note = NoteSwitcherNoteInfo {
            id: "n1".into(),
            title: "T".into(),
            char_count: 0,
            is_current: false,
            is_pinned: true,
            preview: "".into(),
            relative_time: "".into(),
        };
        let actions = get_note_switcher_actions(&[note]);
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    }

    #[test]
    fn cat25_current_gets_check() {
        let note = NoteSwitcherNoteInfo {
            id: "n1".into(),
            title: "T".into(),
            char_count: 0,
            is_current: true,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        };
        let actions = get_note_switcher_actions(&[note]);
        assert_eq!(actions[0].icon, Some(IconName::Check));
    }

    #[test]
    fn cat25_regular_gets_file() {
        let note = NoteSwitcherNoteInfo {
            id: "n1".into(),
            title: "T".into(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        };
        let actions = get_note_switcher_actions(&[note]);
        assert_eq!(actions[0].icon, Some(IconName::File));
    }

    #[test]
    fn cat25_pinned_overrides_current() {
        let note = NoteSwitcherNoteInfo {
            id: "n1".into(),
            title: "T".into(),
            char_count: 0,
            is_current: true,
            is_pinned: true,
            preview: "".into(),
            relative_time: "".into(),
        };
        let actions = get_note_switcher_actions(&[note]);
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    }

    // =========================================================================
    // 26. Note switcher section assignment
    // =========================================================================

    #[test]
    fn cat26_pinned_in_pinned_section() {
        let note = NoteSwitcherNoteInfo {
            id: "n1".into(),
            title: "T".into(),
            char_count: 0,
            is_current: false,
            is_pinned: true,
            preview: "".into(),
            relative_time: "".into(),
        };
        let actions = get_note_switcher_actions(&[note]);
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
    }

    #[test]
    fn cat26_unpinned_in_recent_section() {
        let note = NoteSwitcherNoteInfo {
            id: "n1".into(),
            title: "T".into(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        };
        let actions = get_note_switcher_actions(&[note]);
        assert_eq!(actions[0].section.as_deref(), Some("Recent"));
    }

    #[test]
    fn cat26_current_note_bullet_prefix() {
        let note = NoteSwitcherNoteInfo {
            id: "n1".into(),
            title: "My Note".into(),
            char_count: 0,
            is_current: true,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        };
        let actions = get_note_switcher_actions(&[note]);
        assert!(actions[0].title.starts_with("• "));
    }

    #[test]
    fn cat26_non_current_no_bullet() {
        let note = NoteSwitcherNoteInfo {
            id: "n1".into(),
            title: "My Note".into(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        };
        let actions = get_note_switcher_actions(&[note]);
        assert!(!actions[0].title.starts_with("• "));
    }

    #[test]
    fn cat26_empty_notes_placeholder() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].id, "no_notes");
        assert_eq!(actions[0].icon, Some(IconName::Plus));
    }

    // =========================================================================
    // 27. Action builder chaining preserves fields
    // =========================================================================

    #[test]
    fn cat27_with_icon_preserves_other_fields() {
        let a = Action::new(
            "id",
            "Title",
            Some("Desc".into()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::Copy);
        assert_eq!(a.id, "id");
        assert_eq!(a.title, "Title");
        assert_eq!(a.description, Some("Desc".into()));
        assert_eq!(a.icon, Some(IconName::Copy));
    }

    #[test]
    fn cat27_with_section_preserves_other_fields() {
        let a =
            Action::new("id", "Title", None, ActionCategory::ScriptContext).with_section("MySec");
        assert_eq!(a.section, Some("MySec".into()));
        assert_eq!(a.id, "id");
    }

    #[test]
    fn cat27_chaining_all_builders() {
        let a = Action::new(
            "id",
            "Title",
            Some("D".into()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘E")
        .with_icon(IconName::Settings)
        .with_section("S");
        assert_eq!(a.shortcut, Some("⌘E".into()));
        assert_eq!(a.icon, Some(IconName::Settings));
        assert_eq!(a.section, Some("S".into()));
        assert_eq!(a.title, "Title");
    }

    // =========================================================================
    // 28. Cross-context ID uniqueness
    // =========================================================================

    #[test]
    fn cat28_script_ids_unique() {
        let s = ScriptInfo::new("t", "/p");
        let actions = get_script_context_actions(&s);
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), actions.len(), "Script action IDs must be unique");
    }

    #[test]
    fn cat28_clipboard_ids_unique() {
        let actions = get_clipboard_history_context_actions(&make_text_entry());
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(
            ids.len(),
            actions.len(),
            "Clipboard action IDs must be unique"
        );
    }

    #[test]
    fn cat28_ai_ids_unique() {
        let actions = get_ai_command_bar_actions();
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), actions.len(), "AI action IDs must be unique");
    }

    #[test]
    fn cat28_path_ids_unique() {
        let info = PathInfo {
            path: "/t".into(),
            name: "t".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), actions.len(), "Path action IDs must be unique");
    }

    #[test]
    fn cat28_file_ids_unique() {
        let info = FileInfo {
            path: "/t".into(),
            name: "t".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), actions.len(), "File action IDs must be unique");
    }

