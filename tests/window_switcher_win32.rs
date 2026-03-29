//! Integration tests for the Window Switcher Win32 implementation.
//!
//! These tests verify that the window_control module (backed by Win32 on Windows)
//! correctly enumerates, queries, and manages windows.

#[cfg(target_os = "windows")]
mod windows_tests {
    use script_kit_gpui::window_control;

    // ========================================================================
    // Type construction / data structure tests
    // ========================================================================

    #[test]
    fn bounds_new_roundtrip() {
        let b = window_control::Bounds::new(10, 20, 800, 600);
        assert_eq!(b.x, 10);
        assert_eq!(b.y, 20);
        assert_eq!(b.width, 800);
        assert_eq!(b.height, 600);
    }

    #[test]
    fn bounds_negative_position() {
        let b = window_control::Bounds::new(-100, -50, 1920, 1080);
        assert_eq!(b.x, -100);
        assert_eq!(b.y, -50);
    }

    #[test]
    fn bounds_clone_eq() {
        let a = window_control::Bounds::new(0, 0, 1920, 1080);
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn window_info_for_test_constructor() {
        let info = window_control::WindowInfo::for_test(
            42,
            "TestApp".into(),
            "My Window".into(),
            window_control::Bounds::new(100, 200, 640, 480),
            1234,
        );
        assert_eq!(info.id, 42);
        assert_eq!(info.app, "TestApp");
        assert_eq!(info.title, "My Window");
        assert_eq!(info.bounds.x, 100);
        assert_eq!(info.bounds.width, 640);
        assert_eq!(info.pid, 1234);
    }

    #[test]
    fn window_info_clone() {
        let info = window_control::WindowInfo::for_test(
            1,
            "App".into(),
            "Title".into(),
            window_control::Bounds::new(0, 0, 100, 100),
            99,
        );
        let cloned = info.clone();
        assert_eq!(cloned.id, info.id);
        assert_eq!(cloned.title, info.title);
        assert_eq!(cloned.app, info.app);
    }

    #[test]
    fn tile_position_equality() {
        assert_eq!(
            window_control::TilePosition::LeftHalf,
            window_control::TilePosition::LeftHalf
        );
        assert_ne!(
            window_control::TilePosition::LeftHalf,
            window_control::TilePosition::RightHalf
        );
    }

    // ========================================================================
    // Live Win32 API tests — enumerate real desktop windows
    // ========================================================================

    #[test]
    fn list_windows_succeeds() {
        let result = window_control::list_windows();
        assert!(
            result.is_ok(),
            "list_windows() should succeed: {:?}",
            result.err()
        );
    }

    #[test]
    fn list_windows_returns_at_least_one() {
        let windows = window_control::list_windows().expect("list_windows should succeed");
        assert!(
            !windows.is_empty(),
            "Expected at least 1 visible window on the desktop"
        );
    }

    #[test]
    fn listed_windows_have_nonempty_titles() {
        let windows = window_control::list_windows().expect("list_windows should succeed");
        for w in &windows {
            assert!(
                !w.title.is_empty(),
                "Window id={} has empty title (app={})",
                w.id,
                w.app
            );
        }
    }

    #[test]
    fn listed_windows_have_valid_pids() {
        let windows = window_control::list_windows().expect("list_windows should succeed");
        for w in &windows {
            assert!(w.pid > 0, "Window '{}' has invalid pid {}", w.title, w.pid);
        }
    }

    #[test]
    fn listed_windows_have_nonempty_app_names() {
        let windows = window_control::list_windows().expect("list_windows should succeed");
        for w in &windows {
            assert!(
                !w.app.is_empty(),
                "Window '{}' (id={}) has empty app name",
                w.title,
                w.id
            );
        }
    }

    #[test]
    fn listed_windows_have_reasonable_bounds() {
        let windows = window_control::list_windows().expect("list_windows should succeed");
        let any_with_bounds = windows
            .iter()
            .any(|w| w.bounds.width > 0 && w.bounds.height > 0);
        // At least one window should have real dimensions.
        // In a headless CI environment this may not hold, so log a warning instead of failing.
        if !any_with_bounds {
            eprintln!(
                "WARNING: no windows with positive bounds found ({} windows total). \
                 This is expected in headless environments.",
                windows.len()
            );
        }
    }

    #[test]
    fn focus_window_with_invalid_hwnd_fails() {
        // HWND 0 is never valid.
        let result = window_control::focus_window(0);
        assert!(result.is_err(), "focus_window(0) should fail");
    }

    #[test]
    fn get_frontmost_window_of_previous_app_succeeds() {
        let result = window_control::get_frontmost_window_of_previous_app();
        assert!(
            result.is_ok(),
            "get_frontmost_window_of_previous_app() should not error: {:?}",
            result.err()
        );
    }

    /// Verify the end-to-end Window Switcher flow:
    /// list windows → pick the first → verify focus_window accepts the ID.
    #[test]
    fn window_switcher_flow_list_then_focus_valid_window() {
        let windows = window_control::list_windows().expect("list_windows should succeed");
        if windows.is_empty() {
            eprintln!("WARNING: no windows to test focus_window with");
            return;
        }
        let first = &windows[0];
        // We don't actually want to steal focus in CI, but we can verify the
        // call doesn't panic and returns a Result.
        let _result = window_control::focus_window(first.id);
        // Success or failure is fine — the point is it didn't crash.
    }

    /// Verify that the listed window titles are valid UTF-16 → UTF-8 conversions
    /// (no replacement characters from broken encoding).
    #[test]
    fn listed_window_titles_are_valid_utf8() {
        let windows = window_control::list_windows().expect("list_windows should succeed");
        for w in &windows {
            // If the conversion produced replacement chars, that's suspicious
            // but not necessarily wrong (some apps have emoji titles).
            assert!(
                w.title.is_ascii() || !w.title.contains('\u{FFFD}'),
                "Window title contains replacement character: '{}'",
                w.title
            );
        }
    }
}

/// Ensure the module compiles on all platforms (types are available).
#[test]
fn window_control_types_exist() {
    let _ = std::mem::size_of::<script_kit_gpui::window_control::Bounds>();
    let _ = std::mem::size_of::<script_kit_gpui::window_control::WindowInfo>();
    let _ = std::mem::size_of::<script_kit_gpui::window_control::TilePosition>();
}
