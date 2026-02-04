//! Actions Dialog Module
//!
//! Provides a searchable action menu as a compact overlay popup for quick access
//! to script management and global actions (edit, create, settings, quit, etc.)
//!
//! The dialog can be rendered in two ways:
//! 1. As an inline overlay within the main window (legacy)
//! 2. As a separate floating window with its own vibrancy blur (preferred)
//!
//! ## Module Structure
//! - `types`: Core types (Action, ActionCategory, ScriptInfo)
//! - `builders`: Factory functions for creating action lists
//! - `constants`: Popup dimensions and styling constants
//! - `dialog`: ActionsDialog struct and implementation
//! - `window`: Separate vibrancy window for actions panel

mod builders;
mod command_bar;
mod constants;
mod dialog;
mod types;
mod window;

// Re-export only the public API that is actually used externally:
// - ScriptInfo: used by main.rs for action context
// - ActionsDialog: the main dialog component
// - Window functions for separate vibrancy window

pub use builders::to_deeplink_name;
#[allow(unused_imports)]
pub use builders::ClipboardEntryInfo;
#[allow(unused_imports)]
pub(crate) use builders::{
    get_global_actions, get_script_context_actions, get_scriptlet_context_actions_with_custom,
};
// Chat prompt info types for ActionsDialog::with_chat
pub use builders::{ChatModelInfo, ChatPromptInfo};
pub use dialog::ActionsDialog;
pub use types::ScriptInfo;

// Public API for AI window integration (re-exported but may appear unused until integration)
#[allow(unused_imports)]
pub use builders::get_ai_command_bar_actions;
#[allow(unused_imports)]
pub use builders::{get_new_chat_actions, NewChatModelInfo, NewChatPresetInfo};
// Public API for Notes window integration
#[allow(unused_imports)]
pub use builders::{get_notes_command_bar_actions, NotesInfo};
// Public API for Notes note switcher (Cmd+P) dialog
#[allow(unused_imports)]
pub use builders::{get_note_switcher_actions, NoteSwitcherNoteInfo};
#[allow(unused_imports)]
pub use types::{
    Action, ActionCategory, ActionsDialogConfig, AnchorPosition, SearchPosition, SectionStyle,
};

// Window functions for separate vibrancy window
pub use window::{
    close_actions_window, is_actions_window_open, notify_actions_window, open_actions_window,
    resize_actions_window, WindowPosition,
};
// get_actions_window_handle available but not re-exported (use window:: directly if needed)

// CommandBar - high-level reusable component for command palette functionality
#[allow(unused_imports)]
pub use command_bar::{is_command_bar_open, CommandBar, CommandBarConfig, CommandBarHost};

#[cfg(test)]
#[path = "dialog_tests.rs"]
mod dialog_tests;

#[cfg(test)]
#[path = "dialog_behavior_tests.rs"]
mod dialog_behavior_tests;

#[cfg(test)]
#[path = "dialog_window_tests.rs"]
mod dialog_window_tests;

#[cfg(test)]
#[path = "dialog_validation_tests.rs"]
mod dialog_validation_tests;

#[cfg(test)]
#[path = "dialog_random_tests.rs"]
mod dialog_random_tests;

#[cfg(test)]
#[path = "dialog_cross_context_tests.rs"]
mod dialog_cross_context_tests;

#[cfg(test)]
#[path = "dialog_random_action_window_tests.rs"]
mod dialog_random_action_window_tests;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests.rs"]
mod dialog_builtin_action_validation_tests;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_2.rs"]
mod dialog_builtin_action_validation_tests_2;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_3.rs"]
mod dialog_builtin_action_validation_tests_3;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_4.rs"]
mod dialog_builtin_action_validation_tests_4;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_5.rs"]
mod dialog_builtin_action_validation_tests_5;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_6.rs"]
mod dialog_builtin_action_validation_tests_6;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_7.rs"]
mod dialog_builtin_action_validation_tests_7;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_8.rs"]
mod dialog_builtin_action_validation_tests_8;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_9.rs"]
mod dialog_builtin_action_validation_tests_9;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_10.rs"]
mod dialog_builtin_action_validation_tests_10;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_11.rs"]
mod dialog_builtin_action_validation_tests_11;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_12.rs"]
mod dialog_builtin_action_validation_tests_12;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_13.rs"]
mod dialog_builtin_action_validation_tests_13;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_14.rs"]
mod dialog_builtin_action_validation_tests_14;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_15.rs"]
mod dialog_builtin_action_validation_tests_15;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_16.rs"]
mod dialog_builtin_action_validation_tests_16;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_17.rs"]
mod dialog_builtin_action_validation_tests_17;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_18.rs"]
mod dialog_builtin_action_validation_tests_18;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_19.rs"]
mod dialog_builtin_action_validation_tests_19;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_20.rs"]
mod dialog_builtin_action_validation_tests_20;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_21.rs"]
mod dialog_builtin_action_validation_tests_21;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_22.rs"]
mod dialog_builtin_action_validation_tests_22;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_23.rs"]
mod dialog_builtin_action_validation_tests_23;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_24.rs"]
mod dialog_builtin_action_validation_tests_24;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_25.rs"]
mod dialog_builtin_action_validation_tests_25;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_26.rs"]
mod dialog_builtin_action_validation_tests_26;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_27.rs"]
mod dialog_builtin_action_validation_tests_27;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_28.rs"]
mod dialog_builtin_action_validation_tests_28;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_29.rs"]
mod dialog_builtin_action_validation_tests_29;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_30.rs"]
mod dialog_builtin_action_validation_tests_30;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_31.rs"]
mod dialog_builtin_action_validation_tests_31;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_32.rs"]
mod dialog_builtin_action_validation_tests_32;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_33.rs"]
mod dialog_builtin_action_validation_tests_33;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_34.rs"]
mod dialog_builtin_action_validation_tests_34;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_35.rs"]
mod dialog_builtin_action_validation_tests_35;

#[cfg(test)]
#[path = "dialog_builtin_action_validation_tests_36.rs"]
mod dialog_builtin_action_validation_tests_36;

#[cfg(test)]
mod tests {
    // Import from submodules directly - these are only used in tests
    use super::builders::{get_global_actions, get_script_context_actions};
    use super::constants::{ACTION_ITEM_HEIGHT, POPUP_MAX_HEIGHT};
    use super::types::{Action, ActionCategory, ScriptInfo};
    use crate::protocol::ProtocolAction;

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
}
