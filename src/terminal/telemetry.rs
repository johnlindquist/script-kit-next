//! Structured terminal / PTY lifecycle telemetry.
//!
//! Every event shares the stable `script_kit::terminal` target so agents can
//! reconstruct the spawn→resize→exit arc of an embedded terminal session with
//! `getLogs {target: "script_kit::terminal"}`. Callers pass a static `source`
//! site string.
//!
//! These helpers log the shell/command name, pid, dimensions, and exit codes
//! only — never anything typed into or emitted by the terminal.

/// A PTY child process was spawned. `command` is the shell/executable name and
/// `pid` is its process id when the backend exposes one.
pub(crate) fn log_pty_spawned(
    source: &'static str,
    command: &str,
    pid: Option<u32>,
    cols: u16,
    rows: u16,
) {
    tracing::info!(
        target: "script_kit::terminal",
        category = "TERMINAL",
        event = "pty_spawned",
        source,
        command,
        pid = ?pid,
        cols,
        rows,
        "terminal_lifecycle"
    );
}

/// A PTY child process exited (observed via `wait`).
pub(crate) fn log_pty_exited(source: &'static str, exit_status: &str) {
    tracing::info!(
        target: "script_kit::terminal",
        category = "TERMINAL",
        event = "pty_exited",
        source,
        exit_status,
        "terminal_lifecycle"
    );
}

/// A PTY child process was explicitly killed.
pub(crate) fn log_pty_killed(source: &'static str) {
    tracing::info!(
        target: "script_kit::terminal",
        category = "TERMINAL",
        event = "pty_killed",
        source,
        "terminal_lifecycle"
    );
}

/// The PTY was resized to new dimensions.
pub(crate) fn log_terminal_resized(source: &'static str, cols: u16, rows: u16) {
    tracing::info!(
        target: "script_kit::terminal",
        category = "TERMINAL",
        event = "terminal_resized",
        source,
        cols,
        rows,
        "terminal_lifecycle"
    );
}

/// The terminal emulator observed its child process exit with `exit_code`.
pub(crate) fn log_terminal_child_exit(source: &'static str, exit_code: i32) {
    tracing::info!(
        target: "script_kit::terminal",
        category = "TERMINAL",
        event = "terminal_child_exit",
        source,
        exit_code,
        "terminal_lifecycle"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_telemetry_helpers_do_not_panic() {
        log_pty_spawned("test", "/bin/zsh", Some(4321), 80, 24);
        log_pty_spawned("test", "/bin/zsh", None, 80, 24);
        log_pty_exited("test", "exit_code=0");
        log_pty_killed("test");
        log_terminal_resized("test", 100, 50);
        log_terminal_child_exit("test", 0);
    }
}
