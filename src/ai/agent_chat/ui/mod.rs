//! Agent Chat UI implementation.
//!
//! This module is the live implementation of the Agent Chat chat surface. The
//! The active backend is goose (`crate::ai::agent_chat`). Feature-level code should
//! import Agent Chat UI types from this module.
//!
//! # Module Layout
//!
//! - `catalog` — Schema-versioned agent catalog file and readiness types.
//! - `config` — `AgentChatAgentConfig` for agent discovery and command configuration.
//! - `preflight` — Launch readiness resolution (blockers before thread spawn).
//! - `events` — Typed Agent Chat turn/event primitives (`AgentChatEvent`).
//! - `permission_broker` — Full-option-set permission forwarding to the UI.
//! - `types` — Bridging types between Agent Chat and Script Kit internals.

use gpui::AppContext as _;

pub(crate) mod capabilities;
pub(crate) mod catalog;
pub(crate) mod config;
pub(crate) mod context;
pub(crate) mod conversation_export;
pub(crate) mod entry;
pub(crate) mod events;
pub(crate) mod export;
pub(crate) mod favorite_models;
pub(crate) mod history;
pub(crate) mod history_attachment;
pub(crate) mod hosted;
pub(crate) mod labels;
pub(crate) mod mock_fixture;
pub(crate) mod notifications;
pub(crate) mod permission_broker;
pub(crate) mod popup_automation;
pub(crate) mod portal_contract;
pub(crate) mod preflight;
pub(crate) mod setup_state;
pub(crate) mod streaming_buffer;
pub(crate) mod thread;
pub(crate) mod tool_card;
pub(crate) mod types;
pub(crate) mod ui_variant;

#[cfg(test)]
mod tests;

pub(crate) use catalog::{
    default_agent_chat_agents_path, AgentChatAgentAuthState, AgentChatAgentCatalogEntry,
    AgentChatAgentCatalogFile, AgentChatAgentConfigState, AgentChatAgentInstallState,
    AgentChatAgentSource,
};
pub(crate) use config::{
    agent_chat_cwd_recents_for_profile, claude_code_agent_config_cached,
    ensure_agent_chat_agents_catalog_seeded, load_agent_chat_agent_catalog_entries,
    load_agent_chat_agent_configs, load_agent_chat_agent_runtime_states,
    open_agent_chat_agents_catalog_in_editor, persist_agent_chat_agent_runtime_state,
    prewarm_agent_config, record_agent_chat_cwd_recent,
    refresh_agent_chat_agent_catalog_entries_with_snapshot, AgentChatAgentConfig,
    AgentChatAgentRuntimeState, AgentChatAgentRuntimeStateFile,
};
#[allow(deprecated)]
pub(crate) use context::{
    build_tab_ai_agent_chat_context_blocks, build_tab_ai_agent_chat_guidance_blocks,
    build_tab_ai_agent_chat_guidance_blocks_for_prompt,
};
pub(crate) use events::{AgentChatEvent, AgentChatEventRx, AgentChatForkPoint};
pub(crate) use permission_broker::{
    approval_request_input, AgentChatApprovalOption, AgentChatApprovalPreview,
    AgentChatApprovalPreviewKind, AgentChatApprovalRequest, AgentChatApprovalRequestInput,
    AgentChatPermissionBroker,
};
pub(crate) use preflight::{
    resolve_agent_chat_launch_with_requirements, resolve_default_agent_chat_launch,
    resolve_explicit_agent_chat_launch_with_requirements, setup_title_for_resolution,
    AgentChatLaunchBlocker, AgentChatLaunchRequirements, AgentChatLaunchResolution,
};
pub(crate) use setup_state::{AgentChatInlineSetupState, AgentChatSetupAction};
pub(crate) use thread::{
    AgentChatContextBootstrapState, AgentChatThread, AgentChatThreadInit, AgentChatThreadMessage,
    AgentChatThreadStatus, AgentChatToolCallState,
};
