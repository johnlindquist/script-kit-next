    #[test]
    fn action_no_description_lower_is_none() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext);
        assert!(action.description_lower.is_none());
    }

    #[test]
    fn action_shortcut_lower_set_by_with_shortcut() {
        let action =
            Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧C");
        assert_eq!(action.shortcut_lower, Some("⌘⇧c".to_string()));
    }

    #[test]
    fn action_shortcut_lower_none_without_shortcut() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext);
        assert!(action.shortcut_lower.is_none());
    }

    // ============================================================
    // Additional: non-empty title/ID for all contexts
    // ============================================================

    #[test]
    fn all_script_actions_have_nonempty_title_and_id() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        for action in &get_script_context_actions(&script) {
            assert!(!action.id.is_empty(), "Action ID should not be empty");
            assert!(!action.title.is_empty(), "Action title should not be empty");
        }
    }

    #[test]
    fn all_clipboard_actions_have_nonempty_title_and_id() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        for action in &get_clipboard_history_context_actions(&entry) {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }

    #[test]
    fn all_ai_actions_have_nonempty_title_and_id() {
        for action in &get_ai_command_bar_actions() {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }

    // ============================================================
    // Additional: ordering determinism
    // ============================================================

    #[test]
    fn script_actions_ordering_deterministic() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions1 = get_script_context_actions(&script);
        let actions2 = get_script_context_actions(&script);
        let ids1 = action_ids(&actions1);
        let ids2 = action_ids(&actions2);
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn clipboard_actions_ordering_deterministic() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions1 = get_clipboard_history_context_actions(&entry);
        let actions2 = get_clipboard_history_context_actions(&entry);
        let ids1 = action_ids(&actions1);
        let ids2 = action_ids(&actions2);
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn ai_actions_ordering_deterministic() {
        let actions1 = get_ai_command_bar_actions();
        let actions2 = get_ai_command_bar_actions();
        let ids1 = action_ids(&actions1);
        let ids2 = action_ids(&actions2);
        assert_eq!(ids1, ids2);
    }

    // ============================================================
    // Additional: Clipboard destructive ordering
    // ============================================================

    #[test]
    fn clipboard_destructive_always_last_three() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let len = actions.len();
        assert!(len >= 3);
        assert_eq!(actions[len - 3].id, "clipboard_delete");
        assert_eq!(actions[len - 2].id, "clipboard_delete_multiple");
        assert_eq!(actions[len - 1].id, "clipboard_delete_all");
    }

    #[test]
    fn clipboard_paste_always_first() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions[0].id, "clipboard_paste");
    }

    #[test]
    fn clipboard_copy_always_second() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        assert_eq!(actions[1].id, "clipboard_copy");
    }

    // ============================================================
    // Additional: ActionCategory enum
    // ============================================================

    #[test]
    fn all_script_context_actions_are_script_context_category() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        for action in &get_script_context_actions(&script) {
            assert_eq!(
                action.category,
                ActionCategory::ScriptContext,
                "Action '{}' should be ScriptContext",
                action.id
            );
        }
    }

    #[test]
    fn all_clipboard_actions_are_script_context_category() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        for action in &get_clipboard_history_context_actions(&entry) {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

    #[test]
    fn all_ai_actions_are_script_context_category() {
        for action in &get_ai_command_bar_actions() {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

    #[test]
    fn all_path_actions_are_script_context_category() {
        let path = PathInfo {
            name: "test".to_string(),
            path: "/test".to_string(),
            is_dir: false,
        };
        for action in &get_path_context_actions(&path) {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }
