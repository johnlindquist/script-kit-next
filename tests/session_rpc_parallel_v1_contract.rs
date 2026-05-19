//! Source-level contract pins for Parallel RPC v1 (Oracle devtools-parallel-rpc).
//!
//! These tests guard the session transport invariants without requiring a live
//! app process. Runtime stress belongs in `scripts/agentic/verify-parallel-rpc.sh`.

const SESSION_SH: &str = include_str!("../scripts/agentic/session.sh");
const AWAIT_RESPONSE_TS: &str = include_str!("../scripts/agentic/await-response.ts");
const STDIN_COMMANDS_MOD: &str = include_str!("../src/stdin_commands/mod.rs");
const ACTIONS_TS: &str = include_str!("../scripts/devtools/actions.ts");

fn cmd_rpc_body() -> &'static str {
    let start = SESSION_SH.find("cmd_rpc() {").expect("cmd_rpc must exist");
    let after = &SESSION_SH[start..];
    let end = after
        .find("\n}\n")
        .expect("cmd_rpc must terminate with `\\n}\\n`");
    &after[..end + 3]
}

fn cmd_start_body() -> &'static str {
    let start = SESSION_SH
        .find("cmd_start() {")
        .expect("cmd_start must exist");
    let after = &SESSION_SH[start..];
    let end = after[1..]
        .find("\n# --- send")
        .map(|idx| idx + 1)
        .expect("cmd_start must end before cmd_send");
    &after[..end]
}

#[test]
fn cmd_start_launches_app_with_protocol_bus_env() {
    let body = cmd_start_body();
    assert!(
        body.contains("SCRIPT_KIT_AGENTIC_PROTOCOL_RESPONSES_PATH"),
        "cmd_start must pass SCRIPT_KIT_AGENTIC_PROTOCOL_RESPONSES_PATH to the app"
    );
    assert!(
        body.contains("protocol-responses.ndjson"),
        "cmd_start must create protocol-responses.ndjson"
    );
    assert!(
        body.contains("SCRIPT_KIT_AGENTIC_SESSION_GENERATION"),
        "cmd_start must pass SCRIPT_KIT_AGENTIC_SESSION_GENERATION to the app"
    );
}

#[test]
fn cmd_rpc_serializes_with_session_command_lock() {
    let body = cmd_rpc_body();
    assert!(
        body.contains("acquire_session_lock"),
        "cmd_rpc must acquire the per-session command.lock before send/await"
    );
    assert!(
        body.contains("release_session_lock"),
        "cmd_rpc must release command.lock on all exit paths"
    );
    assert!(
        body.contains("queue_timeout"),
        "cmd_rpc must surface queue_timeout when the lock cannot be acquired"
    );
}

#[test]
fn cmd_rpc_awaits_on_protocol_responses_bus() {
    let body = cmd_rpc_body();
    assert!(
        body.contains("--responses-path"),
        "cmd_rpc must pass --responses-path to await-response.ts"
    );
    assert!(
        body.contains("protocol-responses.ndjson"),
        "cmd_rpc must use protocol-responses.ndjson as the primary response bus"
    );
}

#[test]
fn stdin_stdout_sender_appends_to_agentic_protocol_bus() {
    assert!(
        STDIN_COMMANDS_MOD.contains("agentic_protocol_bus::append_from_json_line"),
        "protocol stdout sender must append each response to the agentic protocol bus"
    );
}

#[test]
fn await_response_prefers_protocol_bus_and_response_timeout() {
    assert!(
        AWAIT_RESPONSE_TS.contains("scanProtocolBus"),
        "await-response.ts must scan protocol-responses.ndjson first"
    );
    assert!(
        AWAIT_RESPONSE_TS.contains("response_timeout"),
        "await-response.ts must use response_timeout (not generic timeout) for missed responses"
    );
    assert!(
        AWAIT_RESPONSE_TS.contains("SCRIPT_KIT_ALLOW_LOG_RPC_FALLBACK"),
        "await-response.ts must gate app.log fallback behind SCRIPT_KIT_ALLOW_LOG_RPC_FALLBACK"
    );
}

#[test]
fn devtools_actions_inspect_serializes_session_rpcs() {
    assert!(
        !ACTIONS_TS.contains("await Promise.all([\n    rpc("),
        "devtools actions inspect must not run getState RPC in parallel with other subprocess primitives"
    );
}
