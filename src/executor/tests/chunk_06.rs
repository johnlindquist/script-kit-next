// ============================================================
// Shell Tool Execution Tests
// ============================================================
//
// Tests for execute_shell_scriptlet() function and SHELL_TOOLS constant.
// These tests verify shell tool execution, error handling, and platform guards.

use super::execute_shell_scriptlet;
use crate::scriptlets::SHELL_TOOLS;

/// Verify SHELL_TOOLS constant contains all expected shells
#[test]
fn test_shell_tools_contains_expected_shells() {
    // Unix shells
    assert!(
        SHELL_TOOLS.contains(&"bash"),
        "SHELL_TOOLS should include bash"
    );
    assert!(
        SHELL_TOOLS.contains(&"zsh"),
        "SHELL_TOOLS should include zsh"
    );
    assert!(SHELL_TOOLS.contains(&"sh"), "SHELL_TOOLS should include sh");
    assert!(
        SHELL_TOOLS.contains(&"fish"),
        "SHELL_TOOLS should include fish"
    );

    // Windows shells
    assert!(
        SHELL_TOOLS.contains(&"cmd"),
        "SHELL_TOOLS should include cmd"
    );
    assert!(
        SHELL_TOOLS.contains(&"powershell"),
        "SHELL_TOOLS should include powershell"
    );
    assert!(
        SHELL_TOOLS.contains(&"pwsh"),
        "SHELL_TOOLS should include pwsh"
    );
}

/// Verify SHELL_TOOLS has exactly 7 shells (no duplicates, no extras)
#[test]
fn test_shell_tools_count() {
    assert_eq!(
        SHELL_TOOLS.len(),
        7,
        "SHELL_TOOLS should have exactly 7 shells"
    );
}

/// Test successful shell execution returns correct exit code and stdout
#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_execute_shell_scriptlet_success_exit_code() {
    let result = execute_shell_scriptlet("bash", "exit 0", &ScriptletExecOptions::default());
    assert!(result.is_ok(), "Expected success, got: {:?}", result);

    let result = result.unwrap();
    assert_eq!(result.exit_code, 0, "Exit code should be 0");
    assert!(result.success, "success flag should be true");
}

/// Test shell execution captures stdout correctly
#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_execute_shell_scriptlet_captures_stdout() {
    let result = execute_shell_scriptlet(
        "bash",
        "echo 'test output'",
        &ScriptletExecOptions::default(),
    );
    assert!(result.is_ok(), "Expected success, got: {:?}", result);

    let result = result.unwrap();
    assert!(
        result.stdout.contains("test output"),
        "stdout should contain 'test output', got: '{}'",
        result.stdout
    );
    assert!(
        result.stderr.is_empty() || !result.stderr.contains("error"),
        "stderr should be empty or not contain 'error': '{}'",
        result.stderr
    );
}

/// Test shell execution captures stderr correctly
#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_execute_shell_scriptlet_captures_stderr() {
    let result = execute_shell_scriptlet(
        "bash",
        "echo 'error message' >&2",
        &ScriptletExecOptions::default(),
    );
    assert!(result.is_ok(), "Expected success, got: {:?}", result);

    let result = result.unwrap();
    assert!(
        result.stderr.contains("error message"),
        "stderr should contain 'error message', got: '{}'",
        result.stderr
    );
}

/// Test non-zero exit code is captured correctly
#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_execute_shell_scriptlet_nonzero_exit_code() {
    let result = execute_shell_scriptlet("bash", "exit 42", &ScriptletExecOptions::default());
    assert!(
        result.is_ok(),
        "Expected success (script ran, just non-zero exit), got: {:?}",
        result
    );

    let result = result.unwrap();
    assert_eq!(result.exit_code, 42, "Exit code should be 42");
    assert!(
        !result.success,
        "success flag should be false for non-zero exit"
    );
}

/// Test script syntax errors are captured in stderr
#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_execute_shell_scriptlet_syntax_error_captured() {
    // Intentional syntax error: unclosed quote
    let result =
        execute_shell_scriptlet("bash", "echo 'unclosed", &ScriptletExecOptions::default());
    assert!(
        result.is_ok(),
        "Script should run (even if shell reports error)"
    );

    let result = result.unwrap();
    // Syntax errors in bash result in non-zero exit
    assert!(!result.success, "Syntax error should result in failure");
    // The error message should appear in stderr
    assert!(
        !result.stderr.is_empty(),
        "stderr should contain error for syntax error, got: '{}'",
        result.stderr
    );
}

/// Test undefined variable doesn't cause hard failure (just empty expansion)
#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_execute_shell_scriptlet_undefined_variable() {
    // By default, bash doesn't fail on undefined variables
    let result = execute_shell_scriptlet(
        "bash",
        "echo $UNDEFINED_VAR_12345",
        &ScriptletExecOptions::default(),
    );
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(
        result.success,
        "Undefined var should not cause failure by default"
    );
    assert_eq!(result.exit_code, 0);
}

/// Test strict mode catches undefined variables
#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_execute_shell_scriptlet_strict_mode_undefined_var() {
    // set -u makes bash fail on undefined variables
    let result = execute_shell_scriptlet(
        "bash",
        "set -u; echo $UNDEFINED_VAR_12345",
        &ScriptletExecOptions::default(),
    );
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(!result.success, "Undefined var with set -u should fail");
    assert!(
        result.stderr.contains("UNDEFINED_VAR_12345") || result.stderr.contains("unbound"),
        "stderr should mention the undefined variable: '{}'",
        result.stderr
    );
}

/// Test command not found error message
#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_execute_shell_scriptlet_command_not_found() {
    let result = execute_shell_scriptlet(
        "bash",
        "nonexistent_command_xyz123",
        &ScriptletExecOptions::default(),
    );
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(!result.success, "Command not found should fail");
    assert!(
        result.exit_code == 127 || result.exit_code != 0,
        "Exit code should indicate failure (typically 127): {}",
        result.exit_code
    );
    assert!(
        result.stderr.contains("not found") || result.stderr.contains("command not found"),
        "stderr should indicate command not found: '{}'",
        result.stderr
    );
}

/// Test missing shell executable returns helpful error
#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_execute_shell_scriptlet_missing_shell() {
    // Try to use a non-existent shell
    let result = execute_shell_scriptlet(
        "nonexistent_shell_xyz123",
        "echo test",
        &ScriptletExecOptions::default(),
    );

    // This should return an error (not Ok with failure) since the shell itself doesn't exist
    assert!(
        result.is_err(),
        "Missing shell should return Err, got: {:?}",
        result
    );

    let err = result.unwrap_err();
    // Error message should be helpful
    assert!(
        err.contains("Failed to execute") || err.contains("nonexistent_shell"),
        "Error should mention the missing shell: '{}'",
        err
    );
}

/// Test sh shell works (most basic POSIX shell)
#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_execute_shell_scriptlet_sh() {
    let result =
        execute_shell_scriptlet("sh", "echo hello from sh", &ScriptletExecOptions::default());
    assert!(result.is_ok(), "sh should work: {:?}", result);

    let result = result.unwrap();
    assert!(result.success);
    assert!(result.stdout.contains("hello from sh"));
}

/// Test zsh shell works (if available)
#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_execute_shell_scriptlet_zsh() {
    // zsh might not be installed, so we check first
    let check = std::process::Command::new("which").arg("zsh").output();

    if check.is_ok() && check.unwrap().status.success() {
        let result = execute_shell_scriptlet(
            "zsh",
            "echo hello from zsh",
            &ScriptletExecOptions::default(),
        );
        assert!(result.is_ok(), "zsh should work: {:?}", result);

        let result = result.unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("hello from zsh"));
    }
    // If zsh not installed, skip test (don't fail)
}

/// Test fish shell works (if available)
#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_execute_shell_scriptlet_fish() {
    // fish might not be installed, so we check first
    let check = std::process::Command::new("which").arg("fish").output();

    if check.is_ok() && check.unwrap().status.success() {
        // fish has slightly different syntax
        let result = execute_shell_scriptlet(
            "fish",
            "echo hello from fish",
            &ScriptletExecOptions::default(),
        );
        assert!(result.is_ok(), "fish should work: {:?}", result);

        let result = result.unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("hello from fish"));
    }
    // If fish not installed, skip test (don't fail)
}

/// Test cwd option changes working directory for shell scripts
#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_execute_shell_scriptlet_with_cwd() {
    let options = ScriptletExecOptions {
        cwd: Some(PathBuf::from("/tmp")),
        ..Default::default()
    };

    let result = execute_shell_scriptlet("bash", "pwd", &options);
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.success);
    // /tmp might be symlinked to /private/tmp on macOS
    assert!(
        result.stdout.contains("/tmp") || result.stdout.contains("/private/tmp"),
        "CWD should be /tmp, got: {}",
        result.stdout
    );
}

/// Test multiline scripts work correctly
#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_execute_shell_scriptlet_multiline() {
    let script = r#"
echo "line 1"
echo "line 2"
echo "line 3"
"#;

    let result = execute_shell_scriptlet("bash", script, &ScriptletExecOptions::default());
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.success);
    assert!(result.stdout.contains("line 1"));
    assert!(result.stdout.contains("line 2"));
    assert!(result.stdout.contains("line 3"));
}

/// Test environment variable access works in shell scripts
#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_execute_shell_scriptlet_environment() {
    // HOME should always be set
    let result = execute_shell_scriptlet("bash", "echo $HOME", &ScriptletExecOptions::default());
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.success);
    // HOME should not be empty
    assert!(!result.stdout.trim().is_empty(), "HOME should be set");
}

/// Test Windows shells return appropriate error on Unix
#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_execute_shell_scriptlet_windows_shell_on_unix() {
    // cmd.exe doesn't exist on Unix
    let result = execute_shell_scriptlet("cmd", "echo test", &ScriptletExecOptions::default());

    // This should fail because cmd doesn't exist
    assert!(
        result.is_err() || !result.as_ref().unwrap().success,
        "cmd should fail on Unix: {:?}",
        result
    );
}

/// Test powershell on Unix (might be installed as pwsh)
#[cfg(all(unix, feature = "slow-tests"))]
#[test]
fn test_execute_shell_scriptlet_powershell_on_unix() {
    // Check if pwsh is installed (PowerShell Core)
    let pwsh_check = std::process::Command::new("which").arg("pwsh").output();

    let has_pwsh = pwsh_check.is_ok() && pwsh_check.unwrap().status.success();

    if has_pwsh {
        // pwsh should work if installed
        let result = execute_shell_scriptlet(
            "pwsh",
            "Write-Output 'hello from pwsh'",
            &ScriptletExecOptions::default(),
        );
        assert!(
            result.is_ok(),
            "pwsh should work if installed: {:?}",
            result
        );

        let result = result.unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("hello from pwsh"));
    } else {
        // If not installed, it should fail
        let result = execute_shell_scriptlet(
            "pwsh",
            "Write-Output 'test'",
            &ScriptletExecOptions::default(),
        );
        assert!(
            result.is_err() || !result.as_ref().unwrap().success,
            "pwsh should fail if not installed: {:?}",
            result
        );
    }
}

