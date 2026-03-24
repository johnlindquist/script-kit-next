// ============================================================
// Selected Text Handler Tests
// ============================================================

use super::{handle_selected_text_message, SelectedTextHandleResult};
use crate::protocol::Message;

// REMOVED: test_handle_get_selected_text_returns_handled — triggers real Cmd+C
// REMOVED: test_handle_set_selected_text_returns_handled — triggers real Cmd+V
// REMOVED: test_handle_check_accessibility_returns_handled — OS side effects
// REMOVED: test_handle_request_accessibility_returns_handled — OS side effects

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

