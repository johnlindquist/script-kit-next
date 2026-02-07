#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::sync::Arc;

    #[test]
    fn test_take_active_script_session_returns_error_when_session_missing() {
        let shared_session: SharedSession = Arc::new(ParkingMutex::new(None));

        let error = take_active_script_session(
            &shared_session,
            "example-script",
            Path::new("/tmp/example-script.ts"),
        )
        .expect_err("missing interactive session should be reported as an error");

        assert!(error.contains("interactive_session_missing"));
        assert!(error.contains("script='example-script'"));
        assert!(error.contains("state=script_session:none"));
        assert!(error.contains("operation=split_interactive_session"));
    }
}
