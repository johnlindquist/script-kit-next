#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ==========================================
    // TDD TESTS - Written FIRST before implementation
    // ==========================================

    #[test]
    fn test_sse_event_format() {
        // Test that SSE events are formatted correctly per the SSE spec:
        // event: {type}\ndata: {json}\n\n

        let data = serde_json::json!({"message": "hello", "progress": 50});
        let formatted = format_sse_event(SseEventType::Progress, &data);

        // Must start with "event: progress\n"
        assert!(
            formatted.starts_with("event: progress\n"),
            "Event line must come first"
        );

        // Must contain "data: " line with JSON
        assert!(formatted.contains("data: "), "Must have data line");
        assert!(
            formatted.contains(r#""message":"hello""#),
            "Data must contain JSON"
        );
        assert!(
            formatted.contains(r#""progress":50"#),
            "Data must contain progress"
        );

        // Must end with double newline
        assert!(
            formatted.ends_with("\n\n"),
            "Must end with double newline for SSE"
        );

        // Test all event types format correctly
        for event_type in [
            SseEventType::Progress,
            SseEventType::Output,
            SseEventType::Error,
            SseEventType::Complete,
        ] {
            let formatted = format_sse_event(event_type, &serde_json::json!({}));
            assert!(
                formatted.starts_with(&format!("event: {}\n", event_type.as_str())),
                "Event type {} should format correctly",
                event_type
            );
        }
    }

    #[test]
    fn test_sse_stream_broadcast() {
        let mut stream = SseStream::new();

        // Initially empty
        assert_eq!(stream.pending_count(), 0);

        // Broadcast some events
        stream.broadcast_event(SseEventType::Progress, &serde_json::json!({"step": 1}));
        stream.broadcast_event(SseEventType::Output, &serde_json::json!({"line": "test"}));

        assert_eq!(stream.pending_count(), 2);

        // Drain events
        let events = stream.drain_events();
        assert_eq!(events.len(), 2);
        assert!(events[0].contains("event: progress"));
        assert!(events[1].contains("event: output"));

        // Buffer should be empty after drain
        assert_eq!(stream.pending_count(), 0);
    }

    #[test]
    fn test_sse_heartbeat_format() {
        let heartbeat = format_sse_heartbeat();

        // Heartbeat is a comment (starts with :)
        assert!(
            heartbeat.starts_with(":"),
            "Heartbeat must be SSE comment (start with :)"
        );
        assert!(
            heartbeat.ends_with("\n\n"),
            "Heartbeat must end with double newline"
        );
    }

    #[test]
    fn test_audit_log_written() {
        // Test that audit logs are actually written to the file
        let temp_dir = TempDir::new().unwrap();
        let logger = AuditLogger::new(temp_dir.path().to_path_buf());

        // Log should not exist yet
        assert!(
            !logger.log_path().exists(),
            "Log file should not exist initially"
        );

        // Log a successful call
        logger
            .log_success(
                "tools/run_script",
                serde_json::json!({"name": "test.ts"}),
                100,
            )
            .expect("Should write log successfully");

        // Log file should now exist
        assert!(logger.log_path().exists(), "Log file should be created");

        // Read and verify content
        let content = fs::read_to_string(logger.log_path()).unwrap();
        assert!(!content.is_empty(), "Log file should have content");

        // Log another entry
        logger
            .log_failure(
                "tools/bad_call",
                serde_json::json!({}),
                50,
                "Invalid params",
            )
            .expect("Should write failure log");

        // Should have two lines
        let content = fs::read_to_string(logger.log_path()).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2, "Should have 2 log entries");
    }

    #[test]
    fn test_audit_log_format() {
        // Test that audit log entries have the correct JSONL format
        let temp_dir = TempDir::new().unwrap();
        let logger = AuditLogger::new(temp_dir.path().to_path_buf());

        let params = serde_json::json!({
            "script": "hello.ts",
            "args": ["--verbose"]
        });

        logger
            .log_success("tools/run_script", params.clone(), 250)
            .expect("Should log successfully");

        // Read and parse the log entry
        let content = fs::read_to_string(logger.log_path()).unwrap();
        let entry: AuditLogEntry =
            serde_json::from_str(content.trim()).expect("Log entry should be valid JSON");

        // Verify all required fields
        assert!(!entry.timestamp.is_empty(), "timestamp must be present");
        assert!(
            entry.timestamp.contains("T"),
            "timestamp must be ISO 8601 format"
        );
        assert_eq!(entry.method, "tools/run_script", "method must match");
        assert_eq!(entry.params, params, "params must match");
        assert_eq!(entry.duration_ms, 250, "duration_ms must match");
        assert!(entry.success, "success must be true");
        assert!(entry.error.is_none(), "error must be None for success");

        // Test failure entry format
        logger
            .log_failure(
                "tools/fail",
                serde_json::json!({}),
                10,
                "Something went wrong",
            )
            .unwrap();

        let content = fs::read_to_string(logger.log_path()).unwrap();
        let last_line = content.lines().last().unwrap();
        let fail_entry: AuditLogEntry = serde_json::from_str(last_line).unwrap();

        assert!(!fail_entry.success, "success must be false for failure");
        assert_eq!(
            fail_entry.error,
            Some("Something went wrong".to_string()),
            "error message must match"
        );
    }

    #[test]
    fn test_audit_entry_constructors() {
        let params = serde_json::json!({"test": true});

        // Test success constructor
        let success = AuditLogEntry::success("my_method", params.clone(), 100);
        assert_eq!(success.method, "my_method");
        assert_eq!(success.params, params);
        assert_eq!(success.duration_ms, 100);
        assert!(success.success);
        assert!(success.error.is_none());

        // Test failure constructor
        let failure = AuditLogEntry::failure("my_method", params.clone(), 50, "oops");
        assert_eq!(failure.method, "my_method");
        assert_eq!(failure.params, params);
        assert_eq!(failure.duration_ms, 50);
        assert!(!failure.success);
        assert_eq!(failure.error, Some("oops".to_string()));
    }

    #[test]
    fn test_sse_event_type_display() {
        assert_eq!(SseEventType::Progress.as_str(), "progress");
        assert_eq!(SseEventType::Output.as_str(), "output");
        assert_eq!(SseEventType::Error.as_str(), "error");
        assert_eq!(SseEventType::Complete.as_str(), "complete");

        assert_eq!(format!("{}", SseEventType::Progress), "progress");
    }

    #[test]
    fn test_iso8601_timestamp_format() {
        let ts = iso8601_now();

        // Should be in format: YYYY-MM-DDTHH:MM:SS.mmmZ
        assert!(ts.len() >= 24, "Timestamp should be at least 24 chars");
        assert!(ts.contains("T"), "Should have T separator");
        assert!(ts.ends_with("Z"), "Should end with Z for UTC");

        // Should be parseable (basic validation)
        let parts: Vec<&str> = ts.split('T').collect();
        assert_eq!(parts.len(), 2, "Should have date and time parts");

        let date_parts: Vec<&str> = parts[0].split('-').collect();
        assert_eq!(date_parts.len(), 3, "Date should have 3 parts");

        // Year should be reasonable
        let year: i32 = date_parts[0].parse().unwrap();
        assert!(year >= 2024, "Year should be current or later");
    }
}
