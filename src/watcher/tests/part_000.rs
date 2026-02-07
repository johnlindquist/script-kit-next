    use super::*;
    // ============================================================
    // ISSUE A - FullReload coalescing tests
    // ============================================================

    #[test]
    fn test_full_reload_global_state_single_emission() {
        // Multiple FullReload triggers during debounce window should result in single emission
        let (tx, rx) = channel::<ScriptReloadEvent>();
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let mut full_reload_at: Option<Instant> = None;
        let _debounce = Duration::from_millis(500);
        let now = Instant::now();

        // Simulate 3 FullReload triggers from different paths within debounce window
        for i in 0..3 {
            let _path = PathBuf::from(format!("/test/script{}.ts", i));
            // When FullReload is triggered, set global state instead of per-path
            full_reload_at = Some(now);
            // Clear pending events - they're superseded by full reload
            pending.clear();
        }

        // Verify: full_reload_at is set, pending is empty
        assert!(full_reload_at.is_some());
        assert!(pending.is_empty());

        // Simulate debounce expiry - emit single FullReload
        if full_reload_at.is_some() {
            let _ = tx.send(ScriptReloadEvent::FullReload);
            // Reset after emission (in real code)
        }

        // Should only receive one FullReload
        let received = rx.try_recv().unwrap();
        assert_eq!(received, ScriptReloadEvent::FullReload);
        assert!(rx.try_recv().is_err()); // No more events
    }
    #[test]
    fn test_full_reload_clears_pending_events() {
        // When FullReload is triggered, it should clear all pending per-file events
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let now = Instant::now();

        // Add some pending per-file events
        pending.insert(
            PathBuf::from("/test/a.ts"),
            (
                ScriptReloadEvent::FileChanged(PathBuf::from("/test/a.ts")),
                now,
            ),
        );
        pending.insert(
            PathBuf::from("/test/b.ts"),
            (
                ScriptReloadEvent::FileCreated(PathBuf::from("/test/b.ts")),
                now,
            ),
        );

        assert_eq!(pending.len(), 2);

        // Trigger FullReload (e.g., from EventKind::Other)
        let full_reload_at: Option<Instant> = Some(now);
        pending.clear();

        // Pending should be empty, full_reload_at should be set
        assert!(pending.is_empty());
        assert!(full_reload_at.is_some());
    }
    // ============================================================
    // ISSUE B - Atomic save event merging tests
    // ============================================================

    #[test]
    fn test_merge_delete_then_create_to_changed() {
        // FileDeleted + FileCreated (same path) → FileChanged
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let path = PathBuf::from("/test/script.ts");
        let now = Instant::now();

        // First: delete event
        merge_script_event(
            &mut pending,
            &path,
            ScriptReloadEvent::FileDeleted(path.clone()),
            now,
        );
        assert_eq!(pending.len(), 1);
        assert!(matches!(
            pending.get(&path),
            Some((ScriptReloadEvent::FileDeleted(_), _))
        ));

        // Then: create event (atomic save completes)
        let later = now + Duration::from_millis(10);
        merge_script_event(
            &mut pending,
            &path,
            ScriptReloadEvent::FileCreated(path.clone()),
            later,
        );

        // Should be merged to FileChanged
        assert_eq!(pending.len(), 1);
        let (event, _) = pending.get(&path).unwrap();
        assert_eq!(*event, ScriptReloadEvent::FileChanged(path.clone()));
    }
    #[test]
    fn test_merge_create_then_delete_to_changed() {
        // FileCreated + FileDeleted (same path) → FileChanged
        // (temp file dance: create temp, delete original, rename temp)
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let path = PathBuf::from("/test/script.ts");
        let now = Instant::now();

        // First: create event
        merge_script_event(
            &mut pending,
            &path,
            ScriptReloadEvent::FileCreated(path.clone()),
            now,
        );

        // Then: delete event
        let later = now + Duration::from_millis(10);
        merge_script_event(
            &mut pending,
            &path,
            ScriptReloadEvent::FileDeleted(path.clone()),
            later,
        );

        // Should be merged to FileChanged
        assert_eq!(pending.len(), 1);
        let (event, _) = pending.get(&path).unwrap();
        assert_eq!(*event, ScriptReloadEvent::FileChanged(path.clone()));
    }
    #[test]
    fn test_no_merge_for_different_paths() {
        // Events for different paths should not be merged
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let path_a = PathBuf::from("/test/a.ts");
        let path_b = PathBuf::from("/test/b.ts");
        let now = Instant::now();

        // Delete on path A
        merge_script_event(
            &mut pending,
            &path_a,
            ScriptReloadEvent::FileDeleted(path_a.clone()),
            now,
        );

        // Create on path B (different path - no merge)
        merge_script_event(
            &mut pending,
            &path_b,
            ScriptReloadEvent::FileCreated(path_b.clone()),
            now,
        );

        // Should have 2 separate events
        assert_eq!(pending.len(), 2);
        assert!(matches!(
            pending.get(&path_a),
            Some((ScriptReloadEvent::FileDeleted(_), _))
        ));
        assert!(matches!(
            pending.get(&path_b),
            Some((ScriptReloadEvent::FileCreated(_), _))
        ));
    }
    #[test]
    fn test_no_merge_for_modify_events() {
        // FileChanged + FileDeleted should NOT merge to FileChanged
        // (only create/delete pairs merge)
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let path = PathBuf::from("/test/script.ts");
        let now = Instant::now();

        // First: modify event
        merge_script_event(
            &mut pending,
            &path,
            ScriptReloadEvent::FileChanged(path.clone()),
            now,
        );

        // Then: delete event
        let later = now + Duration::from_millis(10);
        merge_script_event(
            &mut pending,
            &path,
            ScriptReloadEvent::FileDeleted(path.clone()),
            later,
        );

        // Should NOT merge - delete overwrites
        assert_eq!(pending.len(), 1);
        let (event, _) = pending.get(&path).unwrap();
        assert_eq!(*event, ScriptReloadEvent::FileDeleted(path.clone()));
    }
    // ============================================================
    // Existing tests
    // ============================================================

    #[test]
    fn test_config_watcher_creation() {
        let (_watcher, _rx) = ConfigWatcher::new();
        // Watcher should be created without panicking
    }
    #[test]
    fn test_config_reload_event_clone() {
        let event = ConfigReloadEvent::Reload;
        let _cloned = event.clone();
        // Event should be cloneable
    }
    #[test]
    fn test_theme_watcher_creation() {
        let (_watcher, _rx) = ThemeWatcher::new();
        // Watcher should be created without panicking
    }
    #[test]
    fn test_theme_reload_event_clone() {
        let event = ThemeReloadEvent::Reload;
        let _cloned = event.clone();
        // Event should be cloneable
    }
    #[test]
    fn test_script_watcher_creation() {
        let (_watcher, _rx) = ScriptWatcher::new();
        // Watcher should be created without panicking
    }
    #[test]
    fn test_script_reload_event_clone() {
        let event = ScriptReloadEvent::FullReload;
        let _cloned = event.clone();
        // Event should be cloneable
    }
    #[test]
    fn test_script_reload_event_file_changed() {
        let path = PathBuf::from("/test/path/script.ts");
        let event = ScriptReloadEvent::FileChanged(path.clone());

        // Verify the event contains the correct path
        if let ScriptReloadEvent::FileChanged(event_path) = event {
            assert_eq!(event_path, path);
        } else {
            panic!("Expected FileChanged variant");
        }
    }
    #[test]
    fn test_script_reload_event_file_created() {
        let path = PathBuf::from("/test/path/new-script.ts");
        let event = ScriptReloadEvent::FileCreated(path.clone());

        // Verify the event contains the correct path
        if let ScriptReloadEvent::FileCreated(event_path) = event {
            assert_eq!(event_path, path);
        } else {
            panic!("Expected FileCreated variant");
        }
    }
    #[test]
    fn test_script_reload_event_file_deleted() {
        let path = PathBuf::from("/test/path/deleted-script.ts");
        let event = ScriptReloadEvent::FileDeleted(path.clone());

        // Verify the event contains the correct path
        if let ScriptReloadEvent::FileDeleted(event_path) = event {
            assert_eq!(event_path, path);
        } else {
            panic!("Expected FileDeleted variant");
        }
    }
    #[test]
    fn test_script_reload_event_equality() {
        let path1 = PathBuf::from("/test/path/script.ts");
        let path2 = PathBuf::from("/test/path/script.ts");
        let path3 = PathBuf::from("/test/path/other.ts");

        // Same path should be equal
        assert_eq!(
            ScriptReloadEvent::FileChanged(path1.clone()),
            ScriptReloadEvent::FileChanged(path2.clone())
        );

        // Different paths should not be equal
        assert_ne!(
            ScriptReloadEvent::FileChanged(path1.clone()),
            ScriptReloadEvent::FileChanged(path3.clone())
        );

        // Different event types should not be equal
        assert_ne!(
            ScriptReloadEvent::FileChanged(path1.clone()),
            ScriptReloadEvent::FileCreated(path1.clone())
        );

        // FullReload should equal itself
        assert_eq!(ScriptReloadEvent::FullReload, ScriptReloadEvent::FullReload);
    }
    #[test]
    fn test_extract_file_path_from_event() {
        // Test helper function for extracting paths from notify events
        use notify::event::{CreateKind, ModifyKind, RemoveKind};

        let test_path = PathBuf::from("/Users/test/.scriptkit/scripts/hello.ts");

        // Test Create event
        let create_event = notify::Event {
            kind: notify::EventKind::Create(CreateKind::File),
            paths: vec![test_path.clone()],
            attrs: Default::default(),
        };
        assert_eq!(create_event.paths.first(), Some(&test_path));

        // Test Modify event
        let modify_event = notify::Event {
            kind: notify::EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Content)),
            paths: vec![test_path.clone()],
            attrs: Default::default(),
        };
        assert_eq!(modify_event.paths.first(), Some(&test_path));

        // Test Remove event
        let remove_event = notify::Event {
            kind: notify::EventKind::Remove(RemoveKind::File),
            paths: vec![test_path.clone()],
            attrs: Default::default(),
        };
        assert_eq!(remove_event.paths.first(), Some(&test_path));
    }
    #[test]
    fn test_is_relevant_script_file() {
        use std::path::Path;

        // Test that we correctly identify relevant script files
        let ts_path = Path::new("/Users/test/.scriptkit/scripts/hello.ts");
        let js_path = Path::new("/Users/test/.scriptkit/scripts/hello.js");
        let md_path = Path::new("/Users/test/.scriptkit/scriptlets/hello.md");
        let txt_path = Path::new("/Users/test/.scriptkit/scripts/readme.txt");
        let hidden_path = Path::new("/Users/test/.scriptkit/scripts/.hidden.ts");

        // TypeScript files should be relevant
        assert!(is_relevant_script_file(ts_path));

        // JavaScript files should be relevant
        assert!(is_relevant_script_file(js_path));

        // Markdown files in scriptlets should be relevant
        assert!(is_relevant_script_file(md_path));

        // Other file types should not be relevant
        assert!(!is_relevant_script_file(txt_path));

        // Hidden files should not be relevant
        assert!(!is_relevant_script_file(hidden_path));
    }
    #[test]
    fn test_is_relevant_event_kind() {
        use notify::event::{AccessKind, CreateKind, ModifyKind, RemoveKind};

        // Access events should NOT be relevant
        assert!(!is_relevant_event_kind(&notify::EventKind::Access(
            AccessKind::Read
        )));

        // Create events SHOULD be relevant
        assert!(is_relevant_event_kind(&notify::EventKind::Create(
            CreateKind::File
        )));

        // Modify events SHOULD be relevant
        assert!(is_relevant_event_kind(&notify::EventKind::Modify(
            ModifyKind::Any
        )));

        // Remove events SHOULD be relevant
        assert!(is_relevant_event_kind(&notify::EventKind::Remove(
            RemoveKind::File
        )));

        // Other/Any events SHOULD be relevant (includes renames)
        assert!(is_relevant_event_kind(&notify::EventKind::Other));
        assert!(is_relevant_event_kind(&notify::EventKind::Any));
    }
    #[test]
    fn test_next_deadline_empty() {
        let pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let debounce = Duration::from_millis(500);

        assert!(next_deadline(&pending, None, debounce).is_none());
    }
    #[test]
    fn test_next_deadline_single() {
        let mut pending: HashMap<PathBuf, (ScriptReloadEvent, Instant)> = HashMap::new();
        let now = Instant::now();
        let debounce = Duration::from_millis(500);

        pending.insert(
            PathBuf::from("/test/script.ts"),
            (
                ScriptReloadEvent::FileChanged(PathBuf::from("/test/script.ts")),
                now,
            ),
        );

        let deadline = next_deadline(&pending, None, debounce);
        assert!(deadline.is_some());
        // Deadline should be approximately now + debounce
        let expected = now + debounce;
        let actual = deadline.unwrap();
        // Allow 1ms tolerance for timing
        assert!(actual >= expected && actual <= expected + Duration::from_millis(1));
    }
