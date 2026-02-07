    #[test]
    fn cat28_notes_ids_unique() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), actions.len(), "Notes action IDs must be unique");
    }

    // =========================================================================
    // 29. has_action=false for all built-in actions
    // =========================================================================

    #[test]
    fn cat29_script_all_has_action_false() {
        let s = ScriptInfo::new("t", "/p");
        for a in &get_script_context_actions(&s) {
            assert!(!a.has_action, "{} should be false", a.id);
        }
    }

    #[test]
    fn cat29_clipboard_all_has_action_false() {
        for a in &get_clipboard_history_context_actions(&make_text_entry()) {
            assert!(!a.has_action, "{} should be false", a.id);
        }
    }

    #[test]
    fn cat29_ai_all_has_action_false() {
        for a in &get_ai_command_bar_actions() {
            assert!(!a.has_action, "{} should be false", a.id);
        }
    }

    #[test]
    fn cat29_path_all_has_action_false() {
        let info = PathInfo {
            path: "/t".into(),
            name: "t".into(),
            is_dir: false,
        };
        for a in &get_path_context_actions(&info) {
            assert!(!a.has_action, "{} should be false", a.id);
        }
    }

    #[test]
    fn cat29_file_all_has_action_false() {
        let info = FileInfo {
            path: "/t".into(),
            name: "t".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        for a in &get_file_context_actions(&info) {
            assert!(!a.has_action, "{} should be false", a.id);
        }
    }

    #[test]
    fn cat29_notes_all_has_action_false() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        for a in &get_notes_command_bar_actions(&info) {
            assert!(!a.has_action, "{} should be false", a.id);
        }
    }

    // =========================================================================
    // 30. Ordering determinism
    // =========================================================================

    #[test]
    fn cat30_script_ordering_deterministic() {
        let s = ScriptInfo::new("t", "/p");
        let a = action_ids(&get_script_context_actions(&s));
        let b = action_ids(&get_script_context_actions(&s));
        assert_eq!(a, b);
    }

    #[test]
    fn cat30_clipboard_ordering_deterministic() {
        let a = action_ids(&get_clipboard_history_context_actions(&make_text_entry()));
        let b = action_ids(&get_clipboard_history_context_actions(&make_text_entry()));
        assert_eq!(a, b);
    }

    #[test]
    fn cat30_ai_ordering_deterministic() {
        let a = action_ids(&get_ai_command_bar_actions());
        let b = action_ids(&get_ai_command_bar_actions());
        assert_eq!(a, b);
    }

    #[test]
    fn cat30_notes_ordering_deterministic() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let a = action_ids(&get_notes_command_bar_actions(&info));
        let b = action_ids(&get_notes_command_bar_actions(&info));
        assert_eq!(a, b);
    }

    #[test]
    fn cat30_path_ordering_deterministic() {
        let info = PathInfo {
            path: "/t".into(),
            name: "t".into(),
            is_dir: false,
        };
        let a = action_ids(&get_path_context_actions(&info));
        let b = action_ids(&get_path_context_actions(&info));
        assert_eq!(a, b);
    }
