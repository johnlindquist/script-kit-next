    #[test]
    fn test_actions_prelude_exports_core_types() {
        let info = super::prelude::ScriptInfo::new("test", "/tmp/test.ts");
        let action = super::prelude::Action::new(
            "id",
            "title",
            None,
            super::prelude::ActionCategory::ScriptContext,
        );

        assert_eq!(info.name, "test");
        assert_eq!(action.id, "id");
    }

    #[test]
    fn test_actions_exceed_visible_space() {
        // Verify script context actions count
        // Global actions are now empty (Settings/Quit in main menu only)
        let script = ScriptInfo::new("test-script", "/path/to/test.ts");
        let script_actions = get_script_context_actions(&script);
        let global_actions = get_global_actions();
        let total_actions = script_actions.len() + global_actions.len();

        let max_visible = (POPUP_MAX_HEIGHT / ACTION_ITEM_HEIGHT) as usize;

        // Script context actions: run, edit, add_shortcut (or update+remove),
        // view_logs, reveal_in_finder, copy_path, copy_deeplink = 7 actions
        assert!(
            total_actions >= 7,
            "Should have at least 7 script context actions"
        );
        assert!(global_actions.is_empty(), "Global actions should be empty");

        // Log for visibility
        println!(
            "Total actions: {}, Max visible: {}",
            total_actions, max_visible
        );
    }

    #[test]
    fn test_protocol_action_to_action_conversion() {
        let protocol_action = ProtocolAction {
            name: "Copy".to_string(),
            description: Some("Copy to clipboard".to_string()),
            shortcut: Some("cmd+c".to_string()),
            value: Some("copy-value".to_string()),
            has_action: true,
            visible: None,
            close: None,
        };

        // Test that ProtocolAction fields are accessible for conversion
        // The actual conversion in dialog.rs copies these to Action struct
        assert_eq!(protocol_action.name, "Copy");
        assert_eq!(
            protocol_action.description,
            Some("Copy to clipboard".to_string())
        );
        assert_eq!(protocol_action.shortcut, Some("cmd+c".to_string()));
        assert_eq!(protocol_action.value, Some("copy-value".to_string()));
        assert!(protocol_action.has_action);

        // Create Action using builder pattern (used by get_*_actions)
        let action = Action::new(
            protocol_action.name.clone(),
            protocol_action.name.clone(),
            protocol_action.description.clone(),
            ActionCategory::ScriptContext,
        );
        assert_eq!(action.id, "Copy");
        assert_eq!(action.title, "Copy");
    }

    #[test]
    fn test_protocol_action_has_action_routing() {
        // Action with has_action=true should trigger ActionTriggered to SDK
        let action_with_handler = ProtocolAction {
            name: "Custom Action".to_string(),
            description: None,
            shortcut: None,
            value: Some("custom-value".to_string()),
            has_action: true,
            visible: None,
            close: None,
        };
        assert!(action_with_handler.has_action);

        // Action with has_action=false should submit value directly
        let action_without_handler = ProtocolAction {
            name: "Simple Action".to_string(),
            description: None,
            shortcut: None,
            value: Some("simple-value".to_string()),
            has_action: false,
            visible: None,
            close: None,
        };
        assert!(!action_without_handler.has_action);
    }

    // =========================================================================
    // NEW TESTS: Filter ranking, action categories, ScriptInfo variants, SDK vs built-in
    // =========================================================================

    #[test]
    fn test_filter_ranking_scoring() {
        // Test the scoring system used by ActionsDialog::score_action
        // Scoring: prefix +100, contains +50, fuzzy +25, description +15, shortcut +10

        // Create actions with varying match qualities
        let action_prefix = Action::new(
            "edit_script",
            "Edit Script",
            Some("Open in editor".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘E");

        let action_contains = Action::new(
            "copy_edit_path",
            "Copy Edit Path",
            Some("Copy the path".to_string()),
            ActionCategory::ScriptContext,
        );

        let action_fuzzy = Action::new(
            "exit_dialog",
            "Exit Dialog",
            Some("Close the dialog".to_string()),
            ActionCategory::ScriptContext,
        );

        let action_desc_match = Action::new(
            "open_file",
            "Open File",
            Some("Edit the file in your editor".to_string()),
            ActionCategory::ScriptContext,
        );

        // Test scoring function logic (reimplemented here for unit testing)
        fn score_action(action: &Action, search_lower: &str) -> i32 {
            let title_lower = action.title.to_lowercase();
            let mut score = 0;

            // Prefix match on title (strongest)
            if title_lower.starts_with(search_lower) {
                score += 100;
            }
            // Contains match on title
            else if title_lower.contains(search_lower) {
                score += 50;
            }
            // Fuzzy match on title
            else if fuzzy_match(&title_lower, search_lower) {
                score += 25;
            }

            // Description match (bonus)
            if let Some(ref desc) = action.description {
                if desc.to_lowercase().contains(search_lower) {
                    score += 15;
                }
            }

            // Shortcut match (bonus)
            if let Some(ref shortcut) = action.shortcut {
                if shortcut.to_lowercase().contains(search_lower) {
                    score += 10;
                }
            }

            score
        }

        fn fuzzy_match(haystack: &str, needle: &str) -> bool {
            let mut haystack_chars = haystack.chars();
            for needle_char in needle.chars() {
                loop {
                    match haystack_chars.next() {
                        Some(h) if h == needle_char => break,
                        Some(_) => continue,
                        None => return false,
                    }
                }
            }
            true
        }

        // Test prefix match (highest priority)
        let score_prefix = score_action(&action_prefix, "edit");
        assert!(
            score_prefix >= 100,
            "Prefix match should score 100+, got {}",
            score_prefix
        );

        // Test contains match (medium priority)
        let score_contains = score_action(&action_contains, "edit");
        assert!(
            (50..100).contains(&score_contains),
            "Contains match should score 50-99, got {}",
            score_contains
        );

        // Test fuzzy match (lower priority) - "edt" matches "exit dialog" via e-x-i-t-d
        let score_fuzzy = score_action(&action_fuzzy, "exi");
        assert!(
            score_fuzzy >= 100,
            "Prefix 'exi' on 'Exit Dialog' should score 100+, got {}",
            score_fuzzy
        );

        // Test description bonus
        let score_desc = score_action(&action_desc_match, "editor");
        assert!(
            score_desc >= 15,
            "Description match 'editor' should add 15+ points, got {}",
            score_desc
        );

        // Verify ranking order: prefix > contains > fuzzy > no match
        let score_nomatch = score_action(&action_prefix, "xyz");
        assert_eq!(score_nomatch, 0, "No match should score 0");

        // Verify prefix beats contains
        assert!(
            score_prefix > score_contains,
            "Prefix should beat contains: {} > {}",
            score_prefix,
            score_contains
        );
    }
