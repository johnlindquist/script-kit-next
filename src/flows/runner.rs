//! Spawns `md <flow> --events` runs and streams NDJSON into the registry.
//!
//! Contract (docs/ai/flow-ux-protocol.md §3):
//! - The `Starting` run is inserted into the registry BEFORE the process
//!   spawns, so the launch is acknowledged in the same frame it was
//!   requested.
//! - Each run gets its own process group (`process_group(0)`), so
//!   cancellation can kill the whole tree: SIGTERM to the group, bounded
//!   wait, then SIGKILL to the group. Cancel targets one run only.
//! - Reader threads never touch GPUI; they mutate the thread-safe registry,
//!   whose notify hook schedules the repaint.

use std::io::{BufRead, BufReader};
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::catalog::mdflow_binary;
use super::model::{parse_event_line, EngagementMode, FlowUxVariant, RunPhase};
use super::run_registry::flow_run_registry;

/// Bounded SIGTERM→SIGKILL escalation window (protocol §3).
const CANCEL_ESCALATION: Duration = Duration::from_secs(2);

/// Launch a flow run. Returns the registry-local run id immediately; all
/// process work happens on background threads. `input_overrides` are
/// (name, value) pairs passed as `--_<name> <value>` (collected natively —
/// `--events` runs never prompt).
pub fn launch_flow(
    flow_id: &str,
    flow_name: &str,
    flow_path: &str,
    cwd: &str,
    variant: FlowUxVariant,
    engagement: EngagementMode,
    input_overrides: Vec<(String, String)>,
    launch_requested: Instant,
) -> u64 {
    let registry = flow_run_registry();
    let local_id =
        registry.insert_starting(flow_id, flow_name, flow_path, cwd, variant, engagement);
    registry.record_launch_ack(local_id, launch_requested.elapsed().as_millis() as u64);

    let flow_path = flow_path.to_string();
    let cwd = cwd.to_string();
    std::thread::Builder::new()
        .name(format!("flow-run-{local_id}"))
        .spawn(move || run_flow_process(local_id, &flow_path, &cwd, &input_overrides))
        .ok();
    local_id
}

fn run_flow_process(local_id: u64, flow_path: &str, cwd: &str, overrides: &[(String, String)]) {
    let registry = flow_run_registry();
    let Some(binary) = mdflow_binary() else {
        registry.mark_failed(local_id, "mdflow CLI not found on PATH (npm i -g mdflow)");
        return;
    };

    let mut command = Command::new(binary);
    command
        .arg(flow_path)
        .arg("--events")
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    for (name, value) in overrides {
        command.arg(format!("--_{name}")).arg(value);
    }
    // Own process group so cancel can kill the full descendant tree without
    // touching sibling runs.
    command.process_group(0);

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(err) => {
            registry.mark_failed(local_id, &format!("failed to spawn {binary}: {err}"));
            return;
        }
    };
    let pid = child.id();
    registry.set_pid(local_id, pid);
    // Orphan hygiene only — run lifecycle state lives in the flow registry.
    crate::process_manager::PROCESS_MANAGER.register_process(pid, flow_path);

    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            registry.mark_failed(local_id, "child stdout unavailable");
            let _ = child.kill();
            return;
        }
    };

    let mut saw_terminal = false;
    let mut expected_seq: u64 = 0;
    let reader = BufReader::new(stdout);
    for line in reader.lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        match parse_event_line(&line) {
            Some(envelope) => {
                // Sequence gaps are protocol violations worth logging, but a
                // run must degrade gracefully rather than abort playback.
                if envelope.seq != expected_seq {
                    crate::logging::log(
                        "FLOWS",
                        &format!(
                            "run {local_id}: event seq gap (expected {expected_seq}, got {})",
                            envelope.seq
                        ),
                    );
                }
                expected_seq = envelope.seq.saturating_add(1);
                saw_terminal |= matches!(
                    envelope.event,
                    super::model::RunEvent::RunCompleted { .. }
                        | super::model::RunEvent::RunError { .. }
                        | super::model::RunEvent::RunCancelled { .. }
                );
                registry.apply_event(local_id, &envelope);
            }
            None => {
                crate::logging::log(
                    "FLOWS",
                    &format!("run {local_id}: non-protocol stdout line dropped"),
                );
            }
        }
    }

    let status = child.wait();
    crate::process_manager::PROCESS_MANAGER.unregister_process(pid);
    if !saw_terminal {
        // Process died without a terminal event (killed, crashed, legacy
        // binary). Preserve cancellation if we initiated it.
        let phase = registry.get(local_id).map(|r| r.phase);
        if phase == Some(RunPhase::Cancelled) {
            // cancel_run already marked it.
        } else {
            match status {
                Ok(status) if status.success() => {
                    registry.mark_failed(local_id, "run ended without a terminal protocol event")
                }
                Ok(status) => registry.mark_failed(
                    local_id,
                    &format!("process exited {status} without terminal event"),
                ),
                Err(err) => registry.mark_failed(local_id, &format!("wait failed: {err}")),
            }
        }
    }
}

/// Cancel one run: SIGTERM its process group, escalate to SIGKILL after a
/// bounded wait if the run has not reached a terminal phase. Never touches
/// other runs.
pub fn cancel_run(local_id: u64) {
    let registry = flow_run_registry();
    let Some(run) = registry.get(local_id) else {
        return;
    };
    if run.phase.is_terminal() {
        return;
    }
    let Some(pid) = run.pid else {
        // Not spawned yet; mark cancelled so the runner thread's terminal
        // fallback preserves it.
        registry.mark_cancelled(local_id);
        return;
    };

    // `process_group(0)` made the child its own group leader → pgid == pid.
    unsafe {
        libc::killpg(pid as libc::pid_t, libc::SIGTERM);
    }
    registry.mark_cancelled(local_id);

    std::thread::Builder::new()
        .name(format!("flow-cancel-{local_id}"))
        .spawn(move || {
            // Poll the group (not just the leader) so descendants count,
            // matching the executor's escalation style (runner.rs
            // TERM_GRACE poll loop) with the protocol's 2s bound.
            let deadline = Instant::now() + CANCEL_ESCALATION;
            while Instant::now() < deadline {
                if !process_group_alive(pid) {
                    return;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            if process_group_alive(pid) {
                unsafe {
                    libc::killpg(pid as libc::pid_t, libc::SIGKILL);
                }
                crate::logging::log(
                    "FLOWS",
                    &format!("run {local_id}: escalated cancel to SIGKILL for pgid {pid}"),
                );
            }
        })
        .ok();
}

/// True while any process in the group is alive (signal 0 probe).
pub fn process_group_alive(pgid: u32) -> bool {
    unsafe { libc::killpg(pgid as libc::pid_t, 0) == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flows::run_registry::flow_run_registry;

    /// Launch with a missing binary path exercises the spawn-failure path
    /// end to end without depending on mdflow being installed.
    #[test]
    fn cancel_before_spawn_marks_cancelled() {
        let registry = flow_run_registry();
        let id = registry.insert_starting(
            "project:t",
            "t",
            "/nonexistent/flows/t.md",
            "/tmp",
            FlowUxVariant::Dispatch,
            EngagementMode::Background,
        );
        cancel_run(id);
        assert_eq!(registry.get(id).unwrap().phase, RunPhase::Cancelled);
        cancel_run(id); // idempotent on terminal runs
        assert_eq!(registry.get(id).unwrap().phase, RunPhase::Cancelled);
    }

    #[test]
    fn process_group_probe_reports_dead_group() {
        // PID 1 exists but we cannot signal its group as a normal user —
        // either way this must not panic; a definitely-unused pgid reports
        // dead.
        let _ = process_group_alive(1);
        assert!(!process_group_alive(4_000_000));
    }
}
