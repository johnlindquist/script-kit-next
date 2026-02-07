    #[test]
    fn test_alias_actions_dynamic() {
        // Test that alias actions change based on existing alias

        // No alias -> show "Add Alias"
        let no_alias = ScriptInfo::new("test", "/path/test.ts");
        let actions_no_alias = get_script_context_actions(&no_alias);
        let ids_no_alias: Vec<&str> = actions_no_alias.iter().map(|a| a.id.as_str()).collect();

        assert!(
            ids_no_alias.contains(&"add_alias"),
            "Should have add_alias when no alias exists"
        );
        assert!(
            !ids_no_alias.contains(&"update_alias"),
            "Should NOT have update_alias when no alias exists"
        );
        assert!(
            !ids_no_alias.contains(&"remove_alias"),
            "Should NOT have remove_alias when no alias exists"
        );

        // Has alias -> show "Update" and "Remove"
        let with_alias = ScriptInfo::with_shortcut_and_alias(
            "test",
            "/path/test.ts",
            None,
            Some("ts".to_string()),
        );
        let actions_with_alias = get_script_context_actions(&with_alias);
        let ids_with_alias: Vec<&str> = actions_with_alias.iter().map(|a| a.id.as_str()).collect();

        assert!(
            !ids_with_alias.contains(&"add_alias"),
            "Should NOT have add_alias when alias exists"
        );
        assert!(
            ids_with_alias.contains(&"update_alias"),
            "Should have update_alias when alias exists"
        );
        assert!(
            ids_with_alias.contains(&"remove_alias"),
            "Should have remove_alias when alias exists"
        );
    }

    #[test]
    fn test_frecency_reset_action() {
        // Test that "Reset Ranking" action only appears for suggested items

        // Not suggested -> no reset action
        let not_suggested = ScriptInfo::new("test", "/path/test.ts");
        let actions_not_suggested = get_script_context_actions(&not_suggested);
        let ids_not_suggested: Vec<&str> = actions_not_suggested
            .iter()
            .map(|a| a.id.as_str())
            .collect();

        assert!(
            !ids_not_suggested.contains(&"reset_ranking"),
            "Should NOT have reset_ranking when not suggested"
        );

        // Is suggested -> has reset action
        let suggested =
            ScriptInfo::new("test", "/path/test.ts").with_frecency(true, Some("/path".into()));
        let actions_suggested = get_script_context_actions(&suggested);
        let ids_suggested: Vec<&str> = actions_suggested.iter().map(|a| a.id.as_str()).collect();

        assert!(
            ids_suggested.contains(&"reset_ranking"),
            "Should have reset_ranking when is_suggested=true"
        );
    }

    #[test]
    fn test_protocol_action_visibility() {
        // Test that visible: false actions are filtered out

        let visible_action = ProtocolAction {
            name: "Visible Action".to_string(),
            description: None,
            shortcut: None,
            value: None,
            has_action: true,
            visible: Some(true),
            close: None,
        };
        assert!(visible_action.is_visible());

        let hidden_action = ProtocolAction {
            name: "Hidden Action".to_string(),
            description: None,
            shortcut: None,
            value: None,
            has_action: true,
            visible: Some(false),
            close: None,
        };
        assert!(!hidden_action.is_visible());

        let default_visible = ProtocolAction {
            name: "Default Visible".to_string(),
            description: None,
            shortcut: None,
            value: None,
            has_action: true,
            visible: None, // None defaults to visible
            close: None,
        };
        assert!(
            default_visible.is_visible(),
            "visible: None should default to true"
        );
    }

    #[test]
    fn test_protocol_action_close_behavior() {
        // Test that close: false keeps dialog open after action

        let closes_dialog = ProtocolAction {
            name: "Closes".to_string(),
            description: None,
            shortcut: None,
            value: None,
            has_action: true,
            visible: None,
            close: Some(true),
        };
        assert!(closes_dialog.should_close());

        let stays_open = ProtocolAction {
            name: "Stays Open".to_string(),
            description: None,
            shortcut: None,
            value: None,
            has_action: true,
            visible: None,
            close: Some(false),
        };
        assert!(!stays_open.should_close());

        let default_closes = ProtocolAction {
            name: "Default Close".to_string(),
            description: None,
            shortcut: None,
            value: None,
            has_action: true,
            visible: None,
            close: None, // None defaults to close
        };
        assert!(
            default_closes.should_close(),
            "close: None should default to true"
        );
    }
