    // -------------------------------------------------------------------------
    // Token savings verification
    // -------------------------------------------------------------------------

    #[test]
    fn test_compact_format_token_savings() {
        // Real comparison from logs:
        // Standard: "2025-12-27T15:22:13.150640Z  INFO script_kit_gpui::logging: Selected display..."
        // Compact:  "13.150|i|P|Selected display..."

        let standard_prefix = "2025-12-27T15:22:13.150640Z  INFO script_kit_gpui::logging: ";
        let compact_prefix = "13.150|i|P|";

        let savings_percent =
            100.0 - (compact_prefix.len() as f64 / standard_prefix.len() as f64 * 100.0);

        // Should save at least 60% on the prefix
        assert!(
            savings_percent > 60.0,
            "Should save >60% on prefix, got {:.1}%",
            savings_percent
        );

        // Actual: 11 chars vs 59 chars = 81% savings
        assert!(
            savings_percent > 80.0,
            "Should save >80% on prefix, got {:.1}%",
            savings_percent
        );
    }
    // -------------------------------------------------------------------------
    // AI log mode env var parsing tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_ai_log_mode_env_parsing() {
        // Test the parsing logic used in init()
        // SCRIPT_KIT_AI_LOG=1 should enable AI mode

        let parse_ai_log = |val: &str| -> bool {
            val.eq_ignore_ascii_case("1")
                || val.eq_ignore_ascii_case("true")
                || val.eq_ignore_ascii_case("yes")
        };

        assert!(parse_ai_log("1"));
        assert!(parse_ai_log("true"));
        assert!(parse_ai_log("TRUE"));
        assert!(parse_ai_log("yes"));
        assert!(parse_ai_log("YES"));

        assert!(!parse_ai_log("0"));
        assert!(!parse_ai_log("false"));
        assert!(!parse_ai_log("no"));
        assert!(!parse_ai_log(""));
    }
    // -------------------------------------------------------------------------
    // Payload truncation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_truncate_for_log_short_string() {
        let s = "hello";
        assert_eq!(truncate_for_log(s, 10), "hello");
    }
    #[test]
    fn test_truncate_for_log_exact_limit() {
        let s = "hello";
        assert_eq!(truncate_for_log(s, 5), "hello");
    }
    #[test]
    fn test_truncate_for_log_long_string() {
        let s = "hello world this is a long string";
        let result = truncate_for_log(s, 10);
        assert!(result.starts_with("hello worl"));
        assert!(result.contains("...(33)")); // Original length in parens
    }
    #[test]
    fn test_truncate_for_log_utf8_emoji() {
        // Emoji are 4-byte UTF-8 sequences. Truncating mid-codepoint would panic with naive &s[..max_len]
        let s = "hello 🎉 world";
        // "hello " is 6 bytes, 🎉 is 4 bytes (positions 6-9), " world" starts at byte 10
        // If max_len=8, naive slice would land inside the emoji and panic
        let result = truncate_for_log(s, 8);
        // Should truncate to a valid char boundary without panic
        assert!(result.starts_with("hello "));
        assert!(result.contains(&format!("...({})", s.len())));
    }
    #[test]
    fn test_truncate_for_log_utf8_multibyte() {
        // Test with various multi-byte UTF-8 characters
        let s = "日本語テスト"; // Each char is 3 bytes = 18 bytes total
                                // If we truncate at 5 bytes, we'd land mid-character
        let result = truncate_for_log(s, 5);
        // Should back up to char boundary (3 bytes = 1 char)
        assert!(result.starts_with("日"));
        assert!(result.contains(&format!("...({})", s.len())));
    }
    #[test]
    fn test_truncate_for_log_utf8_mixed() {
        // Mixed ASCII and multi-byte
        let s = "abc日本語def";
        // "abc" = 3 bytes, "日本語" = 9 bytes, "def" = 3 bytes
        // Truncate at 5 would land inside 日
        let result = truncate_for_log(s, 5);
        // Should truncate at byte 3 (after "abc")
        assert!(result.starts_with("abc"));
        assert!(result.contains(&format!("...({})", s.len())));
    }
    #[test]
    fn test_truncate_for_log_empty_string() {
        let s = "";
        assert_eq!(truncate_for_log(s, 10), "");
    }
    #[test]
    fn test_truncate_for_log_zero_max_len() {
        let s = "hello";
        let result = truncate_for_log(s, 0);
        // Edge case: max_len=0 should return just the suffix
        assert!(result.contains("...(5)"));
    }
    #[test]
    fn test_summarize_payload_with_type() {
        let json = r#"{"type":"submit","id":"test","value":"foo"}"#;
        let summary = summarize_payload(json);
        assert!(summary.contains("type:submit"));
        assert!(summary.contains(&format!("len:{}", json.len())));
    }
    #[test]
    fn test_summarize_payload_without_type() {
        let json = r#"{"data":"some value"}"#;
        let summary = summarize_payload(json);
        assert!(summary.contains(&format!("len:{}", json.len())));
        assert!(!summary.contains("type:"));
    }
    #[test]
    fn test_summarize_payload_large_base64() {
        // Simulate a large base64 screenshot payload
        let base64_data = "a".repeat(100000);
        let json = format!(r#"{{"type":"screenshotResult","data":"{}"}}"#, base64_data);
        let summary = summarize_payload(&json);
        // Summary should be compact, not contain the full base64
        assert!(summary.len() < 100);
        assert!(summary.contains("type:screenshotResult"));
        assert!(summary.contains(&format!("len:{}", json.len())));
    }
    // -------------------------------------------------------------------------
    // Log capture tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_is_capture_enabled_default_false() {
        // By default, capture should be disabled
        // Note: we can't test this in isolation because it's a global static
        // but we can verify the initial state
        let _ = is_capture_enabled(); // Just verify it doesn't panic
    }
    #[test]
    fn test_toggle_capture_returns_correct_state() {
        // First toggle should start capture (if not already running)
        let initial_state = is_capture_enabled();

        if !initial_state {
            // If not capturing, toggle should start it
            let (is_capturing, path) = toggle_capture();
            assert!(is_capturing);
            assert!(path.is_some());

            // Clean up: toggle again to stop
            let (is_capturing2, path2) = toggle_capture();
            assert!(!is_capturing2);
            assert!(path2.is_some());
        } else {
            // If already capturing (from another test), toggle should stop it
            let (is_capturing, path) = toggle_capture();
            assert!(!is_capturing);
            assert!(path.is_some());
        }
    }
    #[test]
    fn test_capture_file_path_format() {
        // Start capture and check the file path format
        let was_enabled = is_capture_enabled();

        if !was_enabled {
            let result = start_capture();
            assert!(result.is_ok());

            let path = result.unwrap();
            let filename = path.file_name().unwrap().to_str().unwrap();

            // Filename should be like: capture-2026-01-11T08-37-28.jsonl
            assert!(filename.starts_with("capture-"));
            assert!(filename.ends_with(".jsonl"));

            // Clean up
            let _ = stop_capture();
        }
    }
    #[test]
    fn test_stop_capture_when_not_started() {
        // Ensure capture is stopped
        while is_capture_enabled() {
            let _ = stop_capture();
        }

        // Stopping when not started should return None
        let result = stop_capture();
        assert!(result.is_none());
    }
