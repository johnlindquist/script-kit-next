    #[test]
    fn has_action_false_for_path() {
        let path = PathInfo {
            path: "/test".to_string(),
            name: "test".to_string(),
            is_dir: false,
        };
        for action in &get_path_context_actions(&path) {
            assert!(
                !action.has_action,
                "Built-in action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn has_action_false_for_file() {
        let file = FileInfo {
            path: "/f.txt".to_string(),
            name: "f.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        for action in &get_file_context_actions(&file) {
            assert!(
                !action.has_action,
                "Built-in action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn has_action_false_for_notes() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        for action in &get_notes_command_bar_actions(&info) {
            assert!(
                !action.has_action,
                "Built-in action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn has_action_false_for_chat() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "m".to_string(),
                display_name: "M".to_string(),
                provider: "P".to_string(),
            }],
            has_messages: true,
            has_response: true,
        };
        for action in &get_chat_context_actions(&info) {
            assert!(
                !action.has_action,
                "Built-in action '{}' should have has_action=false",
                action.id
            );
        }
    }

    // ============================================================
    // Additional: Non-empty title and ID for all contexts
    // ============================================================

    #[test]
    fn nonempty_title_id_script() {
        let script = ScriptInfo::new("s", "/s.ts");
        for action in &get_script_context_actions(&script) {
            assert!(!action.id.is_empty(), "Action should have non-empty ID");
            assert!(
                !action.title.is_empty(),
                "Action '{}' should have non-empty title",
                action.id
            );
        }
    }

    #[test]
    fn nonempty_title_id_clipboard() {
        let entry = make_text_entry(false, None);
        for action in &get_clipboard_history_context_actions(&entry) {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }

    #[test]
    fn nonempty_title_id_ai() {
        for action in &get_ai_command_bar_actions() {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }

    #[test]
    fn nonempty_title_id_notes() {
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
    fn nonempty_title_id_path() {
        let path = PathInfo {
            path: "/test".to_string(),
            name: "test".to_string(),
            is_dir: true,
        };
        for action in &get_path_context_actions(&path) {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }

    #[test]
    fn nonempty_title_id_file() {
        let file = FileInfo {
            path: "/f".to_string(),
            name: "f".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        for action in &get_file_context_actions(&file) {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }

    // ============================================================
    // Additional: Note switcher icons and sections
    // ============================================================

    #[test]
    fn note_switcher_pinned_star_icon() {
        let note = make_note("n1", "Note", 10, false, true, "", "");
        let actions = get_note_switcher_actions(&[note]);
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
    }

    #[test]
    fn note_switcher_current_check_icon() {
        let note = make_note("n1", "Note", 10, true, false, "", "");
        let actions = get_note_switcher_actions(&[note]);
        assert_eq!(actions[0].icon, Some(IconName::Check));
        assert_eq!(actions[0].section.as_deref(), Some("Recent"));
    }

    #[test]
    fn note_switcher_default_file_icon() {
        let note = make_note("n1", "Note", 10, false, false, "", "");
        let actions = get_note_switcher_actions(&[note]);
        assert_eq!(actions[0].icon, Some(IconName::File));
        assert_eq!(actions[0].section.as_deref(), Some("Recent"));
    }

    #[test]
    fn note_switcher_current_gets_bullet_prefix() {
        let note = make_note("n1", "My Note", 10, true, false, "", "");
        let actions = get_note_switcher_actions(&[note]);
        assert!(
            actions[0].title.starts_with("• "),
            "Current note should have bullet prefix"
        );
    }

    #[test]
    fn note_switcher_not_current_no_bullet() {
        let note = make_note("n1", "My Note", 10, false, false, "", "");
        let actions = get_note_switcher_actions(&[note]);
        assert!(
            !actions[0].title.starts_with("• "),
            "Non-current note should not have bullet"
        );
    }

    #[test]
    fn note_switcher_id_format() {
        let note = make_note("abc-123", "Note", 10, false, false, "", "");
        let actions = get_note_switcher_actions(&[note]);
        assert_eq!(actions[0].id, "note_abc-123");
    }

    #[test]
    fn note_switcher_empty_shows_placeholder() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].id, "no_notes");
        assert!(actions[0].title.contains("No notes"));
    }

    #[test]
    fn note_switcher_pinned_takes_priority_over_current() {
        let note = make_note("n1", "Note", 10, true, true, "", "");
        let actions = get_note_switcher_actions(&[note]);
        // Pinned icon takes priority
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
        // But still gets the "Pinned" section
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
        // And still gets bullet prefix because is_current
        assert!(actions[0].title.starts_with("• "));
    }
