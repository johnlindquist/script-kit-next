//! Inline setup state for Agent Chat chat views.
//!
//! When no launchable agent exists, the Agent Chat chat view renders an inline
//! setup card instead of a dead-end toast. This module defines the
//! structured state that drives that card.

use gpui::SharedString;

use super::catalog::AgentChatAgentCatalogEntry;
use super::preflight::{
    AgentChatLaunchBlocker, AgentChatLaunchRequirements, AgentChatLaunchResolution,
};
use crate::protocol::{AgentChatSetupActionKind, AgentChatSetupSnapshot};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentChatSetupAction {
    Retry,
    Install,
    Authenticate,
    OpenCatalog,
    SelectAgent,
}

impl AgentChatSetupAction {
    /// Convert to the protocol-layer action kind for serialization.
    pub(crate) fn to_protocol_kind(self) -> AgentChatSetupActionKind {
        match self {
            Self::Retry => AgentChatSetupActionKind::Retry,
            Self::Install => AgentChatSetupActionKind::Install,
            Self::Authenticate => AgentChatSetupActionKind::Authenticate,
            Self::OpenCatalog => AgentChatSetupActionKind::OpenCatalog,
            Self::SelectAgent => AgentChatSetupActionKind::SelectAgent,
        }
    }

    /// Convert from the protocol-layer action kind.
    pub(crate) fn from_protocol_kind(kind: AgentChatSetupActionKind) -> Self {
        match kind {
            AgentChatSetupActionKind::Retry => Self::Retry,
            AgentChatSetupActionKind::Install => Self::Install,
            AgentChatSetupActionKind::Authenticate => Self::Authenticate,
            AgentChatSetupActionKind::OpenCatalog => Self::OpenCatalog,
            AgentChatSetupActionKind::SelectAgent => Self::SelectAgent,
            // Automation-only variants map to SelectAgent as the closest
            // internal equivalent; the picker open/close is handled at the
            // view layer via `perform_setup_automation_action`.
            AgentChatSetupActionKind::OpenAgentPicker
            | AgentChatSetupActionKind::CloseAgentPicker => Self::SelectAgent,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AgentChatInlineSetupState {
    /// Machine-readable reason code for the setup blocker.
    pub reason_code: &'static str,
    pub title: SharedString,
    pub body: SharedString,
    pub primary_action: AgentChatSetupAction,
    pub secondary_action: Option<AgentChatSetupAction>,
    pub selected_agent: Option<AgentChatAgentCatalogEntry>,
    /// The catalog entries available for agent selection in setup mode.
    pub catalog_entries: Vec<AgentChatAgentCatalogEntry>,
    /// Capability requirements derived from the current Agent Chat entry path.
    pub launch_requirements: AgentChatLaunchRequirements,
}

/// Returns `true` if at least one launchable agent exists that is NOT the
/// currently selected one.
fn has_launchable_alternative(
    selected_agent: Option<&AgentChatAgentCatalogEntry>,
    catalog_entries: &[AgentChatAgentCatalogEntry],
) -> bool {
    let selected_id = selected_agent.map(|agent| agent.id.as_ref());
    catalog_entries
        .iter()
        .any(|entry| entry.is_launchable() && Some(entry.id.as_ref()) != selected_id)
}

/// Returns `true` if at least one launchable agent exists that is NOT the
/// currently selected one and satisfies the active launch requirements.
fn has_launchable_capable_alternative(
    selected_agent: Option<&AgentChatAgentCatalogEntry>,
    catalog_entries: &[AgentChatAgentCatalogEntry],
    requirements: AgentChatLaunchRequirements,
) -> bool {
    let selected_id = selected_agent.map(|agent| agent.id.as_ref());
    catalog_entries.iter().any(|entry| {
        entry.is_launchable()
            && entry.satisfies_requirements(requirements)
            && Some(entry.id.as_ref()) != selected_id
    })
}

/// Human-readable message for when no compatible capable fallback exists.
fn capability_gap_message(requirements: AgentChatLaunchRequirements) -> &'static str {
    if requirements.needs_image {
        "No compatible ready agent supports the image or screenshot attachments required for this request."
    } else if requirements.needs_embedded_context {
        "No compatible ready agent supports the embedded context required for this request."
    } else {
        "No compatible ready agent is available for this request."
    }
}

impl AgentChatInlineSetupState {
    /// Build inline setup state from a runtime `SetupRequired` event.
    ///
    /// Called when the Agent Chat client emits `AgentChatEvent::SetupRequired` during a
    /// live session (e.g. auth expired mid-conversation). Now receives the
    /// full catalog so it can suggest switching to a ready alternative.
    pub(crate) fn from_runtime_setup_required(
        selected_agent: Option<AgentChatAgentCatalogEntry>,
        catalog_entries: Vec<AgentChatAgentCatalogEntry>,
        launch_requirements: AgentChatLaunchRequirements,
        reason: &str,
        auth_methods: &[String],
    ) -> Self {
        let can_switch = has_launchable_capable_alternative(
            selected_agent.as_ref(),
            &catalog_entries,
            launch_requirements,
        );

        match reason {
            "auth_required" if can_switch => Self {
                reason_code: "authenticationRequired",
                title: "Authentication required".into(),
                body: if auth_methods.is_empty() {
                    "The selected agent needs authentication, but another compatible ready agent is available.".into()
                } else {
                    format!(
                        "The selected agent needs authentication ({}) but another compatible ready agent is available.",
                        auth_methods.join(", ")
                    ).into()
                },
                primary_action: AgentChatSetupAction::SelectAgent,
                secondary_action: Some(AgentChatSetupAction::Retry),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            "auth_required" => Self {
                reason_code: "authenticationRequired",
                title: "Authentication required".into(),
                body: if auth_methods.is_empty() {
                    "Authenticate the selected agent, then retry this chat.".into()
                } else {
                    format!(
                        "Authenticate the selected agent, then retry this chat. Available methods: {}.",
                        auth_methods.join(", ")
                    )
                    .into()
                },
                primary_action: AgentChatSetupAction::Authenticate,
                secondary_action: Some(AgentChatSetupAction::Retry),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            _ if can_switch => Self {
                reason_code: "runtimeSetupRequired",
                title: "Agent setup required".into(),
                body:
                    "The selected agent cannot continue, but another compatible ready agent is available."
                        .into(),
                primary_action: AgentChatSetupAction::SelectAgent,
                secondary_action: Some(AgentChatSetupAction::Retry),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            _ => Self {
                reason_code: "runtimeSetupRequired",
                title: "Agent setup required".into(),
                body: format!("Agent reported setup requirement: {reason}").into(),
                primary_action: AgentChatSetupAction::Retry,
                secondary_action: Some(AgentChatSetupAction::OpenCatalog),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
        }
    }

    pub(crate) fn from_resolution(
        resolution: &AgentChatLaunchResolution,
        launch_requirements: AgentChatLaunchRequirements,
    ) -> Self {
        let selected_agent = resolution.selected_agent.clone();
        let catalog_entries = resolution.catalog_entries.clone();
        let can_switch = has_launchable_alternative(selected_agent.as_ref(), &catalog_entries);
        let can_switch_capable = has_launchable_capable_alternative(
            selected_agent.as_ref(),
            &catalog_entries,
            launch_requirements,
        );

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "agent_chat_setup_state_from_resolution",
            blocker = ?resolution.blocker,
            selected_agent_id = selected_agent.as_ref().map(|agent| agent.id.as_ref()),
            can_switch,
            can_switch_capable,
            needs_embedded_context = launch_requirements.needs_embedded_context,
            needs_image = launch_requirements.needs_image,
        );

        match resolution.blocker {
            Some(AgentChatLaunchBlocker::NoAgentsAvailable) => Self {
                reason_code: "noAgentsAvailable",
                title: "No agents available".into(),
                body: "Add an agent in ~/.scriptkit/agent_chat/agents.json, then retry.".into(),
                primary_action: AgentChatSetupAction::OpenCatalog,
                secondary_action: Some(AgentChatSetupAction::Retry),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            Some(AgentChatLaunchBlocker::AgentNotInstalled) if can_switch_capable => Self {
                reason_code: "agentNotInstalled",
                title: "Agent install required".into(),
                body: "The preferred agent is not installed, but another compatible ready agent is available.".into(),
                primary_action: AgentChatSetupAction::SelectAgent,
                secondary_action: Some(AgentChatSetupAction::Retry),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            Some(AgentChatLaunchBlocker::AgentNotInstalled) if can_switch => Self {
                reason_code: "agentNotInstalled",
                title: "Agent install required".into(),
                body: format!(
                    "The preferred agent is not installed. {}",
                    capability_gap_message(launch_requirements)
                ).into(),
                primary_action: AgentChatSetupAction::Install,
                secondary_action: Some(AgentChatSetupAction::SelectAgent),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            Some(AgentChatLaunchBlocker::AgentNotInstalled) => Self {
                reason_code: "agentNotInstalled",
                title: "Agent install required".into(),
                body: selected_agent
                    .as_ref()
                    .and_then(|agent| agent.install_hint.clone())
                    .unwrap_or_else(|| "Install the selected agent, then retry.".into()),
                primary_action: AgentChatSetupAction::Install,
                secondary_action: Some(AgentChatSetupAction::SelectAgent),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            Some(AgentChatLaunchBlocker::AuthenticationRequired) if can_switch_capable => Self {
                reason_code: "authenticationRequired",
                title: "Authentication required".into(),
                body: "The selected agent needs authentication, but another compatible ready agent is available.".into(),
                primary_action: AgentChatSetupAction::SelectAgent,
                secondary_action: Some(AgentChatSetupAction::Retry),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            Some(AgentChatLaunchBlocker::AuthenticationRequired) if can_switch => Self {
                reason_code: "authenticationRequired",
                title: "Authentication required".into(),
                body: format!(
                    "Authenticate the selected agent to continue this request. {}",
                    capability_gap_message(launch_requirements)
                ).into(),
                primary_action: AgentChatSetupAction::Authenticate,
                secondary_action: Some(AgentChatSetupAction::SelectAgent),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            Some(AgentChatLaunchBlocker::AuthenticationRequired) => Self {
                reason_code: "authenticationRequired",
                title: "Authentication required".into(),
                body: "Authenticate the selected agent, then retry this chat.".into(),
                primary_action: AgentChatSetupAction::Authenticate,
                secondary_action: Some(AgentChatSetupAction::SelectAgent),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            Some(AgentChatLaunchBlocker::AgentMisconfigured) if can_switch_capable => Self {
                reason_code: "agentMisconfigured",
                title: "Agent configuration required".into(),
                body: "The selected agent is misconfigured, but another compatible ready agent is available.".into(),
                primary_action: AgentChatSetupAction::SelectAgent,
                secondary_action: Some(AgentChatSetupAction::Retry),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            Some(AgentChatLaunchBlocker::AgentMisconfigured) if can_switch => Self {
                reason_code: "agentMisconfigured",
                title: "Agent configuration required".into(),
                body: format!(
                    "Fix the selected agent configuration to continue this request. {}",
                    capability_gap_message(launch_requirements)
                ).into(),
                primary_action: AgentChatSetupAction::OpenCatalog,
                secondary_action: Some(AgentChatSetupAction::SelectAgent),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            Some(AgentChatLaunchBlocker::AgentMisconfigured) => Self {
                reason_code: "agentMisconfigured",
                title: "Agent configuration required".into(),
                body: "Fix the agent configuration in ~/.scriptkit/agent_chat/agents.json, then retry."
                    .into(),
                primary_action: AgentChatSetupAction::OpenCatalog,
                secondary_action: Some(AgentChatSetupAction::SelectAgent),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            Some(AgentChatLaunchBlocker::UnsupportedAgent) => Self {
                reason_code: "unsupportedAgent",
                title: "Unsupported agent".into(),
                body: "The selected agent is not available on this machine.".into(),
                primary_action: AgentChatSetupAction::SelectAgent,
                secondary_action: Some(AgentChatSetupAction::OpenCatalog),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            Some(AgentChatLaunchBlocker::CapabilityMismatch) if can_switch_capable => Self {
                reason_code: "capabilityMismatch",
                title: "Agent capability mismatch".into(),
                body: "The selected agent cannot satisfy this request, but another ready compatible agent is available.".into(),
                primary_action: AgentChatSetupAction::SelectAgent,
                secondary_action: Some(AgentChatSetupAction::Retry),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            Some(AgentChatLaunchBlocker::CapabilityMismatch) => Self {
                reason_code: "capabilityMismatch",
                title: "Agent capability mismatch".into(),
                body: capability_gap_message(launch_requirements).into(),
                primary_action: AgentChatSetupAction::Retry,
                secondary_action: Some(AgentChatSetupAction::OpenCatalog),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            None => Self {
                reason_code: "ready",
                title: "Agent ready".into(),
                body: "The selected agent is ready to launch.".into(),
                primary_action: AgentChatSetupAction::Retry,
                secondary_action: None,
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
        }
    }

    /// Build a protocol-layer setup snapshot for agentic inspection.
    ///
    /// `agent_picker_open` and `agent_picker_selected_id` are passed in from
    /// the view's `setup_agent_picker` state since the setup state itself
    /// does not own the picker overlay.
    pub(crate) fn to_protocol_snapshot(
        &self,
        agent_picker_open: bool,
        agent_picker_selected_id: Option<String>,
    ) -> AgentChatSetupSnapshot {
        let compatible_agent_ids: Vec<String> = self
            .catalog_entries
            .iter()
            .filter(|entry| entry.satisfies_requirements(self.launch_requirements))
            .map(|entry| entry.id.to_string())
            .collect();

        AgentChatSetupSnapshot {
            reason_code: self.reason_code.to_string(),
            title: self.title.to_string(),
            body: self.body.to_string(),
            primary_action: self.primary_action.to_protocol_kind(),
            secondary_action: self.secondary_action.map(|a| a.to_protocol_kind()),
            selected_agent_id: self.selected_agent.as_ref().map(|a| a.id.to_string()),
            catalog_agent_ids: self
                .catalog_entries
                .iter()
                .map(|entry| entry.id.to_string())
                .collect(),
            compatible_agent_ids,
            needs_image: self.launch_requirements.needs_image,
            needs_embedded_context: self.launch_requirements.needs_embedded_context,
            agent_picker_open,
            agent_picker_selected_id,
        }
    }
}
