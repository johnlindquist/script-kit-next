//! Structured script-execution lifecycle telemetry.
//!
//! Every event shares the stable `script_kit::execution` target so agents and
//! log parsers can reconstruct the full spawn→exit arc of a script run with a
//! single `getLogs {target: "script_kit::execution"}` filter. Callers pass a
//! static `source` site string so the emitting call site is always identifiable.
//!
//! These helpers deliberately log only paths, pids, exit codes, counts, and
//! signals — never script arguments, stdin/stdout payloads, or user content.

/// A script process was launched. `script_path` is the on-disk path (safe to
/// log); `pid` is the spawned process id.
pub(crate) fn log_script_spawned(source: &'static str, script_path: &str, pid: u32) {
    tracing::info!(
        target: "script_kit::execution",
        category = "EXEC",
        event = "script_spawned",
        source,
        script_path,
        pid,
        "execution_lifecycle"
    );
}

/// A script process exited on its own. `duration_ms` is `None` when the run
/// duration was not tracked at the observing call site.
pub(crate) fn log_script_exited(
    source: &'static str,
    pid: u32,
    exit_code: i32,
    duration_ms: Option<u64>,
) {
    tracing::info!(
        target: "script_kit::execution",
        category = "EXEC",
        event = "script_exited",
        source,
        pid,
        exit_code,
        duration_ms = ?duration_ms,
        "execution_lifecycle"
    );
}

/// A script process was explicitly killed (SIGTERM/SIGKILL escalation) rather
/// than exiting on its own.
pub(crate) fn log_script_killed(source: &'static str, pid: u32, signal: &'static str) {
    tracing::info!(
        target: "script_kit::execution",
        category = "EXEC",
        event = "script_killed",
        source,
        pid,
        signal,
        "execution_lifecycle"
    );
}

/// The active interactive script session was replaced by a new launch. When a
/// previous session was still tracked, `previous_pid` carries its pid.
///
/// Only called from `execute_script/mod.rs`, which is `include!`d into the
/// binary target (not the library), so the lib build sees it as unused.
#[allow(dead_code)]
pub(crate) fn log_session_replaced(source: &'static str, previous_pid: Option<u32>, new_pid: u32) {
    tracing::info!(
        target: "script_kit::execution",
        category = "EXEC",
        event = "session_replaced",
        source,
        previous_pid = ?previous_pid,
        new_pid,
        "execution_lifecycle"
    );
}

/// The scheduler dispatched a due script for a run.
pub(crate) fn log_scheduler_job_fired(source: &'static str, script_path: &str) {
    tracing::info!(
        target: "script_kit::execution",
        category = "EXEC",
        event = "scheduler_job_fired",
        source,
        script_path,
        "execution_lifecycle"
    );
}

/// The scheduler skipped dispatching a due script (e.g. concurrency limit).
pub(crate) fn log_scheduler_job_skipped(
    source: &'static str,
    script_path: &str,
    reason: &'static str,
) {
    tracing::info!(
        target: "script_kit::execution",
        category = "EXEC",
        event = "scheduler_job_skipped",
        source,
        script_path,
        reason,
        "execution_lifecycle"
    );
}

/// A macOS menu-bar action was executed against an application. `menu_depth` is
/// the number of path segments (e.g. `["File", "New Window"]` => 2).
pub(crate) fn log_menu_action_executed(source: &'static str, bundle_id: &str, menu_depth: usize) {
    tracing::info!(
        target: "script_kit::execution",
        category = "EXEC",
        event = "menu_action_executed",
        source,
        bundle_id,
        menu_depth,
        "execution_lifecycle"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execution_telemetry_helpers_do_not_panic() {
        log_script_spawned("test", "/tmp/x.ts", 1234);
        log_script_exited("test", 1234, 0, Some(42));
        log_script_exited("test", 1234, -1, None);
        log_script_killed("test", 1234, "SIGTERM");
        log_session_replaced("test", Some(1000), 1234);
        log_session_replaced("test", None, 1234);
        log_scheduler_job_fired("test", "/tmp/x.ts");
        log_scheduler_job_skipped("test", "/tmp/x.ts", "concurrency_limit");
        log_menu_action_executed("test", "com.apple.Safari", 2);
    }
}
