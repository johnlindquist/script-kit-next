    #[test]
    fn test_diff_multiple_changes() {
        let old = vec![
            CachedScriptlet::new(
                "Unchanged",
                Some("cmd+1".to_string()),
                None,
                None,
                "/path/to/file.md#unchanged",
            ),
            CachedScriptlet::new(
                "Will Be Removed",
                Some("cmd+2".to_string()),
                None,
                None,
                "/path/to/file.md#will-be-removed",
            ),
            CachedScriptlet::new(
                "Shortcut Changed",
                Some("cmd+3".to_string()),
                None,
                None,
                "/path/to/file.md#shortcut-changed",
            ),
            CachedScriptlet::new(
                "Keyword Changed",
                None,
                Some("old,,".to_string()),
                None,
                "/path/to/file.md#keyword-changed",
            ),
            CachedScriptlet::new(
                "Alias Changed",
                None,
                None,
                Some("oldalias".to_string()),
                "/path/to/file.md#alias-changed",
            ),
        ];

        let new = vec![
            CachedScriptlet::new(
                "Unchanged",
                Some("cmd+1".to_string()),
                None,
                None,
                "/path/to/file.md#unchanged",
            ),
            CachedScriptlet::new(
                "Shortcut Changed",
                Some("cmd+9".to_string()), // Changed
                None,
                None,
                "/path/to/file.md#shortcut-changed",
            ),
            CachedScriptlet::new(
                "Keyword Changed",
                None,
                Some("new,,".to_string()), // Changed
                None,
                "/path/to/file.md#keyword-changed",
            ),
            CachedScriptlet::new(
                "Alias Changed",
                None,
                None,
                Some("newalias".to_string()), // Changed
                "/path/to/file.md#alias-changed",
            ),
            CachedScriptlet::new(
                "New Snippet",
                Some("cmd+0".to_string()),
                None,
                None,
                "/path/to/file.md#new-snippet",
            ),
        ];

        let diff = diff_scriptlets(&old, &new);

        // Verify added
        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.added[0].name, "New Snippet");

        // Verify removed
        assert_eq!(diff.removed.len(), 1);
        assert_eq!(diff.removed[0].name, "Will Be Removed");

        // Verify shortcut change
        assert_eq!(diff.shortcut_changes.len(), 1);
        assert_eq!(diff.shortcut_changes[0].name, "Shortcut Changed");
        assert_eq!(diff.shortcut_changes[0].old, Some("cmd+3".to_string()));
        assert_eq!(diff.shortcut_changes[0].new, Some("cmd+9".to_string()));

        // Verify expand change
        assert_eq!(diff.keyword_changes.len(), 1);
        assert_eq!(diff.keyword_changes[0].name, "Keyword Changed");
        assert_eq!(diff.keyword_changes[0].old, Some("old,,".to_string()));
        assert_eq!(diff.keyword_changes[0].new, Some("new,,".to_string()));

        // Verify alias change
        assert_eq!(diff.alias_changes.len(), 1);
        assert_eq!(diff.alias_changes[0].name, "Alias Changed");
        assert_eq!(diff.alias_changes[0].old, Some("oldalias".to_string()));
        assert_eq!(diff.alias_changes[0].new, Some("newalias".to_string()));

        // Total changes
        assert_eq!(diff.change_count(), 5);
        assert!(!diff.is_empty());
    }
    #[test]
    fn test_diff_empty_to_empty() {
        let diff = diff_scriptlets(&[], &[]);
        assert!(diff.is_empty());
        assert_eq!(diff.change_count(), 0);
    }
    #[test]
    fn test_diff_empty_to_some() {
        let new = vec![CachedScriptlet::new(
            "New",
            Some("cmd+1".to_string()),
            None,
            None,
            "/path/to/file.md#new",
        )];

        let diff = diff_scriptlets(&[], &new);

        assert_eq!(diff.added.len(), 1);
        assert!(diff.removed.is_empty());
        assert_eq!(diff.change_count(), 1);
    }
    #[test]
    fn test_diff_some_to_empty() {
        let old = vec![CachedScriptlet::new(
            "Old",
            Some("cmd+1".to_string()),
            None,
            None,
            "/path/to/file.md#old",
        )];

        let diff = diff_scriptlets(&old, &[]);

        assert!(diff.added.is_empty());
        assert_eq!(diff.removed.len(), 1);
        assert_eq!(diff.change_count(), 1);
    }
    // -------------------------------------------------------------------------
    // Error message formatting tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_format_parse_error_empty() {
        let errors: Vec<ScriptletValidationError> = vec![];
        assert_eq!(super::format_parse_error_message(&errors), "");
    }
    #[test]
    fn test_format_parse_error_single_with_name() {
        let errors = vec![ScriptletValidationError::new(
            "/path/to/snippets.md",
            Some("My Script".to_string()),
            Some(10),
            "Invalid syntax",
        )];

        let msg = super::format_parse_error_message(&errors);
        assert_eq!(msg, "Failed to parse 'My Script' in snippets.md");
    }
    #[test]
    fn test_format_parse_error_single_without_name() {
        let errors = vec![ScriptletValidationError::new(
            "/path/to/snippets.md",
            None,
            Some(5),
            "No code block found",
        )];

        let msg = super::format_parse_error_message(&errors);
        assert_eq!(msg, "Failed to parse scriptlet in snippets.md");
    }
    #[test]
    fn test_format_parse_error_multiple_in_one_file() {
        let errors = vec![
            ScriptletValidationError::new(
                "/path/to/snippets.md",
                Some("Script One".to_string()),
                Some(10),
                "Error 1",
            ),
            ScriptletValidationError::new(
                "/path/to/snippets.md",
                Some("Script Two".to_string()),
                Some(20),
                "Error 2",
            ),
        ];

        let msg = super::format_parse_error_message(&errors);
        assert_eq!(msg, "Failed to parse 2 scriptlet(s) in snippets.md");
    }
    #[test]
    fn test_format_parse_error_multiple_files() {
        let errors = vec![
            ScriptletValidationError::new(
                "/path/to/file1.md",
                Some("Script A".to_string()),
                Some(10),
                "Error A",
            ),
            ScriptletValidationError::new(
                "/path/to/file2.md",
                Some("Script B".to_string()),
                Some(20),
                "Error B",
            ),
            ScriptletValidationError::new(
                "/path/to/file2.md",
                Some("Script C".to_string()),
                Some(30),
                "Error C",
            ),
        ];

        let msg = super::format_parse_error_message(&errors);
        assert_eq!(msg, "Parse errors in 2 file(s) (3 total). Check logs.");
    }
    #[test]
    fn test_get_log_file_path() {
        let path = super::get_log_file_path();
        // Should end with the expected filename
        assert!(path.ends_with("script-kit-gpui.jsonl"));
        // Should contain .scriptkit/logs in the path (or /tmp as fallback)
        let path_str = path.to_string_lossy();
        assert!(
            path_str.contains(".scriptkit/logs") || path_str.contains("/tmp"),
            "Path should be in .scriptkit/logs or /tmp, got: {}",
            path_str
        );
    }
    #[test]
    fn test_create_error_summary_none_for_empty() {
        let errors: Vec<ScriptletValidationError> = vec![];
        assert!(super::create_error_summary(&errors).is_none());
    }
    #[test]
    fn test_create_error_summary_has_fields() {
        let errors = vec![ScriptletValidationError::new(
            "/path/to/file.md",
            Some("Test".to_string()),
            Some(1),
            "Error",
        )];

        let summary = super::create_error_summary(&errors).unwrap();
        assert_eq!(summary.error_count, 1);
        assert!(!summary.hud_message.is_empty());
        assert!(summary.log_file_path.ends_with("script-kit-gpui.jsonl"));
    }
    // -------------------------------------------------------------------------
    // Bug fix tests: file_path change detection (TDD)
    // -------------------------------------------------------------------------

    #[test]
    fn test_diff_file_path_changed_same_name() {
        // BUG: When a scriptlet's anchor/file_path changes but name stays the same,
        // the diff should detect this. Otherwise hotkey registrations point to stale paths.
        let old = vec![CachedScriptlet::new(
            "My Snippet",
            Some("cmd+1".to_string()),
            None,
            None,
            "/path/to/file.md#old-anchor", // Old anchor
        )];

        let new = vec![CachedScriptlet::new(
            "My Snippet",
            Some("cmd+1".to_string()),
            None,
            None,
            "/path/to/file.md#new-anchor", // New anchor - this should be detected!
        )];

        let diff = diff_scriptlets(&old, &new);

        // This is the critical assertion: file_path changes MUST be detected
        assert!(
            !diff.file_path_changes.is_empty(),
            "file_path change should be detected when anchor changes"
        );
        assert_eq!(diff.file_path_changes.len(), 1);
        assert_eq!(diff.file_path_changes[0].name, "My Snippet");
        assert_eq!(diff.file_path_changes[0].old, "/path/to/file.md#old-anchor");
        assert_eq!(diff.file_path_changes[0].new, "/path/to/file.md#new-anchor");
    }
    #[test]
    fn test_diff_file_path_no_change() {
        // When file_path is the same, no file_path_change should be reported
        let old = vec![CachedScriptlet::new(
            "My Snippet",
            Some("cmd+1".to_string()),
            None,
            None,
            "/path/to/file.md#same-anchor",
        )];

        let new = vec![CachedScriptlet::new(
            "My Snippet",
            Some("cmd+2".to_string()), // Shortcut changed, but file_path same
            None,
            None,
            "/path/to/file.md#same-anchor",
        )];

        let diff = diff_scriptlets(&old, &new);

        assert!(
            diff.file_path_changes.is_empty(),
            "No file_path change when paths are identical"
        );
        assert_eq!(diff.shortcut_changes.len(), 1); // But shortcut did change
    }
    #[test]
    fn test_diff_is_empty_includes_file_path_changes() {
        // is_empty() should return false when there are file_path changes
        let old = vec![CachedScriptlet::new(
            "Snippet",
            None,
            None,
            None,
            "/path/to/file.md#old",
        )];

        let new = vec![CachedScriptlet::new(
            "Snippet",
            None,
            None,
            None,
            "/path/to/file.md#new",
        )];

        let diff = diff_scriptlets(&old, &new);

        assert!(
            !diff.is_empty(),
            "Diff with file_path changes should not be empty"
        );
        assert!(
            diff.change_count() > 0,
            "change_count should include file_path changes"
        );
    }
    // -------------------------------------------------------------------------
    // Bug fix tests: FileFingerprint staleness detection (TDD)
    // -------------------------------------------------------------------------

    #[test]
    fn test_fingerprint_equality() {
        let fp1 = FileFingerprint {
            mtime: test_mtime(1000),
            size: 1024,
        };
        let fp2 = FileFingerprint {
            mtime: test_mtime(1000),
            size: 1024,
        };
        let fp3 = FileFingerprint {
            mtime: test_mtime(1000),
            size: 2048, // Different size
        };
        let fp4 = FileFingerprint {
            mtime: test_mtime(2000), // Different mtime
            size: 1024,
        };

        assert_eq!(fp1, fp2, "Same mtime and size should be equal");
        assert_ne!(fp1, fp3, "Different size should not be equal");
        assert_ne!(fp1, fp4, "Different mtime should not be equal");
    }
    #[test]
    fn test_cache_staleness_with_fingerprint() {
        let mut cache = ScriptletCache::new();
        let path = PathBuf::from("/path/to/file.md");
        let fp_old = FileFingerprint {
            mtime: test_mtime(1000),
            size: 1024,
        };
        let fp_same_mtime_diff_size = FileFingerprint {
            mtime: test_mtime(1000),
            size: 2048, // Same mtime but different size
        };

        // Add to cache with fingerprint
        cache.update_file_with_fingerprint(&path, fp_old, vec![]);

        // Same fingerprint = not stale
        assert!(
            !cache.is_stale_fingerprint(&path, fp_old),
            "Same fingerprint should not be stale"
        );

        // Same mtime but different size = stale (this is the bug fix!)
        assert!(
            cache.is_stale_fingerprint(&path, fp_same_mtime_diff_size),
            "Same mtime but different size should be stale"
        );
    }
