use std::path::PathBuf;
use std::sync::Arc;

use agent_client_protocol::{ContentBlock, TextContent};

use super::executor::InlineAgentExecutor;
use super::types::{
    InlineAgentProviderEvent, InlineAgentProviderRequest, InlineAgentSessionId, InlineAgentTurnId,
};

pub(crate) struct AcpInlineAgentExecutor {
    connection: Arc<crate::ai::acp::AcpConnection>,
    cwd: PathBuf,
    model_id: Option<String>,
}

impl AcpInlineAgentExecutor {
    pub(crate) fn new(
        connection: Arc<crate::ai::acp::AcpConnection>,
        cwd: PathBuf,
        model_id: Option<String>,
    ) -> Self {
        Self {
            connection,
            cwd,
            model_id,
        }
    }
}

pub(crate) fn spawn_default_acp_inline_agent_executor() -> Result<AcpInlineAgentExecutor, String> {
    let catalog = crate::ai::acp::load_acp_agent_catalog_entries()
        .map_err(|error| format!("Failed to load ACP catalog: {error}"))?;
    let preferred_agent_id = crate::ai::acp::load_preferred_acp_agent_id();
    let requirements = crate::ai::acp::AcpLaunchRequirements::default();
    let launch_resolution = crate::ai::acp::resolve_acp_launch_with_requirements(
        &catalog,
        preferred_agent_id.as_deref(),
        requirements,
    );

    if !launch_resolution.is_ready() {
        return Err(crate::ai::acp::setup_title_for_resolution(&launch_resolution).to_string());
    }

    let agent = launch_resolution
        .selected_agent
        .as_ref()
        .and_then(|entry| entry.config.clone())
        .ok_or_else(|| "Resolved agent is missing configuration".to_string())?;
    let agent_models = agent.models.clone();
    let persisted_model = crate::config::load_user_preferences().ai.selected_model_id;
    let model_id = persisted_model
        .filter(|id| agent_models.iter().any(|model| model.id == *id))
        .or_else(|| agent_models.first().map(|model| model.id.clone()));

    let (broker, _permission_rx) = crate::ai::acp::AcpPermissionBroker::new();
    let connection =
        crate::ai::acp::AcpConnection::spawn_with_approval(agent, Some(broker.approval_fn()))
            .map_err(|error| format!("Failed to start ACP connection: {error}"))?;

    Ok(AcpInlineAgentExecutor::new(
        Arc::new(connection),
        crate::setup::get_kit_path(),
        model_id,
    ))
}

impl InlineAgentExecutor for AcpInlineAgentExecutor {
    fn start_turn(
        &self,
        request: InlineAgentProviderRequest,
    ) -> anyhow::Result<async_channel::Receiver<InlineAgentProviderEvent>> {
        let acp_events = self
            .connection
            .start_turn(crate::ai::acp::AcpPromptTurnRequest {
                ui_thread_id: request.session_id.0,
                cwd: self.cwd.clone(),
                blocks: vec![ContentBlock::Text(TextContent::new(request.prompt))],
                model_id: self.model_id.clone(),
            })?;

        let (provider_tx, provider_rx) = async_channel::bounded(256);
        std::thread::spawn(move || {
            while let Ok(event) = acp_events.recv_blocking() {
                let Some(provider_event) = map_acp_event(event) else {
                    continue;
                };
                let terminal = matches!(
                    provider_event,
                    InlineAgentProviderEvent::TurnFinished
                        | InlineAgentProviderEvent::Failed { .. }
                );
                if provider_tx.send_blocking(provider_event).is_err() {
                    break;
                }
                if terminal {
                    break;
                }
            }
        });

        Ok(provider_rx)
    }

    fn cancel_turn(
        &self,
        session_id: InlineAgentSessionId,
        _turn_id: InlineAgentTurnId,
    ) -> anyhow::Result<()> {
        self.connection.cancel_turn(session_id.0)
    }
}

fn map_acp_event(event: crate::ai::acp::AcpEvent) -> Option<InlineAgentProviderEvent> {
    match event {
        crate::ai::acp::AcpEvent::AgentMessageDelta(text) => {
            Some(InlineAgentProviderEvent::AgentMessageDelta { text })
        }
        crate::ai::acp::AcpEvent::AgentThoughtDelta(text) => {
            Some(InlineAgentProviderEvent::AgentThoughtDelta { text })
        }
        crate::ai::acp::AcpEvent::UsageUpdated { .. } => {
            Some(InlineAgentProviderEvent::UsageUpdated)
        }
        crate::ai::acp::AcpEvent::TurnFinished { .. } => {
            Some(InlineAgentProviderEvent::TurnFinished)
        }
        crate::ai::acp::AcpEvent::Failed { error } => {
            Some(InlineAgentProviderEvent::Failed { message: error })
        }
        crate::ai::acp::AcpEvent::SetupRequired { reason, .. } => {
            Some(InlineAgentProviderEvent::Failed { message: reason })
        }
        crate::ai::acp::AcpEvent::UserMessageDelta(_)
        | crate::ai::acp::AcpEvent::ToolCallStarted { .. }
        | crate::ai::acp::AcpEvent::ToolCallUpdated { .. }
        | crate::ai::acp::AcpEvent::PlanUpdated { .. }
        | crate::ai::acp::AcpEvent::AvailableCommandsUpdated { .. }
        | crate::ai::acp::AcpEvent::ModeChanged { .. }
        | crate::ai::acp::AcpEvent::ModelsAvailable { .. } => None,
    }
}
