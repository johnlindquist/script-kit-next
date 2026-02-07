    #[test]
    fn test_is_app_bundle_invalid() {
        use std::path::Path;

        // Non-.app files should not be recognized
        let not_app = Path::new("/Applications/readme.txt");
        assert!(!is_app_bundle(not_app));

        let dmg_file = Path::new("/Applications/installer.dmg");
        assert!(!is_app_bundle(dmg_file));

        let ds_store = Path::new("/Applications/.DS_Store");
        assert!(!is_app_bundle(ds_store));

        let hidden = Path::new("/Applications/.Trash");
        assert!(!is_app_bundle(hidden));
    }
    #[test]
    fn test_merge_app_event_remove_then_add() {
        // AppRemoved + AppAdded (same path) → AppUpdated
        let mut pending: HashMap<PathBuf, (AppReloadEvent, Instant)> = HashMap::new();
        let path = PathBuf::from("/Applications/MyApp.app");
        let now = Instant::now();

        // First: remove event
        merge_app_event(
            &mut pending,
            &path,
            AppReloadEvent::AppRemoved(path.clone()),
            now,
        );
        assert_eq!(pending.len(), 1);
        assert!(matches!(
            pending.get(&path),
            Some((AppReloadEvent::AppRemoved(_), _))
        ));

        // Then: add event (app reinstalled/updated)
        let later = now + Duration::from_millis(10);
        merge_app_event(
            &mut pending,
            &path,
            AppReloadEvent::AppAdded(path.clone()),
            later,
        );

        // Should be merged to AppUpdated
        assert_eq!(pending.len(), 1);
        let (event, _) = pending.get(&path).unwrap();
        assert_eq!(*event, AppReloadEvent::AppUpdated(path.clone()));
    }
    #[test]
    fn test_merge_app_event_add_then_remove() {
        // AppAdded + AppRemoved (same path) → AppUpdated
        let mut pending: HashMap<PathBuf, (AppReloadEvent, Instant)> = HashMap::new();
        let path = PathBuf::from("/Applications/MyApp.app");
        let now = Instant::now();

        // First: add event
        merge_app_event(
            &mut pending,
            &path,
            AppReloadEvent::AppAdded(path.clone()),
            now,
        );

        // Then: remove event
        let later = now + Duration::from_millis(10);
        merge_app_event(
            &mut pending,
            &path,
            AppReloadEvent::AppRemoved(path.clone()),
            later,
        );

        // Should be merged to AppUpdated
        assert_eq!(pending.len(), 1);
        let (event, _) = pending.get(&path).unwrap();
        assert_eq!(*event, AppReloadEvent::AppUpdated(path.clone()));
    }
    #[test]
    fn test_no_merge_app_events_different_paths() {
        // Events for different paths should not be merged
        let mut pending: HashMap<PathBuf, (AppReloadEvent, Instant)> = HashMap::new();
        let path_a = PathBuf::from("/Applications/AppA.app");
        let path_b = PathBuf::from("/Applications/AppB.app");
        let now = Instant::now();

        // Remove on path A
        merge_app_event(
            &mut pending,
            &path_a,
            AppReloadEvent::AppRemoved(path_a.clone()),
            now,
        );

        // Add on path B (different path - no merge)
        merge_app_event(
            &mut pending,
            &path_b,
            AppReloadEvent::AppAdded(path_b.clone()),
            now,
        );

        // Should have 2 separate events
        assert_eq!(pending.len(), 2);
        assert!(matches!(
            pending.get(&path_a),
            Some((AppReloadEvent::AppRemoved(_), _))
        ));
        assert!(matches!(
            pending.get(&path_b),
            Some((AppReloadEvent::AppAdded(_), _))
        ));
    }
    #[test]
    fn test_next_app_deadline_empty() {
        let pending: HashMap<PathBuf, (AppReloadEvent, Instant)> = HashMap::new();
        let debounce = Duration::from_millis(500);

        assert!(next_app_deadline(&pending, None, debounce).is_none());
    }
    #[test]
    fn test_next_app_deadline_single() {
        let mut pending: HashMap<PathBuf, (AppReloadEvent, Instant)> = HashMap::new();
        let now = Instant::now();
        let debounce = Duration::from_millis(500);

        pending.insert(
            PathBuf::from("/Applications/Test.app"),
            (
                AppReloadEvent::AppAdded(PathBuf::from("/Applications/Test.app")),
                now,
            ),
        );

        let deadline = next_app_deadline(&pending, None, debounce);
        assert!(deadline.is_some());
        let expected = now + debounce;
        let actual = deadline.unwrap();
        assert!(actual >= expected && actual <= expected + Duration::from_millis(1));
    }
    #[test]
    fn test_next_app_deadline_with_full_reload() {
        let pending: HashMap<PathBuf, (AppReloadEvent, Instant)> = HashMap::new();
        let now = Instant::now();
        let debounce = Duration::from_millis(500);

        let deadline = next_app_deadline(&pending, Some(now), debounce);
        assert!(deadline.is_some());
        let expected = now + debounce;
        let actual = deadline.unwrap();
        assert!(actual >= expected && actual <= expected + Duration::from_millis(1));
    }
    #[test]
    fn test_flush_expired_apps_none_expired() {
        let (tx, _rx) = async_channel::bounded::<AppReloadEvent>(10);
        let mut pending: HashMap<PathBuf, (AppReloadEvent, Instant)> = HashMap::new();
        let mut full_reload_at: Option<Instant> = None;
        let now = Instant::now();
        let debounce = Duration::from_millis(500);

        // Add a fresh event (not expired)
        pending.insert(
            PathBuf::from("/Applications/Test.app"),
            (
                AppReloadEvent::AppAdded(PathBuf::from("/Applications/Test.app")),
                now,
            ),
        );

        flush_expired_apps(&mut pending, &mut full_reload_at, debounce, &tx);

        // Event should still be pending
        assert_eq!(pending.len(), 1);
    }
    #[test]
    fn test_flush_expired_apps_some_expired() {
        let (tx, rx) = async_channel::bounded::<AppReloadEvent>(10);
        let mut pending: HashMap<PathBuf, (AppReloadEvent, Instant)> = HashMap::new();
        let mut full_reload_at: Option<Instant> = None;
        let debounce = Duration::from_millis(500);

        // Add an expired event (from 600ms ago)
        let old_time = Instant::now() - Duration::from_millis(600);
        pending.insert(
            PathBuf::from("/Applications/Old.app"),
            (
                AppReloadEvent::AppAdded(PathBuf::from("/Applications/Old.app")),
                old_time,
            ),
        );

        // Add a fresh event
        pending.insert(
            PathBuf::from("/Applications/New.app"),
            (
                AppReloadEvent::AppAdded(PathBuf::from("/Applications/New.app")),
                Instant::now(),
            ),
        );

        flush_expired_apps(&mut pending, &mut full_reload_at, debounce, &tx);

        // Only fresh event should remain
        assert_eq!(pending.len(), 1);
        assert!(pending.contains_key(&PathBuf::from("/Applications/New.app")));

        // Should have received the expired event
        let received = rx.try_recv().unwrap();
        assert_eq!(
            received,
            AppReloadEvent::AppAdded(PathBuf::from("/Applications/Old.app"))
        );
    }
    #[test]
    fn test_flush_expired_apps_full_reload_supersedes_pending() {
        let (tx, rx) = async_channel::bounded::<AppReloadEvent>(10);
        let mut pending: HashMap<PathBuf, (AppReloadEvent, Instant)> = HashMap::new();
        let debounce = Duration::from_millis(500);

        // Add some expired pending events
        let old_time = Instant::now() - Duration::from_millis(600);
        pending.insert(
            PathBuf::from("/Applications/A.app"),
            (
                AppReloadEvent::AppAdded(PathBuf::from("/Applications/A.app")),
                old_time,
            ),
        );
        pending.insert(
            PathBuf::from("/Applications/B.app"),
            (
                AppReloadEvent::AppRemoved(PathBuf::from("/Applications/B.app")),
                old_time,
            ),
        );

        // Set expired full_reload_at (should supersede pending)
        let mut full_reload_at: Option<Instant> = Some(old_time);

        flush_expired_apps(&mut pending, &mut full_reload_at, debounce, &tx);

        // All pending should be cleared
        assert!(pending.is_empty());
        // full_reload_at should be reset
        assert!(full_reload_at.is_none());

        // Should receive only ONE FullReload (not per-app events)
        let received = rx.try_recv().unwrap();
        assert_eq!(received, AppReloadEvent::FullReload);
        // No more events
        assert!(rx.try_recv().is_err());
    }
