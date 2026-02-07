#[cfg(test)]
mod execute_script_session_tests {
    use super::*;
    use std::path::Path;
    use std::sync::Arc;

    #[test]
    fn test_take_active_script_session_returns_error_when_session_missing() {
        let shared_session: SharedSession = Arc::new(ParkingMutex::new(None));

        let result = take_active_script_session(
            &shared_session,
            "example-script",
            Path::new("/tmp/example-script.ts"),
        );
        assert!(
            result.is_err(),
            "missing interactive session should be reported as an error"
        );

        let error = result.err().unwrap_or_default();

        assert!(error.contains("interactive_session_missing"));
        assert!(error.contains("script='example-script'"));
        assert!(error.contains("state=script_session:none"));
        assert!(error.contains("operation=split_interactive_session"));
    }

    #[test]
    fn test_truncate_clipboard_history_preview_returns_original_when_under_limit() {
        let content = "hello clipboard";
        let truncated = truncate_clipboard_history_preview(content);

        assert_eq!(truncated, content);
    }

    #[test]
    fn test_truncate_clipboard_history_preview_does_not_split_utf8_when_over_limit() {
        let content = format!("{}😀😀", "a".repeat(CLIPBOARD_HISTORY_PREVIEW_CHAR_LIMIT - 1));
        let truncated = truncate_clipboard_history_preview(&content);

        let expected = format!("{}😀...", "a".repeat(CLIPBOARD_HISTORY_PREVIEW_CHAR_LIMIT - 1));
        assert_eq!(truncated, expected);
    }
}
