//! Agent Client Protocol (ACP) integration.
//!
//! Provides a generic transport layer for communicating with ACP-compatible
//! AI coding agents (Claude Code, Gemini CLI, Codex, OpenCode, etc.).
//!
//! # Module Layout
//!
//! - `catalog` — Schema-versioned agent catalog file and readiness types.
//! - `config` — `AcpAgentConfig` for agent discovery and command configuration.
//! - `preflight` — Launch readiness resolution (blockers before thread spawn).
//! - `events` — Typed ACP turn/event primitives (`AcpEvent`, `AcpCommand`).
//! - `permission_broker` — Full-option-set permission forwarding to the UI.
//! - `types` — Bridging types between ACP and Script Kit internals.
//! - `handlers` — Client-side handler implementing the ACP `Client` trait.
//! - `client` — ACP runtime: subprocess lifecycle, initialize, session/prompt loop.

pub(crate) mod catalog;
pub(crate) mod chat_window;
pub(crate) mod client;
pub(crate) mod config;
pub(crate) mod context;
pub(crate) mod events;
pub(crate) mod export;
pub(crate) mod handlers;
pub(crate) mod history;
pub(crate) mod history_attachment;
pub(crate) mod history_popup;
pub(crate) mod model_selector_popup;
pub(crate) mod permission_broker;
pub(crate) mod picker_popup;
pub(crate) mod popup_window;
pub(crate) mod preflight;
pub(crate) mod provider;
pub(crate) mod setup_state;
pub(crate) mod thread;
pub(crate) mod types;
pub(crate) mod view;

#[cfg(test)]
mod tests;

pub(crate) use catalog::{
    default_acp_agents_path, AcpAgentAuthState, AcpAgentCatalogEntry, AcpAgentCatalogFile,
    AcpAgentConfigState, AcpAgentInstallState, AcpAgentSource,
};
pub(crate) use client::{AcpConnection, AcpRuntime};
pub(crate) use config::{
    claude_code_agent_config_cached, ensure_acp_agents_catalog_seeded,
    load_acp_agent_catalog_entries, load_acp_agent_configs, load_acp_agent_runtime_states,
    load_preferred_acp_agent_id, open_acp_agents_catalog_in_editor,
    persist_acp_agent_runtime_state, persist_preferred_acp_agent_id,
    persist_preferred_acp_agent_id_sync, prewarm_agent_config, AcpAgentConfig,
    AcpAgentRuntimeState, AcpAgentRuntimeStateFile,
};
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
pub(crate) use preflight::{
    resolve_acp_launch_with_requirements, resolve_default_acp_launch, setup_title_for_resolution,
    AcpLaunchBlocker, AcpLaunchRequirements, AcpLaunchResolution,
};
pub(crate) use provider::AcpProvider;
pub(crate) use setup_state::{AcpInlineSetupState, AcpSetupAction};
pub(crate) use thread::{
    AcpContextBootstrapState, AcpThread, AcpThreadInit, AcpThreadMessage, AcpThreadStatus,
    AcpToolCallState,
};
pub(crate) use view::{
    build_staged_skill_prompt, AcpChatSession, AcpChatView, AcpHistoryResumeRequest,
    AcpRetryRequest,
};
