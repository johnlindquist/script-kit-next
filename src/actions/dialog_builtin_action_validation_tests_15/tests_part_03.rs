    #[test]
    fn cat12_path_file_primary_first() {
        let pi = PathInfo {
            name: "file.txt".into(),
            path: "/tmp/file.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&pi);
        assert_eq!(actions[0].id, "select_file");
    }

    #[test]
    fn cat12_path_trash_always_last() {
        let pi = PathInfo {
            name: "x".into(),
            path: "/tmp/x".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&pi);
        assert_eq!(actions.last().unwrap().id, "move_to_trash");
    }

    #[test]
    fn cat12_path_dir_and_file_same_count() {
        let dir = PathInfo {
            name: "d".into(),
            path: "/tmp/d".into(),
            is_dir: true,
        };
        let file = PathInfo {
            name: "f".into(),
            path: "/tmp/f".into(),
            is_dir: false,
        };
        assert_eq!(
            get_path_context_actions(&dir).len(),
            get_path_context_actions(&file).len()
        );
    }

    // =========================================================================
    // cat13: File title quoting
    // =========================================================================

    #[test]
    fn cat13_file_title_contains_quoted_name() {
        let fi = FileInfo {
            name: "report.pdf".into(),
            path: "/docs/report.pdf".into(),
            is_dir: false,
            file_type: crate::file_search::FileType::Document,
        };
        let actions = get_file_context_actions(&fi);
        assert!(
            actions[0].title.contains("\"report.pdf\""),
            "Title should contain quoted filename: {}",
            actions[0].title
        );
    }

    #[test]
    fn cat13_dir_title_contains_quoted_name() {
        let fi = FileInfo {
            name: "build".into(),
            path: "/project/build".into(),
            is_dir: true,
            file_type: crate::file_search::FileType::Directory,
        };
        let actions = get_file_context_actions(&fi);
        assert!(
            actions[0].title.contains("\"build\""),
            "Title should contain quoted dirname: {}",
            actions[0].title
        );
    }

    #[test]
    fn cat13_file_primary_is_open_file() {
        let fi = FileInfo {
            name: "x".into(),
            path: "/x".into(),
            is_dir: false,
            file_type: crate::file_search::FileType::Document,
        };
        let actions = get_file_context_actions(&fi);
        assert_eq!(actions[0].id, "open_file");
    }

    #[test]
    fn cat13_dir_primary_is_open_directory() {
        let fi = FileInfo {
            name: "y".into(),
            path: "/y".into(),
            is_dir: true,
            file_type: crate::file_search::FileType::Directory,
        };
        let actions = get_file_context_actions(&fi);
        assert_eq!(actions[0].id, "open_directory");
    }

    // =========================================================================
    // cat14: ScriptInfo::with_all field completeness
    // =========================================================================

    #[test]
    fn cat14_with_all_name_path() {
        let s = ScriptInfo::with_all("MyScript", "/path/my.ts", true, "Execute", None, None);
        assert_eq!(s.name, "MyScript");
        assert_eq!(s.path, "/path/my.ts");
    }

    #[test]
    fn cat14_with_all_is_script() {
        let s = ScriptInfo::with_all("S", "/p", true, "Run", None, None);
        assert!(s.is_script);
        let s2 = ScriptInfo::with_all("S", "/p", false, "Run", None, None);
        assert!(!s2.is_script);
    }

    #[test]
    fn cat14_with_all_verb() {
        let s = ScriptInfo::with_all("S", "/p", true, "Launch", None, None);
        assert_eq!(s.action_verb, "Launch");
    }

    #[test]
    fn cat14_with_all_shortcut_and_alias() {
        let s = ScriptInfo::with_all(
            "S",
            "/p",
            true,
            "Run",
            Some("cmd+k".into()),
            Some("sk".into()),
        );
        assert_eq!(s.shortcut, Some("cmd+k".to_string()));
        assert_eq!(s.alias, Some("sk".to_string()));
    }

    #[test]
    fn cat14_with_all_no_agent_no_scriptlet() {
        let s = ScriptInfo::with_all("S", "/p", true, "Run", None, None);
        assert!(!s.is_agent);
        assert!(!s.is_scriptlet);
        assert!(!s.is_suggested);
    }

    // =========================================================================
    // cat15: Script context run title includes verb + name
    // =========================================================================

    #[test]
    fn cat15_run_title_default_verb() {
        let s = ScriptInfo::new("My Script", "/p");
        let actions = get_script_context_actions(&s);
        assert!(
            actions[0].title.contains("Run"),
            "Default verb is Run: {}",
            actions[0].title
        );
        assert!(
            actions[0].title.contains("My Script"),
            "Title includes name: {}",
            actions[0].title
        );
    }

    #[test]
    fn cat15_run_title_custom_verb() {
        let s = ScriptInfo::with_action_verb("Windows", "/p", true, "Switch to");
        let actions = get_script_context_actions(&s);
        assert!(
            actions[0].title.contains("Switch to"),
            "Custom verb: {}",
            actions[0].title
        );
    }

    #[test]
    fn cat15_run_title_builtin() {
        let s = ScriptInfo::builtin("Clipboard History");
        let actions = get_script_context_actions(&s);
        assert!(
            actions[0].title.contains("Clipboard History"),
            "Builtin title: {}",
            actions[0].title
        );
    }

    // =========================================================================
    // cat16: Ordering idempotency (double-call determinism)
    // =========================================================================

    #[test]
    fn cat16_script_actions_idempotent() {
        let s = ScriptInfo::new("test", "/p");
        let a1 = action_ids(&get_script_context_actions(&s));
        let a2 = action_ids(&get_script_context_actions(&s));
        assert_eq!(a1, a2);
    }

    #[test]
    fn cat16_clipboard_actions_idempotent() {
        let e = ClipboardEntryInfo {
            id: "c".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hi".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let a1 = action_ids(&get_clipboard_history_context_actions(&e));
        let a2 = action_ids(&get_clipboard_history_context_actions(&e));
        assert_eq!(a1, a2);
    }

    #[test]
    fn cat16_ai_actions_idempotent() {
        let a1 = action_ids(&get_ai_command_bar_actions());
        let a2 = action_ids(&get_ai_command_bar_actions());
        assert_eq!(a1, a2);
    }

    #[test]
    fn cat16_notes_actions_idempotent() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let a1 = action_ids(&get_notes_command_bar_actions(&info));
        let a2 = action_ids(&get_notes_command_bar_actions(&info));
        assert_eq!(a1, a2);
    }

    #[test]
    fn cat16_path_actions_idempotent() {
        let pi = PathInfo {
            name: "x".into(),
            path: "/x".into(),
            is_dir: true,
        };
        let a1 = action_ids(&get_path_context_actions(&pi));
        let a2 = action_ids(&get_path_context_actions(&pi));
        assert_eq!(a1, a2);
    }

    // =========================================================================
    // cat17: Note switcher icon hierarchy
    // =========================================================================

    #[test]
    fn cat17_pinned_overrides_current() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n1".into(),
            title: "Both".into(),
            char_count: 10,
            is_current: true,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    }

    #[test]
    fn cat17_current_only_check() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n2".into(),
            title: "Current".into(),
            char_count: 10,
            is_current: true,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::Check));
    }

    #[test]
    fn cat17_regular_file_icon() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n3".into(),
            title: "Regular".into(),
            char_count: 10,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::File));
    }

    #[test]
    fn cat17_pinned_not_current_star() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "n4".into(),
            title: "Pinned".into(),
            char_count: 10,
            is_current: false,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    }

    // =========================================================================
    // cat18: Note switcher section assignment
    // =========================================================================

    #[test]
    fn cat18_pinned_in_pinned_section() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "p1".into(),
            title: "P".into(),
            char_count: 1,
            is_current: false,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
    }

    #[test]
    fn cat18_unpinned_in_recent_section() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "r1".into(),
            title: "R".into(),
            char_count: 1,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Recent"));
    }

    #[test]
    fn cat18_mixed_sections() {
        let notes = vec![
            NoteSwitcherNoteInfo {
                id: "p".into(),
                title: "Pinned".into(),
                char_count: 1,
                is_current: false,
                is_pinned: true,
                preview: String::new(),
                relative_time: String::new(),
            },
            NoteSwitcherNoteInfo {
                id: "r".into(),
                title: "Recent".into(),
                char_count: 1,
                is_current: false,
                is_pinned: false,
                preview: String::new(),
                relative_time: String::new(),
            },
        ];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].section.as_deref(), Some("Pinned"));
        assert_eq!(actions[1].section.as_deref(), Some("Recent"));
    }

    #[test]
    fn cat18_empty_shows_notes_section() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].section.as_deref(), Some("Notes"));
    }

    // =========================================================================
    // cat19: Note switcher current bullet prefix
    // =========================================================================

    #[test]
    fn cat19_current_has_bullet() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "c1".into(),
            title: "My Note".into(),
            char_count: 10,
            is_current: true,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert!(
            actions[0].title.starts_with("• "),
            "Current note should have bullet prefix: {}",
            actions[0].title
        );
    }

    #[test]
    fn cat19_non_current_no_bullet() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "c2".into(),
            title: "Other Note".into(),
            char_count: 10,
            is_current: false,
            is_pinned: false,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert!(
            !actions[0].title.starts_with("• "),
            "Non-current should not have bullet: {}",
            actions[0].title
        );
    }

    #[test]
    fn cat19_current_pinned_has_bullet() {
        // Even pinned+current gets bullet prefix
        let notes = vec![NoteSwitcherNoteInfo {
            id: "c3".into(),
            title: "Pinned Current".into(),
            char_count: 10,
            is_current: true,
            is_pinned: true,
            preview: String::new(),
            relative_time: String::new(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert!(actions[0].title.starts_with("• "));
    }

    // =========================================================================
    // cat20: Clipboard paste title dynamic behavior
    // =========================================================================

