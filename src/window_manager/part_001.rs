// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that WindowRole can be used as HashMap key
    #[test]
    fn test_window_role_hash_eq() {
        let role1 = WindowRole::Main;
        let role2 = WindowRole::Main;
        assert_eq!(role1, role2);

        // Verify it's Copy
        let role3 = role1;
        assert_eq!(role1, role3);
    }

    /// Test WindowRole Debug formatting
    #[test]
    fn test_window_role_debug() {
        let role = WindowRole::Main;
        let debug_str = format!("{:?}", role);
        // WindowRole::Main now from window_state, debug shows "Main"
        assert!(debug_str.contains("Main"));
    }

    // macOS-specific tests
    #[cfg(target_os = "macos")]
    mod macos_tests {
        use super::super::*;

        /// Test registration handle wrapper
        #[test]
        fn test_registered_window_handle_wrapper() {
            let ptr_value: usize = 0x12345678;
            let mock_id = ptr_value as id;

            let handle =
                RegisteredWindowHandle::from_window(mock_id).expect("mock pointer should work");
            assert_eq!(handle.window_ptr_addr, ptr_value);
            assert_eq!(handle.window_number, 0);
        }

        /// Test basic registration and retrieval
        /// Note: Uses a mock pointer since we can't create real NSWindow in tests
        #[test]
        fn test_register_and_get_window() {
            // Create a mock window ID (don't actually use this pointer!)
            let mock_id: id = 0x12345678 as id;

            // Register the window
            register_window(WindowRole::Main, mock_id);

            // Retrieve it
            let retrieved = get_window(WindowRole::Main);
            assert!(retrieved.is_some());
            assert_eq!(retrieved.unwrap(), mock_id);
        }

        /// Test get_main_window convenience function
        #[test]
        fn test_get_main_window_convenience() {
            let mock_id: id = 0x87654321 as id;
            register_window(WindowRole::Main, mock_id);

            let retrieved = get_main_window();
            assert!(retrieved.is_some());
            assert_eq!(retrieved.unwrap(), mock_id);
        }

        /// Test is_window_registered
        #[test]
        fn test_is_window_registered() {
            let mock_id: id = 0xABCDEF00 as id;

            // Register it
            register_window(WindowRole::Main, mock_id);

            // Should be registered now
            assert!(is_window_registered(WindowRole::Main));
        }

        /// Test that registration overwrites previous value
        #[test]
        fn test_registration_overwrites() {
            let first_id: id = 0x11111111 as id;
            let second_id: id = 0x22222222 as id;

            register_window(WindowRole::Main, first_id);
            assert_eq!(get_window(WindowRole::Main), Some(first_id));

            register_window(WindowRole::Main, second_id);
            assert_eq!(get_window(WindowRole::Main), Some(second_id));
        }

        /// Test WindowManager internal struct
        #[test]
        fn test_window_manager_struct() {
            let mut manager = WindowManager::new();

            let mock_id: id = 0x33333333 as id;

            // Initially empty
            assert!(!manager.is_registered(WindowRole::Main));
            assert!(manager.get_handle(WindowRole::Main).is_none());

            // Register
            manager.register(WindowRole::Main, mock_id);
            assert!(manager.is_registered(WindowRole::Main));
            assert_eq!(
                manager.get_handle(WindowRole::Main),
                RegisteredWindowHandle::from_window(mock_id)
            );

            // Unregister
            let removed = manager.unregister(WindowRole::Main);
            assert_eq!(removed, RegisteredWindowHandle::from_window(mock_id));
            assert!(!manager.is_registered(WindowRole::Main));
        }
    }

    // Non-macOS tests
    #[cfg(not(target_os = "macos"))]
    mod non_macos_tests {
        use super::super::*;

        #[test]
        fn test_stubs_return_none() {
            // All stub functions should return None or false
            assert!(get_window(WindowRole::Main).is_none());
            assert!(get_main_window().is_none());
            assert!(!find_and_register_main_window());
            assert!(!is_window_registered(WindowRole::Main));
            assert!(unregister_window(WindowRole::Main).is_none());
        }
    }
}
