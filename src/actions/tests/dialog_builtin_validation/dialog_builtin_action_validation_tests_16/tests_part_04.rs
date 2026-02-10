    #[test]
    fn cat22_clipboard_shortcuts_use_unicode() {
        let entry = ClipboardEntryInfo {
            id: "c1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        for action in &actions {
            if let Some(ref s) = action.shortcut {
                assert!(
                    !s.contains("cmd") && !s.contains("shift"),
                    "Clipboard shortcut '{}' should use Unicode",
                    s
                );
            }
        }
    }

    #[test]
    fn cat22_ai_shortcuts_use_unicode() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            if let Some(ref s) = action.shortcut {
                assert!(
                    !s.contains("cmd") && !s.contains("shift"),
                    "AI shortcut '{}' should use Unicode",
                    s
                );
            }
        }
    }

    #[test]
    fn cat22_path_shortcuts_use_unicode() {
        let info = PathInfo {
            name: "f".into(),
            path: "/f".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        for action in &actions {
            if let Some(ref s) = action.shortcut {
                assert!(
                    !s.contains("cmd") && !s.contains("shift"),
                    "Path shortcut '{}' should use Unicode",
                    s
                );
            }
        }
    }

    // =========================================================================
    // cat23: Clipboard paste_keep_open shortcut
    // =========================================================================

    #[test]
    fn cat23_paste_keep_open_shortcut() {
        let entry = ClipboardEntryInfo {
            id: "pk1".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let pko = actions
            .iter()
            .find(|a| a.id == "clipboard_paste_keep_open")
            .unwrap();
        assert_eq!(pko.shortcut.as_deref(), Some("⌥↵"));
    }

    #[test]
    fn cat23_paste_keep_open_title() {
        let entry = ClipboardEntryInfo {
            id: "pk2".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let pko = actions
            .iter()
            .find(|a| a.id == "clipboard_paste_keep_open")
            .unwrap();
        assert_eq!(pko.title, "Paste and Keep Window Open");
    }

    #[test]
    fn cat23_paste_keep_open_description() {
        let entry = ClipboardEntryInfo {
            id: "pk3".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let pko = actions
            .iter()
            .find(|a| a.id == "clipboard_paste_keep_open")
            .unwrap();
        assert!(pko.description.is_some());
    }

    // =========================================================================
    // cat24: Path context copy_filename has no shortcut
    // =========================================================================

    #[test]
    fn cat24_path_copy_filename_no_shortcut() {
        let info = PathInfo {
            name: "test.txt".into(),
            path: "/test.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let cf = actions.iter().find(|a| a.id == "copy_filename").unwrap();
        assert!(
            cf.shortcut.is_none(),
            "path copy_filename should have no shortcut"
        );
    }

    #[test]
    fn cat24_path_copy_filename_present() {
        let info = PathInfo {
            name: "readme.md".into(),
            path: "/readme.md".into(),
            is_dir: false,
        };
        let ids = action_ids(&get_path_context_actions(&info));
        assert!(ids.contains(&"copy_filename".to_string()));
    }

    #[test]
    fn cat24_path_copy_filename_description() {
        let info = PathInfo {
            name: "data.json".into(),
            path: "/data.json".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let cf = actions.iter().find(|a| a.id == "copy_filename").unwrap();
        assert!(cf
            .description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("filename"));
    }

    // =========================================================================
    // cat25: File context open_with macOS shortcut
    // =========================================================================

    #[cfg(target_os = "macos")]
    #[test]
    fn cat25_file_open_with_shortcut() {
        let file = FileInfo {
            path: "/test.txt".into(),
            name: "test.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        let ow = actions.iter().find(|a| a.id == "open_with").unwrap();
        assert_eq!(ow.shortcut.as_deref(), Some("⌘O"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat25_file_show_info_shortcut() {
        let file = FileInfo {
            path: "/img.png".into(),
            name: "img.png".into(),
            file_type: crate::file_search::FileType::Image,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        let si = actions.iter().find(|a| a.id == "show_info").unwrap();
        assert_eq!(si.shortcut.as_deref(), Some("⌘I"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat25_file_quick_look_shortcut() {
        let file = FileInfo {
            path: "/readme.md".into(),
            name: "readme.md".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        let ql = actions.iter().find(|a| a.id == "quick_look").unwrap();
        assert_eq!(ql.shortcut.as_deref(), Some("⌘Y"));
    }

    // =========================================================================
    // cat26: Notes format shortcut exact value
    // =========================================================================

    #[test]
    fn cat26_notes_format_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let fmt = actions.iter().find(|a| a.id == "format").unwrap();
        assert_eq!(fmt.shortcut.as_deref(), Some("⇧⌘T"));
    }

    #[test]
    fn cat26_notes_format_icon() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let fmt = actions.iter().find(|a| a.id == "format").unwrap();
        assert_eq!(fmt.icon, Some(IconName::Code));
    }

    #[test]
    fn cat26_notes_format_section() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let fmt = actions.iter().find(|a| a.id == "format").unwrap();
        assert_eq!(fmt.section.as_deref(), Some("Edit"));
    }

    #[test]
    fn cat26_notes_find_in_note_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let find = actions.iter().find(|a| a.id == "find_in_note").unwrap();
        assert_eq!(find.shortcut.as_deref(), Some("⌘F"));
    }

    // =========================================================================
    // cat27: AI command bar icon name correctness
    // =========================================================================

    #[test]
    fn cat27_ai_copy_response_icon() {
        let actions = get_ai_command_bar_actions();
        let cr = actions.iter().find(|a| a.id == "copy_response").unwrap();
        assert_eq!(cr.icon, Some(IconName::Copy));
    }

    #[test]
    fn cat27_ai_submit_icon() {
        let actions = get_ai_command_bar_actions();
        let submit = actions.iter().find(|a| a.id == "submit").unwrap();
        assert_eq!(submit.icon, Some(IconName::ArrowUp));
    }

    #[test]
    fn cat27_ai_new_chat_icon() {
        let actions = get_ai_command_bar_actions();
        let nc = actions.iter().find(|a| a.id == "new_chat").unwrap();
        assert_eq!(nc.icon, Some(IconName::Plus));
    }

    #[test]
    fn cat27_ai_delete_chat_icon() {
        let actions = get_ai_command_bar_actions();
        let dc = actions.iter().find(|a| a.id == "delete_chat").unwrap();
        assert_eq!(dc.icon, Some(IconName::Trash));
    }

    #[test]
    fn cat27_ai_change_model_icon() {
        let actions = get_ai_command_bar_actions();
        let cm = actions.iter().find(|a| a.id == "change_model").unwrap();
        assert_eq!(cm.icon, Some(IconName::Settings));
    }

    #[test]
    fn cat27_ai_toggle_shortcuts_help_icon() {
        let actions = get_ai_command_bar_actions();
        let ts = actions
            .iter()
            .find(|a| a.id == "toggle_shortcuts_help")
            .unwrap();
        assert_eq!(ts.icon, Some(IconName::Star));
    }

    // =========================================================================
    // cat28: Script context run title format
    // =========================================================================

    #[test]
    fn cat28_run_title_default_verb() {
        let script = ScriptInfo::new("My Script", "/p/my-script.ts");
        let actions = get_script_context_actions(&script);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert_eq!(run.title, "Run \"My Script\"");
    }

    #[test]
    fn cat28_run_title_custom_verb() {
        let script = ScriptInfo::with_action_verb("Safari", "/app", false, "Launch");
        let actions = get_script_context_actions(&script);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert_eq!(run.title, "Launch \"Safari\"");
    }

    #[test]
    fn cat28_run_title_switch_to_verb() {
        let script = ScriptInfo::with_action_verb("Terminal", "window:1", false, "Switch to");
        let actions = get_script_context_actions(&script);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert_eq!(run.title, "Switch to \"Terminal\"");
    }

    #[test]
    fn cat28_run_title_builtin() {
        let builtin = ScriptInfo::builtin("Clipboard History");
        let actions = get_script_context_actions(&builtin);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert_eq!(run.title, "Run \"Clipboard History\"");
    }

    #[test]
    fn cat28_run_shortcut_always_enter() {
        let script = ScriptInfo::new("test", "/p/test.ts");
        let actions = get_script_context_actions(&script);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert_eq!(run.shortcut.as_deref(), Some("↵"));
    }

    // =========================================================================
    // cat29: Ordering consistency across repeated calls
    // =========================================================================

    #[test]
    fn cat29_script_ordering_deterministic() {
        let script = ScriptInfo::new("test", "/p/test.ts");
        let a1 = action_ids(&get_script_context_actions(&script));
        let a2 = action_ids(&get_script_context_actions(&script));
        assert_eq!(a1, a2);
    }

    #[test]
    fn cat29_clipboard_ordering_deterministic() {
        let entry = ClipboardEntryInfo {
            id: "c".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let a1 = action_ids(&get_clipboard_history_context_actions(&entry));
        let a2 = action_ids(&get_clipboard_history_context_actions(&entry));
        assert_eq!(a1, a2);
    }

    #[test]
    fn cat29_ai_ordering_deterministic() {
        let a1 = action_ids(&get_ai_command_bar_actions());
        let a2 = action_ids(&get_ai_command_bar_actions());
        assert_eq!(a1, a2);
    }

    #[test]
    fn cat29_notes_ordering_deterministic() {
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
    fn cat29_path_ordering_deterministic() {
        let info = PathInfo {
            name: "f".into(),
            path: "/f".into(),
            is_dir: false,
        };
        let a1 = action_ids(&get_path_context_actions(&info));
        let a2 = action_ids(&get_path_context_actions(&info));
        assert_eq!(a1, a2);
    }

    // =========================================================================
    // cat30: Cross-context non-empty ID and title, has_action=false, ID uniqueness
    // =========================================================================

    #[test]
    fn cat30_script_non_empty_ids_and_titles() {
        let script = ScriptInfo::new("test", "/p/test.ts");
        for action in &get_script_context_actions(&script) {
            assert!(!action.id.is_empty(), "action ID should not be empty");
            assert!(!action.title.is_empty(), "action title should not be empty");
        }
    }

    #[test]
    fn cat30_clipboard_non_empty_ids_and_titles() {
        let entry = ClipboardEntryInfo {
            id: "c".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "x".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        for action in &get_clipboard_history_context_actions(&entry) {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }

    #[test]
    fn cat30_ai_non_empty_ids_and_titles() {
        for action in &get_ai_command_bar_actions() {
            assert!(!action.id.is_empty());
            assert!(!action.title.is_empty());
        }
    }

