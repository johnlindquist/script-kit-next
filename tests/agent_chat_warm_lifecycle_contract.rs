const AGENT_CHAT_MOD_SOURCE: &str = include_str!("../src/ai/agent_chat/mod.rs");
const WARM_SESSION_SOURCE: &str = include_str!("../src/ai/agent_chat/warm_session.rs");
const AGENT_CHAT_LAUNCH_SOURCE: &str = include_str!("../src/ai/agent_chat/launch.rs");
const AGENT_CHAT_VIEW_SOURCE: &str = include_str!("../src/ai/agent_chat/ui/view.rs");
const AGENT_CHAT_THREAD_SOURCE: &str = include_str!("../src/ai/agent_chat/ui/thread.rs");
const TAB_AI_MODE_SOURCE: &str = include_str!("../src/app_impl/agent_handoff/mod.rs");
const TAB_AI_AGENT_CHAT_LAUNCH_SOURCE: &str =
    include_str!("../src/app_impl/agent_handoff/agent_chat_launch.rs");
const AGENT_CHAT_SETUP_SOURCE: &str =
    include_str!("../src/app_impl/agent_handoff/agent_chat_setup.rs");

#[test]
fn warm_session_manager_module_is_declared_under_agent_chat() {
    assert!(AGENT_CHAT_MOD_SOURCE.contains("pub mod warm_session;"));
}

#[test]
fn warm_session_lifecycle_symbols_are_explicit() {
    for symbol in [
        "AgentChatWarmSessionManager",
        "prepare_warm",
        "acquire_warm",
        "dismiss_reset",
        "AgentChatWarmSessionLease",
        "AgentChatWarmSessionSpec",
        "AgentChatWarmRuntimeFactory",
    ] {
        assert!(WARM_SESSION_SOURCE.contains(symbol), "missing {}", symbol);
    }
}

#[test]
fn warm_session_manager_is_backend_neutral() {
    assert!(WARM_SESSION_SOURCE.contains("Arc<dyn AgentChatConnection>"));
    for forbidden in [
        "PiRpcRuntime",
        "PiLaunchSpec",
        "AgentChatView",
        "AppView",
        "ScriptListApp",
        "gpui::",
    ] {
        assert!(
            !WARM_SESSION_SOURCE.contains(forbidden),
            "warm session manager must not depend on {}",
            forbidden
        );
    }
}

#[test]
fn warm_session_lifecycle_routes_pi_only_through_launch_helper() {
    assert!(AGENT_CHAT_LAUNCH_SOURCE.contains("PiAgentChatLaunch"));
    assert!(AGENT_CHAT_LAUNCH_SOURCE.contains("warm_session_manager"));
    assert!(AGENT_CHAT_LAUNCH_SOURCE.contains("PiRpcRuntime::spawn"));
    assert!(AGENT_CHAT_LAUNCH_SOURCE.contains("resolve_selected_pi_launch_with_cwd_override"));
    assert!(AGENT_CHAT_LAUNCH_SOURCE.contains("resolve_selected_pi_launch_with_cwd_override"));
    assert!(TAB_AI_AGENT_CHAT_LAUNCH_SOURCE.contains("manager.acquire_ready_or_spawn_cold"));
    assert!(TAB_AI_MODE_SOURCE.contains("dismiss_active_agent_chat_warm_lease"));
    assert!(
        !AGENT_CHAT_SETUP_SOURCE.contains("PiRpcRuntime")
            && !AGENT_CHAT_SETUP_SOURCE.contains("AgentChatBackend::Pi"),
        "setup card routing must stay out of the Pi warm path"
    );
}

#[test]
fn startup_open_and_cwd_prewarm_share_selected_pi_cwd_launch_resolution() {
    assert!(AGENT_CHAT_LAUNCH_SOURCE
        .contains("pub(crate) fn resolve_selected_pi_launch_with_cwd_override"));
    assert!(
        AGENT_CHAT_LAUNCH_SOURCE
            .contains("resolve_selected_pi_launch_with_cwd_override(ai, ctx, None)"),
        "default selected launch must delegate to the cwd-aware helper"
    );

    let startup_body = TAB_AI_MODE_SOURCE
        .split("pub(crate) fn warm_agent_chat_on_startup")
        .nth(1)
        .expect("warm_agent_chat_on_startup must exist");
    let open_body = TAB_AI_AGENT_CHAT_LAUNCH_SOURCE
        .split("fn open_tab_ai_agent_chat_view_from_request_impl")
        .nth(1)
        .expect("open_tab_ai_agent_chat_view_from_request_impl must exist");
    let cwd_prewarm_body = TAB_AI_AGENT_CHAT_LAUNCH_SOURCE
        .split("pub(crate) fn prewarm_selected_agent_chat_profile_for_current_cwd")
        .nth(1)
        .expect("prewarm selected profile helper must exist");

    for (name, body) in [
        ("startup", startup_body),
        ("open", open_body),
        ("cwd/profile prewarm", cwd_prewarm_body),
    ] {
        assert!(
            body.contains("resolve_selected_pi_launch_with_cwd_override"),
            "{name} must use shared selected-profile/cwd Pi launch resolution"
        );
    }

    assert!(
        !open_body.contains("PiAgentChatLaunch::from_profile_with_cwd_override"),
        "open path must not hand-roll selected profile cwd launch resolution"
    );
}

#[test]
fn warm_session_lifecycle_keeps_agent_chat_ui_out_of_manager() {
    for source in [AGENT_CHAT_VIEW_SOURCE, AGENT_CHAT_THREAD_SOURCE] {
        assert!(!source.contains("AgentChatWarmSessionManager"));
        assert!(!source.contains("warm_session"));
    }
}

#[test]
fn dismiss_replacement_semantics_are_source_guarded() {
    for symbol in [
        "cancel_turn",
        "generation",
        "ui_thread_id_source",
        "prepare_session",
    ] {
        assert!(WARM_SESSION_SOURCE.contains(symbol), "missing {}", symbol);
    }
}

#[test]
fn warm_ready_requires_bounded_prepare_readiness_event() {
    for symbol in [
        "DEFAULT_PREPARE_READY_TIMEOUT",
        "wait_for_prepare_ready",
        "AgentChatEvent::ModelsAvailable",
        "AgentChatEvent::SetupRequired",
        "AgentChatEvent::Failed",
        "agent_chat_warm_prepare_ready_timeout",
    ] {
        assert!(WARM_SESSION_SOURCE.contains(symbol), "missing {}", symbol);
    }

    let prepare_slot = WARM_SESSION_SOURCE
        .split("fn prepare_new_slot")
        .nth(1)
        .expect("prepare_new_slot must exist");
    assert!(prepare_slot.contains("connection.prepare_session"));
    assert!(prepare_slot.contains("wait_for_prepare_ready"));
    assert!(
        !prepare_slot.contains(".prepare_session(ui_thread_id.clone(), spec.cwd.clone())\n                    .is_ok()"),
        "warm sessions must not become Ready merely because prepare_session enqueued"
    );
}

#[test]
fn warm_prepare_reserves_preparing_slot_before_spawn() {
    let prepare_warm = WARM_SESSION_SOURCE
        .split("pub(crate) fn prepare_warm")
        .nth(1)
        .expect("prepare_warm must exist");
    let prepare_warm = prepare_warm
        .split("pub(crate) fn acquire_warm")
        .next()
        .expect("prepare_warm body must precede acquire_warm");

    assert!(prepare_warm.contains("inner.slots.insert"));
    assert!(prepare_warm.contains("state: AgentChatWarmSessionState::Preparing"));
    assert!(prepare_warm.contains("prepare_slot_with_generation"));
    assert!(prepare_warm.contains("current.generation != generation"));
    assert!(
        prepare_warm.find("state: AgentChatWarmSessionState::Preparing")
            < prepare_warm.find("prepare_slot_with_generation"),
        "prepare_warm must reserve a Preparing slot before spawning/preparing the runtime"
    );
}
