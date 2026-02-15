// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Main Thread Assertion Tests
    // =========================================================================

    /// Test that is_main_thread returns false on a test worker thread.
    /// Note: Rust test harness does NOT run tests on the main thread - it uses a thread pool.
    #[cfg(target_os = "macos")]
    #[test]
    fn test_is_main_thread_detects_non_main_thread() {
        // Rust tests run on thread pool workers, NOT the main thread.
        assert!(
            !is_main_thread(),
            "Expected test to run on non-main thread (Rust test harness behavior)"
        );
    }

    /// Test that require_main_thread returns true (bail) on a background thread.
    #[cfg(target_os = "macos")]
    #[test]
    fn test_require_main_thread_returns_true_on_background_thread() {
        // On a non-main thread, require_main_thread should return true (bail).
        assert!(
            require_main_thread("test_function"),
            "require_main_thread should return true on non-main thread"
        );
    }

    // =========================================================================
    // Characterization Tests (AppKit functions)
    // =========================================================================
    // NOTE: These tests are ignored on macOS because they require the main thread.
    // Rust's test harness runs tests on thread pool workers, not the main thread.
    // In production, these functions are called from GPUI's main thread event loop.

    /// Test that ensure_move_to_active_space can be called without panicking.
    /// This is a characterization test - it verifies the function doesn't crash.
    /// On non-macOS, this is a no-op. On macOS without a window, it logs a warning.
    #[test]
    #[cfg_attr(target_os = "macos", ignore = "requires main thread (run via GPUI)")]
    fn test_ensure_move_to_active_space_does_not_panic() {
        // Should not panic even without a window registered
        ensure_move_to_active_space();
    }

    /// Test that configure_as_floating_panel can be called without panicking.
    /// This is a characterization test - it verifies the function doesn't crash.
    /// On non-macOS, this is a no-op. On macOS without NSApp/keyWindow, it handles gracefully.
    #[test]
    #[cfg_attr(target_os = "macos", ignore = "requires main thread (run via GPUI)")]
    fn test_configure_as_floating_panel_does_not_panic() {
        // Should not panic even without an app running
        configure_as_floating_panel();
    }

    /// Verify the macOS constants have the correct values.
    #[cfg(target_os = "macos")]
    #[test]
    fn test_macos_constants() {
        assert_eq!(NS_FLOATING_WINDOW_LEVEL, 3);
        assert_eq!(NS_WINDOW_COLLECTION_BEHAVIOR_MOVE_TO_ACTIVE_SPACE, 2);
    }

    /// Test that both functions can be called in sequence.
    /// This mirrors the typical usage pattern in main.rs where both are called
    /// during window setup.
    #[test]
    #[cfg_attr(target_os = "macos", ignore = "requires main thread (run via GPUI)")]
    fn test_functions_can_be_called_in_sequence() {
        // This is the typical call order in main.rs
        ensure_move_to_active_space();
        configure_as_floating_panel();
        // Should complete without panicking
    }

    /// Test that functions are idempotent - can be called multiple times safely.
    #[test]
    #[cfg_attr(target_os = "macos", ignore = "requires main thread (run via GPUI)")]
    fn test_functions_are_idempotent() {
        for _ in 0..3 {
            ensure_move_to_active_space();
            configure_as_floating_panel();
        }
        // Should complete without panicking or causing issues
    }

    // =========================================================================
    // Mouse Position Tests
    // =========================================================================

    /// Test get_global_mouse_position returns valid coordinates or None.
    /// On macOS with display, returns Some((x, y)).
    /// On other platforms or without display, returns None.
    #[test]
    fn test_get_global_mouse_position_does_not_panic() {
        // Should not panic regardless of whether we can get the position
        let _ = get_global_mouse_position();
    }

    // =========================================================================
    // Display Information Tests
    // =========================================================================

    /// Test DisplayBounds struct creation and field access.
    #[test]
    fn test_display_bounds_struct() {
        let bounds = DisplayBounds {
            origin_x: 100.0,
            origin_y: 200.0,
            width: 1920.0,
            height: 1080.0,
        };

        assert_eq!(bounds.origin_x, 100.0);
        assert_eq!(bounds.origin_y, 200.0);
        assert_eq!(bounds.width, 1920.0);
        assert_eq!(bounds.height, 1080.0);
    }

    /// Test DisplayBounds Clone implementation.
    #[test]
    fn test_display_bounds_clone() {
        let original = DisplayBounds {
            origin_x: 0.0,
            origin_y: 0.0,
            width: 2560.0,
            height: 1440.0,
        };

        let cloned = original.clone();
        assert_eq!(cloned.width, 2560.0);
        assert_eq!(cloned.height, 1440.0);
    }

    /// Test get_macos_displays returns at least one display (or fallback).
    #[test]
    #[cfg_attr(target_os = "macos", ignore = "requires main thread (run via GPUI)")]
    fn test_get_macos_displays_returns_at_least_one() {
        let displays = get_macos_displays();
        assert!(!displays.is_empty(), "Should return at least one display");
    }

    /// Test get_macos_displays returns displays with valid dimensions.
    #[test]
    #[cfg_attr(target_os = "macos", ignore = "requires main thread (run via GPUI)")]
    fn test_get_macos_displays_valid_dimensions() {
        let displays = get_macos_displays();
        for display in displays {
            assert!(display.width > 0.0, "Display width must be positive");
            assert!(display.height > 0.0, "Display height must be positive");
        }
    }

    // =========================================================================
    // Window Movement Tests
    // =========================================================================

    /// Test move_first_window_to does not panic without a window.
    #[test]
    #[cfg_attr(target_os = "macos", ignore = "requires main thread (run via GPUI)")]
    fn test_move_first_window_to_does_not_panic() {
        // Should not panic even without a registered window
        move_first_window_to(100.0, 100.0, 800.0, 600.0);
    }

    /// Test move_first_window_to_bounds wrapper function.
    #[test]
    #[cfg_attr(target_os = "macos", ignore = "requires main thread (run via GPUI)")]
    fn test_move_first_window_to_bounds_does_not_panic() {
        use gpui::size;
        let bounds = Bounds {
            origin: point(px(100.0), px(100.0)),
            size: size(px(800.0), px(600.0)),
        };
        // Should not panic
        move_first_window_to_bounds(&bounds);
    }

    // =========================================================================
    // Eye-line Positioning Tests
    // =========================================================================

    /// Test calculate_eye_line_bounds returns valid bounds.
    #[test]
    #[cfg_attr(target_os = "macos", ignore = "requires main thread (run via GPUI)")]
    fn test_calculate_eye_line_bounds_returns_valid() {
        use gpui::size;
        let window_size = size(px(750.0), px(500.0));
        let bounds = calculate_eye_line_bounds_on_mouse_display(window_size);

        // Bounds should have the same size as input
        assert_eq!(bounds.size.width, window_size.width);
        assert_eq!(bounds.size.height, window_size.height);
    }

    /// Test eye-line calculation positions window in upper portion of screen.
    #[test]
    #[cfg_attr(target_os = "macos", ignore = "requires main thread (run via GPUI)")]
    fn test_calculate_eye_line_bounds_upper_portion() {
        use gpui::size;
        let window_size = size(px(750.0), px(500.0));
        let bounds = calculate_eye_line_bounds_on_mouse_display(window_size);

        // Get the display bounds for comparison
        let displays = get_macos_displays();
        if let Some(display) = displays.first() {
            let origin_y: f64 = bounds.origin.y.into();
            let display_height = display.height;

            // Eye-line should be in upper half of screen (top 50%)
            assert!(
                origin_y < display.origin_y + display_height * 0.5,
                "Window should be in upper half: origin_y={}, display_mid={}",
                origin_y,
                display.origin_y + display_height * 0.5
            );
        }
    }

    // =========================================================================
    // Path Action Tests
    // =========================================================================

    #[test]
    fn test_reveal_in_finder_returns_error_for_missing_path() {
        let temp_dir = tempfile::tempdir().expect("Failed to create tempdir");
        let missing_path = temp_dir.path().join("missing-script-path.js");

        let result = reveal_in_finder(&missing_path);
        let error = result.expect_err("Expected missing path to fail");

        assert!(
            error.contains("platform_reveal_in_finder_failed"),
            "Expected structured error id, got: {error}"
        );
        assert!(
            error.contains("canonicalize_path"),
            "Expected canonicalize stage, got: {error}"
        );
    }

    #[test]
    fn test_open_in_default_app_returns_error_for_missing_path() {
        let temp_dir = tempfile::tempdir().expect("Failed to create tempdir");
        let missing_path = temp_dir.path().join("missing-extension-path.ts");

        let result = open_in_default_app(&missing_path);
        let error = result.expect_err("Expected missing path to fail");

        assert!(
            error.contains("platform_open_in_default_app_failed"),
            "Expected structured error id, got: {error}"
        );
        assert!(
            error.contains("canonicalize_path"),
            "Expected canonicalize stage, got: {error}"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_copy_text_to_clipboard_returns_main_thread_error_on_test_thread() {
        let result = copy_text_to_clipboard("script kit clipboard");
        let error = result.expect_err("Expected AppKit main-thread guard in tests");

        assert!(
            error.contains("platform_copy_text_to_clipboard_failed"),
            "Expected structured error id, got: {error}"
        );
        assert!(
            error.contains("main_thread_required"),
            "Expected main-thread stage, got: {error}"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_run_osascript_returns_stdout_when_script_succeeds() {
        let output = run_osascript(
            r#"return "osascript success""#,
            "test_run_osascript_returns_stdout_when_script_succeeds",
        )
        .expect("Expected osascript to return output");

        assert_eq!(output, "osascript success");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_run_osascript_includes_context_when_script_fails() {
        let context = "test_run_osascript_includes_context_when_script_fails";
        let error = run_osascript("this is not valid applescript syntax (((", context)
            .expect_err("Expected invalid AppleScript syntax to fail")
            .to_string();

        assert!(
            error.contains("platform_run_osascript_failed"),
            "Expected structured error id, got: {error}"
        );
        assert!(
            error.contains("stage=exit_status"),
            "Expected exit-status stage, got: {error}"
        );
        assert!(
            error.contains(context),
            "Expected context in error, got: {error}"
        );
    }
}
