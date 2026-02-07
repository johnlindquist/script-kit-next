    #[test]
    fn cat30_path_ids_unique() {
        let path = PathInfo {
            path: "/t".into(),
            name: "t".into(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&path);
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }

    #[test]
    fn cat30_builtin_ids_snake_case() {
        let script = ScriptInfo::new("T", "/t.ts");
        let actions = get_script_context_actions(&script);
        for a in &actions {
            assert!(
                !a.id.contains(' ') && !a.id.contains('-'),
                "Action ID '{}' should be snake_case",
                a.id
            );
        }
    }

    #[test]
    fn cat30_notes_ids_unique() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let ids: HashSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids.len(), actions.len());
    }
