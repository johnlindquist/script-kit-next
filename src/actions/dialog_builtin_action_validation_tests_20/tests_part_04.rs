    #[test]
    fn cat23_single_action_has_action_true() {
        let mut scriptlet = Scriptlet::new("T".into(), "bash".into(), "echo".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "Copy".into(),
            command: "copy".into(),
            tool: "bash".into(),
            code: "echo | pbcopy".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert_eq!(actions.len(), 1);
        assert!(actions[0].has_action);
    }

    #[test]
    fn cat23_action_id_prefix() {
        let mut scriptlet = Scriptlet::new("T".into(), "bash".into(), "echo".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "Do Thing".into(),
            command: "do-thing".into(),
            tool: "bash".into(),
            code: "echo".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert!(actions[0].id.starts_with("scriptlet_action:"));
    }

    #[test]
    fn cat23_action_value_is_command() {
        let mut scriptlet = Scriptlet::new("T".into(), "bash".into(), "echo".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "Do".into(),
            command: "my-cmd".into(),
            tool: "bash".into(),
            code: "echo".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert_eq!(actions[0].value, Some("my-cmd".to_string()));
    }

    #[test]
    fn cat23_action_shortcut_formatted() {
        let mut scriptlet = Scriptlet::new("T".into(), "bash".into(), "echo".into());
        scriptlet.actions = vec![ScriptletAction {
            name: "Copy".into(),
            command: "copy".into(),
            tool: "bash".into(),
            code: "echo".into(),
            inputs: vec![],
            shortcut: Some("cmd+c".into()),
            description: None,
        }];
        let actions = get_scriptlet_defined_actions(&scriptlet);
        assert_eq!(actions[0].shortcut, Some("⌘C".to_string()));
    }

    // =========================================================================
    // Category 24: Script context run_script title includes action_verb
    // =========================================================================

    #[test]
    fn cat24_default_verb_run() {
        let script = ScriptInfo::new("Test", "/test.ts");
        let actions = get_script_context_actions(&script);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert!(run.title.starts_with("Run"));
    }

    #[test]
    fn cat24_custom_verb_launch() {
        let script = ScriptInfo::with_action_verb("Safari", "/app", false, "Launch");
        let actions = get_script_context_actions(&script);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert!(run.title.starts_with("Launch"));
    }

    #[test]
    fn cat24_custom_verb_switch_to() {
        let script = ScriptInfo::with_action_verb("Window", "w:1", false, "Switch to");
        let actions = get_script_context_actions(&script);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert!(run.title.starts_with("Switch to"));
    }

    #[test]
    fn cat24_title_contains_name_in_quotes() {
        let script = ScriptInfo::new("My Script", "/test.ts");
        let actions = get_script_context_actions(&script);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert!(run.title.contains("\"My Script\""));
    }

    // =========================================================================
    // Category 25: Notes command bar section assignments
    // =========================================================================

    #[test]
    fn cat25_new_note_section_notes() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let nn = actions.iter().find(|a| a.id == "new_note").unwrap();
        assert_eq!(nn.section.as_ref().unwrap(), "Notes");
    }

    #[test]
    fn cat25_find_in_note_section_edit() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let fin = actions.iter().find(|a| a.id == "find_in_note").unwrap();
        assert_eq!(fin.section.as_ref().unwrap(), "Edit");
    }

    #[test]
    fn cat25_format_section_edit() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let fmt = actions.iter().find(|a| a.id == "format").unwrap();
        assert_eq!(fmt.section.as_ref().unwrap(), "Edit");
    }

    #[test]
    fn cat25_copy_note_as_section_copy() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let cna = actions.iter().find(|a| a.id == "copy_note_as").unwrap();
        assert_eq!(cna.section.as_ref().unwrap(), "Copy");
    }

    #[test]
    fn cat25_export_section_export() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let exp = actions.iter().find(|a| a.id == "export").unwrap();
        assert_eq!(exp.section.as_ref().unwrap(), "Export");
    }

    // =========================================================================
    // Category 26: AI command bar icon assignments
    // =========================================================================

    #[test]
    fn cat26_copy_response_icon_copy() {
        let actions = get_ai_command_bar_actions();
        let a = actions.iter().find(|a| a.id == "copy_response").unwrap();
        assert_eq!(a.icon, Some(IconName::Copy));
    }

    #[test]
    fn cat26_submit_icon_arrow_up() {
        let actions = get_ai_command_bar_actions();
        let a = actions.iter().find(|a| a.id == "submit").unwrap();
        assert_eq!(a.icon, Some(IconName::ArrowUp));
    }

    #[test]
    fn cat26_new_chat_icon_plus() {
        let actions = get_ai_command_bar_actions();
        let a = actions.iter().find(|a| a.id == "new_chat").unwrap();
        assert_eq!(a.icon, Some(IconName::Plus));
    }

    #[test]
    fn cat26_delete_chat_icon_trash() {
        let actions = get_ai_command_bar_actions();
        let a = actions.iter().find(|a| a.id == "delete_chat").unwrap();
        assert_eq!(a.icon, Some(IconName::Trash));
    }

    #[test]
    fn cat26_export_markdown_icon_filecode() {
        let actions = get_ai_command_bar_actions();
        let a = actions.iter().find(|a| a.id == "export_markdown").unwrap();
        assert_eq!(a.icon, Some(IconName::FileCode));
    }

    // =========================================================================
    // Category 27: Note switcher icon priority hierarchy
    // pinned > current > regular
    // =========================================================================

    #[test]
    fn cat27_pinned_icon_star_filled() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "1".into(),
            title: "N".into(),
            char_count: 0,
            is_current: false,
            is_pinned: true,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    }

    #[test]
    fn cat27_current_icon_check() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "1".into(),
            title: "N".into(),
            char_count: 0,
            is_current: true,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::Check));
    }

    #[test]
    fn cat27_regular_icon_file() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "1".into(),
            title: "N".into(),
            char_count: 0,
            is_current: false,
            is_pinned: false,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        assert_eq!(actions[0].icon, Some(IconName::File));
    }

    #[test]
    fn cat27_pinned_and_current_icon_star() {
        let notes = vec![NoteSwitcherNoteInfo {
            id: "1".into(),
            title: "N".into(),
            char_count: 0,
            is_current: true,
            is_pinned: true,
            preview: "".into(),
            relative_time: "".into(),
        }];
        let actions = get_note_switcher_actions(&notes);
        // Pinned takes priority over current
        assert_eq!(actions[0].icon, Some(IconName::StarFilled));
    }

    // =========================================================================
    // Category 28: Note switcher empty state placeholder
    // =========================================================================

    #[test]
    fn cat28_empty_notes_single_placeholder() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions.len(), 1);
    }

    #[test]
    fn cat28_empty_notes_id_no_notes() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions[0].id, "no_notes");
    }

    #[test]
    fn cat28_empty_notes_title() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions[0].title, "No notes yet");
    }

    #[test]
    fn cat28_empty_notes_icon_plus() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions[0].icon, Some(IconName::Plus));
    }

    #[test]
    fn cat28_empty_notes_section_notes() {
        let actions = get_note_switcher_actions(&[]);
        assert_eq!(actions[0].section.as_ref().unwrap(), "Notes");
    }

    // =========================================================================
    // Category 29: Cross-context all descriptions non-empty
    // =========================================================================

    #[test]
    fn cat29_script_all_have_descriptions() {
        let script = ScriptInfo::new("T", "/t.ts");
        let actions = get_script_context_actions(&script);
        for a in &actions {
            assert!(
                a.description.is_some(),
                "Script action '{}' missing description",
                a.id
            );
        }
    }

    #[test]
    fn cat29_clipboard_all_have_descriptions() {
        let entry = ClipboardEntryInfo {
            id: "t".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hi".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        for a in &actions {
            assert!(
                a.description.is_some(),
                "Clipboard action '{}' missing description",
                a.id
            );
        }
    }

    #[test]
    fn cat29_ai_all_have_descriptions() {
        let actions = get_ai_command_bar_actions();
        for a in &actions {
            assert!(
                a.description.is_some(),
                "AI action '{}' missing description",
                a.id
            );
        }
    }

    #[test]
    fn cat29_path_all_have_descriptions() {
        let path = PathInfo {
            path: "/t".into(),
            name: "t".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&path);
        for a in &actions {
            assert!(
                a.description.is_some(),
                "Path action '{}' missing description",
                a.id
            );
        }
    }

    #[test]
    fn cat29_file_all_have_descriptions() {
        let file = FileInfo {
            path: "/t".into(),
            name: "t".into(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file);
        for a in &actions {
            assert!(
                a.description.is_some(),
                "File action '{}' missing description",
                a.id
            );
        }
    }

    #[test]
    fn cat29_notes_all_have_descriptions() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        for a in &actions {
            assert!(
                a.description.is_some(),
                "Notes action '{}' missing description",
                a.id
            );
        }
    }

    // =========================================================================
    // Category 30: Cross-context ID uniqueness and snake_case invariant
    // =========================================================================

    #[test]
    fn cat30_script_ids_unique() {
        let script = ScriptInfo::new("T", "/t.ts");
        let actions = get_script_context_actions(&script);
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn cat30_clipboard_text_ids_unique() {
        let entry = ClipboardEntryInfo {
            id: "t".into(),
            content_type: ContentType::Text,
            pinned: false,
            preview: "hi".into(),
            image_dimensions: None,
            frontmost_app_name: None,
        };
        let actions = get_clipboard_history_context_actions(&entry);
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn cat30_ai_ids_unique() {
        let actions = get_ai_command_bar_actions();
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

