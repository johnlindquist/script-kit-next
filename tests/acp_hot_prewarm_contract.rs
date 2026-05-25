use std::fs;

fn fn_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("{signature} must exist"));
    let rest = &source[start..];
    let brace = rest
        .find('{')
        .unwrap_or_else(|| panic!("{signature} must have a body"));
    let mut depth = 0usize;
    let mut end = None;
    for (idx, ch) in rest[brace..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(brace + idx + 1);
                    break;
                }
            }
            _ => {}
        }
    }
    &rest[..end.unwrap_or(rest.len())]
}

#[test]
fn script_list_app_has_separate_hidden_acp_prewarm_slot() {
    let app_state =
        fs::read_to_string("src/main_sections/app_state.rs").expect("read app state source");
    let startup = fs::read_to_string("src/app_impl/startup.rs").expect("read startup source");

    // doc-anchor-removed: [[removed-docs Chat#Model selection#Hot prewarm before first submit]]
    assert!(
        app_state
            .contains("prewarmed_acp_chat: Option<Entity<crate::ai::acp::view::AcpChatView>>,"),
        "ScriptListApp must keep a hidden ACP prewarm slot distinct from embedded_acp_chat"
    );
    assert!(
        startup.contains("prewarmed_acp_chat: None,"),
        "startup initialization must clear the hidden ACP prewarm slot"
    );
}

#[test]
fn startup_schedules_acp_connection_prewarm() {
    let startup = fs::read_to_string("src/app_impl/startup.rs").expect("read startup source");
    let dev_sh = fs::read_to_string("dev.sh").expect("read dev.sh");

    assert!(
        startup.contains("crate::ai::acp::prewarm_agent_config();"),
        "startup should still prewarm config before hot ACP connection work"
    );
    assert!(
        startup.contains("this.warm_acp_chat_on_startup(cx);"),
        "startup must schedule hidden ACP Chat prewarm"
    );

    let config_idx = startup
        .find("crate::ai::acp::prewarm_agent_config();")
        .expect("config prewarm call exists");
    let chat_idx = startup
        .find("this.warm_acp_chat_on_startup(cx);")
        .expect("ACP chat warm call exists");
    assert!(
        config_idx < chat_idx,
        "agent config prewarm should be kicked off before ACP chat prewarm"
    );
    assert!(
        dev_sh.contains("SCRIPT_KIT_DISABLE_ACP_HOT_PREWARM"),
        "./dev.sh must opt out of hidden ACP connection prewarm so dev launch cannot trigger codex-acp keychain prompts"
    );
    assert!(
        !dev_sh.contains("SCRIPT_KIT_DISABLE_CODEX_ACP"),
        "./dev.sh must not hide Codex from the visible Agent Chat catalog; disabling hidden ACP prewarm is the prompt-safe dev guard"
    );
}

#[test]
fn acp_hot_prewarm_helper_creates_hidden_hosted_view() {
    let tab_ai = fs::read_to_string("src/app_impl/tab_ai_mode/mod.rs").expect("read tab ai source");
    let body = fn_body(&tab_ai, "pub(crate) fn warm_acp_chat_on_startup(");

    assert!(
        body.contains("SCRIPT_KIT_DISABLE_ACP_HOT_PREWARM"),
        "warm_acp_chat_on_startup must honor the dev opt-out before spawning an ACP runtime"
    );
    assert!(
        body.contains("crate::ai::acp::hosted::spawn_hosted_view("),
        "warm_acp_chat_on_startup must use the host-neutral ACP bootstrap"
    );
    assert!(
        body.contains("crate::ai::acp::AcpLaunchRequirements::default()"),
        "startup prewarm should only create a default-requirements session"
    );
    assert!(
        body.contains("self.wire_embedded_acp_footer_callbacks(&view, cx);"),
        "prewarmed views must be fully wired before they are shown later"
    );
    assert!(
        body.contains("self.prewarmed_acp_chat = Some(view);"),
        "the hidden view must be retained for later launch consumption"
    );
}

#[test]
fn codex_acp_kill_switch_is_manual_not_dev_default() {
    let config = fs::read_to_string("src/ai/acp/config.rs").expect("read ACP config source");
    let dev_sh = fs::read_to_string("dev.sh").expect("read dev.sh");

    assert!(
        config.contains("SCRIPT_KIT_DISABLE_CODEX_ACP"),
        "ACP config must keep a manual Codex ACP kill switch"
    );
    assert!(
        !dev_sh.contains("SCRIPT_KIT_DISABLE_CODEX_ACP"),
        "dev.sh must not set the manual Codex ACP kill switch by default"
    );
    assert!(
        config.contains("should_be_implicit_codex_default: false"),
        "disabled Codex ACP must not remain the implicit default"
    );
    assert!(
        config.contains("agent.id == CODEX_ACP_AGENT_ID")
            && config.contains("event = \"acp_codex_agent_skipped\""),
        "catalog codex-acp entries must be skipped when disabled"
    );
    assert!(
        config.contains("!codex_acp_disabled_by_env()")
            && config.contains("agents.push(codex_acp_agent_config());"),
        "built-in Codex ACP auto-detection must be gated by the disable flag"
    );
}

#[test]
fn acp_open_uses_pi_warm_session_without_fresh_acp_runtime() {
    let acp_launch = fs::read_to_string("src/app_impl/tab_ai_mode/acp_launch.rs")
        .expect("read ACP launch source");
    let body = fn_body(&acp_launch, "fn open_tab_ai_acp_view_from_request_impl(");

    assert!(
        body.contains("open_tab_ai_pi_view_from_launch"),
        "open path must route through Pi warm Agent Chat launch"
    );
    assert!(
        !body.contains("spawn_with_approval"),
        "open path must not spawn a fresh ACP runtime"
    );
}

#[test]
fn prewarm_consume_requires_matching_default_launch() {
    let tab_ai = fs::read_to_string("src/app_impl/tab_ai_mode/mod.rs").expect("read tab ai source");
    let body = fn_body(&tab_ai, "fn take_prewarmed_acp_chat_for_launch(");

    assert!(
        body.contains("requirements != crate::ai::acp::AcpLaunchRequirements::default()"),
        "prewarmed default sessions must not be consumed by capability-specific launches"
    );
    assert!(
        body.contains("thread_selected_agent_id.as_deref() != selected_agent_id"),
        "a prewarmed session must not be reused for a different selected agent"
    );
    assert!(
        body.contains("self.prewarmed_acp_chat = None;"),
        "successful consumption must remove the hidden prewarm slot"
    );
}
