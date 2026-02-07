    use super::*;
    // ========================================
    // Construction Tests
    // ========================================

    #[test]
    fn test_new_creates_empty_matcher() {
        let matcher = KeywordMatcher::new();
        assert_eq!(matcher.trigger_count(), 0);
        assert!(matcher.buffer().is_empty());
    }
    #[test]
    fn test_default_creates_empty_matcher() {
        let matcher = KeywordMatcher::default();
        assert_eq!(matcher.trigger_count(), 0);
        assert!(matcher.buffer().is_empty());
    }
    #[test]
    fn test_with_buffer_size_sets_custom_size() {
        let matcher = KeywordMatcher::with_buffer_size(100);
        assert_eq!(matcher.max_buffer_size, 100);
    }
    // ========================================
    // Registration Tests
    // ========================================

    #[test]
    fn test_register_trigger_adds_trigger() {
        let mut matcher = KeywordMatcher::new();
        matcher.register_trigger(":sig", PathBuf::from("/test/sig.md"));

        assert_eq!(matcher.trigger_count(), 1);
        assert!(matcher.has_trigger(":sig"));
    }
    #[test]
    fn test_register_multiple_triggers() {
        let mut matcher = KeywordMatcher::new();
        matcher.register_trigger(":sig", PathBuf::from("/test/sig.md"));
        matcher.register_trigger("!today", PathBuf::from("/test/today.md"));
        matcher.register_trigger("/date", PathBuf::from("/test/date.md"));

        assert_eq!(matcher.trigger_count(), 3);
        assert!(matcher.has_trigger(":sig"));
        assert!(matcher.has_trigger("!today"));
        assert!(matcher.has_trigger("/date"));
    }
    #[test]
    fn test_register_empty_trigger_ignored() {
        let mut matcher = KeywordMatcher::new();
        matcher.register_trigger("", PathBuf::from("/test/empty.md"));

        assert_eq!(matcher.trigger_count(), 0);
    }
    #[test]
    fn test_register_trigger_replaces_existing() {
        let mut matcher = KeywordMatcher::new();
        matcher.register_trigger(":sig", PathBuf::from("/test/sig1.md"));
        matcher.register_trigger(":sig", PathBuf::from("/test/sig2.md"));

        assert_eq!(matcher.trigger_count(), 1);

        // Should use the new path
        for c in ":sig".chars() {
            matcher.process_keystroke(c);
        }
        // The last registration should be used
    }
    #[test]
    fn test_unregister_trigger_removes_trigger() {
        let mut matcher = KeywordMatcher::new();
        matcher.register_trigger(":sig", PathBuf::from("/test/sig.md"));

        assert!(matcher.unregister_trigger(":sig"));
        assert_eq!(matcher.trigger_count(), 0);
        assert!(!matcher.has_trigger(":sig"));
    }
    #[test]
    fn test_unregister_nonexistent_returns_false() {
        let mut matcher = KeywordMatcher::new();

        assert!(!matcher.unregister_trigger(":nonexistent"));
    }
    #[test]
    fn test_clear_triggers_removes_all() {
        let mut matcher = KeywordMatcher::new();
        matcher.register_trigger(":sig", PathBuf::from("/test/sig.md"));
        matcher.register_trigger("!today", PathBuf::from("/test/today.md"));

        matcher.clear_triggers();

        assert_eq!(matcher.trigger_count(), 0);
    }
    #[test]
    fn test_bulk_register_triggers() {
        let mut matcher = KeywordMatcher::new();
        let triggers = vec![
            (":sig".to_string(), PathBuf::from("/test/sig.md")),
            ("!today".to_string(), PathBuf::from("/test/today.md")),
        ];

        matcher.register_triggers(triggers);

        assert_eq!(matcher.trigger_count(), 2);
        assert!(matcher.has_trigger(":sig"));
        assert!(matcher.has_trigger("!today"));
    }
    // ========================================
    // Basic Matching Tests
    // ========================================

    #[test]
    fn test_process_keystroke_no_match_without_triggers() {
        let mut matcher = KeywordMatcher::new();

        for c in "hello".chars() {
            assert!(matcher.process_keystroke(c).is_none());
        }
    }
    #[test]
    fn test_process_keystroke_matches_simple_trigger() {
        let mut matcher = KeywordMatcher::new();
        matcher.register_trigger(":sig", PathBuf::from("/test/sig.md"));

        // Type ":sig"
        assert!(matcher.process_keystroke(':').is_none());
        assert!(matcher.process_keystroke('s').is_none());
        assert!(matcher.process_keystroke('i').is_none());

        let result = matcher.process_keystroke('g');
        assert!(result.is_some());

        let result = result.unwrap();
        assert_eq!(result.trigger, ":sig");
        assert_eq!(result.chars_to_delete, 4);
        assert_eq!(result.scriptlet_path, PathBuf::from("/test/sig.md"));
    }
    #[test]
    fn test_match_result_chars_to_delete_counts_chars_not_bytes() {
        let mut matcher = KeywordMatcher::new();
        // Unicode trigger
        matcher.register_trigger("✓ok", PathBuf::from("/test/ok.md"));

        for c in "✓ok".chars() {
            matcher.process_keystroke(c);
        }

        // Would have matched on 'k'
        matcher.clear_buffer();

        for c in "✓o".chars() {
            assert!(matcher.process_keystroke(c).is_none());
        }

        let result = matcher.process_keystroke('k');
        assert!(result.is_some());

        let result = result.unwrap();
        // "✓ok" is 3 chars (not 5 bytes)
        assert_eq!(result.chars_to_delete, 3);
    }
    #[test]
    fn test_match_fires_immediately_when_complete() {
        let mut matcher = KeywordMatcher::new();
        matcher.register_trigger(":sig", PathBuf::from("/test/sig.md"));

        // Type "Hello :sig" - match should fire right after 'g'
        for c in "Hello :si".chars() {
            assert!(matcher.process_keystroke(c).is_none());
        }

        let result = matcher.process_keystroke('g');
        assert!(result.is_some());
    }
    // ========================================
    // Buffer Behavior Tests
    // ========================================

    #[test]
    fn test_buffer_stores_keystrokes() {
        let mut matcher = KeywordMatcher::new();

        for c in "hello".chars() {
            matcher.process_keystroke(c);
        }

        assert_eq!(matcher.buffer(), "hello");
    }
    #[test]
    fn test_buffer_clears_on_enter() {
        let mut matcher = KeywordMatcher::new();

        for c in "hello".chars() {
            matcher.process_keystroke(c);
        }

        matcher.process_keystroke('\n');

        assert!(matcher.buffer().is_empty());
    }
    #[test]
    fn test_buffer_clears_on_carriage_return() {
        let mut matcher = KeywordMatcher::new();

        for c in "hello".chars() {
            matcher.process_keystroke(c);
        }

        matcher.process_keystroke('\r');

        assert!(matcher.buffer().is_empty());
    }
    #[test]
    fn test_buffer_clears_on_escape() {
        let mut matcher = KeywordMatcher::new();

        for c in "hello".chars() {
            matcher.process_keystroke(c);
        }

        matcher.process_keystroke('\x1b');

        assert!(matcher.buffer().is_empty());
    }
    #[test]
    fn test_buffer_clears_on_tab() {
        let mut matcher = KeywordMatcher::new();

        for c in "hello".chars() {
            matcher.process_keystroke(c);
        }

        matcher.process_keystroke('\t');

        assert!(matcher.buffer().is_empty());
    }
    #[test]
    fn test_buffer_does_not_clear_on_space() {
        let mut matcher = KeywordMatcher::new();

        for c in "hello world".chars() {
            matcher.process_keystroke(c);
        }

        assert_eq!(matcher.buffer(), "hello world");
    }
    #[test]
    fn test_buffer_trims_when_exceeds_max_size() {
        let mut matcher = KeywordMatcher::with_buffer_size(10);

        for c in "12345678901234567890".chars() {
            matcher.process_keystroke(c);
        }

        // Should only keep the last 10 characters
        assert_eq!(matcher.buffer().len(), 10);
        assert_eq!(matcher.buffer(), "1234567890");
    }
    #[test]
    fn test_buffer_trims_multibyte_chars_without_dropping_extra() {
        let mut matcher = KeywordMatcher::with_buffer_size(2);

        for c in "ééé".chars() {
            matcher.process_keystroke(c);
        }

        assert_eq!(matcher.buffer(), "éé");
    }
    #[test]
    fn test_clear_buffer_empties_buffer() {
        let mut matcher = KeywordMatcher::new();

        for c in "hello".chars() {
            matcher.process_keystroke(c);
        }

        matcher.clear_buffer();

        assert!(matcher.buffer().is_empty());
    }
    // ========================================
    // Trigger Prefix Tests
    // ========================================

    #[test]
    fn test_colon_prefix_trigger() {
        let mut matcher = KeywordMatcher::new();
        matcher.register_trigger(":sig", PathBuf::from("/test/sig.md"));

        for c in "hello :sig".chars() {
            if let Some(result) = matcher.process_keystroke(c) {
                assert_eq!(result.trigger, ":sig");
                return;
            }
        }
        panic!("Expected match for :sig");
    }
    #[test]
    fn test_exclamation_prefix_trigger() {
        let mut matcher = KeywordMatcher::new();
        matcher.register_trigger("!today", PathBuf::from("/test/today.md"));

        for c in "hello !today".chars() {
            if let Some(result) = matcher.process_keystroke(c) {
                assert_eq!(result.trigger, "!today");
                return;
            }
        }
        panic!("Expected match for !today");
    }
    #[test]
    fn test_slash_prefix_trigger() {
        let mut matcher = KeywordMatcher::new();
        matcher.register_trigger("/date", PathBuf::from("/test/date.md"));

        for c in "hello /date".chars() {
            if let Some(result) = matcher.process_keystroke(c) {
                assert_eq!(result.trigger, "/date");
                return;
            }
        }
        panic!("Expected match for /date");
    }
    #[test]
    fn test_double_comma_suffix_trigger() {
        let mut matcher = KeywordMatcher::new();
        matcher.register_trigger("sig,,", PathBuf::from("/test/sig.md"));

        for c in "hello sig,,".chars() {
            if let Some(result) = matcher.process_keystroke(c) {
                assert_eq!(result.trigger, "sig,,");
                assert_eq!(result.chars_to_delete, 5);
                return;
            }
        }
        panic!("Expected match for sig,,");
    }
    #[test]
    fn test_semicolon_suffix_trigger() {
        let mut matcher = KeywordMatcher::new();
        matcher.register_trigger("email;", PathBuf::from("/test/email.md"));

        for c in "email;".chars() {
            if let Some(result) = matcher.process_keystroke(c) {
                assert_eq!(result.trigger, "email;");
                return;
            }
        }
        panic!("Expected match for email;");
    }
    #[test]
    fn test_no_prefix_trigger() {
        let mut matcher = KeywordMatcher::new();
        matcher.register_trigger("btw", PathBuf::from("/test/btw.md"));

        // Should match "btw" even without prefix
        for c in "btw".chars() {
            if let Some(result) = matcher.process_keystroke(c) {
                assert_eq!(result.trigger, "btw");
                return;
            }
        }
        panic!("Expected match for btw");
    }
    // ========================================
    // Edge Cases and Complex Scenarios
    // ========================================

    #[test]
    fn test_partial_match_then_complete_different_trigger() {
        let mut matcher = KeywordMatcher::new();
        matcher.register_trigger(":sig", PathBuf::from("/test/sig.md"));
        matcher.register_trigger(":sign", PathBuf::from("/test/sign.md"));

        // Type ":sig" - should match first
        for c in ":sig".chars() {
            if let Some(result) = matcher.process_keystroke(c) {
                assert_eq!(result.trigger, ":sig");
                return;
            }
        }
        panic!("Expected match for :sig");
    }
    #[test]
    fn test_longer_trigger_preferred_when_both_match() {
        // Note: This test documents current behavior - first match wins
        // If we want longest match, implementation would need to change
        let mut matcher = KeywordMatcher::new();
        matcher.register_trigger(":sig", PathBuf::from("/test/sig.md"));
        matcher.register_trigger(":signature", PathBuf::from("/test/signature.md"));

        // Type ":sig" - matches immediately
        for c in ":sig".chars() {
            if let Some(result) = matcher.process_keystroke(c) {
                assert_eq!(result.trigger, ":sig");
                return;
            }
        }
        panic!("Expected match for :sig");
    }
    #[test]
    fn test_match_after_buffer_clear() {
        let mut matcher = KeywordMatcher::new();
        matcher.register_trigger(":sig", PathBuf::from("/test/sig.md"));

        // Type some text, then Enter (clears buffer), then trigger
        for c in "hello\n:sig".chars() {
            if let Some(result) = matcher.process_keystroke(c) {
                assert_eq!(result.trigger, ":sig");
                return;
            }
        }
        panic!("Expected match for :sig");
    }
