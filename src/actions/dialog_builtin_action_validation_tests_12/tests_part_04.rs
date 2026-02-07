    #[test]
    fn cat20_no_selection_minimal() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        // Minimal: new_note, browse_notes, possibly enable_auto_sizing
        assert!(actions.len() <= 3);
    }

    #[test]
    fn cat20_auto_sizing_disabled_adds_settings_action() {
        let disabled = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let enabled = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let d_actions = get_notes_command_bar_actions(&disabled);
        let e_actions = get_notes_command_bar_actions(&enabled);
        assert!(d_actions.iter().any(|a| a.id == "enable_auto_sizing"));
        assert!(!e_actions.iter().any(|a| a.id == "enable_auto_sizing"));
    }

    #[test]
    fn cat20_notes_section_contains_new_note_and_browse() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let notes_section: Vec<_> = actions
            .iter()
            .filter(|a| a.section.as_deref() == Some("Notes"))
            .collect();
        let ids: Vec<_> = notes_section.iter().map(|a| a.id.as_str()).collect();
        assert!(ids.contains(&"new_note"));
        assert!(ids.contains(&"browse_notes"));
    }

    // =========================================================================
    // 21. coerce_action_selection edge cases
    // =========================================================================

    #[test]
    fn cat21_empty_rows_returns_none() {
        assert_eq!(coerce_action_selection(&[], 0), None);
    }

    #[test]
    fn cat21_on_item_returns_same() {
        let rows = vec![GroupedActionItem::Item(0)];
        assert_eq!(coerce_action_selection(&rows, 0), Some(0));
    }

    #[test]
    fn cat21_header_searches_down() {
        let rows = vec![
            GroupedActionItem::SectionHeader("H".into()),
            GroupedActionItem::Item(0),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), Some(1));
    }

    #[test]
    fn cat21_trailing_header_searches_up() {
        let rows = vec![
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("H".into()),
        ];
        assert_eq!(coerce_action_selection(&rows, 1), Some(0));
    }

    #[test]
    fn cat21_all_headers_returns_none() {
        let rows = vec![
            GroupedActionItem::SectionHeader("A".into()),
            GroupedActionItem::SectionHeader("B".into()),
        ];
        assert_eq!(coerce_action_selection(&rows, 0), None);
    }

    #[test]
    fn cat21_out_of_bounds_clamped() {
        let rows = vec![GroupedActionItem::Item(0)];
        assert_eq!(coerce_action_selection(&rows, 100), Some(0));
    }

    // =========================================================================
    // 22. fuzzy_match edge cases
    // =========================================================================

    #[test]
    fn cat22_empty_needle_matches() {
        assert!(ActionsDialog::fuzzy_match("anything", ""));
    }

    #[test]
    fn cat22_empty_haystack_no_match() {
        assert!(!ActionsDialog::fuzzy_match("", "x"));
    }

    #[test]
    fn cat22_both_empty_matches() {
        assert!(ActionsDialog::fuzzy_match("", ""));
    }

    #[test]
    fn cat22_exact_match() {
        assert!(ActionsDialog::fuzzy_match("hello", "hello"));
    }

    #[test]
    fn cat22_subsequence_match() {
        assert!(ActionsDialog::fuzzy_match("hello world", "hlw"));
    }

    #[test]
    fn cat22_no_subsequence() {
        assert!(!ActionsDialog::fuzzy_match("hello", "xyz"));
    }

    #[test]
    fn cat22_needle_longer_than_haystack() {
        assert!(!ActionsDialog::fuzzy_match("hi", "hello"));
    }

    // =========================================================================
    // 23. Script context exact action ordering
    // =========================================================================

    #[test]
    fn cat23_script_run_always_first() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        assert_eq!(actions[0].id, "run_script");
    }

    #[test]
    fn cat23_script_copy_deeplink_always_present() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        assert!(actions.iter().any(|a| a.id == "copy_deeplink"));
    }

    #[test]
    fn cat23_builtin_run_first() {
        let builtin = ScriptInfo::builtin("Test Builtin");
        let actions = get_script_context_actions(&builtin);
        assert_eq!(actions[0].id, "run_script");
    }

    #[test]
    fn cat23_scriptlet_run_first() {
        let scriptlet = ScriptInfo::scriptlet("Test", "/path.md", None, None);
        let actions = get_script_context_actions(&scriptlet);
        assert_eq!(actions[0].id, "run_script");
    }

    #[test]
    fn cat23_run_action_title_includes_verb_and_name() {
        let script = ScriptInfo::with_action_verb("Safari", "/app", false, "Launch");
        let actions = get_script_context_actions(&script);
        assert_eq!(actions[0].title, "Launch \"Safari\"");
    }

    // =========================================================================
    // 24. Clipboard paste title dynamic behavior
    // =========================================================================

    #[test]
    fn cat24_paste_no_app() {
        let entry = make_text_entry();
        let actions = get_clipboard_history_context_actions(&entry);
        let paste = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
        assert_eq!(paste.title, "Paste to Active App");
    }

    #[test]
    fn cat24_paste_with_app() {
        let mut entry = make_text_entry();
        entry.frontmost_app_name = Some("Safari".into());
        let actions = get_clipboard_history_context_actions(&entry);
        let paste = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
        assert_eq!(paste.title, "Paste to Safari");
    }

    #[test]
    fn cat24_paste_with_unicode_app() {
        let mut entry = make_text_entry();
        entry.frontmost_app_name = Some("日本語App".into());
        let actions = get_clipboard_history_context_actions(&entry);
        let paste = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
        assert_eq!(paste.title, "Paste to 日本語App");
    }

    #[test]
    fn cat24_paste_with_empty_app_string() {
        let mut entry = make_text_entry();
        entry.frontmost_app_name = Some("".into());
        let actions = get_clipboard_history_context_actions(&entry);
        let paste = actions.iter().find(|a| a.id == "clipboard_paste").unwrap();
        // Empty string still produces "Paste to " (the code uses the string as-is)
        assert_eq!(paste.title, "Paste to ");
    }

    // =========================================================================
    // 25. Action lowercase caching
    // =========================================================================

    #[test]
    fn cat25_title_lower_computed_on_creation() {
        let action = Action::new("x", "Hello World", None, ActionCategory::ScriptContext);
        assert_eq!(action.title_lower, "hello world");
    }

    #[test]
    fn cat25_description_lower_computed() {
        let action = Action::new(
            "x",
            "X",
            Some("Foo Bar".into()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(action.description_lower, Some("foo bar".into()));
    }

    #[test]
    fn cat25_description_lower_none_when_no_desc() {
        let action = Action::new("x", "X", None, ActionCategory::ScriptContext);
        assert!(action.description_lower.is_none());
    }

    #[test]
    fn cat25_shortcut_lower_none_until_with_shortcut() {
        let action = Action::new("x", "X", None, ActionCategory::ScriptContext);
        assert!(action.shortcut_lower.is_none());
        let action = action.with_shortcut("⌘C");
        assert_eq!(action.shortcut_lower, Some("⌘c".into()));
    }

    #[test]
    fn cat25_with_shortcut_opt_none_no_shortcut_lower() {
        let action =
            Action::new("x", "X", None, ActionCategory::ScriptContext).with_shortcut_opt(None);
        assert!(action.shortcut_lower.is_none());
        assert!(action.shortcut.is_none());
    }

    // =========================================================================
    // 26. Note switcher description rendering
    // =========================================================================

    #[test]
    fn cat26_preview_with_time_uses_separator() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "1".into(),
            title: "Note".into(),
            char_count: 50,
            is_current: false,
            is_pinned: false,
            preview: "Some preview".into(),
            relative_time: "5m ago".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(
            desc.contains(" · "),
            "Expected ' · ' separator in: {}",
            desc
        );
    }

    #[test]
    fn cat26_empty_preview_empty_time_uses_char_count() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "1".into(),
            title: "Note".into(),
            char_count: 42,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].description.as_deref(), Some("42 chars"));
    }

    #[test]
    fn cat26_singular_char_count() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "1".into(),
            title: "Note".into(),
            char_count: 1,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].description.as_deref(), Some("1 char"));
    }

    #[test]
    fn cat26_zero_char_count() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "1".into(),
            title: "Note".into(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].description.as_deref(), Some("0 chars"));
    }

    #[test]
    fn cat26_preview_truncated_at_61_chars() {
        let long_preview = "a".repeat(61);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "1".into(),
            title: "Note".into(),
            char_count: 61,
            is_current: false,
            is_pinned: false,
            preview: long_preview,
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(desc.ends_with('…'), "Should end with … : {}", desc);
    }

    #[test]
    fn cat26_preview_not_truncated_at_60_chars() {
        let preview = "a".repeat(60);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "1".into(),
            title: "Note".into(),
            char_count: 60,
            is_current: false,
            is_pinned: false,
            preview,
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        let desc = actions[0].description.as_ref().unwrap();
        assert!(!desc.ends_with('…'), "Should not truncate at 60: {}", desc);
    }

    // =========================================================================
    // 27. Note switcher icon hierarchy
    // =========================================================================

    #[test]
    fn cat27_pinned_icon_star_filled() {
        let notes = vec![make_note("1", "N", true, false)];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    }

    #[test]
    fn cat27_current_icon_check() {
        let notes = vec![make_note("1", "N", false, true)];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::Check));
    }

    #[test]
    fn cat27_regular_icon_file() {
        let notes = vec![make_note("1", "N", false, false)];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::File));
    }

    #[test]
    fn cat27_pinned_overrides_current() {
        let notes = vec![make_note("1", "N", true, true)];
        let actions = get_note_switcher_actions(&notes);
        // Pinned takes priority in the if-else chain
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    }

    // =========================================================================
    // 28. Cross-context ID uniqueness
    // =========================================================================

    #[test]
    fn cat28_script_ids_unique() {
        let actions = get_script_context_actions(&ScriptInfo::new("t", "/t.ts"));
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn cat28_clipboard_ids_unique() {
        let actions = get_clipboard_history_context_actions(&make_text_entry());
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn cat28_ai_ids_unique() {
        let actions = get_ai_command_bar_actions();
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn cat28_path_ids_unique() {
        let info = PathInfo {
            path: "/t".into(),
            name: "t".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn cat28_file_ids_unique() {
        let info = FileInfo {
            path: "/t".into(),
            name: "t".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&info);
        let ids: HashSet<_> = actions.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), actions.len());
    }

