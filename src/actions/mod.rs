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
pub use dialog::ActionsDialog;
pub use types::ScriptInfo;

// Public API for AI window integration (re-exported but may appear unused until integration)
#[allow(unused_imports)]
pub use builders::get_ai_command_bar_actions;
#[allow(unused_imports)]
pub use types::{
    Action, ActionCategory, ActionsDialogConfig, AnchorPosition, SearchPosition, SectionStyle,
};

// Window functions for separate vibrancy window
pub use window::{
    close_actions_window, is_actions_window_open, notify_actions_window, open_actions_window,
    resize_actions_window,
};
// get_actions_window_handle available but not re-exported (use window:: directly if needed)

// CommandBar - high-level reusable component for command palette functionality
#[allow(unused_imports)]
pub use command_bar::{is_command_bar_open, CommandBar, CommandBarConfig, CommandBarHost};

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
}
