#[allow(unused_imports)]
use super::builders::*;
#[allow(unused_imports)]
use super::constants::*;
#[allow(unused_imports)]
use super::types::*;
#[allow(unused_imports)]
use crate::protocol::ProtocolAction;

// --- merged from core_part_01.rs ---
mod core_part_01 {
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
}

// --- merged from core_part_02.rs ---
mod core_part_02 {
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
}

// --- merged from core_part_03.rs ---
mod core_part_03 {
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
}

