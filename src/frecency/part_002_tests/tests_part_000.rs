    use super::*;
    use std::fs;
    // Helper to create a test store with a temp file
    fn create_test_store() -> (FrecencyStore, PathBuf) {
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join(format!("frecency_test_{}.json", uuid::Uuid::new_v4()));
        let store = FrecencyStore::with_path(temp_path.clone());
        (store, temp_path)
    }
    // Cleanup helper
    fn cleanup_temp_file(path: &PathBuf) {
        let _ = fs::remove_file(path);
    }
    #[test]
    fn test_frecency_entry_new() {
        let entry = FrecencyEntry::new();
        assert_eq!(entry.count, 1);
        assert!(entry.last_used > 0);
        assert!(entry.score > 0.0);
    }
    #[test]
    fn test_frecency_entry_record_use() {
        let mut entry = FrecencyEntry::new();
        let initial_count = entry.count;
        let initial_last_used = entry.last_used;

        // Small delay to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(10));

        entry.record_use();

        assert_eq!(entry.count, initial_count + 1);
        assert!(entry.last_used >= initial_last_used);
    }
    #[test]
    fn test_calculate_score_no_decay() {
        // Score right now should be close to count
        let now = current_timestamp();
        let score = calculate_score(5, now, HALF_LIFE_DAYS);

        // Should be approximately 5 (allowing for tiny time difference)
        assert!((score - 5.0).abs() < 0.01);
    }
    #[test]
    fn test_calculate_score_with_decay_true_half_life() {
        let now = current_timestamp();
        let count = 10;

        // One half-life ago (7 days)
        let one_half_life_ago = now - (HALF_LIFE_DAYS * SECONDS_PER_DAY) as u64;
        let score = calculate_score(count, one_half_life_ago, HALF_LIFE_DAYS);

        // With TRUE half-life, score should be exactly count/2 (50% decay at one half-life)
        // Formula: count * 2^(-days/half_life) = count * 2^(-1) = count/2
        let expected = count as f64 * 0.5;
        assert!(
            (score - expected).abs() < 0.01,
            "Expected ~{} (50% of {}), got {} - half-life formula should give 50% decay at half-life",
            expected, count, score
        );
    }
    #[test]
    fn test_calculate_score_two_half_lives() {
        let now = current_timestamp();
        let count = 100;

        // Two half-lives ago (14 days)
        let two_half_lives_ago = now - (2.0 * HALF_LIFE_DAYS * SECONDS_PER_DAY) as u64;
        let score = calculate_score(count, two_half_lives_ago, HALF_LIFE_DAYS);

        // After 2 half-lives, should be 25% (0.5^2 = 0.25)
        let expected = count as f64 * 0.25;
        assert!(
            (score - expected).abs() < 0.1,
            "Expected ~{} (25% of {}), got {} - two half-lives should give 25% remaining",
            expected,
            count,
            score
        );
    }
    #[test]
    fn test_calculate_score_old_item() {
        let now = current_timestamp();
        let count = 100;

        // 30 days ago (about 4.3 half-lives with 7-day half-life)
        let thirty_days_ago = now - (30 * SECONDS_PER_DAY as u64);
        let score = calculate_score(count, thirty_days_ago, HALF_LIFE_DAYS);

        // With true half-life: 100 * 0.5^(30/7) = 100 * 0.5^4.28 ≈ 5.15
        // Should be heavily decayed but still detectable
        let expected = count as f64 * 0.5_f64.powf(30.0 / HALF_LIFE_DAYS);
        assert!(
            (score - expected).abs() < 0.5,
            "Expected ~{:.2}, got {:.2}",
            expected,
            score
        );
        // Verify it's indeed heavily decayed (less than 10% of original)
        assert!(score < 10.0, "Should be heavily decayed, got {}", score);
    }
    #[test]
    fn test_frecency_store_new() {
        let store = FrecencyStore::new();
        assert!(store.is_empty());
        assert!(!store.is_dirty());
    }
    #[test]
    fn test_frecency_store_record_use() {
        let (mut store, _temp) = create_test_store();

        store.record_use("/path/to/script.ts");

        assert_eq!(store.len(), 1);
        assert!(store.is_dirty());
        assert!(store.get_score("/path/to/script.ts") > 0.0);
    }
    #[test]
    fn test_frecency_store_record_use_increments() {
        let (mut store, _temp) = create_test_store();

        store.record_use("/path/to/script.ts");
        let score1 = store.get_score("/path/to/script.ts");

        store.record_use("/path/to/script.ts");
        let score2 = store.get_score("/path/to/script.ts");

        // Second use should have higher score
        assert!(score2 > score1);
    }
    #[test]
    fn test_frecency_store_get_score_unknown() {
        let (store, _temp) = create_test_store();
        assert_eq!(store.get_score("/unknown/script.ts"), 0.0);
    }
    #[test]
    fn test_frecency_store_record_use_respects_configured_half_life() {
        // Create two stores with different half-lives
        let temp_dir = std::env::temp_dir();
        let path1 = temp_dir.join(format!("frecency_test_hl1_{}.json", uuid::Uuid::new_v4()));
        let path2 = temp_dir.join(format!("frecency_test_hl2_{}.json", uuid::Uuid::new_v4()));

        // Store with short half-life (1 day) - scores should decay faster
        let mut store_short = FrecencyStore::with_path(path1.clone());
        store_short.set_half_life_days(1.0);

        // Store with long half-life (30 days) - scores should decay slower
        let mut store_long = FrecencyStore::with_path(path2.clone());
        store_long.set_half_life_days(30.0);

        // Record use on both stores
        store_short.record_use("/test.ts");
        store_long.record_use("/test.ts");

        // Scores should be identical right after use (both just recorded)
        let score_short = store_short.get_score("/test.ts");
        let score_long = store_long.get_score("/test.ts");

        // Both should be approximately 1.0 (count=1, no decay yet)
        assert!(
            (score_short - 1.0).abs() < 0.01,
            "Short half-life store: expected ~1.0, got {}",
            score_short
        );
        assert!(
            (score_long - 1.0).abs() < 0.01,
            "Long half-life store: expected ~1.0, got {}",
            score_long
        );

        cleanup_temp_file(&path1);
        cleanup_temp_file(&path2);
    }
    #[test]
    fn test_frecency_store_record_use_uses_store_half_life_not_default() {
        // This test verifies that record_use() uses the store's configured half-life
        // instead of the DEFAULT_SUGGESTED_HALF_LIFE_DAYS constant
        //
        // With the incremental model: new_score = old_score * decay(elapsed) + 1
        // Different half-lives produce different decay factors for the same elapsed time.
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join(format!("frecency_test_hl_{}.json", uuid::Uuid::new_v4()));

        // Create store with custom half-life (much longer than default 7 days)
        let mut store = FrecencyStore::with_path(path.clone());
        let custom_half_life = 30.0; // 30 days (longer than default 7)
        store.set_half_life_days(custom_half_life);

        // Manually create an entry with an old timestamp (7 days ago) and accumulated score
        let now = current_timestamp();
        let seven_days_ago = now - (7 * SECONDS_PER_DAY as u64);
        let old_entry = FrecencyEntry {
            count: 5,
            last_used: seven_days_ago,
            score: 5.0, // Accumulated score as of 7 days ago
        };
        store.entries.insert("/test.ts".to_string(), old_entry);

        // Now record another use - this should apply decay with custom half-life (30 days)
        store.record_use("/test.ts");

        // Get the entry and verify it was calculated with the custom half-life
        let entry = store.entries.get("/test.ts").expect("Entry should exist");

        // count should now be 6
        assert_eq!(entry.count, 6, "Count should be incremented to 6");

        // With incremental model: new_score = old_score * decay(7 days, half_life=30) + 1
        // decay(7 days, 30) = 2^(-7/30) ≈ 0.85
        // new_score = 5.0 * 0.85 + 1.0 ≈ 5.25
        let decay_factor_30 = 2f64.powf(-7.0 / 30.0); // ≈ 0.85
        let expected_score = 5.0 * decay_factor_30 + 1.0;

        assert!(
            (entry.score - expected_score).abs() < 0.1,
            "Entry score {} should match expected {} using custom half-life {}. \
             With 30-day half-life, 7 days only decays to ~85%. \
             If this is wrong, record_use() isn't using store config.",
            entry.score,
            expected_score,
            custom_half_life
        );

        // With default 7-day half-life, 7 days would decay to 50%:
        // default_score = 5.0 * 0.5 + 1.0 = 3.5
        let decay_factor_7 = 0.5;
        let default_half_life_score = 5.0 * decay_factor_7 + 1.0;

        // Our score should be higher than what we'd get with default half-life
        // because 30-day half-life decays more slowly
        assert!(
            entry.score > default_half_life_score,
            "Score {} with 30-day half-life should be higher than {} with 7-day half-life",
            entry.score,
            default_half_life_score
        );

        cleanup_temp_file(&path);
    }
    #[test]
    fn test_frecency_store_get_recent_items() {
        let (mut store, _temp) = create_test_store();

        // Add items with different use counts
        store.record_use("/a.ts");
        store.record_use("/b.ts");
        store.record_use("/b.ts");
        store.record_use("/c.ts");
        store.record_use("/c.ts");
        store.record_use("/c.ts");

        let recent = store.get_recent_items(2);

        assert_eq!(recent.len(), 2);
        // c.ts should be first (3 uses), b.ts second (2 uses)
        assert_eq!(recent[0].0, "/c.ts");
        assert_eq!(recent[1].0, "/b.ts");
    }
    #[test]
    fn test_frecency_store_get_recent_items_limit() {
        let (mut store, _temp) = create_test_store();

        for i in 0..10 {
            store.record_use(&format!("/script{}.ts", i));
        }

        let recent = store.get_recent_items(5);
        assert_eq!(recent.len(), 5);
    }
    #[test]
    fn test_frecency_store_save_and_load() {
        let (_, path) = create_test_store();

        // Create and populate store
        {
            let mut store = FrecencyStore::with_path(path.clone());
            store.record_use("/script1.ts");
            store.record_use("/script1.ts");
            store.record_use("/script2.ts");
            store.save().unwrap();
        }

        // Load into new store
        {
            let mut store = FrecencyStore::with_path(path.clone());
            store.load().unwrap();

            assert_eq!(store.len(), 2);
            assert!(store.get_score("/script1.ts") > store.get_score("/script2.ts"));
        }

        cleanup_temp_file(&path);
    }
    #[test]
    fn test_frecency_store_load_missing_file() {
        let mut store = FrecencyStore::with_path(PathBuf::from("/nonexistent/path/frecency.json"));
        let result = store.load();
        assert!(result.is_ok());
        assert!(store.is_empty());
    }
    #[test]
    fn test_frecency_store_load_invalid_json() {
        let (_, path) = create_test_store();
        fs::write(&path, "not valid json").unwrap();

        let mut store = FrecencyStore::with_path(path.clone());
        let result = store.load();
        assert!(result.is_err());

        cleanup_temp_file(&path);
    }
    #[test]
    fn test_frecency_store_remove() {
        let (mut store, _temp) = create_test_store();

        store.record_use("/script.ts");
        assert_eq!(store.len(), 1);

        let removed = store.remove("/script.ts");
        assert!(removed.is_some());
        assert!(store.is_empty());
        assert!(store.is_dirty());
    }
    #[test]
    fn test_frecency_store_remove_nonexistent() {
        let (mut store, _temp) = create_test_store();

        let removed = store.remove("/nonexistent.ts");
        assert!(removed.is_none());
    }
    #[test]
    fn test_frecency_store_clear() {
        let (mut store, _temp) = create_test_store();

        store.record_use("/a.ts");
        store.record_use("/b.ts");
        store.dirty = false; // Reset dirty flag

        store.clear();

        assert!(store.is_empty());
        assert!(store.is_dirty());
    }
    #[test]
    fn test_frecency_store_save_not_dirty() {
        let (mut store, _temp) = create_test_store();

        // Save without changes should succeed without writing
        let result = store.save();
        assert!(result.is_ok());
    }
    #[test]
    fn test_frecency_entry_serialization() {
        // score is NOT serialized (it's derived on load), so we only verify
        // that count and last_used round-trip correctly
        let entry = FrecencyEntry {
            count: 5,
            last_used: 1704067200, // 2024-01-01 00:00:00 UTC
            score: 4.5,            // This will NOT be serialized
        };

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: FrecencyEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(entry.count, deserialized.count);
        assert_eq!(entry.last_used, deserialized.last_used);
        // score defaults to 0.0 since it's not serialized
        assert_eq!(
            deserialized.score, 0.0,
            "Score should default to 0.0 when deserializing"
        );
    }
    #[test]
    fn test_frecency_entry_deserialization_without_score() {
        // Score was added later, so old data might not have it
        let json = r#"{"count": 5, "last_used": 1704067200}"#;
        let entry: FrecencyEntry = serde_json::from_str(json).unwrap();

        assert_eq!(entry.count, 5);
        assert_eq!(entry.last_used, 1704067200);
        assert_eq!(entry.score, 0.0); // Default
    }
    #[test]
    fn test_frecency_store_recalculates_scores_on_load() {
        let (_, path) = create_test_store();

        // Write data with stale scores
        let old_data =
            r#"{"entries": {"/script.ts": {"count": 10, "last_used": 0, "score": 100.0}}}"#;
        fs::write(&path, old_data).unwrap();

        let mut store = FrecencyStore::with_path(path.clone());
        store.load().unwrap();

        // Score should be recalculated (timestamp 0 is very old, so score should be tiny)
        let score = store.get_score("/script.ts");
        assert!(score < 1.0); // Should be heavily decayed, not the stale 100.0

        cleanup_temp_file(&path);
    }
    #[test]
    fn test_half_life_constant() {
        assert_eq!(HALF_LIFE_DAYS, 7.0);
    }
    #[test]
    fn test_seconds_per_day_constant() {
        assert_eq!(SECONDS_PER_DAY, 86400.0);
    }
