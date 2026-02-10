    #[test]
    fn cat06_note_switcher_preview_no_time() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n2".into(),
            title: "Test".into(),
            char_count: 50,
            is_current: false,
            is_pinned: false,
            preview: "Some content".into(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_deref().unwrap();
        assert_eq!(desc, "Some content");
        assert!(!desc.contains("·"));
    }

    #[test]
    fn cat06_note_switcher_no_preview_with_time() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n3".into(),
            title: "Test".into(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: "1h ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_deref().unwrap();
        assert_eq!(desc, "1h ago");
    }

    #[test]
    fn cat06_note_switcher_no_preview_no_time() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n4".into(),
            title: "Test".into(),
            char_count: 42,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_deref().unwrap();
        assert_eq!(desc, "42 chars");
    }

    #[test]
    fn cat06_note_switcher_singular_char() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n5".into(),
            title: "T".into(),
            char_count: 1,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_deref().unwrap();
        assert_eq!(desc, "1 char");
    }

    #[test]
    fn cat06_note_switcher_zero_chars() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n6".into(),
            title: "Empty".into(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_deref().unwrap();
        assert_eq!(desc, "0 chars");
    }

    #[test]
    fn cat06_note_switcher_preview_exactly_60_no_ellipsis() {
        let preview: String = "a".repeat(60);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n7".into(),
            title: "T".into(),
            char_count: 60,
            is_current: false,
            is_pinned: false,
            preview,
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_deref().unwrap();
        assert!(!desc.contains('…'), "60 chars should not be truncated");
    }

    #[test]
    fn cat06_note_switcher_preview_61_has_ellipsis() {
        let preview: String = "b".repeat(61);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n8".into(),
            title: "T".into(),
            char_count: 61,
            is_current: false,
            is_pinned: false,
            preview,
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_deref().unwrap();
        assert!(desc.contains('…'), "61 chars should be truncated with …");
    }

    // =========================================================================
    // cat07: AI command bar per-section ID enumeration
    // =========================================================================

    #[test]
    fn cat07_ai_response_section_ids() {
        let actions = get_ai_command_bar_actions();
        let response_ids: Vec<&str> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Response"))
            .map(|a| a.id.as_str())
            .collect();
        assert_eq!(
            response_ids,
            vec!["copy_response", "copy_chat", "copy_last_code"]
        );
    }

    #[test]
    fn cat07_ai_actions_section_ids() {
        let actions = get_ai_command_bar_actions();
        let action_ids: Vec<&str> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Actions"))
            .map(|a| a.id.as_str())
            .collect();
        assert_eq!(
            action_ids,
            vec!["submit", "new_chat", "delete_chat", "branch_from_last"]
        );
    }

    #[test]
    fn cat07_ai_attachments_section_ids() {
        let actions = get_ai_command_bar_actions();
        let att_ids: Vec<&str> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Attachments"))
            .map(|a| a.id.as_str())
            .collect();
        assert_eq!(att_ids, vec!["add_attachment", "paste_image"]);
    }

    #[test]
    fn cat07_ai_export_section_ids() {
        let actions = get_ai_command_bar_actions();
        let export_ids: Vec<&str> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Export"))
            .map(|a| a.id.as_str())
            .collect();
        assert_eq!(export_ids, vec!["export_markdown"]);
    }

    #[test]
    fn cat07_ai_help_section_ids() {
        let actions = get_ai_command_bar_actions();
        let help_ids: Vec<&str> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Help"))
            .map(|a| a.id.as_str())
            .collect();
        assert_eq!(help_ids, vec!["toggle_shortcuts_help"]);
    }

    #[test]
    fn cat07_ai_settings_section_ids() {
        let actions = get_ai_command_bar_actions();
        let settings_ids: Vec<&str> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Settings"))
            .map(|a| a.id.as_str())
            .collect();
        assert_eq!(settings_ids, vec!["change_model"]);
    }

    // =========================================================================
    // cat08: Action builder overwrite semantics
    // =========================================================================

    #[test]
    fn cat08_with_shortcut_overwrites_previous() {
        let action = Action::new("x", "X", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘A")
            .with_shortcut("⌘B");
        assert_eq!(action.shortcut.as_deref(), Some("⌘B"));
    }

    #[test]
    fn cat08_with_icon_overwrites_previous() {
        let action = Action::new("x", "X", None, ActionCategory::ScriptContext)
            .with_icon(IconName::Star)
            .with_icon(IconName::Trash);
        assert_eq!(action.icon, Some(IconName::Trash));
    }

    #[test]
    fn cat08_with_section_overwrites_previous() {
        let action = Action::new("x", "X", None, ActionCategory::ScriptContext)
            .with_section("A")
            .with_section("B");
        assert_eq!(action.section.as_deref(), Some("B"));
    }

    #[test]
    fn cat08_with_shortcut_opt_none_preserves() {
        // with_shortcut_opt(None) does NOT clear existing shortcut
        let action = Action::new("x", "X", None, ActionCategory::ScriptContext)
            .with_shortcut("⌘A")
            .with_shortcut_opt(None);
        assert_eq!(
            action.shortcut.as_deref(),
            Some("⌘A"),
            "None does not clear existing shortcut"
        );
    }

    #[test]
    fn cat08_with_shortcut_opt_some_sets() {
        let action = Action::new("x", "X", None, ActionCategory::ScriptContext)
            .with_shortcut_opt(Some("⌘Z".to_string()));
        assert_eq!(action.shortcut.as_deref(), Some("⌘Z"));
    }

    // =========================================================================
    // cat09: CommandBarConfig preset field comparison matrix
    // =========================================================================

    #[test]
    fn cat09_default_vs_ai_style() {
        let def = CommandBarConfig::default();
        let ai = CommandBarConfig::ai_style();
        // AI style uses Headers, default uses Separators
        assert_eq!(ai.dialog_config.section_style, SectionStyle::Headers);
        assert_eq!(def.dialog_config.section_style, SectionStyle::Separators);
    }

    #[test]
    fn cat09_notes_style_search_top() {
        let notes = CommandBarConfig::notes_style();
        assert_eq!(notes.dialog_config.search_position, SearchPosition::Top);
    }

    #[test]
    fn cat09_no_search_hidden() {
        let ns = CommandBarConfig::no_search();
        assert_eq!(ns.dialog_config.search_position, SearchPosition::Hidden);
    }

    #[test]
    fn cat09_main_menu_bottom() {
        let mm = CommandBarConfig::main_menu_style();
        assert_eq!(mm.dialog_config.search_position, SearchPosition::Bottom);
    }

    #[test]
    fn cat09_all_presets_close_on_select_true() {
        assert!(CommandBarConfig::default().close_on_select);
        assert!(CommandBarConfig::ai_style().close_on_select);
        assert!(CommandBarConfig::main_menu_style().close_on_select);
        assert!(CommandBarConfig::no_search().close_on_select);
        assert!(CommandBarConfig::notes_style().close_on_select);
    }

    #[test]
    fn cat09_all_presets_close_on_escape_true() {
        assert!(CommandBarConfig::default().close_on_escape);
        assert!(CommandBarConfig::ai_style().close_on_escape);
        assert!(CommandBarConfig::main_menu_style().close_on_escape);
        assert!(CommandBarConfig::no_search().close_on_escape);
        assert!(CommandBarConfig::notes_style().close_on_escape);
    }

    // =========================================================================
    // cat10: Cross-context category uniformity
    // =========================================================================

    #[test]
    fn cat10_script_actions_all_script_context() {
        let script = ScriptInfo::new("test", "/p");
        for action in &get_script_context_actions(&script) {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

    #[test]
    fn cat10_clipboard_actions_all_script_context() {
        let entry = ClipboardEntryInfo {
            id: "c1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        for action in &get_clipboard_history_context_actions(&entry) {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

    #[test]
    fn cat10_ai_actions_all_script_context() {
        for action in &get_ai_command_bar_actions() {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

    #[test]
    fn cat10_path_actions_all_script_context() {
        let pi = PathInfo {
            name: "dir".into(),
            path: "/tmp/dir".into(),
            is_dir: true,
        };
        for action in &get_path_context_actions(&pi) {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

    #[test]
    fn cat10_notes_actions_all_script_context() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        for action in &get_notes_command_bar_actions(&info) {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

    #[test]
    fn cat10_chat_actions_all_script_context() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: true,
            has_response: true,
        };
        for action in &get_chat_context_actions(&info) {
            assert_eq!(action.category, ActionCategory::ScriptContext);
        }
    }

    // =========================================================================
    // cat11: Clipboard exact action counts on macOS
    // =========================================================================

    #[cfg(target_os = "macos")]
    #[test]
    fn cat11_clipboard_text_count_macos() {
        let entry = ClipboardEntryInfo {
            id: "t1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hi".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        // paste, copy, paste_keep_open, share, attach, quick_look,
        // pin, save_snippet, save_file, delete, delete_multiple, delete_all = 12
        assert_eq!(actions.len(), 12, "Text on macOS: {}", actions.len());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat11_clipboard_image_count_macos() {
        let entry = ClipboardEntryInfo {
            id: "i1".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        // paste, copy, paste_keep_open, share, attach, quick_look,
        // open_with, annotate_cleanshot, upload_cleanshot,
        // pin, ocr, save_snippet, save_file, delete, delete_multiple, delete_all = 16
        assert_eq!(actions.len(), 16, "Image on macOS: {}", actions.len());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat11_clipboard_image_more_than_text_macos() {
        let img = ClipboardEntryInfo {
            id: "i".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: String::new(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let txt = ClipboardEntryInfo {
            id: "t".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let img_count = get_clipboard_history_context_actions(&img).len();
        let txt_count = get_clipboard_history_context_actions(&txt).len();
        assert!(
            img_count > txt_count,
            "Image ({}) should have more actions than text ({})",
            img_count,
            txt_count
        );
    }

    // =========================================================================
    // cat12: Path primary-action insertion position
    // =========================================================================

    #[test]
    fn cat12_path_dir_primary_first() {
        let pi = PathInfo {
            name: "mydir".into(),
            path: "/tmp/mydir".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&pi);
        assert_eq!(actions[0].id, "open_directory");
    }

