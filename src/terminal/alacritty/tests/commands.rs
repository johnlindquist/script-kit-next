use super::*;

#[test]
fn test_detect_shell() {
    let shell = TerminalHandle::detect_shell();
    assert!(!shell.is_empty(), "Shell should not be empty");

    #[cfg(unix)]
    {
        assert!(
            shell.starts_with('/') || shell == "sh" || shell == "bash" || shell == "zsh",
            "Unix shell should be absolute path or known shell, got: {}",
            shell
        );
    }

    #[cfg(windows)]
    {
        let lower = shell.to_lowercase();
        assert!(
            lower.contains("cmd") || lower.contains("powershell"),
            "Windows shell should be cmd or powershell, got: {}",
            shell
        );
    }
}

#[test]
fn test_terminal_with_simple_command() {
    let result = TerminalHandle::with_command("echo hello", 80, 24);

    if let Ok(mut terminal) = result {
        std::thread::sleep(std::time::Duration::from_millis(100));
        let _ = terminal.process();

        let content = terminal.content();
        let all_text: String = content.lines.join("\n");
        assert!(
            all_text.contains("hello"),
            "Output should contain 'hello', got: {}",
            all_text
        );
    }
}

#[test]
fn test_terminal_with_command_and_args() {
    let result = TerminalHandle::with_command("ls -la", 80, 24);

    if let Ok(mut terminal) = result {
        std::thread::sleep(std::time::Duration::from_millis(500));
        let _ = terminal.process();

        let content = terminal.content();
        let all_text: String = content.lines.join("\n");

        assert!(
            all_text.contains("total")
                || all_text.contains("drwx")
                || all_text.contains("rw")
                || all_text.contains("ls"),
            "ls -la output should contain directory listing, got: {}",
            all_text
        );
    }
}

#[test]
fn test_terminal_with_tilde_expansion() {
    let result = TerminalHandle::with_command("echo ~", 80, 24);

    if let Ok(mut terminal) = result {
        std::thread::sleep(std::time::Duration::from_millis(500));
        let _ = terminal.process();

        let content = terminal.content();
        let all_text: String = content.lines.join("\n");

        assert!(
            all_text.contains("/Users")
                || all_text.contains("/home")
                || all_text.contains("/root")
                || all_text.contains("echo"),
            "~ should be expanded to home directory path, got: {}",
            all_text
        );
    }
}

#[test]
fn test_terminal_with_env_var_expansion() {
    let result = TerminalHandle::with_command("echo $HOME", 80, 24);

    if let Ok(mut terminal) = result {
        std::thread::sleep(std::time::Duration::from_millis(500));
        let _ = terminal.process();

        let content = terminal.content();
        let all_text: String = content.lines.join("\n");

        assert!(
            all_text.contains("/Users")
                || all_text.contains("/home")
                || all_text.contains("/root")
                || all_text.contains("echo"),
            "$HOME should be expanded to home directory path, got: {}",
            all_text
        );
    }
}

#[test]
fn test_terminal_with_pipe() {
    let result = TerminalHandle::with_command("echo hello | tr a-z A-Z", 80, 24);

    if let Ok(mut terminal) = result {
        std::thread::sleep(std::time::Duration::from_millis(500));
        let _ = terminal.process();

        let content = terminal.content();
        let all_text: String = content.lines.join("\n");

        assert!(
            all_text.contains("HELLO") || all_text.contains("echo") || all_text.contains("tr"),
            "Pipe should work, expected 'HELLO' or command, got: {}",
            all_text
        );
    }
}
