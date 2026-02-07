    // -------------------------------------------------------------------------
    // API improvement tests: upsert_file returning diff (TDD)
    // -------------------------------------------------------------------------

    #[test]
    fn test_upsert_file_returns_diff_for_new_file() {
        let mut cache = ScriptletCache::new();
        let path = PathBuf::from("/path/to/file.md");
        let fp = FileFingerprint {
            mtime: test_mtime(1000),
            size: 1024,
        };

        let scriptlets = vec![CachedScriptlet::new(
            "New Snippet",
            Some("cmd+1".to_string()),
            None,
            None,
            "/path/to/file.md#new-snippet",
        )];

        let diff = cache.upsert_file(path.clone(), fp, scriptlets);

        // New file means all scriptlets are "added"
        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.added[0].name, "New Snippet");
        assert!(diff.removed.is_empty());
        assert!(diff.shortcut_changes.is_empty());

        // File should now be in cache
        assert_eq!(cache.len(), 1);
    }
    #[test]
    fn test_upsert_file_returns_diff_for_existing_file() {
        let mut cache = ScriptletCache::new();
        let path = PathBuf::from("/path/to/file.md");
        let fp1 = FileFingerprint {
            mtime: test_mtime(1000),
            size: 1024,
        };
        let fp2 = FileFingerprint {
            mtime: test_mtime(2000),
            size: 2048,
        };

        // Initial insert
        let initial = vec![
            CachedScriptlet::new(
                "Snippet A",
                Some("cmd+1".to_string()),
                None,
                None,
                "/path/to/file.md#snippet-a",
            ),
            CachedScriptlet::new(
                "Snippet B",
                Some("cmd+2".to_string()),
                None,
                None,
                "/path/to/file.md#snippet-b",
            ),
        ];
        cache.upsert_file(path.clone(), fp1, initial);

        // Update: change shortcut, remove B, add C
        let updated = vec![
            CachedScriptlet::new(
                "Snippet A",
                Some("cmd+9".to_string()), // Changed shortcut
                None,
                None,
                "/path/to/file.md#snippet-a",
            ),
            CachedScriptlet::new(
                "Snippet C",
                Some("cmd+3".to_string()),
                None,
                None,
                "/path/to/file.md#snippet-c",
            ),
        ];

        let diff = cache.upsert_file(path.clone(), fp2, updated);

        // Verify diff captures all changes
        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.added[0].name, "Snippet C");

        assert_eq!(diff.removed.len(), 1);
        assert_eq!(diff.removed[0].name, "Snippet B");

        assert_eq!(diff.shortcut_changes.len(), 1);
        assert_eq!(diff.shortcut_changes[0].name, "Snippet A");
        assert_eq!(diff.shortcut_changes[0].old, Some("cmd+1".to_string()));
        assert_eq!(diff.shortcut_changes[0].new, Some("cmd+9".to_string()));

        // Cache should have updated content
        let cached = cache.get_scriptlets(&path).unwrap();
        assert_eq!(cached.len(), 2);
    }
    #[test]
    fn test_remove_file_returns_removed_scriptlets() {
        let mut cache = ScriptletCache::new();
        let path = PathBuf::from("/path/to/file.md");
        let fp = FileFingerprint {
            mtime: test_mtime(1000),
            size: 1024,
        };

        let scriptlets = vec![
            CachedScriptlet::new("A", None, None, None, "/path/to/file.md#a"),
            CachedScriptlet::new("B", None, None, None, "/path/to/file.md#b"),
        ];
        cache.upsert_file(path.clone(), fp, scriptlets);

        // Remove returns the scriptlets that were removed (for unregistration)
        let removed = cache.remove_file_with_scriptlets(&path);
        assert!(removed.is_some());
        let removed = removed.unwrap();
        assert_eq!(removed.len(), 2);
        assert!(removed.iter().any(|s| s.name == "A"));
        assert!(removed.iter().any(|s| s.name == "B"));

        // Cache should be empty
        assert!(cache.is_empty());
    }
    // -------------------------------------------------------------------------
    // Zero-copy API tests (TDD)
    // -------------------------------------------------------------------------

    #[test]
    fn test_get_scriptlets_ref_returns_slice() {
        let mut cache = ScriptletCache::new();
        let path = PathBuf::from("/path/to/file.md");
        let fp = FileFingerprint {
            mtime: test_mtime(1000),
            size: 1024,
        };

        let scriptlets = vec![
            CachedScriptlet::new("A", None, None, None, "/path/to/file.md#a"),
            CachedScriptlet::new("B", None, None, None, "/path/to/file.md#b"),
        ];
        cache.upsert_file(path.clone(), fp, scriptlets);

        // Zero-copy API should return a reference to the slice
        let slice = cache.get_scriptlets_ref(&path);
        assert!(slice.is_some());
        let slice = slice.unwrap();
        assert_eq!(slice.len(), 2);
        assert_eq!(slice[0].name, "A");
        assert_eq!(slice[1].name, "B");
    }
    #[test]
    fn test_get_scriptlets_ref_returns_none_for_missing() {
        let cache = ScriptletCache::new();
        let path = PathBuf::from("/nonexistent.md");

        let slice = cache.get_scriptlets_ref(&path);
        assert!(slice.is_none());
    }
    // -------------------------------------------------------------------------
    // Path normalization tests (TDD)
    // -------------------------------------------------------------------------

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "ScriptletCache expects absolute paths")]
    fn test_update_file_rejects_relative_path_in_debug() {
        let mut cache = ScriptletCache::new();
        let relative_path = PathBuf::from("relative/path/file.md");
        let fp = FileFingerprint {
            mtime: test_mtime(1000),
            size: 1024,
        };

        // Should panic in debug mode when given a relative path
        cache.upsert_file(relative_path, fp, vec![]);
    }
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "ScriptletCache expects absolute paths")]
    fn test_is_stale_rejects_relative_path_in_debug() {
        let cache = ScriptletCache::new();
        let relative_path = PathBuf::from("relative/path/file.md");
        let fp = FileFingerprint {
            mtime: test_mtime(1000),
            size: 1024,
        };

        // Should panic in debug mode when given a relative path
        let _ = cache.is_stale_fingerprint(&relative_path, fp);
    }
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "ScriptletCache expects absolute paths")]
    fn test_get_scriptlets_ref_rejects_relative_path_in_debug() {
        let cache = ScriptletCache::new();
        let relative_path = PathBuf::from("relative/path/file.md");

        // Should panic in debug mode when given a relative path
        let _ = cache.get_scriptlets_ref(&relative_path);
    }
