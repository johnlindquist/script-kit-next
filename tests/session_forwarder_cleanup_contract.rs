//! Source-level contract tests for `scripts/agentic/session.sh` session
//! ownership. A healthy PID file is not enough when stale forwarders are
//! still reading the same input FIFO.
//!
//! Run 10's first agentic-testing pass found `session.sh status default`
//! reporting healthy while `send --await-parse`, `getState`, and
//! `surface-proof --kind main` all timed out. The live machine had many
//! old forwarder/cat processes reading `/tmp/sk-agentic-sessions/default/input`,
//! so writes to the current session FIFO could be consumed by an orphan
//! before the active app saw them.

const SESSION_SH: &str = include_str!("../scripts/agentic/session.sh");

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
fn session_wrapper_can_find_forwarders_by_pipe_and_input_fifo() {
    assert!(
        SESSION_SH.contains("session_forwarder_pids()"),
        "session.sh must define a helper that finds forwarder ownership by \
         both the primary pipe path and input FIFO path."
    );
    assert!(
        SESSION_SH.contains("pgrep -f \"$pipe_pattern\""),
        "session_forwarder_pids must use pgrep -f for the primary pipe path. \
         `ps -axo pid=,command=` is not safe here because bash -c command \
         strings contain embedded newlines on macOS, so path matches can \
         appear on continuation lines where $1 is not a PID."
    );
    assert!(
        SESSION_SH.contains("pgrep -f \"cat ${input_pattern}\""),
        "session_forwarder_pids must also match cat processes reading the \
         input FIFO; orphan cats can consume commands before the active \
         forwarder sees them."
    );
    assert!(
        SESSION_SH.contains("regex_escape()"),
        "session_forwarder_pids must escape session paths before feeding \
         them to pgrep -f, because session names and /tmp paths can contain \
         regex metacharacters."
    );
    assert!(
        SESSION_SH.contains("path_aliases()")
            && SESSION_SH.contains("${SESSION_DIR_RAW}${file_path#\"$SESSION_DIR\"}"),
        "session_forwarder_pids must search both canonical `/private/tmp` \
         paths and the raw `/tmp` path spelling. Existing forwarders may \
         have been launched before SESSION_DIR canonicalization."
    );
}

#[test]
fn cleanup_preserves_active_forwarder_process_tree() {
    assert!(
        SESSION_SH.contains("is_descendant_of()"),
        "session.sh must define descendant detection so resume cleanup can \
         keep the active forwarder's process-substitution child and cat reader."
    );
    assert!(
        SESSION_SH.contains("cleanup_orphan_session_forwarders()"),
        "session.sh must expose one cleanup helper for start/resume/stop."
    );
    assert!(
        SESSION_SH.contains("is_descendant_of \"$orphan_pid\" \"$keep_pid\""),
        "cleanup_orphan_session_forwarders must skip the kept forwarder and \
         its descendants. Killing the process-substitution child of the \
         active forwarder would make a resumed session look healthy while \
         dropping every command."
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
            && forwarder_block.contains("cat \"$input_fifo\"")
            && forwarder_block.contains("done > \"$pipe_path\""),
        "session.sh's background forwarder must own the primary pipe writer \
         while repeatedly reopening the input FIFO. A process-substitution \
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
fn start_and_stop_call_forwarder_cleanup_at_ownership_boundaries() {
    let resume_marker =
        "cleanup_orphan_session_forwarders \"$primary_pipe\" \"$input_fifo\" \"$old_fwd_pid\"";
    assert!(
        SESSION_SH.contains(resume_marker),
        "cmd_start must clean orphan readers before resuming a healthy \
         session, while preserving the recorded active forwarder. Expected \
         marker:\n\n{resume_marker}\n"
    );

    let stale_marker = "cleanup_orphan_session_forwarders \"$primary_pipe\" \"$input_fifo\"";
    assert!(
        SESSION_SH.contains(stale_marker),
        "cmd_start must clean old readers when replacing a stale session. \
         Otherwise orphan forwarders remain attached to the next input FIFO."
    );

    let fresh_marker = "cleanup_orphan_session_forwarders \"$pipe_path\" \"$input_fifo\"";
    assert!(
        SESSION_SH.contains(fresh_marker),
        "cmd_start must clean session-path orphans before launching a fresh \
         forwarder/app pair."
    );

    let stop_marker = "cleanup_orphan_session_forwarders \"${sdir}/pipe\" \"${sdir}/input\"";
    assert!(
        SESSION_SH.contains(stop_marker),
        "cmd_stop must clean all forwarder/cat processes still attached to \
         the named session path, not just the PID recorded in fwd_pid."
    );
}

#[test]
fn start_sends_startup_keepalive_before_waiting_for_ready() {
    assert!(
        SESSION_SH.contains("send_startup_keepalive()"),
        "session.sh must define a startup keepalive helper. Fresh app \
         launches can exit after the no-stdin watchdog if the caller does \
         not send a JSON command immediately."
    );

    let launch_marker = "nohup \"$BINARY\" < \"$pipe_path\" > \"$log_path\" 2>&1 &";
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
