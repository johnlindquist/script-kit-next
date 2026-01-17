//! Selected Text Message Handlers
//!
//! This module handles protocol messages related to selected text operations
//! (get, set, check/request accessibility permissions).

use crate::logging;
use crate::protocol::Message;
use tracing::{debug, info, instrument, warn};

// Conditionally import selected_text for macOS only
#[cfg(target_os = "macos")]
use crate::selected_text;

/// Result of handling a selected text message
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum SelectedTextHandleResult {
    /// Message was handled, here's the response to send back
    Handled(Message),
    /// Message was not a selected text operation
    NotHandled,
}

/// Handle selected text protocol messages.
///
/// This function checks if a message is a selected text operation and handles it
/// by calling the appropriate selected_text module functions.
///
/// # Arguments
/// * `msg` - The incoming message to potentially handle
///
/// # Returns
/// * `SelectedTextHandleResult::Handled(response)` - Message was handled, send response back
/// * `SelectedTextHandleResult::NotHandled` - Message was not a selected text operation
///
#[instrument(skip_all)]
pub fn handle_selected_text_message(msg: &Message) -> SelectedTextHandleResult {
    match msg {
        Message::GetSelectedText { request_id } => {
            debug!(request_id = %request_id, "Handling GetSelectedText");
            let response = handle_get_selected_text(request_id);
            SelectedTextHandleResult::Handled(response)
        }
        Message::SetSelectedText { text, request_id } => {
            debug!(request_id = %request_id, text_len = text.len(), "Handling SetSelectedText");
            let response = handle_set_selected_text(text, request_id);
            SelectedTextHandleResult::Handled(response)
        }
        Message::CheckAccessibility { request_id } => {
            debug!(request_id = %request_id, "Handling CheckAccessibility");
            let response = handle_check_accessibility(request_id);
            SelectedTextHandleResult::Handled(response)
        }
        Message::RequestAccessibility { request_id } => {
            debug!(request_id = %request_id, "Handling RequestAccessibility");
            let response = handle_request_accessibility(request_id);
            SelectedTextHandleResult::Handled(response)
        }
        _ => SelectedTextHandleResult::NotHandled,
    }
}

/// Handle GET_SELECTED_TEXT request
#[cfg(target_os = "macos")]
fn handle_get_selected_text(request_id: &str) -> Message {
    logging::log("EXEC", &format!("GetSelectedText request: {}", request_id));
    logging::bench_log("get_selected_text_handler_start");

    // Hide the main window first so the previous app regains focus
    // This is required for the AX API to access the text selection
    if crate::is_main_window_visible() {
        logging::bench_log("hiding_window_for_ax");
        crate::platform::hide_main_window();
    }

    // Small delay to ensure focus has transferred to the previous app
    // Testing shows 10-20ms is usually sufficient (was 50ms in SDK)
    logging::bench_log("focus_delay_start");
    std::thread::sleep(std::time::Duration::from_millis(20));
    logging::bench_log("focus_delay_done");

    logging::bench_log("ax_api_call_start");
    let result = selected_text::get_selected_text();
    logging::bench_log("ax_api_call_done");

    match result {
        Ok(text) => {
            info!(request_id = %request_id, text_len = text.len(), "Got selected text");
            logging::log(
                "EXEC",
                &format!("GetSelectedText success: {} chars", text.len()),
            );
            // Return as Submit message so SDK pending map can match by id
            Message::Submit {
                id: request_id.to_string(),
                value: Some(text),
            }
        }
        Err(e) => {
            warn!(request_id = %request_id, error = %e, "Failed to get selected text");
            logging::log("EXEC", &format!("GetSelectedText error: {}", e));
            // Return error prefixed with ERROR: so SDK can detect and reject
            Message::Submit {
                id: request_id.to_string(),
                value: Some(format!("ERROR: {}", e)),
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn handle_get_selected_text(request_id: &str) -> Message {
    logging::log(
        "EXEC",
        &format!(
            "GetSelectedText request: {} (not supported on this platform)",
            request_id
        ),
    );
    warn!(request_id = %request_id, "Selected text not supported on this platform");
    Message::Submit {
        id: request_id.to_string(),
        value: Some(String::new()),
    }
}

/// Handle SET_SELECTED_TEXT request
#[cfg(target_os = "macos")]
fn handle_set_selected_text(text: &str, request_id: &str) -> Message {
    logging::log(
        "EXEC",
        &format!(
            "SetSelectedText request: {} ({} chars)",
            request_id,
            text.len()
        ),
    );

    match selected_text::set_selected_text(text) {
        Ok(()) => {
            info!(request_id = %request_id, "Set selected text successfully");
            logging::log("EXEC", "SetSelectedText success");
            // Return success as Submit with empty value
            Message::Submit {
                id: request_id.to_string(),
                value: None,
            }
        }
        Err(e) => {
            warn!(request_id = %request_id, error = %e, "Failed to set selected text");
            logging::log("EXEC", &format!("SetSelectedText error: {}", e));
            // Return error prefixed with ERROR: so SDK can detect and reject
            Message::Submit {
                id: request_id.to_string(),
                value: Some(format!("ERROR: {}", e)),
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn handle_set_selected_text(_text: &str, request_id: &str) -> Message {
    logging::log(
        "EXEC",
        &format!(
            "SetSelectedText request: {} (not supported on this platform)",
            request_id
        ),
    );
    warn!(request_id = %request_id, "Selected text not supported on this platform");
    Message::Submit {
        id: request_id.to_string(),
        value: Some("ERROR: Not supported on this platform".to_string()),
    }
}

/// Handle CHECK_ACCESSIBILITY request
#[cfg(target_os = "macos")]
fn handle_check_accessibility(request_id: &str) -> Message {
    logging::log(
        "EXEC",
        &format!("CheckAccessibility request: {}", request_id),
    );

    let granted = selected_text::has_accessibility_permission();
    info!(request_id = %request_id, granted = granted, "Checked accessibility permission");
    logging::log("EXEC", &format!("CheckAccessibility: granted={}", granted));

    // Return as Submit with "true" or "false" string value
    Message::Submit {
        id: request_id.to_string(),
        value: Some(granted.to_string()),
    }
}

#[cfg(not(target_os = "macos"))]
fn handle_check_accessibility(request_id: &str) -> Message {
    logging::log(
        "EXEC",
        &format!(
            "CheckAccessibility request: {} (not supported on this platform)",
            request_id
        ),
    );
    // On non-macOS, report as "not granted" since the feature isn't available
    Message::Submit {
        id: request_id.to_string(),
        value: Some("false".to_string()),
    }
}

/// Handle REQUEST_ACCESSIBILITY request
#[cfg(target_os = "macos")]
fn handle_request_accessibility(request_id: &str) -> Message {
    logging::log(
        "EXEC",
        &format!("RequestAccessibility request: {}", request_id),
    );

    let granted = selected_text::request_accessibility_permission();
    info!(request_id = %request_id, granted = granted, "Requested accessibility permission");
    logging::log(
        "EXEC",
        &format!("RequestAccessibility: granted={}", granted),
    );

    // Return as Submit with "true" or "false" string value
    Message::Submit {
        id: request_id.to_string(),
        value: Some(granted.to_string()),
    }
}

#[cfg(not(target_os = "macos"))]
fn handle_request_accessibility(request_id: &str) -> Message {
    logging::log(
        "EXEC",
        &format!(
            "RequestAccessibility request: {} (not supported on this platform)",
            request_id
        ),
    );
    // On non-macOS, can't request permissions
    Message::Submit {
        id: request_id.to_string(),
        value: Some("false".to_string()),
    }
}
