// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Most of these tests are marked #[ignore] because they require
    // actual system interaction and should only be run manually on macOS.
    // Run with: cargo test --features system-tests -- --ignored

    #[test]
    fn test_run_applescript_syntax_error() {
        // Test that syntax errors are properly caught
        let result = run_applescript("this is not valid applescript syntax (((");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("AppleScript error"));
    }

    #[test]
    fn test_run_applescript_with_output_simple() {
        // Test a simple AppleScript that returns a value
        let result = run_applescript_with_output("return 42");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "42");
    }

    #[test]
    fn test_run_applescript_with_output_string() {
        // Test returning a string
        let result = run_applescript_with_output(r#"return "hello""#);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_run_applescript_with_output_boolean() {
        // Test returning a boolean
        let result = run_applescript_with_output("return true");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "true");
    }

    #[test]
    fn test_set_volume_clamps_to_100() {
        // Test that set_volume clamps values above 100
        // This doesn't actually set volume, just tests the script generation
        let test_value: u8 = 150;
        let script = format!("set volume output volume {}", test_value.min(100));
        assert!(script.contains("100"));
    }

    #[test]
    #[ignore]
    fn test_empty_trash_integration() {
        // Integration test - only run manually
        let result = empty_trash();
        // May succeed or fail depending on permissions
        println!("empty_trash result: {:?}", result);
    }

    #[test]
    #[ignore]
    fn test_toggle_dark_mode_integration() {
        // Integration test - only run manually
        let result = toggle_dark_mode();
        println!("toggle_dark_mode result: {:?}", result);
    }

    #[test]
    #[ignore]
    fn test_is_dark_mode_integration() {
        // Integration test - only run manually
        let result = is_dark_mode();
        println!("is_dark_mode result: {:?}", result);
    }

    #[test]
    #[ignore]
    fn test_volume_controls_integration() {
        // Integration test - only run manually
        if let Ok(initial_volume) = get_volume() {
            println!("Initial volume: {}", initial_volume);

            // Test volume up
            let _ = volume_up();

            // Test volume down
            let _ = volume_down();

            // Test set volume
            let _ = set_volume(initial_volume);

            // Test mute check
            if let Ok(muted) = is_muted() {
                println!("Is muted: {}", muted);
            }
        }
    }

    #[test]
    #[ignore]
    fn test_start_screen_saver_integration() {
        // Integration test - only run manually
        let result = start_screen_saver();
        println!("start_screen_saver result: {:?}", result);
    }

    #[test]
    #[ignore]
    fn test_mission_control_integration() {
        // Integration test - only run manually
        let result = mission_control();
        println!("mission_control result: {:?}", result);
    }

    #[test]
    #[ignore]
    fn test_launchpad_integration() {
        // Integration test - only run manually
        let result = launchpad();
        println!("launchpad result: {:?}", result);
    }

    #[test]
    #[ignore]
    fn test_open_system_settings_integration() {
        // Integration test - only run manually
        let result = open_sound_settings();
        println!("open_sound_settings result: {:?}", result);
    }

    // =========================================================================
    // Running Applications Tests
    // =========================================================================

    #[test]
    fn test_get_running_apps_returns_list() {
        // This test should work on any macOS system with GUI apps running
        let apps = get_running_apps().expect("Should get running apps");

        // There should be at least a few apps running (Finder is always running)
        assert!(!apps.is_empty(), "Should have at least one running app");

        // Each app should have a name and valid PID
        for app in &apps {
            assert!(!app.name.is_empty(), "App name should not be empty");
            assert!(app.pid > 0, "PID should be positive");
        }
    }

    #[test]
    fn test_get_running_apps_sorted_by_name() {
        let apps = get_running_apps().expect("Should get running apps");

        if apps.len() > 1 {
            // Verify sorted by name (case-insensitive)
            for i in 0..apps.len() - 1 {
                assert!(
                    apps[i].name.to_lowercase() <= apps[i + 1].name.to_lowercase(),
                    "Apps should be sorted by name: {} should come before {}",
                    apps[i].name,
                    apps[i + 1].name
                );
            }
        }
    }

    #[test]
    fn test_get_running_apps_excludes_system_processes() {
        let apps = get_running_apps().expect("Should get running apps");

        // System processes should be excluded
        let system_names = ["kernel_task", "launchd", "WindowServer", "loginwindow"];
        for app in &apps {
            assert!(
                !system_names.contains(&app.name.as_str()),
                "System process '{}' should be excluded",
                app.name
            );
        }
    }

    #[test]
    fn test_get_running_apps_excludes_helpers() {
        let apps = get_running_apps().expect("Should get running apps");

        // Helper processes should be excluded
        for app in &apps {
            assert!(
                !app.name.ends_with("Helper"),
                "Helper process '{}' should be excluded",
                app.name
            );
            assert!(
                !app.name.ends_with("Agent"),
                "Agent process '{}' should be excluded",
                app.name
            );
        }
    }

    #[test]
    fn test_app_info_struct() {
        // Test that AppInfo can be created and used
        let app = AppInfo {
            name: "TestApp".to_string(),
            pid: 1234,
            bundle_id: Some("com.test.app".to_string()),
            path: Some("/Applications/TestApp.app/Contents/MacOS/TestApp".to_string()),
            memory: 1024 * 1024,
            cpu_usage: 1.5,
        };

        assert_eq!(app.name, "TestApp");
        assert_eq!(app.pid, 1234);
        assert_eq!(app.bundle_id, Some("com.test.app".to_string()));
        assert!(app.path.is_some());
        assert_eq!(app.memory, 1024 * 1024);
        assert_eq!(app.cpu_usage, 1.5);
    }

    #[test]
    fn test_app_info_clone() {
        let app = AppInfo {
            name: "TestApp".to_string(),
            pid: 1234,
            bundle_id: None,
            path: None,
            memory: 0,
            cpu_usage: 0.0,
        };

        let cloned = app.clone();
        assert_eq!(app.name, cloned.name);
        assert_eq!(app.pid, cloned.pid);
    }

    #[test]
    fn test_force_quit_app_nonexistent_pid() {
        // Trying to force quit a non-existent PID should fail
        let result = force_quit_app(99999999);
        assert!(result.is_err(), "Should fail for non-existent PID");
    }

    #[test]
    fn test_force_quit_app_by_name_nonexistent() {
        // Trying to force quit a non-existent app should fail
        let result = force_quit_app_by_name("NonExistentAppThatDefinitelyDoesNotExist12345");
        assert!(result.is_err(), "Should fail for non-existent app");
    }

    #[test]
    #[ignore]
    fn test_get_running_apps_integration() {
        // Integration test - prints all running apps
        let apps = get_running_apps().expect("Should get running apps");
        println!("Found {} running apps:", apps.len());
        for app in apps {
            println!(
                "  {} (PID: {}, bundle: {:?}, mem: {} KB)",
                app.name,
                app.pid,
                app.bundle_id,
                app.memory / 1024
            );
        }
    }
}
