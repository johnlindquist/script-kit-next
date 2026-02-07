    // ========================================
    // New incremental frecency model tests
    // ========================================

    #[test]
    fn test_score_at_computes_decay_at_query_time() {
        // score_at() should compute the decayed score at query time,
        // not return a stale cached value
        let now = current_timestamp();
        let half_life = 7.0;

        let entry = FrecencyEntry {
            count: 5,
            last_used: now - (7 * SECONDS_PER_DAY as u64), // 7 days ago
            score: 10.0,                                   // stored score (as of last_used)
        };

        // score_at(now) should decay the stored score by elapsed time
        // 7 days = 1 half-life, so score should be ~10.0 * 0.5 = 5.0
        let score = entry.score_at(now, half_life);
        assert!(
            (score - 5.0).abs() < 0.1,
            "Expected ~5.0 (50% of 10.0), got {}",
            score
        );
    }
    #[test]
    fn test_score_at_zero_elapsed_time() {
        // When queried at the exact moment of last_used, no decay
        let now = current_timestamp();

        let entry = FrecencyEntry {
            count: 1,
            last_used: now,
            score: 3.0,
        };

        let score = entry.score_at(now, 7.0);
        assert!(
            (score - 3.0).abs() < 0.01,
            "No decay expected, got {}",
            score
        );
    }
    #[test]
    fn test_record_use_with_timestamp_incremental_model() {
        // Test the new incremental model: score = score*decay(dt) + 1
        let now = current_timestamp();
        let half_life = 7.0;

        let mut entry = FrecencyEntry {
            count: 10,
            last_used: now - (7 * SECONDS_PER_DAY as u64), // 7 days ago
            score: 4.0,                                    // accumulated score as of 7 days ago
        };

        // record_use should:
        // 1. compute current score: 4.0 * 0.5 = 2.0 (one half-life decay)
        // 2. add 1 for new use: 2.0 + 1.0 = 3.0
        entry.record_use_with_timestamp(now, half_life);

        assert_eq!(entry.count, 11);
        assert_eq!(entry.last_used, now);
        assert!(
            (entry.score - 3.0).abs() < 0.1,
            "Expected ~3.0 (2.0 decayed + 1.0 new), got {}",
            entry.score
        );
    }
    #[test]
    fn test_incremental_model_prevents_rich_get_richer() {
        // Scenario: Script A was used 100 times last year (then abandoned)
        // Script B was used 3 times this week
        // Script B should rank higher
        let now = current_timestamp();
        let half_life = 7.0;
        let year_ago = now - (365 * SECONDS_PER_DAY as u64);
        let two_days_ago = now - (2 * SECONDS_PER_DAY as u64);

        // Script A: high historical usage, long ago
        // With incremental model, even if it had score=100, after a year it's nearly 0
        // 365/7 ≈ 52 half-lives, 0.5^52 ≈ 2.2e-16
        let entry_a = FrecencyEntry {
            count: 100,
            last_used: year_ago,
            score: 100.0, // high accumulated score... a year ago
        };

        // Script B: recent usage
        let entry_b = FrecencyEntry {
            count: 3,
            last_used: two_days_ago,
            score: 3.0, // recently accumulated
        };

        let score_a = entry_a.score_at(now, half_life);
        let score_b = entry_b.score_at(now, half_life);

        assert!(
            score_b > score_a,
            "Recent script B (score={}) should rank higher than abandoned A (score={})",
            score_b,
            score_a
        );
    }
    #[test]
    fn test_get_recent_items_uses_live_scores() {
        // get_recent_items() should compute score_at(now) for ranking,
        // not use stale cached scores
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join(format!("frecency_test_live_{}.json", uuid::Uuid::new_v4()));
        let mut store = FrecencyStore::with_path(path.clone());

        let now = current_timestamp();
        let week_ago = now - (7 * SECONDS_PER_DAY as u64);

        // Insert entries with explicit timestamps via entries map
        // Entry A: high stored score but old
        store.entries.insert(
            "/old-popular.ts".to_string(),
            FrecencyEntry {
                count: 50,
                last_used: week_ago,
                score: 10.0, // will decay to ~5.0
            },
        );

        // Entry B: lower stored score but recent
        store.entries.insert(
            "/recent.ts".to_string(),
            FrecencyEntry {
                count: 3,
                last_used: now,
                score: 3.0, // no decay
            },
        );

        let _recent = store.get_recent_items(10);

        // This test is superseded by test_get_recent_items_live_vs_stale
        // which uses more extreme values to clearly demonstrate live computation.
        // This test just verifies no crashes with mixed timestamp entries.
        cleanup_temp_file(&path);
    }
    #[test]
    fn test_get_recent_items_live_vs_stale() {
        // This test verifies that get_recent_items uses live scores
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join(format!("frecency_test_live2_{}.json", uuid::Uuid::new_v4()));
        let mut store = FrecencyStore::with_path(path.clone());

        let now = current_timestamp();
        let month_ago = now - (30 * SECONDS_PER_DAY as u64); // ~4.3 half-lives

        // Entry A: very high stored score but a month old
        // 4.3 half-lives: 0.5^4.3 ≈ 0.05, so 100 * 0.05 = ~5.0 live
        store.entries.insert(
            "/old.ts".to_string(),
            FrecencyEntry {
                count: 100,
                last_used: month_ago,
                score: 100.0, // stale score
            },
        );

        // Entry B: moderate stored score but recent
        store.entries.insert(
            "/recent.ts".to_string(),
            FrecencyEntry {
                count: 8,
                last_used: now,
                score: 8.0, // live score
            },
        );

        let recent = store.get_recent_items(10);

        // With live computation: /recent.ts (8.0) should beat /old.ts (~5.0)
        assert_eq!(
            recent[0].0, "/recent.ts",
            "Recent script should rank first with live score computation. \
             Got {:?}",
            recent
        );

        cleanup_temp_file(&path);
    }
    // ========================================
    // Revision counter tests
    // ========================================

    #[test]
    fn test_revision_increments_on_record_use() {
        let (mut store, path) = create_test_store();
        let initial_rev = store.revision();

        store.record_use("/test.ts");

        assert!(
            store.revision() > initial_rev,
            "Revision should increment after record_use"
        );
        cleanup_temp_file(&path);
    }
    #[test]
    fn test_revision_increments_on_remove() {
        let (mut store, path) = create_test_store();
        store.record_use("/test.ts");
        let rev_after_add = store.revision();

        store.remove("/test.ts");

        assert!(
            store.revision() > rev_after_add,
            "Revision should increment after remove"
        );
        cleanup_temp_file(&path);
    }
    #[test]
    fn test_revision_increments_on_clear() {
        let (mut store, path) = create_test_store();
        store.record_use("/test.ts");
        let rev_after_add = store.revision();

        store.clear();

        assert!(
            store.revision() > rev_after_add,
            "Revision should increment after clear"
        );
        cleanup_temp_file(&path);
    }
    #[test]
    fn test_revision_increments_on_half_life_change() {
        let (mut store, path) = create_test_store();
        let initial_rev = store.revision();

        store.set_half_life_days(14.0);

        assert!(
            store.revision() > initial_rev,
            "Revision should increment after half-life change"
        );
        cleanup_temp_file(&path);
    }
    // ========================================
    // Deterministic tie-breaker tests
    // ========================================

    #[test]
    fn test_tie_breaker_by_last_used() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join(format!("frecency_test_tie_{}.json", uuid::Uuid::new_v4()));
        let mut store = FrecencyStore::with_path(path.clone());

        let now = current_timestamp();

        // Two items with identical scores but different last_used
        store.entries.insert(
            "/older.ts".to_string(),
            FrecencyEntry {
                count: 1,
                last_used: now - 100, // 100 seconds older
                score: 1.0,
            },
        );
        store.entries.insert(
            "/newer.ts".to_string(),
            FrecencyEntry {
                count: 1,
                last_used: now,
                score: 1.0,
            },
        );

        let recent = store.get_recent_items(10);

        // With tie-breaker by last_used desc, newer should be first
        assert_eq!(
            recent[0].0, "/newer.ts",
            "More recent item should win tie-breaker"
        );
        cleanup_temp_file(&path);
    }
    #[test]
    fn test_tie_breaker_by_path() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join(format!("frecency_test_tie2_{}.json", uuid::Uuid::new_v4()));
        let mut store = FrecencyStore::with_path(path.clone());

        let now = current_timestamp();

        // Two items with identical scores AND identical last_used
        store.entries.insert(
            "/bbb.ts".to_string(),
            FrecencyEntry {
                count: 1,
                last_used: now,
                score: 1.0,
            },
        );
        store.entries.insert(
            "/aaa.ts".to_string(),
            FrecencyEntry {
                count: 1,
                last_used: now,
                score: 1.0,
            },
        );

        let recent = store.get_recent_items(10);

        // With tie-breaker by path asc, /aaa.ts should be first
        assert_eq!(
            recent[0].0, "/aaa.ts",
            "Alphabetically first path should win final tie-breaker"
        );
        cleanup_temp_file(&path);
    }
    // ========================================
    // Atomic save tests
    // ========================================

    #[test]
    fn test_save_creates_valid_json() {
        let (mut store, path) = create_test_store();
        store.record_use("/test.ts");
        store.save().unwrap();

        // Verify the file exists and contains valid JSON
        let content = fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(parsed.get("entries").is_some());

        cleanup_temp_file(&path);
    }
    #[test]
    fn test_save_no_temp_file_left_behind() {
        let (mut store, path) = create_test_store();
        store.record_use("/test.ts");
        store.save().unwrap();

        // Verify no temp file is left behind
        let temp_path = path.with_extension("json.tmp");
        assert!(
            !temp_path.exists(),
            "Temp file should be cleaned up after save"
        );

        cleanup_temp_file(&path);
    }
    #[test]
    fn test_save_preserves_data_integrity() {
        let (mut store, path) = create_test_store();

        // Add multiple entries
        store.record_use("/a.ts");
        store.record_use("/a.ts");
        store.record_use("/b.ts");
        store.save().unwrap();

        // Load into new store and verify data
        let mut loaded = FrecencyStore::with_path(path.clone());
        loaded.load().unwrap();

        assert_eq!(loaded.len(), 2);
        assert!(loaded.get_score("/a.ts") > 0.0);
        assert!(loaded.get_score("/b.ts") > 0.0);

        cleanup_temp_file(&path);
    }
    #[test]
    fn test_save_uses_compact_json() {
        // Verify we're not using pretty-print (for performance)
        let (mut store, path) = create_test_store();
        store.record_use("/test.ts");
        store.save().unwrap();

        let content = fs::read_to_string(&path).unwrap();

        // Compact JSON should not have excessive newlines
        // Pretty JSON has newlines after every field
        let newline_count = content.matches('\n').count();
        // Compact JSON has at most a few newlines (maybe 0-2)
        // Pretty JSON with 1 entry would have ~5+ newlines
        assert!(
            newline_count <= 2,
            "Expected compact JSON with few newlines, got {} newlines",
            newline_count
        );

        cleanup_temp_file(&path);
    }
    // ========================================
    // Clock injection tests
    // ========================================

    #[test]
    fn test_record_use_with_injected_timestamp() {
        // Test that we can inject timestamps for deterministic testing
        let (mut store, path) = create_test_store();
        let fixed_time = 1704067200u64; // 2024-01-01 00:00:00 UTC

        store.record_use_at("/test.ts", fixed_time);

        let entry = store.entries.get("/test.ts").expect("Entry should exist");
        assert_eq!(entry.last_used, fixed_time, "Should use injected timestamp");

        cleanup_temp_file(&path);
    }
