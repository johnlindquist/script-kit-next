//! Action types and data structures
//!
//! Core types for the actions system including Action, ActionCategory, and ScriptInfo.
//!
//! # Architecture Overview
//!
//! The actions system is **intentionally decoupled** from the standard selection callbacks:
//!
//! - **on_select callback is bypassed by design** for keyboard navigation
//! - Actions route through `handle_action()` in the main app via keyboard events
//! - This enables consistent keyboard-driven action execution across all contexts
//!
//! ## Key Types
//!
//! - [`Action`]: Represents a single action item with id, title, category, and optional shortcut
//! - [`ActionCategory`]: Categorizes actions (ScriptContext, ScriptOps, GlobalOps)
//! - [`ScriptInfo`]: Context about the focused script/item for building context-specific actions
//!
//! ## Action ID Conventions
//!
//! All built-in action IDs use **snake_case** format:
//! - `run_script`, `edit_script`, `copy_path`, `reveal_in_finder`
//! - `add_shortcut`, `update_shortcut`, `remove_shortcut`
//! - `add_alias`, `update_alias`, `remove_alias`
//! - `copy_deeplink`, `reset_ranking`
//!
//! SDK-provided actions (from ProtocolAction) use their `name` field as-is for the ID.
//!
//! ## has_action Field
//!
//! The `has_action` field determines routing:
//! - `has_action=true`: Send ActionTriggered event to SDK, let SDK handle the action
//! - `has_action=false`: Submit value directly via protocol (built-in actions)
//!
//! Built-in actions (from `builders.rs`) have `has_action=false` by default.
//! SDK actions with handlers should set `has_action=true`.

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
    /// Used by "Reset Ranking" to know which frecency entry to remove
    pub frecency_path: Option<String>,
    /// Whether this is an agent file (.claude.md or similar)
    /// Agents have their own actions (Edit Agent, Copy Content, etc.)
    pub is_agent: bool,
}

impl ScriptInfo {
    /// Create a ScriptInfo for a real script file
    pub fn new(name: impl Into<String>, path: impl Into<String>) -> Self {
        ScriptInfo {
            name: name.into(),
            path: path.into(),
            is_script: true,
            is_scriptlet: false,
            is_agent: false,
            action_verb: "Run".to_string(),
            shortcut: None,
            alias: None,
            is_suggested: false,
            frecency_path: None,
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
            is_agent: false,
            action_verb: "Run".to_string(),
            shortcut,
            alias: None,
            is_suggested: false,
            frecency_path: None,
        }
    }

    /// Create a ScriptInfo for a scriptlet (snippet from markdown file)
    /// Scriptlets have their own actions that work with the source markdown file
    pub fn scriptlet(
        name: impl Into<String>,
        markdown_path: impl Into<String>,
        shortcut: Option<String>,
        alias: Option<String>,
    ) -> Self {
        ScriptInfo {
            name: name.into(),
            path: markdown_path.into(),
            is_script: false,
            is_scriptlet: true,
            is_agent: false,
            action_verb: "Run".to_string(),
            shortcut,
            alias,
            is_suggested: false,
            frecency_path: None,
        }
    }

    /// Create a ScriptInfo for a real script file with shortcut and alias info
    #[allow(dead_code)]
    pub fn with_shortcut_and_alias(
        name: impl Into<String>,
        path: impl Into<String>,
        shortcut: Option<String>,
        alias: Option<String>,
    ) -> Self {
        ScriptInfo {
            name: name.into(),
            path: path.into(),
            is_script: true,
            is_scriptlet: false,
            is_agent: false,
            action_verb: "Run".to_string(),
            shortcut,
            alias,
            is_suggested: false,
            frecency_path: None,
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
            is_agent: false,
            action_verb: "Run".to_string(),
            shortcut: None,
            alias: None,
            is_suggested: false,
            frecency_path: None,
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
            is_agent: false,
            action_verb: "Run".to_string(),
            shortcut: None,
            alias: None,
            is_suggested: false,
            frecency_path: None,
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
            is_agent: false,
            action_verb: action_verb.into(),
            shortcut: None,
            alias: None,
            is_suggested: false,
            frecency_path: None,
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
            is_agent: false,
            action_verb: action_verb.into(),
            shortcut,
            alias: None,
            is_suggested: false,
            frecency_path: None,
        }
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
        ScriptInfo {
            name: name.into(),
            path: path.into(),
            is_script,
            is_scriptlet: false,
            is_agent: false,
            action_verb: action_verb.into(),
            shortcut,
            alias,
            is_suggested: false,
            frecency_path: None,
        }
    }

    /// Set whether this item is suggested (has frecency data) and its frecency path
    pub fn with_frecency(mut self, is_suggested: bool, frecency_path: Option<String>) -> Self {
        self.is_suggested = is_suggested;
        self.frecency_path = frecency_path;
        self
    }
}

/// Represents a single action item in the actions menu.
///
/// Actions are created by builder functions in `builders.rs` or converted from
/// SDK-provided `ProtocolAction` messages. Each action has a unique identifier,
/// display title, and category for grouping.
///
/// # Action ID Convention
///
/// - Built-in actions: snake_case IDs (`edit_script`, `copy_path`, etc.)
/// - SDK actions: Use the `name` field from ProtocolAction as-is
/// - Scriptlet actions: Prefixed with `scriptlet_action:` followed by command
///
/// # Routing via has_action
///
/// The `has_action` field determines how actions are executed:
/// - `false` (default for built-ins): Handle locally in Rust via `handle_action()`
/// - `true` (SDK actions): Send `ActionTriggered` message to script for handling
///
/// Note: The routing logic in `handle_action()` may also read from the original
/// `ProtocolAction` for SDK-provided actions to ensure consistency.
///
/// # Examples
///
/// ```ignore
/// // Built-in action (has_action defaults to false)
/// let action = Action::new(
///     "edit_script",
///     "Edit Script",
///     Some("Open in $EDITOR".to_string()),
///     ActionCategory::ScriptContext,
/// ).with_shortcut("⌘E");
///
/// // Scriptlet action (has_action=true for SDK handling)
/// let mut action = Action::new(
///     "scriptlet_action:copy-to-clipboard",
///     "Copy to Clipboard",
///     None,
///     ActionCategory::ScriptContext,
/// );
/// action.has_action = true;
/// action.value = Some("copy-to-clipboard".to_string());
/// ```
#[derive(Debug, Clone)]
pub struct Action {
    /// Unique identifier for action routing.
    /// Built-in IDs use snake_case (e.g., `edit_script`, `copy_path`).
    /// SDK action IDs match the ProtocolAction name.
    pub id: String,

    /// Display title shown in the actions menu
    pub title: String,

    /// Optional description shown below the title
    #[allow(dead_code)]
    pub description: Option<String>,

    /// Category for grouping actions in the menu
    pub category: ActionCategory,

    /// Optional keyboard shortcut hint (e.g., "⌘E", "⇧⌘K")
    /// Displayed as a badge next to the action title
    pub shortcut: Option<String>,

    /// Routing flag: if true, send ActionTriggered to SDK; if false, handle locally.
    /// Built-in actions default to false. SDK actions with handlers set this to true.
    #[allow(dead_code)]
    pub has_action: bool,

    /// Optional value to submit when action is triggered.
    /// For scriptlet actions, this contains the command to execute.
    #[allow(dead_code)]
    pub value: Option<String>,

    /// Optional icon to display next to the action title
    pub icon: Option<IconName>,

    /// Section/group name for display (used with SectionStyle::Headers)
    pub section: Option<String>,

    // === Cached lowercase fields for fast filtering (performance optimization) ===
    // These are pre-computed on Action creation to avoid repeated to_lowercase() calls
    // during search/filter operations which happen on every keystroke.
    /// Cached lowercase title for fast filtering
    pub title_lower: String,

    /// Cached lowercase description for fast filtering
    pub description_lower: Option<String>,

    /// Cached lowercase shortcut for fast filtering
    pub shortcut_lower: Option<String>,
}

/// Configuration for how the search input is positioned
#[allow(dead_code)] // Public API - will be used by AI window integration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchPosition {
    /// Search input at top (AI chat style - list grows downward)
    Top,
    /// Search input at bottom (main menu style - list grows upward)
    #[default]
    Bottom,
    /// No search input (external search handling)
    Hidden,
}

/// Configuration for how sections/categories are displayed
#[allow(dead_code)] // Public API - will be used by AI window integration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SectionStyle {
    /// Show text headers for sections (AI chat style)
    Headers,
    /// Show subtle separators between categories (main menu style)
    #[default]
    Separators,
    /// No section indicators
    None,
}

/// Configuration for dialog anchor position during resize
#[allow(dead_code)] // Public API - will be used by AI window integration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnchorPosition {
    /// Dialog grows/shrinks from top (content pinned to top)
    Top,
    /// Dialog grows/shrinks from bottom (content pinned to bottom)
    #[default]
    Bottom,
}

/// Complete configuration for ActionsDialog appearance and behavior
#[allow(dead_code)] // Public API - will be used by AI window integration
#[derive(Debug, Clone, Default)]
pub struct ActionsDialogConfig {
    /// Position of search input
    pub search_position: SearchPosition,
    /// How to display section/category divisions
    pub section_style: SectionStyle,
    /// Which edge the dialog anchors to during resize
    pub anchor: AnchorPosition,
    /// Whether to show icons for actions (if available)
    pub show_icons: bool,
    /// Whether to show the footer with keyboard hints
    pub show_footer: bool,
}

/// Category for grouping actions in the actions menu.
///
/// Actions are organized by category to help users find relevant options:
/// - `ScriptContext`: Actions specific to the currently focused script/item
/// - `ScriptOps`: Script management operations (reserved for future use)
/// - `GlobalOps`: Application-wide actions like Settings, Quit (reserved)
///
/// Currently, most actions are `ScriptContext` since they operate on the
/// focused list item. The other categories are reserved for future expansion.
#[derive(Debug, Clone, PartialEq)]
pub enum ActionCategory {
    /// Actions specific to the currently focused script/item.
    /// Examples: Run, Edit, Copy Path, Configure Shortcut, Reset Ranking
    ScriptContext,

    /// Script management operations (reserved for future use).
    /// Intended for: Create Script, Delete Script, Duplicate Script
    #[allow(dead_code)]
    ScriptOps,

    /// Application-wide actions (reserved for future use).
    /// Intended for: Open Settings, Quit App, Check for Updates
    #[allow(dead_code)]
    GlobalOps,

    /// Terminal actions (Clear, Copy, Paste, Scroll, etc.)
    #[allow(dead_code)]
    Terminal,
}

impl Action {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: Option<String>,
        category: ActionCategory,
    ) -> Self {
        let title_str = title.into();
        let title_lower = title_str.to_lowercase();
        let description_lower = description.as_ref().map(|d| d.to_lowercase());

        Action {
            id: id.into(),
            title: title_str,
            description,
            category,
            shortcut: None,
            has_action: false,
            value: None,
            icon: None,
            section: None,
            // Pre-compute lowercase for fast filtering
            title_lower,
            description_lower,
            shortcut_lower: None,
        }
    }

    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        let shortcut_str = shortcut.into();
        self.shortcut_lower = Some(shortcut_str.to_lowercase());
        self.shortcut = Some(shortcut_str);
        self
    }

    #[allow(dead_code)] // Public API - used by get_ai_command_bar_actions
    /// Add an optional shortcut to the action
    pub fn with_shortcut_opt(mut self, shortcut: Option<String>) -> Self {
        if let Some(s) = shortcut {
            self.shortcut_lower = Some(s.to_lowercase());
            self.shortcut = Some(s);
        }
        self
    }
    pub fn with_icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    #[allow(dead_code)] // Public API - used by get_ai_command_bar_actions
    pub fn with_section(mut self, section: impl Into<String>) -> Self {
        self.section = Some(section.into());
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
        assert!(script.alias.is_none());
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
            Some("gh".to_string()),
        );
        assert_eq!(scriptlet.name, "Open GitHub");
        assert_eq!(scriptlet.path, "/path/to/url.md#open-github");
        assert!(!scriptlet.is_script);
        assert!(scriptlet.is_scriptlet);
        assert_eq!(scriptlet.shortcut, Some("cmd+g".to_string()));
        assert_eq!(scriptlet.alias, Some("gh".to_string()));
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
        assert!(builtin.alias.is_none());
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

    #[test]
    fn test_script_info_with_shortcut_and_alias() {
        let script = ScriptInfo::with_shortcut_and_alias(
            "test-script",
            "/path/to/test-script.ts",
            Some("cmd+shift+t".to_string()),
            Some("ts".to_string()),
        );
        assert_eq!(script.name, "test-script");
        assert_eq!(script.shortcut, Some("cmd+shift+t".to_string()));
        assert_eq!(script.alias, Some("ts".to_string()));
    }

    #[test]
    fn test_script_info_with_all() {
        let script = ScriptInfo::with_all(
            "App Launcher",
            "builtin:app-launcher",
            false,
            "Open",
            Some("cmd+space".to_string()),
            Some("apps".to_string()),
        );
        assert_eq!(script.name, "App Launcher");
        assert_eq!(script.path, "builtin:app-launcher");
        assert!(!script.is_script);
        assert_eq!(script.action_verb, "Open");
        assert_eq!(script.shortcut, Some("cmd+space".to_string()));
        assert_eq!(script.alias, Some("apps".to_string()));
    }

    #[test]
    fn test_script_info_with_frecency() {
        // Test with_frecency builder method
        let script = ScriptInfo::new("test-script", "/path/to/script.ts")
            .with_frecency(true, Some("/path/to/script.ts".to_string()));

        assert!(script.is_suggested);
        assert_eq!(script.frecency_path, Some("/path/to/script.ts".to_string()));
    }

    #[test]
    fn test_script_info_default_frecency_values() {
        // Test that default values are correct (not suggested, no frecency path)
        let script = ScriptInfo::new("test-script", "/path/to/script.ts");
        assert!(!script.is_suggested);
        assert!(script.frecency_path.is_none());

        let scriptlet = ScriptInfo::scriptlet("Open GitHub", "/path/to/url.md", None, None);
        assert!(!scriptlet.is_suggested);
        assert!(scriptlet.frecency_path.is_none());

        let builtin = ScriptInfo::builtin("Clipboard History");
        assert!(!builtin.is_suggested);
        assert!(builtin.frecency_path.is_none());
    }

    #[test]
    fn test_script_info_frecency_chaining() {
        // Test that with_frecency can be chained with other constructors
        let script = ScriptInfo::with_shortcut_and_alias(
            "test-script",
            "/path/to/test.ts",
            Some("cmd+t".to_string()),
            Some("ts".to_string()),
        )
        .with_frecency(true, Some("frecency:path".to_string()));

        // Original fields preserved
        assert_eq!(script.shortcut, Some("cmd+t".to_string()));
        assert_eq!(script.alias, Some("ts".to_string()));

        // Frecency fields set
        assert!(script.is_suggested);
        assert_eq!(script.frecency_path, Some("frecency:path".to_string()));
    }
}
