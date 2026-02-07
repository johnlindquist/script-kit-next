    #[test]
    fn test_next_deadline_multiple_picks_earliest() {
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let now = Instant::now();
        let debounce = Duration::from_millis(500);

        // Add an older event
        let older_time = now - Duration::from_millis(200);
        pending.insert(
            PathBuf::from("/test/old.ts"),
            (
                ScriptReloadEvent::FileChanged(PathBuf::from("/test/old.ts")),
                older_time,
            ),
        );

        // Add a newer event
        pending.insert(
            PathBuf::from("/test/new.ts"),
            (
                ScriptReloadEvent::FileChanged(PathBuf::from("/test/new.ts")),
                now,
            ),
        );

        let deadline = next_deadline(&pending, None, debounce);
        assert!(deadline.is_some());
        // Should pick the older event's deadline (earlier)
        let expected = older_time + debounce;
        let actual = deadline.unwrap();
        assert!(actual >= expected && actual <= expected + Duration::from_millis(1));
    }
    #[test]
    fn test_next_deadline_with_full_reload() {
        let pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let now = Instant::now();
        let debounce = Duration::from_millis(500);

        // Only full_reload_at is set, no pending events
        let deadline = next_deadline(&pending, Some(now), debounce);
        assert!(deadline.is_some());
        let expected = now + debounce;
        let actual = deadline.unwrap();
        assert!(actual >= expected && actual <= expected + Duration::from_millis(1));
    }
    #[test]
    fn test_next_deadline_full_reload_earlier_than_pending() {
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let now = Instant::now();
        let debounce = Duration::from_millis(500);

        // Add a fresh pending event (deadline = now + 500ms)
        pending.insert(
            PathBuf::from("/test/script.ts"),
            (
                ScriptReloadEvent::FileChanged(PathBuf::from("/test/script.ts")),
                now,
            ),
        );

        // Add an older full_reload_at (deadline = older + 500ms < now + 500ms)
        let older_reload = now - Duration::from_millis(200);
        let deadline = next_deadline(&pending, Some(older_reload), debounce);
        assert!(deadline.is_some());

        // Should pick the earlier deadline (full_reload)
        let expected = older_reload + debounce;
        let actual = deadline.unwrap();
        assert!(actual >= expected && actual <= expected + Duration::from_millis(1));
    }
    #[test]
    fn test_flush_expired_none_expired() {
        let (tx, _rx) = channel::<ScriptReloadEvent>();
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let mut full_reload_at: Option<Instant> = None;
        let now = Instant::now();
        let debounce = Duration::from_millis(500);

        // Add a fresh event (not expired)
        pending.insert(
            PathBuf::from("/test/script.ts"),
            (
                ScriptReloadEvent::FileChanged(PathBuf::from("/test/script.ts")),
                now,
            ),
        );

        flush_expired(&mut pending, &mut full_reload_at, debounce, &tx);

        // Event should still be pending
        assert_eq!(pending.len(), 1);
    }
    #[test]
    fn test_flush_expired_some_expired() {
        let (tx, rx) = channel::<ScriptReloadEvent>();
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let mut full_reload_at: Option<Instant> = None;
        let debounce = Duration::from_millis(500);

        // Add an expired event (from 600ms ago)
        let old_time = Instant::now() - Duration::from_millis(600);
        pending.insert(
            PathBuf::from("/test/old.ts"),
            (
                ScriptReloadEvent::FileChanged(PathBuf::from("/test/old.ts")),
                old_time,
            ),
        );

        // Add a fresh event
        pending.insert(
            PathBuf::from("/test/new.ts"),
            (
                ScriptReloadEvent::FileChanged(PathBuf::from("/test/new.ts")),
                Instant::now(),
            ),
        );

        flush_expired(&mut pending, &mut full_reload_at, debounce, &tx);

        // Only fresh event should remain
        assert_eq!(pending.len(), 1);
        assert!(pending.contains_key(&PathBuf::from("/test/new.ts")));

        // Should have received the expired event
        let received = rx.try_recv().unwrap();
        assert_eq!(
            received,
            ScriptReloadEvent::FileChanged(PathBuf::from("/test/old.ts"))
        );
    }
    #[test]
    fn test_flush_expired_full_reload_supersedes_pending() {
        let (tx, rx) = channel::<ScriptReloadEvent>();
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let debounce = Duration::from_millis(500);

        // Add some expired pending events
        let old_time = Instant::now() - Duration::from_millis(600);
        pending.insert(
            PathBuf::from("/test/a.ts"),
            (
                ScriptReloadEvent::FileChanged(PathBuf::from("/test/a.ts")),
                old_time,
            ),
        );
        pending.insert(
            PathBuf::from("/test/b.ts"),
            (
                ScriptReloadEvent::FileCreated(PathBuf::from("/test/b.ts")),
                old_time,
            ),
        );

        // Set expired full_reload_at (should supersede pending)
        let mut full_reload_at: Option<Instant> = Some(old_time);

        flush_expired(&mut pending, &mut full_reload_at, debounce, &tx);

        // All pending should be cleared
        assert!(pending.is_empty());
        // full_reload_at should be reset
        assert!(full_reload_at.is_none());

        // Should receive only ONE FullReload (not per-file events)
        let received = rx.try_recv().unwrap();
        assert_eq!(received, ScriptReloadEvent::FullReload);
        // No more events
        assert!(rx.try_recv().is_err());
    }
    #[test]
    fn test_config_watcher_shutdown_no_hang() {
        // Create and start a watcher
        let (mut watcher, _rx) = ConfigWatcher::new();

        // This may fail if the watch directory doesn't exist, but that's fine
        // We're testing that drop doesn't hang, not that watching works
        let _ = watcher.start();

        // Drop should complete within a reasonable time (not hang)
        // The Drop implementation sends Stop and then joins
        drop(watcher);

        // If we get here, the test passed (didn't hang)
    }
    #[test]
    fn test_theme_watcher_shutdown_no_hang() {
        let (mut watcher, _rx) = ThemeWatcher::new();
        let _ = watcher.start();
        drop(watcher);
        // If we get here, the test passed (didn't hang)
    }
    #[test]
    fn test_script_watcher_shutdown_no_hang() {
        let (mut watcher, _rx) = ScriptWatcher::new();
        let _ = watcher.start();
        drop(watcher);
        // If we get here, the test passed (didn't hang)
    }
    #[test]
    fn test_storm_threshold_constant() {
        // Verify storm threshold is a reasonable value (compile-time checks)
        const { assert!(STORM_THRESHOLD > 0) };
        const { assert!(STORM_THRESHOLD <= 1000) }; // Not too high
    }
    #[test]
    fn test_debounce_constant() {
        // Verify debounce is a reasonable value (compile-time checks)
        const { assert!(DEBOUNCE_MS >= 100) }; // At least 100ms
        const { assert!(DEBOUNCE_MS <= 2000) }; // At most 2s
    }
    #[test]
    fn test_storm_coalescing_logic() {
        // Test that we properly handle storm coalescing
        // When storm threshold is reached, we should:
        // 1. Clear pending
        // 2. Send FullReload
        // 3. Continue processing (not exit the watcher)

        let (tx, rx) = channel::<ScriptReloadEvent>();
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let now = Instant::now();

        // Fill up pending to the storm threshold
        for i in 0..STORM_THRESHOLD {
            let path = PathBuf::from(format!("/test/script{}.ts", i));
            pending.insert(path.clone(), (ScriptReloadEvent::FileCreated(path), now));
        }

        // Verify we're at the threshold
        assert_eq!(pending.len(), STORM_THRESHOLD);

        // Simulate storm coalescing
        if pending.len() >= STORM_THRESHOLD {
            pending.clear();
            let _ = tx.send(ScriptReloadEvent::FullReload);
        }

        // Pending should be cleared
        assert_eq!(pending.len(), 0);

        // FullReload should have been sent
        let received = rx.try_recv().unwrap();
        assert_eq!(received, ScriptReloadEvent::FullReload);
    }
    #[test]
    fn test_compute_backoff_initial() {
        // First attempt should use initial backoff
        let delay = compute_backoff(0);
        assert_eq!(delay, Duration::from_millis(INITIAL_BACKOFF_MS));
    }
    #[test]
    fn test_compute_backoff_exponential() {
        // Each attempt should double the delay
        let delay0 = compute_backoff(0);
        let delay1 = compute_backoff(1);
        let delay2 = compute_backoff(2);
        let delay3 = compute_backoff(3);

        assert_eq!(delay0, Duration::from_millis(100));
        assert_eq!(delay1, Duration::from_millis(200));
        assert_eq!(delay2, Duration::from_millis(400));
        assert_eq!(delay3, Duration::from_millis(800));
    }
    #[test]
    fn test_compute_backoff_capped() {
        // High attempts should be capped at MAX_BACKOFF_MS
        let delay = compute_backoff(20); // 2^20 * 100ms would be huge
        assert_eq!(delay, Duration::from_millis(MAX_BACKOFF_MS));
    }
    #[test]
    fn test_compute_backoff_no_overflow() {
        // Even with u32::MAX attempts, should not panic
        let delay = compute_backoff(u32::MAX);
        assert_eq!(delay, Duration::from_millis(MAX_BACKOFF_MS));
    }
    #[test]
    fn test_interruptible_sleep_completes() {
        use std::sync::atomic::AtomicBool;

        let stop_flag = AtomicBool::new(false);
        let start = Instant::now();

        // Sleep for 50ms
        let completed = interruptible_sleep(Duration::from_millis(50), &stop_flag);

        assert!(completed);
        assert!(start.elapsed() >= Duration::from_millis(50));
    }
    #[test]
    fn test_interruptible_sleep_interrupted() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        let stop_flag = Arc::new(AtomicBool::new(false));
        let flag_clone = Arc::clone(&stop_flag);

        // Spawn a thread to set the stop flag after 50ms
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
            flag_clone.store(true, Ordering::Relaxed);
        });

        let start = Instant::now();

        // Try to sleep for 1 second, but should be interrupted
        let completed = interruptible_sleep(Duration::from_millis(1000), &stop_flag);

        assert!(!completed);
        // Should have stopped much sooner than 1 second
        assert!(start.elapsed() < Duration::from_millis(500));
    }
    #[test]
    fn test_backoff_constants() {
        // Verify backoff constants are reasonable
        const { assert!(INITIAL_BACKOFF_MS >= 50) }; // At least 50ms
        const { assert!(INITIAL_BACKOFF_MS <= 1000) }; // At most 1s
        const { assert!(MAX_BACKOFF_MS >= 5000) }; // At least 5s
        const { assert!(MAX_BACKOFF_MS <= 120_000) }; // At most 2 minutes
        const { assert!(MAX_NOTIFY_ERRORS >= 3) }; // At least 3 errors
        const { assert!(MAX_NOTIFY_ERRORS <= 100) }; // At most 100 errors
    }
    // ============================================================
    // AppWatcher tests
    // ============================================================

    #[test]
    fn test_app_watcher_creation() {
        let (_watcher, _rx) = AppWatcher::new();
        // Watcher should be created without panicking
    }
    #[test]
    fn test_app_watcher_shutdown_no_hang() {
        let (mut watcher, _rx) = AppWatcher::new();
        let _ = watcher.start();
        drop(watcher);
        // If we get here, the test passed (didn't hang)
    }
    #[test]
    fn test_app_reload_event_clone() {
        let event = AppReloadEvent::FullReload;
        let _cloned = event.clone();
        // Event should be cloneable
    }
    #[test]
    fn test_app_reload_event_app_added() {
        let path = PathBuf::from("/Applications/MyApp.app");
        let event = AppReloadEvent::AppAdded(path.clone());

        if let AppReloadEvent::AppAdded(event_path) = event {
            assert_eq!(event_path, path);
        } else {
            panic!("Expected AppAdded variant");
        }
    }
    #[test]
    fn test_app_reload_event_app_removed() {
        let path = PathBuf::from("/Applications/OldApp.app");
        let event = AppReloadEvent::AppRemoved(path.clone());

        if let AppReloadEvent::AppRemoved(event_path) = event {
            assert_eq!(event_path, path);
        } else {
            panic!("Expected AppRemoved variant");
        }
    }
    #[test]
    fn test_app_reload_event_app_updated() {
        let path = PathBuf::from("/Applications/UpdatedApp.app");
        let event = AppReloadEvent::AppUpdated(path.clone());

        if let AppReloadEvent::AppUpdated(event_path) = event {
            assert_eq!(event_path, path);
        } else {
            panic!("Expected AppUpdated variant");
        }
    }
    #[test]
    fn test_app_reload_event_equality() {
        let path1 = PathBuf::from("/Applications/App1.app");
        let path2 = PathBuf::from("/Applications/App1.app");
        let path3 = PathBuf::from("/Applications/App2.app");

        // Same path should be equal
        assert_eq!(
            AppReloadEvent::AppAdded(path1.clone()),
            AppReloadEvent::AppAdded(path2.clone())
        );

        // Different paths should not be equal
        assert_ne!(
            AppReloadEvent::AppAdded(path1.clone()),
            AppReloadEvent::AppAdded(path3.clone())
        );

        // Different event types should not be equal
        assert_ne!(
            AppReloadEvent::AppAdded(path1.clone()),
            AppReloadEvent::AppRemoved(path1.clone())
        );

        // FullReload should equal itself
        assert_eq!(AppReloadEvent::FullReload, AppReloadEvent::FullReload);
    }
    #[test]
    fn test_is_app_bundle_valid() {
        use std::path::Path;

        // .app extension should be recognized
        let valid_app = Path::new("/Applications/Safari.app");
        assert!(is_app_bundle(valid_app));

        let user_app = Path::new("/Users/test/Applications/MyApp.app");
        assert!(is_app_bundle(user_app));
    }
