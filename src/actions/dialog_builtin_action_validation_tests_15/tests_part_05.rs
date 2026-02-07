    #[test]
    fn cat28_clipboard_has_action_false() {
        let e = ClipboardEntryInfo {
            id: "c".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        for action in &get_clipboard_history_context_actions(&e) {
            assert!(
                !action.has_action,
                "Clipboard action {} should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn cat28_ai_has_action_false() {
        for action in &get_ai_command_bar_actions() {
            assert!(
                !action.has_action,
                "AI action {} should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn cat28_path_has_action_false() {
        let pi = PathInfo {
            name: "x".into(),
            path: "/x".into(),
            is_dir: false,
        };
        for action in &get_path_context_actions(&pi) {
            assert!(
                !action.has_action,
                "Path action {} should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn cat28_notes_has_action_false() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        for action in &get_notes_command_bar_actions(&info) {
            assert!(
                !action.has_action,
                "Notes action {} should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn cat28_file_has_action_false() {
        let fi = FileInfo {
            name: "f".into(),
            path: "/f".into(),
            is_dir: false,
            file_type: crate::file_search::FileType::Document,
        };
        for action in &get_file_context_actions(&fi) {
            assert!(
                !action.has_action,
                "File action {} should have has_action=false",
                action.id
            );
        }
    }

    // =========================================================================
    // cat29: ID uniqueness across contexts
    // =========================================================================

    #[test]
    fn cat29_script_ids_unique() {
        let s =
            ScriptInfo::with_shortcut_and_alias("t", "/p", Some("cmd+t".into()), Some("al".into()));
        let actions = get_script_context_actions(&s);
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn cat29_clipboard_ids_unique() {
        let e = ClipboardEntryInfo {
            id: "c".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&e);
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn cat29_path_ids_unique() {
        let pi = PathInfo {
            name: "x".into(),
            path: "/x".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&pi);
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn cat29_file_ids_unique() {
        let fi = FileInfo {
            name: "f".into(),
            path: "/f".into(),
            is_dir: false,
            file_type: crate::file_search::FileType::Document,
        };
        let actions = get_file_context_actions(&fi);
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn cat29_notes_ids_unique() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn cat29_note_switcher_ids_unique() {
        let notes = vec![
            NoteSwitcherNoteInfo {
                id: "a".into(),
                title: "A".into(),
                char_count: 1,
                is_current: false,
                is_pinned: false,
                preview: String::new(),
                relative_time: String::new(),
            },
            NoteSwitcherNoteInfo {
                id: "b".into(),
                title: "B".into(),
                char_count: 1,
                is_current: false,
                is_pinned: false,
                preview: String::new(),
                relative_time: String::new(),
            },
        ];
        let actions = get_note_switcher_actions(&notes);
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    // =========================================================================
    // cat30: Non-empty id and title invariant
    // =========================================================================

    #[test]
    fn cat30_script_nonempty_id_title() {
        let s = ScriptInfo::new("t", "/p");
        for action in &get_script_context_actions(&s) {
            assert!(!action.id.is_empty(), "ID should not be empty");
            assert!(!action.title.is_empty(), "Title should not be empty");
        }
    }

    #[test]
    fn cat30_clipboard_nonempty_id_title() {
        let e = ClipboardEntryInfo {
            id: "c".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        for action in &get_clipboard_history_context_actions(&e) {
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
        let pi = PathInfo {
            name: "x".into(),
            path: "/x".into(),
            is_dir: false,
        };
        for action in &get_path_context_actions(&pi) {
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
        let fi = FileInfo {
            name: "f".into(),
            path: "/f".into(),
            is_dir: false,
            file_type: crate::file_search::FileType::Document,
        };
        for action in &get_file_context_actions(&fi) {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }
