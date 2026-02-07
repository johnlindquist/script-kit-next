use std::io;

use super::*;

#[test]
fn test_detect_shell() {
    let shell = PtyManager::detect_shell();
    assert!(!shell.is_empty(), "Shell should not be empty");

    #[cfg(unix)]
    {
        assert!(
            shell.starts_with('/') || shell == "sh",
            "Unix shell should be absolute path or 'sh'"
        );
    }

    #[cfg(windows)]
    {
        let lower = shell.to_lowercase();
        assert!(
            lower.contains("cmd") || lower.contains("powershell"),
            "Windows shell should be cmd or powershell"
        );
    }
}

#[test]
fn test_pty_size_default() {
    let pty = PtyManager::with_command("echo", &["test"]);

    if let Ok(pty) = pty {
        assert_eq!(pty.size(), (80, 24), "Default size should be 80x24");
    }
}

#[test]
fn test_pty_size_custom() {
    let pty = PtyManager::with_command_and_size("echo", &["test"], 120, 40);

    if let Ok(pty) = pty {
        assert_eq!(pty.size(), (120, 40), "Custom size should be 120x40");
    }
}

#[test]
fn test_pty_spawn_and_exit() {
    let pty = PtyManager::with_command("echo", &["hello"]);

    if let Ok(mut pty) = pty {
        let mut buf = [0u8; 1024];
        let mut output = Vec::new();

        loop {
            match pty.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => output.extend_from_slice(&buf[..n]),
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(_) => break,
            }
        }

        let status = pty.wait();
        assert!(status.is_ok(), "Wait should succeed");

        let output_str = String::from_utf8_lossy(&output);
        assert!(
            output_str.contains("hello"),
            "Output should contain 'hello', got: {}",
            output_str
        );
    }
}

#[test]
fn test_pty_write() {
    let pty = PtyManager::with_command("cat", &[]);

    if let Ok(mut pty) = pty {
        let write_result = pty.write(b"test input\n");
        assert!(write_result.is_ok(), "Write should succeed");

        let flush_result = pty.flush();
        assert!(flush_result.is_ok(), "Flush should succeed");

        let _ = pty.kill();
    }
}

#[test]
fn test_pty_resize() {
    let pty = PtyManager::with_command("sleep", &["0.1"]);

    if let Ok(mut pty) = pty {
        let resize_result = pty.resize(100, 50);
        assert!(resize_result.is_ok(), "Resize should succeed");
        assert_eq!(pty.size(), (100, 50), "Size should be updated");

        let _ = pty.kill();
    }
}

#[test]
fn test_pty_is_running() {
    let pty = PtyManager::with_command("sleep", &["10"]);

    if let Ok(mut pty) = pty {
        assert!(pty.is_running(), "Process should be running");

        let _ = pty.kill();
        std::thread::sleep(std::time::Duration::from_millis(100));

        assert!(
            !pty.is_running(),
            "Process should not be running after kill"
        );
    }
}

#[test]
fn test_pty_manager_debug() {
    let pty = PtyManager::with_command("echo", &["test"]);

    if let Ok(pty) = pty {
        let debug_str = format!("{:?}", pty);
        assert!(debug_str.contains("PtyManager"));
        assert!(debug_str.contains("size"));
    }
}
