//! Source-level contract pins for devtools-session front door (Oracle isolated-devtools-agent-bootstrap).

const DEVTOOLS_SESSION_SH: &str = include_str!("../scripts/agentic/devtools-session.sh");
const DEVTOOLS_SESSION_LIB_SH: &str = include_str!("../scripts/agentic/devtools-session-lib.sh");
const START_ISOLATED_SH: &str = include_str!("../scripts/agentic/start-isolated.sh");
const BUILD_ISOLATED_SH: &str = include_str!("../scripts/agentic/build-isolated-binary.sh");
const AGENT_CARGO_SH: &str = include_str!("../scripts/agentic/agent-cargo.sh");
const SESSION_SH: &str = include_str!("../scripts/agentic/session.sh");
const PREFLIGHT_ISOLATED_SH: &str = include_str!("../scripts/agentic/preflight-isolated.sh");
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
        DEVTOOLS_SESSION_LIB_SH.contains("\"event\":\"progress\""),
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
        BUILD_ISOLATED_SH.contains("target-agent/pools"),
        "build must use bounded agent target pools"
    );
}

#[test]
fn agent_cargo_defaults_to_named_pool_with_visible_lock() {
    assert!(
        AGENT_CARGO_SH.contains("SCRIPT_KIT_CARGO_TARGET_POOL:-agent-debug"),
        "agent-cargo must default to the shared agent-debug pool"
    );
    assert!(
        AGENT_CARGO_SH.contains("target-agent/pools/${pool}"),
        "agent-cargo pool mode must write to target-agent/pools/<pool>"
    );
    assert!(
        AGENT_CARGO_SH.contains("SCRIPT_KIT_AGENT_TARGET_MODE:-pool"),
        "agent-cargo must require explicit opt-in for exclusive per-agent targets"
    );
    assert!(
        AGENT_CARGO_SH.contains("target-agent/.locks"),
        "agent-cargo must expose a visible pool lock"
    );
    assert!(
        AGENT_CARGO_SH.contains("SCRIPT_KIT_AGENT_LOCK_TIMEOUT_SEC:-600"),
        "agent-cargo pool lock waits must be bounded"
    );
}

#[test]
fn isolated_binary_is_staged_not_promoted_to_dev_target() {
    assert!(
        BUILD_ISOLATED_SH.contains("target-agent/runtime/${SESSION_NAME}"),
        "isolated builds must stage a per-session runtime binary"
    );
    assert!(
        BUILD_ISOLATED_SH.contains("\"binaryPath\""),
        "build JSON must report the staged binary path"
    );
    assert!(
        BUILD_ISOLATED_SH.contains("tee -a \"$LOG\" >&2"),
        "build stdout must remain a machine-readable JSON envelope"
    );
    assert!(
        BUILD_ISOLATED_SH.contains("manifest.json"),
        "staged binaries must carry a manifest"
    );
    assert!(
        !BUILD_ISOLATED_SH.contains("promotedTo"),
        "isolated builds must not advertise target/debug promotion"
    );
    assert!(
        !BUILD_ISOLATED_SH.contains("cp -f \"$SRC\" \"$DEVTOOLS_SESSION_BINARY\""),
        "isolated builds must not copy into target/debug through DEVTOOLS_SESSION_BINARY"
    );
}

#[test]
fn sessions_honor_dynamic_binary_paths() {
    assert!(
        SESSION_SH.contains(
            "BINARY=\"${SCRIPT_KIT_GPUI_BINARY:-${PROJECT_ROOT}/target/debug/script-kit-gpui}\""
        ),
        "session.sh must allow isolated sessions to launch a staged binary"
    );
    assert!(
        SESSION_SH.contains("binary:\\\"${BINARY}\\\""),
        "session.sh start/status JSON must expose the binary path"
    );
    assert!(
        DEVTOOLS_SESSION_LIB_SH.contains("DEVTOOLS_SESSION_BINARY=\"${SCRIPT_KIT_GPUI_BINARY:-${DEVTOOLS_SESSION_REPO_ROOT}/target/debug/script-kit-gpui}\""),
        "devtools-session helpers must inherit SCRIPT_KIT_GPUI_BINARY"
    );
}

#[test]
fn devtools_session_uses_global_session_root_and_staged_binary() {
    assert!(
        DEVTOOLS_SESSION_SH.contains("SCRIPT_KIT_SESSION_DIR:-/tmp/sk-agentic-sessions"),
        "devtools-session must default to the same global session root as session.sh"
    );
    assert!(
        !DEVTOOLS_SESSION_SH.contains("/tmp/sk-agentic-sessions-${SESSION}"),
        "devtools-session must not invent per-session roots that break cleanup/prove"
    );
    assert!(
        START_ISOLATED_SH.contains("SCRIPT_KIT_SESSION_DIR:-/tmp/sk-agentic-sessions"),
        "start-isolated must default to the global session root"
    );
    assert!(
        PREFLIGHT_ISOLATED_SH.contains("--skip-binary"),
        "preflight must support a pre-build phase before the staged binary exists"
    );
    assert!(
        DEVTOOLS_SESSION_SH.contains("SCRIPT_KIT_GPUI_BINARY=\"$binary_path\""),
        "devtools-session must launch the staged binary after build"
    );
}

#[test]
fn skill_documents_devtools_session_front_door() {
    assert!(
        SKILL_MD.contains("devtools-session.sh"),
        "SKILL.md must point agents at scripts/agentic/devtools-session.sh"
    );
}
