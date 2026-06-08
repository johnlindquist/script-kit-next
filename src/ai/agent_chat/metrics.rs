//! Neutral Agent Chat runtime metric labels.
//!
//! Phase 2 does not change existing Agent Chat tracing behavior.

pub(crate) const AGENT_CHAT_RUNTIME_TRACE_TARGET: &str = "script_kit::agent_chat";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentChatRuntimeOperation {
    PrepareSession,
    StartTurn,
    CancelTurn,
}
