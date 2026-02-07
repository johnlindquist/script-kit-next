    #[test]
    fn path_context_action_count_dir_vs_file() {
        let dir_info = PathInfo {
            path: "/test/dir".into(),
            name: "dir".into(),
            is_dir: true,
        };
        let file_info = PathInfo {
            path: "/test/file.txt".into(),
            name: "file.txt".into(),
            is_dir: false,
        };
        let dir_actions = get_path_context_actions(&dir_info);
        let file_actions = get_path_context_actions(&file_info);
        // Both should have same count (primary + copy_path + open_in_finder + open_in_editor
        // + open_in_terminal + copy_filename + move_to_trash = 7)
        assert_eq!(dir_actions.len(), file_actions.len());
        assert_eq!(dir_actions.len(), 7);
    }

    // =========================================================================
    // 14. File context FileType variants
    // =========================================================================

    #[test]
    fn file_context_document_type() {
        let info = FileInfo {
            path: "/test/doc.pdf".into(),
            name: "doc.pdf".into(),
            file_type: FileType::Document,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        assert_eq!(actions[0].id, "open_file");
        assert!(actions[0].title.contains("doc.pdf"));
    }

    #[test]
    fn file_context_image_type() {
        let info = FileInfo {
            path: "/test/photo.jpg".into(),
            name: "photo.jpg".into(),
            file_type: FileType::Image,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        assert_eq!(actions[0].id, "open_file");
    }

    #[test]
    fn file_context_audio_type() {
        let info = FileInfo {
            path: "/test/song.mp3".into(),
            name: "song.mp3".into(),
            file_type: FileType::Audio,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        assert_eq!(actions[0].id, "open_file");
    }

    #[test]
    fn file_context_video_type() {
        let info = FileInfo {
            path: "/test/movie.mp4".into(),
            name: "movie.mp4".into(),
            file_type: FileType::Video,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        assert_eq!(actions[0].id, "open_file");
    }

    #[test]
    fn file_context_application_type() {
        let info = FileInfo {
            path: "/Applications/Safari.app".into(),
            name: "Safari.app".into(),
            file_type: FileType::Application,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        assert_eq!(actions[0].id, "open_file");
        assert!(actions[0].title.contains("Safari.app"));
    }

    #[test]
    fn file_context_directory_type() {
        let info = FileInfo {
            path: "/test/folder".into(),
            name: "folder".into(),
            file_type: FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&info);
        assert_eq!(actions[0].id, "open_directory");
        assert!(actions[0].title.contains("folder"));
    }

    #[test]
    fn file_context_other_type() {
        let info = FileInfo {
            path: "/test/unknown.xyz".into(),
            name: "unknown.xyz".into(),
            file_type: FileType::Other,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        assert_eq!(actions[0].id, "open_file");
    }

    // =========================================================================
    // 15. Action builder chaining immutability
    // =========================================================================

    #[test]
    fn action_with_shortcut_preserves_other_fields() {
        let action = Action::new(
            "test",
            "Test Title",
            Some("Test Description".into()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘T");
        assert_eq!(action.id, "test");
        assert_eq!(action.title, "Test Title");
        assert_eq!(action.description, Some("Test Description".into()));
        assert_eq!(action.shortcut, Some("⌘T".into()));
    }

    #[test]
    fn action_with_icon_preserves_other_fields() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘T")
            .with_icon(IconName::Copy);
        assert_eq!(action.shortcut, Some("⌘T".into()));
        assert_eq!(action.icon, Some(IconName::Copy));
    }

    #[test]
    fn action_with_section_preserves_other_fields() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘T")
            .with_icon(IconName::Copy)
            .with_section("MySection");
        assert_eq!(action.shortcut, Some("⌘T".into()));
        assert_eq!(action.icon, Some(IconName::Copy));
        assert_eq!(action.section, Some("MySection".into()));
    }

    #[test]
    fn action_with_shortcut_opt_none_leaves_shortcut_none() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_shortcut_opt(None);
        assert!(action.shortcut.is_none());
        assert!(action.shortcut_lower.is_none());
    }

    #[test]
    fn action_with_shortcut_opt_some_sets_shortcut() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext)
            .with_shortcut_opt(Some("⌘K".into()));
        assert_eq!(action.shortcut, Some("⌘K".into()));
        assert_eq!(action.shortcut_lower, Some("⌘k".into()));
    }

    // =========================================================================
    // 16. Action lowercase cache correctness
    // =========================================================================

    #[test]
    fn title_lower_matches_title_to_lowercase() {
        let action = Action::new(
            "test",
            "Copy Path To Clipboard",
            None,
            ActionCategory::ScriptContext,
        );
        assert_eq!(action.title_lower, "copy path to clipboard");
    }

    #[test]
    fn description_lower_matches_description_to_lowercase() {
        let action = Action::new(
            "test",
            "Test",
            Some("Open In $EDITOR".into()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(action.description_lower, Some("open in $editor".into()));
    }

    #[test]
    fn description_lower_none_when_no_description() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext);
        assert!(action.description_lower.is_none());
    }

    #[test]
    fn shortcut_lower_set_by_with_shortcut() {
        let action =
            Action::new("test", "Test", None, ActionCategory::ScriptContext).with_shortcut("⌘⇧C");
        assert_eq!(action.shortcut_lower, Some("⌘⇧c".into()));
    }

    #[test]
    fn shortcut_lower_none_when_no_shortcut() {
        let action = Action::new("test", "Test", None, ActionCategory::ScriptContext);
        assert!(action.shortcut_lower.is_none());
    }

    // =========================================================================
    // 17. CommandBarConfig default field values
    // =========================================================================

    #[test]
    fn commandbar_default_close_flags_all_true() {
        let config = CommandBarConfig::default();
        assert!(config.close_on_select);
        assert!(config.close_on_click_outside);
        assert!(config.close_on_escape);
    }

    #[test]
    fn commandbar_ai_style_search_at_top() {
        let config = CommandBarConfig::ai_style();
        assert_eq!(config.dialog_config.search_position, SearchPosition::Top);
        assert_eq!(config.dialog_config.section_style, SectionStyle::Headers);
        assert!(config.dialog_config.show_icons);
        assert!(config.dialog_config.show_footer);
    }

    #[test]
    fn commandbar_main_menu_search_at_bottom() {
        let config = CommandBarConfig::main_menu_style();
        assert_eq!(config.dialog_config.search_position, SearchPosition::Bottom);
        assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
        assert!(!config.dialog_config.show_icons);
        assert!(!config.dialog_config.show_footer);
    }

    #[test]
    fn commandbar_no_search_hidden() {
        let config = CommandBarConfig::no_search();
        assert_eq!(config.dialog_config.search_position, SearchPosition::Hidden);
    }

    #[test]
    fn commandbar_notes_style_search_at_top_with_separators() {
        let config = CommandBarConfig::notes_style();
        assert_eq!(config.dialog_config.search_position, SearchPosition::Top);
        assert_eq!(config.dialog_config.section_style, SectionStyle::Separators);
        assert!(config.dialog_config.show_icons);
        assert!(config.dialog_config.show_footer);
    }

    // =========================================================================
    // 18. Scriptlet context vs script context action comparison
    // =========================================================================

    #[test]
    fn scriptlet_context_has_edit_scriptlet_not_edit_script() {
        let info = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"edit_scriptlet"));
        assert!(!ids.contains(&"edit_script"));
    }

    #[test]
    fn scriptlet_context_has_reveal_scriptlet_not_reveal() {
        let info = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"reveal_scriptlet_in_finder"));
        assert!(!ids.contains(&"reveal_in_finder"));
    }

    #[test]
    fn scriptlet_context_has_copy_scriptlet_path_not_copy_path() {
        let info = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"copy_scriptlet_path"));
        assert!(!ids.contains(&"copy_path"));
    }

    #[test]
    fn scriptlet_context_has_copy_content() {
        let info = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"copy_content"));
    }

    #[test]
    fn scriptlet_context_with_custom_actions_interleaved() {
        let info = ScriptInfo::scriptlet("Test", "/path/test.md", None, None);
        let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo test".into());
        scriptlet.actions = vec![
            ScriptletAction {
                name: "Action A".into(),
                command: "action-a".into(),
                tool: "bash".into(),
                code: "echo a".into(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
            ScriptletAction {
                name: "Action B".into(),
                command: "action-b".into(),
                tool: "bash".into(),
                code: "echo b".into(),
                inputs: vec![],
                shortcut: Some("cmd+b".into()),
                description: Some("Do B".into()),
            },
        ];
        let actions = get_scriptlet_context_actions_with_custom(&info, Some(&scriptlet));

        // run_script first
        assert_eq!(actions[0].id, "run_script");
        // Then custom actions
        assert_eq!(actions[1].id, "scriptlet_action:action-a");
        assert_eq!(actions[2].id, "scriptlet_action:action-b");
        // Custom actions have has_action=true
        assert!(actions[1].has_action);
        assert!(actions[2].has_action);
        // Custom action B has shortcut formatted
        assert!(actions[2].shortcut.is_some());
        // Custom action B has description
        assert_eq!(actions[2].description.as_deref(), Some("Do B"));
    }

    #[test]
    fn scriptlet_context_with_shortcut_and_alias() {
        let info = ScriptInfo::scriptlet(
            "Test",
            "/path/test.md",
            Some("cmd+t".into()),
            Some("ts".into()),
        );
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"update_shortcut"));
        assert!(ids.contains(&"remove_shortcut"));
        assert!(ids.contains(&"update_alias"));
        assert!(ids.contains(&"remove_alias"));
        assert!(!ids.contains(&"add_shortcut"));
        assert!(!ids.contains(&"add_alias"));
    }

    #[test]
    fn scriptlet_context_with_frecency() {
        let info = ScriptInfo::scriptlet("Test", "/path/test.md", None, None)
            .with_frecency(true, Some("scriptlet:Test".into()));
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"reset_ranking"));
    }

    // =========================================================================
    // 19. AI command bar actions detailed validation
    // =========================================================================

    #[test]
    fn ai_command_bar_has_exactly_12_actions() {
        let actions = get_ai_command_bar_actions();
        assert_eq!(actions.len(), 12);
    }

    #[test]
    fn ai_command_bar_response_section_actions() {
        let actions = get_ai_command_bar_actions();
        let response_actions: Vec<&Action> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Response"))
            .collect();
        assert_eq!(response_actions.len(), 3);
        let ids: Vec<&str> = response_actions.iter().map(|a| a.id.as_str()).collect();
        assert!(ids.contains(&"copy_response"));
        assert!(ids.contains(&"copy_chat"));
        assert!(ids.contains(&"copy_last_code"));
    }

    #[test]
    fn ai_command_bar_actions_section_actions() {
        let actions = get_ai_command_bar_actions();
        let action_section: Vec<&Action> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Actions"))
            .collect();
        assert_eq!(action_section.len(), 4);
        let ids: Vec<&str> = action_section.iter().map(|a| a.id.as_str()).collect();
        assert!(ids.contains(&"submit"));
        assert!(ids.contains(&"new_chat"));
        assert!(ids.contains(&"delete_chat"));
        assert!(ids.contains(&"branch_from_last"));
    }

    #[test]
    fn ai_command_bar_attachments_section_actions() {
        let actions = get_ai_command_bar_actions();
        let att_actions: Vec<&Action> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Attachments"))
            .collect();
        assert_eq!(att_actions.len(), 2);
    }

    #[test]
    fn ai_command_bar_settings_section_actions() {
        let actions = get_ai_command_bar_actions();
        let settings_actions: Vec<&Action> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Settings"))
            .collect();
        assert_eq!(settings_actions.len(), 1);
        assert_eq!(settings_actions[0].id, "change_model");
    }

    #[test]
    fn ai_command_bar_all_actions_have_icons() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            assert!(
                action.icon.is_some(),
                "AI action '{}' should have icon",
                action.id
            );
        }
    }

