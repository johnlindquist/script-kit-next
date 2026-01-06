//! Action builders
//!
//! Factory functions for creating context-specific action lists.

use super::types::{Action, ActionCategory, ScriptInfo};
use crate::prompts::PathInfo;

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
fn to_deeplink_name(name: &str) -> String {
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

    // Run action - always available for both scripts and built-ins
    actions.push(
        Action::new(
            "run_script",
            format!("Run \"{}\"", script.name),
            Some("Execute this script".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("↵"),
    );

    // Script-only actions (not available for built-ins)
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
                "configure_shortcut",
                "Configure Keyboard Shortcut",
                Some("Set or change the script's hotkey".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⇧K"),
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

    actions
}

/// Predefined global actions
pub fn get_global_actions() -> Vec<Action> {
    vec![
        Action::new(
            "create_script",
            "Create New Script",
            Some("Create a new TypeScript script".to_string()),
            ActionCategory::ScriptOps,
        )
        .with_shortcut("⌘N"),
        Action::new(
            "reload_scripts",
            "Reload Scripts",
            Some("Refresh the scripts list".to_string()),
            ActionCategory::GlobalOps,
        )
        .with_shortcut("⌘R"),
        Action::new(
            "settings",
            "Settings",
            Some("Configure preferences".to_string()),
            ActionCategory::GlobalOps,
        )
        .with_shortcut("⌘,"),
        Action::new(
            "quit",
            "Quit Script Kit",
            Some("Exit the application".to_string()),
            ActionCategory::GlobalOps,
        )
        .with_shortcut("⌘Q"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_script_context_actions() {
        let script = ScriptInfo::new("my-script", "/path/to/my-script.ts");
        let actions = get_script_context_actions(&script);

        assert!(!actions.is_empty());
        // Script-specific actions should be present
        assert!(actions.iter().any(|a| a.id == "edit_script"));
        assert!(actions.iter().any(|a| a.id == "view_logs"));
        assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
        assert!(actions.iter().any(|a| a.id == "copy_path"));
        assert!(actions.iter().any(|a| a.id == "run_script"));
        // New actions
        assert!(actions.iter().any(|a| a.id == "configure_shortcut"));
        assert!(actions.iter().any(|a| a.id == "copy_deeplink"));
    }

    #[test]
    fn test_get_builtin_context_actions() {
        // Built-in commands should have limited actions
        let builtin = ScriptInfo::builtin("Clipboard History");
        let actions = get_script_context_actions(&builtin);

        // Should have run and copy_deeplink
        assert!(actions.iter().any(|a| a.id == "run_script"));
        assert!(actions.iter().any(|a| a.id == "copy_deeplink"));

        // Should NOT have script-only actions
        assert!(!actions.iter().any(|a| a.id == "edit_script"));
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
        assert!(!actions.iter().any(|a| a.id == "reveal_in_finder"));
        assert!(!actions.iter().any(|a| a.id == "copy_path"));
        assert!(!actions.iter().any(|a| a.id == "configure_shortcut"));
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

        assert!(!actions.is_empty());
        assert!(actions.iter().any(|a| a.id == "create_script"));
        assert!(actions.iter().any(|a| a.id == "reload_scripts"));
        assert!(actions.iter().any(|a| a.id == "settings"));
        assert!(actions.iter().any(|a| a.id == "quit"));
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
}
