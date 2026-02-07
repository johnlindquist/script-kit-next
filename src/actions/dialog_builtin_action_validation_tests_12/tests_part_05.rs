    #[test]
    fn cat28_notes_ids_unique() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), actions.len());
    }

    // =========================================================================
    // 29. has_action=false invariant for all built-ins
    // =========================================================================

    #[test]
    fn cat29_script_has_action_false() {
        for action in &get_script_context_actions(&ScriptInfo::new("t", "/t.ts")) {
            assert!(
                !action.has_action,
                "{} should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn cat29_clipboard_has_action_false() {
        for action in &get_clipboard_history_context_actions(&make_text_entry()) {
            assert!(
                !action.has_action,
                "{} should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn cat29_ai_has_action_false() {
        for action in &get_ai_command_bar_actions() {
            assert!(
                !action.has_action,
                "{} should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn cat29_path_has_action_false() {
        let info = PathInfo {
            path: "/t".into(),
            name: "t".into(),
            is_dir: false,
        };
        for action in &get_path_context_actions(&info) {
            assert!(
                !action.has_action,
                "{} should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn cat29_file_has_action_false() {
        let info = FileInfo {
            path: "/t".into(),
            name: "t".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        for action in &get_file_context_actions(&info) {
            assert!(
                !action.has_action,
                "{} should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn cat29_notes_has_action_false() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        for action in &get_notes_command_bar_actions(&info) {
            assert!(
                !action.has_action,
                "{} should have has_action=false",
                action.id
            );
        }
    }

    // =========================================================================
    // 30. Ordering determinism
    // =========================================================================

    #[test]
    fn cat30_script_ordering_deterministic() {
        let s = ScriptInfo::new("t", "/t.ts");
        let a = action_ids(&get_script_context_actions(&s));
        let b = action_ids(&get_script_context_actions(&s));
        assert_eq!(a, b);
    }

    #[test]
    fn cat30_clipboard_ordering_deterministic() {
        let e = make_text_entry();
        let a = action_ids(&get_clipboard_history_context_actions(&e));
        let b = action_ids(&get_clipboard_history_context_actions(&e));
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
