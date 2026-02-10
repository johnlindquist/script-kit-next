    #[test]
    fn cat20_paste_no_app() {
        let entry = ClipboardEntryInfo {
            id: "p1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions[0].title, "Paste to Active App");
    }

    #[test]
    fn cat20_paste_with_app() {
        let entry = ClipboardEntryInfo {
            id: "p2".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: Some("Safari".into()),
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions[0].title, "Paste to Safari");
    }

    #[test]
    fn cat20_paste_with_unicode_app() {
        let entry = ClipboardEntryInfo {
            id: "p3".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: Some("日本語エディタ".into()),
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions[0].title, "Paste to 日本語エディタ");
    }

    #[test]
    fn cat20_paste_with_empty_string_app() {
        let entry = ClipboardEntryInfo {
            id: "p4".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: Some(String::new()),
        };
        let actions = get_clipboard_history_context_actions(&entry);
        // Some("") → "Paste to " (empty name)
        assert_eq!(actions[0].title, "Paste to ");
    }

    // =========================================================================
    // cat21: Clipboard pin/unpin toggle
    // =========================================================================

    #[test]
    fn cat21_unpinned_shows_pin() {
        let entry = ClipboardEntryInfo {
            id: "u1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"clipboard_pin".to_string()));
        assert!(!ids.contains(&"clipboard_unpin".to_string()));
    }

    #[test]
    fn cat21_pinned_shows_unpin() {
        let entry = ClipboardEntryInfo {
            id: "u2".into(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"clipboard_unpin".to_string()));
        assert!(!ids.contains(&"clipboard_pin".to_string()));
    }

    #[test]
    fn cat21_pin_unpin_same_shortcut() {
        let pin_entry = ClipboardEntryInfo {
            id: "s1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let unpin_entry = ClipboardEntryInfo {
            id: "s2".into(),
            content_type: ContentType::Text,
            pinned: true,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let pin_actions = get_clipboard_history_context_actions(&pin_entry);
        let unpin_actions = get_clipboard_history_context_actions(&unpin_entry);
        let pin_sc = pin_actions
            .iter()
            .find(|a| a.id == "clipboard_pin")
            .unwrap()
            .shortcut
            .as_deref();
        let unpin_sc = unpin_actions
            .iter()
            .find(|a| a.id == "clipboard_unpin")
            .unwrap()
            .shortcut
            .as_deref();
        assert_eq!(pin_sc, unpin_sc, "Pin and Unpin share same shortcut");
        assert_eq!(pin_sc, Some("⇧⌘P"));
    }

    // =========================================================================
    // cat22: Clipboard destructive actions always last three
    // =========================================================================

    #[test]
    fn cat22_text_last_three_destructive() {
        let entry = ClipboardEntryInfo {
            id: "d1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
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
    fn cat22_image_last_three_destructive() {
        let entry = ClipboardEntryInfo {
            id: "d2".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
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
    fn cat22_paste_always_first() {
        let entry = ClipboardEntryInfo {
            id: "d3".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions[0].id, "clipboard_paste");
    }

    #[test]
    fn cat22_copy_always_second() {
        let entry = ClipboardEntryInfo {
            id: "d4".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions[1].id, "clipboard_copy");
    }

    // =========================================================================
    // cat23: Action lowercase caching
    // =========================================================================

    #[test]
    fn cat23_title_lower_cached() {
        let action = Action::new("test", "My Title", None, ActionCategory::ScriptContext);
        assert_eq!(action.title_lower, "my title");
    }

    #[test]
    fn cat23_description_lower_cached() {
        let action = Action::new(
            "test",
            "T",
            Some("My Description".into()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(action.description_lower.as_deref(), Some("my description"));
    }

    #[test]
    fn cat23_description_none_lower_none() {
        let action = Action::new("test", "T", None, ActionCategory::ScriptContext);
        assert_eq!(action.description_lower, None);
    }

    #[test]
    fn cat23_shortcut_lower_none_initially() {
        let action = Action::new("test", "T", None, ActionCategory::ScriptContext);
        assert_eq!(action.shortcut_lower, None);
    }

    #[test]
    fn cat23_shortcut_lower_set_after_with_shortcut() {
        let action =
            Action::new("test", "T", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧C");
        assert_eq!(action.shortcut_lower.as_deref(), Some("⌘⇧c"));
    }

    #[test]
    fn cat23_title_lower_unicode() {
        let action = Action::new("test", "Café Résumé", None, ActionCategory::ScriptContext);
        assert_eq!(action.title_lower, "café résumé");
    }

    // =========================================================================
    // cat24: AI command bar total count and all have icons
    // =========================================================================

    #[test]
    fn cat24_ai_total_12() {
        assert_eq!(get_ai_command_bar_actions().len(), 12);
    }

    #[test]
    fn cat24_ai_all_have_icons() {
        for action in &get_ai_command_bar_actions() {
            assert!(
                action.icon.is_some(),
                "AI action {} should have icon",
                action.id
            );
        }
    }

    #[test]
    fn cat24_ai_all_have_sections() {
        for action in &get_ai_command_bar_actions() {
            assert!(
                action.section.is_some(),
                "AI action {} should have section",
                action.id
            );
        }
    }

    #[test]
    fn cat24_ai_6_unique_sections() {
        let actions = get_ai_command_bar_actions();
        let sections: HashSet<&str> = actions
            .iter()
            .filter_map(|a| a.section.as_deref())
            .collect();
        assert_eq!(sections.len(), 6);
    }

    #[test]
    fn cat24_ai_ids_unique() {
        let actions = get_ai_command_bar_actions();
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len(), "AI action IDs should be unique");
    }

    // =========================================================================
    // cat25: Notes format action shortcut and icon
    // =========================================================================

    #[test]
    fn cat25_notes_format_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let format = actions.iter().find(|a| a.id == "format").unwrap();
        assert_eq!(format.shortcut.as_deref(), Some("⇧⌘T"));
    }

    #[test]
    fn cat25_notes_format_icon_code() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let format = actions.iter().find(|a| a.id == "format").unwrap();
        assert_eq!(format.icon, Some(IconName::Code));
    }

    #[test]
    fn cat25_notes_format_section_edit() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let format = actions.iter().find(|a| a.id == "format").unwrap();
        assert_eq!(format.section.as_deref(), Some("Edit"));
    }

    // =========================================================================
    // cat26: File context common actions always present
    // =========================================================================

    #[test]
    fn cat26_file_has_reveal() {
        let fi = FileInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
            file_type: crate::file_search::FileType::Document,
        };
        let ids = action_ids(&get_file_context_actions(&fi));
        assert!(ids.contains(&"reveal_in_finder".to_string()));
    }

    #[test]
    fn cat26_file_has_copy_path() {
        let fi = FileInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
            file_type: crate::file_search::FileType::Document,
        };
        let ids = action_ids(&get_file_context_actions(&fi));
        assert!(ids.contains(&"copy_path".to_string()));
    }

    #[test]
    fn cat26_file_has_copy_filename() {
        let fi = FileInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
            file_type: crate::file_search::FileType::Document,
        };
        let ids = action_ids(&get_file_context_actions(&fi));
        assert!(ids.contains(&"copy_filename".to_string()));
    }

    #[test]
    fn cat26_dir_has_reveal() {
        let fi = FileInfo {
            name: "d".into(),
            path: "/d".into(),
            is_dir: true,
            file_type: crate::file_search::FileType::Directory,
        };
        let ids = action_ids(&get_file_context_actions(&fi));
        assert!(ids.contains(&"reveal_in_finder".to_string()));
    }

    #[test]
    fn cat26_dir_has_copy_path() {
        let fi = FileInfo {
            name: "d".into(),
            path: "/d".into(),
            is_dir: true,
            file_type: crate::file_search::FileType::Directory,
        };
        let ids = action_ids(&get_file_context_actions(&fi));
        assert!(ids.contains(&"copy_path".to_string()));
    }

    // =========================================================================
    // cat27: Script context shortcut/alias dynamic action count
    // =========================================================================

    #[test]
    fn cat27_no_shortcut_no_alias_count() {
        let s = ScriptInfo::new("test", "/p");
        let actions = get_script_context_actions(&s);
        // run, edit, add_shortcut, add_alias, view_logs, reveal, copy_path, copy_content, copy_deeplink = 9
        assert_eq!(actions.len(), 9);
    }

    #[test]
    fn cat27_with_shortcut_count() {
        let s = ScriptInfo::with_shortcut("test", "/p", Some("cmd+t".into()));
        let actions = get_script_context_actions(&s);
        // run, edit, update_shortcut, remove_shortcut, add_alias, view_logs, reveal, copy_path, copy_content, copy_deeplink = 10
        assert_eq!(actions.len(), 10);
    }

    #[test]
    fn cat27_with_both_count() {
        let s = ScriptInfo::with_shortcut_and_alias(
            "test",
            "/p",
            Some("cmd+t".into()),
            Some("ts".into()),
        );
        let actions = get_script_context_actions(&s);
        // run, edit, update_shortcut, remove_shortcut, update_alias, remove_alias, view_logs, reveal, copy_path, copy_content, copy_deeplink = 11
        assert_eq!(actions.len(), 11);
    }

    #[test]
    fn cat27_frecency_adds_one() {
        let s = ScriptInfo::new("test", "/p").with_frecency(true, Some("/f".into()));
        let actions = get_script_context_actions(&s);
        // 9 + 1 (reset_ranking) = 10
        assert_eq!(actions.len(), 10);
    }

    // =========================================================================
    // cat28: has_action=false invariant for all built-ins
    // =========================================================================

    #[test]
    fn cat28_script_has_action_false() {
        let s = ScriptInfo::new("t", "/p");
        for action in &get_script_context_actions(&s) {
            assert!(
                !action.has_action,
                "Script action {} should have has_action=false",
                action.id
            );
        }
    }

