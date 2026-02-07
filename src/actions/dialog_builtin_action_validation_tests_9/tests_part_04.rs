    #[test]
    fn chat_context_copy_response_only_when_has_response() {
        let no_response = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: false,
        };
        let with_response = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: true,
        };
        let actions_no = get_chat_context_actions(&no_response);
        let actions_yes = get_chat_context_actions(&with_response);
        assert!(find_action(&actions_no, "copy_response").is_none());
        assert!(find_action(&actions_yes, "copy_response").is_some());
    }

    #[test]
    fn chat_context_clear_conversation_only_when_has_messages() {
        let no_messages = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: true,
        };
        let with_messages = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: true,
        };
        let actions_no = get_chat_context_actions(&no_messages);
        let actions_yes = get_chat_context_actions(&with_messages);
        assert!(find_action(&actions_no, "clear_conversation").is_none());
        assert!(find_action(&actions_yes, "clear_conversation").is_some());
    }

    // ============================================================
    // 22. Scriptlet context copy_content always present
    // ============================================================

    #[test]
    fn scriptlet_context_has_copy_content() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        assert!(
            find_action(&actions, "copy_content").is_some(),
            "Scriptlet context should always have copy_content"
        );
    }

    #[test]
    fn scriptlet_context_copy_content_shortcut() {
        let script = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let action = find_action(&actions, "copy_content").unwrap();
        assert_eq!(action.shortcut.as_deref(), Some("⌘⌥C"));
    }

    #[test]
    fn scriptlet_context_has_copy_deeplink() {
        let script = ScriptInfo::scriptlet("My Scriptlet", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&script, None);
        let deeplink = find_action(&actions, "copy_deeplink").unwrap();
        assert!(
            deeplink
                .description
                .as_ref()
                .unwrap()
                .contains("my-scriptlet"),
            "Deeplink description should contain deeplink name"
        );
    }

    // ============================================================
    // 23. Notes command bar icon name validation
    // ============================================================

    #[test]
    fn notes_command_bar_all_actions_have_icons() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        for action in &actions {
            assert!(
                action.icon.is_some(),
                "Notes action '{}' should have an icon",
                action.id
            );
        }
    }

    #[test]
    fn notes_command_bar_new_note_has_plus_icon() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let new_note = find_action(&actions, "new_note").unwrap();
        assert_eq!(new_note.icon, Some(IconName::Plus));
    }

    #[test]
    fn notes_command_bar_browse_notes_has_folder_icon() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let browse = find_action(&actions, "browse_notes").unwrap();
        assert_eq!(browse.icon, Some(IconName::FolderOpen));
    }

    // ============================================================
    // 24. Cross-context action count stability
    // ============================================================

    #[test]
    fn script_context_action_count_deterministic() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let count1 = get_script_context_actions(&script).len();
        let count2 = get_script_context_actions(&script).len();
        assert_eq!(count1, count2, "Same input should produce same count");
    }

    #[test]
    fn clipboard_context_action_count_deterministic() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let count1 = get_clipboard_history_context_actions(&entry).len();
        let count2 = get_clipboard_history_context_actions(&entry).len();
        assert_eq!(count1, count2);
    }

    #[test]
    fn path_context_dir_and_file_same_count() {
        let dir = PathInfo {
            name: "test".to_string(),
            path: "/test".to_string(),
            is_dir: true,
        };
        let file = PathInfo {
            name: "test.txt".to_string(),
            path: "/test.txt".to_string(),
            is_dir: false,
        };
        let dir_count = get_path_context_actions(&dir).len();
        let file_count = get_path_context_actions(&file).len();
        assert_eq!(
            dir_count, file_count,
            "Dir and file path contexts should have same action count"
        );
    }

    // ============================================================
    // 25. Action builder chaining
    // ============================================================

    #[test]
    fn action_with_shortcut_opt_none_preserves_fields() {
        let action = Action::new(
            "test",
            "Test",
            Some("Desc".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_icon(IconName::Star)
        .with_section("MySection")
        .with_shortcut_opt(None);
        assert_eq!(action.icon, Some(IconName::Star));
        assert_eq!(action.section.as_deref(), Some("MySection"));
        assert!(action.shortcut.is_none());
    }

    #[test]
    fn action_with_shortcut_opt_some_sets_shortcut() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_shortcut_opt(Some("⌘Z".to_string()));
        assert_eq!(action.shortcut.as_deref(), Some("⌘Z"));
        assert_eq!(action.shortcut_lower.as_deref(), Some("⌘z"));
    }

    #[test]
    fn action_with_icon_and_section_order_independent() {
        let a1 = Action::new("t", "T", None, ActionCategory::ScriptContext)
            .with_icon(IconName::Code)
            .with_section("S");
        let a2 = Action::new("t", "T", None, ActionCategory::ScriptContext)
            .with_section("S")
            .with_icon(IconName::Code);
        assert_eq!(a1.icon, a2.icon);
        assert_eq!(a1.section, a2.section);
    }

    // ============================================================
    // 26. ActionsDialogConfig and enum defaults
    // ============================================================

    #[test]
    fn actions_dialog_config_default_values() {
        let config = ActionsDialogConfig::default();
        assert_eq!(config.search_position, SearchPosition::Bottom);
        assert_eq!(config.section_style, SectionStyle::Separators);
        assert_eq!(config.anchor, AnchorPosition::Bottom);
        assert!(!config.show_icons);
        assert!(!config.show_footer);
    }

    #[test]
    fn search_position_hidden_not_eq_top_or_bottom() {
        assert_ne!(SearchPosition::Hidden, SearchPosition::Top);
        assert_ne!(SearchPosition::Hidden, SearchPosition::Bottom);
    }

    #[test]
    fn section_style_headers_not_eq_separators() {
        assert_ne!(SectionStyle::Headers, SectionStyle::Separators);
        assert_ne!(SectionStyle::Headers, SectionStyle::None);
    }

    // ============================================================
    // 27. Clipboard action IDs all prefixed
    // ============================================================

    #[test]
    fn all_clipboard_action_ids_prefixed() {
        let entry = make_clipboard_entry(ContentType::Image, true, Some("Safari"));
        let actions = get_clipboard_history_context_actions(&entry);
        for action in &actions {
            assert!(
                action.id.starts_with("clipboard_"),
                "Clipboard action ID '{}' should start with 'clipboard_'",
                action.id
            );
        }
    }

    // ============================================================
    // 28. Fuzzy match edge cases
    // ============================================================

    #[test]
    fn fuzzy_match_empty_needle_matches() {
        assert!(ActionsDialog::fuzzy_match("anything", ""));
    }

    #[test]
    fn fuzzy_match_empty_haystack_empty_needle() {
        assert!(ActionsDialog::fuzzy_match("", ""));
    }

    #[test]
    fn fuzzy_match_empty_haystack_nonempty_needle() {
        assert!(!ActionsDialog::fuzzy_match("", "x"));
    }

    #[test]
    fn fuzzy_match_subsequence() {
        assert!(ActionsDialog::fuzzy_match("edit script", "esi"));
    }

    #[test]
    fn fuzzy_match_no_subsequence() {
        assert!(!ActionsDialog::fuzzy_match("abc", "abd"));
    }

    #[test]
    fn fuzzy_match_exact() {
        assert!(ActionsDialog::fuzzy_match("test", "test"));
    }

    #[test]
    fn fuzzy_match_needle_longer_than_haystack() {
        assert!(!ActionsDialog::fuzzy_match("ab", "abc"));
    }

    // ============================================================
    // 29. has_action=false invariant for all builtin contexts
    // ============================================================

    #[test]
    fn script_context_all_has_action_false() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        for action in &get_script_context_actions(&script) {
            assert!(
                !action.has_action,
                "Script builtin '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn clipboard_context_all_has_action_false() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        for action in &get_clipboard_history_context_actions(&entry) {
            assert!(
                !action.has_action,
                "Clipboard builtin '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn path_context_all_has_action_false() {
        let path = PathInfo {
            name: "test".to_string(),
            path: "/test".to_string(),
            is_dir: false,
        };
        for action in &get_path_context_actions(&path) {
            assert!(
                !action.has_action,
                "Path builtin '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn file_context_all_has_action_false() {
        let file = FileInfo {
            path: "/test/f.txt".to_string(),
            name: "f.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        for action in &get_file_context_actions(&file) {
            assert!(
                !action.has_action,
                "File builtin '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn ai_command_bar_all_has_action_false() {
        for action in &get_ai_command_bar_actions() {
            assert!(
                !action.has_action,
                "AI builtin '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn notes_command_bar_all_has_action_false() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        for action in &get_notes_command_bar_actions(&info) {
            assert!(
                !action.has_action,
                "Notes builtin '{}' should have has_action=false",
                action.id
            );
        }
    }

    // ============================================================
    // 30. ID uniqueness across contexts
    // ============================================================

    #[test]
    fn script_context_ids_unique() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        let ids: HashSet<_> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn clipboard_context_ids_unique() {
        let entry = make_clipboard_entry(ContentType::Image, true, Some("App"));
        let actions = get_clipboard_history_context_actions(&entry);
        let ids: HashSet<_> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn path_context_ids_unique() {
        let path = PathInfo {
            name: "dir".to_string(),
            path: "/dir".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&path);
        let ids: HashSet<_> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn ai_command_bar_ids_no_overlap_with_notes() {
        let ai_actions = get_ai_command_bar_actions();
        let notes_info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let notes_actions = get_notes_command_bar_actions(&notes_info);
        let ai_ids: HashSet<_> = ai_actions.iter().map(|a| a.id.as_str()).collect();
        let notes_ids: HashSet<_> = notes_actions.iter().map(|a| a.id.as_str()).collect();
        let overlap: Vec<_> = ai_ids.intersection(&notes_ids).collect();
        // copy_deeplink appears in both — that's expected, it's the same action concept
        // But most should be unique
        assert!(
            overlap.len() <= 1,
            "AI and Notes should have minimal ID overlap, found: {:?}",
            overlap
        );
    }

    // ============================================================
    // Additional: title_lower/description_lower caching
    // ============================================================

    #[test]
    fn action_title_lower_computed_on_creation() {
        let action = Action::new("test", "Edit Script", None, ActionCategory::ScriptContext);
        assert_eq!(action.title_lower, "edit script");
    }

    #[test]
    fn action_description_lower_computed_on_creation() {
        let action = Action::new(
            "test",
            "Test",
            Some("Open in $EDITOR".to_string()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(
            action.description_lower,
            Some("open in $editor".to_string())
        );
    }

