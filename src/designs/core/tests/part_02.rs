    use crate::metadata_parser::TypedMetadata;
    use crate::scripts::{MatchIndices, Script, ScriptMatch, Scriptlet, ScriptletMatch};
    use std::path::PathBuf;
    use std::sync::Arc;

    fn make_test_script(name: &str) -> Script {
        Script {
            name: name.to_string(),
            path: PathBuf::from(format!(
                "/test/{}.ts",
                name.to_lowercase().replace(' ', "-")
            )),
            extension: "ts".to_string(),
            ..Default::default()
        }
    }

    fn make_script_search_result(script: Script) -> SearchResult {
        SearchResult::Script(ScriptMatch {
            filename: format!("{}.ts", script.name.to_lowercase().replace(' ', "-")),
            script: Arc::new(script),
            score: 100,
            match_indices: MatchIndices::default(),
        })
    }

    fn make_scriptlet_search_result(scriptlet: Scriptlet) -> SearchResult {
        SearchResult::Scriptlet(ScriptletMatch {
            scriptlet: Arc::new(scriptlet),
            score: 100,
            display_file_path: None,
            match_indices: MatchIndices::default(),
        })
    }

    #[test]
    fn test_search_accessories_hide_source_hint_during_filtering() {
        let mut script = make_test_script("Clipboard Variables");
        script.kit_name = Some("clipboard".to_string());
        script.shortcut = Some("cmd shift v".to_string());
        let result = make_script_search_result(script);

        let accessories = resolve_search_accessories(&result, "clip");
        assert!(
            accessories.type_tag.is_some(),
            "type label should stay visible"
        );
        assert_eq!(
            accessories.source_hint, None,
            "source/category metadata should be hidden during filtering"
        );
    }

    #[test]
    fn test_resolve_tool_badge_hidden_during_filtering_for_scriptlets() {
        let scriptlet = Scriptlet {
            name: "Paste Rich Link".to_string(),
            description: Some("Paste as markdown link".to_string()),
            code: "https://example.com".to_string(),
            tool: "paste".to_string(),
            shortcut: None,
            keyword: Some("!mdlink".to_string()),
            group: Some("Clipboard Transformations".to_string()),
            file_path: None,
            command: None,
            alias: None,
        };
        let result = make_scriptlet_search_result(scriptlet);

        assert_eq!(resolve_tool_badge(&result, true), None);
    }

    #[test]
    fn test_resolve_tool_badge_kept_when_not_filtering_for_scriptlets() {
        let scriptlet = Scriptlet {
            name: "Paste Rich Link".to_string(),
            description: Some("Paste as markdown link".to_string()),
            code: "https://example.com".to_string(),
            tool: "paste".to_string(),
            shortcut: None,
            keyword: Some("!mdlink".to_string()),
            group: Some("Clipboard Transformations".to_string()),
            file_path: None,
            command: None,
            alias: None,
        };
        let result = make_scriptlet_search_result(scriptlet);

        assert_eq!(
            resolve_tool_badge(&result, false),
            Some("paste".to_string())
        );
    }

    #[test]
    fn test_auto_description_preserves_explicit() {
        let mut s = make_test_script("My Script");
        s.description = Some("Explicit description".to_string());
        assert_eq!(
            auto_description_for_script(&s),
            Some("Explicit description".to_string())
        );
    }

    #[test]
    fn test_auto_description_cron() {
        let mut s = make_test_script("Daily Backup");
        s.typed_metadata = Some(TypedMetadata {
            cron: Some("0 0 * * *".to_string()),
            ..Default::default()
        });
        assert_eq!(
            auto_description_for_script(&s),
            Some("Cron: 0 0 * * *".to_string())
        );
    }

    #[test]
    fn test_auto_description_schedule_over_cron() {
        let mut s = make_test_script("Scheduled Task");
        s.typed_metadata = Some(TypedMetadata {
            schedule: Some("every weekday at 9am".to_string()),
            cron: Some("0 9 * * 1-5".to_string()),
            ..Default::default()
        });
        // schedule takes priority over cron
        assert_eq!(
            auto_description_for_script(&s),
            Some("Scheduled: every weekday at 9am".to_string())
        );
    }

    #[test]
    fn test_auto_description_watch() {
        let mut s = make_test_script("Config Watcher");
        s.typed_metadata = Some(TypedMetadata {
            watch: vec!["~/.config/**".to_string()],
            ..Default::default()
        });
        assert_eq!(
            auto_description_for_script(&s),
            Some("Watches: ~/.config/**".to_string())
        );
    }

    #[test]
    fn test_auto_description_watch_truncates_long_pattern() {
        let mut s = make_test_script("Long Watcher");
        let long_pattern =
            "/very/long/path/to/some/deeply/nested/directory/with/many/levels/**/*.json"
                .to_string();
        s.typed_metadata = Some(TypedMetadata {
            watch: vec![long_pattern],
            ..Default::default()
        });
        let desc = auto_description_for_script(&s).unwrap();
        assert!(desc.starts_with("Watches: "));
        assert!(desc.ends_with("..."));
    }

    #[test]
    fn test_auto_description_background() {
        let mut s = make_test_script("Bg Task");
        s.typed_metadata = Some(TypedMetadata {
            background: true,
            ..Default::default()
        });
        assert_eq!(
            auto_description_for_script(&s),
            Some("Background process".to_string())
        );
    }

    #[test]
    fn test_auto_description_system() {
        let mut s = make_test_script("Sys Handler");
        s.typed_metadata = Some(TypedMetadata {
            system: true,
            ..Default::default()
        });
        assert_eq!(
            auto_description_for_script(&s),
            Some("System event handler".to_string())
        );
    }

    #[test]
    fn test_auto_description_filename_fallback() {
        // Script name differs from filename
        let s = make_test_script("My Script");
        // Path is /test/my-script.ts, filename is "my-script.ts", name is "My Script"
        let desc = auto_description_for_script(&s);
        assert_eq!(desc, Some("my-script.ts".to_string()));
    }

    #[test]
    fn test_auto_description_no_filename_when_same_as_name() {
        let mut s = make_test_script("exact");
        s.path = PathBuf::from("/test/exact");
        s.name = "exact".to_string();
        // filename == name → falls through to language label (extension is "ts")
        assert_eq!(
            auto_description_for_script(&s),
            Some("TypeScript".to_string())
        );
    }

    // =========================================================================
    // Grouped view hint tests
    // =========================================================================

    #[test]
    fn test_hint_shortcut_shows_alias() {
        let mut s = make_test_script("Git Commit");
        s.shortcut = Some("opt g".to_string());
        s.alias = Some("gc".to_string());
        assert_eq!(grouped_view_hint_for_script(&s), Some("/gc".to_string()));
    }

    #[test]
    fn test_hint_shortcut_falls_back_to_tags() {
        let mut s = make_test_script("Git Commit");
        s.shortcut = Some("opt g".to_string());
        s.typed_metadata = Some(TypedMetadata {
            tags: vec!["git".to_string(), "dev".to_string()],
            ..Default::default()
        });
        // No alias, so falls back to tags
        assert_eq!(
            grouped_view_hint_for_script(&s),
            Some("git · dev".to_string())
        );
    }

    #[test]
    fn test_hint_alias_badge_shows_tags() {
        let mut s = make_test_script("Git Commit");
        s.alias = Some("gc".to_string());
        s.typed_metadata = Some(TypedMetadata {
            tags: vec!["git".to_string()],
            ..Default::default()
        });
        // Alias is badge, tags shown as hint
        assert_eq!(grouped_view_hint_for_script(&s), Some("git".to_string()));
    }

    #[test]
    fn test_hint_alias_badge_falls_back_to_kit() {
        let mut s = make_test_script("Capture Window");
        s.alias = Some("cw".to_string());
        s.kit_name = Some("cleanshot".to_string());
        // Alias is badge, no tags, so falls back to kit name
        assert_eq!(
            grouped_view_hint_for_script(&s),
            Some("cleanshot".to_string())
        );
    }

    #[test]
    fn test_hint_no_badge_shows_tags() {
        let mut s = make_test_script("Notes");
        s.typed_metadata = Some(TypedMetadata {
            tags: vec!["productivity".to_string(), "notes".to_string()],
            ..Default::default()
        });
        assert_eq!(
            grouped_view_hint_for_script(&s),
            Some("productivity · notes".to_string())
        );
    }

    #[test]
    fn test_hint_no_badge_falls_back_to_kit() {
        let mut s = make_test_script("Annotate");
        s.kit_name = Some("cleanshot".to_string());
        assert_eq!(
            grouped_view_hint_for_script(&s),
            Some("cleanshot".to_string())
        );
    }

    #[test]
    fn test_hint_main_kit_not_shown() {
        let mut s = make_test_script("Notes");
        s.kit_name = Some("main".to_string());
        // "main" kit should not produce a hint
        assert_eq!(grouped_view_hint_for_script(&s), None);
    }

    #[test]
    fn test_scriptlet_hint_group_shown() {
        use crate::scripts::Scriptlet;
        let sl = Scriptlet {
            name: "Open GitHub".to_string(),
            description: None,
            code: "open https://github.com".to_string(),
            tool: "open".to_string(),
            shortcut: None,
            keyword: None,
            group: Some("Development".to_string()),
            file_path: None,
            command: None,
            alias: None,
        };
        assert_eq!(
            grouped_view_hint_for_scriptlet(&sl),
            Some("Development".to_string())
        );
    }

    #[test]
    fn test_scriptlet_hint_main_group_hidden() {
        use crate::scripts::Scriptlet;
        let sl = Scriptlet {
            name: "Hello".to_string(),
            description: None,
            code: "echo hello".to_string(),
            tool: "bash".to_string(),
            shortcut: None,
            keyword: None,
            group: Some("main".to_string()),
            file_path: None,
            command: None,
            alias: None,
        };
        assert_eq!(grouped_view_hint_for_scriptlet(&sl), None);
    }

    // =========================================================================
    // Enter text hint tests
    // =========================================================================

    #[test]
    fn test_hint_enter_text_shown_as_fallback() {
        let mut s = make_test_script("Deploy");
        s.kit_name = Some("main".to_string());
        s.typed_metadata = Some(TypedMetadata {
            enter: Some("Deploy Now".to_string()),
            ..Default::default()
        });
        // No tags, main kit → falls back to enter text
        assert_eq!(
            grouped_view_hint_for_script(&s),
            Some("→ Deploy Now".to_string())
        );
    }

    #[test]
    fn test_hint_enter_text_not_shown_for_generic_run() {
        let mut s = make_test_script("Basic");
        s.kit_name = Some("main".to_string());
        s.typed_metadata = Some(TypedMetadata {
            enter: Some("Run".to_string()),
            ..Default::default()
        });
        // "Run" is generic, should not show
        assert_eq!(grouped_view_hint_for_script(&s), None);
    }

    #[test]
    fn test_hint_tags_preferred_over_enter_text() {
        let mut s = make_test_script("Deploy");
        s.typed_metadata = Some(TypedMetadata {
            enter: Some("Deploy Now".to_string()),
            tags: vec!["devops".to_string()],
            ..Default::default()
        });
        // Tags take priority over enter text
        assert_eq!(grouped_view_hint_for_script(&s), Some("devops".to_string()));
    }

