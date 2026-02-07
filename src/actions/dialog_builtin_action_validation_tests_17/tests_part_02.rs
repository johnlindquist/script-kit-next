    #[test]
    fn cat08_non_current_model_no_checkmark() {
        let info = ChatPromptInfo {
            current_model: Some("GPT-4".into()),
            available_models: vec![ChatModelInfo {
                id: "claude-3".into(),
                display_name: "Claude 3".into(),
                provider: "Anthropic".into(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        let model_action = actions
            .iter()
            .find(|a| a.id == "select_model_claude-3")
            .unwrap();
        assert!(!model_action.title.contains("✓"));
    }

    // ================================================================
    // Cat 09: Scriptlet defined action ID prefix invariant
    // ================================================================

    #[test]
    fn cat09_scriptlet_action_id_has_prefix() {
        let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo hi".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "Do Thing".into(),
            command: "do-thing".into(),
            tool: "bash".into(),
            code: "echo done".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert!(actions[0].id.starts_with("scriptlet_action:"));
    }

    #[test]
    fn cat09_scriptlet_action_id_contains_command() {
        let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo hi".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "Do Thing".into(),
            command: "do-thing".into(),
            tool: "bash".into(),
            code: "echo done".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert_eq!(actions[0].id, "scriptlet_action:do-thing");
    }

    #[test]
    fn cat09_all_scriptlet_actions_have_prefix() {
        let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo hi".into());
        scriptlet.actions = vec![
            ScriptletAction {
                name: "A".into(),
                command: "a".into(),
                tool: "bash".into(),
                code: "echo a".into(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
            ScriptletAction {
                name: "B".into(),
                command: "b".into(),
                tool: "bash".into(),
                code: "echo b".into(),
                inputs: vec![],
                shortcut: None,
                description: None,
            },
        ];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        for action in &actions {
            assert!(
                action.id.starts_with("scriptlet_action:"),
                "ID: {}",
                action.id
            );
        }
    }

    #[test]
    fn cat09_scriptlet_actions_all_have_action_true() {
        let mut scriptlet = Scriptlet::new("Test".into(), "bash".into(), "echo hi".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "A".into(),
            command: "a".into(),
            tool: "bash".into(),
            code: "echo a".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert!(actions[0].has_action);
    }

    // ================================================================
    // Cat 10: Agent context reveal_in_finder and copy_path shortcuts
    // ================================================================

    #[test]
    fn cat10_agent_has_reveal_in_finder() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
    }

    #[test]
    fn cat10_agent_reveal_shortcut() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
        assert_eq!(reveal.shortcut.as_deref(), Some("⌘⇧F"));
    }

    #[test]
    fn cat10_agent_copy_path_shortcut() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
        assert_eq!(cp.shortcut.as_deref(), Some("⌘⇧C"));
    }

    #[test]
    fn cat10_agent_edit_shortcut() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit.shortcut.as_deref(), Some("⌘E"));
    }

    #[test]
    fn cat10_agent_has_copy_content() {
        let mut script = ScriptInfo::new("My Agent", "/path/agent.md");
        script.is_script = false;
        script.is_agent = true;
        let actions = get_script_context_actions(&script);
        assert!(actions.iter().any(|a| a.id == "copy_content"));
    }

    // ================================================================
    // Cat 11: File context exact description strings
    // ================================================================

    #[test]
    fn cat11_file_open_description() {
        let fi = FileInfo {
            path: "/test/file.txt".into(),
            name: "file.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&fi);
        let open = actions.iter().find(|a| a.id == "open_file").unwrap();
        assert_eq!(
            open.description.as_deref(),
            Some("Open with default application")
        );
    }

    #[test]
    fn cat11_dir_open_description() {
        let fi = FileInfo {
            path: "/test/dir".into(),
            name: "dir".into(),
            file_type: crate::file_search::FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&fi);
        let open = actions.iter().find(|a| a.id == "open_directory").unwrap();
        assert_eq!(open.description.as_deref(), Some("Open this folder"));
    }

    #[test]
    fn cat11_reveal_description() {
        let fi = FileInfo {
            path: "/test/file.txt".into(),
            name: "file.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&fi);
        let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
        assert_eq!(reveal.description.as_deref(), Some("Reveal in Finder"));
    }

    #[test]
    fn cat11_copy_path_description() {
        let fi = FileInfo {
            path: "/test/file.txt".into(),
            name: "file.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&fi);
        let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
        assert!(cp.description.as_ref().unwrap().contains("path"));
    }

    #[test]
    fn cat11_copy_filename_description() {
        let fi = FileInfo {
            path: "/test/file.txt".into(),
            name: "file.txt".into(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&fi);
        let cf = actions.iter().find(|a| a.id == "copy_filename").unwrap();
        assert!(cf.description.as_ref().unwrap().contains("filename"));
    }

    // ================================================================
    // Cat 12: Path context exact description strings
    // ================================================================

    #[test]
    fn cat12_path_dir_primary_description() {
        let info = PathInfo {
            path: "/a/b".into(),
            name: "b".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        let primary = actions.iter().find(|a| a.id == "open_directory").unwrap();
        assert!(primary.description.as_ref().unwrap().contains("directory"));
    }

    #[test]
    fn cat12_path_file_primary_description() {
        let info = PathInfo {
            path: "/a/b.txt".into(),
            name: "b.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let primary = actions.iter().find(|a| a.id == "select_file").unwrap();
        assert!(primary.description.as_ref().unwrap().contains("file"));
    }

    #[test]
    fn cat12_path_open_in_editor_description() {
        let info = PathInfo {
            path: "/a/b".into(),
            name: "b".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        let editor = actions.iter().find(|a| a.id == "open_in_editor").unwrap();
        assert!(editor.description.as_ref().unwrap().contains("$EDITOR"));
    }

    #[test]
    fn cat12_path_move_to_trash_dir_description() {
        let info = PathInfo {
            path: "/a/b".into(),
            name: "b".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
        assert!(trash.description.as_ref().unwrap().contains("folder"));
    }

    #[test]
    fn cat12_path_move_to_trash_file_description() {
        let info = PathInfo {
            path: "/a/b.txt".into(),
            name: "b.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let trash = actions.iter().find(|a| a.id == "move_to_trash").unwrap();
        assert!(trash.description.as_ref().unwrap().contains("file"));
    }

    // ================================================================
    // Cat 13: Clipboard text/image macOS action count difference
    // ================================================================

    #[cfg(target_os = "macos")]
    #[test]
    fn cat13_image_has_more_actions_than_text_macos() {
        let text_entry = ClipboardEntryInfo {
            id: "t".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let img_entry = ClipboardEntryInfo {
            id: "i".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "img".into(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        };
        let text_actions = get_clipboard_history_context_actions(&text_entry);
        let img_actions = get_clipboard_history_context_actions(&img_entry);
        assert!(img_actions.len() > text_actions.len());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat13_image_has_ocr_text_does_not_macos() {
        let text_entry = ClipboardEntryInfo {
            id: "t".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let img_entry = ClipboardEntryInfo {
            id: "i".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "img".into(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        };
        let text_ids = action_ids(&get_clipboard_history_context_actions(&text_entry));
        let img_ids = action_ids(&get_clipboard_history_context_actions(&img_entry));
        assert!(!text_ids.contains(&"clipboard_ocr".to_string()));
        assert!(img_ids.contains(&"clipboard_ocr".to_string()));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat13_text_count_macos() {
        let entry = ClipboardEntryInfo {
            id: "t".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "text".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        // paste, copy, paste_keep_open, share, attach_to_ai, quick_look,
        // pin, save_snippet, save_file, delete, delete_multiple, delete_all = 12
        assert_eq!(actions.len(), 12);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cat13_image_count_macos() {
        let entry = ClipboardEntryInfo {
            id: "i".into(),
            content_type: ContentType::Image,
            pinned: false,
            preview: "img".into(),
            image_dimensions: Some((100, 100)),
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        // paste, copy, paste_keep_open, share, attach_to_ai, quick_look,
        // open_with, annotate_cleanshot, upload_cleanshot, pin, ocr,
        // save_snippet, save_file, delete, delete_multiple, delete_all = 16
        assert_eq!(actions.len(), 16);
    }

    // ================================================================
    // Cat 14: Script run title format includes quotes
    // ================================================================

    #[test]
    fn cat14_run_title_has_quotes_around_name() {
        let script = ScriptInfo::new("My Script", "/path/my-script.ts");
        let actions = get_script_context_actions(&script);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert_eq!(run.title, "Run \"My Script\"");
    }

    #[test]
    fn cat14_custom_verb_in_title() {
        let script =
            ScriptInfo::with_action_verb("Safari", "/Applications/Safari.app", false, "Launch");
        let actions = get_script_context_actions(&script);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert_eq!(run.title, "Launch \"Safari\"");
    }

    #[test]
    fn cat14_switch_to_verb_in_title() {
        let script = ScriptInfo::with_action_verb("My Window", "window:123", false, "Switch to");
        let actions = get_script_context_actions(&script);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert_eq!(run.title, "Switch to \"My Window\"");
    }

    #[test]
    fn cat14_run_shortcut_is_enter() {
        let script = ScriptInfo::new("Test", "/path/test.ts");
        let actions = get_script_context_actions(&script);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert_eq!(run.shortcut.as_deref(), Some("↵"));
    }

    // ================================================================
    // Cat 15: to_deeplink_name with whitespace variations
    // ================================================================

    #[test]
    fn cat15_single_space() {
        assert_eq!(to_deeplink_name("A B"), "a-b");
    }

    #[test]
    fn cat15_multiple_spaces() {
        assert_eq!(to_deeplink_name("A  B   C"), "a-b-c");
    }

    #[test]
    fn cat15_tabs_converted() {
        assert_eq!(to_deeplink_name("A\tB"), "a-b");
    }

    #[test]
    fn cat15_leading_trailing_spaces() {
        assert_eq!(to_deeplink_name("  hello  "), "hello");
    }

