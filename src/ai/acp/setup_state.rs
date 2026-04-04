//! Inline setup state for ACP chat views.
//!
//! When no launchable agent exists, the ACP chat view renders an inline
//! setup card instead of a dead-end toast. This module defines the
//! structured state that drives that card.

use gpui::SharedString;

use super::catalog::AcpAgentCatalogEntry;
use super::preflight::{AcpLaunchBlocker, AcpLaunchResolution};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AcpSetupAction {
    Retry,
    Install,
    Authenticate,
    OpenCatalog,
    SelectAgent,
}

#[derive(Debug, Clone)]
pub(crate) struct AcpInlineSetupState {
    pub title: SharedString,
    pub body: SharedString,
    pub primary_action: AcpSetupAction,
    pub secondary_action: Option<AcpSetupAction>,
    pub selected_agent: Option<AcpAgentCatalogEntry>,
}

impl AcpInlineSetupState {
    /// Build inline setup state from a runtime `SetupRequired` event.
    ///
    /// Called when the ACP client emits `AcpEvent::SetupRequired` during a
    /// live session (e.g. auth expired mid-conversation). Preserves the
    /// selected agent context so the recovery card can show agent-specific
    /// guidance.
    pub(crate) fn from_runtime_setup_required(
        selected_agent: Option<AcpAgentCatalogEntry>,
        reason: &str,
        auth_methods: &[String],
    ) -> Self {
        match reason {
            "auth_required" => Self {
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
            },
            _ => Self {
                title: "ACP agent setup required".into(),
                body: format!("Agent reported setup requirement: {reason}").into(),
                primary_action: AcpSetupAction::Retry,
                secondary_action: Some(AcpSetupAction::OpenCatalog),
                selected_agent,
            },
        }
    }

    pub(crate) fn from_resolution(resolution: &AcpLaunchResolution) -> Self {
        let selected_agent = resolution.selected_agent.clone();

        match resolution.blocker {
            Some(AcpLaunchBlocker::NoAgentsAvailable) => Self {
                title: "No ACP agents available".into(),
                body: "Add an ACP agent in ~/.scriptkit/acp/agents.json, then retry.".into(),
                primary_action: AcpSetupAction::OpenCatalog,
                secondary_action: Some(AcpSetupAction::Retry),
                selected_agent,
            },
            Some(AcpLaunchBlocker::AgentNotInstalled) => Self {
                title: "Agent install required".into(),
                body: selected_agent
                    .as_ref()
                    .and_then(|agent| agent.install_hint.clone())
                    .unwrap_or_else(|| "Install the selected ACP agent, then retry.".into()),
                primary_action: AcpSetupAction::Install,
                secondary_action: Some(AcpSetupAction::Retry),
                selected_agent,
            },
            Some(AcpLaunchBlocker::AuthenticationRequired) => Self {
                title: "Authentication required".into(),
                body: "Authenticate the selected ACP agent, then retry this chat.".into(),
                primary_action: AcpSetupAction::Authenticate,
                secondary_action: Some(AcpSetupAction::Retry),
                selected_agent,
            },
            Some(AcpLaunchBlocker::AgentMisconfigured) => Self {
                title: "Agent configuration required".into(),
                body: "Fix the agent configuration in ~/.scriptkit/acp/agents.json, then retry."
                    .into(),
                primary_action: AcpSetupAction::OpenCatalog,
                secondary_action: Some(AcpSetupAction::Retry),
                selected_agent,
            },
            Some(AcpLaunchBlocker::UnsupportedAgent) => Self {
                title: "Unsupported ACP agent".into(),
                body: "The selected ACP agent is not available on this machine.".into(),
                primary_action: AcpSetupAction::SelectAgent,
                secondary_action: Some(AcpSetupAction::OpenCatalog),
                selected_agent,
            },
            None => Self {
                title: "ACP ready".into(),
                body: "The selected ACP agent is ready to launch.".into(),
                primary_action: AcpSetupAction::Retry,
                secondary_action: None,
                selected_agent,
            },
        }
    }
}
