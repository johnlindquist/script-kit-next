    use super::*;
    use std::time::Duration;
    // Helper to create a test mtime
    fn test_mtime(secs: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(secs)
    }
    // -------------------------------------------------------------------------
    // Cache tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_cache_add_and_retrieve() {
        let mut cache = ScriptletCache::new();
        let mtime = test_mtime(1000);
        let path = PathBuf::from("/path/to/file.md");

        let scriptlets = vec![
            CachedScriptlet::new(
                "Snippet One",
                Some("cmd+shift+1".to_string()),
                None,
                None,
                "/path/to/file.md#snippet-one",
            ),
            CachedScriptlet::new(
                "Snippet Two",
                None,
                Some("snip,,".to_string()),
                None,
                "/path/to/file.md#snippet-two",
            ),
        ];

        cache.update_file(&path, mtime, scriptlets.clone());

        // Verify file is in cache
        assert_eq!(cache.len(), 1);
        assert!(!cache.is_empty());

        // Retrieve scriptlets
        let retrieved = cache.get_scriptlets(&path).unwrap();
        assert_eq!(retrieved.len(), 2);
        assert_eq!(retrieved[0].name, "Snippet One");
        assert_eq!(retrieved[0].shortcut, Some("cmd+shift+1".to_string()));
        assert_eq!(retrieved[1].name, "Snippet Two");
        assert_eq!(retrieved[1].keyword, Some("snip,,".to_string()));

        // Verify get_file
        let file = cache.get_file(&path).unwrap();
        assert_eq!(file.mtime, mtime);
        assert_eq!(file.path, path);
    }
    #[test]
    fn test_cache_staleness_detection() {
        let mut cache = ScriptletCache::new();
        let path = PathBuf::from("/path/to/file.md");
        let mtime_old = test_mtime(1000);
        let mtime_new = test_mtime(2000);

        // Not in cache = stale
        assert!(cache.is_stale(&path, mtime_old));

        // Add to cache
        cache.update_file(&path, mtime_old, vec![]);

        // Same mtime = not stale
        assert!(!cache.is_stale(&path, mtime_old));

        // Different mtime = stale
        assert!(cache.is_stale(&path, mtime_new));
    }
    #[test]
    fn test_cache_remove_file() {
        let mut cache = ScriptletCache::new();
        let path = PathBuf::from("/path/to/file.md");
        let mtime = test_mtime(1000);

        let scriptlets = vec![CachedScriptlet::new(
            "Test",
            None,
            None,
            None,
            "/path/to/file.md#test",
        )];

        cache.update_file(&path, mtime, scriptlets);
        assert_eq!(cache.len(), 1);

        // Remove file
        let removed = cache.remove_file(&path);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().scriptlets.len(), 1);

        // Verify removed
        assert_eq!(cache.len(), 0);
        assert!(cache.get_scriptlets(&path).is_none());

        // Remove non-existent returns None
        assert!(cache.remove_file(&path).is_none());
    }
    #[test]
    fn test_cache_update_existing() {
        let mut cache = ScriptletCache::new();
        let path = PathBuf::from("/path/to/file.md");
        let mtime1 = test_mtime(1000);
        let mtime2 = test_mtime(2000);

        // Initial add
        let scriptlets1 = vec![CachedScriptlet::new(
            "Original",
            Some("cmd+1".to_string()),
            None,
            None,
            "/path/to/file.md#original",
        )];
        cache.update_file(&path, mtime1, scriptlets1);

        // Update with new data
        let scriptlets2 = vec![
            CachedScriptlet::new(
                "Updated",
                Some("cmd+2".to_string()),
                None,
                None,
                "/path/to/file.md#updated",
            ),
            CachedScriptlet::new(
                "New One",
                None,
                Some("new,,".to_string()),
                None,
                "/path/to/file.md#new-one",
            ),
        ];
        cache.update_file(&path, mtime2, scriptlets2);

        // Verify update
        assert_eq!(cache.len(), 1); // Still one file
        let file = cache.get_file(&path).unwrap();
        assert_eq!(file.mtime, mtime2);
        assert_eq!(file.scriptlets.len(), 2);
        assert_eq!(file.scriptlets[0].name, "Updated");
    }
    #[test]
    fn test_cache_clear() {
        let mut cache = ScriptletCache::new();
        let mtime = test_mtime(1000);

        cache.update_file("/path/a.md", mtime, vec![]);
        cache.update_file("/path/b.md", mtime, vec![]);
        cache.update_file("/path/c.md", mtime, vec![]);

        assert_eq!(cache.len(), 3);

        cache.clear();

        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }
    #[test]
    fn test_cache_file_paths() {
        let mut cache = ScriptletCache::new();
        let mtime = test_mtime(1000);

        cache.update_file("/path/a.md", mtime, vec![]);
        cache.update_file("/path/b.md", mtime, vec![]);

        let paths: Vec<_> = cache.file_paths().collect();
        assert_eq!(paths.len(), 2);
    }
    // -------------------------------------------------------------------------
    // Diff tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_diff_no_changes() {
        let scriptlets = vec![
            CachedScriptlet::new(
                "Snippet One",
                Some("cmd+1".to_string()),
                None,
                Some("s1".to_string()),
                "/path/to/file.md#snippet-one",
            ),
            CachedScriptlet::new(
                "Snippet Two",
                None,
                Some("snip,,".to_string()),
                None,
                "/path/to/file.md#snippet-two",
            ),
        ];

        let diff = diff_scriptlets(&scriptlets, &scriptlets);

        assert!(diff.is_empty());
        assert_eq!(diff.change_count(), 0);
    }
    #[test]
    fn test_diff_scriptlet_added() {
        let old = vec![CachedScriptlet::new(
            "Existing",
            Some("cmd+1".to_string()),
            None,
            None,
            "/path/to/file.md#existing",
        )];

        let new = vec![
            CachedScriptlet::new(
                "Existing",
                Some("cmd+1".to_string()),
                None,
                None,
                "/path/to/file.md#existing",
            ),
            CachedScriptlet::new(
                "New Snippet",
                Some("cmd+2".to_string()),
                Some("new,,".to_string()),
                None,
                "/path/to/file.md#new-snippet",
            ),
        ];

        let diff = diff_scriptlets(&old, &new);

        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.added[0].name, "New Snippet");
        assert_eq!(diff.added[0].shortcut, Some("cmd+2".to_string()));
        assert_eq!(diff.added[0].keyword, Some("new,,".to_string()));
        assert!(diff.removed.is_empty());
        assert!(diff.shortcut_changes.is_empty());
        assert!(diff.keyword_changes.is_empty());
        assert!(diff.alias_changes.is_empty());
    }
    #[test]
    fn test_diff_scriptlet_removed() {
        let old = vec![
            CachedScriptlet::new(
                "Will Stay",
                Some("cmd+1".to_string()),
                None,
                None,
                "/path/to/file.md#will-stay",
            ),
            CachedScriptlet::new(
                "Will Be Removed",
                Some("cmd+2".to_string()),
                None,
                None,
                "/path/to/file.md#will-be-removed",
            ),
        ];

        let new = vec![CachedScriptlet::new(
            "Will Stay",
            Some("cmd+1".to_string()),
            None,
            None,
            "/path/to/file.md#will-stay",
        )];

        let diff = diff_scriptlets(&old, &new);

        assert!(diff.added.is_empty());
        assert_eq!(diff.removed.len(), 1);
        assert_eq!(diff.removed[0].name, "Will Be Removed");
        assert!(diff.shortcut_changes.is_empty());
        assert!(diff.keyword_changes.is_empty());
        assert!(diff.alias_changes.is_empty());
    }
    #[test]
    fn test_diff_shortcut_changed() {
        let old = vec![CachedScriptlet::new(
            "Snippet",
            Some("cmd+1".to_string()),
            None,
            None,
            "/path/to/file.md#snippet",
        )];

        let new = vec![CachedScriptlet::new(
            "Snippet",
            Some("cmd+2".to_string()),
            None,
            None,
            "/path/to/file.md#snippet",
        )];

        let diff = diff_scriptlets(&old, &new);

        assert!(diff.added.is_empty());
        assert!(diff.removed.is_empty());
        assert_eq!(diff.shortcut_changes.len(), 1);
        assert_eq!(diff.shortcut_changes[0].name, "Snippet");
        assert_eq!(diff.shortcut_changes[0].old, Some("cmd+1".to_string()));
        assert_eq!(diff.shortcut_changes[0].new, Some("cmd+2".to_string()));
        assert!(diff.keyword_changes.is_empty());
        assert!(diff.alias_changes.is_empty());
    }
    #[test]
    fn test_diff_shortcut_added() {
        let old = vec![CachedScriptlet::new(
            "Snippet",
            None, // No shortcut initially
            None,
            None,
            "/path/to/file.md#snippet",
        )];

        let new = vec![CachedScriptlet::new(
            "Snippet",
            Some("cmd+1".to_string()), // Shortcut added
            None,
            None,
            "/path/to/file.md#snippet",
        )];

        let diff = diff_scriptlets(&old, &new);

        assert_eq!(diff.shortcut_changes.len(), 1);
        assert_eq!(diff.shortcut_changes[0].old, None);
        assert_eq!(diff.shortcut_changes[0].new, Some("cmd+1".to_string()));
    }
    #[test]
    fn test_diff_shortcut_removed() {
        let old = vec![CachedScriptlet::new(
            "Snippet",
            Some("cmd+1".to_string()), // Has shortcut
            None,
            None,
            "/path/to/file.md#snippet",
        )];

        let new = vec![CachedScriptlet::new(
            "Snippet",
            None, // Shortcut removed
            None,
            None,
            "/path/to/file.md#snippet",
        )];

        let diff = diff_scriptlets(&old, &new);

        assert_eq!(diff.shortcut_changes.len(), 1);
        assert_eq!(diff.shortcut_changes[0].old, Some("cmd+1".to_string()));
        assert_eq!(diff.shortcut_changes[0].new, None);
    }
    #[test]
    fn test_diff_keyword_changed() {
        let old = vec![CachedScriptlet::new(
            "Snippet",
            None,
            Some("old,,".to_string()),
            None,
            "/path/to/file.md#snippet",
        )];

        let new = vec![CachedScriptlet::new(
            "Snippet",
            None,
            Some("new,,".to_string()),
            None,
            "/path/to/file.md#snippet",
        )];

        let diff = diff_scriptlets(&old, &new);

        assert!(diff.added.is_empty());
        assert!(diff.removed.is_empty());
        assert!(diff.shortcut_changes.is_empty());
        assert_eq!(diff.keyword_changes.len(), 1);
        assert_eq!(diff.keyword_changes[0].name, "Snippet");
        assert_eq!(diff.keyword_changes[0].old, Some("old,,".to_string()));
        assert_eq!(diff.keyword_changes[0].new, Some("new,,".to_string()));
        assert!(diff.alias_changes.is_empty());
    }
    #[test]
    fn test_diff_alias_changed() {
        let old = vec![CachedScriptlet::new(
            "Snippet",
            None,
            None,
            Some("old".to_string()),
            "/path/to/file.md#snippet",
        )];

        let new = vec![CachedScriptlet::new(
            "Snippet",
            None,
            None,
            Some("new".to_string()),
            "/path/to/file.md#snippet",
        )];

        let diff = diff_scriptlets(&old, &new);

        assert!(diff.added.is_empty());
        assert!(diff.removed.is_empty());
        assert!(diff.shortcut_changes.is_empty());
        assert!(diff.keyword_changes.is_empty());
        assert_eq!(diff.alias_changes.len(), 1);
        assert_eq!(diff.alias_changes[0].name, "Snippet");
        assert_eq!(diff.alias_changes[0].old, Some("old".to_string()));
        assert_eq!(diff.alias_changes[0].new, Some("new".to_string()));
    }
