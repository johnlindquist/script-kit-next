    #[test]
    fn new_chat_preset_description_is_none() {
        let presets = vec![NewChatPresetInfo {
            id: "general".into(),
            name: "General".into(),
            icon: IconName::Star,
        }];
        let actions = get_new_chat_actions(&[], &presets, &[]);
        assert!(actions[0].description.is_none());
    }

    // =========================================================================
    // 27. Note switcher: empty notes produces no_notes action
    // =========================================================================

    #[test]
    fn note_switcher_empty_produces_no_notes() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].id, "no_notes");
    }

    #[test]
    fn note_switcher_empty_title() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions[0].title, "No notes yet");
    }

    #[test]
    fn note_switcher_empty_icon_plus() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions[0].icon, Some(IconName::Plus));
    }

    #[test]
    fn note_switcher_empty_desc_mentions_cmd_n() {
        let actions = get_note_switcher_actions(&[]);
        assert!(actions[0].description.as_ref().unwrap().contains("⌘N"));
    }

    // =========================================================================
    // 28. Note switcher: preview with relative_time has separator
    // =========================================================================

    #[test]
    fn note_switcher_preview_and_time_has_dot_separator() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc-123".into(),
            title: "Test Note".into(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: "Hello world".into(),
            relative_time: "2m ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert!(actions[0].description.as_ref().unwrap().contains(" · "));
    }

    #[test]
    fn note_switcher_preview_only_no_separator() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc-123".into(),
            title: "Test Note".into(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: "Hello world".into(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert!(!actions[0].description.as_ref().unwrap().contains(" · "));
    }

    #[test]
    fn note_switcher_time_only_when_empty_preview() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc-123".into(),
            title: "Test Note".into(),
            char_count: 100,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: "5d ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].description.as_ref().unwrap(), "5d ago");
    }

    #[test]
    fn note_switcher_no_preview_no_time_shows_char_count() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "abc-123".into(),
            title: "Test Note".into(),
            char_count: 42,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].description.as_ref().unwrap(), "42 chars");
    }

    // =========================================================================
    // 29. ProtocolAction: with_value sets value and has_action false
    // =========================================================================

    #[test]
    fn protocol_action_with_value_sets_name() {
        let pa = ProtocolAction::with_value("Copy".into(), "copy-cmd".into());
        assert_eq!(pa.name, "Copy");
    }

    #[test]
    fn protocol_action_with_value_sets_value() {
        let pa = ProtocolAction::with_value("Copy".into(), "copy-cmd".into());
        assert_eq!(pa.value, Some("copy-cmd".to_string()));
    }

    #[test]
    fn protocol_action_with_value_has_action_false() {
        let pa = ProtocolAction::with_value("Copy".into(), "copy-cmd".into());
        assert!(!pa.has_action);
    }

    #[test]
    fn protocol_action_with_value_defaults_visible_close_none() {
        let pa = ProtocolAction::with_value("X".into(), "y".into());
        assert!(pa.visible.is_none());
        assert!(pa.close.is_none());
        // But defaults still work:
        assert!(pa.is_visible());
        assert!(pa.should_close());
    }

    // =========================================================================
    // 30. format_shortcut_hint: dialog vs builders produce different results
    // =========================================================================

    #[test]
    fn dialog_format_shortcut_hint_handles_meta() {
        let result = ActionsDialog::format_shortcut_hint("meta+c");
        assert_eq!(result, "⌘C");
    }

    #[test]
    fn dialog_format_shortcut_hint_handles_super() {
        let result = ActionsDialog::format_shortcut_hint("super+x");
        assert_eq!(result, "⌘X");
    }

    #[test]
    fn dialog_format_shortcut_hint_handles_option() {
        let result = ActionsDialog::format_shortcut_hint("option+v");
        assert_eq!(result, "⌥V");
    }

    #[test]
    fn dialog_format_shortcut_hint_handles_compound() {
        let result = ActionsDialog::format_shortcut_hint("ctrl+shift+alt+k");
        assert_eq!(result, "⌃⇧⌥K");
    }
