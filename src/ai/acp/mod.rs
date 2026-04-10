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

use gpui::AppContext as _;

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
pub(crate) mod hosted;
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

pub(crate) fn open_or_focus_chat_with_input(
    input: String,
    cx: &mut gpui::App,
) -> Result<(), String> {
    if let Some(entity) = chat_window::get_detached_acp_view_entity() {
        entity
            .update(cx, |chat, cx| {
                if chat.is_setup_mode() {
                    return Err("ACP Chat is in setup mode".to_string());
                }
                chat.set_input(input, cx);
                Ok::<(), String>(())
            })
            .map_err(|error| error.to_string())?;
        chat_window::activate_chat_window(cx);
        return Ok(());
    }

    if chat_window::is_chat_window_open() {
        chat_window::close_chat_window(cx);
    }

    let catalog = load_acp_agent_catalog_entries()
        .map_err(|error| format!("Failed to load ACP catalog: {error}"))?;
    let preferred_agent_id = load_preferred_acp_agent_id();
    let requirements = AcpLaunchRequirements::default();
    let launch_resolution =
        resolve_acp_launch_with_requirements(&catalog, preferred_agent_id.as_deref(), requirements);

    if !launch_resolution.is_ready() {
        return Err(setup_title_for_resolution(&launch_resolution).to_string());
    }

    let agent = launch_resolution
        .selected_agent
        .as_ref()
        .and_then(|entry| entry.config.clone())
        .ok_or_else(|| "Resolved ACP agent is missing configuration".to_string())?;
    let agent_display_name = agent.display_name().to_string();
    let agent_models = agent.models.clone();
    let persisted_model = crate::config::load_user_preferences().ai.selected_model_id;
    let default_model_id = persisted_model
        .filter(|id| agent_models.iter().any(|model| model.id == *id))
        .or_else(|| agent_models.first().map(|model| model.id.clone()));

    let (broker, permission_rx) = AcpPermissionBroker::new();
    let connection = AcpConnection::spawn_with_approval(agent, Some(broker.approval_fn()))
        .map_err(|error| format!("Failed to start ACP connection: {error}"))?;
    let cwd = crate::setup::get_kit_path();

    let thread = cx.new(|cx| {
        AcpThread::new(
            std::sync::Arc::new(connection),
            permission_rx,
            AcpThreadInit {
                ui_thread_id: uuid::Uuid::new_v4().to_string(),
                cwd,
                initial_input: Some(input),
                display_name: agent_display_name.into(),
                selected_agent: launch_resolution.selected_agent.clone(),
                available_agents: launch_resolution.catalog_entries.clone(),
                launch_requirements: requirements,
                available_models: agent_models,
                selected_model_id: default_model_id,
            },
            cx,
        )
    });

    thread.update(cx, |thread, cx| {
        thread.mark_context_bootstrap_ready(cx);
    });

    chat_window::open_chat_window_with_thread(thread, None, cx)
        .map_err(|error| error.to_string())?;
    Ok(())
}
