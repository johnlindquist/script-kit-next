//! Backend-neutral Agent Chat event aliases.
//!
//! Phase 2 intentionally keeps the UI event stream ACP-shaped. Future
//! backends map their native events into these types at their adapter boundary,
//! not inside `AcpThread` or `AcpChatView`.

pub(crate) type AgentChatEvent = crate::ai::acp::AcpEvent;
pub(crate) type AgentChatEventRx = crate::ai::acp::AcpEventRx;
pub(crate) type AgentChatApprovalRequest = crate::ai::acp::AcpApprovalRequest;
pub(crate) type AgentChatApprovalRx = async_channel::Receiver<AgentChatApprovalRequest>;
pub(crate) type AgentChatModelEntry = crate::ai::acp::config::AcpModelEntry;
