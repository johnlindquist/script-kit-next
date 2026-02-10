    #[test]
    fn title_lower_matches_title_for_ai() {
        for action in &get_ai_command_bar_actions() {
            assert_eq!(
                action.title_lower,
                action.title.to_lowercase(),
                "title_lower mismatch for '{}'",
                action.id
            );
        }
    }

    #[test]
    fn title_lower_matches_title_for_notes() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        for action in &get_notes_command_bar_actions(&info) {
            assert_eq!(
                action.title_lower,
                action.title.to_lowercase(),
                "title_lower mismatch for '{}'",
                action.id
            );
        }
    }

    #[test]
    fn title_lower_matches_title_for_new_chat() {
        let lu = vec![NewChatModelInfo {
            model_id: "m".to_string(),
            display_name: "Model ABC".to_string(),
            provider: "p".to_string(),
            provider_display_name: "P".to_string(),
        }];
        for action in &get_new_chat_actions(&lu, &[], &[]) {
            assert_eq!(
                action.title_lower,
                action.title.to_lowercase(),
                "title_lower mismatch for '{}'",
                action.id
            );
        }
    }

    #[test]
    fn title_lower_matches_title_for_note_switcher() {
        let notes = vec![make_note("n1", "My Note", 50, false, false, "", "")];
        for action in &get_note_switcher_actions(&notes) {
            assert_eq!(
                action.title_lower,
                action.title.to_lowercase(),
                "title_lower mismatch for '{}'",
                action.id
            );
        }
    }

    #[test]
    fn title_lower_matches_title_for_path() {
        let path = PathInfo {
            path: "/test/MyDir".to_string(),
            name: "MyDir".to_string(),
            is_dir: true,
        };
        for action in &get_path_context_actions(&path) {
            assert_eq!(
                action.title_lower,
                action.title.to_lowercase(),
                "title_lower mismatch for '{}'",
                action.id
            );
        }
    }

    #[test]
    fn title_lower_matches_title_for_file() {
        let file = FileInfo {
            path: "/test/MyFile.txt".to_string(),
            name: "MyFile.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        for action in &get_file_context_actions(&file) {
            assert_eq!(
                action.title_lower,
                action.title.to_lowercase(),
                "title_lower mismatch for '{}'",
                action.id
            );
        }
    }

    #[test]
    fn description_lower_matches_description_for_script() {
        let script = ScriptInfo::new("My Script", "/path/s.ts");
        for action in &get_script_context_actions(&script) {
            match (&action.description, &action.description_lower) {
                (Some(desc), Some(desc_lower)) => {
                    assert_eq!(
                        *desc_lower,
                        desc.to_lowercase(),
                        "description_lower mismatch for '{}'",
                        action.id
                    );
                }
                (None, None) => {} // Both absent is fine
                _ => panic!(
                    "description and description_lower mismatch for '{}': desc={:?}, lower={:?}",
                    action.id, action.description, action.description_lower
                ),
            }
        }
    }

    // ============================================================
    // Additional: Scriptlet with custom actions
    // ============================================================

    #[test]
    fn scriptlet_custom_actions_have_has_action_true() {
        let script = ScriptInfo::scriptlet("Test", "/test.md", None, None);
        let mut scriptlet = Scriptlet::new(
            "Test".to_string(),
            "bash".to_string(),
            "echo main".to_string(),
        );
        scriptlet.actions = vec![
            ScriptletAction {
                name: "Copy".to_string(),
                command: "copy".to_string(),
                tool: "bash".to_string(),
                code: "echo copy".to_string(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
            ScriptletAction {
                name: "Open".to_string(),
                command: "open".to_string(),
                tool: "open".to_string(),
                code: "https://example.com".to_string(),
                inputs: vec![],
                shortcut: Some("cmd+o".to_string()),
                description: Some("Open in browser".to_string()),
            },
        ];
        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        let custom: Vec<&Action> = actions
            .iter()
            .filter(|a| a.id.starts_with("scriptlet_action:"))
            .collect();
        assert_eq!(custom.len(), 2);
        for ca in &custom {
            assert!(
                ca.has_action,
                "Custom action '{}' should have has_action=true",
                ca.id
            );
            assert!(
                ca.value.is_some(),
                "Custom action '{}' should have value",
                ca.id
            );
        }
    }

    #[test]
    fn scriptlet_custom_actions_appear_after_run_before_edit() {
        let script = ScriptInfo::scriptlet("Test", "/test.md", None, None);
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
        let run_pos = actions.iter().position(|a| a.id == "run_script").unwrap();
        let custom_pos = actions
            .iter()
            .position(|a| a.id == "scriptlet_action:custom")
            .unwrap();
        let edit_pos = actions
            .iter()
            .position(|a| a.id == "edit_scriptlet")
            .unwrap();
        assert!(run_pos < custom_pos, "Run before custom");
        assert!(custom_pos < edit_pos, "Custom before edit");
    }

    #[test]
    fn scriptlet_custom_action_shortcut_formatted() {
        let script = ScriptInfo::scriptlet("Test", "/test.md", None, None);
        let mut scriptlet = Scriptlet::new(
            "Test".to_string(),
            "bash".to_string(),
            "echo main".to_string(),
        );
        scriptlet.actions = vec![ScriptletAction {
            name: "Copy".to_string(),
            command: "copy".to_string(),
            tool: "bash".to_string(),
            code: "echo".to_string(),
            inputs: vec![],
            shortcut: Some("cmd+c".to_string()),
            description: None,
        }];
        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        let custom = find_action(&actions, "scriptlet_action:copy").unwrap();
        // The shortcut should be formatted using builders.rs format_shortcut_hint
        assert_eq!(custom.shortcut, Some("⌘C".to_string()));
    }

    // ============================================================
    // Additional: to_deeplink_name edge cases
    // ============================================================

    #[test]
    fn deeplink_unicode_chars_stripped() {
        // Non-alphanumeric chars (including accented) should be replaced with hyphens
        // Actually accented chars are NOT alphanumeric in Rust's is_alphanumeric()
        // Wait, they ARE: 'é'.is_alphanumeric() == true
        let result = to_deeplink_name("café");
        assert_eq!(result, "café");
    }

    #[test]
    fn deeplink_numbers_preserved() {
        let result = to_deeplink_name("Script 123");
        assert_eq!(result, "script-123");
    }

    #[test]
    fn deeplink_all_special_returns_empty() {
        let result = to_deeplink_name("!@#$%");
        assert_eq!(result, "");
    }

    #[test]
    fn deeplink_already_hyphenated_passthrough() {
        let result = to_deeplink_name("my-script");
        assert_eq!(result, "my-script");
    }

    #[test]
    fn deeplink_leading_trailing_special() {
        let result = to_deeplink_name(" !hello! ");
        assert_eq!(result, "hello");
    }

    // ============================================================
    // Additional: Ordering determinism
    // ============================================================

    #[test]
    fn ordering_determinism_script() {
        let script = ScriptInfo::new("Test", "/test.ts");
        let actions_1 = get_script_context_actions(&script);
        let ids1 = action_ids(&actions_1);
        let actions_2 = get_script_context_actions(&script);
        let ids2 = action_ids(&actions_2);
        assert_eq!(ids1, ids2, "Script actions should be deterministic");
    }

    #[test]
    fn ordering_determinism_clipboard() {
        let entry = make_text_entry(false, None);
        let actions_1 = get_clipboard_history_context_actions(&entry);
        let ids1 = action_ids(&actions_1);
        let actions_2 = get_clipboard_history_context_actions(&entry);
        let ids2 = action_ids(&actions_2);
        assert_eq!(ids1, ids2, "Clipboard actions should be deterministic");
    }

    #[test]
    fn ordering_determinism_ai() {
        let actions_1 = get_ai_command_bar_actions();
        let ids1 = action_ids(&actions_1);
        let actions_2 = get_ai_command_bar_actions();
        let ids2 = action_ids(&actions_2);
        assert_eq!(ids1, ids2, "AI actions should be deterministic");
    }

    #[test]
    fn ordering_determinism_notes() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions_1 = get_notes_command_bar_actions(&info);
        let ids1 = action_ids(&actions_1);
        let actions_2 = get_notes_command_bar_actions(&info);
        let ids2 = action_ids(&actions_2);
        assert_eq!(ids1, ids2, "Notes actions should be deterministic");
    }

    #[test]
    fn ordering_determinism_path() {
        let path = PathInfo {
            path: "/test".to_string(),
            name: "test".to_string(),
            is_dir: false,
        };
        let actions_1 = get_path_context_actions(&path);
        let ids1 = action_ids(&actions_1);
        let actions_2 = get_path_context_actions(&path);
        let ids2 = action_ids(&actions_2);
        assert_eq!(ids1, ids2, "Path actions should be deterministic");
    }

    // ============================================================
    // Additional: ID uniqueness per context
    // ============================================================

    #[test]
    fn id_uniqueness_script() {
        let script = ScriptInfo::new("s", "/s.ts");
        let actions = get_script_context_actions(&script);
        let ids: HashSet<&str> = action_ids(&actions).into_iter().collect();
        assert_eq!(
            ids.len(),
            actions.len(),
            "Script action IDs should be unique"
        );
    }

    #[test]
    fn id_uniqueness_clipboard() {
        let entry = make_text_entry(false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let ids: HashSet<&str> = action_ids(&actions).into_iter().collect();
        assert_eq!(
            ids.len(),
            actions.len(),
            "Clipboard action IDs should be unique"
        );
    }

    #[test]
    fn id_uniqueness_ai() {
        let actions = get_ai_command_bar_actions();
        let ids: HashSet<&str> = action_ids(&actions).into_iter().collect();
        assert_eq!(ids.len(), actions.len(), "AI action IDs should be unique");
    }

    #[test]
    fn id_uniqueness_path() {
        let path = PathInfo {
            path: "/test".to_string(),
            name: "test".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path);
        let ids: HashSet<&str> = action_ids(&actions).into_iter().collect();
        assert_eq!(ids.len(), actions.len(), "Path action IDs should be unique");
    }

    #[test]
    fn id_uniqueness_file() {
        let file = FileInfo {
            path: "/f.txt".to_string(),
            name: "f.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        let ids: HashSet<&str> = action_ids(&actions).into_iter().collect();
        assert_eq!(ids.len(), actions.len(), "File action IDs should be unique");
    }

    #[test]
    fn id_uniqueness_notes() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids: HashSet<&str> = action_ids(&actions).into_iter().collect();
        assert_eq!(
            ids.len(),
            actions.len(),
            "Notes action IDs should be unique"
        );
    }

    // ============================================================
    // Additional: has_action=false invariant for all built-in actions
    // ============================================================

    #[test]
    fn has_action_false_for_script() {
        let script = ScriptInfo::new("s", "/s.ts");
        for action in &get_script_context_actions(&script) {
            assert!(
                !action.has_action,
                "Built-in action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn has_action_false_for_clipboard() {
        let entry = make_text_entry(false, None);
        for action in &get_clipboard_history_context_actions(&entry) {
            assert!(
                !action.has_action,
                "Built-in action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn has_action_false_for_ai() {
        for action in &get_ai_command_bar_actions() {
            assert!(
                !action.has_action,
                "Built-in action '{}' should have has_action=false",
                action.id
            );
        }
    }

