//! Inline setup state for ACP chat views.
//!
//! When no launchable agent exists, the ACP chat view renders an inline
//! setup card instead of a dead-end toast. This module defines the
//! structured state that drives that card.

use gpui::SharedString;

use super::catalog::AcpAgentCatalogEntry;
use super::preflight::{AcpLaunchBlocker, AcpLaunchRequirements, AcpLaunchResolution};
use crate::protocol::{AcpSetupActionKind, AcpSetupSnapshot};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AcpSetupAction {
    Retry,
    Install,
    Authenticate,
    OpenCatalog,
    SelectAgent,
}

impl AcpSetupAction {
    /// Convert to the protocol-layer action kind for serialization.
    pub(crate) fn to_protocol_kind(self) -> AcpSetupActionKind {
        match self {
            Self::Retry => AcpSetupActionKind::Retry,
            Self::Install => AcpSetupActionKind::Install,
            Self::Authenticate => AcpSetupActionKind::Authenticate,
            Self::OpenCatalog => AcpSetupActionKind::OpenCatalog,
            Self::SelectAgent => AcpSetupActionKind::SelectAgent,
        }
    }

    /// Convert from the protocol-layer action kind.
    pub(crate) fn from_protocol_kind(kind: AcpSetupActionKind) -> Self {
        match kind {
            AcpSetupActionKind::Retry => Self::Retry,
            AcpSetupActionKind::Install => Self::Install,
            AcpSetupActionKind::Authenticate => Self::Authenticate,
            AcpSetupActionKind::OpenCatalog => Self::OpenCatalog,
            AcpSetupActionKind::SelectAgent => Self::SelectAgent,
            // Automation-only variants map to SelectAgent as the closest
            // internal equivalent; the picker open/close is handled at the
            // view layer via `perform_setup_automation_action`.
            AcpSetupActionKind::OpenAgentPicker | AcpSetupActionKind::CloseAgentPicker => {
                Self::SelectAgent
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AcpInlineSetupState {
    /// Machine-readable reason code for the setup blocker.
    pub reason_code: &'static str,
    pub title: SharedString,
    pub body: SharedString,
    pub primary_action: AcpSetupAction,
    pub secondary_action: Option<AcpSetupAction>,
    pub selected_agent: Option<AcpAgentCatalogEntry>,
    /// The catalog entries available for agent selection in setup mode.
    pub catalog_entries: Vec<AcpAgentCatalogEntry>,
    /// Capability requirements derived from the current ACP entry path.
    pub launch_requirements: AcpLaunchRequirements,
}

/// Returns `true` if at least one launchable agent exists that is NOT the
/// currently selected one.
fn has_launchable_alternative(
    selected_agent: Option<&AcpAgentCatalogEntry>,
    catalog_entries: &[AcpAgentCatalogEntry],
) -> bool {
    let selected_id = selected_agent.map(|agent| agent.id.as_ref());
    catalog_entries
        .iter()
        .any(|entry| entry.is_launchable() && Some(entry.id.as_ref()) != selected_id)
}

/// Returns `true` if at least one launchable agent exists that is NOT the
/// currently selected one and satisfies the active launch requirements.
fn has_launchable_capable_alternative(
    selected_agent: Option<&AcpAgentCatalogEntry>,
    catalog_entries: &[AcpAgentCatalogEntry],
    requirements: AcpLaunchRequirements,
) -> bool {
    let selected_id = selected_agent.map(|agent| agent.id.as_ref());
    catalog_entries.iter().any(|entry| {
        entry.is_launchable()
            && entry.satisfies_requirements(requirements)
            && Some(entry.id.as_ref()) != selected_id
    })
}

/// Human-readable message for when no compatible capable fallback exists.
fn capability_gap_message(requirements: AcpLaunchRequirements) -> &'static str {
    if requirements.needs_image {
        "No compatible ready agent supports the image or screenshot attachments required for this request."
    } else if requirements.needs_embedded_context {
        "No compatible ready agent supports the embedded context required for this request."
    } else {
        "No compatible ready agent is available for this request."
    }
}

impl AcpInlineSetupState {
    /// Build inline setup state from a runtime `SetupRequired` event.
    ///
    /// Called when the ACP client emits `AcpEvent::SetupRequired` during a
    /// live session (e.g. auth expired mid-conversation). Now receives the
    /// full catalog so it can suggest switching to a ready alternative.
    pub(crate) fn from_runtime_setup_required(
        selected_agent: Option<AcpAgentCatalogEntry>,
        catalog_entries: Vec<AcpAgentCatalogEntry>,
        launch_requirements: AcpLaunchRequirements,
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
                    "The selected ACP agent needs authentication, but another compatible ready agent is available.".into()
                } else {
                    format!(
                        "The selected ACP agent needs authentication ({}) but another compatible ready agent is available.",
                        auth_methods.join(", ")
                    ).into()
                },
                primary_action: AcpSetupAction::SelectAgent,
                secondary_action: Some(AcpSetupAction::Retry),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            "auth_required" => Self {
                reason_code: "authenticationRequired",
                title: "Authentication required".into(),
                body: if auth_methods.is_empty() {
                    "Authenticate the selected ACP agent, then retry this chat.".into()
                } else {
                    format!(
                        "Authenticate the selected ACP agent, then retry this chat. Available methods: {}.",
                        auth_methods.join(", ")
                    )
                    .into()
                },
                primary_action: AcpSetupAction::Authenticate,
                secondary_action: Some(AcpSetupAction::Retry),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            _ if can_switch => Self {
                reason_code: "runtimeSetupRequired",
                title: "ACP agent setup required".into(),
                body:
                    "The selected ACP agent cannot continue, but another compatible ready agent is available."
                        .into(),
                primary_action: AcpSetupAction::SelectAgent,
                secondary_action: Some(AcpSetupAction::Retry),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            _ => Self {
                reason_code: "runtimeSetupRequired",
                title: "ACP agent setup required".into(),
                body: format!("Agent reported setup requirement: {reason}").into(),
                primary_action: AcpSetupAction::Retry,
                secondary_action: Some(AcpSetupAction::OpenCatalog),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
        }
    }

    pub(crate) fn from_resolution(
        resolution: &AcpLaunchResolution,
        launch_requirements: AcpLaunchRequirements,
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
            event = "acp_setup_state_from_resolution",
            blocker = ?resolution.blocker,
            selected_agent_id = selected_agent.as_ref().map(|agent| agent.id.as_ref()),
            can_switch,
            can_switch_capable,
            needs_embedded_context = launch_requirements.needs_embedded_context,
            needs_image = launch_requirements.needs_image,
        );

        match resolution.blocker {
            Some(AcpLaunchBlocker::NoAgentsAvailable) => Self {
                reason_code: "noAgentsAvailable",
                title: "No ACP agents available".into(),
                body: "Add an ACP agent in ~/.scriptkit/acp/agents.json, then retry.".into(),
                primary_action: AcpSetupAction::OpenCatalog,
                secondary_action: Some(AcpSetupAction::Retry),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            Some(AcpLaunchBlocker::AgentNotInstalled) if can_switch_capable => Self {
                reason_code: "agentNotInstalled",
                title: "Agent install required".into(),
                body: "The preferred ACP agent is not installed, but another compatible ready agent is available.".into(),
                primary_action: AcpSetupAction::SelectAgent,
                secondary_action: Some(AcpSetupAction::Retry),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            Some(AcpLaunchBlocker::AgentNotInstalled) if can_switch => Self {
                reason_code: "agentNotInstalled",
                title: "Agent install required".into(),
                body: format!(
                    "The preferred ACP agent is not installed. {}",
                    capability_gap_message(launch_requirements)
                ).into(),
                primary_action: AcpSetupAction::Install,
                secondary_action: Some(AcpSetupAction::SelectAgent),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            Some(AcpLaunchBlocker::AgentNotInstalled) => Self {
                reason_code: "agentNotInstalled",
                title: "Agent install required".into(),
                body: selected_agent
                    .as_ref()
                    .and_then(|agent| agent.install_hint.clone())
                    .unwrap_or_else(|| "Install the selected ACP agent, then retry.".into()),
                primary_action: AcpSetupAction::Install,
                secondary_action: Some(AcpSetupAction::SelectAgent),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            Some(AcpLaunchBlocker::AuthenticationRequired) if can_switch_capable => Self {
                reason_code: "authenticationRequired",
                title: "Authentication required".into(),
                body: "The selected ACP agent needs authentication, but another compatible ready agent is available.".into(),
                primary_action: AcpSetupAction::SelectAgent,
                secondary_action: Some(AcpSetupAction::Retry),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            Some(AcpLaunchBlocker::AuthenticationRequired) if can_switch => Self {
                reason_code: "authenticationRequired",
                title: "Authentication required".into(),
                body: format!(
                    "Authenticate the selected ACP agent to continue this request. {}",
                    capability_gap_message(launch_requirements)
                ).into(),
                primary_action: AcpSetupAction::Authenticate,
                secondary_action: Some(AcpSetupAction::SelectAgent),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            Some(AcpLaunchBlocker::AuthenticationRequired) => Self {
                reason_code: "authenticationRequired",
                title: "Authentication required".into(),
                body: "Authenticate the selected ACP agent, then retry this chat.".into(),
                primary_action: AcpSetupAction::Authenticate,
                secondary_action: Some(AcpSetupAction::SelectAgent),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            Some(AcpLaunchBlocker::AgentMisconfigured) if can_switch_capable => Self {
                reason_code: "agentMisconfigured",
                title: "Agent configuration required".into(),
                body: "The selected ACP agent is misconfigured, but another compatible ready agent is available.".into(),
                primary_action: AcpSetupAction::SelectAgent,
                secondary_action: Some(AcpSetupAction::Retry),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            Some(AcpLaunchBlocker::AgentMisconfigured) if can_switch => Self {
                reason_code: "agentMisconfigured",
                title: "Agent configuration required".into(),
                body: format!(
                    "Fix the selected ACP agent configuration to continue this request. {}",
                    capability_gap_message(launch_requirements)
                ).into(),
                primary_action: AcpSetupAction::OpenCatalog,
                secondary_action: Some(AcpSetupAction::SelectAgent),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            Some(AcpLaunchBlocker::AgentMisconfigured) => Self {
                reason_code: "agentMisconfigured",
                title: "Agent configuration required".into(),
                body: "Fix the agent configuration in ~/.scriptkit/acp/agents.json, then retry."
                    .into(),
                primary_action: AcpSetupAction::OpenCatalog,
                secondary_action: Some(AcpSetupAction::SelectAgent),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            Some(AcpLaunchBlocker::UnsupportedAgent) => Self {
                reason_code: "unsupportedAgent",
                title: "Unsupported ACP agent".into(),
                body: "The selected ACP agent is not available on this machine.".into(),
                primary_action: AcpSetupAction::SelectAgent,
                secondary_action: Some(AcpSetupAction::OpenCatalog),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            Some(AcpLaunchBlocker::CapabilityMismatch) if can_switch_capable => Self {
                reason_code: "capabilityMismatch",
                title: "ACP capability mismatch".into(),
                body: "The selected ACP agent cannot satisfy this request, but another ready compatible agent is available.".into(),
                primary_action: AcpSetupAction::SelectAgent,
                secondary_action: Some(AcpSetupAction::Retry),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            Some(AcpLaunchBlocker::CapabilityMismatch) => Self {
                reason_code: "capabilityMismatch",
                title: "ACP capability mismatch".into(),
                body: capability_gap_message(launch_requirements).into(),
                primary_action: AcpSetupAction::Retry,
                secondary_action: Some(AcpSetupAction::OpenCatalog),
                selected_agent,
                catalog_entries,
                launch_requirements,
            },
            None => Self {
                reason_code: "ready",
                title: "ACP ready".into(),
                body: "The selected ACP agent is ready to launch.".into(),
                primary_action: AcpSetupAction::Retry,
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
    ) -> AcpSetupSnapshot {
        let compatible_agent_ids: Vec<String> = self
            .catalog_entries
            .iter()
            .filter(|entry| entry.satisfies_requirements(self.launch_requirements))
            .map(|entry| entry.id.to_string())
            .collect();

        AcpSetupSnapshot {
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
