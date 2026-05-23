use std::fs;

const MAIN: &str = include_str!("../src/main.rs");
const CONFIG: &str = include_str!("../src/ai/acp/config.rs");
const ADAPTER: &str = include_str!("../src/ai/acp/agy_adapter.rs");
const BUNDLE_VERIFY: &str = include_str!("../scripts/verify-macos-bundle.sh");

#[test]
fn agy_adapter_runs_inside_primary_binary_for_bundle_verifier() {
    assert!(
        MAIN.contains("--agy-acp-adapter") && MAIN.contains("ai::acp::agy_adapter::run_stdio()"),
        "script-kit-gpui must expose a hidden AGY ACP adapter mode"
    );
    assert!(
        BUNDLE_VERIFY.contains("! -name 'script-kit-gpui'"),
        "bundle verifier still allows only the primary binary"
    );
    assert!(
        !CONFIG.contains("command: \"agy-acp-adapter\"")
            && !CONFIG.contains("command = \"agy-acp-adapter\""),
        "config should not depend on a second bundled adapter executable"
    );
}

#[test]
fn agy_config_seeds_and_normalizes_subscription_cli_agent() {
    assert!(CONFIG.contains("pub(crate) const AGY_ACP_AGENT_ID: &str = \"agy-acp\""));
    assert!(CONFIG.contains("fn agy_acp_agent_config() -> AcpAgentConfig"));
    assert!(CONFIG.contains("AGY_ACP_ADAPTER_ARG"));
    assert!(CONFIG.contains("command_exists(\"agy\")"));
    assert!(CONFIG.contains("Google sign-in and subscription"));
    assert!(
        CONFIG.contains("agy-acp-adapter.js"),
        "legacy Node prototype catalog entries must migrate to the bundled adapter mode"
    );
}

#[test]
fn agy_adapter_preserves_conversation_and_filters_cli_noise() {
    assert!(ADAPTER.contains("find_conversation_created_after"));
    assert!(ADAPTER.contains(".gemini/antigravity-cli/conversations"));
    assert!(ADAPTER.contains("--conversation"));
    assert!(ADAPTER.contains("--print-timeout"));
    assert!(ADAPTER.contains("AGY_SKIP_PERMISSIONS"));
    assert!(ADAPTER.contains("TranscriptFilter"));
    assert!(ADAPTER.contains("SessionNotification"));
    assert!(ADAPTER.contains("AgentMessageChunk"));
    assert!(ADAPTER.contains("StopReason::Cancelled"));
}

#[test]
fn agy_adapter_source_file_exists() {
    assert!(fs::metadata("src/ai/acp/agy_adapter.rs")
        .expect("agy adapter source should exist")
        .is_file());
}
