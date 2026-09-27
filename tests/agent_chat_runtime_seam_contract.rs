const AGENT_CHAT_EVENTS_SOURCE: &str = include_str!("../src/ai/agent_chat/events.rs");
const AGENT_CHAT_RUNTIME_SOURCE: &str = include_str!("../src/ai/agent_chat/runtime.rs");
const AGENT_CHAT_THREAD_SOURCE: &str = include_str!("../src/ai/agent_chat/ui/thread.rs");

#[test]
fn agent_chat_connection_trait_is_backend_neutral_and_object_safe_by_shape() {
    assert!(AGENT_CHAT_RUNTIME_SOURCE.contains("pub(crate) trait AgentChatConnection"));
    assert!(AGENT_CHAT_RUNTIME_SOURCE.contains("Send + Sync + 'static"));
    assert!(AGENT_CHAT_RUNTIME_SOURCE.contains("fn start_turn(&self"));
    assert!(AGENT_CHAT_RUNTIME_SOURCE.contains("fn cancel_turn(&self"));
    assert!(AGENT_CHAT_RUNTIME_SOURCE.contains("fn prepare_session("));
    assert!(
        !AGENT_CHAT_RUNTIME_SOURCE.contains("async fn"),
        "runtime seam must remain object-safe"
    );
    assert!(
        !AGENT_CHAT_RUNTIME_SOURCE.contains("GooseLaunchSpec"),
        "Phase 2 seam must not know about goose launch specs"
    );
}

#[test]
fn phase_two_event_boundary_aliases_current_agent_chat_stream() {
    assert!(AGENT_CHAT_EVENTS_SOURCE
        .contains("type AgentChatEvent = crate::ai::agent_chat::ui::AgentChatEvent"));
    assert!(AGENT_CHAT_EVENTS_SOURCE
        .contains("type AgentChatEventRx = crate::ai::agent_chat::ui::AgentChatEventRx"));
    assert!(
        !AGENT_CHAT_EVENTS_SOURCE.contains("enum AgentChatEvent"),
        "Phase 2 must not fork a second event enum"
    );
}

#[test]
fn agent_chat_thread_depends_on_trait_object_not_concrete_agent_chat_runtime() {
    assert!(
        AGENT_CHAT_THREAD_SOURCE.contains("Arc<dyn AgentChatConnection>"),
        "AgentChatThread should store the neutral runtime seam"
    );
    assert!(
        AGENT_CHAT_THREAD_SOURCE.contains("connection: Arc<dyn AgentChatConnection>"),
        "AgentChatThread must store the neutral Agent Chat runtime seam"
    );
}
