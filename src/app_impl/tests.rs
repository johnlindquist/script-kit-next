#[cfg(test)]
mod app_impl_state_sync_tests {
    use super::{calculate_fallback_error_message, ScriptListApp};

    #[test]
    fn test_sync_builtin_query_state_updates_query_and_selection_when_changed() {
        let mut query = String::from("old");
        let mut selected_index = 3;

        let changed =
            ScriptListApp::sync_builtin_query_state(&mut query, &mut selected_index, "new");

        assert!(changed);
        assert_eq!(query, "new");
        assert_eq!(selected_index, 0);
    }

    #[test]
    fn test_sync_builtin_query_state_noop_when_query_is_unchanged() {
        let mut query = String::from("same");
        let mut selected_index = 4;

        let changed =
            ScriptListApp::sync_builtin_query_state(&mut query, &mut selected_index, "same");

        assert!(!changed);
        assert_eq!(query, "same");
        assert_eq!(selected_index, 4);
    }

    #[test]
    fn test_clear_builtin_query_state_clears_text_and_resets_selection() {
        let mut query = String::from("abc");
        let mut selected_index = 2;

        ScriptListApp::clear_builtin_query_state(&mut query, &mut selected_index);

        assert!(query.is_empty());
        assert_eq!(selected_index, 0);
    }

    #[test]
    fn test_calculate_fallback_error_message_includes_expression_and_recovery() {
        let message = calculate_fallback_error_message("2 + )");
        assert!(message.contains("2 + )"));
        assert!(message.contains("Could not evaluate expression"));
        assert!(message.contains("Check the syntax and try again"));
    }
}
