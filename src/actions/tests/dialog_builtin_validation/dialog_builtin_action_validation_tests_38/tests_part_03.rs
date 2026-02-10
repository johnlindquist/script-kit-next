    #[test]
    fn protocol_action_with_value_sets_name() {
        let pa = ProtocolAction::with_value("Test Action".into(), "test-val".into());
        assert_eq!(pa.name, "Test Action");
    }

    #[test]
    fn protocol_action_with_value_sets_value() {
        let pa = ProtocolAction::with_value("Test".into(), "my-value".into());
        assert_eq!(pa.value, Some("my-value".to_string()));
    }

    #[test]
    fn protocol_action_with_value_has_action_false() {
        let pa = ProtocolAction::with_value("Test".into(), "val".into());
        assert!(!pa.has_action);
    }

    #[test]
    fn protocol_action_with_value_defaults_visible_and_close_none() {
        let pa = ProtocolAction::with_value("Test".into(), "val".into());
        assert!(pa.visible.is_none());
        assert!(pa.close.is_none());
        // But is_visible() and should_close() default to true
        assert!(pa.is_visible());
        assert!(pa.should_close());
    }

    // =========================================================================
    // 20. score_action: whitespace and special character searches
    // =========================================================================

    #[test]
    fn score_action_whitespace_search_no_match() {
        let action = Action::new(
            "test",
            "Run Script",
            Some("Execute script".into()),
            ActionCategory::ScriptContext,
        );
        // Whitespace in search - "run script" is lowered and contains space
        let score = ActionsDialog::score_action(&action, "run script");
        // "run script" is a prefix of "run script" (title_lower)
        assert!(score >= 100);
    }

    #[test]
    fn score_action_dash_in_search() {
        let action = Action::new(
            "test",
            "copy-path",
            Some("Copy the path".into()),
            ActionCategory::ScriptContext,
        );
        let score = ActionsDialog::score_action(&action, "copy-");
        assert!(score >= 100); // prefix match
    }

    #[test]
    fn score_action_single_char_search() {
        let action = Action::new("test", "Run", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "r");
        assert!(score >= 100); // "r" is prefix of "run"
    }

    #[test]
    fn score_action_no_match_returns_zero() {
        let action = Action::new("test", "Run Script", None, ActionCategory::ScriptContext);
        let score = ActionsDialog::score_action(&action, "zzz");
        assert_eq!(score, 0);
    }

    // =========================================================================
    // 21. fuzzy_match: repeated and edge characters
    // =========================================================================

    #[test]
    fn fuzzy_match_repeated_chars_in_needle() {
        // "aab" should match "a_a_b_" since both a's and b are found in order
        assert!(ActionsDialog::fuzzy_match("a_a_b_", "aab"));
    }

    #[test]
    fn fuzzy_match_needle_equals_haystack() {
        assert!(ActionsDialog::fuzzy_match("exact", "exact"));
    }

    #[test]
    fn fuzzy_match_reverse_order_fails() {
        // "ba" should not match "ab" because b comes after a
        assert!(!ActionsDialog::fuzzy_match("ab", "ba"));
    }

    #[test]
    fn fuzzy_match_single_char() {
        assert!(ActionsDialog::fuzzy_match("hello", "h"));
    }

    // =========================================================================
    // 22. to_deeplink_name: numeric and edge cases
    // =========================================================================

    #[test]
    fn to_deeplink_name_numeric_only() {
        assert_eq!(to_deeplink_name("123"), "123");
    }

    #[test]
    fn to_deeplink_name_leading_trailing_spaces() {
        assert_eq!(to_deeplink_name("  hello  "), "hello");
    }

    #[test]
    fn to_deeplink_name_mixed_case() {
        assert_eq!(to_deeplink_name("Hello World"), "hello-world");
    }

    #[test]
    fn to_deeplink_name_underscores_to_hyphens() {
        assert_eq!(to_deeplink_name("my_cool_script"), "my-cool-script");
    }

    // =========================================================================
    // 23. CommandBarConfig: all styles have expected close defaults
    // =========================================================================

    #[test]
    fn command_bar_config_default_close_flags() {
        let cfg = CommandBarConfig::default();
        assert!(cfg.close_on_select);
        assert!(cfg.close_on_click_outside);
        assert!(cfg.close_on_escape);
    }

    #[test]
    fn command_bar_config_ai_style_inherits_close_flags() {
        let cfg = CommandBarConfig::ai_style();
        assert!(cfg.close_on_select);
        assert!(cfg.close_on_escape);
    }

    #[test]
    fn command_bar_config_main_menu_inherits_close_flags() {
        let cfg = CommandBarConfig::main_menu_style();
        assert!(cfg.close_on_select);
        assert!(cfg.close_on_escape);
    }

    #[test]
    fn command_bar_config_notes_inherits_close_flags() {
        let cfg = CommandBarConfig::notes_style();
        assert!(cfg.close_on_select);
        assert!(cfg.close_on_escape);
    }

    // =========================================================================
    // 24. ActionsDialogConfig Default trait
    // =========================================================================

    #[test]
    fn actions_dialog_config_default_search_bottom() {
        let cfg = crate::actions::types::ActionsDialogConfig::default();
        assert_eq!(
            cfg.search_position,
            crate::actions::types::SearchPosition::Bottom
        );
    }

    #[test]
    fn actions_dialog_config_default_section_style_separators() {
        let cfg = crate::actions::types::ActionsDialogConfig::default();
        assert_eq!(cfg.section_style, SectionStyle::Separators);
    }

    #[test]
    fn actions_dialog_config_default_anchor_bottom() {
        let cfg = crate::actions::types::ActionsDialogConfig::default();
        assert_eq!(cfg.anchor, AnchorPosition::Bottom);
    }

    #[test]
    fn actions_dialog_config_default_show_icons_false() {
        let cfg = crate::actions::types::ActionsDialogConfig::default();
        assert!(!cfg.show_icons);
        assert!(!cfg.show_footer);
    }

    // =========================================================================
    // 25. File context: reveal_in_finder always present
    // =========================================================================

    #[cfg(target_os = "macos")]
    #[test]
    fn file_context_reveal_in_finder_present_for_file() {
        let file = FileInfo {
            path: "/test.txt".into(),
            name: "test.txt".into(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn file_context_reveal_in_finder_present_for_dir() {
        let dir = FileInfo {
            path: "/docs".into(),
            name: "docs".into(),
            file_type: FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&dir);
        assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn file_context_reveal_shortcut() {
        let file = FileInfo {
            path: "/test.txt".into(),
            name: "test.txt".into(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
        assert_eq!(reveal.shortcut.as_deref(), Some("⌘↵"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn file_context_reveal_desc_mentions_finder() {
        let file = FileInfo {
            path: "/test.txt".into(),
            name: "test.txt".into(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
        assert!(reveal.description.as_ref().unwrap().contains("Finder"));
    }

    // =========================================================================
    // 26. Path context: primary action differences for file vs dir
    // =========================================================================

    #[test]
    fn path_context_file_primary_is_select() {
        let info = PathInfo::new("readme.md", "/home/readme.md", false);
        let actions = get_path_context_actions(&info);
        assert_eq!(actions[0].id, "select_file");
    }

    #[test]
    fn path_context_dir_primary_is_open_directory() {
        let info = PathInfo::new("docs", "/home/docs", true);
        let actions = get_path_context_actions(&info);
        assert_eq!(actions[0].id, "open_directory");
    }

    #[test]
    fn path_context_file_primary_title_quotes_name() {
        let info = PathInfo::new("data.csv", "/home/data.csv", false);
        let actions = get_path_context_actions(&info);
        assert!(actions[0].title.contains("\"data.csv\""));
    }

    #[test]
    fn path_context_dir_primary_title_quotes_name() {
        let info = PathInfo::new("src", "/home/src", true);
        let actions = get_path_context_actions(&info);
        assert!(actions[0].title.contains("\"src\""));
    }

    // =========================================================================
    // 27. Script context: builtin has exactly 4 actions (no shortcut/alias)
    // =========================================================================

    #[test]
    fn builtin_script_action_count_no_extras() {
        let builtin = ScriptInfo::builtin("Clipboard History");
        let actions = get_script_context_actions(&builtin);
        // run_script + add_shortcut + add_alias + copy_deeplink = 4
        assert_eq!(actions.len(), 4);
    }

    #[test]
    fn builtin_script_no_edit_actions() {
        let builtin = ScriptInfo::builtin("App Launcher");
        let actions = get_script_context_actions(&builtin);
        assert!(!actions.iter().any(|a| a.id == "edit_script"));
        assert!(!actions.iter().any(|a| a.id == "edit_scriptlet"));
    }

    #[test]
    fn builtin_script_no_reveal_or_copy_path() {
        let builtin = ScriptInfo::builtin("App Launcher");
        let actions = get_script_context_actions(&builtin);
        assert!(!actions.iter().any(|a| a.id == "reveal_in_finder"));
        assert!(!actions.iter().any(|a| a.id == "copy_path"));
    }

    #[test]
    fn builtin_script_has_copy_deeplink() {
        let builtin = ScriptInfo::builtin("App Launcher");
        let actions = get_script_context_actions(&builtin);
        assert!(actions.iter().any(|a| a.id == "copy_deeplink"));
    }

    // =========================================================================
    // 28. Script context: agent actions include specific IDs
    // =========================================================================

    #[test]
    fn agent_script_has_edit_agent_title() {
        let mut info = ScriptInfo::new("my-agent", "/agents/my-agent.md");
        info.is_script = false;
        info.is_agent = true;
        let actions = get_script_context_actions(&info);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit.title, "Edit Agent");
    }

    #[test]
    fn agent_script_desc_mentions_agent() {
        let mut info = ScriptInfo::new("my-agent", "/agents/my-agent.md");
        info.is_script = false;
        info.is_agent = true;
        let actions = get_script_context_actions(&info);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert!(edit.description.as_ref().unwrap().contains("agent"));
    }

    #[test]
    fn agent_script_has_copy_content() {
        let mut info = ScriptInfo::new("my-agent", "/agents/my-agent.md");
        info.is_script = false;
        info.is_agent = true;
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "copy_content"));
    }

    #[test]
    fn agent_script_no_view_logs() {
        let mut info = ScriptInfo::new("my-agent", "/agents/my-agent.md");
        info.is_script = false;
        info.is_agent = true;
        let actions = get_script_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
    }

    // =========================================================================
    // 29. Scriptlet context: copy_scriptlet_path details
    // =========================================================================

    #[test]
    fn scriptlet_copy_path_id() {
        let info = ScriptInfo::scriptlet("Test", "/path.md#test", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        assert!(actions.iter().any(|a| a.id == "copy_scriptlet_path"));
    }

    #[test]
    fn scriptlet_copy_path_shortcut() {
        let info = ScriptInfo::scriptlet("Test", "/path.md#test", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        let cp = actions
            .iter()
            .find(|a| a.id == "copy_scriptlet_path")
            .unwrap();
        assert_eq!(cp.shortcut.as_deref(), Some("⌘⇧C"));
    }

    #[test]
    fn scriptlet_copy_path_desc_mentions_path() {
        let info = ScriptInfo::scriptlet("Test", "/path.md#test", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        let cp = actions
            .iter()
            .find(|a| a.id == "copy_scriptlet_path")
            .unwrap();
        assert!(cp.description.as_ref().unwrap().contains("path"));
    }

    #[test]
    fn scriptlet_edit_scriptlet_desc_mentions_editor() {
        let info = ScriptInfo::scriptlet("Test", "/path.md#test", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        let edit = actions.iter().find(|a| a.id == "edit_scriptlet").unwrap();
        assert!(edit.description.as_ref().unwrap().contains("$EDITOR"));
    }

    // =========================================================================
    // 30. Cross-context: all action titles are non-empty and IDs are non-empty
    // =========================================================================

    #[test]
    fn cross_context_script_all_titles_and_ids_nonempty() {
        let script = ScriptInfo::new("test", "/test.ts");
        let actions = get_script_context_actions(&script);
        for action in &actions {
            assert!(
                !action.title.is_empty(),
                "Empty title for id: {}",
                action.id
            );
            assert!(!action.id.is_empty(), "Empty id found");
        }
    }

    #[test]
    fn cross_context_clipboard_all_titles_and_ids_nonempty() {
        let entry = ClipboardEntryInfo {
            id: "1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "test".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        for action in &actions {
            assert!(
                !action.title.is_empty(),
                "Empty title for id: {}",
                action.id
            );
            assert!(!action.id.is_empty(), "Empty id found");
        }
    }

    #[test]
    fn cross_context_ai_bar_all_titles_and_ids_nonempty() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            assert!(
                !action.title.is_empty(),
                "Empty title for id: {}",
                action.id
            );
            assert!(!action.id.is_empty(), "Empty id found");
        }
    }

