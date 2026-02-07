// ============================================================
// Shell Not Found Suggestions Tests
// ============================================================

use super::shell_not_found_suggestions;

/// Test that suggestions are provided for each shell type
#[test]
fn test_shell_not_found_suggestions_bash() {
    let suggestions = shell_not_found_suggestions("bash");
    assert!(suggestions.contains("bash"), "Should mention bash");
    assert!(suggestions.contains("PATH"), "Should mention PATH");
    assert!(
        suggestions.contains("SHELL_TOOLS"),
        "Should mention SHELL_TOOLS alternatives"
    );
}

#[test]
fn test_shell_not_found_suggestions_zsh() {
    let suggestions = shell_not_found_suggestions("zsh");
    assert!(suggestions.contains("zsh"), "Should mention zsh");
    assert!(suggestions.contains("PATH"), "Should mention PATH");
}

#[test]
fn test_shell_not_found_suggestions_sh() {
    let suggestions = shell_not_found_suggestions("sh");
    assert!(suggestions.contains("sh"), "Should mention sh");
    assert!(
        suggestions.contains("POSIX") || suggestions.contains("PATH"),
        "Should mention POSIX or PATH"
    );
}

#[test]
fn test_shell_not_found_suggestions_fish() {
    let suggestions = shell_not_found_suggestions("fish");
    assert!(suggestions.contains("fish"), "Should mention fish");
    assert!(
        suggestions.contains("fishshell.com") || suggestions.contains("brew"),
        "Should provide installation hint"
    );
}

#[test]
fn test_shell_not_found_suggestions_cmd() {
    let suggestions = shell_not_found_suggestions("cmd");
    assert!(suggestions.contains("cmd"), "Should mention cmd");
    // On Unix, should suggest using Unix shells instead
    #[cfg(unix)]
    {
        assert!(
            suggestions.contains("Windows-only") || suggestions.contains("bash"),
            "Should mention cmd is Windows-only on Unix"
        );
    }
}

#[test]
fn test_shell_not_found_suggestions_powershell() {
    let suggestions = shell_not_found_suggestions("powershell");
    assert!(
        suggestions.contains("powershell") || suggestions.contains("PowerShell"),
        "Should mention powershell"
    );
}

#[test]
fn test_shell_not_found_suggestions_pwsh() {
    let suggestions = shell_not_found_suggestions("pwsh");
    assert!(
        suggestions.contains("PowerShell"),
        "Should mention PowerShell Core"
    );
    assert!(
        suggestions.contains("install-powershell"),
        "Should provide install link"
    );
}

#[test]
fn test_shell_not_found_suggestions_unknown() {
    let suggestions = shell_not_found_suggestions("unknown_shell");
    assert!(
        suggestions.contains("unknown_shell"),
        "Should mention the shell name"
    );
    assert!(
        suggestions.contains("not recognized") || suggestions.contains("PATH"),
        "Should suggest checking PATH"
    );
    assert!(
        suggestions.contains("SHELL_TOOLS"),
        "Should mention alternatives"
    );
}

/// Test that error message includes suggestions when shell is not found
#[cfg(unix)]
#[test]
fn test_execute_shell_scriptlet_error_includes_suggestions() {
    let result = execute_shell_scriptlet(
        "nonexistent_shell_xyz",
        "echo test",
        &ScriptletExecOptions::default(),
    );
    assert!(result.is_err(), "Should fail for nonexistent shell");

    let err = result.unwrap_err();
    assert!(
        err.contains("Suggestions"),
        "Error should include suggestions section"
    );
    assert!(err.contains("PATH"), "Error should mention PATH");
    assert!(
        err.contains("SHELL_TOOLS"),
        "Error should mention SHELL_TOOLS alternatives"
    );
}

