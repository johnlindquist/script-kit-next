//! Source-level contract pins for devtools-session front door (Oracle isolated-devtools-agent-bootstrap).

const DEVTOOLS_SESSION_SH: &str = include_str!("../scripts/agentic/devtools-session.sh");
const START_ISOLATED_SH: &str = include_str!("../scripts/agentic/start-isolated.sh");
const BUILD_ISOLATED_SH: &str = include_str!("../scripts/agentic/build-isolated-binary.sh");
const SKILL_MD: &str = include_str!("../.agents/skills/isolated-devtools-session/SKILL.md");

#[test]
fn devtools_session_exposes_required_subcommands() {
    for sub in ["classify", "verify-script", "start", "prove", "cleanup"] {
        assert!(
            DEVTOOLS_SESSION_SH.contains(&format!("  {sub})")),
            "devtools-session.sh must implement subcommand {sub}"
        );
    }
}

#[test]
fn devtools_session_emits_json_envelope_and_progress() {
    assert!(
        DEVTOOLS_SESSION_SH.contains("\"tool\":\"devtools-session\""),
        "final JSON must identify tool devtools-session"
    );
    assert!(
        DEVTOOLS_SESSION_SH.contains("event\":\"progress\""),
        "stderr progress NDJSON must be emitted"
    );
}

#[test]
fn devtools_session_maps_dev_sh_exit_code() {
    assert!(
        DEVTOOLS_SESSION_SH.contains("exit 11"),
        "dev_sh_running must map to exit 11"
    );
    assert!(
        DEVTOOLS_SESSION_SH.contains("dev_sh_running"),
        "error code dev_sh_running must exist"
    );
}

#[test]
fn start_isolated_uses_short_internal_ready_timeout() {
    assert!(
        START_ISOLATED_SH.contains("SCRIPT_KIT_SESSION_READY_TIMEOUT_MS"),
        "start-isolated must set internal ready timeout"
    );
    assert!(
        START_ISOLATED_SH.contains("5000"),
        "internal session start timeout must be 5000ms (visible wait is wait-session-ready)"
    );
    assert!(
        START_ISOLATED_SH.contains("wait-session-ready.sh"),
        "start-isolated must delegate visible readiness to wait-session-ready.sh"
    );
}

#[test]
fn build_isolated_binary_is_timeboxed() {
    assert!(
        BUILD_ISOLATED_SH.contains("TIMEOUT_SEC"),
        "build must accept a timeout"
    );
    assert!(
        BUILD_ISOLATED_SH.contains("build_timeout"),
        "build must surface build_timeout"
    );
    assert!(
        BUILD_ISOLATED_SH.contains("target-agent"),
        "build must use agent-cargo target-agent output"
    );
}

#[test]
fn skill_documents_devtools_session_front_door() {
    assert!(
        SKILL_MD.contains("devtools-session.sh"),
        "SKILL.md must point agents at scripts/agentic/devtools-session.sh"
    );
}
