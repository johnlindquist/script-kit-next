    #[test]
    fn new_chat_last_used_has_bolt_icon() {
        let last_used = vec![NewChatModelInfo {
            model_id: "claude-3".to_string(),
            display_name: "Claude 3".to_string(),
            provider: "anthropic".to_string(),
            provider_display_name: "Anthropic".to_string(),
        }];
        let actions = get_new_chat_actions(&last_used, &[], &[]);
        let action = &actions[0];
        assert_eq!(action.icon, Some(IconName::BoltFilled));
        assert_eq!(action.section.as_deref(), Some("Last Used Settings"));
    }

    #[test]
    fn new_chat_presets_use_custom_icon() {
        let presets = vec![NewChatPresetInfo {
            id: "general".to_string(),
            name: "General".to_string(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        let action = &actions[0];
        assert_eq!(action.icon, Some(IconName::Star));
        assert_eq!(action.section.as_deref(), Some("Presets"));
        assert!(
            action.description.is_none(),
            "Presets should have no description"
        );
    }

    #[test]
    fn new_chat_models_use_settings_icon() {
        let models = vec![NewChatModelInfo {
            model_id: "gpt-4".to_string(),
            display_name: "GPT-4".to_string(),
            provider: "openai".to_string(),
            provider_display_name: "OpenAI".to_string(),
        }];
        let actions = get_new_chat_actions(&[], &[], &models);
        let action = &actions[0];
        assert_eq!(action.icon, Some(IconName::Settings));
        assert_eq!(action.section.as_deref(), Some("Models"));
        assert_eq!(
            action.description.as_deref(),
            Some("OpenAI"),
            "Model should show provider display name"
        );
    }

    #[test]
    fn new_chat_empty_inputs_empty_output() {
        let actions = get_new_chat_actions(&[], &[], &[]);
        assert!(actions.is_empty());
    }

    #[test]
    fn new_chat_all_sections_ordered() {
        let last_used = vec![NewChatModelInfo {
            model_id: "lu".to_string(),
            display_name: "LU".to_string(),
            provider: "p".to_string(),
            provider_display_name: "P".to_string(),
        }];
        let presets = vec![NewChatPresetInfo {
            id: "pr".to_string(),
            name: "PR".to_string(),
            icon: IconName::File,
        }];
        let models = vec![NewChatModelInfo {
            model_id: "m".to_string(),
            display_name: "M".to_string(),
            provider: "p".to_string(),
            provider_display_name: "P".to_string(),
        }];
        let actions = get_new_chat_actions(&last_used, &presets, &models);
        assert_eq!(actions.len(), 3);
        assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
        assert_eq!(actions[1].section.as_deref(), Some("Presets"));
        assert_eq!(actions[2].section.as_deref(), Some("Models"));
    }

    // ============================================================
    // 15. Score_action with multi-word queries
    // ============================================================

    #[test]
    fn score_action_multi_word_prefix() {
        let action = Action::new(
            "edit_script",
            "Edit Script",
            Some("Open in editor".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "edit s");
        assert!(
            score >= 100,
            "Multi-word prefix should score >= 100, got {}",
            score
        );
    }

    #[test]
    fn score_action_multi_word_contains() {
        let action = Action::new(
            "copy_path",
            "Copy Path",
            Some("Copy the full path".to_string()),
            ActionCategory::ScriptContext,
        );
        // "path" is not a prefix of "Copy Path" but it is contained
        let score = ActionsDialog::score_action(&action, "path");
        assert!(
            score >= 50,
            "'path' should match contains on 'copy path', got {}",
            score
        );
    }

    #[test]
    fn score_action_description_only_match() {
        let action = Action::new(
            "reveal",
            "Reveal in Finder",
            Some("Show the file in your filesystem browser".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "filesystem");
        assert_eq!(
            score, 15,
            "Description-only match should score exactly 15, got {}",
            score
        );
    }

    #[test]
    fn score_action_no_match_returns_zero() {
        let action = Action::new(
            "test",
            "Test Action",
            Some("A test".to_string()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "zzzzz");
        assert_eq!(score, 0, "No match should return 0");
    }

    #[test]
    fn score_action_shortcut_match_bonus() {
        let action = Action::new("test", "Test Action", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘E");
        let score = ActionsDialog::score_action(&action, "⌘e");
        assert!(
            score >= 10,
            "Shortcut match should add >=10 bonus, got {}",
            score
        );
    }

    // ============================================================
    // 16. build_grouped_items_static with Headers style
    // ============================================================

    #[test]
    fn build_grouped_items_headers_inserts_section_headers() {
        let actions = vec![
            make_action("a1", "Action 1", Some("Group A")),
            make_action("a2", "Action 2", Some("Group A")),
            make_action("b1", "Action 3", Some("Group B")),
        ];
        let filtered: Vec<usize> = (0..3).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Should be: Header("Group A"), Item(0), Item(1), Header("Group B"), Item(2)
        assert_eq!(grouped.len(), 5);
        assert!(matches!(&grouped[0], GroupedActionItem::SectionHeader(s) if s == "Group A"));
        assert!(matches!(&grouped[1], GroupedActionItem::Item(0)));
        assert!(matches!(&grouped[2], GroupedActionItem::Item(1)));
        assert!(matches!(&grouped[3], GroupedActionItem::SectionHeader(s) if s == "Group B"));
        assert!(matches!(&grouped[4], GroupedActionItem::Item(2)));
    }

    #[test]
    fn build_grouped_items_separators_no_headers() {
        let actions = vec![
            make_action("a1", "Action 1", Some("Group A")),
            make_action("b1", "Action 2", Some("Group B")),
        ];
        let filtered: Vec<usize> = (0..2).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        // Separators style should NOT insert section headers
        assert_eq!(grouped.len(), 2);
        assert!(matches!(&grouped[0], GroupedActionItem::Item(0)));
        assert!(matches!(&grouped[1], GroupedActionItem::Item(1)));
    }

    #[test]
    fn build_grouped_items_none_no_headers() {
        let actions = vec![
            make_action("a1", "Action 1", Some("Group A")),
            make_action("b1", "Action 2", Some("Group B")),
        ];
        let filtered: Vec<usize> = (0..2).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
        assert_eq!(grouped.len(), 2);
    }

    #[test]
    fn build_grouped_items_empty_returns_empty() {
        let actions: Vec<Action> = vec![];
        let filtered: Vec<usize> = vec![];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        assert!(grouped.is_empty());
    }

    #[test]
    fn build_grouped_items_same_section_no_duplicate_header() {
        let actions = vec![
            make_action("a1", "A1", Some("Same")),
            make_action("a2", "A2", Some("Same")),
            make_action("a3", "A3", Some("Same")),
        ];
        let filtered: Vec<usize> = (0..3).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        let header_count = grouped
            .iter()
            .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 1, "Same section should produce only 1 header");
    }

    // ============================================================
    // 17. coerce_action_selection edge cases
    // ============================================================

    #[test]
    fn coerce_action_selection_single_item() {
        let rows = vec![GroupedActionItem::Item(0)];
        assert_eq!(coerce_action_selection(&rows, 0), Some(0));
    }

    #[test]
    fn coerce_action_selection_single_header() {
        let rows = vec![GroupedActionItem::SectionHeader("Test".to_string())];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    #[test]
    fn coerce_action_selection_header_then_item() {
        let rows = vec![
            GroupedActionItem::SectionHeader("S".to_string()),
            GroupedActionItem::Item(0),
        ];
        // Landing on header should move to item
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
    }

    #[test]
    fn coerce_action_selection_item_then_header() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("S".to_string()),
        ];
        // Landing on header should search up to item
        assert_eq!(coerce_action_selection(&rows, 1), Some(0));
    }

    #[test]
    fn coerce_action_selection_out_of_bounds_clamped() {
        let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
        assert_eq!(
            coerce_action_selection(&rows, 999),
            Some(1),
            "Out of bounds should clamp to last"
        );
    }

    // ============================================================
    // 18. parse_shortcut_keycaps sequences
    // ============================================================

    #[test]
    fn parse_keycaps_modifier_and_letter() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘E");
        assert_eq!(keycaps, vec!["⌘", "E"]);
    }

    #[test]
    fn parse_keycaps_two_modifiers_and_letter() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘⇧C");
        assert_eq!(keycaps, vec!["⌘", "⇧", "C"]);
    }

    #[test]
    fn parse_keycaps_enter_symbol() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("↵");
        assert_eq!(keycaps, vec!["↵"]);
    }

    #[test]
    fn parse_keycaps_cmd_enter() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘↵");
        assert_eq!(keycaps, vec!["⌘", "↵"]);
    }

    #[test]
    fn parse_keycaps_escape() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⎋");
        assert_eq!(keycaps, vec!["⎋"]);
    }

    #[test]
    fn parse_keycaps_space() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("␣");
        assert_eq!(keycaps, vec!["␣"]);
    }

    #[test]
    fn parse_keycaps_arrows() {
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("↑"), vec!["↑"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("↓"), vec!["↓"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("←"), vec!["←"]);
        assert_eq!(ActionsDialog::parse_shortcut_keycaps("→"), vec!["→"]);
    }

    #[test]
    fn parse_keycaps_all_four_modifiers() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘⌃⌥⇧X");
        assert_eq!(keycaps, vec!["⌘", "⌃", "⌥", "⇧", "X"]);
    }

    #[test]
    fn parse_keycaps_lowercase_uppercased() {
        let keycaps = ActionsDialog::parse_shortcut_keycaps("⌘a");
        assert_eq!(keycaps, vec!["⌘", "A"]);
    }

    // ============================================================
    // 19. Deeplink name edge cases
    // ============================================================

    #[test]
    fn deeplink_name_preserves_unicode_alphanumeric() {
        assert_eq!(to_deeplink_name("café"), "café");
    }

    #[test]
    fn deeplink_name_numbers_preserved() {
        assert_eq!(to_deeplink_name("script123"), "script123");
    }

    #[test]
    fn deeplink_name_all_special_returns_empty() {
        assert_eq!(to_deeplink_name("@#$%^&"), "");
    }

    #[test]
    fn deeplink_name_mixed_case_lowered() {
        assert_eq!(to_deeplink_name("Hello World"), "hello-world");
    }

    #[test]
    fn deeplink_name_consecutive_specials_collapsed() {
        assert_eq!(to_deeplink_name("a---b"), "a-b");
    }

    // ============================================================
    // 20. File context open title includes quoted name
    // ============================================================

    #[test]
    fn file_context_open_title_includes_filename() {
        let file_info = FileInfo {
            path: "/test/report.pdf".to_string(),
            name: "report.pdf".to_string(),
            file_type: FileType::Document,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);
        let open = find_action(&actions, "open_file").unwrap();
        assert!(
            open.title.contains("report.pdf"),
            "Open title should include filename: {}",
            open.title
        );
        assert!(
            open.title.contains('"'),
            "Open title should quote the filename"
        );
    }

    #[test]
    fn file_context_dir_title_includes_dirname() {
        let file_info = FileInfo {
            path: "/test/Documents".to_string(),
            name: "Documents".to_string(),
            file_type: FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&file_info);
        let open = find_action(&actions, "open_directory").unwrap();
        assert!(
            open.title.contains("Documents"),
            "Open title should include dirname: {}",
            open.title
        );
    }

    // ============================================================
    // 21. Chat context continue_in_chat always present
    // ============================================================

    #[test]
    fn chat_context_continue_in_chat_always_present() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(
            find_action(&actions, "continue_in_chat").is_some(),
            "continue_in_chat should always be present"
        );
    }

    #[test]
    fn chat_context_continue_in_chat_shortcut() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let action = find_action(&actions, "continue_in_chat").unwrap();
        assert_eq!(action.shortcut.as_deref(), Some("⌘↵"));
    }

