const AGENT_CHAT_MOD_SOURCE: &str = include_str!("../src/ai/agent_chat/mod.rs");
const AGENT_CHAT_EVENTS_SOURCE: &str = include_str!("../src/ai/agent_chat/events.rs");
const AGENT_CHAT_RUNTIME_SOURCE: &str = include_str!("../src/ai/agent_chat/runtime.rs");
const ACP_CLIENT_SOURCE: &str = include_str!("../src/ai/acp/client.rs");
const ACP_THREAD_SOURCE: &str = include_str!("../src/ai/acp/thread.rs");
const ACP_VIEW_SOURCE: &str = include_str!("../src/ai/acp/view.rs");
const TAB_AI_MODE_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode/mod.rs");
const ACP_LAUNCH_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode/acp_launch.rs");

#[test]
fn agent_chat_runtime_modules_are_declared() {
    assert!(AGENT_CHAT_MOD_SOURCE.contains("pub mod events;"));
    assert!(AGENT_CHAT_MOD_SOURCE.contains("pub mod runtime;"));
    assert!(AGENT_CHAT_MOD_SOURCE.contains("pub mod metrics;"));
}

#[test]
fn agent_chat_connection_trait_is_backend_neutral_and_object_safe_by_shape() {
    assert!(AGENT_CHAT_RUNTIME_SOURCE.contains("pub(crate) trait AgentChatConnection"));
    assert!(AGENT_CHAT_RUNTIME_SOURCE.contains("Send + Sync + 'static"));
    assert!(AGENT_CHAT_RUNTIME_SOURCE.contains("fn start_turn(&self"));
    assert!(AGENT_CHAT_RUNTIME_SOURCE.contains("fn cancel_turn(&self"));
    assert!(AGENT_CHAT_RUNTIME_SOURCE.contains("fn prepare_session(&self"));
    assert!(
        !AGENT_CHAT_RUNTIME_SOURCE.contains("async fn"),
        "runtime seam must remain object-safe"
    );
    assert!(
        !AGENT_CHAT_RUNTIME_SOURCE.contains("PiLaunchSpec"),
        "Phase 2 seam must not know about Pi launch specs"
    );
}

#[test]
fn phase_two_event_boundary_aliases_current_acp_stream() {
    assert!(AGENT_CHAT_EVENTS_SOURCE.contains("type AgentChatEvent = crate::ai::acp::AcpEvent"));
    assert!(AGENT_CHAT_EVENTS_SOURCE.contains("type AgentChatEventRx = crate::ai::acp::AcpEventRx"));
    assert!(
        !AGENT_CHAT_EVENTS_SOURCE.contains("enum AgentChatEvent"),
        "Phase 2 must not fork a second event enum"
    );
}

#[test]
fn acp_runtime_implements_neutral_runtime_seam() {
    assert!(
        ACP_CLIENT_SOURCE.contains("impl AgentChatConnection for AcpRuntime")
            || ACP_CLIENT_SOURCE.contains(
                "impl crate::ai::agent_chat::runtime::AgentChatConnection for AcpRuntime",
            )
    );
    assert!(ACP_CLIENT_SOURCE.contains("AcpRuntime::start_turn(self"));
    assert!(ACP_CLIENT_SOURCE.contains("AcpRuntime::prepare_session(self"));
    assert!(ACP_CLIENT_SOURCE.contains("AcpRuntime::cancel_turn(self"));
}

#[test]
fn acp_thread_depends_on_trait_object_not_concrete_acp_runtime() {
    assert!(
        ACP_THREAD_SOURCE.contains("Arc<dyn AgentChatConnection>"),
        "AcpThread should store the neutral runtime seam"
    );
    assert!(
        !ACP_THREAD_SOURCE.contains("connection: Arc<AcpConnection>"),
        "AcpThread must not store the concrete ACP connection after Phase 2"
    );
}

#[test]
fn phase_two_keeps_view_out_of_runtime_refactor() {
    assert!(
        !ACP_VIEW_SOURCE.contains("AgentChatConnection"),
        "AcpChatView should keep owning UI state only; runtime seam belongs in AcpThread"
    );
    assert!(
        !ACP_VIEW_SOURCE.contains("PiLaunchSpec"),
        "Phase 2 must not route the view toward Pi"
    );
}

#[test]
fn phase_two_does_not_spawn_or_route_pi() {
    for (name, source) in [
        ("agent_chat_runtime", AGENT_CHAT_RUNTIME_SOURCE),
        ("acp_client", ACP_CLIENT_SOURCE),
        ("acp_thread", ACP_THREAD_SOURCE),
        ("tab_ai_mode", TAB_AI_MODE_SOURCE),
        ("acp_launch", ACP_LAUNCH_SOURCE),
    ] {
        assert!(
            !source.contains("PiLaunchSpec"),
            "{name} must not import or use PiLaunchSpec in Phase 2"
        );
        assert!(
            !source.contains("agent_chat::pi"),
            "{name} must not import Pi runtime modules in Phase 2"
        );
        assert!(
            !source.contains("Command::new(\"pi\")")
                && !source.contains("tokio::process::Command::new(\"pi\")")
                && !source.contains("std::process::Command::new(\"pi\")"),
            "{name} must not spawn Pi in Phase 2"
        );
    }
}

#[test]
fn phase_two_keeps_tab_routing_on_acp() {
    assert!(
        ACP_LAUNCH_SOURCE.contains("AcpConnection::spawn_with_approval")
            || ACP_LAUNCH_SOURCE.contains("AcpRuntime::spawn_with_approval"),
        "Tab Agent Chat launch should still instantiate the ACP runtime"
    );
    assert!(
        !TAB_AI_MODE_SOURCE.contains("AgentChatBackend::Pi"),
        "Tab routing must not branch to Pi in Phase 2"
    );
    assert!(
        !TAB_AI_MODE_SOURCE.contains("selected_backend")
            && !TAB_AI_MODE_SOURCE.contains("selectedBackend"),
        "selectedBackend remains schema-only until the opt-in routing phase"
    );
}
