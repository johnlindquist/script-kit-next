    #[test]
    fn notes_actions_deterministic() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions1 = get_notes_command_bar_actions(&info);
        let actions2 = get_notes_command_bar_actions(&info);
        let a1 = action_ids(&actions1);
        let a2 = action_ids(&actions2);
        assert_eq!(a1, a2, "Notes actions should be deterministic");
    }

    // ============================================================
    // Cross-cutting: non-empty titles and IDs
    // ============================================================

    #[test]
    fn script_all_actions_have_nonempty_id_and_title() {
        let script = ScriptInfo::new("test", "/path/to/test.ts");
        for action in &get_script_context_actions(&script) {
            assert!(!action.id.is_empty(), "Action ID should not be empty");
            assert!(!action.title.is_empty(), "Action title should not be empty");
        }
    }

    #[test]
    fn clipboard_all_actions_have_nonempty_id_and_title() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        for action in &get_clipboard_history_context_actions(&entry) {
            assert!(!action.id.is_empty(), "Action ID should not be empty");
            assert!(!action.title.is_empty(), "Action title should not be empty");
        }
    }

    #[test]
    fn ai_all_actions_have_nonempty_id_and_title() {
        for action in &get_ai_command_bar_actions() {
            assert!(!action.id.is_empty(), "Action ID should not be empty");
            assert!(!action.title.is_empty(), "Action title should not be empty");
        }
    }

    // ============================================================
    // Fuzzy match edge cases
    // ============================================================

    #[test]
    fn fuzzy_match_empty_needle_always_true() {
        assert!(ActionsDialog::fuzzy_match("anything", ""));
    }

    #[test]
    fn fuzzy_match_empty_haystack_nonempty_needle_false() {
        assert!(!ActionsDialog::fuzzy_match("", "a"));
    }

    #[test]
    fn fuzzy_match_both_empty_true() {
        assert!(ActionsDialog::fuzzy_match("", ""));
    }

    #[test]
    fn fuzzy_match_exact_match_true() {
        assert!(ActionsDialog::fuzzy_match("hello", "hello"));
    }

    #[test]
    fn fuzzy_match_subsequence_true() {
        assert!(ActionsDialog::fuzzy_match("hello world", "hlo"));
    }

    #[test]
    fn fuzzy_match_no_subsequence_false() {
        assert!(!ActionsDialog::fuzzy_match("hello", "z"));
    }

    #[test]
    fn fuzzy_match_needle_longer_than_haystack_false() {
        assert!(!ActionsDialog::fuzzy_match("hi", "hello"));
    }

    // ============================================================
    // ActionCategory invariants
    // ============================================================

    #[test]
    fn all_script_actions_are_script_context() {
        let script = ScriptInfo::new("test", "/path/to/test.ts");
        for action in &get_script_context_actions(&script) {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

    #[test]
    fn all_clipboard_actions_are_script_context() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        for action in &get_clipboard_history_context_actions(&entry) {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

    #[test]
    fn all_ai_actions_are_script_context() {
        for action in &get_ai_command_bar_actions() {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

    #[test]
    fn all_path_actions_are_script_context() {
        let info = PathInfo::new("test", "/tmp/test", false);
        for action in &get_path_context_actions(&info) {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

    #[test]
    fn all_file_actions_are_script_context() {
        let info = FileInfo {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        for action in &get_file_context_actions(&info) {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }
