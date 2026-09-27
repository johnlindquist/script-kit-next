//! Source-level contract tests for `scripts/agentic/session.sh` session
//! ownership. A healthy PID file is not enough when stale forwarders are
//! still reading the same input FIFO.
//!
//! Run 10's first agentic-testing pass found `session.sh status default`
//! reporting healthy while `send --await-parse`, `getState`, and
//! `surface-proof --kind main` all timed out. The live machine had many
//! old forwarder/cat processes reading `/tmp/sk-agentic-sessions/default/input`,
//! so writes to the current session FIFO could be consumed by an orphan
//! before the active app saw them. Each session now records its forwarder's
//! start identity and stops exactly that process.

const SESSION_SH: &str = include_str!("../scripts/agentic/session.sh");

fn shell_function_body(name: &str) -> &'static str {
    let opener = format!("\n{name}() {{\n");
    let start = SESSION_SH
        .find(&opener)
        .unwrap_or_else(|| panic!("session.sh must define {name}()"));
    let body = &SESSION_SH[start + 1..];
    let end = body
        .find("\n}\n")
        .unwrap_or_else(|| panic!("session.sh {name}() must close"));
    &body[..end]
}

#[test]
fn startup_ready_marker_matches_compact_log_shape() {
    assert!(
        SESSION_SH.contains("READY_LOG_MARKER_STARTUP=\"STARTUP_READY \""),
        "session.sh must match the compact log line emitted by \
         app_run_setup.rs (`STARTUP_READY profile=...`). The older \
         `|STARTUP|STARTUP_READY` marker never appears in SCRIPT_KIT_AI_LOG=1 \
         output, so session start reported ready=false even after startup."
    );
}

#[test]
fn startup_forwarder_does_not_depend_on_process_substitution() {
    let forwarder_start = SESSION_SH
        .find("# Background forwarder: reads from input_fifo and writes to pipe.")
        .expect("session.sh must contain the background forwarder block");
    let launch_start = SESSION_SH
        .find("# Launch the app reading from the pipe after the forwarder has opened the")
        .expect("session.sh must launch the app after the forwarder block");
    let forwarder_block = &SESSION_SH[forwarder_start..launch_start];

    assert!(
        forwarder_block.contains("while [ -p \"$input_fifo\" ]; do")
            && forwarder_block.contains("IFS= read -r line <&3")
            && forwarder_block.contains("done > \"$pipe_path\""),
        "session.sh's background forwarder must own the primary pipe writer \
         while reading the input FIFO. A process-substitution \
         reader can die as the start shell exits, closing the app stdin pipe \
         immediately after startup."
    );
    assert!(
        !forwarder_block.contains("done < <("),
        "session.sh must not use process substitution for the persistent \
         input forwarder. The forwarder itself owns the primary pipe writer \
         and should not depend on a helper process that can vanish."
    );
}

#[test]
fn start_and_stop_stop_only_the_recorded_forwarder_at_ownership_boundaries() {
    let stop_exact = shell_function_body("stop_exact_forwarder");
    let identity = stop_exact
        .find("[[ -n \"$expected\" && \"$expected\" == \"$actual\" ]]")
        .expect("stop_exact_forwarder must prove the forwarder's recorded start identity");
    let kill = stop_exact
        .find("kill -TERM \"$pid\"")
        .expect("stop_exact_forwarder must stop the recorded forwarder");
    assert!(
        identity < kill,
        "the forwarder is signalled only after its start identity matches the one this session recorded"
    );

    let start = shell_function_body("cmd_start");
    assert!(
        start.contains("> \"${sdir}/fwd-start\""),
        "cmd_start must record the new forwarder's start identity"
    );
    assert!(
        start.contains("stop_exact_forwarder \"$sdir\" || return 1"),
        "cmd_start must stop the stale session's own forwarder before replacing it"
    );
    assert!(
        shell_function_body("cmd_stop").contains("stop_exact_forwarder \"$sdir\" || return 1"),
        "cmd_stop must stop the session's own forwarder before removing its FIFOs"
    );
}

#[test]
fn session_errors_include_lifecycle_reason_and_keep_open_metadata() {
    for needle in [
        "lifecycle.ndjson",
        "json_lifecycle_error",
        "app_process_dead_before_send",
        "app_process_dead_before_rpc",
        "forwarder_dead_before_send",
        "forwarder_dead_before_rpc",
        "keepActionsWindowOpen",
    ] {
        assert!(
            SESSION_SH.contains(needle),
            "session.sh lifecycle errors must include {needle}"
        );
    }
}

#[test]
fn start_sends_startup_keepalive_before_waiting_for_ready() {
    assert!(
        SESSION_SH.contains("send_startup_keepalive()"),
        "session.sh must define a startup keepalive helper. Fresh app \
         launches can exit after the no-stdin watchdog if the caller does \
         not send a JSON command immediately."
    );

    let launch_marker = "--binary \"$BINARY\"";
    let keepalive_marker = "if send_startup_keepalive \"$input_fifo\" \"$app_pid\"; then";
    let wait_marker =
        "if wait_for_ready_log \"$log_path\" \"$app_pid\" \"$READY_TIMEOUT_MS\"; then";

    let launch_pos = SESSION_SH
        .find(launch_marker)
        .expect("start must launch the app");
    let keepalive_pos = SESSION_SH
        .find(keepalive_marker)
        .expect("start must send a startup keepalive");
    let wait_pos = SESSION_SH
        .find(wait_marker)
        .expect("start must wait for the readiness marker");

    assert!(
        launch_pos < keepalive_pos && keepalive_pos < wait_pos,
        "session.sh must send the startup keepalive after app launch but \
         before waiting for readiness. Sending it after readiness reopens \
         the two-second no-stdin race."
    );

    assert!(
        SESSION_SH.contains("startupKeepalive:${startup_keepalive}"),
        "start's JSON envelope should report whether the keepalive write \
         succeeded so failed startup races are visible in receipts."
    );
}

#[test]
fn session_start_reports_keep_actions_window_open_lifecycle() {
    for needle in [
        "SCRIPT_KIT_AGENTIC_KEEP_ACTIONS_WINDOW_OPEN",
        "keep_actions_window_open",
        "keepActionsWindowOpen",
        "session_env_mismatch",
    ] {
        assert!(
            SESSION_SH.contains(needle),
            "session lifecycle must expose keep-open state: {needle}"
        );
    }
}
