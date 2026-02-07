    #[test]
    fn notes_new_note_description_mentions_create() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let new_note = actions.iter().find(|a| a.id == "new_note").unwrap();
        let desc = new_note.description.as_ref().unwrap().to_lowercase();
        assert!(desc.contains("create") || desc.contains("new"));
    }

    // =========================================================================
    // 9. Score_action with cached lowercase fields
    // =========================================================================

    #[test]
    fn score_action_uses_title_lower_cache() {
        let action = Action::new(
            "test",
            "Edit Script",
            Some("Open in editor".into()),
            ActionCategory::ScriptContext,
        );
        // title_lower should be "edit script"
        assert_eq!(action.title_lower, "edit script");
        let score = ActionsDialog::score_action(&action, "edit");
        assert!(
            score >= 100,
            "Prefix match on title_lower should score 100+"
        );
    }

    #[test]
    fn score_action_description_lower_bonus() {
        let action = Action::new(
            "open",
            "Open File",
            Some("Open with default application".into()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(
            action.description_lower.as_deref(),
            Some("open with default application")
        );
        let score = ActionsDialog::score_action(&action, "default");
        // "default" not in title, but in description
        assert!(
            score >= 15,
            "Description match should add 15+ points, got {}",
            score
        );
    }

    #[test]
    fn score_action_shortcut_lower_bonus() {
        let action = Action::new("test", "Run Script", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘R");
        assert_eq!(action.shortcut_lower.as_deref(), Some("⌘r"));
        let score = ActionsDialog::score_action(&action, "⌘r");
        // "⌘r" in shortcut_lower
        assert!(
            score >= 10,
            "Shortcut match should add 10+ points, got {}",
            score
        );
    }

    #[test]
    fn score_action_empty_query_returns_zero() {
        let action = Action::new("test", "Run Script", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "");
        // Empty string is a prefix of everything, so it scores 100
        assert_eq!(score, 100);
    }

    #[test]
    fn score_action_no_match_returns_zero() {
        let action = Action::new("test", "Run Script", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "zzzzz");
        assert_eq!(score, 0);
    }

    #[test]
    fn score_action_prefix_beats_contains() {
        let prefix_action = Action::new("e", "Edit Script", None, ActionCategory::ScriptContext);
        let contains_action = Action::new("c", "My Edit Tool", None, ActionCategory::ScriptContext);
        let prefix_score = ActionsDialog::score_action(&prefix_action, "edit");
        let contains_score = ActionsDialog::score_action(&contains_action, "edit");
        assert!(prefix_score > contains_score);
    }

    #[test]
    fn score_action_contains_beats_fuzzy() {
        let contains_action = Action::new("c", "My Edit Tool", None, ActionCategory::ScriptContext);
        let fuzzy_action = Action::new("f", "Erase Dict", None, ActionCategory::ScriptContext);
        let contains_score = ActionsDialog::score_action(&contains_action, "edit");
        let fuzzy_score = ActionsDialog::score_action(&fuzzy_action, "edit");
        assert!(
            contains_score > fuzzy_score,
            "Contains {} should beat fuzzy {}",
            contains_score,
            fuzzy_score
        );
    }

    #[test]
    fn score_action_stacks_title_and_description() {
        let action = Action::new(
            "test",
            "Edit Script",
            Some("Edit the script in your editor".into()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "edit");
        // Prefix match (100) + description match (15) = 115
        assert_eq!(score, 115);
    }

    // =========================================================================
    // 10. fuzzy_match edge cases
    // =========================================================================

    #[test]
    fn fuzzy_match_empty_needle_always_matches() {
        assert!(ActionsDialog::fuzzy_match("anything", ""));
    }

    #[test]
    fn fuzzy_match_empty_haystack_no_match() {
        assert!(!ActionsDialog::fuzzy_match("", "a"));
    }

    #[test]
    fn fuzzy_match_both_empty_matches() {
        assert!(ActionsDialog::fuzzy_match("", ""));
    }

    #[test]
    fn fuzzy_match_subsequence() {
        assert!(ActionsDialog::fuzzy_match("edit script", "es"));
        assert!(ActionsDialog::fuzzy_match("edit script", "eit"));
        assert!(ActionsDialog::fuzzy_match("edit script", "edsc"));
    }

    #[test]
    fn fuzzy_match_no_subsequence() {
        assert!(!ActionsDialog::fuzzy_match("edit", "xyz"));
        assert!(!ActionsDialog::fuzzy_match("abc", "abdc"));
    }

    #[test]
    fn fuzzy_match_needle_longer_than_haystack() {
        assert!(!ActionsDialog::fuzzy_match("ab", "abc"));
    }

    #[test]
    fn fuzzy_match_exact_equals() {
        assert!(ActionsDialog::fuzzy_match("hello", "hello"));
    }

    // =========================================================================
    // 11. Scriptlet with_custom multiple custom actions ordering
    // =========================================================================

    #[test]
    fn scriptlet_three_custom_actions_maintain_order() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo main".into());
        scriptlet.actions = vec![
            ScriptletAction {
                name: "First".into(),
                command: "first".into(),
                tool: "bash".into(),
                code: "echo 1".into(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
            ScriptletAction {
                name: "Second".into(),
                command: "second".into(),
                tool: "bash".into(),
                code: "echo 2".into(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
            ScriptletAction {
                name: "Third".into(),
                command: "third".into(),
                tool: "bash".into(),
                code: "echo 3".into(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
        ];

        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        let ids = action_ids(&actions);

        let first_pos = ids
            .iter()
            .position(|&id| id == "scriptlet_action:first")
            .unwrap();
        let second_pos = ids
            .iter()
            .position(|&id| id == "scriptlet_action:second")
            .unwrap();
        let third_pos = ids
            .iter()
            .position(|&id| id == "scriptlet_action:third")
            .unwrap();

        assert!(first_pos < second_pos);
        assert!(second_pos < third_pos);
    }

    #[test]
    fn scriptlet_custom_actions_after_run_before_shortcut() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo main".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "Custom".into(),
            command: "custom".into(),
            tool: "bash".into(),
            code: "echo custom".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];

        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        let ids = action_ids(&actions);

        let run_pos = ids.iter().position(|&id| id == "run_script").unwrap();
        let custom_pos = ids
            .iter()
            .position(|&id| id == "scriptlet_action:custom")
            .unwrap();
        let shortcut_pos = ids.iter().position(|&id| id == "add_shortcut").unwrap();

        assert_eq!(run_pos, 0);
        assert!(custom_pos > run_pos);
        assert!(custom_pos < shortcut_pos);
    }

    #[test]
    fn scriptlet_custom_actions_all_have_has_action_true() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo main".into());
        scriptlet.actions = vec![
            ScriptletAction {
                name: "A".into(),
                command: "a".into(),
                tool: "bash".into(),
                code: "echo a".into(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
            ScriptletAction {
                name: "B".into(),
                command: "b".into(),
                tool: "bash".into(),
                code: "echo b".into(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
        ];

        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        for action in &actions {
            if action.id.starts_with("scriptlet_action:") {
                assert!(
                    action.has_action,
                    "Scriptlet custom action '{}' should have has_action=true",
                    action.id
                );
                assert!(
                    action.value.is_some(),
                    "Scriptlet custom action '{}' should have a value",
                    action.id
                );
            }
        }
    }

    #[test]
    fn scriptlet_custom_action_value_matches_command() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo main".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "Copy Stuff".into(),
            command: "copy-stuff".into(),
            tool: "bash".into(),
            code: "echo copy".into(),
            inputs: vec![],
            shortcut: Some("cmd+c".into()),
            description: Some("Copy stuff desc".into()),
        }];

        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));
        let custom = actions
            .iter()
            .find(|a| a.id == "scriptlet_action:copy-stuff")
            .unwrap();
        assert_eq!(custom.value.as_deref(), Some("copy-stuff"));
        assert_eq!(custom.title, "Copy Stuff");
        assert_eq!(custom.description.as_deref(), Some("Copy stuff desc"));
        assert_eq!(custom.shortcut.as_deref(), Some("⌘C"));
    }

    // =========================================================================
    // 12. CommandBarConfig field validation
    // =========================================================================

    #[test]
    fn command_bar_config_ai_style_fields() {
        let config = CommandBarConfig::ai_style();
        assert_eq!(config.dialog_config.search_position, SearchPosition::Top);
        assert_eq!(config.dialog_config.section_style, SectionStyle::Headers);
        assert_eq!(config.dialog_config.anchor, AnchorPosition::Top);
        assert!(config.dialog_config.show_icons);
        assert!(config.dialog_config.show_footer);
        assert!(config.close_on_select);
        assert!(config.close_on_escape);
        assert!(config.close_on_click_outside);
    }

    #[test]
    fn command_bar_config_main_menu_style_fields() {
        let config = CommandBarConfig::main_menu_style();
        assert_eq!(config.dialog_config.search_position, SearchPosition::Bottom);
        assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
        assert_eq!(config.dialog_config.anchor, AnchorPosition::Bottom);
        assert!(!config.dialog_config.show_icons);
        assert!(!config.dialog_config.show_footer);
    }

    #[test]
    fn command_bar_config_no_search_fields() {
        let config = CommandBarConfig::no_search();
        assert_eq!(config.dialog_config.search_position, SearchPosition::Hidden);
        assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
        assert!(!config.dialog_config.show_icons);
    }

    #[test]
    fn command_bar_config_notes_style_fields() {
        let config = CommandBarConfig::notes_style();
        assert_eq!(config.dialog_config.search_position, SearchPosition::Top);
        assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
        assert_eq!(config.dialog_config.anchor, AnchorPosition::Top);
        assert!(config.dialog_config.show_icons);
        assert!(config.dialog_config.show_footer);
    }

    #[test]
    fn command_bar_config_default_all_close_true() {
        let config = CommandBarConfig::default();
        assert!(config.close_on_select);
        assert!(config.close_on_escape);
        assert!(config.close_on_click_outside);
    }

    // =========================================================================
    // 13. Chat context action interactions
    // =========================================================================

    #[test]
    fn chat_no_models_no_messages_no_response() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        // Only continue_in_chat (no models, no copy, no clear)
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].id, "continue_in_chat");
    }

    #[test]
    fn chat_with_models_and_response_and_messages() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".into()),
            available_models: vec![
                ChatModelInfo {
                    id: "claude".into(),
                    display_name: "Claude".into(),
                    provider: "Anthropic".into(),
                },
                ChatModelInfo {
                    id: "gpt4".into(),
                    display_name: "GPT-4".into(),
                    provider: "OpenAI".into(),
                },
            ],
            has_messages: true,
            has_response: true,
        };
        let actions = get_chat_context_actions(&info);
        // 2 models + continue_in_chat + copy_response + clear_conversation = 5
        assert_eq!(actions.len(), 5);
    }

    #[test]
    fn chat_current_model_gets_checkmark() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".into()),
            available_models: vec![
                ChatModelInfo {
                    id: "claude".into(),
                    display_name: "Claude".into(),
                    provider: "Anthropic".into(),
                },
                ChatModelInfo {
                    id: "gpt4".into(),
                    display_name: "GPT-4".into(),
                    provider: "OpenAI".into(),
                },
            ],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let claude = actions
            .iter()
            .find(|a| a.id == "select_model_claude")
            .unwrap();
        assert!(claude.title.contains("✓"), "Current model should have ✓");

        let gpt4 = actions
            .iter()
            .find(|a| a.id == "select_model_gpt4")
            .unwrap();
        assert!(
            !gpt4.title.contains("✓"),
            "Non-current model should not have ✓"
        );
    }

