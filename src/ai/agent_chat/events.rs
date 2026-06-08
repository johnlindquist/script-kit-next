//! Backend-neutral Agent Chat event aliases.
//!
//! Phase 2 intentionally keeps the UI event stream Agent Chat-shaped. Future
//! backends map their native events into these types at their adapter boundary,
//! not inside `AgentChatThread` or `AgentChatView`.

pub(crate) type AgentChatEvent = crate::ai::agent_chat::ui::AgentChatEvent;
pub(crate) type AgentChatEventRx = crate::ai::agent_chat::ui::AgentChatEventRx;
pub(crate) type AgentChatApprovalRequest = crate::ai::agent_chat::ui::AgentChatApprovalRequest;
pub(crate) type AgentChatApprovalRx = async_channel::Receiver<AgentChatApprovalRequest>;
pub(crate) type AgentChatModelEntry = crate::ai::agent_chat::ui::config::AgentChatModelEntry;
