//! Legacy ACP-named implementation for the Agent Chat UI.
//!
//! This module is the live implementation of the Agent Chat chat surface. The
//! `Acp*` names are retained as **compatibility contracts** (action IDs, route
//! IDs, `getAcpState`, serialized surface IDs, telemetry labels), not because
//! an Agent Client Protocol transport is still in use — the active backend is
//! Pi (`crate::ai::agent_chat`). New feature-level code should import the
//! canonical `AgentChat*` types from `crate::ai::agent_chat::ui` rather than
//! reaching into this module directly.
//!
//! # Module Layout
//!
//! - `catalog` — Schema-versioned agent catalog file and readiness types.
//! - `config` — `AcpAgentConfig` for agent discovery and command configuration.
//! - `preflight` — Launch readiness resolution (blockers before thread spawn).
//! - `events` — Typed Agent Chat turn/event primitives (`AcpEvent`).
//! - `permission_broker` — Full-option-set permission forwarding to the UI.
//! - `types` — Bridging types between Agent Chat and Script Kit internals.

use gpui::AppContext as _;

pub(crate) mod catalog;
pub(crate) mod chat_window;
pub(crate) mod components;
pub(crate) mod composer_state;
pub(crate) mod config;
pub(crate) mod context;
pub(crate) mod conversation_export;
pub(crate) mod events;
pub(crate) mod export;
pub(crate) mod history;
pub(crate) mod history_attachment;
pub(crate) mod history_popup;
pub(crate) mod hosted;
pub(crate) mod labels;
pub(crate) mod model_selector_popup;
pub(crate) mod permission_broker;
pub(crate) mod picker_popup;
pub(crate) mod popup_registry;
pub(crate) mod popup_window;
pub(crate) mod portal_contract;
pub(crate) mod preflight;
pub(crate) mod profile_selector_popup;
pub(crate) mod setup_state;
pub(crate) mod surface_state;
pub(crate) mod thread;
pub(crate) mod types;
pub(crate) mod ui_variant;
pub(crate) mod view;

#[cfg(test)]
mod tests;

pub(crate) use catalog::{
    default_acp_agents_path, AcpAgentAuthState, AcpAgentCatalogEntry, AcpAgentCatalogFile,
    AcpAgentConfigState, AcpAgentInstallState, AcpAgentSource,
};
pub(crate) use config::{
    claude_code_agent_config_cached, ensure_acp_agents_catalog_seeded,
    load_acp_agent_catalog_entries, load_acp_agent_configs, load_acp_agent_runtime_states,
    open_acp_agents_catalog_in_editor, persist_acp_agent_runtime_state, prewarm_agent_config,
    refresh_acp_agent_catalog_entries_with_snapshot, AcpAgentConfig, AcpAgentRuntimeState,
    AcpAgentRuntimeStateFile,
};
#[allow(deprecated)]
pub(crate) use context::{
    build_tab_ai_acp_context_blocks, build_tab_ai_acp_guidance_blocks,
    build_tab_ai_acp_guidance_blocks_for_prompt,
};
pub(crate) use events::{AcpEvent, AcpEventRx};
pub(crate) use permission_broker::{
    approval_request_input, AcpApprovalOption, AcpApprovalPreview, AcpApprovalPreviewKind,
    AcpApprovalRequest, AcpApprovalRequestInput, AcpPermissionBroker,
};
pub(crate) use preflight::{
    resolve_acp_launch_with_requirements, resolve_default_acp_launch,
    resolve_explicit_acp_launch_with_requirements, setup_title_for_resolution, AcpLaunchBlocker,
    AcpLaunchRequirements, AcpLaunchResolution,
};
pub(crate) use setup_state::{AcpInlineSetupState, AcpSetupAction};
pub(crate) use thread::{
    AcpContextBootstrapState, AcpThread, AcpThreadInit, AcpThreadMessage, AcpThreadStatus,
    AcpToolCallState,
};
pub(crate) use view::{
    build_skill_context_part, build_skill_slash_command_text, build_staged_skill_prompt,
    AcpChatSession, AcpChatView, AcpHistoryResumeRequest, AcpRetryRequest,
};

pub(crate) fn open_or_focus_chat_with_input(
    input: String,
    cx: &mut gpui::App,
) -> Result<(), String> {
    if let Some(entity) = chat_window::get_detached_acp_view_entity() {
        entity
            .update(cx, |chat, cx| {
                if chat.is_setup_mode() {
                    return Err("Agent Chat is in setup mode".to_string());
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

    let profile_ctx = crate::ai::agent_chat::profiles::AgentChatProfileContext::from_setup();
    let ai_preferences = crate::config::load_user_preferences().ai;
    let pi_launch =
        crate::ai::agent_chat::launch::resolve_selected_pi_launch(&ai_preferences, &profile_ctx)
            .map_err(|error| error.to_string())?;
    let warm_spec = pi_launch.warm_spec();
    let manager = crate::ai::agent_chat::launch::warm_session_manager();
    manager
        .prepare_warm(warm_spec)
        .map_err(|error| format!("Failed to prepare Pi Agent Chat warm session: {error}"))?;
    let lease = manager
        .acquire_warm(&pi_launch.warm_key)
        .ok_or_else(|| "Failed to start Pi Agent Chat warm session".to_string())?;

    let (_broker, permission_rx) = AcpPermissionBroker::new();
    let thread = cx.new(|cx| {
        AcpThread::new(
            lease.connection.clone(),
            permission_rx,
            AcpThreadInit {
                ui_thread_id: lease.ui_thread_id.clone(),
                cwd: lease.cwd.clone(),
                initial_input: Some(input),
                initial_context_parts: Vec::new(),
                display_name: pi_launch.profile.name.clone().into(),
                profile_display_name: Some(pi_launch.profile.name.clone().into()),
                profile_icon_name: pi_launch.profile.icon_name.clone(),
                selected_agent: None,
                available_agents: Vec::new(),
                launch_requirements: AcpLaunchRequirements::default(),
                available_models: pi_launch.available_models.clone(),
                selected_model_id: pi_launch.selected_model_id.clone(),
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
