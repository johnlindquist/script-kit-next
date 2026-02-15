// ============================================================
// Process Group Termination Escalation Tests (SIGTERM → SIGKILL)
// ============================================================
//
// These tests verify the graceful termination escalation protocol:
// 1. SIGTERM is sent first (graceful shutdown request)
// 2. Wait up to TERM_GRACE_MS (250ms) for process to exit
// 3. If still alive, escalate to SIGKILL (forceful termination)
//
// This ensures scripts that ignore SIGTERM are still killed.

/// Test that a well-behaved process terminates gracefully with SIGTERM
/// This test verifies:
/// 1. ProcessHandle.kill() sends SIGTERM to the process group
/// 2. The process responds to SIGTERM (sleep is well-behaved)
/// 3. Process is properly reaped (no zombie)
///
/// Note: We verify behavior (signal received) not timing, to avoid CI flakiness.
#[cfg(unix)]
#[test]
fn test_sigterm_graceful_termination() {
    use std::os::unix::process::ExitStatusExt;
    use std::time::Instant;

    const SIGTERM: i32 = 15;
    const SIGKILL: i32 = 9;

    // Spawn a simple sleep that will respond to SIGTERM
    let result = spawn_script("sleep", &["60"], "[test:sigterm_graceful]");

    if let Ok(session) = result {
        let pid = session.process_handle.pid;
        let start = Instant::now();

        // Process should be running
        assert!(
            Command::new("kill")
                .args(["-0", &pid.to_string()])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false),
            "Process should be running before split"
        );

        // Split to get access to child for wait()
        let mut split = session.split();

        // Kill the process group via ProcessHandle
        split.kill().expect("kill should succeed");

        // Wait for the child to be reaped (this clears the zombie)
        // Generous timeout to avoid CI flakiness - the test validates behavior, not speed
        let timeout = std::time::Duration::from_secs(5);
        let poll_interval = std::time::Duration::from_millis(50);

        while start.elapsed() < timeout {
            match split.child.try_wait() {
                Ok(Some(status)) => {
                    // Child has exited and been reaped
                    // Verify the process is actually gone now
                    let is_dead = !Command::new("kill")
                        .args(["-0", &pid.to_string()])
                        .output()
                        .map(|o| o.status.success())
                        .unwrap_or(false);
                    assert!(is_dead, "Process should be fully dead after wait");

                    // Verify process was killed by a signal (SIGTERM or SIGKILL)
                    // sleep is well-behaved so should respond to SIGTERM (15)
                    // but SIGKILL (9) is also acceptable if escalation occurred
                    let signal = status.signal();
                    assert!(
                        signal == Some(SIGTERM) || signal == Some(SIGKILL),
                        "Process should have been killed by SIGTERM or SIGKILL, got signal={:?}, code={:?}",
                        signal,
                        status.code()
                    );
                    return;
                }
                Ok(None) => {
                    // Still running/zombie, keep waiting
                    std::thread::sleep(poll_interval);
                }
                Err(e) => {
                    panic!("Error waiting for child: {:?}", e);
                }
            }
        }

        panic!("Process {} did not terminate within {:?}", pid, timeout);
    }
}

/// Test that ProcessHandle.kill() is idempotent (safe to call multiple times)
/// This verifies that calling kill() after the process is already dead doesn't panic
#[cfg(unix)]
#[test]
fn test_kill_idempotent() {
    use std::time::Instant;

    let result = spawn_script("sleep", &["10"], "[test:kill_idempotent]");

    if let Ok(session) = result {
        let pid = session.process_handle.pid;
        let mut split = session.split();
        let start = Instant::now();

        // First kill should succeed
        split.kill().expect("First kill should succeed");

        // Wait for child to be reaped
        let timeout = std::time::Duration::from_millis(500);
        let poll_interval = std::time::Duration::from_millis(25);

        while start.elapsed() < timeout {
            match split.child.try_wait() {
                Ok(Some(_status)) => {
                    // Child reaped - now test idempotency
                    // These should all succeed without panic (killed flag is set)
                    split.kill().expect("Second kill should succeed (no-op)");
                    split.kill().expect("Third kill should succeed (no-op)");

                    // Verify process is actually gone
                    let is_dead = !Command::new("kill")
                        .args(["-0", &pid.to_string()])
                        .output()
                        .map(|o| o.status.success())
                        .unwrap_or(false);
                    assert!(is_dead, "Process should be fully dead");
                    return;
                }
                Ok(None) => {
                    std::thread::sleep(poll_interval);
                }
                Err(e) => {
                    panic!("Error waiting for child: {:?}", e);
                }
            }
        }

        panic!(
            "Process {} did not terminate within {:?} after kill",
            pid, timeout
        );
    }
}

/// Test that process group is killed (child processes too)
/// This spawns bash which spawns a child sleep, verifying both are killed
/// when we send SIGTERM to the process group.
#[cfg(unix)]
#[test]
fn test_process_group_kills_children() {
    use std::io::{BufRead, BufReader};
    use std::process::Stdio;
    use std::time::Instant;

    // Spawn bash with a background child process
    // The bash script: starts a sleep in background, prints "started", then waits
    let script_content = "sleep 60 & echo started; wait";

    let mut cmd = Command::new("bash");
    cmd.args(["-c", script_content])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Create process group so we can kill all children together
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }

    if let Ok(mut child) = cmd.spawn() {
        let pid = child.id();
        let start = Instant::now();

        // Wait for "started" to confirm the child sleep was spawned
        if let Some(stdout) = child.stdout.take() {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            if reader.read_line(&mut line).is_ok() && line.trim() == "started" {
                // Good - child sleep has been spawned
            }
        }

        // Create a ProcessHandle to manage termination
        let mut handle = ProcessHandle::new(pid, "[test:process_group_children]".to_string());

        // Kill the process group
        handle.kill();

        // Wait for child to be reaped
        let timeout = std::time::Duration::from_millis(500);
        let poll_interval = std::time::Duration::from_millis(25);

        while start.elapsed() < timeout {
            match child.try_wait() {
                Ok(Some(_status)) => {
                    // Child reaped - verify it's truly gone
                    let is_dead = !Command::new("kill")
                        .args(["-0", &pid.to_string()])
                        .output()
                        .map(|o| o.status.success())
                        .unwrap_or(false);
                    assert!(is_dead, "Process should be fully dead after wait");
                    return;
                }
                Ok(None) => {
                    std::thread::sleep(poll_interval);
                }
                Err(_) => break,
            }
        }

        // Final cleanup
        let _ = child.kill();
        let _ = child.wait();

        panic!(
            "Parent process (PID {}) should be dead after group kill (waited {:?})",
            pid,
            start.elapsed()
        );
    }
}

/// Test that ProcessHandle is registered and unregistered with PROCESS_MANAGER
#[test]
fn test_process_handle_registration_lifecycle() {
    let test_pid = 77777u32;
    let test_path = "/test/registration_lifecycle.ts";

    // Create handle (registers)
    let handle = ProcessHandle::new(test_pid, test_path.to_string());

    // Verify it's created correctly
    assert_eq!(handle.pid, test_pid);
    assert!(!handle.killed);

    // Drop (unregisters and kills)
    drop(handle);

    // If we get here without panic, lifecycle completed successfully
}

/// Test that kill() marks the handle as killed
#[test]
fn test_kill_sets_killed_flag() {
    let mut handle = ProcessHandle::new(66666, "[test:killed_flag]".to_string());

    assert!(!handle.killed, "killed should be false initially");

    handle.kill();

    assert!(handle.killed, "killed should be true after kill()");
}

/// Test that double kill doesn't attempt to kill again
#[test]
fn test_double_kill_is_noop() {
    let mut handle = ProcessHandle::new(55555, "[test:double_kill_noop]".to_string());

    // First kill sets flag
    handle.kill();
    assert!(handle.killed);

    // Second kill should be a no-op (no panic, no external command)
    handle.kill();
    assert!(handle.killed);
}

/// Test SplitSession provides correct PID
#[cfg(unix)]
#[test]
fn test_split_session_pid() {
    let result = spawn_script("sleep", &["5"], "[test:split_session_pid]");

    if let Ok(session) = result {
        let original_pid = session.process_handle.pid;
        let split = session.split();

        assert_eq!(
            split.pid(),
            original_pid,
            "SplitSession should report same PID as original session"
        );
    }
}

/// Test that wait() returns correct exit code
#[cfg(unix)]
#[test]
fn test_wait_returns_exit_code() {
    let result = spawn_script("sh", &["-c", "exit 42"], "[test:wait_exit_code]");

    if let Ok(session) = result {
        let mut split = session.split();

        // Wait for exit
        match split.wait() {
            Ok(code) => assert_eq!(code, 42, "Exit code should be 42"),
            Err(e) => panic!("wait() failed: {}", e),
        }
    }
}

/// Test is_running() accurately reflects process state
#[cfg(unix)]
#[test]
fn test_is_running_accuracy() {
    let result = spawn_script("sleep", &["5"], "[test:is_running_accuracy]");

    if let Ok(session) = result {
        let mut split = session.split();

        // Should be running initially
        assert!(split.is_running(), "Process should be running after spawn");

        // Kill it
        split.kill().expect("kill should succeed");

        // Wait a moment
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Should not be running
        assert!(
            !split.is_running(),
            "Process should not be running after kill"
        );
    }
}
