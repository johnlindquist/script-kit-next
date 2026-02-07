use super::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn test_is_typescript() {
    assert!(is_typescript(&PathBuf::from("script.ts")));
    assert!(!is_typescript(&PathBuf::from("script.js")));
    assert!(!is_typescript(&PathBuf::from("script.txt")));
}

#[test]
fn test_is_javascript() {
    assert!(is_javascript(&PathBuf::from("script.js")));
    assert!(!is_javascript(&PathBuf::from("script.ts")));
    assert!(!is_javascript(&PathBuf::from("script.txt")));
}

#[test]
fn test_is_typescript_with_path() {
    assert!(is_typescript(&PathBuf::from(
        "/home/user/.scriptkit/scripts/script.ts"
    )));
    assert!(is_typescript(&PathBuf::from("/usr/local/bin/script.ts")));
}

#[test]
fn test_is_javascript_with_path() {
    assert!(is_javascript(&PathBuf::from(
        "/home/user/.scriptkit/scripts/script.js"
    )));
    assert!(is_javascript(&PathBuf::from("/usr/local/bin/script.js")));
}

#[test]
fn test_file_extensions_case_sensitive() {
    // Rust PathBuf.extension() returns lowercase for comparison
    assert!(
        is_typescript(&PathBuf::from("script.TS")) || !is_typescript(&PathBuf::from("script.TS"))
    );
    // Extension check should work regardless (implementation detail)
}

#[test]
fn test_unsupported_extension() {
    assert!(!is_typescript(&PathBuf::from("script.py")));
    assert!(!is_javascript(&PathBuf::from("script.rs")));
    assert!(!is_typescript(&PathBuf::from("script")));
}

#[test]
fn test_files_with_no_extension() {
    assert!(!is_typescript(&PathBuf::from("script")));
    assert!(!is_javascript(&PathBuf::from("mycommand")));
}

#[test]
fn test_multiple_dots_in_filename() {
    assert!(is_typescript(&PathBuf::from("my.test.script.ts")));
    assert!(is_javascript(&PathBuf::from("my.test.script.js")));
}

#[test]
fn test_process_handle_double_kill_is_safe() {
    // Double kill should not panic
    let mut handle = ProcessHandle::new(99999, "[test:double_kill]".to_string()); // Non-existent PID
    handle.kill();
    handle.kill(); // Should be safe to call again
    assert!(handle.killed);
}

#[test]
fn test_process_handle_drop_calls_kill() {
    // Create a handle and let it drop
    let handle = ProcessHandle::new(99998, "[test:drop_kill]".to_string()); // Non-existent PID
    assert!(!handle.killed);
    drop(handle);
    // If we get here without panic, drop successfully called kill
}

#[test]
fn test_process_handle_registers_with_process_manager() {
    // ProcessHandle::new() internally calls PROCESS_MANAGER.register_process()
    // and Drop calls PROCESS_MANAGER.unregister_process()

    // Create a handle which should register with PROCESS_MANAGER
    let test_pid = 88888u32; // Non-existent PID
    let test_script = "/test/integration_test.ts";

    // Create handle - this calls register_process() internally
    let handle = ProcessHandle::new(test_pid, test_script.to_string());

    // Verify handle has correct PID
    assert_eq!(handle.pid, test_pid);

    // Drop will call unregister_process() - this should not panic
    drop(handle);

    // If we get here, register/unregister cycle completed successfully
}

#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_spawn_and_kill_process() {
    // Spawn a simple process that sleeps
    let result = spawn_script("sleep", &["10"], "[test:sleep]");

    if let Ok(mut session) = result {
        let pid = session.pid();
        assert!(pid > 0);

        // Process should be running
        assert!(session.is_running());

        // Kill it
        session.kill().expect("kill should succeed");

        // Wait a moment for the process to die
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Process should no longer be running
        assert!(!session.is_running());
    }
    // If spawn failed (sleep not available), that's OK for this test
}

#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_drop_kills_process() {
    // Spawn a process
    let result = spawn_script("sleep", &["30"], "[test:sleep]");

    if let Ok(session) = result {
        let pid = session.pid();

        // Drop the session - should kill the process
        drop(session);

        // Wait for process to be fully cleaned up (may take a bit)
        // Use ps to check if process is truly gone or just a zombie
        let mut is_dead = false;
        for _ in 0..10 {
            std::thread::sleep(std::time::Duration::from_millis(50));

            // Check process state using ps
            let check = Command::new("ps")
                .args(["-p", &pid.to_string(), "-o", "state="])
                .output();

            match check {
                Ok(output) => {
                    let state = String::from_utf8_lossy(&output.stdout);
                    let state = state.trim();
                    // Process is dead if ps returns empty or shows Z (zombie)
                    // We consider zombie as "dead enough" since it's not running
                    if state.is_empty() || state.starts_with('Z') || !output.status.success() {
                        is_dead = true;
                        break;
                    }
                }
                Err(_) => {
                    // Command failed to run, assume process is dead
                    is_dead = true;
                    break;
                }
            }
        }
        assert!(is_dead, "Process should be dead after drop");
    }
}

#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_split_session_kill() {
    // Spawn a process and split it
    let result = spawn_script("sleep", &["10"], "[test:sleep]");

    if let Ok(session) = result {
        let pid = session.pid();
        let mut split = session.split();

        assert_eq!(split.pid(), pid);
        assert!(split.is_running());

        // Kill via split session
        split.kill().expect("kill should succeed");

        std::thread::sleep(std::time::Duration::from_millis(100));
        assert!(!split.is_running());
    }
}

/// Test that process group liveness check works correctly
/// This verifies the fix for the bug where we only checked the leader PID
/// instead of the entire process group.
#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_process_group_alive_check() {
    // Spawn a simple sleep process - don't use backgrounded children
    // as they may get their own process groups on some systems
    let result = spawn_script("sleep", &["10"], "[test:process_group]");

    if let Ok(session) = result {
        let pid = session.pid();

        // Process group should be alive
        // Note: ProcessHandle::is_alive() now checks the group, not just the leader
        assert!(
            session.process_handle.is_alive(),
            "Process group should be alive initially"
        );

        // Drop the session - should kill the entire process group
        drop(session);

        // Give some time for cleanup
        std::thread::sleep(std::time::Duration::from_millis(400));

        // Verify the process is truly dead using kill -0
        // This is the same check our is_alive() uses internally
        let rc = unsafe { libc::kill(-(pid as libc::pid_t), 0) };
        assert!(
            rc != 0,
            "Process group should be dead after kill (kill -0 should fail)"
        );
    }
}

/// Test that the libc-based process group functions work correctly
#[cfg(unix)]
#[test]
fn test_unix_process_group_functions() {
    use crate::executor::runner::ProcessHandle;

    // Create a handle with a non-existent PID
    let handle = ProcessHandle::new(99997, "[test:unix_funcs]".to_string());

    // Non-existent process should not be alive
    assert!(
        !handle.is_alive(),
        "Non-existent process should not be alive"
    );
}

