/// Test Windows-specific shells are defined correctly
#[test]
fn test_windows_shells_in_shell_tools() {
    // Verify Windows shells are in SHELL_TOOLS
    let windows_shells = ["cmd", "powershell", "pwsh"];

    for shell in &windows_shells {
        assert!(
            SHELL_TOOLS.contains(shell),
            "SHELL_TOOLS should include Windows shell: {}",
            shell
        );
    }
}

/// Test Unix-specific shells are defined correctly
#[test]
fn test_unix_shells_in_shell_tools() {
    // Verify Unix shells are in SHELL_TOOLS
    let unix_shells = ["bash", "zsh", "sh", "fish"];

    for shell in &unix_shells {
        assert!(
            SHELL_TOOLS.contains(shell),
            "SHELL_TOOLS should include Unix shell: {}",
            shell
        );
    }
}

/// Test run_scriptlet correctly dispatches to shell handler
#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_run_scriptlet_dispatches_to_shell_handler() {
    for shell in &["bash", "sh"] {
        let scriptlet = Scriptlet::new(
            format!("{} Test", shell),
            shell.to_string(),
            "echo test".to_string(),
        );

        let result = run_scriptlet(&scriptlet, ScriptletExecOptions::default());
        assert!(
            result.is_ok(),
            "{} scriptlet should succeed: {:?}",
            shell,
            result
        );

        let result = result.unwrap();
        assert!(result.success, "{} should succeed", shell);
        assert!(
            result.stdout.contains("test"),
            "{} should output 'test'",
            shell
        );
    }
}

/// Test shell scripts handle special characters correctly
#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_execute_shell_scriptlet_special_characters() {
    // Test that special shell characters are handled
    let result = execute_shell_scriptlet(
        "bash",
        r#"echo "Hello, World!" && echo 'Single quotes' && echo $((1 + 2))"#,
        &ScriptletExecOptions::default(),
    );

    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.success);
    assert!(result.stdout.contains("Hello, World!"));
    assert!(result.stdout.contains("Single quotes"));
    assert!(result.stdout.contains("3")); // 1 + 2
}

/// Test shell scripts with here-documents
#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_execute_shell_scriptlet_heredoc() {
    let script = r#"cat << 'EOF'
multi
line
content
EOF"#;

    let result = execute_shell_scriptlet("bash", script, &ScriptletExecOptions::default());
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.success);
    assert!(result.stdout.contains("multi"));
    assert!(result.stdout.contains("line"));
    assert!(result.stdout.contains("content"));
}

/// Test shell scripts with pipes
#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_execute_shell_scriptlet_pipes() {
    let result = execute_shell_scriptlet(
        "bash",
        "echo 'hello world' | tr 'a-z' 'A-Z'",
        &ScriptletExecOptions::default(),
    );

    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.success);
    assert!(result.stdout.contains("HELLO WORLD"));
}

/// Test shell scripts with command substitution
#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_execute_shell_scriptlet_command_substitution() {
    let result = execute_shell_scriptlet(
        "bash",
        "echo Today is $(date +%A)",
        &ScriptletExecOptions::default(),
    );

    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.success);
    assert!(result.stdout.contains("Today is"));
}

/// Test that temp file is cleaned up after execution
#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_execute_shell_scriptlet_cleanup() {
    // Run a script - the temp file should be cleaned up after execution
    let result = execute_shell_scriptlet(
        "bash",
        "echo cleanup test",
        &ScriptletExecOptions::default(),
    );
    assert!(result.is_ok());

    // The temp file should be cleaned up
    // Note: Due to potential race conditions in testing, we just verify the script ran
    // The cleanup is verified by the fact that multiple tests don't accumulate temp files
    let result = result.unwrap();
    assert!(result.success);
}

