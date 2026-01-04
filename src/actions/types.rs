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
}

impl ScriptInfo {
    pub fn new(name: impl Into<String>, path: impl Into<String>) -> Self {
        ScriptInfo {
            name: name.into(),
            path: path.into(),
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
    ScriptOps,     // Edit, Create, Delete script operations
    GlobalOps,     // Settings, Quit, etc.
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
