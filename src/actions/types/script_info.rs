// Action types and data structures
//
// Core types for the actions system including Action, ActionCategory, and ScriptInfo.
//
// # Architecture Overview
//
// The actions system is **intentionally decoupled** from the standard selection callbacks:
//
// - **on_select callback is bypassed by design** for keyboard navigation
// - Actions route through `handle_action()` in the main app via keyboard events
// - This enables consistent keyboard-driven action execution across all contexts
//
// ## Key Types
//
// - [`Action`]: Represents a single action item with id, title, category, and optional shortcut
// - [`ActionCategory`]: Categorizes actions (ScriptContext, ScriptOps, GlobalOps)
// - [`ScriptInfo`]: Context about the focused script/item for building context-specific actions
//
// ## Action ID Conventions
//
// All built-in action IDs use **snake_case** format:
// - `run_script`, `edit_script`, `copy_path`, `reveal_in_finder`
// - `add_shortcut`, `update_shortcut`, `remove_shortcut`
// - `add_alias`, `update_alias`, `remove_alias`
// - `copy_deeplink`, `reset_ranking`
//
// SDK-provided actions (from ProtocolAction) use their `name` field as-is for the ID.
//
// ## has_action Field
//
// The `has_action` field determines routing:
// - `has_action=true`: Send ActionTriggered event to SDK, let SDK handle the action
// - `has_action=false`: Submit value directly via protocol (built-in actions)
//
// Built-in actions (from `builders.rs`) have `has_action=false` by default.
// SDK actions with handlers should set `has_action=true`.

use crate::designs::icon_variations::IconName;
use std::sync::Arc;

/// Callback for action selection
/// Signature: (action_id: String)
pub type ActionCallback = Arc<dyn Fn(String) + Send + Sync>;

/// Callback for dialog close (escape pressed, window dismissed)
/// Used to notify the main app to restore focus
/// Takes &mut App so the callback can update the main app entity
pub type CloseCallback = Arc<dyn Fn(&mut gpui::App) + Send + Sync>;

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
    /// Current alias assigned to this script/item (if any)
    /// Used to determine which alias actions to show in the actions menu
    pub alias: Option<String>,
    /// Whether this item appears in the "Suggested" section (has frecency data)
    /// Used to show/hide the "Reset Ranking" action
    pub is_suggested: bool,
    /// The frecency path used to track this item's usage
    /// Used by "Reset Ranking" to know which frecency entry to remove.
    /// When `is_suggested` is true, this should be a non-empty path.
    pub frecency_path: Option<String>,
    /// Whether this is an agent file (.claude.md or similar)
    /// Agents have their own actions (Edit Agent, Copy Content, etc.)
    pub is_agent: bool,
}

impl Default for ScriptInfo {
    fn default() -> Self {
        Self {
            name: String::new(),
            path: String::new(),
            is_script: false,
            is_scriptlet: false,
            action_verb: ScriptInfo::DEFAULT_ACTION_VERB.to_string(),
            shortcut: None,
            alias: None,
            is_suggested: false,
            frecency_path: None,
            is_agent: false,
        }
    }
}

impl ScriptInfo {
    const DEFAULT_ACTION_VERB: &'static str = "Run";

    fn normalized_optional_text(value: Option<String>) -> Option<String> {
        value.and_then(|text| {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
    }

    fn normalized_action_verb(action_verb: impl Into<String>) -> String {
        let action_verb = action_verb.into();
        let trimmed = action_verb.trim();
        if trimmed.is_empty() {
            Self::DEFAULT_ACTION_VERB.to_string()
        } else {
            trimmed.to_string()
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn build(
        name: impl Into<String>,
        path: impl Into<String>,
        is_script: bool,
        is_scriptlet: bool,
        is_agent: bool,
        action_verb: impl Into<String>,
        shortcut: Option<String>,
        alias: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            is_script,
            is_scriptlet,
            is_agent,
            action_verb: Self::normalized_action_verb(action_verb),
            shortcut: Self::normalized_optional_text(shortcut),
            alias: Self::normalized_optional_text(alias),
            ..Self::default()
        }
    }

    /// Create a ScriptInfo for a real script file
    pub fn new(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self::build(
            name,
            path,
            true,
            false,
            false,
            Self::DEFAULT_ACTION_VERB,
            None,
            None,
        )
    }

    /// Create a ScriptInfo for a real script file with shortcut info
    #[allow(dead_code)]
    pub fn with_shortcut(
        name: impl Into<String>,
        path: impl Into<String>,
        shortcut: Option<String>,
    ) -> Self {
        Self::build(
            name,
            path,
            true,
            false,
            false,
            Self::DEFAULT_ACTION_VERB,
            shortcut,
            None,
        )
    }

    /// Create a ScriptInfo for a scriptlet (snippet from markdown file)
    /// Scriptlets have their own actions that work with the source markdown file
    pub fn scriptlet(
        name: impl Into<String>,
        markdown_path: impl Into<String>,
        shortcut: Option<String>,
        alias: Option<String>,
    ) -> Self {
        Self::build(
            name,
            markdown_path,
            false,
            true,
            false,
            Self::DEFAULT_ACTION_VERB,
            shortcut,
            alias,
        )
    }

    /// Create a ScriptInfo for a real script file with shortcut and alias info
    #[allow(dead_code)]
    pub fn with_shortcut_and_alias(
        name: impl Into<String>,
        path: impl Into<String>,
        shortcut: Option<String>,
        alias: Option<String>,
    ) -> Self {
        Self::build(
            name,
            path,
            true,
            false,
            false,
            Self::DEFAULT_ACTION_VERB,
            shortcut,
            alias,
        )
    }

    /// Create a ScriptInfo for a built-in command (not a real script)
    /// Built-ins have limited actions (no edit, view logs, reveal in finder, copy path, configure shortcut)
    #[allow(dead_code)]
    pub fn builtin(name: impl Into<String>) -> Self {
        Self::build(
            name,
            String::new(),
            false,
            false,
            false,
            Self::DEFAULT_ACTION_VERB,
            None,
            None,
        )
    }

    /// Create a ScriptInfo with explicit is_script flag and default action verb
    #[allow(dead_code)]
    pub fn with_is_script(
        name: impl Into<String>,
        path: impl Into<String>,
        is_script: bool,
    ) -> Self {
        Self::build(
            name,
            path,
            is_script,
            false,
            false,
            Self::DEFAULT_ACTION_VERB,
            None,
            None,
        )
    }

    /// Create a ScriptInfo for an agent file.
    /// Agents are not scripts/scriptlets but expose agent-specific actions.
    #[allow(dead_code)]
    pub fn agent(
        name: impl Into<String>,
        path: impl Into<String>,
        shortcut: Option<String>,
        alias: Option<String>,
    ) -> Self {
        Self::build(
            name,
            path,
            false,
            false,
            true,
            Self::DEFAULT_ACTION_VERB,
            shortcut,
            alias,
        )
    }

    /// Create a ScriptInfo with all options including custom action verb
    pub fn with_action_verb(
        name: impl Into<String>,
        path: impl Into<String>,
        is_script: bool,
        action_verb: impl Into<String>,
    ) -> Self {
        Self::build(name, path, is_script, false, false, action_verb, None, None)
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
        Self::build(
            name,
            path,
            is_script,
            false,
            false,
            action_verb,
            shortcut,
            None,
        )
    }

    /// Create a ScriptInfo with all options including custom action verb, shortcut, and alias
    #[allow(dead_code)]
    pub fn with_all(
        name: impl Into<String>,
        path: impl Into<String>,
        is_script: bool,
        action_verb: impl Into<String>,
        shortcut: Option<String>,
        alias: Option<String>,
    ) -> Self {
        Self::build(
            name,
            path,
            is_script,
            false,
            false,
            action_verb,
            shortcut,
            alias,
        )
    }

    /// Set whether this item is suggested (has frecency data) and its frecency path
    pub fn with_frecency(mut self, is_suggested: bool, frecency_path: Option<String>) -> Self {
        let normalized_path = Self::normalized_optional_text(frecency_path);
        self.is_suggested = is_suggested && normalized_path.is_some();
        self.frecency_path = normalized_path;
        self
    }
}

impl<Name, Path> From<(Name, Path)> for ScriptInfo
where
    Name: Into<String>,
    Path: Into<String>,
{
    fn from(value: (Name, Path)) -> Self {
        Self::new(value.0, value.1)
    }
}

#[cfg(test)]
mod script_info_completeness_tests {
    use super::ScriptInfo;

    #[test]
    fn test_script_info_agent_sets_expected_flags_when_constructed() {
        let info = ScriptInfo::agent(
            "my-agent",
            "/agents/my-agent.md",
            Some("cmd+shift+a".to_string()),
            Some("agent".to_string()),
        );

        assert_eq!(info.name, "my-agent");
        assert_eq!(info.path, "/agents/my-agent.md");
        assert!(!info.is_script);
        assert!(!info.is_scriptlet);
        assert!(info.is_agent);
        assert_eq!(info.shortcut.as_deref(), Some("cmd+shift+a"));
        assert_eq!(info.alias.as_deref(), Some("agent"));
    }

    #[test]
    fn test_script_info_from_converts_mixed_tuple_when_name_owned_path_borrowed() {
        let info = ScriptInfo::from(("script".to_string(), "/path/script.ts"));

        assert_eq!(info.name, "script");
        assert_eq!(info.path, "/path/script.ts");
        assert!(info.is_script);
    }

    #[test]
    fn test_script_info_from_converts_mixed_tuple_when_name_borrowed_path_owned() {
        let info = ScriptInfo::from(("script", "/path/script.ts".to_string()));

        assert_eq!(info.name, "script");
        assert_eq!(info.path, "/path/script.ts");
        assert!(info.is_script);
    }
}
