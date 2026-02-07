// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Create a ProcessManager with a temporary directory for testing
    fn create_test_manager() -> (ProcessManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let manager = ProcessManager {
            active_processes: RwLock::new(HashMap::new()),
            main_pid_path: temp_dir.path().join("script-kit.pid"),
            active_pids_path: temp_dir.path().join("active-bun-pids.json"),
        };
        (manager, temp_dir)
    }

    #[test]
    fn test_write_and_read_main_pid() {
        let (manager, _temp_dir) = create_test_manager();

        // Write PID
        manager.write_main_pid().unwrap();

        // Read it back
        let pid = manager.read_main_pid();
        assert_eq!(pid, Some(std::process::id()));
    }

    #[test]
    fn test_remove_main_pid() {
        let (manager, _temp_dir) = create_test_manager();

        // Write and remove
        manager.write_main_pid().unwrap();
        assert!(manager.main_pid_path.exists());

        manager.remove_main_pid();
        assert!(!manager.main_pid_path.exists());
    }

    #[test]
    fn test_register_and_unregister_process() {
        let (manager, _temp_dir) = create_test_manager();

        // Register a process
        manager.register_process(12345, "/path/to/test.ts");

        // Check it's tracked
        let active = manager.get_active_processes();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].pid, 12345);
        assert_eq!(active[0].script_path, "/path/to/test.ts");

        // Check persistence
        assert!(manager.active_pids_path.exists());

        // Unregister
        manager.unregister_process(12345);

        // Check it's gone
        let active = manager.get_active_processes();
        assert!(active.is_empty());
    }

    #[test]
    fn test_multiple_processes() {
        let (manager, _temp_dir) = create_test_manager();

        // Register multiple processes
        manager.register_process(1001, "/path/to/script1.ts");
        manager.register_process(1002, "/path/to/script2.ts");
        manager.register_process(1003, "/path/to/script3.ts");

        assert_eq!(manager.active_count(), 3);

        // Unregister one
        manager.unregister_process(1002);
        assert_eq!(manager.active_count(), 2);

        // Verify correct one was removed
        let active = manager.get_active_processes();
        let pids: Vec<u32> = active.iter().map(|p| p.pid).collect();
        assert!(pids.contains(&1001));
        assert!(!pids.contains(&1002));
        assert!(pids.contains(&1003));
    }

    #[test]
    fn test_get_active_processes_sorted_newest_first() {
        let (manager, _temp_dir) = create_test_manager();
        manager.register_process(3001, "/path/to/first.ts");
        std::thread::sleep(std::time::Duration::from_millis(5));
        manager.register_process(3002, "/path/to/second.ts");

        let sorted = manager.get_active_processes_sorted();
        assert_eq!(sorted.len(), 2);
        assert_eq!(sorted[0].pid, 3002);
        assert_eq!(sorted[1].pid, 3001);
    }

    #[test]
    fn test_format_active_process_report_includes_summary_and_limit() {
        let (manager, _temp_dir) = create_test_manager();
        manager.register_process(4001, "/path/to/alpha.ts");
        manager.register_process(4002, "/path/to/beta.ts");

        let report = manager.format_active_process_report(1);
        assert!(report.contains("Active Script Kit processes: 2"));
        assert!(report.contains("PID "));
        assert!(report.contains("... and 1 more"));
    }

    #[test]
    fn test_format_active_process_report_empty_state() {
        let (manager, _temp_dir) = create_test_manager();
        let report = manager.format_active_process_report(5);
        assert_eq!(report, "No active Script Kit processes.");
    }

    #[test]
    fn test_kill_all_clears_tracking() {
        let (manager, _temp_dir) = create_test_manager();

        // Register some fake processes (won't actually exist)
        manager.register_process(99991, "/fake/script1.ts");
        manager.register_process(99992, "/fake/script2.ts");

        assert_eq!(manager.active_count(), 2);

        // Kill all (these PIDs don't exist, so kill will fail gracefully)
        manager.kill_all_processes();

        // Should be cleared
        assert_eq!(manager.active_count(), 0);
        assert!(!manager.active_pids_path.exists());
    }

    #[test]
    fn test_is_process_running_current_process() {
        let (manager, _temp_dir) = create_test_manager();

        // Current process should be running
        let current_pid = std::process::id();
        assert!(manager.is_process_running(current_pid));

        // Non-existent PID should not be running
        assert!(!manager.is_process_running(u32::MAX - 1));
    }

    #[test]
    fn test_persist_and_load_pids() {
        let (manager, _temp_dir) = create_test_manager();

        // Register processes
        manager.register_process(5001, "/test/a.ts");
        manager.register_process(5002, "/test/b.ts");

        // Load from disk
        let loaded = manager.load_persisted_pids();
        assert_eq!(loaded.len(), 2);

        let pids: Vec<u32> = loaded.iter().map(|p| p.pid).collect();
        assert!(pids.contains(&5001));
        assert!(pids.contains(&5002));
    }

    #[test]
    fn test_process_info_serialization() {
        let info = ProcessInfo {
            pid: 42,
            script_path: "/path/to/script.ts".to_string(),
            started_at: Utc::now(),
        };

        let json = serde_json::to_string(&info).unwrap();
        let parsed: ProcessInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.pid, 42);
        assert_eq!(parsed.script_path, "/path/to/script.ts");
    }

    #[test]
    fn test_cleanup_orphans_with_no_file() {
        let (manager, _temp_dir) = create_test_manager();

        // No file exists, should return 0
        let killed = manager.cleanup_orphans();
        assert_eq!(killed, 0);
    }

    #[test]
    fn test_main_pid_stale_detection() {
        let (manager, _temp_dir) = create_test_manager();

        // No PID file - not stale
        assert!(!manager.is_main_pid_stale());

        // Write current PID - not stale (current process is running)
        manager.write_main_pid().unwrap();
        assert!(!manager.is_main_pid_stale());

        // Write a fake PID that doesn't exist
        let fake_pid_path = manager.main_pid_path.clone();
        fs::write(&fake_pid_path, "999999999").unwrap();
        assert!(manager.is_main_pid_stale());
    }

    #[test]
    fn test_default_paths() {
        let manager = ProcessManager::new();

        // Should use ~/.scriptkit/ paths
        let home = dirs::home_dir().unwrap();
        assert_eq!(
            manager.main_pid_path,
            home.join(".scriptkit/script-kit.pid")
        );
        assert_eq!(
            manager.active_pids_path,
            home.join(".scriptkit/active-bun-pids.json")
        );
    }
}
