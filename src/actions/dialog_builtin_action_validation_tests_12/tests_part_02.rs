    #[test]
    fn cat06_ai_6_unique_sections() {
        let actions = get_ai_command_bar_actions();
        let sections: HashSet<_> = actions
            .iter()
            .filter_map(|a| a.section.as_deref())
            .collect();
        assert_eq!(sections.len(), 6);
    }

    // =========================================================================
    // 7. Path context description substring matching
    // =========================================================================

    #[test]
    fn cat07_path_dir_open_description_mentions_directory() {
        let info = PathInfo {
            path: "/tmp/docs".into(),
            name: "docs".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        let open = actions.iter().find(|a| a.id == "open_directory").unwrap();
        assert!(
            open.description.as_ref().unwrap().contains("directory")
                || open.description.as_ref().unwrap().contains("Navigate")
        );
    }

    #[test]
    fn cat07_path_file_select_description_mentions_file() {
        let info = PathInfo {
            path: "/tmp/file.txt".into(),
            name: "file.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let sel = actions.iter().find(|a| a.id == "select_file").unwrap();
        assert!(
            sel.description.as_ref().unwrap().contains("file")
                || sel.description.as_ref().unwrap().contains("Submit")
        );
    }

    #[test]
    fn cat07_path_trash_dir_says_folder() {
        let info = PathInfo {
            path: "/tmp/docs".into(),
            name: "docs".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
        assert!(trash.description.as_ref().unwrap().contains("folder"));
    }

    #[test]
    fn cat07_path_trash_file_says_file() {
        let info = PathInfo {
            path: "/tmp/a.txt".into(),
            name: "a.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
        assert!(trash.description.as_ref().unwrap().contains("file"));
    }

    #[test]
    fn cat07_path_open_in_editor_mentions_editor() {
        let info = PathInfo {
            path: "/tmp/a.txt".into(),
            name: "a.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let ed = actions.iter().find(|a| a.id == "open_in_editor").unwrap();
        assert!(
            ed.description.as_ref().unwrap().contains("$EDITOR")
                || ed.description.as_ref().unwrap().contains("editor")
        );
    }

    // =========================================================================
    // 8. Chat context multi-model ID generation
    // =========================================================================

    #[test]
    fn cat08_multiple_models_generate_sequential_ids() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![
                ChatModelInfo {
                    id: "gpt-4".into(),
                    display_name: "GPT-4".into(),
                    provider: "OpenAI".into(),
                },
                ChatModelInfo {
                    id: "claude-3".into(),
                    display_name: "Claude 3".into(),
                    provider: "Anthropic".into(),
                },
                ChatModelInfo {
                    id: "gemini".into(),
                    display_name: "Gemini".into(),
                    provider: "Google".into(),
                },
            ],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "select_model_gpt-4"));
        assert!(actions.iter().any(|a| a.id == "select_model_claude-3"));
        assert!(actions.iter().any(|a| a.id == "select_model_gemini"));
    }

    #[test]
    fn cat08_current_model_gets_checkmark() {
        let info = ChatPromptInfo {
            current_model: Some("GPT-4".into()),
            available_models: vec![
                ChatModelInfo {
                    id: "gpt-4".into(),
                    display_name: "GPT-4".into(),
                    provider: "OpenAI".into(),
                },
                ChatModelInfo {
                    id: "claude-3".into(),
                    display_name: "Claude 3".into(),
                    provider: "Anthropic".into(),
                },
            ],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let gpt = actions
            .iter()
            .find(|a| a.id == "select_model_gpt-4")
            .unwrap();
        assert!(gpt.title.contains("✓"));
        let claude = actions
            .iter()
            .find(|a| a.id == "select_model_claude-3")
            .unwrap();
        assert!(!claude.title.contains("✓"));
    }

    #[test]
    fn cat08_no_models_still_has_continue() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "continue_in_chat"));
    }

    #[test]
    fn cat08_model_description_shows_provider() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "m1".into(),
                display_name: "Model 1".into(),
                provider: "Anthropic".into(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let m = actions.iter().find(|a| a.id == "select_model_m1").unwrap();
        assert!(m.description.as_ref().unwrap().contains("Anthropic"));
    }

    #[test]
    fn cat08_has_response_adds_copy_response() {
        let with = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: true,
        };
        let without = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        assert!(get_chat_context_actions(&with)
            .iter()
            .any(|a| a.id == "copy_response"));
        assert!(!get_chat_context_actions(&without)
            .iter()
            .any(|a| a.id == "copy_response"));
    }

    #[test]
    fn cat08_has_messages_adds_clear_conversation() {
        let with = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: false,
        };
        let without = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        assert!(get_chat_context_actions(&with)
            .iter()
            .any(|a| a.id == "clear_conversation"));
        assert!(!get_chat_context_actions(&without)
            .iter()
            .any(|a| a.id == "clear_conversation"));
    }

    // =========================================================================
    // 9. Score_action stacking with multi-field matches
    // =========================================================================

    #[test]
    fn cat09_prefix_plus_description_stacks() {
        let action = Action::new(
            "edit",
            "Edit Script",
            Some("Edit the script file".into()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "edit");
        assert!(score >= 115, "prefix(100)+desc(15)={}", score);
    }

    #[test]
    fn cat09_prefix_plus_shortcut_stacks() {
        let action =
            Action::new("edit", "Edit", None, ActionCategory::ScriptContext).with_shortcut("edit");
        let score = ActionsDialog::score_action(&action, "edit");
        assert!(score >= 110, "prefix(100)+shortcut(10)={}", score);
    }

    #[test]
    fn cat09_contains_plus_description_stacks() {
        let action = Action::new(
            "x",
            "Copy Edit Path",
            Some("Edit mode".into()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "edit");
        assert!(score >= 65, "contains(50)+desc(15)={}", score);
    }

    #[test]
    fn cat09_no_match_returns_zero() {
        let action = Action::new(
            "x",
            "Foo",
            Some("Bar".into()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(ActionsDialog::score_action(&action, "zzz"), 0);
    }

    #[test]
    fn cat09_empty_query_matches_prefix() {
        let action = Action::new("x", "Anything", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "");
        assert!(
            score >= 100,
            "empty query is prefix of everything: {}",
            score
        );
    }

    // =========================================================================
    // 10. build_grouped_items_static with alternating sections
    // =========================================================================

    #[test]
    fn cat10_alternating_sections_produce_multiple_headers() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Alpha"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Beta"),
            Action::new("c", "C", None, ActionCategory::ScriptContext).with_section("Alpha"),
        ];
        let filtered: Vec<usize> = (0..3).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        let header_count = grouped
            .iter()
            .filter(|i| matches!(i, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 3, "Alpha, Beta, Alpha = 3 headers");
    }

    #[test]
    fn cat10_same_section_no_duplicate_header() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("Same"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Same"),
        ];
        let filtered: Vec<usize> = (0..2).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        let header_count = grouped
            .iter()
            .filter(|i| matches!(i, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 1);
    }

    #[test]
    fn cat10_separators_no_headers() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("X"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("Y"),
        ];
        let filtered: Vec<usize> = (0..2).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        let header_count = grouped
            .iter()
            .filter(|i| matches!(i, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 0);
    }

    #[test]
    fn cat10_none_style_no_headers() {
        let actions =
            vec![Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("X")];
        let filtered: Vec<usize> = vec![0];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
        let header_count = grouped
            .iter()
            .filter(|i| matches!(i, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 0);
    }

    #[test]
    fn cat10_empty_filtered_returns_empty() {
        let actions = vec![Action::new("a", "A", None, ActionCategory::ScriptContext)];
        let grouped = build_grouped_items_static(&actions, &[], SectionStyle::Headers);
        assert!(grouped.is_empty());
    }

    // =========================================================================
    // 11. Cross-context action description non-empty invariant
    // =========================================================================

    #[test]
    fn cat11_script_actions_all_have_descriptions() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        for action in &get_script_context_actions(&script) {
            assert!(
                action.description.is_some(),
                "Script action '{}' missing description",
                action.id
            );
        }
    }

    #[test]
    fn cat11_clipboard_actions_all_have_descriptions() {
        for action in &get_clipboard_history_context_actions(&make_text_entry()) {
            assert!(
                action.description.is_some(),
                "Clipboard action '{}' missing description",
                action.id
            );
        }
    }

    #[test]
    fn cat11_ai_actions_all_have_descriptions() {
        for action in &get_ai_command_bar_actions() {
            assert!(
                action.description.is_some(),
                "AI action '{}' missing description",
                action.id
            );
        }
    }

    #[test]
    fn cat11_path_actions_all_have_descriptions() {
        let info = PathInfo {
            path: "/tmp/x".into(),
            name: "x".into(),
            is_dir: false,
        };
        for action in &get_path_context_actions(&info) {
            assert!(
                action.description.is_some(),
                "Path action '{}' missing description",
                action.id
            );
        }
    }

    #[test]
    fn cat11_file_actions_all_have_descriptions() {
        let info = FileInfo {
            path: "/tmp/x.txt".into(),
            name: "x.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        for action in &get_file_context_actions(&info) {
            assert!(
                action.description.is_some(),
                "File action '{}' missing description",
                action.id
            );
        }
    }

    #[test]
    fn cat11_notes_actions_all_have_descriptions() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        for action in &get_notes_command_bar_actions(&info) {
            assert!(
                action.description.is_some(),
                "Notes action '{}' missing description",
                action.id
            );
        }
    }

    // =========================================================================
    // 12. format_shortcut_hint chaining — multiple modifiers
    // =========================================================================

    // format_shortcut_hint is private, but we test it indirectly through scriptlet actions
