    #[test]
    fn cat26_script_ordering_deterministic() {
        let script = ScriptInfo::new("test", "/test.ts");
        let ids1 = action_ids(&get_script_context_actions(&script));
        let ids2 = action_ids(&get_script_context_actions(&script));
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn cat26_clipboard_ordering_deterministic() {
        let entry = ClipboardEntryInfo {
            id: "c".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "t".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let ids1 = action_ids(&get_clipboard_history_context_actions(&entry));
        let ids2 = action_ids(&get_clipboard_history_context_actions(&entry));
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn cat26_ai_ordering_deterministic() {
        let ids1 = action_ids(&get_ai_command_bar_actions());
        let ids2 = action_ids(&get_ai_command_bar_actions());
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn cat26_notes_ordering_deterministic() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let ids1 = action_ids(&get_notes_command_bar_actions(&info));
        let ids2 = action_ids(&get_notes_command_bar_actions(&info));
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn cat26_path_ordering_deterministic() {
        let info = PathInfo {
            path: "/test".into(),
            name: "test".into(),
            is_dir: true,
        };
        let ids1 = action_ids(&get_path_context_actions(&info));
        let ids2 = action_ids(&get_path_context_actions(&info));
        assert_eq!(ids1, ids2);
    }

    // =========================================================================
    // 27. Action builder chaining
    // =========================================================================

    #[test]
    fn cat27_with_icon_preserves_shortcut() {
        let action = Action::new("x", "X", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘X")
            .with_icon(IconName::Copy);
        assert_eq!(action.shortcut, Some("⌘X".to_string()));
        assert_eq!(action.icon, Some(IconName::Copy));
    }

    #[test]
    fn cat27_with_section_preserves_icon() {
        let action = Action::new("x", "X", None, ActionCategory::ScriptContext)
            .with_icon(IconName::Star)
            .with_section("Test");
        assert_eq!(action.icon, Some(IconName::Star));
        assert_eq!(action.section, Some("Test".to_string()));
    }

    #[test]
    fn cat27_full_chain_preserves_all() {
        let action = Action::new(
            "test",
            "Test",
            Some("Desc".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘T")
        .with_icon(IconName::Plus)
        .with_section("Section");
        assert_eq!(action.id, "test");
        assert_eq!(action.title, "Test");
        assert_eq!(action.description, Some("Desc".to_string()));
        assert_eq!(action.shortcut, Some("⌘T".to_string()));
        assert_eq!(action.icon, Some(IconName::Plus));
        assert_eq!(action.section, Some("Section".to_string()));
    }

    // =========================================================================
    // 28. Note switcher description rendering
    // =========================================================================

    #[test]
    fn cat28_note_switcher_preview_with_time() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n1".into(),
            title: "T".into(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: "Hello world".into(),
            relative_time: "2m ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.contains("Hello world"));
        assert!(desc.contains("2m ago"));
        assert!(desc.contains("·"));
    }

    #[test]
    fn cat28_note_switcher_empty_preview_shows_char_count() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n2".into(),
            title: "T".into(),
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
    fn cat28_note_switcher_singular_char() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n3".into(),
            title: "T".into(),
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
    fn cat28_note_switcher_truncation_at_61() {
        let long_preview = "a".repeat(61);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n4".into(),
            title: "T".into(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: long_preview,
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.ends_with('…'));
    }

    #[test]
    fn cat28_note_switcher_no_truncation_at_60() {
        let exact_preview = "b".repeat(60);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n5".into(),
            title: "T".into(),
            char_count: 60,
            is_current: false,
            is_pinned: false,
            preview: exact_preview.clone(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(!desc.contains('…'));
        assert_eq!(desc.as_str(), exact_preview.as_str());
    }

    #[test]
    fn cat28_note_switcher_empty_preview_with_time() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n6".into(),
            title: "T".into(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "3h ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "3h ago");
    }

    // =========================================================================
    // 29. File context title includes quoted name
    // =========================================================================

    #[test]
    fn cat29_file_title_includes_filename() {
        let info = FileInfo {
            path: "/test/report.pdf".into(),
            name: "report.pdf".into(),
            file_type: crate::file_search::FileType::Document,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        assert!(actions[0].title.contains("report.pdf"));
    }

    #[test]
    fn cat29_file_dir_title_includes_dirname() {
        let info = FileInfo {
            path: "/test/MyDir".into(),
            name: "MyDir".into(),
            file_type: crate::file_search::FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&info);
        assert!(actions[0].title.contains("MyDir"));
    }

    #[test]
    fn cat29_file_title_has_quotes() {
        let info = FileInfo {
            path: "/test/file.txt".into(),
            name: "file.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        assert!(actions[0].title.contains('"'));
    }

    // =========================================================================
    // 30. Non-empty id and title for all contexts
    // =========================================================================

    #[test]
    fn cat30_script_nonempty_id_title() {
        let script = ScriptInfo::new("test", "/test.ts");
        for action in &get_script_context_actions(&script) {
            assert!(!action.id.is_empty(), "Action should have non-empty id");
            assert!(
                !action.title.is_empty(),
                "Action {} should have non-empty title",
                action.id
            );
        }
    }

    #[test]
    fn cat30_clipboard_nonempty_id_title() {
        let entry = ClipboardEntryInfo {
            id: "c".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "t".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        for action in &get_clipboard_history_context_actions(&entry) {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }

    #[test]
    fn cat30_ai_nonempty_id_title() {
        for action in &get_ai_command_bar_actions() {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }

    #[test]
    fn cat30_path_nonempty_id_title() {
        let info = PathInfo {
            path: "/test".into(),
            name: "test".into(),
            is_dir: false,
        };
        for action in &get_path_context_actions(&info) {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }

    #[test]
    fn cat30_notes_nonempty_id_title() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        for action in &get_notes_command_bar_actions(&info) {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }

    #[test]
    fn cat30_file_nonempty_id_title() {
        let info = FileInfo {
            path: "/test/f.txt".into(),
            name: "f.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        for action in &get_file_context_actions(&info) {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }
