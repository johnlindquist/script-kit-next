//! Host-neutral Agent Chat bootstrap.
//!
//! Extracts the warm Pi connection/thread creation logic out of the
//! detached-only `open_or_focus_chat_with_input` so that any host surface
//! (launcher, Notes, detached window) can spawn a live Agent Chat view without
//! knowing the window ownership details.

use gpui::{App, AppContext as _, Entity};

use super::thread::{AgentChatThread, AgentChatThreadInit};
use super::view::AgentChatView;
use super::{AgentChatLaunchRequirements, AgentChatPermissionBroker};

/// Spawn a new `AgentChatThread` entity with the standard catalog/preflight/connection
/// bootstrap.  The returned entity is ready for embedding in any host surface.
pub(crate) fn spawn_hosted_thread(
    initial_input: Option<String>,
    requirements: AgentChatLaunchRequirements,
    cx: &mut App,
) -> Result<Entity<AgentChatThread>, String> {
    let profile_ctx = crate::ai::agent_chat::profiles::AgentChatProfileContext::from_setup();
    let ai_preferences = crate::config::load_user_preferences().ai;
    let pi_launch =
        crate::ai::agent_chat::launch::resolve_selected_pi_launch(&ai_preferences, &profile_ctx)
            .map_err(|error| error.to_string())?;
    let manager = crate::ai::agent_chat::launch::warm_session_manager();
    manager
        .prepare_warm(pi_launch.warm_spec())
        .map_err(|error| format!("Failed to prepare Pi Agent Chat warm session: {error}"))?;
    let lease = manager
        .acquire_warm(&pi_launch.warm_key)
        .ok_or_else(|| "Failed to start Pi Agent Chat warm session".to_string())?;

    let (_broker, permission_rx) = AgentChatPermissionBroker::new();

    let thread = cx.new(|cx| {
        AgentChatThread::new(
            lease.connection.clone(),
            permission_rx,
            AgentChatThreadInit {
                ui_thread_id: lease.ui_thread_id.clone(),
                cwd: lease.cwd.clone(),
                initial_input,
                initial_context_parts: Vec::new(),
                display_name: pi_launch.profile.name.clone().into(),
                profile_display_name: Some(pi_launch.profile.name.clone().into()),
                profile_icon_name: pi_launch.profile.icon_name.clone(),
                selected_agent: None,
                available_agents: Vec::new(),
                launch_requirements: requirements,
                available_models: pi_launch.available_models.clone(),
                selected_model_id: pi_launch.selected_model_id.clone(),
            },
            cx,
        )
    });

    thread.update(cx, |thread: &mut AgentChatThread, cx| {
        thread.mark_context_bootstrap_ready(cx);
    });

    Ok(thread)
}

/// Spawn a new `AgentChatView` entity backed by a fresh hosted thread.
///
/// The returned view has no host callbacks wired — the caller is responsible
/// for calling `set_on_toggle_actions`, `set_on_close_requested`, etc.
pub(crate) fn spawn_hosted_view(
    initial_input: Option<String>,
    requirements: AgentChatLaunchRequirements,
    cx: &mut App,
) -> Result<Entity<AgentChatView>, String> {
    let thread = spawn_hosted_thread(initial_input, requirements, cx)?;
    let view = cx.new(|cx| AgentChatView::new(thread, cx));
    Ok(view)
}
