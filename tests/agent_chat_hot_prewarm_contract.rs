use std::fs;

#[test]
fn codex_agent_chat_kill_switch_is_manual_not_dev_default() {
    let config = fs::read_to_string("src/ai/agent_chat/ui/config.rs")
        .expect("read Agent Chat config source");
    let dev_sh = fs::read_to_string("dev.sh").expect("read dev.sh");

    assert!(
        config.contains("SCRIPT_KIT_DISABLE_CODEX_AGENT_CHAT"),
        "Agent Chat config must keep a manual Codex Agent Chat kill switch"
    );
    assert!(
        !dev_sh.contains("SCRIPT_KIT_DISABLE_CODEX_AGENT_CHAT"),
        "dev.sh must not set the manual Codex Agent Chat kill switch by default"
    );
    assert!(
        config.contains("should_be_implicit_codex_default: false"),
        "disabled Codex Agent Chat must not remain the implicit default"
    );
    assert!(
        config.contains("agent.id == CODEX_AGENT_CHAT_AGENT_ID")
            && config.contains("event = \"agent_chat_codex_agent_skipped\""),
        "catalog codex-agent_chat entries must be skipped when disabled"
    );
    assert!(
        config.contains("!codex_agent_chat_disabled_by_env()")
            && config.contains("agents.push(codex_agent_chat_agent_config());"),
        "built-in Codex Agent Chat auto-detection must be gated by the disable flag"
    );
}
