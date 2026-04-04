//! Agent Client Protocol (ACP) integration.
//!
//! Provides a generic transport layer for communicating with ACP-compatible
//! AI coding agents (Claude Code, Gemini CLI, Codex, OpenCode, etc.).
//!
//! # Module Layout
//!
//! - `config` — `AcpAgentConfig` for agent discovery and command configuration.
//! - `events` — Typed ACP turn/event primitives (`AcpEvent`, `AcpCommand`).
//! - `permission_broker` — Full-option-set permission forwarding to the UI.
//! - `types` — Bridging types between ACP and Script Kit internals.
//! - `handlers` — Client-side handler implementing the ACP `Client` trait.
//! - `client` — ACP runtime: subprocess lifecycle, initialize, session/prompt loop.

pub(crate) mod chat_window;
pub(crate) mod client;
pub(crate) mod config;
pub(crate) mod context;
pub(crate) mod events;
pub(crate) mod handlers;
pub(crate) mod history;
pub(crate) mod permission_broker;
pub(crate) mod provider;
pub(crate) mod thread;
pub(crate) mod types;
pub(crate) mod view;

#[cfg(test)]
mod tests;

pub(crate) use client::{AcpConnection, AcpRuntime};
pub(crate) use config::{claude_code_agent_config_cached, prewarm_agent_config, AcpAgentConfig};
#[allow(deprecated)]
pub(crate) use context::{
    build_tab_ai_acp_context_blocks, build_tab_ai_acp_guidance_blocks,
    build_tab_ai_acp_guidance_blocks_for_prompt,
};
pub(crate) use events::{AcpCommand, AcpEvent, AcpEventRx, AcpPromptTurnRequest};
pub(crate) use permission_broker::{
    approval_request_input, AcpApprovalOption, AcpApprovalPreview, AcpApprovalPreviewKind,
    AcpApprovalRequest, AcpApprovalRequestInput, AcpPermissionBroker,
};
pub(crate) use provider::AcpProvider;
pub(crate) use thread::{
    AcpContextBootstrapState, AcpThread, AcpThreadInit, AcpThreadMessage, AcpThreadStatus,
    AcpToolCallState,
};
pub(crate) use view::AcpChatView;
