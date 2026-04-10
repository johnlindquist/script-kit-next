//! Host-neutral ACP bootstrap.
//!
//! Extracts the catalog/preflight/connection/thread creation logic out of the
//! detached-only `open_or_focus_chat_with_input` so that any host surface
//! (launcher, Notes, detached window) can spawn a live ACP chat view without
//! knowing the window ownership details.

use gpui::{App, AppContext as _, Entity};

use super::thread::{AcpThread, AcpThreadInit};
use super::view::AcpChatView;
use super::{
    load_acp_agent_catalog_entries, load_preferred_acp_agent_id,
    resolve_acp_launch_with_requirements, setup_title_for_resolution, AcpConnection,
    AcpLaunchRequirements, AcpPermissionBroker,
};

/// Spawn a new `AcpThread` entity with the standard catalog/preflight/connection
/// bootstrap.  The returned entity is ready for embedding in any host surface.
pub(crate) fn spawn_hosted_thread(
    initial_input: Option<String>,
    requirements: AcpLaunchRequirements,
    cx: &mut App,
) -> Result<Entity<AcpThread>, String> {
    let catalog = load_acp_agent_catalog_entries()
        .map_err(|error| format!("Failed to load ACP catalog: {error}"))?;
    let preferred_agent_id = load_preferred_acp_agent_id();
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
                initial_input,
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

    thread.update(cx, |thread: &mut AcpThread, cx| {
        thread.mark_context_bootstrap_ready(cx);
    });

    Ok(thread)
}

/// Spawn a new `AcpChatView` entity backed by a fresh hosted thread.
///
/// The returned view has no host callbacks wired — the caller is responsible
/// for calling `set_on_toggle_actions`, `set_on_close_requested`, etc.
pub(crate) fn spawn_hosted_view(
    initial_input: Option<String>,
    requirements: AcpLaunchRequirements,
    cx: &mut App,
) -> Result<Entity<AcpChatView>, String> {
    let thread = spawn_hosted_thread(initial_input, requirements, cx)?;
    let view = cx.new(|cx| AcpChatView::new(thread, cx));
    Ok(view)
}
