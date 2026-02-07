    #[test]
    fn test_no_match_when_trigger_split_by_clear() {
        let mut matcher = KeywordMatcher::new();
        matcher.register_trigger(":sig", PathBuf::from("/test/sig.md"));

        // Type ":si", then Enter (clears buffer), then "g"
        for c in ":si".chars() {
            assert!(matcher.process_keystroke(c).is_none());
        }

        matcher.process_keystroke('\n'); // Clear buffer

        // "g" alone shouldn't match
        assert!(matcher.process_keystroke('g').is_none());
    }
    #[test]
    fn test_trigger_in_middle_of_sentence() {
        let mut matcher = KeywordMatcher::new();
        matcher.register_trigger(":sig", PathBuf::from("/test/sig.md"));

        // Type "Please sign here :sig thanks"
        for c in "Please sign here :sig".chars() {
            if let Some(result) = matcher.process_keystroke(c) {
                assert_eq!(result.trigger, ":sig");
                return;
            }
        }
        panic!("Expected match for :sig");
    }
    #[test]
    fn test_multiple_triggers_same_text() {
        let mut matcher = KeywordMatcher::new();
        matcher.register_trigger(":sig", PathBuf::from("/test/sig.md"));

        // Type ":sig" twice - should match twice
        let mut match_count = 0;

        for c in ":sig :sig".chars() {
            if matcher.process_keystroke(c).is_some() {
                match_count += 1;
            }
        }

        assert_eq!(match_count, 2);
    }
    #[test]
    fn test_trigger_with_numbers() {
        let mut matcher = KeywordMatcher::new();
        matcher.register_trigger(":addr1", PathBuf::from("/test/addr1.md"));

        for c in ":addr1".chars() {
            if let Some(result) = matcher.process_keystroke(c) {
                assert_eq!(result.trigger, ":addr1");
                return;
            }
        }
        panic!("Expected match for :addr1");
    }
    #[test]
    fn test_case_sensitive_triggers() {
        let mut matcher = KeywordMatcher::new();
        matcher.register_trigger(":Sig", PathBuf::from("/test/sig.md"));

        // Lowercase should NOT match
        for c in ":sig".chars() {
            assert!(matcher.process_keystroke(c).is_none());
        }

        matcher.clear_buffer();

        // Correct case should match
        for c in ":Si".chars() {
            assert!(matcher.process_keystroke(c).is_none());
        }

        let result = matcher.process_keystroke('g');
        assert!(result.is_some());
        assert_eq!(result.unwrap().trigger, ":Sig");
    }
    // ========================================
    // Iterator and Inspection Tests
    // ========================================

    #[test]
    fn test_triggers_iterator() {
        let mut matcher = KeywordMatcher::new();
        matcher.register_trigger(":sig", PathBuf::from("/test/sig.md"));
        matcher.register_trigger("!today", PathBuf::from("/test/today.md"));

        let triggers: Vec<_> = matcher.triggers().collect();

        assert_eq!(triggers.len(), 2);
    }
    #[test]
    fn test_has_trigger_returns_true_for_registered() {
        let mut matcher = KeywordMatcher::new();
        matcher.register_trigger(":sig", PathBuf::from("/test/sig.md"));

        assert!(matcher.has_trigger(":sig"));
        assert!(!matcher.has_trigger(":nonexistent"));
    }
    // ========================================
    // Integration-style Tests
    // ========================================

    #[test]
    fn test_realistic_usage_scenario() {
        let mut matcher = KeywordMatcher::new();

        // Register common text expansion triggers
        matcher.register_trigger(":sig", PathBuf::from("/scriptlets/signature.md"));
        matcher.register_trigger(":email", PathBuf::from("/scriptlets/email.md"));
        matcher.register_trigger("!date", PathBuf::from("/scriptlets/date.md"));
        matcher.register_trigger("addr,,", PathBuf::from("/scriptlets/address.md"));

        // Simulate typing an email
        let text = "Dear John,\n\nThank you for your :email regarding the project.\n\nHere is my address: addr,,\n\nBest regards,\n:sig";

        let mut matches = Vec::new();

        for c in text.chars() {
            if let Some(result) = matcher.process_keystroke(c) {
                matches.push(result.trigger.clone());
            }
        }

        // Should have matched :email, addr,,, and :sig
        // Note: \n clears buffer, so triggers after newlines still work
        assert!(matches.contains(&":email".to_string()));
        assert!(matches.contains(&"addr,,".to_string()));
        assert!(matches.contains(&":sig".to_string()));
    }
    #[test]
    fn test_buffer_wrapping_preserves_recent_context() {
        let mut matcher = KeywordMatcher::with_buffer_size(20);
        matcher.register_trigger(":sig", PathBuf::from("/test/sig.md"));

        // Type a lot of text to cause buffer trimming
        let long_text = "This is a very long sentence that will definitely exceed the buffer size ";
        for c in long_text.chars() {
            assert!(matcher.process_keystroke(c).is_none());
        }

        // Now type the trigger - should still match because buffer keeps recent chars
        for c in ":sig".chars() {
            if let Some(result) = matcher.process_keystroke(c) {
                assert_eq!(result.trigger, ":sig");
                return;
            }
        }
        panic!("Expected match for :sig after buffer wrap");
    }
