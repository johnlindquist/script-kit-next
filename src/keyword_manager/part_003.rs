// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_disabled_manager() {
        let manager = KeywordManager::new();
        assert!(!manager.is_enabled());
        assert_eq!(manager.trigger_count(), 0);
    }

    #[test]
    fn test_default_creates_disabled_manager() {
        let manager = KeywordManager::default();
        assert!(!manager.is_enabled());
        assert_eq!(manager.trigger_count(), 0);
    }

    #[test]
    fn test_custom_config() {
        let config = KeywordManagerConfig {
            stop_delay_ms: 100,
            restart_delay_ms: 200,
            ..Default::default()
        };
        let manager = KeywordManager::with_config(config.clone());
        assert_eq!(manager.config.stop_delay_ms, 100);
        assert_eq!(manager.config.restart_delay_ms, 200);
    }

    #[test]
    fn test_register_trigger_manually() {
        let mut manager = KeywordManager::new();

        manager.register_trigger(":test", "Test Snippet", "Hello, World!", "paste");

        assert_eq!(manager.trigger_count(), 1);

        let triggers = manager.list_triggers();
        assert_eq!(triggers.len(), 1);
        assert_eq!(triggers[0].0, ":test");
        assert_eq!(triggers[0].1, "Test Snippet");
    }

    #[test]
    fn test_register_empty_trigger_ignored() {
        let mut manager = KeywordManager::new();

        manager.register_trigger("", "Empty", "Content", "paste");

        assert_eq!(manager.trigger_count(), 0);
    }

    #[test]
    fn test_clear_triggers() {
        let mut manager = KeywordManager::new();

        manager.register_trigger(":a", "A", "Content A", "paste");
        manager.register_trigger(":b", "B", "Content B", "paste");

        assert_eq!(manager.trigger_count(), 2);

        manager.clear_triggers();

        assert_eq!(manager.trigger_count(), 0);
    }

    #[test]
    fn test_list_triggers() {
        let mut manager = KeywordManager::new();

        manager.register_trigger(":sig", "Signature", "Best regards", "paste");
        manager.register_trigger(":addr", "Address", "123 Main St", "type");

        let triggers = manager.list_triggers();
        assert_eq!(triggers.len(), 2);

        // Check both triggers exist (order not guaranteed due to HashMap)
        let trigger_names: Vec<_> = triggers.iter().map(|(t, _)| t.as_str()).collect();
        assert!(trigger_names.contains(&":sig"));
        assert!(trigger_names.contains(&":addr"));
    }

    #[test]
    fn test_accessibility_check_does_not_panic() {
        // Just verify it doesn't panic - actual result depends on system
        let _ = KeywordManager::has_accessibility_permission();
    }

    // ========================================
    // Unregister Trigger Tests
    // ========================================

    #[test]
    fn test_unregister_trigger() {
        let mut manager = KeywordManager::new();

        manager.register_trigger(":test", "Test Snippet", "Hello, World!", "paste");
        assert_eq!(manager.trigger_count(), 1);

        // Unregister the trigger
        let removed = manager.unregister_trigger(":test");
        assert!(removed);
        assert_eq!(manager.trigger_count(), 0);

        // Verify it's not in the list
        let triggers = manager.list_triggers();
        assert!(triggers.is_empty());
    }

    #[test]
    fn test_unregister_nonexistent_trigger() {
        let mut manager = KeywordManager::new();

        let removed = manager.unregister_trigger(":nonexistent");
        assert!(!removed);
    }

    #[test]
    fn test_unregister_one_of_multiple_triggers() {
        let mut manager = KeywordManager::new();

        manager.register_trigger(":a", "A", "Content A", "paste");
        manager.register_trigger(":b", "B", "Content B", "paste");
        manager.register_trigger(":c", "C", "Content C", "paste");

        assert_eq!(manager.trigger_count(), 3);

        // Unregister just one
        let removed = manager.unregister_trigger(":b");
        assert!(removed);
        assert_eq!(manager.trigger_count(), 2);

        // Verify the right ones remain
        let triggers = manager.list_triggers();
        let trigger_names: Vec<_> = triggers.iter().map(|(t, _)| t.as_str()).collect();
        assert!(trigger_names.contains(&":a"));
        assert!(!trigger_names.contains(&":b"));
        assert!(trigger_names.contains(&":c"));
    }

    // ========================================
    // Clear Triggers For File Tests
    // ========================================

    #[test]
    fn test_clear_triggers_for_file() {
        let mut manager = KeywordManager::new();
        let path = PathBuf::from("/test/scriptlets/test.md");

        // Register triggers from a file
        manager.register_trigger_from_file(":sig", "Signature", "Best regards", "paste", &path);
        manager.register_trigger_from_file(":addr", "Address", "123 Main St", "paste", &path);

        assert_eq!(manager.trigger_count(), 2);
        assert_eq!(manager.get_triggers_for_file(&path).len(), 2);

        // Clear triggers for the file
        let cleared = manager.clear_triggers_for_file(&path);
        assert_eq!(cleared, 2);
        assert_eq!(manager.trigger_count(), 0);
        assert!(manager.get_triggers_for_file(&path).is_empty());
    }

    #[test]
    fn test_clear_triggers_for_file_only_affects_that_file() {
        let mut manager = KeywordManager::new();
        let path1 = PathBuf::from("/test/file1.md");
        let path2 = PathBuf::from("/test/file2.md");

        // Register triggers from two different files
        manager.register_trigger_from_file(":a", "A", "Content A", "paste", &path1);
        manager.register_trigger_from_file(":b", "B", "Content B", "paste", &path1);
        manager.register_trigger_from_file(":c", "C", "Content C", "paste", &path2);

        assert_eq!(manager.trigger_count(), 3);

        // Clear triggers for file1 only
        let cleared = manager.clear_triggers_for_file(&path1);
        assert_eq!(cleared, 2);
        assert_eq!(manager.trigger_count(), 1);

        // Verify file2's trigger is still there
        let triggers = manager.list_triggers();
        assert_eq!(triggers.len(), 1);
        assert_eq!(triggers[0].0, ":c");
    }

    #[test]
    fn test_clear_triggers_for_nonexistent_file() {
        let mut manager = KeywordManager::new();
        let path = PathBuf::from("/test/nonexistent.md");

        let cleared = manager.clear_triggers_for_file(&path);
        assert_eq!(cleared, 0);
    }

    // ========================================
    // Update Triggers For File Tests
    // ========================================

    #[test]
    fn test_update_triggers_add_new() {
        let mut manager = KeywordManager::new();
        let path = PathBuf::from("/test/file.md");

        // Start with no triggers
        assert_eq!(manager.trigger_count(), 0);

        // Add new triggers
        let new_triggers = vec![
            (
                ":a".to_string(),
                "A".to_string(),
                "Content A".to_string(),
                "paste".to_string(),
            ),
            (
                ":b".to_string(),
                "B".to_string(),
                "Content B".to_string(),
                "paste".to_string(),
            ),
        ];

        let (added, removed, updated) = manager.update_triggers_for_file(&path, &new_triggers);

        assert_eq!(added, 2);
        assert_eq!(removed, 0);
        assert_eq!(updated, 0);
        assert_eq!(manager.trigger_count(), 2);
    }

    #[test]
    fn test_update_triggers_remove_old() {
        let mut manager = KeywordManager::new();
        let path = PathBuf::from("/test/file.md");

        // Start with two triggers
        manager.register_trigger_from_file(":a", "A", "Content A", "paste", &path);
        manager.register_trigger_from_file(":b", "B", "Content B", "paste", &path);
        assert_eq!(manager.trigger_count(), 2);

        // Update with only one trigger (removes :b)
        let new_triggers = vec![(
            ":a".to_string(),
            "A".to_string(),
            "Content A".to_string(),
            "paste".to_string(),
        )];

        let (added, removed, updated) = manager.update_triggers_for_file(&path, &new_triggers);

        assert_eq!(added, 0);
        assert_eq!(removed, 1);
        assert_eq!(updated, 0);
        assert_eq!(manager.trigger_count(), 1);

        let triggers = manager.list_triggers();
        assert_eq!(triggers[0].0, ":a");
    }

    #[test]
    fn test_update_triggers_change_content() {
        let mut manager = KeywordManager::new();
        let path = PathBuf::from("/test/file.md");

        // Start with a trigger
        manager.register_trigger_from_file(":sig", "Signature", "Old content", "paste", &path);
        assert_eq!(manager.trigger_count(), 1);

        // Update with changed content
        let new_triggers = vec![(
            ":sig".to_string(),
            "Signature".to_string(),
            "New content".to_string(),
            "paste".to_string(),
        )];

        let (added, removed, updated) = manager.update_triggers_for_file(&path, &new_triggers);

        assert_eq!(added, 0);
        assert_eq!(removed, 0);
        assert_eq!(updated, 1);
        assert_eq!(manager.trigger_count(), 1);
    }

    #[test]
    fn test_update_triggers_mixed_operations() {
        let mut manager = KeywordManager::new();
        let path = PathBuf::from("/test/file.md");

        // Start with triggers :a, :b, :c
        manager.register_trigger_from_file(":a", "A", "Content A", "paste", &path);
        manager.register_trigger_from_file(":b", "B", "Content B", "paste", &path);
        manager.register_trigger_from_file(":c", "C", "Content C", "paste", &path);
        assert_eq!(manager.trigger_count(), 3);

        // Update:
        // - Keep :a unchanged
        // - Remove :b
        // - Change :c content
        // - Add :d
        let new_triggers = vec![
            (
                ":a".to_string(),
                "A".to_string(),
                "Content A".to_string(),
                "paste".to_string(),
            ),
            (
                ":c".to_string(),
                "C".to_string(),
                "New content C".to_string(),
                "paste".to_string(),
            ),
            (
                ":d".to_string(),
                "D".to_string(),
                "Content D".to_string(),
                "paste".to_string(),
            ),
        ];

        let (added, removed, updated) = manager.update_triggers_for_file(&path, &new_triggers);

        assert_eq!(added, 1); // :d
        assert_eq!(removed, 1); // :b
        assert_eq!(updated, 1); // :c
        assert_eq!(manager.trigger_count(), 3);

        let triggers = manager.list_triggers();
        let trigger_names: Vec<_> = triggers.iter().map(|(t, _)| t.as_str()).collect();
        assert!(trigger_names.contains(&":a"));
        assert!(!trigger_names.contains(&":b"));
        assert!(trigger_names.contains(&":c"));
        assert!(trigger_names.contains(&":d"));
    }

    #[test]
    fn test_update_triggers_empty_removes_all() {
        let mut manager = KeywordManager::new();
        let path = PathBuf::from("/test/file.md");

        // Start with triggers
        manager.register_trigger_from_file(":a", "A", "Content A", "paste", &path);
        manager.register_trigger_from_file(":b", "B", "Content B", "paste", &path);
        assert_eq!(manager.trigger_count(), 2);

        // Update with empty list
        let new_triggers: Vec<(String, String, String, String)> = vec![];

        let (added, removed, updated) = manager.update_triggers_for_file(&path, &new_triggers);

        assert_eq!(added, 0);
        assert_eq!(removed, 2);
        assert_eq!(updated, 0);
        assert_eq!(manager.trigger_count(), 0);
    }

    #[test]
    fn test_update_triggers_does_not_affect_other_files() {
        let mut manager = KeywordManager::new();
        let path1 = PathBuf::from("/test/file1.md");
        let path2 = PathBuf::from("/test/file2.md");

        // Register triggers from two files
        manager.register_trigger_from_file(":a", "A", "Content A", "paste", &path1);
        manager.register_trigger_from_file(":b", "B", "Content B", "paste", &path2);
        assert_eq!(manager.trigger_count(), 2);

        // Update file1 to remove its trigger
        let new_triggers: Vec<(String, String, String, String)> = vec![];
        manager.update_triggers_for_file(&path1, &new_triggers);

        // File2's trigger should still exist
        assert_eq!(manager.trigger_count(), 1);
        let triggers = manager.list_triggers();
        assert_eq!(triggers[0].0, ":b");
    }

    // ========================================
    // Register Trigger From File Tests
    // ========================================

    #[test]
    fn test_register_trigger_from_file() {
        let mut manager = KeywordManager::new();
        let path = PathBuf::from("/test/file.md");

        manager.register_trigger_from_file(":test", "Test", "Content", "paste", &path);

        assert_eq!(manager.trigger_count(), 1);
        assert_eq!(
            manager.get_triggers_for_file(&path),
            vec![":test".to_string()]
        );
    }

    #[test]
    fn test_register_trigger_from_file_empty_ignored() {
        let mut manager = KeywordManager::new();
        let path = PathBuf::from("/test/file.md");

        manager.register_trigger_from_file("", "Test", "Content", "paste", &path);

        assert_eq!(manager.trigger_count(), 0);
    }

    #[test]
    fn test_get_triggers_for_file() {
        let mut manager = KeywordManager::new();
        let path = PathBuf::from("/test/file.md");

        manager.register_trigger_from_file(":a", "A", "Content A", "paste", &path);
        manager.register_trigger_from_file(":b", "B", "Content B", "paste", &path);

        let triggers = manager.get_triggers_for_file(&path);
        assert_eq!(triggers.len(), 2);
        assert!(triggers.contains(&":a".to_string()));
        assert!(triggers.contains(&":b".to_string()));
    }

    // Integration tests that require system permissions
    #[test]
    #[ignore = "Requires accessibility permissions"]
    fn test_enable_disable_cycle() {
        let mut manager = KeywordManager::new();
        manager.register_trigger(":test", "Test", "Content", "paste");

        assert!(manager.enable().is_ok());
        assert!(manager.is_enabled());

        manager.disable();
        assert!(!manager.is_enabled());
    }
}
