//! Error Parsing and Suggestion Generation
//!
//! This module provides utilities for parsing script errors and generating
//! helpful suggestions for users.

use crate::utils::truncate_str_chars;

/// Parse stderr output to extract stack trace if present
pub fn parse_stack_trace(stderr: &str) -> Option<String> {
    // Look for common stack trace patterns
    let lines: Vec<&str> = stderr.lines().collect();

    // Find the start of a stack trace (lines starting with "at ")
    let stack_start = lines.iter().position(|line| {
        let trimmed = line.trim();
        trimmed.starts_with("at ")
            || trimmed.contains("    at ")
            || trimmed.starts_with("Error:")
            || trimmed.starts_with("TypeError:")
            || trimmed.starts_with("ReferenceError:")
            || trimmed.starts_with("SyntaxError:")
    });

    if let Some(start) = stack_start {
        // Collect lines that look like stack trace entries
        let stack_lines: Vec<&str> = lines[start..]
            .iter()
            .take_while(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty()
                    && (trimmed.starts_with("at ")
                        || trimmed.contains("    at ")
                        || trimmed.starts_with("Error:")
                        || trimmed.starts_with("TypeError:")
                        || trimmed.starts_with("ReferenceError:")
                        || trimmed.starts_with("SyntaxError:")
                        || trimmed.contains("error")
                        || trimmed.contains("Error"))
            })
            .take(20) // Limit to 20 lines
            .copied()
            .collect();

        if !stack_lines.is_empty() {
            return Some(stack_lines.join("\n"));
        }
    }

    None
}

/// Extract a user-friendly error message from stderr
pub fn extract_error_message(stderr: &str) -> String {
    let lines: Vec<&str> = stderr.lines().collect();

    // Look for common error patterns
    for line in &lines {
        let trimmed = line.trim();

        // Check for error type prefixes
        if trimmed.starts_with("Error:")
            || trimmed.starts_with("TypeError:")
            || trimmed.starts_with("ReferenceError:")
            || trimmed.starts_with("SyntaxError:")
            || trimmed.starts_with("error:")
        {
            return trimmed.to_string();
        }

        // Check for bun-specific errors
        if trimmed.contains("error:") && !trimmed.starts_with("at ") {
            return trimmed.to_string();
        }
    }

    // If no specific error found, return first non-empty line (truncated)
    for line in &lines {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            return if trimmed.chars().count() > 200 {
                format!("{}...", truncate_str_chars(trimmed, 200))
            } else {
                trimmed.to_string()
            };
        }
    }

    "Script execution failed".to_string()
}

/// Generate suggestions based on error type
pub fn generate_suggestions(stderr: &str, exit_code: Option<i32>) -> Vec<String> {
    let mut suggestions = Vec::new();
    let stderr_lower = stderr.to_lowercase();

    // Check for common error patterns and suggest fixes
    if stderr_lower.contains("cannot find module") || stderr_lower.contains("module not found") {
        suggestions.push("Run 'bun install' or 'npm install' to install dependencies".to_string());
    }

    if stderr_lower.contains("syntaxerror") || stderr_lower.contains("unexpected token") {
        suggestions.push("Check for syntax errors in your script".to_string());
    }

    if stderr_lower.contains("referenceerror") || stderr_lower.contains("is not defined") {
        suggestions.push(
            "Check that all variables and functions are properly imported or defined".to_string(),
        );
    }

    if stderr_lower.contains("typeerror") {
        suggestions
            .push("Check that you're using the correct types for function arguments".to_string());
    }

    if stderr_lower.contains("permission denied") || stderr_lower.contains("eacces") {
        suggestions
            .push("Check file permissions or try running with elevated privileges".to_string());
    }

    if stderr_lower.contains("enoent") || stderr_lower.contains("no such file") {
        suggestions.push("Check that the file path exists and is correct".to_string());
    }

    if stderr_lower.contains("timeout") || stderr_lower.contains("timed out") {
        suggestions.push(
            "The operation timed out - check network connectivity or increase timeout".to_string(),
        );
    }

    // Exit code specific suggestions
    match exit_code {
        Some(1) => {
            if suggestions.is_empty() {
                suggestions.push("Check the error message above for details".to_string());
            }
        }
        Some(127) => {
            suggestions.push(
                "Command not found - check that the executable is installed and in PATH"
                    .to_string(),
            );
        }
        Some(126) => {
            suggestions.push("Permission denied - check file permissions".to_string());
        }
        Some(134) => {
            // 128 + 6 = SIGABRT
            suggestions.push(
                "Process aborted (SIGABRT) - check for assertion failures or abort() calls"
                    .to_string(),
            );
        }
        Some(137) => {
            // 128 + 9 = SIGKILL
            suggestions.push(
                "Process was killed (SIGKILL) - possibly out of memory or manually killed"
                    .to_string(),
            );
        }
        Some(139) => {
            // 128 + 11 = SIGSEGV
            suggestions.push(
                "Segmentation fault (SIGSEGV) - memory access violation in native code".to_string(),
            );
        }
        Some(143) => {
            // 128 + 15 = SIGTERM
            suggestions.push("Process was terminated by signal (SIGTERM)".to_string());
        }
        Some(code) if code > 128 => {
            // Other signals: 128 + signal_number
            let signal = code - 128;
            let sig_name = match signal {
                1 => "SIGHUP",
                2 => "SIGINT",
                3 => "SIGQUIT",
                4 => "SIGILL",
                5 => "SIGTRAP",
                6 => "SIGABRT",
                7 => "SIGBUS",
                8 => "SIGFPE",
                10 => "SIGUSR1",
                12 => "SIGUSR2",
                13 => "SIGPIPE",
                14 => "SIGALRM",
                _ => "unknown signal",
            };
            suggestions.push(format!(
                "Process terminated by {} (exit code {})",
                sig_name, code
            ));
        }
        _ => {}
    }

    suggestions
}

/// Information about how a script process crashed
///
/// This struct provides detailed information about process termination,
/// including signal detection on Unix systems. Use `from_exit_status()`
/// to create from a process's exit status.
///
#[derive(Debug, Clone)]
#[allow(dead_code)] // Infrastructure ready for integration into main.rs
pub struct CrashInfo {
    /// Whether the process was terminated by a signal
    pub was_signaled: bool,
    /// The signal number (if was_signaled is true, on Unix)
    pub signal: Option<i32>,
    /// Human-readable signal name (e.g., "SIGKILL", "SIGSEGV")
    pub signal_name: Option<String>,
    /// The exit code (if not signaled)
    pub exit_code: Option<i32>,
    /// Whether this appears to be a crash vs normal exit
    pub is_crash: bool,
}

#[allow(dead_code)] // Infrastructure ready for integration into main.rs
impl CrashInfo {
    /// Create CrashInfo from an ExitStatus
    #[cfg(unix)]
    pub fn from_exit_status(status: std::process::ExitStatus) -> Self {
        use std::os::unix::process::ExitStatusExt;

        let signal = status.signal();
        let was_signaled = signal.is_some();
        let signal_name = signal.map(signal_to_name);
        let exit_code = status.code();

        // Consider it a crash if:
        // - Killed by signal (except SIGTERM which is graceful)
        // - Exit code > 128 (typically indicates signal)
        // - Exit code 1 with no stderr (likely uncaught exception)
        let is_crash =
            was_signaled || exit_code.map(|c| c > 128).unwrap_or(false) || exit_code == Some(1);

        Self {
            was_signaled,
            signal,
            signal_name,
            exit_code,
            is_crash,
        }
    }

    #[cfg(not(unix))]
    pub fn from_exit_status(status: std::process::ExitStatus) -> Self {
        let exit_code = status.code();
        let is_crash = exit_code.map(|c| c != 0).unwrap_or(true);

        Self {
            was_signaled: false,
            signal: None,
            signal_name: None,
            exit_code,
            is_crash,
        }
    }

    /// Create a descriptive error message for this crash
    pub fn error_message(&self) -> String {
        if let Some(ref sig_name) = self.signal_name {
            format!(
                "Script crashed: {} (signal {})",
                sig_name,
                self.signal.unwrap_or(-1)
            )
        } else if let Some(code) = self.exit_code {
            if code > 128 {
                // High exit codes often indicate signal on Unix
                let sig = code - 128;
                format!(
                    "Script crashed: {} (exit code {})",
                    signal_to_name(sig),
                    code
                )
            } else {
                format!("Script exited with error code {}", code)
            }
        } else {
            "Script terminated unexpectedly".to_string()
        }
    }
}

/// Convert a signal number to its name
pub fn signal_to_name(signal: i32) -> String {
    match signal {
        1 => "SIGHUP".to_string(),
        2 => "SIGINT".to_string(),
        3 => "SIGQUIT".to_string(),
        4 => "SIGILL".to_string(),
        5 => "SIGTRAP".to_string(),
        6 => "SIGABRT".to_string(),
        7 => "SIGBUS".to_string(),
        8 => "SIGFPE".to_string(),
        9 => "SIGKILL".to_string(),
        10 => "SIGUSR1".to_string(),
        11 => "SIGSEGV".to_string(),
        12 => "SIGUSR2".to_string(),
        13 => "SIGPIPE".to_string(),
        14 => "SIGALRM".to_string(),
        15 => "SIGTERM".to_string(),
        _ => format!("SIG{}", signal),
    }
}

/// Generate suggestions specifically for crash scenarios
#[allow(dead_code)] // Infrastructure ready for integration into main.rs
pub fn generate_crash_suggestions(crash_info: &CrashInfo) -> Vec<String> {
    let mut suggestions = Vec::new();

    if let Some(signal) = crash_info.signal {
        match signal {
            6 => {
                suggestions.push(
                    "Check for assertion failures or abort() calls in native addons".to_string(),
                );
                suggestions.push("Look for uncaught exceptions that trigger abort".to_string());
            }
            9 => {
                suggestions.push("Process was forcefully killed (SIGKILL)".to_string());
                suggestions.push(
                    "This could be due to: out of memory, manual kill, or system constraints"
                        .to_string(),
                );
            }
            11 => {
                suggestions.push("Segmentation fault - memory access violation".to_string());
                suggestions.push("Check native addons or C++ bindings for memory bugs".to_string());
                suggestions
                    .push("Try running with smaller data sets to identify the issue".to_string());
            }
            15 => {
                suggestions
                    .push("Process received SIGTERM (graceful termination request)".to_string());
            }
            _ => {
                suggestions.push(format!(
                    "Process received signal: {}",
                    signal_to_name(signal)
                ));
            }
        }
    } else if let Some(code) = crash_info.exit_code {
        if code > 128 {
            let implied_signal = code - 128;
            suggestions.extend(generate_crash_suggestions(&CrashInfo {
                was_signaled: true,
                signal: Some(implied_signal),
                signal_name: Some(signal_to_name(implied_signal)),
                exit_code: Some(code),
                is_crash: true,
            }));
        }
    }

    if suggestions.is_empty() {
        suggestions.push("Script exited unexpectedly".to_string());
        suggestions.push("Check script logs for more details".to_string());
    }

    suggestions
}

#[cfg(test)]
mod tests {
    use super::extract_error_message;

    #[test]
    fn test_extract_error_message_does_not_split_utf8_when_fallback_line_is_multibyte() {
        let stderr = format!("{}\n", "🙂".repeat(250));
        let message = extract_error_message(&stderr);

        assert!(message.ends_with("..."));
        assert_eq!(message.trim_end_matches("...").chars().count(), 200);
        assert!(std::str::from_utf8(message.as_bytes()).is_ok());
    }
}
