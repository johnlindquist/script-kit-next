use super::*;
use crate::actions::{
    get_ai_command_bar_actions, get_note_switcher_actions,
    get_scriptlet_context_actions_with_custom, NoteSwitcherNoteInfo,
};

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

    #[test]
    fn test_protocol_action_deserializes_default_routing_flags() {
        let parsed: ProtocolAction = serde_json::from_str(r#"{"name":"Default Action"}"#)
            .expect("ProtocolAction should deserialize with defaults");

        assert_eq!(parsed.name, "Default Action");
        assert!(!parsed.has_action, "hasAction should default to false");
        assert!(parsed.is_visible(), "visible should default to true");
        assert!(parsed.should_close(), "close should default to true");
    }

    #[test]
    fn test_protocol_action_deserializes_explicit_camel_case_flags() {
        let parsed: ProtocolAction = serde_json::from_str(
            r#"{
                "name":"SDK Action",
                "hasAction":true,
                "visible":false,
                "close":false,
                "shortcut":"cmd+k",
                "value":"route-me"
            }"#,
        )
        .expect("ProtocolAction should deserialize camelCase flags");

        assert!(parsed.has_action);
        assert!(!parsed.is_visible());
        assert!(!parsed.should_close());
        assert_eq!(parsed.shortcut.as_deref(), Some("cmd+k"));
        assert_eq!(parsed.value.as_deref(), Some("route-me"));
    }

    #[test]
    fn test_scriptlet_custom_actions_are_converted_with_sdk_routing() {
        let script = ScriptInfo::scriptlet("Quick Open", "/path/to/urls.md#quick-open", None, None);
        let mut scriptlet = crate::scriptlets::Scriptlet::new(
            "Quick Open".to_string(),
            "bash".to_string(),
            "echo test".to_string(),
        );
        scriptlet.actions.push(crate::scriptlets::ScriptletAction {
            name: "Copy to Clipboard".to_string(),
            command: "copy-to-clipboard".to_string(),
            tool: "bash".to_string(),
            code: "echo '{{selection}}' | pbcopy".to_string(),
            inputs: vec!["selection".to_string()],
            shortcut: Some("cmd+shift+c".to_string()),
            description: Some("Copy current selection".to_string()),
        });

        let actions = get_scriptlet_context_actions_with_custom(&script, Some(&scriptlet));

        assert_eq!(actions[0].id, "run_script");
        assert_eq!(actions[1].id, "scriptlet_action:copy-to-clipboard");

        let custom = actions
            .iter()
            .find(|a| a.id == "scriptlet_action:copy-to-clipboard")
            .expect("custom scriptlet action should exist");
        assert!(custom.has_action, "custom scriptlet actions must route to SDK");
        assert_eq!(custom.value.as_deref(), Some("copy-to-clipboard"));
        assert_eq!(custom.shortcut.as_deref(), Some("⌘⇧C"));
        assert_eq!(custom.section.as_deref(), Some("Actions"));
    }

    #[test]
    fn test_note_switcher_preview_description_truncates_and_includes_relative_time() {
        let preview = "a".repeat(65);
        let notes = vec![NoteSwitcherNoteInfo {
            id: "note-1".to_string(),
            title: "Preview Note".to_string(),
            char_count: 999,
            is_current: false,
            is_pinned: false,
            preview,
            relative_time: "2m ago".to_string(),
        }];

        let actions = get_note_switcher_actions(&notes);
        let description = actions[0]
            .description
            .as_deref()
            .expect("description should be present");
        assert_eq!(description, format!("{}… · 2m ago", "a".repeat(60)));
    }

    #[test]
    fn test_ai_command_bar_extended_actions_have_expected_sections() {
        let actions = get_ai_command_bar_actions();

        let export = actions
            .iter()
            .find(|a| a.id == "export_markdown")
            .expect("missing export_markdown action");
        assert_eq!(export.section.as_deref(), Some("Export"));
        assert!(export.icon.is_some());

        let branch = actions
            .iter()
            .find(|a| a.id == "branch_from_last")
            .expect("missing branch_from_last action");
        assert_eq!(branch.section.as_deref(), Some("Actions"));
        assert!(branch.icon.is_some());

        let help = actions
            .iter()
            .find(|a| a.id == "toggle_shortcuts_help")
            .expect("missing toggle_shortcuts_help action");
        assert_eq!(help.section.as_deref(), Some("Help"));
        assert_eq!(help.shortcut.as_deref(), Some("⌘/"));
    }

    #[test]
    fn test_script_context_run_title_uses_custom_action_verb() {
        let script =
            ScriptInfo::with_action_verb("Window Switcher", "builtin:windows", false, "Switch to");
        let actions = get_script_context_actions(&script);
        let run_action = actions
            .iter()
            .find(|a| a.id == "run_script")
            .expect("run_script action should exist");

        assert_eq!(run_action.title, "Switch to \"Window Switcher\"");
        assert_eq!(run_action.description.as_deref(), Some("Switch to this item"));
    }
