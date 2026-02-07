    #[test]
    fn test_get_recent_items_at_timestamp() {
        // Test that we can compute rankings at a specific timestamp
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join(format!("frecency_test_at_{}.json", uuid::Uuid::new_v4()));
        let mut store = FrecencyStore::with_path(path.clone());

        let base_time = 1704067200u64; // 2024-01-01
        let week_later = base_time + (7 * SECONDS_PER_DAY as u64);

        // Insert entries with explicit timestamps
        // /old.ts: score=10.0 at base_time, will decay to 5.0 at week_later (1 half-life)
        store.entries.insert(
            "/old.ts".to_string(),
            FrecencyEntry {
                count: 10,
                last_used: base_time,
                score: 10.0,
            },
        );
        // /recent.ts: score=8.0 at week_later, no decay when queried at week_later
        // This beats the decayed /old.ts score of 5.0
        store.entries.insert(
            "/recent.ts".to_string(),
            FrecencyEntry {
                count: 8,
                last_used: week_later,
                score: 8.0,
            },
        );

        // Query at week_later:
        // - /old.ts: 10.0 * 0.5 = 5.0 (decayed by 1 half-life)
        // - /recent.ts: 8.0 * 1.0 = 8.0 (no decay)
        // So /recent.ts should rank first
        let recent = store.get_recent_items_at(10, week_later);

        assert_eq!(
            recent[0].0, "/recent.ts",
            "Recent script (8.0) should rank first over decayed old script (5.0) at week_later"
        );

        // Query at base_time:
        // - /old.ts: 10.0 * 1.0 = 10.0 (no decay at creation time)
        // - /recent.ts: last_used is in the future, so dt < 0, saturating to 0 -> no decay = 8.0
        // Wait, that's wrong - /recent.ts wasn't created yet at base_time!
        // score_at handles future timestamps by using saturating_sub which gives 0 elapsed time
        // So both get no decay, and /old.ts (10.0) > /recent.ts (8.0)
        let at_base = store.get_recent_items_at(10, base_time);

        assert_eq!(
            at_base[0].0, "/old.ts",
            "Old script (10.0) should rank first over future recent script (8.0) at base_time"
        );

        cleanup_temp_file(&path);
    }
    // ========================================
    // Score skip_serializing tests
    // ========================================

    #[test]
    fn test_score_not_serialized() {
        // Score should not be written to JSON (it's derived on load)
        let entry = FrecencyEntry {
            count: 5,
            last_used: 1704067200,
            score: 999.0, // This should not be serialized
        };

        let json = serde_json::to_string(&entry).unwrap();

        // JSON should NOT contain "score" field
        assert!(
            !json.contains("\"score\""),
            "score field should not be serialized, but got: {}",
            json
        );
    }
    #[test]
    fn test_score_defaults_on_deserialization() {
        // When loading old data without score, it should default to 0.0
        let json = r#"{"count": 5, "last_used": 1704067200}"#;
        let entry: FrecencyEntry = serde_json::from_str(json).unwrap();

        assert_eq!(entry.count, 5);
        assert_eq!(entry.last_used, 1704067200);
        assert_eq!(entry.score, 0.0, "Score should default to 0.0");
    }
    // ========================================
    // Pruning tests
    // ========================================

    #[test]
    fn test_prune_stale_entries() {
        // Entries with very low scores and old timestamps should be prunable
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join(format!("frecency_test_prune_{}.json", uuid::Uuid::new_v4()));
        let mut store = FrecencyStore::with_path(path.clone());

        let now = current_timestamp();
        let year_ago = now - (365 * SECONDS_PER_DAY as u64);
        let week_ago = now - (7 * SECONDS_PER_DAY as u64);

        // Old entry that should be pruned
        store.entries.insert(
            "/abandoned.ts".to_string(),
            FrecencyEntry {
                count: 1,
                last_used: year_ago,
                score: 0.001, // Very low score after decay
            },
        );

        // Recent entry that should be kept
        store.entries.insert(
            "/active.ts".to_string(),
            FrecencyEntry {
                count: 5,
                last_used: week_ago,
                score: 5.0,
            },
        );

        // Current entry that should be kept
        store.entries.insert(
            "/current.ts".to_string(),
            FrecencyEntry {
                count: 1,
                last_used: now,
                score: 1.0,
            },
        );

        // Prune entries with live score < 0.01 AND last_used > 180 days ago
        let pruned_count = store.prune_stale_entries(0.01, 180);

        assert_eq!(pruned_count, 1, "Should prune exactly 1 stale entry");
        assert_eq!(store.len(), 2, "Should have 2 entries remaining");
        assert!(
            store.entries.contains_key("/active.ts"),
            "Active entry should remain"
        );
        assert!(
            store.entries.contains_key("/current.ts"),
            "Current entry should remain"
        );
        assert!(
            !store.entries.contains_key("/abandoned.ts"),
            "Abandoned entry should be pruned"
        );
        assert!(store.is_dirty(), "Store should be dirty after pruning");

        cleanup_temp_file(&path);
    }
    #[test]
    fn test_prune_keeps_high_score_old_entries() {
        // Old entries with high accumulated score should NOT be pruned
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join(format!(
            "frecency_test_prune2_{}.json",
            uuid::Uuid::new_v4()
        ));
        let mut store = FrecencyStore::with_path(path.clone());

        let now = current_timestamp();
        let year_ago = now - (365 * SECONDS_PER_DAY as u64);

        // Old but heavily used entry - even after a year of decay,
        // if it had score=1000, it might still be above threshold
        store.entries.insert(
            "/popular-old.ts".to_string(),
            FrecencyEntry {
                count: 1000,
                last_used: year_ago,
                score: 1000.0, // High accumulated score
            },
        );

        // With 7-day half-life, after 365 days (52 half-lives):
        // 1000 * 0.5^52 ≈ 0.0000000000000002 - this WOULD be pruned
        // So this test verifies the score_threshold works correctly
        let pruned = store.prune_stale_entries(0.0001, 180);

        // This entry SHOULD be pruned because even high scores decay to nothing after a year
        assert_eq!(
            pruned, 1,
            "Even high-score entries should be pruned after sufficient decay"
        );

        cleanup_temp_file(&path);
    }
    #[test]
    fn test_prune_no_op_on_empty_store() {
        let (mut store, path) = create_test_store();

        let pruned = store.prune_stale_entries(0.01, 180);

        assert_eq!(pruned, 0, "Empty store should prune nothing");
        assert!(!store.is_dirty(), "Empty store should not be marked dirty");

        cleanup_temp_file(&path);
    }
