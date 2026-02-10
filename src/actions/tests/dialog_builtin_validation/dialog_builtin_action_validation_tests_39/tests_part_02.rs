    #[test]
    fn file_context_file_has_more_actions_than_dir() {
        let file = FileInfo {
            name: "readme.md".into(),
            path: "/readme.md".into(),
            is_dir: false,
            file_type: FileType::File,
        };
        let dir = FileInfo {
            name: "src".into(),
            path: "/src".into(),
            is_dir: true,
            file_type: FileType::Directory,
        };
        let file_actions = get_file_context_actions(&file);
        let dir_actions = get_file_context_actions(&dir);
        // Files have quick_look, dirs don't (on macOS)
        assert!(file_actions.len() >= dir_actions.len());
    }

    #[test]
    fn file_context_dir_no_quick_look() {
        let dir = FileInfo {
            name: "src".into(),
            path: "/src".into(),
            is_dir: true,
            file_type: FileType::Directory,
        };
        let actions = get_file_context_actions(&dir);
        assert!(!actions.iter().any(|a| a.id == "quick_look"));
    }

    #[test]
    fn file_context_both_have_reveal_in_finder() {
        let file = FileInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
            file_type: FileType::File,
        };
        let dir = FileInfo {
            name: "b".into(),
            path: "/b".into(),
            is_dir: true,
            file_type: FileType::Directory,
        };
        assert!(get_file_context_actions(&file)
            .iter()
            .any(|a| a.id == "reveal_in_finder"));
        assert!(get_file_context_actions(&dir)
            .iter()
            .any(|a| a.id == "reveal_in_finder"));
    }

    #[test]
    fn file_context_both_have_copy_path_and_copy_filename() {
        let file = FileInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
            file_type: FileType::File,
        };
        let actions = get_file_context_actions(&file);
        assert!(actions.iter().any(|a| a.id == "copy_path"));
        assert!(actions.iter().any(|a| a.id == "copy_filename"));
    }

    // =========================================================================
    // 12. File context: copy_filename shortcut ⌘C differs from path context (no shortcut)
    // =========================================================================

    #[test]
    fn file_context_copy_filename_has_shortcut() {
        let file = FileInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
            file_type: FileType::File,
        };
        let actions = get_file_context_actions(&file);
        let cf = actions.iter().find(|a| a.id == "copy_filename").unwrap();
        assert_eq!(cf.shortcut.as_deref(), Some("⌘C"));
    }

    #[test]
    fn path_context_copy_filename_has_no_shortcut() {
        let info = PathInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let cf = actions.iter().find(|a| a.id == "copy_filename").unwrap();
        assert!(cf.shortcut.is_none());
    }

    #[test]
    fn file_context_copy_path_shortcut_matches_path_context() {
        let file = FileInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
            file_type: FileType::File,
        };
        let path = PathInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
        };
        let fc = get_file_context_actions(&file);
        let pc = get_path_context_actions(&path);
        let fcp = fc.iter().find(|a| a.id == "copy_path").unwrap();
        let pcp = pc.iter().find(|a| a.id == "copy_path").unwrap();
        assert_eq!(fcp.shortcut, pcp.shortcut);
    }

    #[test]
    fn file_and_path_copy_path_shortcut_is_cmd_shift_c() {
        let file = FileInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
            file_type: FileType::File,
        };
        let actions = get_file_context_actions(&file);
        let cp = actions.iter().find(|a| a.id == "copy_path").unwrap();
        assert_eq!(cp.shortcut.as_deref(), Some("⌘⇧C"));
    }

    // =========================================================================
    // 13. Path context: total action count file vs dir
    // =========================================================================

    #[test]
    fn path_context_dir_has_one_more_than_common() {
        let file = PathInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
        };
        let dir = PathInfo {
            name: "b".into(),
            path: "/b".into(),
            is_dir: true,
        };
        // Both have same total: primary + 6 common = 7
        assert_eq!(
            get_path_context_actions(&file).len(),
            get_path_context_actions(&dir).len()
        );
    }

    #[test]
    fn path_context_file_primary_is_select_file() {
        let info = PathInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions[0].id, "select_file");
    }

    #[test]
    fn path_context_dir_primary_is_open_directory() {
        let info = PathInfo {
            name: "b".into(),
            path: "/b".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions[0].id, "open_directory");
    }

    #[test]
    fn path_context_both_have_7_actions() {
        let file = PathInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
        };
        let dir = PathInfo {
            name: "b".into(),
            path: "/b".into(),
            is_dir: true,
        };
        assert_eq!(get_path_context_actions(&file).len(), 7);
        assert_eq!(get_path_context_actions(&dir).len(), 7);
    }

    // =========================================================================
    // 14. Path context: move_to_trash is always last
    // =========================================================================

    #[test]
    fn path_context_file_last_is_move_to_trash() {
        let info = PathInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions.last().unwrap().id, "move_to_trash");
    }

    #[test]
    fn path_context_dir_last_is_move_to_trash() {
        let info = PathInfo {
            name: "b".into(),
            path: "/b".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        assert_eq!(actions.last().unwrap().id, "move_to_trash");
    }

    #[test]
    fn path_context_move_to_trash_desc_file() {
        let info = PathInfo {
            name: "a.txt".into(),
            path: "/a.txt".into(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let trash = actions.last().unwrap();
        assert!(trash.description.as_ref().unwrap().contains("file"));
    }

    #[test]
    fn path_context_move_to_trash_desc_folder() {
        let info = PathInfo {
            name: "b".into(),
            path: "/b".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        let trash = actions.last().unwrap();
        assert!(trash.description.as_ref().unwrap().contains("folder"));
    }

    // =========================================================================
    // 15. Script context: with_frecency adds reset_ranking
    // =========================================================================

    #[test]
    fn script_with_frecency_has_reset_ranking() {
        let info = ScriptInfo::new("test", "/test.ts").with_frecency(true, Some("/test.ts".into()));
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "reset_ranking"));
    }

    #[test]
    fn script_without_frecency_no_reset_ranking() {
        let info = ScriptInfo::new("test", "/test.ts");
        let actions = get_script_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "reset_ranking"));
    }

    #[test]
    fn script_with_frecency_reset_ranking_is_last() {
        let info = ScriptInfo::new("test", "/test.ts").with_frecency(true, Some("/test.ts".into()));
        let actions = get_script_context_actions(&info);
        assert_eq!(actions.last().unwrap().id, "reset_ranking");
    }

    #[test]
    fn script_with_frecency_reset_ranking_no_shortcut() {
        let info = ScriptInfo::new("test", "/test.ts").with_frecency(true, Some("/test.ts".into()));
        let actions = get_script_context_actions(&info);
        let rr = actions.iter().find(|a| a.id == "reset_ranking").unwrap();
        assert_eq!(rr.shortcut.as_deref(), Some("⌃⌘R"));
    }

    // =========================================================================
    // 16. Script context: agent has no view_logs but has copy_path
    // =========================================================================

    #[test]
    fn agent_context_no_view_logs() {
        let mut info = ScriptInfo::new("my-agent", "/agents/my-agent.md");
        info.is_agent = true;
        info.is_script = false;
        let actions = get_script_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
    }

    #[test]
    fn agent_context_has_copy_path() {
        let mut info = ScriptInfo::new("my-agent", "/agents/my-agent.md");
        info.is_agent = true;
        info.is_script = false;
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "copy_path"));
    }

    #[test]
    fn agent_context_has_reveal_in_finder() {
        let mut info = ScriptInfo::new("my-agent", "/agents/my-agent.md");
        info.is_agent = true;
        info.is_script = false;
        let actions = get_script_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
    }

    #[test]
    fn agent_context_edit_title_says_agent() {
        let mut info = ScriptInfo::new("my-agent", "/agents/my-agent.md");
        info.is_agent = true;
        info.is_script = false;
        let actions = get_script_context_actions(&info);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit.title, "Edit Agent");
    }

    // =========================================================================
    // 17. Scriptlet context: total action count without custom actions
    // =========================================================================

    #[test]
    fn scriptlet_context_no_custom_no_shortcut_no_alias_count() {
        let info = ScriptInfo::scriptlet("My Script", "/scripts.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        // run + add_shortcut + add_alias + edit_scriptlet + reveal + copy_path + copy_content + copy_deeplink = 8
        assert_eq!(actions.len(), 8);
    }

    #[test]
    fn scriptlet_context_with_shortcut_adds_two_actions() {
        let info = ScriptInfo::scriptlet("My Script", "/scripts.md", Some("cmd+m".into()), None);
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        // run + update_shortcut + remove_shortcut + add_alias + edit + reveal + copy_path + copy_content + copy_deeplink = 9
        assert_eq!(actions.len(), 9);
    }

    #[test]
    fn scriptlet_context_with_both_shortcut_alias_count() {
        let info = ScriptInfo::scriptlet(
            "My Script",
            "/scripts.md",
            Some("cmd+m".into()),
            Some("ms".into()),
        );
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        // run + update_shortcut + remove_shortcut + update_alias + remove_alias + edit + reveal + copy_path + copy_content + copy_deeplink = 10
        assert_eq!(actions.len(), 10);
    }

    #[test]
    fn scriptlet_context_suggested_adds_reset_ranking() {
        let info = ScriptInfo::scriptlet("My Script", "/scripts.md", None, None)
            .with_frecency(true, Some("/scripts.md".into()));
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        assert!(actions.iter().any(|a| a.id == "reset_ranking"));
    }

    // =========================================================================
    // 18. Scriptlet context with custom: custom actions appear after run
    // =========================================================================

    #[test]
    fn scriptlet_custom_actions_appear_after_run() {
        let info = ScriptInfo::scriptlet("My Script", "/scripts.md", None, None);
        let mut scriptlet = Scriptlet::new("My Script".into(), "bash".into(), "echo hi".into());
        scriptlet.actions.push(crate::scriptlets::ScriptletAction {
            name: "Copy".into(),
            command: "copy".into(),
            tool: "bash".into(),
            code: "echo copy".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        });
        let actions = get_scriptlet_context_actions_with_custom(&info, Some(&scriptlet));
        assert_eq!(actions[0].id, "run_script");
        assert_eq!(actions[1].id, "scriptlet_action:copy");
    }

    #[test]
    fn scriptlet_custom_actions_have_has_action_true() {
        let info = ScriptInfo::scriptlet("My Script", "/scripts.md", None, None);
        let mut scriptlet = Scriptlet::new("My Script".into(), "bash".into(), "echo hi".into());
        scriptlet.actions.push(crate::scriptlets::ScriptletAction {
            name: "Do Thing".into(),
            command: "do-thing".into(),
            tool: "bash".into(),
            code: "echo thing".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        });
        let actions = get_scriptlet_context_actions_with_custom(&info, Some(&scriptlet));
        let custom = actions
            .iter()
            .find(|a| a.id == "scriptlet_action:do-thing")
            .unwrap();
        assert!(custom.has_action);
    }

    #[test]
    fn scriptlet_custom_action_value_is_command() {
        let info = ScriptInfo::scriptlet("My Script", "/scripts.md", None, None);
        let mut scriptlet = Scriptlet::new("My Script".into(), "bash".into(), "echo hi".into());
        scriptlet.actions.push(crate::scriptlets::ScriptletAction {
            name: "Do Thing".into(),
            command: "do-thing".into(),
            tool: "bash".into(),
            code: "echo thing".into(),
            inputs: vec![],
            shortcut: None,
            description: None,
        });
        let actions = get_scriptlet_context_actions_with_custom(&info, Some(&scriptlet));
        let custom = actions
            .iter()
            .find(|a| a.id == "scriptlet_action:do-thing")
            .unwrap();
        assert_eq!(custom.value.as_deref(), Some("do-thing"));
    }

    #[test]
    fn scriptlet_no_scriptlet_no_custom_actions() {
        let info = ScriptInfo::scriptlet("My Script", "/scripts.md", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&info, None);
        assert!(!actions
            .iter()
            .any(|a| a.id.starts_with("scriptlet_action:")));
    }

    // =========================================================================
    // 19. AI bar: paste_image details
    // =========================================================================

    #[test]
    fn ai_bar_paste_image_shortcut() {
        let actions = get_ai_command_bar_actions();
        let pi = actions.iter().find(|a| a.id == "paste_image").unwrap();
        assert_eq!(pi.shortcut.as_deref(), Some("⌘V"));
    }

    #[test]
    fn ai_bar_paste_image_icon() {
        let actions = get_ai_command_bar_actions();
        let pi = actions.iter().find(|a| a.id == "paste_image").unwrap();
        assert_eq!(pi.icon, Some(IconName::File));
    }
