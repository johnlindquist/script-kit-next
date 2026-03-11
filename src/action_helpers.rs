//! Action helper functions for handle_action and trigger_action_by_name
//!
//! This module extracts common functionality to reduce duplication:
//! - Path extraction from search results
//! - SDK action routing
//! - pbcopy helper with proper stdin handling

use std::path::PathBuf;
use std::sync::mpsc::SyncSender;

use gpui::SharedString;

use crate::protocol::{self, ProtocolAction};
use crate::scripts::SearchResult;

/// Error type for path extraction operations
#[derive(Debug, Clone, PartialEq)]
pub enum PathExtractionError {
    /// No item is selected
    NoSelection,
    /// The selected item type doesn't support this operation
    UnsupportedType(SharedString),
}

impl PathExtractionError {
    /// Get the error message for UI display
    pub fn message(&self) -> SharedString {
        match self {
            PathExtractionError::NoSelection => SharedString::from("No item selected"),
            PathExtractionError::UnsupportedType(msg) => msg.clone(),
        }
    }
}

/// Extract the filesystem path from a SearchResult for reveal/copy operations.
///
/// Supports: Script, App, Agent
/// Not supported: Scriptlet, BuiltIn, Window, Fallback
pub fn extract_path_for_reveal(
    result: Option<&SearchResult>,
) -> Result<PathBuf, PathExtractionError> {
    match result {
        None => Err(PathExtractionError::NoSelection),
        Some(SearchResult::Script(m)) => Ok(m.script.path.clone()),
        Some(SearchResult::App(m)) => Ok(m.app.path.clone()),
        Some(SearchResult::Agent(m)) => Ok(m.agent.path.clone()),
        Some(SearchResult::Scriptlet(_)) => Err(PathExtractionError::UnsupportedType(
            SharedString::from("Cannot reveal scriptlets in Finder"),
        )),
        Some(SearchResult::BuiltIn(_)) => Err(PathExtractionError::UnsupportedType(
            SharedString::from("Cannot reveal built-in features"),
        )),
        Some(SearchResult::Window(_)) => Err(PathExtractionError::UnsupportedType(
            SharedString::from("Cannot reveal windows in Finder"),
        )),
        Some(SearchResult::Fallback(_)) => Err(PathExtractionError::UnsupportedType(
            SharedString::from("Cannot reveal fallback commands in Finder"),
        )),
    }
}

/// Extract the filesystem path from a SearchResult for copy path operations.
pub fn extract_path_for_copy(
    result: Option<&SearchResult>,
) -> Result<PathBuf, PathExtractionError> {
    match result {
        None => Err(PathExtractionError::NoSelection),
        Some(SearchResult::Script(m)) => Ok(m.script.path.clone()),
        Some(SearchResult::App(m)) => Ok(m.app.path.clone()),
        Some(SearchResult::Agent(m)) => Ok(m.agent.path.clone()),
        Some(SearchResult::Scriptlet(_)) => Err(PathExtractionError::UnsupportedType(
            SharedString::from("Cannot copy scriptlet path"),
        )),
        Some(SearchResult::BuiltIn(_)) => Err(PathExtractionError::UnsupportedType(
            SharedString::from("Cannot copy built-in path"),
        )),
        Some(SearchResult::Window(_)) => Err(PathExtractionError::UnsupportedType(
            SharedString::from("Cannot copy window path"),
        )),
        Some(SearchResult::Fallback(_)) => Err(PathExtractionError::UnsupportedType(
            SharedString::from("Cannot copy fallback command path"),
        )),
    }
}

/// Extract the filesystem path from a SearchResult for edit operations.
///
/// Supports: Script, Agent
/// Not supported: Scriptlet, BuiltIn, App, Window, Fallback
pub fn extract_path_for_edit(
    result: Option<&SearchResult>,
) -> Result<PathBuf, PathExtractionError> {
    match result {
        None => Err(PathExtractionError::NoSelection),
        Some(SearchResult::Script(m)) => Ok(m.script.path.clone()),
        Some(SearchResult::Agent(m)) => Ok(m.agent.path.clone()),
        Some(SearchResult::Scriptlet(_)) => Err(PathExtractionError::UnsupportedType(
            SharedString::from("Cannot edit scriptlets"),
        )),
        Some(SearchResult::BuiltIn(_)) => Err(PathExtractionError::UnsupportedType(
            SharedString::from("Cannot edit built-in features"),
        )),
        Some(SearchResult::App(_)) => Err(PathExtractionError::UnsupportedType(
            SharedString::from("Cannot edit applications"),
        )),
        Some(SearchResult::Window(_)) => Err(PathExtractionError::UnsupportedType(
            SharedString::from("Cannot edit windows"),
        )),
        Some(SearchResult::Fallback(_)) => Err(PathExtractionError::UnsupportedType(
            SharedString::from("Cannot edit fallback commands"),
        )),
    }
}

/// Copy text to clipboard using pbcopy on macOS.
///
/// **Critical fix**: This properly closes stdin before waiting to prevent hangs.
/// pbcopy reads until EOF, so stdin must be dropped before wait() is called.
#[cfg(target_os = "macos")]
pub fn pbcopy(text: &str) -> Result<(), std::io::Error> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new("pbcopy").stdin(Stdio::piped()).spawn()?;

    // Take ownership of stdin, write, then drop to signal EOF
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes())?;
        // stdin is dropped here => EOF delivered to pbcopy
    }

    // Now it's safe to wait - pbcopy has received EOF
    let status = child.wait()?;
    if !status.success() {
        return Err(std::io::Error::other(format!(
            "pbcopy exited with status: {}",
            status
        )));
    }
    Ok(())
}

/// Result of attempting to trigger an SDK action.
#[derive(Debug, Clone, PartialEq)]
pub enum SdkActionResult {
    /// Message was successfully sent to the response channel.
    Sent,
    /// No response channel available (no running script).
    NoSender,
    /// Action has no handler and no value — nothing to send.
    NoEffect,
    /// Response channel is full — message was dropped.
    ChannelFull,
    /// Response channel is disconnected — script has exited.
    ChannelDisconnected,
}

impl SdkActionResult {
    /// Whether the action was successfully dispatched.
    pub fn is_sent(&self) -> bool {
        matches!(self, SdkActionResult::Sent)
    }

    /// User-facing error message, if any.
    pub fn error_message(&self, action_name: &str) -> Option<String> {
        match self {
            SdkActionResult::Sent | SdkActionResult::NoEffect => None,
            SdkActionResult::NoSender => {
                Some(format!("Action '{}' failed: no active script", action_name))
            }
            SdkActionResult::ChannelFull => {
                Some(format!("Action '{}' failed: channel busy", action_name))
            }
            SdkActionResult::ChannelDisconnected => {
                Some("Action failed: script has exited".to_string())
            }
        }
    }
}

/// Trigger an SDK action and send the appropriate message to the response channel.
///
/// Routes based on `action.has_action`:
/// - `true`: Sends `ActionTriggered` message
/// - `false` with value: Sends `Submit` message with the value
/// - `false` without value: Logs warning, no message sent
///
/// Returns an `SdkActionResult` indicating what happened, so callers can
/// show appropriate feedback (e.g. Toast on error).
pub fn trigger_sdk_action(
    action_name: &str,
    action: &ProtocolAction,
    current_input: &str,
    sender: Option<&SyncSender<protocol::Message>>,
) -> SdkActionResult {
    let Some(sender) = sender else {
        tracing::warn!(action = %action_name, "no response sender for SDK action");
        return SdkActionResult::NoSender;
    };

    let send_result = if action.has_action {
        tracing::info!(action = %action_name, "SDK action with handler, sending ActionTriggered");
        let msg = protocol::Message::action_triggered(
            action_name.to_string(),
            action.value.clone(),
            current_input.to_string(),
        );
        sender.try_send(msg)
    } else if let Some(ref value) = action.value {
        tracing::info!(action = %action_name, value = ?value, "SDK action without handler, submitting value");
        let msg = protocol::Message::Submit {
            id: "action".to_string(),
            value: Some(value.clone()),
        };
        sender.try_send(msg)
    } else {
        tracing::info!(action = %action_name, "SDK action has no value and has_action=false");
        return SdkActionResult::NoEffect;
    };

    match send_result {
        Ok(()) => SdkActionResult::Sent,
        Err(std::sync::mpsc::TrySendError::Full(_)) => {
            tracing::warn!(action = %action_name, "response channel full - SDK action dropped");
            SdkActionResult::ChannelFull
        }
        Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
            tracing::info!(action = %action_name, "response channel disconnected - script exited");
            SdkActionResult::ChannelDisconnected
        }
    }
}

/// Reserved built-in action IDs that SDK actions cannot shadow.
pub const RESERVED_ACTION_IDS: &[&str] = &[
    // Script context actions
    "run_script",
    "view_logs",
    "reveal_in_finder",
    "copy_path",
    "edit_script",
    "copy_deeplink",
    "reset_ranking",
    // Dynamic shortcut actions (context-dependent)
    "add_shortcut",
    "update_shortcut",
    "remove_shortcut",
    // File search context actions
    "open_file",
    "open_directory",
    "quick_look",
    "open_with",
    "show_info",
    "copy_filename",
    // Path prompt context actions
    "select_file",
    "open_in_finder",
    "open_in_editor",
    "open_in_terminal",
    "move_to_trash",
    // Internal
    "__cancel__",
];

/// Check if an action ID is reserved by the built-in actions.
pub fn is_reserved_action_id(action_id: &str) -> bool {
    RESERVED_ACTION_IDS.contains(&action_id)
}

/// Find an SDK action by name, with optional shadowing warning.
pub fn find_sdk_action<'a>(
    actions: Option<&'a [ProtocolAction]>,
    action_name: &str,
    warn_on_shadow: bool,
) -> Option<&'a ProtocolAction> {
    let actions = actions?;

    if warn_on_shadow && is_reserved_action_id(action_name) {
        tracing::warn!(action = %action_name, "SDK action shadows a built-in action - will be ignored");
    }

    actions.iter().find(|a| a.name == action_name)
}

#[cfg(test)]
#[path = "action_helpers_tests.rs"]
mod tests;
