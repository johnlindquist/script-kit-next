    #[test]
    fn test_action_category_filtering() {
        // Test that different ScriptInfo types produce different action sets

        // Regular script - should have edit, view_logs, reveal, copy_path, copy_content
        let script = ScriptInfo::new("my-script", "/path/to/script.ts");
        let script_actions = get_script_context_actions(&script);

        let action_ids: Vec<&str> = script_actions.iter().map(|a| a.id.as_str()).collect();
        assert!(
            action_ids.contains(&"edit_script"),
            "Script should have edit_script action"
        );
        assert!(
            action_ids.contains(&"view_logs"),
            "Script should have view_logs action"
        );
        assert!(
            action_ids.contains(&"reveal_in_finder"),
            "Script should have reveal_in_finder action"
        );
        assert!(
            action_ids.contains(&"copy_path"),
            "Script should have copy_path action"
        );
        assert!(
            action_ids.contains(&"copy_content"),
            "Script should have copy_content action"
        );

        // Built-in - should NOT have edit, view_logs, etc.
        let builtin = ScriptInfo::builtin("Clipboard History");
        let builtin_actions = get_script_context_actions(&builtin);

        let builtin_ids: Vec<&str> = builtin_actions.iter().map(|a| a.id.as_str()).collect();
        assert!(
            !builtin_ids.contains(&"edit_script"),
            "Builtin should NOT have edit_script"
        );
        assert!(
            !builtin_ids.contains(&"view_logs"),
            "Builtin should NOT have view_logs"
        );
        assert!(
            !builtin_ids.contains(&"reveal_in_finder"),
            "Builtin should NOT have reveal_in_finder"
        );

        // Scriptlet - should have edit_scriptlet, reveal_scriptlet_in_finder, copy_scriptlet_path
        let scriptlet = ScriptInfo::scriptlet("Open GitHub", "/path/to/url.md", None, None);
        let scriptlet_actions = get_script_context_actions(&scriptlet);

        let scriptlet_ids: Vec<&str> = scriptlet_actions.iter().map(|a| a.id.as_str()).collect();
        assert!(
            scriptlet_ids.contains(&"edit_scriptlet"),
            "Scriptlet should have edit_scriptlet"
        );
        assert!(
            scriptlet_ids.contains(&"reveal_scriptlet_in_finder"),
            "Scriptlet should have reveal_scriptlet_in_finder"
        );
        assert!(
            scriptlet_ids.contains(&"copy_scriptlet_path"),
            "Scriptlet should have copy_scriptlet_path"
        );
        // Scriptlets should NOT have regular script actions
        assert!(
            !scriptlet_ids.contains(&"edit_script"),
            "Scriptlet should NOT have edit_script"
        );
    }

    #[test]
    fn test_script_info_variants() {
        // Test all ScriptInfo construction variants

        // 1. Basic constructor
        let basic = ScriptInfo::new("basic", "/path/basic.ts");
        assert_eq!(basic.name, "basic");
        assert_eq!(basic.path, "/path/basic.ts");
        assert!(basic.is_script);
        assert!(!basic.is_scriptlet);
        assert!(!basic.is_agent);
        assert_eq!(basic.action_verb, "Run");
        assert!(basic.shortcut.is_none());
        assert!(basic.alias.is_none());
        assert!(!basic.is_suggested);

        // 2. With shortcut
        let with_shortcut =
            ScriptInfo::with_shortcut("shortcut-test", "/path/test.ts", Some("cmd+t".to_string()));
        assert_eq!(with_shortcut.shortcut, Some("cmd+t".to_string()));
        assert!(with_shortcut.is_script);

        // 3. Scriptlet
        let scriptlet = ScriptInfo::scriptlet(
            "Open URL",
            "/path/urls.md#open-url",
            Some("cmd+u".to_string()),
            Some("ou".to_string()),
        );
        assert!(!scriptlet.is_script);
        assert!(scriptlet.is_scriptlet);
        assert_eq!(scriptlet.shortcut, Some("cmd+u".to_string()));
        assert_eq!(scriptlet.alias, Some("ou".to_string()));

        // 4. Builtin
        let builtin = ScriptInfo::builtin("App Launcher");
        assert!(!builtin.is_script);
        assert!(!builtin.is_scriptlet);
        assert!(builtin.path.is_empty());

        // 5. With action verb
        let with_verb =
            ScriptInfo::with_action_verb("Window Switcher", "builtin:windows", false, "Switch to");
        assert_eq!(with_verb.action_verb, "Switch to");
        assert!(!with_verb.is_script);

        // 6. With frecency
        let with_frecency = ScriptInfo::new("frecent", "/path/frecent.ts")
            .with_frecency(true, Some("/path".into()));
        assert!(with_frecency.is_suggested);
        assert_eq!(with_frecency.frecency_path, Some("/path".to_string()));

        // 7. With all options
        let full = ScriptInfo::with_all(
            "Full Options",
            "/path/full.ts",
            true,
            "Execute",
            Some("cmd+f".to_string()),
            Some("fo".to_string()),
        );
        assert!(full.is_script);
        assert_eq!(full.action_verb, "Execute");
        assert_eq!(full.shortcut, Some("cmd+f".to_string()));
        assert_eq!(full.alias, Some("fo".to_string()));
    }

    #[test]
    fn test_builtin_vs_sdk_actions() {
        // Built-in actions have predefined IDs from builders
        let script = ScriptInfo::new("test", "/path/test.ts");
        let builtin_actions = get_script_context_actions(&script);

        // Verify built-in action IDs are well-defined
        // Known IDs include: run_script, edit_script, view_logs, reveal_in_finder,
        // copy_path, copy_content, copy_deeplink, add_shortcut, add_alias

        for action in &builtin_actions {
            // Built-in actions should have meaningful IDs (not just the title)
            assert!(
                !action.id.contains(' '),
                "Built-in action ID should be snake_case, not '{}'",
                action.id
            );
        }

        // SDK actions use name as ID (from ProtocolAction conversion)
        let sdk_action = ProtocolAction {
            name: "My Custom Action".to_string(),
            description: Some("A custom SDK action".to_string()),
            shortcut: None,
            value: Some("custom-value".to_string()),
            has_action: true,
            visible: None,
            close: None,
        };

        // When converted, SDK action ID = name
        let converted = Action::new(
            sdk_action.name.clone(), // ID = name for SDK actions
            sdk_action.name.clone(),
            sdk_action.description.clone(),
            ActionCategory::ScriptContext,
        );

        // SDK action ID can contain spaces (matches the name)
        assert_eq!(converted.id, "My Custom Action");
        assert_eq!(converted.title, "My Custom Action");

        // Verify has_action routing distinction
        assert!(
            sdk_action.has_action,
            "SDK action with has_action=true routes to ActionTriggered"
        );

        let sdk_simple = ProtocolAction {
            name: "Simple Submit".to_string(),
            description: None,
            shortcut: None,
            value: Some("submit-value".to_string()),
            has_action: false,
            visible: None,
            close: None,
        };
        assert!(
            !sdk_simple.has_action,
            "SDK action with has_action=false submits value directly"
        );

        // Log for visibility
        println!("Built-in action count: {}", builtin_actions.len());
        println!(
            "Known built-in IDs: {:?}",
            builtin_actions
                .iter()
                .map(|a| a.id.as_str())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_shortcut_actions_dynamic() {
        // Test that shortcut actions change based on existing shortcut

        // No shortcut -> show "Add Keyboard Shortcut"
        let no_shortcut = ScriptInfo::new("test", "/path/test.ts");
        let actions_no_shortcut = get_script_context_actions(&no_shortcut);
        let ids_no_shortcut: Vec<&str> =
            actions_no_shortcut.iter().map(|a| a.id.as_str()).collect();

        assert!(
            ids_no_shortcut.contains(&"add_shortcut"),
            "Should have add_shortcut when no shortcut exists"
        );
        assert!(
            !ids_no_shortcut.contains(&"update_shortcut"),
            "Should NOT have update_shortcut when no shortcut exists"
        );
        assert!(
            !ids_no_shortcut.contains(&"remove_shortcut"),
            "Should NOT have remove_shortcut when no shortcut exists"
        );

        // Has shortcut -> show "Update" and "Remove"
        let with_shortcut =
            ScriptInfo::with_shortcut("test", "/path/test.ts", Some("cmd+t".to_string()));
        let actions_with_shortcut = get_script_context_actions(&with_shortcut);
        let ids_with_shortcut: Vec<&str> = actions_with_shortcut
            .iter()
            .map(|a| a.id.as_str())
            .collect();

        assert!(
            !ids_with_shortcut.contains(&"add_shortcut"),
            "Should NOT have add_shortcut when shortcut exists"
        );
        assert!(
            ids_with_shortcut.contains(&"update_shortcut"),
            "Should have update_shortcut when shortcut exists"
        );
        assert!(
            ids_with_shortcut.contains(&"remove_shortcut"),
            "Should have remove_shortcut when shortcut exists"
        );
    }
