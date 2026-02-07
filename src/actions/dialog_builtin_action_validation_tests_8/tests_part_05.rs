    #[test]
    fn scriptlet_custom_actions_after_run_before_shortcut() {
        let script = ScriptInfo::scriptlet("Test", "/path/to/test.md", None, None);
        let mut scriptlet = Scriptlet::new(
            "Test".to_string(),
            "bash".to_string(),
            "echo main".to_string(),
        );
        scriptlet.actions = vec![ScriptletAction {
            name: "Custom".to_string(),
            command: "custom".to_string(),
            tool: "bash".to_string(),
            code: "echo custom".to_string(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];

        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        let run_idx = actions.iter().position(|a| a.id == "run_script").unwrap();
        let custom_idx = actions
            .iter()
            .position(|a| a.id == "scriptlet_action:custom")
            .unwrap();
        let shortcut_idx = actions.iter().position(|a| a.id == "add_shortcut").unwrap();
        assert!(run_idx < custom_idx, "Run before custom");
        assert!(
            custom_idx < shortcut_idx,
            "Custom before shortcut management"
        );
    }

    // ============================================================
    // 26. Chat model checkmark exact match only
    // ============================================================

    #[test]
    fn chat_no_checkmark_on_partial_match() {
        let info = ChatPromptInfo {
            current_model: Some("GPT-4".to_string()),
            available_models: vec![
                ChatModelInfo {
                    id: "gpt4o".to_string(),
                    display_name: "GPT-4o".to_string(),
                    provider: "OpenAI".to_string(),
                },
                ChatModelInfo {
                    id: "gpt4".to_string(),
                    display_name: "GPT-4".to_string(),
                    provider: "OpenAI".to_string(),
                },
            ],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let gpt4o = find_action(&actions, "select_model_gpt4o").unwrap();
        let gpt4 = find_action(&actions, "select_model_gpt4").unwrap();
        assert!(
            !gpt4o.title.contains('✓'),
            "GPT-4o should not have checkmark"
        );
        assert!(gpt4.title.contains('✓'), "GPT-4 should have checkmark");
    }

    // ============================================================
    // 27. Note switcher empty/placeholder title
    // ============================================================

    #[test]
    fn note_switcher_description_falls_back_to_char_count() {
        let notes = vec![make_note("id1", "Note", 42, false, false, "", "")];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "42 chars");
    }

    #[test]
    fn note_switcher_singular_char_count() {
        let notes = vec![make_note("id1", "Note", 1, false, false, "", "")];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "1 char");
    }

    #[test]
    fn note_switcher_zero_chars() {
        let notes = vec![make_note("id1", "Note", 0, false, false, "", "")];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "0 chars");
    }

    #[test]
    fn note_switcher_preview_with_time() {
        let notes = vec![make_note(
            "id1",
            "Note",
            100,
            false,
            false,
            "Hello world",
            "5m ago",
        )];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "Hello world · 5m ago");
    }

    #[test]
    fn note_switcher_preview_truncation_at_61() {
        let long_preview = "a".repeat(61);
        let notes = vec![make_note(
            "id1",
            "Note",
            100,
            false,
            false,
            &long_preview,
            "",
        )];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.ends_with('…'), "Should be truncated with ellipsis");
        // 60 chars + ellipsis
        assert_eq!(desc.chars().count(), 61);
    }

    #[test]
    fn note_switcher_preview_not_truncated_at_60() {
        let exact_preview = "a".repeat(60);
        let notes = vec![make_note(
            "id1",
            "Note",
            100,
            false,
            false,
            &exact_preview,
            "",
        )];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(!desc.ends_with('…'), "60 chars should NOT be truncated");
        assert_eq!(desc.chars().count(), 60);
    }

    // ============================================================
    // 28. Action with_section/with_icon chaining order independence
    // ============================================================

    #[test]
    fn action_chaining_shortcut_preserves_title_lower() {
        let a = Action::new("id", "Hello World", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘H");
        assert_eq!(a.title_lower, "hello world");
        assert_eq!(a.shortcut_lower, Some("⌘h".to_string()));
    }

    #[test]
    fn action_description_lower_computed() {
        let a = Action::new(
            "id",
            "Title",
            Some("Mixed CASE Desc".to_string()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(a.description_lower, Some("mixed case desc".to_string()));
    }

    #[test]
    fn action_no_description_lower_is_none() {
        let a = Action::new("id", "Title", None, ActionCategory::ScriptContext);
        assert!(a.description_lower.is_none());
    }

    // ============================================================
    // 29. Clipboard delete_multiple description
    // ============================================================

    #[test]
    fn clipboard_delete_multiple_desc_mentions_filter() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let dm = find_action(&actions, "clipboard_delete_multiple").unwrap();
        assert!(
            dm.description
                .as_ref()
                .unwrap()
                .to_lowercase()
                .contains("filter")
                || dm
                    .description
                    .as_ref()
                    .unwrap()
                    .to_lowercase()
                    .contains("matching"),
            "delete_multiple desc should mention filtering/matching"
        );
    }

    #[test]
    fn clipboard_delete_all_desc_mentions_pinned() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let da = find_action(&actions, "clipboard_delete_all").unwrap();
        assert!(
            da.description
                .as_ref()
                .unwrap()
                .to_lowercase()
                .contains("pinned"),
            "delete_all desc should mention pinned"
        );
    }

    // ============================================================
    // 30. Deeplink name edge cases
    // ============================================================

    #[test]
    fn deeplink_name_basic() {
        assert_eq!(to_deeplink_name("My Script"), "my-script");
    }

    #[test]
    fn deeplink_name_already_lowercase_hyphenated() {
        assert_eq!(to_deeplink_name("my-script"), "my-script");
    }

    #[test]
    fn deeplink_name_underscores() {
        assert_eq!(to_deeplink_name("hello_world"), "hello-world");
    }

    #[test]
    fn deeplink_name_numbers_preserved() {
        assert_eq!(to_deeplink_name("script123"), "script123");
    }

    #[test]
    fn deeplink_name_leading_trailing_stripped() {
        assert_eq!(to_deeplink_name("  hello  "), "hello");
    }

    // ============================================================
    // Cross-cutting: ID uniqueness across contexts
    // ============================================================

    #[test]
    fn script_action_ids_unique() {
        let script = ScriptInfo::new("test", "/path/to/test.ts");
        let actions = get_script_context_actions(&script);
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(
            ids.len(),
            actions.len(),
            "Script action IDs should be unique"
        );
    }

    #[test]
    fn clipboard_action_ids_unique() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(
            ids.len(),
            actions.len(),
            "Clipboard action IDs should be unique"
        );
    }

    #[test]
    fn ai_command_bar_ids_unique() {
        let actions = get_ai_command_bar_actions();
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(
            ids.len(),
            actions.len(),
            "AI command bar IDs should be unique"
        );
    }

    #[test]
    fn path_action_ids_unique() {
        let info = PathInfo::new("test", "/tmp/test", false);
        let actions = get_path_context_actions(&info);
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), actions.len(), "Path action IDs should be unique");
    }

    #[test]
    fn file_action_ids_unique() {
        let info = FileInfo {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), actions.len(), "File action IDs should be unique");
    }

    // ============================================================
    // Cross-cutting: has_action invariant
    // ============================================================

    #[test]
    fn all_script_actions_have_has_action_false() {
        let script = ScriptInfo::new("test", "/path/to/test.ts");
        let actions = get_script_context_actions(&script);
        for action in &actions {
            assert!(
                !action.has_action,
                "Script action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn all_clipboard_actions_have_has_action_false() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        for action in &actions {
            assert!(
                !action.has_action,
                "Clipboard action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn all_ai_actions_have_has_action_false() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            assert!(
                !action.has_action,
                "AI action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn all_path_actions_have_has_action_false() {
        let info = PathInfo::new("test", "/tmp/test", false);
        let actions = get_path_context_actions(&info);
        for action in &actions {
            assert!(
                !action.has_action,
                "Path action '{}' should have has_action=false",
                action.id
            );
        }
    }

    // ============================================================
    // Cross-cutting: title_lower invariant
    // ============================================================

    #[test]
    fn title_lower_matches_lowercase_for_all_script_actions() {
        let script = ScriptInfo::new("test", "/path/to/test.ts");
        let actions = get_script_context_actions(&script);
        for action in &actions {
            assert_eq!(
                action.title_lower,
                action.title.to_lowercase(),
                "title_lower mismatch for '{}'",
                action.id
            );
        }
    }

    #[test]
    fn title_lower_matches_lowercase_for_ai_actions() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            assert_eq!(
                action.title_lower,
                action.title.to_lowercase(),
                "title_lower mismatch for '{}'",
                action.id
            );
        }
    }

    #[test]
    fn title_lower_matches_lowercase_for_clipboard_actions() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        for action in &actions {
            assert_eq!(
                action.title_lower,
                action.title.to_lowercase(),
                "title_lower mismatch for '{}'",
                action.id
            );
        }
    }

    // ============================================================
    // Cross-cutting: ordering determinism
    // ============================================================

    #[test]
    fn script_actions_deterministic() {
        let script = ScriptInfo::new("test", "/path/to/test.ts");
        let actions1 = get_script_context_actions(&script);
        let actions2 = get_script_context_actions(&script);
        let a1 = action_ids(&actions1);
        let a2 = action_ids(&actions2);
        assert_eq!(a1, a2, "Script actions should be deterministic");
    }

    #[test]
    fn clipboard_actions_deterministic() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions1 = get_clipboard_history_context_actions(&entry);
        let actions2 = get_clipboard_history_context_actions(&entry);
        let a1 = action_ids(&actions1);
        let a2 = action_ids(&actions2);
        assert_eq!(a1, a2, "Clipboard actions should be deterministic");
    }

    #[test]
    fn ai_actions_deterministic() {
        let actions1 = get_ai_command_bar_actions();
        let actions2 = get_ai_command_bar_actions();
        let a1 = action_ids(&actions1);
        let a2 = action_ids(&actions2);
        assert_eq!(a1, a2, "AI actions should be deterministic");
    }

