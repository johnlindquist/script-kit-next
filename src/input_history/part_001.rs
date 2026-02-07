#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_test_history() -> (InputHistory, PathBuf) {
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join(format!("input_history_test_{}.json", uuid::Uuid::new_v4()));
        let history = InputHistory::with_path(temp_path.clone());
        (history, temp_path)
    }

    fn cleanup_temp_file(path: &PathBuf) {
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_new_history_is_empty() {
        let (history, path) = create_test_history();
        assert!(history.is_empty());
        assert_eq!(history.len(), 0);
        cleanup_temp_file(&path);
    }

    #[test]
    fn test_add_entry() {
        let (mut history, path) = create_test_history();

        history.add_entry("hello");
        assert_eq!(history.len(), 1);
        assert_eq!(history.entries()[0], "hello");

        cleanup_temp_file(&path);
    }

    #[test]
    fn test_add_entry_prepends() {
        let (mut history, path) = create_test_history();

        history.add_entry("first");
        history.add_entry("second");
        history.add_entry("third");

        assert_eq!(history.entries(), &["third", "second", "first"]);

        cleanup_temp_file(&path);
    }

    #[test]
    fn test_add_entry_deduplicates() {
        let (mut history, path) = create_test_history();

        history.add_entry("apple");
        history.add_entry("banana");
        history.add_entry("apple"); // Duplicate

        assert_eq!(history.len(), 2);
        assert_eq!(history.entries(), &["apple", "banana"]);

        cleanup_temp_file(&path);
    }

    #[test]
    fn test_add_entry_caps_at_max() {
        let (mut history, path) = create_test_history();

        // Add more than MAX_ENTRIES to verify truncation
        for i in 0..120 {
            history.add_entry(&format!("entry{}", i));
        }

        assert_eq!(history.len(), MAX_ENTRIES);
        assert_eq!(history.entries()[0], "entry119"); // Most recent

        cleanup_temp_file(&path);
    }

    #[test]
    fn test_add_entry_skips_empty() {
        let (mut history, path) = create_test_history();

        history.add_entry("");
        history.add_entry("   ");
        history.add_entry("\t\n");

        assert!(history.is_empty());

        cleanup_temp_file(&path);
    }

    #[test]
    fn test_add_entry_trims_whitespace() {
        let (mut history, path) = create_test_history();

        history.add_entry("  hello  ");
        assert_eq!(history.entries()[0], "hello");

        cleanup_temp_file(&path);
    }

    #[test]
    fn test_navigate_up_empty() {
        let (mut history, path) = create_test_history();

        assert!(history.navigate_up().is_none());

        cleanup_temp_file(&path);
    }

    #[test]
    fn test_navigate_up() {
        let (mut history, path) = create_test_history();

        history.add_entry("first");
        history.add_entry("second");
        history.add_entry("third");

        assert_eq!(history.navigate_up(), Some("third".to_string()));
        assert_eq!(history.current_index(), Some(0));

        assert_eq!(history.navigate_up(), Some("second".to_string()));
        assert_eq!(history.current_index(), Some(1));

        assert_eq!(history.navigate_up(), Some("first".to_string()));
        assert_eq!(history.current_index(), Some(2));

        assert_eq!(history.navigate_up(), None); // At end
        assert_eq!(history.current_index(), Some(2)); // Stays at 2

        cleanup_temp_file(&path);
    }

    #[test]
    fn test_navigate_down() {
        let (mut history, path) = create_test_history();

        history.add_entry("first");
        history.add_entry("second");
        history.add_entry("third");

        // Navigate up first
        history.navigate_up(); // third (index 0)
        history.navigate_up(); // second (index 1)
        history.navigate_up(); // first (index 2)

        assert_eq!(history.navigate_down(), Some("second".to_string()));
        assert_eq!(history.current_index(), Some(1));

        assert_eq!(history.navigate_down(), Some("third".to_string()));
        assert_eq!(history.current_index(), Some(0));

        assert_eq!(history.navigate_down(), None); // Past newest
        assert_eq!(history.current_index(), None); // Reset

        cleanup_temp_file(&path);
    }

    #[test]
    fn test_navigate_down_not_navigating() {
        let (mut history, path) = create_test_history();

        history.add_entry("test");
        assert!(history.navigate_down().is_none());

        cleanup_temp_file(&path);
    }

    #[test]
    fn test_reset_navigation() {
        let (mut history, path) = create_test_history();

        history.add_entry("test");
        history.navigate_up();
        assert!(history.current_index().is_some());

        history.reset_navigation();
        assert!(history.current_index().is_none());

        cleanup_temp_file(&path);
    }

    #[test]
    fn test_add_entry_resets_navigation() {
        let (mut history, path) = create_test_history();

        history.add_entry("first");
        history.navigate_up();
        assert!(history.current_index().is_some());

        history.add_entry("second");
        assert!(history.current_index().is_none());

        cleanup_temp_file(&path);
    }

    #[test]
    fn test_save_and_load() {
        let (_, path) = create_test_history();

        // Create and populate history
        {
            let mut history = InputHistory::with_path(path.clone());
            history.add_entry("first");
            history.add_entry("second");
            history.add_entry("third");
            history.save().unwrap();
        }

        // Load into new history
        {
            let mut history = InputHistory::with_path(path.clone());
            history.load().unwrap();

            assert_eq!(history.len(), 3);
            assert_eq!(history.entries(), &["third", "second", "first"]);
            assert!(history.current_index().is_none()); // Navigation reset on load
        }

        cleanup_temp_file(&path);
    }

    #[test]
    fn test_load_missing_file() {
        let mut history = InputHistory::with_path(PathBuf::from("/nonexistent/path/history.json"));
        let result = history.load();
        assert!(result.is_ok());
        assert!(history.is_empty());
    }

    #[test]
    fn test_load_invalid_json() {
        let (_, path) = create_test_history();
        fs::write(&path, "not valid json").unwrap();

        let mut history = InputHistory::with_path(path.clone());
        let result = history.load();
        assert!(result.is_err());

        cleanup_temp_file(&path);
    }

    #[test]
    fn test_load_truncates_excessive_entries() {
        let (_, path) = create_test_history();

        // Write file with too many entries
        let entries: Vec<String> = (0..100).map(|i| format!("entry{}", i)).collect();
        let data = InputHistoryData { entries };
        fs::write(&path, serde_json::to_string(&data).unwrap()).unwrap();

        let mut history = InputHistory::with_path(path.clone());
        history.load().unwrap();

        assert_eq!(history.len(), MAX_ENTRIES);

        cleanup_temp_file(&path);
    }

    #[test]
    fn test_clear() {
        let (mut history, path) = create_test_history();

        history.add_entry("test");
        history.navigate_up();

        history.clear();

        assert!(history.is_empty());
        assert!(history.current_index().is_none());

        cleanup_temp_file(&path);
    }

    #[test]
    fn test_max_entries_constant() {
        // User requirement: store at least 100 recent entries
        assert_eq!(MAX_ENTRIES, 100);
    }
}
