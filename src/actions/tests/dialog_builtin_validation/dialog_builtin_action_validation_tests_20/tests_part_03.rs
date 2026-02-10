    #[test]
    fn cat15_same_section_no_duplicate_header() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S1"),
        ];
        let filtered: Vec<usize> = (0..actions.len()).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        // Should have: Header("S1"), Item(0), Item(1) — only one header
        assert_eq!(grouped.len(), 3);
        let header_count = grouped
            .iter()
            .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 1);
    }

    #[test]
    fn cat15_separators_no_headers() {
        let actions = vec![
            Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1"),
            Action::new("b", "B", None, ActionCategory::ScriptContext).with_section("S2"),
        ];
        let filtered: Vec<usize> = (0..actions.len()).collect();
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Separators);
        let header_count = grouped
            .iter()
            .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 0);
    }

    #[test]
    fn cat15_empty_filtered_empty_result() {
        let actions = vec![Action::new("a", "A", None, ActionCategory::ScriptContext)];
        let filtered: Vec<usize> = vec![];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::Headers);
        assert!(grouped.is_empty());
    }

    #[test]
    fn cat15_none_style_no_headers() {
        let actions =
            vec![Action::new("a", "A", None, ActionCategory::ScriptContext).with_section("S1")];
        let filtered: Vec<usize> = vec![0];
        let grouped = build_grouped_items_static(&actions, &filtered, SectionStyle::None);
        let header_count = grouped
            .iter()
            .filter(|g| matches!(g, GroupedActionItem::SectionHeader(_)))
            .count();
        assert_eq!(header_count, 0);
    }

    // =========================================================================
    // Category 16: coerce_action_selection skips headers
    // =========================================================================

    #[test]
    fn cat16_item_stays() {
        let rows = vec![GroupedActionItem::Item(0)];
        assert_eq!(coerce_action_selection(&rows, 0), Some(0));
    }

    #[test]
    fn cat16_header_skips_down() {
        let rows = vec![
            GroupedActionItem::SectionHeader("S".into()),
            GroupedActionItem::Item(0),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
    }

    #[test]
    fn cat16_trailing_header_skips_up() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("S".into()),
        ];
        assert_eq!(coerce_action_selection(&rows, 1), Some(0));
    }

    #[test]
    fn cat16_all_headers_none() {
        let rows = vec![
            GroupedActionItem::SectionHeader("S1".into()),
            GroupedActionItem::SectionHeader("S2".into()),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    #[test]
    fn cat16_empty_rows_none() {
        let rows: Vec<GroupedActionItem> = vec![];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    // =========================================================================
    // Category 17: CommandBarConfig preset field matrix
    // =========================================================================

    #[test]
    fn cat17_default_close_on_select() {
        let config = CommandBarConfig::default();
        assert!(config.close_on_select);
        assert!(config.close_on_escape);
    }

    #[test]
    fn cat17_ai_style_top_search() {
        let config = CommandBarConfig::ai_style();
        assert!(matches!(
            config.dialog_config.search_position,
            SearchPosition::Top
        ));
    }

    #[test]
    fn cat17_main_menu_bottom_search() {
        let config = CommandBarConfig::main_menu_style();
        assert!(matches!(
            config.dialog_config.search_position,
            SearchPosition::Bottom
        ));
    }

    #[test]
    fn cat17_no_search_hidden() {
        let config = CommandBarConfig::no_search();
        assert!(matches!(
            config.dialog_config.search_position,
            SearchPosition::Hidden
        ));
    }

    #[test]
    fn cat17_notes_style_separators() {
        let config = CommandBarConfig::notes_style();
        assert!(matches!(
            config.dialog_config.section_style,
            SectionStyle::Separators
        ));
        assert!(config.dialog_config.show_icons);
        assert!(config.dialog_config.show_footer);
    }

    // =========================================================================
    // Category 18: Action builder with_shortcut_opt behavior
    // =========================================================================

    #[test]
    fn cat18_with_shortcut_opt_none_preserves() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘A")
            .with_shortcut_opt(None);
        // None should not clear the existing shortcut
        assert_eq!(action.shortcut, Some("⌘A".to_string()));
    }

    #[test]
    fn cat18_with_shortcut_opt_some_sets() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext)
            .with_shortcut_opt(Some("⌘B".to_string()));
        assert_eq!(action.shortcut, Some("⌘B".to_string()));
    }

    #[test]
    fn cat18_with_icon_preserves_shortcut() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘C")
            .with_icon(IconName::Copy);
        assert_eq!(action.shortcut, Some("⌘C".to_string()));
        assert_eq!(action.icon, Some(IconName::Copy));
    }

    #[test]
    fn cat18_with_section_preserves_all() {
        let action = Action::new("id", "T", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘D")
            .with_icon(IconName::Plus)
            .with_section("MySection");
        assert_eq!(action.shortcut, Some("⌘D".to_string()));
        assert_eq!(action.icon, Some(IconName::Plus));
        assert_eq!(action.section, Some("MySection".to_string()));
    }

    // =========================================================================
    // Category 19: ScriptInfo with_is_script vs with_action_verb
    // =========================================================================

    #[test]
    fn cat19_with_is_script_true() {
        let s = ScriptInfo::with_is_script("test", "/path", true);
        assert!(s.is_script);
        assert_eq!(s.action_verb, "Run");
    }

    #[test]
    fn cat19_with_is_script_false() {
        let s = ScriptInfo::with_is_script("app", "/app", false);
        assert!(!s.is_script);
    }

    #[test]
    fn cat19_with_action_verb_custom() {
        let s = ScriptInfo::with_action_verb("Window", "w:1", false, "Switch to");
        assert_eq!(s.action_verb, "Switch to");
    }

    #[test]
    fn cat19_with_action_verb_and_shortcut() {
        let s = ScriptInfo::with_action_verb_and_shortcut(
            "W",
            "w:1",
            false,
            "Launch",
            Some("cmd+l".to_string()),
        );
        assert_eq!(s.action_verb, "Launch");
        assert_eq!(s.shortcut, Some("cmd+l".to_string()));
    }

    // =========================================================================
    // Category 20: Notes command bar conditional actions per flags
    // =========================================================================

    #[test]
    fn cat20_duplicate_note_requires_selection_no_trash() {
        let yes = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&yes);
        assert!(actions.iter().any(|a| a.id == "duplicate_note"));

        let no_sel = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions2 = get_notes_command_bar_actions(&no_sel);
        assert!(!actions2.iter().any(|a| a.id == "duplicate_note"));
    }

    #[test]
    fn cat20_find_in_note_absent_in_trash() {
        let trash = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&trash);
        assert!(!actions.iter().any(|a| a.id == "find_in_note"));
    }

    #[test]
    fn cat20_export_requires_selection_no_trash() {
        let yes = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&yes);
        assert!(actions.iter().any(|a| a.id == "export"));

        let no = NotesInfo {
            has_selection: true,
            is_trash_view: true,
            auto_sizing_enabled: false,
        };
        let actions2 = get_notes_command_bar_actions(&no);
        assert!(!actions2.iter().any(|a| a.id == "export"));
    }

    #[test]
    fn cat20_auto_sizing_absent_when_enabled() {
        let enabled = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&enabled);
        assert!(!actions.iter().any(|a| a.id == "enable_auto_sizing"));
    }

    #[test]
    fn cat20_auto_sizing_present_when_disabled() {
        let disabled = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&disabled);
        assert!(actions.iter().any(|a| a.id == "enable_auto_sizing"));
    }

    // =========================================================================
    // Category 21: Clipboard share and attach_to_ai universality
    // Both text and image entries should have these actions.
    // =========================================================================

    #[test]
    fn cat21_text_has_share() {
        let entry = ClipboardEntryInfo {
            id: "t".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hi".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_share"));
    }

    #[test]
    fn cat21_image_has_share() {
        let entry = ClipboardEntryInfo {
            id: "i".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "img".into(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_share"));
    }

    #[test]
    fn cat21_text_has_attach_to_ai() {
        let entry = ClipboardEntryInfo {
            id: "t".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hi".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_attach_to_ai"));
    }

    #[test]
    fn cat21_image_has_attach_to_ai() {
        let entry = ClipboardEntryInfo {
            id: "i".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "img".into(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        assert!(actions.iter().any(|a| a.id == "clipboard_attach_to_ai"));
    }

    #[test]
    fn cat21_share_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "t".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hi".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let s = actions.iter().find(|a| a.id == "clipboard_share").unwrap();
        assert_eq!(s.shortcut.as_ref().unwrap(), "⇧⌘E");
    }

    // =========================================================================
    // Category 22: File context directory lacks quick_look (macOS)
    // =========================================================================

    #[cfg(target_os = "macos")]
    #[test]
    fn cat22_file_dir_no_quick_look() {
        let file_info = FileInfo {
            path: "/test/Dir".into(),
            name: "Dir".into(),
            file_type: FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&file_info);
        assert!(!actions.iter().any(|a| a.id == "quick_look"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat22_file_file_has_quick_look() {
        let file_info = FileInfo {
            path: "/test/f.txt".into(),
            name: "f.txt".into(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);
        assert!(actions.iter().any(|a| a.id == "quick_look"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat22_file_dir_has_open_with() {
        let file_info = FileInfo {
            path: "/test/Dir".into(),
            name: "Dir".into(),
            file_type: FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&file_info);
        assert!(actions.iter().any(|a| a.id == "open_with"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat22_file_dir_has_show_info() {
        let file_info = FileInfo {
            path: "/test/Dir".into(),
            name: "Dir".into(),
            file_type: FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&file_info);
        assert!(actions.iter().any(|a| a.id == "show_info"));
    }

    // =========================================================================
    // Category 23: Scriptlet defined actions from H3 headers
    // =========================================================================

    #[test]
    fn cat23_empty_scriptlet_no_actions() {
        let scriptlet = Scriptlet::new("T".into(), "bash".into(), "echo".into());
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert!(actions.is_empty());
    }

