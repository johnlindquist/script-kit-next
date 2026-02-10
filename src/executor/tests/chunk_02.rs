// ============================================================
// Selected Text Handler Tests
// ============================================================

use super::{handle_selected_text_message, SelectedTextHandleResult};
use crate::protocol::Message;

#[cfg(feature = "system-tests")]
#[test]
fn test_handle_get_selected_text_returns_handled() {
    let msg = Message::get_selected_text("req-001".to_string());
    let result = handle_selected_text_message(&msg);

    match result {
        SelectedTextHandleResult::Handled(response) => {
            // Response should be Submit message (for SDK compatibility)
            match response {
                Message::Submit { id, .. } => {
                    assert_eq!(id, "req-001");
                }
                _ => panic!("Expected Submit response, got {:?}", response),
            }
        }
        SelectedTextHandleResult::NotHandled => {
            panic!("Expected message to be handled");
        }
    }
}

#[cfg(feature = "system-tests")]
#[test]
fn test_handle_set_selected_text_returns_handled() {
    let msg = Message::set_selected_text_msg("test text".to_string(), "req-002".to_string());
    let result = handle_selected_text_message(&msg);

    match result {
        SelectedTextHandleResult::Handled(response) => {
            // Response should be Submit message (for SDK compatibility)
            match response {
                Message::Submit { id, .. } => {
                    assert_eq!(id, "req-002");
                }
                _ => panic!("Expected Submit response, got {:?}", response),
            }
        }
        SelectedTextHandleResult::NotHandled => {
            panic!("Expected message to be handled");
        }
    }
}

#[cfg(feature = "system-tests")]
#[test]
fn test_handle_check_accessibility_returns_handled() {
    let msg = Message::check_accessibility("req-003".to_string());
    let result = handle_selected_text_message(&msg);

    match result {
        SelectedTextHandleResult::Handled(response) => {
            // Response should be Submit message with "true" or "false" value
            match response {
                Message::Submit { id, value } => {
                    assert_eq!(id, "req-003");
                    // value should be "true" or "false"
                    assert!(
                        value == Some("true".to_string()) || value == Some("false".to_string())
                    );
                }
                _ => panic!("Expected Submit response, got {:?}", response),
            }
        }
        SelectedTextHandleResult::NotHandled => {
            panic!("Expected message to be handled");
        }
    }
}

#[cfg(feature = "system-tests")]
#[test]
fn test_handle_request_accessibility_returns_handled() {
    let msg = Message::request_accessibility("req-004".to_string());
    let result = handle_selected_text_message(&msg);

    match result {
        SelectedTextHandleResult::Handled(response) => {
            // Response should be Submit message with "true" or "false" value
            match response {
                Message::Submit { id, value } => {
                    assert_eq!(id, "req-004");
                    // value should be "true" or "false"
                    assert!(
                        value == Some("true".to_string()) || value == Some("false".to_string())
                    );
                }
                _ => panic!("Expected Submit response, got {:?}", response),
            }
        }
        SelectedTextHandleResult::NotHandled => {
            panic!("Expected message to be handled");
        }
    }
}

#[test]
fn test_unrelated_message_returns_not_handled() {
    let msg = Message::beep();
    let result = handle_selected_text_message(&msg);

    match result {
        SelectedTextHandleResult::Handled(_) => {
            panic!("Expected message to not be handled");
        }
        SelectedTextHandleResult::NotHandled => {
            // Expected
        }
    }
}

#[test]
fn test_arg_message_returns_not_handled() {
    let msg = Message::arg("1".to_string(), "Pick".to_string(), vec![]);
    let result = handle_selected_text_message(&msg);

    match result {
        SelectedTextHandleResult::Handled(_) => {
            panic!("Expected message to not be handled");
        }
        SelectedTextHandleResult::NotHandled => {
            // Expected
        }
    }
}

#[test]
fn test_response_messages_not_handled() {
    // Response messages shouldn't be handled (they're outgoing, not incoming)
    // Submit messages are responses, so they should not be handled
    let msg1 = Message::Submit {
        id: "req-x".to_string(),
        value: Some("text".to_string()),
    };

    assert!(matches!(
        handle_selected_text_message(&msg1),
        SelectedTextHandleResult::NotHandled
    ));
}

