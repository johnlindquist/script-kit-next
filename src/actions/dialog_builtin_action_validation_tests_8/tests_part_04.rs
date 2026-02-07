    #[test]
    fn script_with_frecency_has_one_more_action() {
        let script = ScriptInfo::new("test", "/path/to/test.ts");
        let script_with_frecency = ScriptInfo::new("test", "/path/to/test.ts")
            .with_frecency(true, Some("/path".to_string()));
        let actions = get_script_context_actions(&script);
        let actions_f = get_script_context_actions(&script_with_frecency);
        assert_eq!(
            actions_f.len(),
            actions.len() + 1,
            "Frecency adds exactly 1 action (reset_ranking)"
        );
    }

    // ============================================================
    // 17. Action builder chaining order independence
    // ============================================================

    #[test]
    fn with_icon_then_section_same_as_reverse() {
        let a1 = Action::new("id", "Title", None, ActionCategory::ScriptContext)
            .with_icon(IconName::Copy)
            .with_section("Section");
        let a2 = Action::new("id", "Title", None, ActionCategory::ScriptContext)
            .with_section("Section")
            .with_icon(IconName::Copy);
        assert_eq!(a1.icon, a2.icon);
        assert_eq!(a1.section, a2.section);
        assert_eq!(a1.id, a2.id);
        assert_eq!(a1.title, a2.title);
    }

    #[test]
    fn with_shortcut_then_icon_then_section() {
        let a = Action::new("id", "Title", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘C")
            .with_icon(IconName::Copy)
            .with_section("Section");
        assert_eq!(a.shortcut, Some("⌘C".to_string()));
        assert_eq!(a.icon, Some(IconName::Copy));
        assert_eq!(a.section, Some("Section".to_string()));
    }

    #[test]
    fn with_shortcut_opt_none_preserves_fields() {
        let a = Action::new(
            "id",
            "Title",
            Some("Desc".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::Plus)
        .with_shortcut_opt(None);
        assert_eq!(a.icon, Some(IconName::Plus));
        assert!(a.shortcut.is_none());
        assert_eq!(a.description, Some("Desc".to_string()));
    }

    #[test]
    fn with_shortcut_sets_lowercase_cache() {
        let a =
            Action::new("id", "Title", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧C");
        assert_eq!(a.shortcut_lower, Some("⌘⇧c".to_string()));
    }

    // ============================================================
    // 18. Clipboard destructive action ordering stability
    // ============================================================

    #[test]
    fn clipboard_destructive_actions_always_last_three() {
        let entries = vec![
            make_clipboard_entry(ContentType::Text, false, None),
            make_clipboard_entry(ContentType::Text, true, Some("Chrome")),
            make_clipboard_entry(ContentType::Image, false, Some("Slack")),
            make_clipboard_entry(ContentType::Image, true, None),
        ];
        for entry in &entries {
            let actions = get_clipboard_history_context_actions(entry);
            let len = actions.len();
            assert!(len >= 3);
            // Last 3 should always be delete, delete_multiple, delete_all
            assert_eq!(
                actions[len - 3].id,
                "clipboard_delete",
                "Third from last should be clipboard_delete for {:?}",
                entry.content_type
            );
            assert_eq!(
                actions[len - 2].id,
                "clipboard_delete_multiple",
                "Second from last should be clipboard_delete_multiple"
            );
            assert_eq!(
                actions[len - 1].id,
                "clipboard_delete_all",
                "Last should be clipboard_delete_all"
            );
        }
    }

    #[test]
    fn clipboard_paste_always_first() {
        let entries = vec![
            make_clipboard_entry(ContentType::Text, false, None),
            make_clipboard_entry(ContentType::Image, true, Some("App")),
        ];
        for entry in &entries {
            let actions = get_clipboard_history_context_actions(entry);
            assert_eq!(
                actions[0].id, "clipboard_paste",
                "Paste should always be first"
            );
        }
    }

    #[test]
    fn clipboard_copy_always_second() {
        let entries = vec![
            make_clipboard_entry(ContentType::Text, false, None),
            make_clipboard_entry(ContentType::Image, false, None),
        ];
        for entry in &entries {
            let actions = get_clipboard_history_context_actions(entry);
            assert_eq!(
                actions[1].id, "clipboard_copy",
                "Copy should always be second"
            );
        }
    }

    // ============================================================
    // 19. File context title includes exact filename
    // ============================================================

    #[test]
    fn file_context_title_includes_filename() {
        let file_info = FileInfo {
            path: "/Users/test/report.pdf".to_string(),
            name: "report.pdf".to_string(),
            file_type: FileType::Document,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);
        let primary = &actions[0];
        assert!(
            primary.title.contains("report.pdf"),
            "Primary title '{}' should contain filename",
            primary.title
        );
    }

    #[test]
    fn file_context_dir_title_includes_dirname() {
        let file_info = FileInfo {
            path: "/Users/test/Documents".to_string(),
            name: "Documents".to_string(),
            file_type: FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&file_info);
        let primary = &actions[0];
        assert!(
            primary.title.contains("Documents"),
            "Primary title '{}' should contain dirname",
            primary.title
        );
    }

    #[test]
    fn file_context_all_have_descriptions() {
        let file_info = FileInfo {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);
        for action in &actions {
            assert!(
                action.description.is_some(),
                "File action '{}' should have a description",
                action.id
            );
        }
    }

    // ============================================================
    // 20. Notes info all-true/all-false edge cases
    // ============================================================

    #[test]
    fn notes_all_true_max_actions() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        // Should be the maximum action count
        assert!(
            actions.len() >= 10,
            "Full features should have >= 10 actions, got {}",
            actions.len()
        );
    }

    #[test]
    fn notes_all_false_min_actions() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        // Minimal: new_note + browse_notes
        assert_eq!(
            actions.len(),
            2,
            "Minimal should have exactly 2 actions, got {}",
            actions.len()
        );
    }

    #[test]
    fn notes_auto_sizing_disabled_adds_one() {
        let with_auto = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let without_auto = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let with_actions = get_notes_command_bar_actions(&with_auto);
        let without_actions = get_notes_command_bar_actions(&without_auto);
        assert_eq!(
            without_actions.len(),
            with_actions.len() + 1,
            "Disabled auto-sizing adds exactly 1 action"
        );
    }

    // ============================================================
    // 21. ScriptInfo agent flag interactions with frecency chaining
    // ============================================================

    #[test]
    fn agent_with_frecency_has_reset_ranking() {
        let mut agent = ScriptInfo::new("Agent", "/path/to/agent");
        agent.is_script = false;
        agent.is_agent = true;
        let agent = agent.with_frecency(true, Some("agent:path".to_string()));
        let actions = get_script_context_actions(&agent);
        assert!(actions.iter().any(|a| a.id == "reset_ranking"));
    }

    #[test]
    fn agent_frecency_preserves_agent_flag() {
        let mut agent = ScriptInfo::new("Agent", "/path/to/agent");
        agent.is_script = false;
        agent.is_agent = true;
        let agent = agent.with_frecency(true, Some("agent:path".to_string()));
        assert!(agent.is_agent);
        assert!(!agent.is_script);
        assert!(agent.is_suggested);
    }

    // ============================================================
    // 22. Agent actions: no view_logs, has copy_content
    // ============================================================

    #[test]
    fn agent_has_no_view_logs() {
        let mut agent = ScriptInfo::new("Agent", "/path/to/agent");
        agent.is_script = false;
        agent.is_agent = true;
        let actions = get_script_context_actions(&agent);
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
    }

    #[test]
    fn agent_has_copy_content() {
        let mut agent = ScriptInfo::new("Agent", "/path/to/agent");
        agent.is_script = false;
        agent.is_agent = true;
        let actions = get_script_context_actions(&agent);
        assert!(actions.iter().any(|a| a.id == "copy_content"));
    }

    #[test]
    fn agent_edit_title_says_agent() {
        let mut agent = ScriptInfo::new("Agent", "/path/to/agent");
        agent.is_script = false;
        agent.is_agent = true;
        let actions = get_script_context_actions(&agent);
        let edit = find_action(&actions, "edit_script").unwrap();
        assert_eq!(edit.title, "Edit Agent");
    }

    #[test]
    fn agent_has_reveal_and_copy_path() {
        let mut agent = ScriptInfo::new("Agent", "/path/to/agent");
        agent.is_script = false;
        agent.is_agent = true;
        let actions = get_script_context_actions(&agent);
        assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
        assert!(actions.iter().any(|a| a.id == "copy_path"));
    }

    // ============================================================
    // 23. Builtin with full optional fields
    // ============================================================

    #[test]
    fn builtin_with_shortcut_and_alias_and_frecency() {
        let builtin = ScriptInfo::with_all(
            "Clipboard History",
            "builtin:clipboard",
            false,
            "Open",
            Some("cmd+shift+c".to_string()),
            Some("ch".to_string()),
        )
        .with_frecency(true, Some("builtin:clipboard".to_string()));

        let actions = get_script_context_actions(&builtin);

        // Should have update/remove instead of add
        assert!(actions.iter().any(|a| a.id == "update_shortcut"));
        assert!(actions.iter().any(|a| a.id == "remove_shortcut"));
        assert!(actions.iter().any(|a| a.id == "update_alias"));
        assert!(actions.iter().any(|a| a.id == "remove_alias"));
        assert!(actions.iter().any(|a| a.id == "reset_ranking"));

        // Should NOT have script-specific actions
        assert!(!actions.iter().any(|a| a.id == "edit_script"));
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
    }

    #[test]
    fn builtin_primary_uses_custom_verb() {
        let builtin =
            ScriptInfo::with_action_verb("Clipboard History", "builtin:clipboard", false, "Open");
        let actions = get_script_context_actions(&builtin);
        assert_eq!(actions[0].title, "Open \"Clipboard History\"");
    }

    // ============================================================
    // 24. Path context dir vs file action count equality
    // ============================================================

    #[test]
    fn path_dir_file_action_count_equal() {
        let dir = PathInfo::new("dir", "/tmp/dir", true);
        let file = PathInfo::new("file", "/tmp/file", false);
        let dir_actions = get_path_context_actions(&dir);
        let file_actions = get_path_context_actions(&file);
        assert_eq!(dir_actions.len(), file_actions.len());
    }

    #[test]
    fn path_always_has_copy_path_and_copy_filename() {
        let dir = PathInfo::new("dir", "/tmp/dir", true);
        let file = PathInfo::new("file", "/tmp/file", false);
        for actions in [
            get_path_context_actions(&dir),
            get_path_context_actions(&file),
        ] {
            assert!(actions.iter().any(|a| a.id == "copy_path"));
            assert!(actions.iter().any(|a| a.id == "copy_filename"));
        }
    }

    // ============================================================
    // 25. Multiple scriptlet custom actions ordering
    // ============================================================

    #[test]
    fn scriptlet_multiple_custom_actions_maintain_order() {
        let script = ScriptInfo::scriptlet("Test", "/path/to/test.md", None, None);
        let mut scriptlet = Scriptlet::new(
            "Test".to_string(),
            "bash".to_string(),
            "echo main".to_string(),
        );
        scriptlet.actions = vec![
            ScriptletAction {
                name: "First Action".to_string(),
                command: "first".to_string(),
                tool: "bash".to_string(),
                code: "echo first".to_string(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
            ScriptletAction {
                name: "Second Action".to_string(),
                command: "second".to_string(),
                tool: "bash".to_string(),
                code: "echo second".to_string(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
            ScriptletAction {
                name: "Third Action".to_string(),
                command: "third".to_string(),
                tool: "bash".to_string(),
                code: "echo third".to_string(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
        ];

        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        let first_idx = actions
            .iter()
            .position(|a| a.id == "scriptlet_action:first")
            .unwrap();
        let second_idx = actions
            .iter()
            .position(|a| a.id == "scriptlet_action:second")
            .unwrap();
        let third_idx = actions
            .iter()
            .position(|a| a.id == "scriptlet_action:third")
            .unwrap();
        assert!(first_idx < second_idx, "First before second");
        assert!(second_idx < third_idx, "Second before third");
    }

