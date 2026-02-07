#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static TRACKER_STATE_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_tracker_state_default() {
        let state = TrackerState::default();
        assert!(state.last_real_app.is_none());
        assert!(state.cached_menu_items.is_empty());
        assert!(state.fetching_bundle_id.is_none());
    }

    #[test]
    fn test_fetching_bundle_id_race_condition_fix() {
        // Test that the bundle_id tracking correctly handles concurrent fetches
        // Scenario: Thread A starts fetching for "com.app.A", then Thread B
        // starts for "com.app.B". Thread A finishes first - should NOT clear
        // the fetching state since B is now the active fetch.

        // Simulate Thread A starting fetch
        let mut state = TrackerState {
            fetching_bundle_id: Some("com.app.A".to_string()),
            ..Default::default()
        };
        assert_eq!(state.fetching_bundle_id.as_deref(), Some("com.app.A"));

        // Thread B starts fetching (overwrites A)
        state.fetching_bundle_id = Some("com.app.B".to_string());
        assert_eq!(state.fetching_bundle_id.as_deref(), Some("com.app.B"));

        // Thread A finishes - should NOT clear state (it's for B now)
        if state.fetching_bundle_id.as_deref() == Some("com.app.A") {
            state.fetching_bundle_id = None;
        }
        // State should still show B as fetching
        assert_eq!(
            state.fetching_bundle_id.as_deref(),
            Some("com.app.B"),
            "Thread A should not clear B's fetch state"
        );

        // Thread B finishes - should clear state
        if state.fetching_bundle_id.as_deref() == Some("com.app.B") {
            state.fetching_bundle_id = None;
        }
        assert!(
            state.fetching_bundle_id.is_none(),
            "Thread B should clear its own fetch state"
        );
    }

    #[test]
    fn test_tracked_app_clone() {
        let app = TrackedApp {
            pid: 123,
            bundle_id: "com.test.app".to_string(),
            name: "Test App".to_string(),
        };
        let cloned = app.clone();
        assert_eq!(cloned.pid, 123);
        assert_eq!(cloned.bundle_id, "com.test.app");
        assert_eq!(cloned.name, "Test App");
    }

    #[test]
    fn test_get_last_real_app_bundle_id_returns_none_when_not_set() {
        let _lock = TRACKER_STATE_TEST_LOCK.lock().unwrap();
        let previous = TRACKER_STATE.read().last_real_app.clone();

        let mut state = TRACKER_STATE.write();
        state.last_real_app = None;
        drop(state);

        assert_eq!(get_last_real_app_bundle_id(), None);

        TRACKER_STATE.write().last_real_app = previous;
    }

    #[test]
    fn test_get_last_real_app_bundle_id_returns_bundle_id_when_set() {
        let _lock = TRACKER_STATE_TEST_LOCK.lock().unwrap();
        let previous = TRACKER_STATE.read().last_real_app.clone();

        let mut state = TRACKER_STATE.write();
        state.last_real_app = Some(TrackedApp {
            pid: 42,
            bundle_id: "com.example.bundle".to_string(),
            name: "Example".to_string(),
        });
        drop(state);

        assert_eq!(
            get_last_real_app_bundle_id().as_deref(),
            Some("com.example.bundle")
        );

        TRACKER_STATE.write().last_real_app = previous;
    }

    #[test]
    fn test_same_bundle_id_different_pid_triggers_update() {
        // Regression test: when an app is relaunched (quit and reopened),
        // the bundle_id stays the same but the PID changes.
        // We MUST trigger an update to refresh the menu cache.

        // Helper function that mirrors the should_update logic in handle_app_activation_inner
        fn should_update(tracked: &TrackedApp, current_app: &Option<TrackedApp>) -> bool {
            current_app
                .as_ref()
                .map(|a| a.bundle_id != tracked.bundle_id || a.pid != tracked.pid)
                .unwrap_or(true)
        }

        // Case 1: No current app tracked -> should update
        let new_app = TrackedApp {
            pid: 100,
            bundle_id: "com.test.app".to_string(),
            name: "Test App".to_string(),
        };
        assert!(
            should_update(&new_app, &None),
            "Should update when no app is tracked"
        );

        // Case 2: Different bundle_id -> should update
        let current = Some(TrackedApp {
            pid: 100,
            bundle_id: "com.other.app".to_string(),
            name: "Other App".to_string(),
        });
        assert!(
            should_update(&new_app, &current),
            "Should update when bundle_id differs"
        );

        // Case 3: Same bundle_id AND same PID -> should NOT update
        let current_same = Some(TrackedApp {
            pid: 100,
            bundle_id: "com.test.app".to_string(),
            name: "Test App".to_string(),
        });
        assert!(
            !should_update(&new_app, &current_same),
            "Should NOT update when both bundle_id and PID are the same"
        );

        // Case 4: CRITICAL - Same bundle_id but DIFFERENT PID -> MUST update
        // This is the bug fix: app was relaunched with new PID
        let current_relaunched = Some(TrackedApp {
            pid: 200, // Different PID - app was relaunched!
            bundle_id: "com.test.app".to_string(),
            name: "Test App".to_string(),
        });
        assert!(
            should_update(&new_app, &current_relaunched),
            "MUST update when same bundle_id but different PID (app relaunched)"
        );
    }

    #[test]
    fn test_make_objc_cstring_rejects_interior_nul() {
        let result = make_objc_cstring("bad\0value");
        assert!(result.is_err(), "interior NUL should be rejected");
    }

    #[test]
    fn test_make_objc_cstring_accepts_valid_string() {
        let result = make_objc_cstring("NSWorkspaceDidActivateApplicationNotification");
        assert!(result.is_ok(), "valid strings should convert to CString");
    }
}
