//! Action builders
//!
//! Factory functions for creating context-specific action lists.

use super::types::{Action, ActionCategory, ScriptInfo};
use crate::file_search::FileInfo;
use crate::prompts::PathInfo;

/// Get actions specific to a file search result
/// Actions: Open (default), Show in Finder, Quick Look, Open With..., Show Info
pub fn get_file_context_actions(file_info: &FileInfo) -> Vec<Action> {
    let mut actions = Vec::new();

    // Primary action - Open file
    if file_info.is_dir {
        actions.push(
            Action::new(
                "open_directory",
                format!("Open \"{}\"", file_info.name),
                Some("Open this folder".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("↵"),
        );
    } else {
        actions.push(
            Action::new(
                "open_file",
                format!("Open \"{}\"", file_info.name),
                Some("Open with default application".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("↵"),
        );
    }

    // Show in Finder (Cmd+Enter)
    actions.push(
        Action::new(
            "reveal_in_finder",
            "Show in Finder",
            Some("Reveal in Finder".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘↵"),
    );

    // Quick Look (Cmd+Y) - macOS only
    #[cfg(target_os = "macos")]
    if !file_info.is_dir {
        actions.push(
            Action::new(
                "quick_look",
                "Quick Look",
                Some("Preview with Quick Look".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘Y"),
        );
    }

    // Open With... (Cmd+O) - macOS only
    #[cfg(target_os = "macos")]
    actions.push(
        Action::new(
            "open_with",
            "Open With...",
            Some("Choose application to open with".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘O"),
    );

    // Show Info in Finder (Cmd+I) - macOS only
    #[cfg(target_os = "macos")]
    actions.push(
        Action::new(
            "show_info",
            "Get Info",
            Some("Show file information in Finder".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘I"),
    );

    // Copy Path
    actions.push(
        Action::new(
            "copy_path",
            "Copy Path",
            Some("Copy the full path to clipboard".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⇧C"),
    );

    // Copy Filename
    actions.push(
        Action::new(
            "copy_filename",
            "Copy Filename",
            Some("Copy just the filename to clipboard".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘C"),
    );

    actions
}

/// Get actions specific to a file/folder path
pub fn get_path_context_actions(path_info: &PathInfo) -> Vec<Action> {
    let mut actions = vec![
        Action::new(
            "copy_path",
            "Copy Path",
            Some("Copy the full path to clipboard".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⇧C"),
        Action::new(
            "open_in_finder",
            "Open in Finder",
            Some("Reveal in Finder".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⇧F"),
        Action::new(
            "open_in_editor",
            "Open in Editor",
            Some("Open in $EDITOR".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘E"),
        Action::new(
            "open_in_terminal",
            "Open in Terminal",
            Some("Open terminal at this location".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘T"),
        Action::new(
            "copy_filename",
            "Copy Filename",
            Some("Copy just the filename".to_string()),
            ActionCategory::ScriptContext,
        ),
        Action::new(
            "move_to_trash",
            "Move to Trash",
            Some(format!(
                "Delete {}",
                if path_info.is_dir { "folder" } else { "file" }
            )),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⌫"),
    ];

    // Add directory-specific action for navigating into
    if path_info.is_dir {
        actions.insert(
            0,
            Action::new(
                "open_directory",
                format!("Open \"{}\"", path_info.name),
                Some("Navigate into this directory".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("↵"),
        );
    } else {
        actions.insert(
            0,
            Action::new(
                "select_file",
                format!("Select \"{}\"", path_info.name),
                Some("Submit this file".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("↵"),
        );
    }

    actions
}

/// Convert a script name to a deeplink-safe format (lowercase, hyphenated)
///
/// Examples:
/// - "My Script" → "my-script"
/// - "Clipboard History" → "clipboard-history"
/// - "hello_world" → "hello-world"
pub fn to_deeplink_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Get actions specific to the focused script
/// Actions are filtered based on whether this is a real script or a built-in command
pub fn get_script_context_actions(script: &ScriptInfo) -> Vec<Action> {
    let mut actions = Vec::new();

    // Primary action - always available for both scripts and built-ins
    // Uses the action_verb from ScriptInfo (e.g., "Run", "Launch", "Switch to")
    actions.push(
        Action::new(
            "run_script",
            format!("{} \"{}\"", script.action_verb, script.name),
            Some(format!("{} this item", script.action_verb)),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("↵"),
    );

    // Dynamic shortcut actions based on whether a shortcut already exists
    // If NO shortcut: Show "Add Keyboard Shortcut"
    // If HAS shortcut: Show "Update Keyboard Shortcut" and "Remove Keyboard Shortcut"
    if script.shortcut.is_some() {
        // Has existing shortcut - show Update and Remove options
        actions.push(
            Action::new(
                "update_shortcut",
                "Update Keyboard Shortcut",
                Some("Change the keyboard shortcut".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧K"),
        );
        actions.push(
            Action::new(
                "remove_shortcut",
                "Remove Keyboard Shortcut",
                Some("Remove the current keyboard shortcut".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⌥K"),
        );
    } else {
        // No shortcut - show Add option
        actions.push(
            Action::new(
                "add_shortcut",
                "Add Keyboard Shortcut",
                Some("Set a keyboard shortcut".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧K"),
        );
    }

    // Dynamic alias actions based on whether an alias already exists
    // If NO alias: Show "Add Alias"
    // If HAS alias: Show "Update Alias" and "Remove Alias"
    if script.alias.is_some() {
        // Has existing alias - show Update and Remove options
        actions.push(
            Action::new(
                "update_alias",
                "Update Alias",
                Some("Change the alias trigger".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧A"),
        );
        actions.push(
            Action::new(
                "remove_alias",
                "Remove Alias",
                Some("Remove the current alias".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⌥A"),
        );
    } else {
        // No alias - show Add option
        actions.push(
            Action::new(
                "add_alias",
                "Add Alias",
                Some("Set an alias trigger (type alias + space to run)".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧A"),
        );
    }

    // Script-only actions (not available for built-ins, apps, windows, scriptlets)
    if script.is_script {
        actions.push(
            Action::new(
                "edit_script",
                "Edit Script",
                Some("Open in $EDITOR".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘E"),
        );

        actions.push(
            Action::new(
                "view_logs",
                "View Logs",
                Some("Show script execution logs".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘L"),
        );

        actions.push(
            Action::new(
                "reveal_in_finder",
                "Reveal in Finder",
                Some("Show script file in Finder".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧F"),
        );

        actions.push(
            Action::new(
                "copy_path",
                "Copy Path",
                Some("Copy script path to clipboard".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧C"),
        );
    }

    // Scriptlet-specific actions (work with the markdown file containing the scriptlet)
    if script.is_scriptlet {
        actions.push(
            Action::new(
                "edit_scriptlet",
                "Edit Scriptlet",
                Some("Open the markdown file in $EDITOR".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘E"),
        );

        actions.push(
            Action::new(
                "reveal_scriptlet_in_finder",
                "Reveal in Finder",
                Some("Show scriptlet bundle in Finder".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧F"),
        );

        actions.push(
            Action::new(
                "copy_scriptlet_path",
                "Copy Path",
                Some("Copy scriptlet bundle path to clipboard".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧C"),
        );
    }

    // Copy deeplink - available for both scripts and built-ins
    let deeplink_name = to_deeplink_name(&script.name);
    actions.push(
        Action::new(
            "copy_deeplink",
            "Copy Deeplink",
            Some(format!(
                "Copy scriptkit://run/{} URL to clipboard",
                deeplink_name
            )),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⇧D"),
    );

    // Reset Ranking - only available for items that are suggested (have frecency data)
    if script.is_suggested {
        actions.push(Action::new(
            "reset_ranking",
            "Reset Ranking",
            Some("Remove this item from Suggested section".to_string()),
            ActionCategory::ScriptContext,
        ));
    }

    actions
}

/// Predefined global actions
/// Note: Settings and Quit are available from the main menu, not shown in actions dialog
pub fn get_global_actions() -> Vec<Action> {
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_script_context_actions_no_shortcut() {
        // Script without shortcut should show "Add Keyboard Shortcut"
        let script = ScriptInfo::new("my-script", "/path/to/my-script.ts");
        let actions = get_script_context_actions(&script);

        assert!(!actions.is_empty());
        // Script-specific actions should be present
        assert!(actions.iter().any(|a| a.id == "edit_script"));
        assert!(actions.iter().any(|a| a.id == "view_logs"));
        assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
        assert!(actions.iter().any(|a| a.id == "copy_path"));
        assert!(actions.iter().any(|a| a.id == "run_script"));
        // Dynamic shortcut action - no shortcut means "Add"
        assert!(actions.iter().any(|a| a.id == "add_shortcut"));
        assert!(!actions.iter().any(|a| a.id == "update_shortcut"));
        assert!(!actions.iter().any(|a| a.id == "remove_shortcut"));
        // Dynamic alias action - no alias means "Add"
        assert!(actions.iter().any(|a| a.id == "add_alias"));
        assert!(!actions.iter().any(|a| a.id == "update_alias"));
        assert!(!actions.iter().any(|a| a.id == "remove_alias"));
        assert!(actions.iter().any(|a| a.id == "copy_deeplink"));
    }

    #[test]
    fn test_get_script_context_actions_with_shortcut() {
        // Script with shortcut should show "Update" and "Remove" options
        let script = ScriptInfo::with_shortcut(
            "my-script",
            "/path/to/my-script.ts",
            Some("cmd+shift+m".to_string()),
        );
        let actions = get_script_context_actions(&script);

        // Dynamic shortcut actions - has shortcut means "Update" and "Remove"
        assert!(!actions.iter().any(|a| a.id == "add_shortcut"));
        assert!(actions.iter().any(|a| a.id == "update_shortcut"));
        assert!(actions.iter().any(|a| a.id == "remove_shortcut"));
    }

    #[test]
    fn test_get_builtin_context_actions() {
        // Built-in commands should have limited actions
        let builtin = ScriptInfo::builtin("Clipboard History");
        let actions = get_script_context_actions(&builtin);

        // Should have run, copy_deeplink, add_shortcut, and add_alias (no shortcut/alias by default)
        assert!(actions.iter().any(|a| a.id == "run_script"));
        assert!(actions.iter().any(|a| a.id == "copy_deeplink"));
        assert!(actions.iter().any(|a| a.id == "add_shortcut"));
        assert!(actions.iter().any(|a| a.id == "add_alias"));

        // Should NOT have script-only actions
        assert!(!actions.iter().any(|a| a.id == "edit_script"));
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
        assert!(!actions.iter().any(|a| a.id == "reveal_in_finder"));
        assert!(!actions.iter().any(|a| a.id == "copy_path"));
    }

    #[test]
    fn test_get_scriptlet_context_actions() {
        // Scriptlets should have scriptlet-specific actions
        let scriptlet = ScriptInfo::scriptlet("Open GitHub", "/path/to/url.md", None, None);
        let actions = get_script_context_actions(&scriptlet);

        // Should have run, copy_deeplink, and add_shortcut (no shortcut by default)
        assert!(actions.iter().any(|a| a.id == "run_script"));
        assert!(actions.iter().any(|a| a.id == "copy_deeplink"));
        assert!(actions.iter().any(|a| a.id == "add_shortcut"));

        // Should have scriptlet-specific actions
        assert!(actions.iter().any(|a| a.id == "edit_scriptlet"));
        assert!(actions.iter().any(|a| a.id == "reveal_scriptlet_in_finder"));
        assert!(actions.iter().any(|a| a.id == "copy_scriptlet_path"));

        // Verify edit_scriptlet has correct title
        let edit_action = actions.iter().find(|a| a.id == "edit_scriptlet").unwrap();
        assert_eq!(edit_action.title, "Edit Scriptlet");

        // Should NOT have script-only actions
        assert!(!actions.iter().any(|a| a.id == "edit_script"));
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
        assert!(!actions.iter().any(|a| a.id == "reveal_in_finder"));
        assert!(!actions.iter().any(|a| a.id == "copy_path"));
    }

    #[test]
    fn test_get_script_context_actions_with_alias() {
        // Script with alias should show "Update Alias" and "Remove Alias" options
        let script = ScriptInfo::with_shortcut_and_alias(
            "my-script",
            "/path/to/my-script.ts",
            None,
            Some("ms".to_string()),
        );
        let actions = get_script_context_actions(&script);

        // Dynamic alias actions - has alias means "Update" and "Remove"
        assert!(!actions.iter().any(|a| a.id == "add_alias"));
        assert!(actions.iter().any(|a| a.id == "update_alias"));
        assert!(actions.iter().any(|a| a.id == "remove_alias"));
    }

    #[test]
    fn test_get_builtin_context_actions_with_alias() {
        // Built-in with alias should show "Update Alias" and "Remove Alias"
        let builtin = ScriptInfo::with_all(
            "Clipboard History",
            "builtin:clipboard-history",
            false,
            "Open",
            None,
            Some("ch".to_string()),
        );
        let actions = get_script_context_actions(&builtin);

        // Should have alias actions for update/remove
        assert!(!actions.iter().any(|a| a.id == "add_alias"));
        assert!(actions.iter().any(|a| a.id == "update_alias"));
        assert!(actions.iter().any(|a| a.id == "remove_alias"));
    }

    #[test]
    fn test_to_deeplink_name() {
        // Test the deeplink name conversion
        assert_eq!(to_deeplink_name("My Script"), "my-script");
        assert_eq!(to_deeplink_name("Clipboard History"), "clipboard-history");
        assert_eq!(to_deeplink_name("hello_world"), "hello-world");
        assert_eq!(
            to_deeplink_name("Test  Multiple   Spaces"),
            "test-multiple-spaces"
        );
        assert_eq!(to_deeplink_name("special!@#chars"), "special-chars");
    }

    #[test]
    fn test_get_global_actions() {
        let actions = get_global_actions();
        // Global actions are now empty - Settings/Quit available from main menu
        assert!(actions.is_empty());
    }

    #[test]
    fn test_built_in_actions_have_no_has_action() {
        // All built-in actions should have has_action=false
        let script = ScriptInfo::new("test-script", "/path/to/test.ts");
        let script_actions = get_script_context_actions(&script);
        let global_actions = get_global_actions();

        for action in script_actions.iter() {
            assert!(
                !action.has_action,
                "Built-in action '{}' should have has_action=false",
                action.id
            );
        }

        for action in global_actions.iter() {
            assert!(
                !action.has_action,
                "Built-in action '{}' should have has_action=false",
                action.id
            );
        }
    }

    #[test]
    fn test_copy_deeplink_description_format() {
        // Verify the deeplink description shows the correct URL format
        let script = ScriptInfo::new("My Cool Script", "/path/to/script.ts");
        let actions = get_script_context_actions(&script);

        let deeplink_action = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        assert!(deeplink_action
            .description
            .as_ref()
            .unwrap()
            .contains("scriptkit://run/my-cool-script"));
    }

    #[test]
    fn test_get_file_context_actions_file() {
        // Test file actions for a regular file
        let file_info = FileInfo {
            path: "/Users/test/document.pdf".to_string(),
            name: "document.pdf".to_string(),
            file_type: crate::file_search::FileType::Document,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);

        // Should have open_file as primary action
        assert!(actions.iter().any(|a| a.id == "open_file"));
        assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
        assert!(actions.iter().any(|a| a.id == "copy_path"));
        assert!(actions.iter().any(|a| a.id == "copy_filename"));

        // Should NOT have open_directory (not a directory)
        assert!(!actions.iter().any(|a| a.id == "open_directory"));

        // On macOS, should have Quick Look, Open With, Get Info
        #[cfg(target_os = "macos")]
        {
            assert!(actions.iter().any(|a| a.id == "quick_look"));
            assert!(actions.iter().any(|a| a.id == "open_with"));
            assert!(actions.iter().any(|a| a.id == "show_info"));
        }
    }

    #[test]
    fn test_get_file_context_actions_directory() {
        // Test file actions for a directory
        let file_info = FileInfo {
            path: "/Users/test/Documents".to_string(),
            name: "Documents".to_string(),
            file_type: crate::file_search::FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&file_info);

        // Should have open_directory as primary action
        assert!(actions.iter().any(|a| a.id == "open_directory"));
        assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
        assert!(actions.iter().any(|a| a.id == "copy_path"));
        assert!(actions.iter().any(|a| a.id == "copy_filename"));

        // Should NOT have open_file (it's a directory)
        assert!(!actions.iter().any(|a| a.id == "open_file"));

        // Directory should NOT have quick_look (only files)
        #[cfg(target_os = "macos")]
        {
            assert!(!actions.iter().any(|a| a.id == "quick_look"));
            // But should have Open With and Get Info
            assert!(actions.iter().any(|a| a.id == "open_with"));
            assert!(actions.iter().any(|a| a.id == "show_info"));
        }
    }

    #[test]
    fn test_file_context_actions_shortcuts() {
        // Verify the keyboard shortcuts are correct
        let file_info = FileInfo {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            file_type: crate::file_search::FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&file_info);

        // Check specific shortcuts
        let open_action = actions.iter().find(|a| a.id == "open_file").unwrap();
        assert_eq!(open_action.shortcut.as_ref().unwrap(), "↵");

        let reveal_action = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
        assert_eq!(reveal_action.shortcut.as_ref().unwrap(), "⌘↵");

        #[cfg(target_os = "macos")]
        {
            let quick_look_action = actions.iter().find(|a| a.id == "quick_look").unwrap();
            assert_eq!(quick_look_action.shortcut.as_ref().unwrap(), "⌘Y");

            let show_info_action = actions.iter().find(|a| a.id == "show_info").unwrap();
            assert_eq!(show_info_action.shortcut.as_ref().unwrap(), "⌘I");
        }
    }

    #[test]
    fn test_reset_ranking_not_shown_when_not_suggested() {
        // Script without is_suggested should NOT show "Reset Ranking" action
        let script = ScriptInfo::new("test-script", "/path/to/test-script.ts");
        assert!(!script.is_suggested);

        let actions = get_script_context_actions(&script);

        // Should NOT have reset_ranking action
        assert!(
            !actions.iter().any(|a| a.id == "reset_ranking"),
            "reset_ranking should not be shown when is_suggested is false"
        );
    }

    #[test]
    fn test_reset_ranking_shown_when_suggested() {
        // Script with is_suggested should show "Reset Ranking" action
        let script = ScriptInfo::new("test-script", "/path/to/test-script.ts")
            .with_frecency(true, Some("/path/to/test-script.ts".to_string()));
        assert!(script.is_suggested);

        let actions = get_script_context_actions(&script);

        // Should have reset_ranking action
        assert!(
            actions.iter().any(|a| a.id == "reset_ranking"),
            "reset_ranking should be shown when is_suggested is true"
        );

        // Verify action details
        let reset_action = actions.iter().find(|a| a.id == "reset_ranking").unwrap();
        assert_eq!(reset_action.title, "Reset Ranking");
        assert_eq!(
            reset_action.description,
            Some("Remove this item from Suggested section".to_string())
        );
    }

    #[test]
    fn test_with_frecency_builder() {
        // Test the with_frecency builder method
        let script = ScriptInfo::new("test", "/path/to/test.ts")
            .with_frecency(true, Some("frecency:path".to_string()));

        assert!(script.is_suggested);
        assert_eq!(script.frecency_path, Some("frecency:path".to_string()));
    }

    #[test]
    fn test_reset_ranking_for_scriptlet() {
        // Scriptlet with is_suggested should show "Reset Ranking" action
        let scriptlet = ScriptInfo::scriptlet("Open GitHub", "/path/to/url.md", None, None)
            .with_frecency(true, Some("scriptlet:Open GitHub".to_string()));

        let actions = get_script_context_actions(&scriptlet);

        // Should have reset_ranking action for suggested scriptlet
        assert!(
            actions.iter().any(|a| a.id == "reset_ranking"),
            "reset_ranking should be shown for suggested scriptlets"
        );
    }

    #[test]
    fn test_reset_ranking_for_builtin() {
        // Built-in with is_suggested should show "Reset Ranking" action
        let builtin = ScriptInfo::builtin("Clipboard History")
            .with_frecency(true, Some("builtin:Clipboard History".to_string()));

        let actions = get_script_context_actions(&builtin);

        // Should have reset_ranking action for suggested built-in
        assert!(
            actions.iter().any(|a| a.id == "reset_ranking"),
            "reset_ranking should be shown for suggested built-ins"
        );
    }

    #[test]
    fn test_reset_ranking_for_app() {
        // App with is_suggested should show "Reset Ranking" action
        let app =
            ScriptInfo::with_action_verb("Safari", "/Applications/Safari.app", false, "Launch")
                .with_frecency(true, Some("/Applications/Safari.app".to_string()));

        let actions = get_script_context_actions(&app);

        // Should have reset_ranking action for suggested app
        assert!(
            actions.iter().any(|a| a.id == "reset_ranking"),
            "reset_ranking should be shown for suggested apps"
        );
    }

    #[test]
    fn test_reset_ranking_for_window() {
        // Window with is_suggested should show "Reset Ranking" action
        let window = ScriptInfo::with_action_verb("My Document", "window:123", false, "Switch to")
            .with_frecency(true, Some("window:Preview:My Document".to_string()));

        let actions = get_script_context_actions(&window);

        // Should have reset_ranking action for suggested window
        assert!(
            actions.iter().any(|a| a.id == "reset_ranking"),
            "reset_ranking should be shown for suggested windows"
        );
    }

    #[test]
    fn test_reset_ranking_for_agent() {
        // Agent with is_suggested should show "Reset Ranking" action
        let agent = ScriptInfo::new("My Agent", "agent:/path/to/agent")
            .with_frecency(true, Some("agent:/path/to/agent".to_string()));

        let actions = get_script_context_actions(&agent);

        // Should have reset_ranking action for suggested agent
        assert!(
            actions.iter().any(|a| a.id == "reset_ranking"),
            "reset_ranking should be shown for suggested agents"
        );
    }

    #[test]
    fn test_reset_ranking_frecency_path_preserved() {
        // Verify that the frecency_path is correctly preserved through the builder
        let script = ScriptInfo::new("test", "/path/to/test.ts")
            .with_frecency(true, Some("/path/to/test.ts".to_string()));

        // Frecency path should be exactly what we set
        assert_eq!(script.frecency_path, Some("/path/to/test.ts".to_string()));
        assert!(script.is_suggested);
    }
}
