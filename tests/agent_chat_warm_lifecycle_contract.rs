const AGENT_CHAT_MOD_SOURCE: &str = include_str!("../src/ai/agent_chat/mod.rs");
const WARM_SESSION_SOURCE: &str = include_str!("../src/ai/agent_chat/warm_session.rs");
const AGENT_CHAT_LAUNCH_SOURCE: &str = include_str!("../src/ai/agent_chat/launch.rs");
const ACP_VIEW_SOURCE: &str = include_str!("../src/ai/acp/view.rs");
const ACP_THREAD_SOURCE: &str = include_str!("../src/ai/acp/thread.rs");
const TAB_AI_MODE_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode/mod.rs");
const ACP_LAUNCH_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode/acp_launch.rs");
const ACP_SETUP_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode/acp_setup.rs");

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
        "AcpChatView",
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
    assert!(ACP_LAUNCH_SOURCE.contains("resolve_effective_profile"));
    assert!(ACP_LAUNCH_SOURCE.contains("PiAgentChatLaunch::from_profile"));
    assert!(ACP_LAUNCH_SOURCE.contains("manager.acquire_warm"));
    assert!(TAB_AI_MODE_SOURCE.contains("dismiss_active_agent_chat_warm_lease"));
    assert!(
        !ACP_SETUP_SOURCE.contains("PiRpcRuntime")
            && !ACP_SETUP_SOURCE.contains("AgentChatBackend::Pi"),
        "setup card routing must stay out of the Pi warm path"
    );
}

#[test]
fn warm_session_lifecycle_keeps_acp_ui_out_of_manager() {
    for source in [ACP_VIEW_SOURCE, ACP_THREAD_SOURCE] {
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
