    use super::*;
    #[test]
    fn test_hud_notification_creation() {
        let notif = HudNotification {
            text: "Test".to_string(),
            duration_ms: 2000,
            created_at: Instant::now(),
            action_label: None,
            action: None,
        };
        assert_eq!(notif.text, "Test");
        assert_eq!(notif.duration_ms, 2000);
    }
    #[test]
    fn test_hud_manager_state_creation() {
        let state = HudManagerState::new();
        assert!(state.active_huds.is_empty());
        assert!(state.pending_queue.is_empty());
    }
    #[test]
    fn test_is_duration_expired_boundary_condition() {
        // HUD should be expired when elapsed == duration (not just >)
        // This tests the fix for the boundary condition bug

        // Create timestamp from 100ms ago
        let created_at = Instant::now() - Duration::from_millis(100);
        let duration = Duration::from_millis(100);

        // When elapsed >= duration, should be expired
        assert!(
            is_duration_expired(created_at, duration),
            "Should be expired when elapsed >= duration"
        );
    }
    #[test]
    fn test_is_duration_expired_definitely_expired() {
        // Create timestamp from 200ms ago with 100ms duration
        let created_at = Instant::now() - Duration::from_millis(200);
        let duration = Duration::from_millis(100);

        // When elapsed > duration, definitely expired
        assert!(
            is_duration_expired(created_at, duration),
            "Should be expired when elapsed > duration"
        );
    }
    #[test]
    fn test_is_duration_expired_not_expired_yet() {
        // Create timestamp from now with a long duration
        let created_at = Instant::now();
        let duration = Duration::from_millis(10000); // 10 seconds

        assert!(
            !is_duration_expired(created_at, duration),
            "Should not be expired immediately after creation"
        );
    }
    #[test]
    fn test_hud_notification_has_action() {
        let notif_without_action = HudNotification {
            text: "Test".to_string(),
            duration_ms: 2000,
            created_at: Instant::now(),
            action_label: None,
            action: None,
        };
        assert!(!notif_without_action.has_action());

        let notif_with_action = HudNotification {
            text: "Test".to_string(),
            duration_ms: 2000,
            created_at: Instant::now(),
            action_label: Some("Open".to_string()),
            action: Some(HudAction::OpenUrl("https://example.com".to_string())),
        };
        assert!(notif_with_action.has_action());
    }
    #[test]
    fn test_hud_id_generation() {
        // IDs should be unique and increasing
        let id1 = next_hud_id();
        let id2 = next_hud_id();
        let id3 = next_hud_id();

        assert!(id2 > id1, "IDs should be strictly increasing");
        assert!(id3 > id2, "IDs should be strictly increasing");
        assert_ne!(id1, id2, "IDs should be unique");
        assert_ne!(id2, id3, "IDs should be unique");
    }
    #[test]
    fn test_hud_view_has_action() {
        // Test that HudView correctly reports whether it has an action
        let view_without_action = HudView::new("Test message".to_string());
        assert!(
            !view_without_action.has_action(),
            "HudView without action should report has_action() = false"
        );

        let view_with_action = HudView::with_action(
            "Test message".to_string(),
            "Open".to_string(),
            HudAction::OpenUrl("https://example.com".to_string()),
        );
        assert!(
            view_with_action.has_action(),
            "HudView with action should report has_action() = true"
        );
    }
    #[test]
    fn test_hud_action_execute_open_url() {
        // Test that HudAction::OpenUrl can be created and executed doesn't panic
        // (actual URL opening is mocked in unit tests)
        let action = HudAction::OpenUrl("https://example.com".to_string());
        // Just verify it can be constructed - actual execution requires system integration
        match action {
            HudAction::OpenUrl(url) => assert_eq!(url, "https://example.com"),
            _ => panic!("Expected OpenUrl variant"),
        }
    }
    #[test]
    fn test_hud_action_execute_open_file() {
        // Test that HudAction::OpenFile can be created
        let action = HudAction::OpenFile(std::path::PathBuf::from("/tmp/test.txt"));
        match action {
            HudAction::OpenFile(path) => {
                assert_eq!(path, std::path::PathBuf::from("/tmp/test.txt"))
            }
            _ => panic!("Expected OpenFile variant"),
        }
    }
    #[test]
    fn test_hud_action_execute_run_command() {
        // Test that HudAction::RunCommand can be created
        let action = HudAction::RunCommand("echo hello".to_string());
        match action {
            HudAction::RunCommand(cmd) => assert_eq!(cmd, "echo hello"),
            _ => panic!("Expected RunCommand variant"),
        }
    }
    // =============================================================================
    // Theme Integration Tests
    // =============================================================================

    #[test]
    fn test_lighten_color() {
        // Test lightening pure black by 50%
        let black = 0x000000;
        let lightened = lighten_color(black, 0.5);
        // Should be ~0x7f7f7f (half way to white)
        assert_eq!(lightened, 0x7f7f7f);

        // Test lightening pure red by 10%
        let red = 0xff0000;
        let lightened_red = lighten_color(red, 0.1);
        // Red channel is already max, green/blue should be ~0x19 (25)
        assert_eq!(lightened_red >> 16, 0xff); // Red stays at max
        assert!((lightened_red >> 8) & 0xff >= 0x19); // Green increased
        assert!(lightened_red & 0xff >= 0x19); // Blue increased
    }
    #[test]
    fn test_darken_color() {
        // Test darkening pure white by 50%
        let white = 0xffffff;
        let darkened = darken_color(white, 0.5);
        // Should be ~0x7f7f7f (half way to black)
        assert_eq!(darkened, 0x7f7f7f);

        // Test darkening a color by 10%
        let color = 0x646464; // RGB(100, 100, 100)
        let darkened_color = darken_color(color, 0.1);
        // Each component should be 90% of original: 100 * 0.9 = 90 = 0x5a
        assert_eq!(darkened_color, 0x5a5a5a);
    }
    #[test]
    fn test_lighten_darken_boundary_conditions() {
        // Lightening white should stay white
        let white = 0xffffff;
        assert_eq!(lighten_color(white, 0.5), 0xffffff);

        // Darkening black should stay black
        let black = 0x000000;
        assert_eq!(darken_color(black, 0.5), 0x000000);
    }
    #[test]
    fn test_hud_colors_default() {
        // Test that default colors are valid (non-zero)
        let colors = HudColors::dark_default();
        assert_ne!(colors.background, 0);
        assert_ne!(colors.text_primary, 0);
        assert_ne!(colors.accent, 0);
        assert_ne!(colors.accent_hover, 0);
        assert_ne!(colors.accent_active, 0);
    }
    #[test]
    fn test_hud_colors_light_default() {
        // Test that light mode colors are valid
        let colors = HudColors::light_default();
        // Light mode should have light background
        assert_eq!(colors.background, 0xfafafa);
        // Light mode should have dark text
        assert_eq!(colors.text_primary, 0x000000);
        // Accent colors should be non-zero
        assert_ne!(colors.accent, 0);
        assert_ne!(colors.accent_hover, 0);
        assert_ne!(colors.accent_active, 0);
    }
    #[test]
    fn test_hud_colors_light_vs_dark_contrast() {
        // Test that light and dark themes have appropriate contrast
        let dark = HudColors::dark_default();
        let light = HudColors::light_default();

        // Dark mode: dark background, light text
        let dark_bg_brightness = ((dark.background >> 16) & 0xff)
            + ((dark.background >> 8) & 0xff)
            + (dark.background & 0xff);
        let dark_text_brightness = ((dark.text_primary >> 16) & 0xff)
            + ((dark.text_primary >> 8) & 0xff)
            + (dark.text_primary & 0xff);
        assert!(
            dark_bg_brightness < dark_text_brightness,
            "Dark mode: background should be darker than text"
        );

        // Light mode: light background, dark text
        let light_bg_brightness = ((light.background >> 16) & 0xff)
            + ((light.background >> 8) & 0xff)
            + (light.background & 0xff);
        let light_text_brightness = ((light.text_primary >> 16) & 0xff)
            + ((light.text_primary >> 8) & 0xff)
            + (light.text_primary & 0xff);
        assert!(
            light_bg_brightness > light_text_brightness,
            "Light mode: background should be lighter than text"
        );
    }
    #[test]
    fn test_hud_colors_accent_variants() {
        // Test that hover is lighter than accent, and active is darker
        let colors = HudColors::dark_default();

        // Extract brightness (simple sum of components)
        let brightness = |c: u32| ((c >> 16) & 0xff) + ((c >> 8) & 0xff) + (c & 0xff);

        // Hover should be brighter than base accent
        assert!(
            brightness(colors.accent_hover) >= brightness(colors.accent),
            "Hover should be at least as bright as accent"
        );

        // Active should be darker than base accent
        assert!(
            brightness(colors.accent_active) <= brightness(colors.accent),
            "Active should be at most as bright as accent"
        );
    }
    #[test]
    fn test_hud_view_with_custom_colors() {
        // Test that HudView can be created with custom colors
        let custom_colors = HudColors {
            background: 0x2a2a2a,
            text_primary: 0xeeeeee,
            accent: 0x00ff00,
            accent_hover: 0x33ff33,
            accent_active: 0x00cc00,
        };

        let view = HudView::with_colors("Custom themed HUD".to_string(), custom_colors);
        assert_eq!(view.colors.background, 0x2a2a2a);
        assert_eq!(view.colors.text_primary, 0xeeeeee);
        assert_eq!(view.colors.accent, 0x00ff00);
    }
    // =============================================================================
    // HUD Manager State Tests
    // =============================================================================

    #[test]
    fn test_hud_manager_state_queue_operations() {
        // Test that pending queue works correctly
        let mut state = HudManagerState::new();

        // Queue should start empty
        assert!(state.pending_queue.is_empty());

        // Add items to queue
        state.pending_queue.push_back(HudNotification {
            text: "First".to_string(),
            duration_ms: 2000,
            created_at: Instant::now(),
            action_label: None,
            action: None,
        });

        state.pending_queue.push_back(HudNotification {
            text: "Second".to_string(),
            duration_ms: 2000,
            created_at: Instant::now(),
            action_label: None,
            action: None,
        });

        assert_eq!(state.pending_queue.len(), 2);

        // Pop front should return first item
        let first = state.pending_queue.pop_front().unwrap();
        assert_eq!(first.text, "First");

        // Queue should still have one item
        assert_eq!(state.pending_queue.len(), 1);

        let second = state.pending_queue.pop_front().unwrap();
        assert_eq!(second.text, "Second");

        // Queue should be empty now
        assert!(state.pending_queue.is_empty());
    }
    #[test]
    fn test_hud_notification_partial_has_action() {
        // Test edge cases for has_action()

        // Only action_label set (should NOT have action)
        let notif_label_only = HudNotification {
            text: "Test".to_string(),
            duration_ms: 2000,
            created_at: Instant::now(),
            action_label: Some("Click".to_string()),
            action: None,
        };
        assert!(
            !notif_label_only.has_action(),
            "Should not have action with only label"
        );

        // Only action set (should NOT have action)
        let notif_action_only = HudNotification {
            text: "Test".to_string(),
            duration_ms: 2000,
            created_at: Instant::now(),
            action_label: None,
            action: Some(HudAction::OpenUrl("https://example.com".to_string())),
        };
        assert!(
            !notif_action_only.has_action(),
            "Should not have action with only action (no label)"
        );
    }
    #[test]
    fn test_hud_constants() {
        // Verify constants are sensible values using const blocks (clippy-compliant)

        // Default duration should be reasonable (1-10 seconds)
        const _: () = assert!(DEFAULT_HUD_DURATION_MS >= 1000 && DEFAULT_HUD_DURATION_MS <= 10000);

        // Stack gap should be positive and reasonable
        // Note: f32 comparisons in const context require workaround
        assert!(
            (HUD_STACK_GAP as i32) > 0 && (HUD_STACK_GAP as i32) < 100,
            "Stack gap should be positive and reasonable"
        );

        // Max simultaneous HUDs should be at least 1
        const _: () = assert!(MAX_SIMULTANEOUS_HUDS >= 1);

        // HUD dimensions should be positive
        assert!((HUD_WIDTH as i32) > 0, "HUD width should be positive");
        assert!((HUD_HEIGHT as i32) > 0, "HUD height should be positive");
    }
    #[test]
    fn test_hud_action_variants_debug() {
        // Test Debug impl for HudAction variants
        let open_file = HudAction::OpenFile(std::path::PathBuf::from("/test/path.txt"));
        let debug_str = format!("{:?}", open_file);
        assert!(
            debug_str.contains("OpenFile"),
            "Debug should contain variant name"
        );

        let open_url = HudAction::OpenUrl("https://test.com".to_string());
        let debug_str = format!("{:?}", open_url);
        assert!(
            debug_str.contains("OpenUrl"),
            "Debug should contain variant name"
        );

        let run_cmd = HudAction::RunCommand("ls -la".to_string());
        let debug_str = format!("{:?}", run_cmd);
        assert!(
            debug_str.contains("RunCommand"),
            "Debug should contain variant name"
        );
    }
    #[test]
    fn test_hud_action_clone() {
        // Test Clone impl for HudAction
        let original = HudAction::OpenUrl("https://clone-test.com".to_string());
        let cloned = original.clone();

        match (original, cloned) {
            (HudAction::OpenUrl(orig_url), HudAction::OpenUrl(clone_url)) => {
                assert_eq!(orig_url, clone_url, "Cloned URL should match original");
            }
            _ => panic!("Clone should preserve variant type"),
        }
    }
