//! Action types and data structures
//!
//! Core types for the actions system including Action, ActionCategory, and ScriptInfo.

use std::sync::Arc;

/// Callback for action selection
/// Signature: (action_id: String)
pub type ActionCallback = Arc<dyn Fn(String) + Send + Sync>;

/// Information about the currently focused/selected script
/// Used for context-aware actions in the actions dialog
#[derive(Debug, Clone)]
pub struct ScriptInfo {
    /// Display name of the script
    pub name: String,
    // Note: path is written during construction for completeness but currently
    // action handlers read directly from ProtocolAction. Kept for API consistency.
    #[allow(dead_code)]
    /// Full path to the script file
    pub path: String,
    /// Whether this is a real script file (true) or a built-in command (false)
    /// Built-in commands (like Clipboard History, App Launcher) have limited actions
    pub is_script: bool,
    /// Whether this is a scriptlet (snippet from markdown file)
    /// Scriptlets have their own actions (Edit Scriptlet, etc.) that work with the markdown file
    pub is_scriptlet: bool,
    /// The verb to use for the primary action (e.g., "Run", "Launch", "Switch to")
    /// Defaults to "Run" for scripts
    pub action_verb: String,
    /// Current keyboard shortcut assigned to this script/item (if any)
    /// Used to determine which shortcut actions to show in the actions menu
    pub shortcut: Option<String>,
}

impl ScriptInfo {
    /// Create a ScriptInfo for a real script file
    pub fn new(name: impl Into<String>, path: impl Into<String>) -> Self {
        ScriptInfo {
            name: name.into(),
            path: path.into(),
            is_script: true,
            is_scriptlet: false,
            action_verb: "Run".to_string(),
            shortcut: None,
        }
    }

    /// Create a ScriptInfo for a real script file with shortcut info
    #[allow(dead_code)]
    pub fn with_shortcut(
        name: impl Into<String>,
        path: impl Into<String>,
        shortcut: Option<String>,
    ) -> Self {
        ScriptInfo {
            name: name.into(),
            path: path.into(),
            is_script: true,
            is_scriptlet: false,
            action_verb: "Run".to_string(),
            shortcut,
        }
    }

    /// Create a ScriptInfo for a scriptlet (snippet from markdown file)
    /// Scriptlets have their own actions that work with the source markdown file
    pub fn scriptlet(
        name: impl Into<String>,
        markdown_path: impl Into<String>,
        shortcut: Option<String>,
    ) -> Self {
        ScriptInfo {
            name: name.into(),
            path: markdown_path.into(),
            is_script: false,
            is_scriptlet: true,
            action_verb: "Run".to_string(),
            shortcut,
        }
    }

    /// Create a ScriptInfo for a built-in command (not a real script)
    /// Built-ins have limited actions (no edit, view logs, reveal in finder, copy path, configure shortcut)
    #[allow(dead_code)]
    pub fn builtin(name: impl Into<String>) -> Self {
        ScriptInfo {
            name: name.into(),
            path: String::new(),
            is_script: false,
            is_scriptlet: false,
            action_verb: "Run".to_string(),
            shortcut: None,
        }
    }

    /// Create a ScriptInfo with explicit is_script flag and custom action verb
    #[allow(dead_code)]
    pub fn with_is_script(
        name: impl Into<String>,
        path: impl Into<String>,
        is_script: bool,
    ) -> Self {
        ScriptInfo {
            name: name.into(),
            path: path.into(),
            is_script,
            is_scriptlet: false,
            action_verb: "Run".to_string(),
            shortcut: None,
        }
    }

    /// Create a ScriptInfo with all options including custom action verb
    pub fn with_action_verb(
        name: impl Into<String>,
        path: impl Into<String>,
        is_script: bool,
        action_verb: impl Into<String>,
    ) -> Self {
        ScriptInfo {
            name: name.into(),
            path: path.into(),
            is_script,
            is_scriptlet: false,
            action_verb: action_verb.into(),
            shortcut: None,
        }
    }

    /// Create a ScriptInfo with all options including custom action verb and shortcut
    #[allow(dead_code)]
    pub fn with_action_verb_and_shortcut(
        name: impl Into<String>,
        path: impl Into<String>,
        is_script: bool,
        action_verb: impl Into<String>,
        shortcut: Option<String>,
    ) -> Self {
        ScriptInfo {
            name: name.into(),
            path: path.into(),
            is_script,
            is_scriptlet: false,
            action_verb: action_verb.into(),
            shortcut,
        }
    }
}

/// Available actions in the actions menu
///
/// Note: The `has_action` and `value` fields are populated from ProtocolAction
/// for consistency, but the actual routing logic reads from the original
/// ProtocolAction. These fields are kept for future use cases where Action
/// might need independent behavior.
#[derive(Debug, Clone)]
pub struct Action {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub category: ActionCategory,
    /// Optional keyboard shortcut hint (e.g., "⌘E")
    pub shortcut: Option<String>,
    /// If true, send ActionTriggered to SDK; if false, submit value directly
    #[allow(dead_code)]
    pub has_action: bool,
    /// Optional value to submit when action is triggered
    #[allow(dead_code)]
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionCategory {
    ScriptContext, // Actions specific to the focused script
    #[allow(dead_code)]
    ScriptOps, // Edit, Create, Delete script operations (reserved for future use)
    #[allow(dead_code)]
    GlobalOps, // Settings, Quit, etc.
}

impl Action {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: Option<String>,
        category: ActionCategory,
    ) -> Self {
        Action {
            id: id.into(),
            title: title.into(),
            description,
            category,
            shortcut: None,
            has_action: false,
            value: None,
        }
    }

    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_info_creation() {
        let script = ScriptInfo::new("test-script", "/path/to/test-script.ts");
        assert_eq!(script.name, "test-script");
        assert_eq!(script.path, "/path/to/test-script.ts");
        assert!(script.is_script);
        assert!(!script.is_scriptlet);
        assert!(script.shortcut.is_none());
    }

    #[test]
    fn test_script_info_with_shortcut() {
        let script = ScriptInfo::with_shortcut(
            "test-script",
            "/path/to/test-script.ts",
            Some("cmd+shift+t".to_string()),
        );
        assert_eq!(script.name, "test-script");
        assert!(script.is_script);
        assert!(!script.is_scriptlet);
        assert_eq!(script.shortcut, Some("cmd+shift+t".to_string()));
    }

    #[test]
    fn test_script_info_scriptlet() {
        let scriptlet = ScriptInfo::scriptlet(
            "Open GitHub",
            "/path/to/url.md#open-github",
            Some("cmd+g".to_string()),
        );
        assert_eq!(scriptlet.name, "Open GitHub");
        assert_eq!(scriptlet.path, "/path/to/url.md#open-github");
        assert!(!scriptlet.is_script);
        assert!(scriptlet.is_scriptlet);
        assert_eq!(scriptlet.shortcut, Some("cmd+g".to_string()));
        assert_eq!(scriptlet.action_verb, "Run");
    }

    #[test]
    fn test_script_info_builtin() {
        let builtin = ScriptInfo::builtin("Clipboard History");
        assert_eq!(builtin.name, "Clipboard History");
        assert_eq!(builtin.path, "");
        assert!(!builtin.is_script);
        assert!(!builtin.is_scriptlet);
        assert!(builtin.shortcut.is_none());
    }

    #[test]
    fn test_script_info_with_is_script() {
        let script = ScriptInfo::with_is_script("my-script", "/path/to/script.ts", true);
        assert!(script.is_script);
        assert!(!script.is_scriptlet);
        assert!(script.shortcut.is_none());

        let builtin = ScriptInfo::with_is_script("App Launcher", "", false);
        assert!(!builtin.is_script);
        assert!(!builtin.is_scriptlet);
    }

    #[test]
    fn test_script_info_with_action_verb_and_shortcut() {
        let script = ScriptInfo::with_action_verb_and_shortcut(
            "test",
            "/path",
            true,
            "Launch",
            Some("cmd+k".to_string()),
        );
        assert_eq!(script.action_verb, "Launch");
        assert!(!script.is_scriptlet);
        assert_eq!(script.shortcut, Some("cmd+k".to_string()));
    }

    #[test]
    fn test_action_with_shortcut() {
        let action =
            Action::new("test", "Test Action", None, ActionCategory::GlobalOps).with_shortcut("⌘T");
        assert_eq!(action.shortcut, Some("⌘T".to_string()));
    }

    #[test]
    fn test_action_new_defaults() {
        let action = Action::new(
            "id",
            "title",
            Some("desc".to_string()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(action.id, "id");
        assert_eq!(action.title, "title");
        assert_eq!(action.description, Some("desc".to_string()));
        assert_eq!(action.category, ActionCategory::ScriptContext);
        assert!(action.shortcut.is_none());
    }
}
