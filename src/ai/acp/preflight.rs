//! ACP launch preflight resolution.
//!
//! Resolves a catalog of agents into a deterministic launch decision:
//! either a ready agent or a structured blocker state that the UI
//! can render inline instead of dead-ending with a toast.

use gpui::SharedString;

use super::catalog::{
    AcpAgentAuthState, AcpAgentCatalogEntry, AcpAgentConfigState, AcpAgentInstallState,
};

/// Why an ACP launch cannot proceed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AcpLaunchBlocker {
    /// No agents in the catalog at all.
    NoAgentsAvailable,
    /// Selected agent binary is not installed.
    AgentNotInstalled,
    /// Selected agent requires authentication.
    AuthenticationRequired,
    /// Selected agent config is missing or invalid.
    AgentMisconfigured,
    /// Selected agent is unsupported on this machine.
    UnsupportedAgent,
}

/// Result of preflight resolution: a selected agent and optional blocker.
#[derive(Debug, Clone)]
pub(crate) struct AcpLaunchResolution {
    pub selected_agent: Option<AcpAgentCatalogEntry>,
    pub blocker: Option<AcpLaunchBlocker>,
    /// The full catalog used for this resolution, carried so setup UIs can
    /// render the same agent list without re-loading from disk.
    pub catalog_entries: Vec<AcpAgentCatalogEntry>,
}

impl AcpLaunchResolution {
    /// Ready to launch: has an agent and no blockers.
    pub(crate) fn is_ready(&self) -> bool {
        self.selected_agent.is_some() && self.blocker.is_none()
    }

    /// The selected agent's ID, if any.
    pub(crate) fn selected_agent_id(&self) -> Option<&str> {
        self.selected_agent.as_ref().map(|entry| entry.id.as_ref())
    }
}

/// Resolve which agent to launch and whether anything blocks it.
///
/// Selection priority:
/// 1. Preferred agent ID (if provided and found in catalog)
/// 2. First launchable agent
/// 3. First agent in catalog (will have a blocker)
/// 4. None (empty catalog → NoAgentsAvailable)
pub(crate) fn resolve_default_acp_launch(
    agents: &[AcpAgentCatalogEntry],
    preferred_agent_id: Option<&str>,
) -> AcpLaunchResolution {
    let selected = preferred_agent_id
        .and_then(|preferred| agents.iter().find(|entry| entry.id.as_ref() == preferred))
        .cloned()
        .or_else(|| agents.iter().find(|entry| entry.is_launchable()).cloned())
        .or_else(|| agents.first().cloned());

    let Some(selected_agent) = selected else {
        return AcpLaunchResolution {
            selected_agent: None,
            blocker: Some(AcpLaunchBlocker::NoAgentsAvailable),
            catalog_entries: agents.to_vec(),
        };
    };

    let blocker = match (
        selected_agent.install_state,
        selected_agent.auth_state,
        selected_agent.config_state,
    ) {
        (AcpAgentInstallState::Unsupported, _, _) => Some(AcpLaunchBlocker::UnsupportedAgent),
        (AcpAgentInstallState::NeedsInstall, _, _) => Some(AcpLaunchBlocker::AgentNotInstalled),
        (_, AcpAgentAuthState::NeedsAuthentication, _) => {
            Some(AcpLaunchBlocker::AuthenticationRequired)
        }
        (_, _, AcpAgentConfigState::Missing | AcpAgentConfigState::Invalid) => {
            Some(AcpLaunchBlocker::AgentMisconfigured)
        }
        _ => None,
    };

    AcpLaunchResolution {
        selected_agent: Some(selected_agent),
        blocker,
        catalog_entries: agents.to_vec(),
    }
}

/// Human-readable title for the resolution state.
pub(crate) fn setup_title_for_resolution(resolution: &AcpLaunchResolution) -> SharedString {
    match resolution.blocker {
        Some(AcpLaunchBlocker::NoAgentsAvailable) => "No ACP agents available".into(),
        Some(AcpLaunchBlocker::AgentNotInstalled) => "ACP agent install required".into(),
        Some(AcpLaunchBlocker::AuthenticationRequired) => {
            "ACP agent authentication required".into()
        }
        Some(AcpLaunchBlocker::AgentMisconfigured) => "ACP agent configuration required".into(),
        Some(AcpLaunchBlocker::UnsupportedAgent) => "ACP agent unsupported".into(),
        None => "ACP ready".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::acp::catalog::{AcpAgentAuthState, AcpAgentSource};

    fn make_entry(
        id: &str,
        install: AcpAgentInstallState,
        auth: AcpAgentAuthState,
        config: AcpAgentConfigState,
    ) -> AcpAgentCatalogEntry {
        AcpAgentCatalogEntry {
            id: id.to_string().into(),
            display_name: id.to_string().into(),
            source: AcpAgentSource::ScriptKitCatalog,
            install_state: install,
            auth_state: auth,
            config_state: config,
            install_hint: None,
            config_hint: None,
            config: None,
        }
    }

    #[test]
    fn empty_catalog_returns_no_agents_blocker() {
        let result = resolve_default_acp_launch(&[], None);
        assert_eq!(result.blocker, Some(AcpLaunchBlocker::NoAgentsAvailable));
        assert!(!result.is_ready());
        assert!(result.selected_agent.is_none());
    }

    #[test]
    fn ready_agent_returns_no_blocker() {
        let agents = vec![make_entry(
            "ready",
            AcpAgentInstallState::Ready,
            AcpAgentAuthState::Unknown,
            AcpAgentConfigState::Valid,
        )];
        let result = resolve_default_acp_launch(&agents, None);
        assert!(result.is_ready());
        assert_eq!(result.selected_agent_id(), Some("ready"));
        assert!(result.blocker.is_none());
    }

    #[test]
    fn prefers_ready_agent_over_blocked() {
        let agents = vec![
            make_entry(
                "blocked",
                AcpAgentInstallState::NeedsInstall,
                AcpAgentAuthState::Unknown,
                AcpAgentConfigState::Valid,
            ),
            make_entry(
                "ready",
                AcpAgentInstallState::Ready,
                AcpAgentAuthState::Unknown,
                AcpAgentConfigState::Valid,
            ),
        ];
        let result = resolve_default_acp_launch(&agents, None);
        assert_eq!(result.selected_agent_id(), Some("ready"));
        assert!(result.is_ready());
    }

    #[test]
    fn preferred_agent_id_overrides_auto_selection() {
        let agents = vec![
            make_entry(
                "auto-ready",
                AcpAgentInstallState::Ready,
                AcpAgentAuthState::Unknown,
                AcpAgentConfigState::Valid,
            ),
            make_entry(
                "preferred-blocked",
                AcpAgentInstallState::NeedsInstall,
                AcpAgentAuthState::Unknown,
                AcpAgentConfigState::Valid,
            ),
        ];
        let result = resolve_default_acp_launch(&agents, Some("preferred-blocked"));
        assert_eq!(result.selected_agent_id(), Some("preferred-blocked"));
        assert_eq!(result.blocker, Some(AcpLaunchBlocker::AgentNotInstalled));
    }

    #[test]
    fn needs_install_blocker() {
        let agents = vec![make_entry(
            "missing",
            AcpAgentInstallState::NeedsInstall,
            AcpAgentAuthState::Unknown,
            AcpAgentConfigState::Valid,
        )];
        let result = resolve_default_acp_launch(&agents, None);
        assert_eq!(result.blocker, Some(AcpLaunchBlocker::AgentNotInstalled));
    }

    #[test]
    fn unsupported_blocker() {
        let agents = vec![make_entry(
            "unsupported",
            AcpAgentInstallState::Unsupported,
            AcpAgentAuthState::Unknown,
            AcpAgentConfigState::Valid,
        )];
        let result = resolve_default_acp_launch(&agents, None);
        assert_eq!(result.blocker, Some(AcpLaunchBlocker::UnsupportedAgent));
    }

    #[test]
    fn auth_required_blocker() {
        let agents = vec![make_entry(
            "needs-auth",
            AcpAgentInstallState::Ready,
            AcpAgentAuthState::NeedsAuthentication,
            AcpAgentConfigState::Valid,
        )];
        let result = resolve_default_acp_launch(&agents, None);
        assert_eq!(
            result.blocker,
            Some(AcpLaunchBlocker::AuthenticationRequired)
        );
    }

    #[test]
    fn misconfigured_blocker() {
        let agents = vec![make_entry(
            "bad-config",
            AcpAgentInstallState::Ready,
            AcpAgentAuthState::Unknown,
            AcpAgentConfigState::Invalid,
        )];
        let result = resolve_default_acp_launch(&agents, None);
        assert_eq!(result.blocker, Some(AcpLaunchBlocker::AgentMisconfigured));
    }

    #[test]
    fn auth_required_agent_is_skipped_for_auto_selection() {
        let agents = vec![
            make_entry(
                "needs-auth",
                AcpAgentInstallState::Ready,
                AcpAgentAuthState::NeedsAuthentication,
                AcpAgentConfigState::Valid,
            ),
            make_entry(
                "ready",
                AcpAgentInstallState::Ready,
                AcpAgentAuthState::Authenticated,
                AcpAgentConfigState::Valid,
            ),
        ];
        let result = resolve_default_acp_launch(&agents, None);
        assert_eq!(result.selected_agent_id(), Some("ready"));
        assert!(result.blocker.is_none());
    }

    #[test]
    fn preferred_auth_required_agent_gets_blocker() {
        let agents = vec![
            make_entry(
                "needs-auth",
                AcpAgentInstallState::Ready,
                AcpAgentAuthState::NeedsAuthentication,
                AcpAgentConfigState::Valid,
            ),
            make_entry(
                "ready",
                AcpAgentInstallState::Ready,
                AcpAgentAuthState::Authenticated,
                AcpAgentConfigState::Valid,
            ),
        ];
        let result = resolve_default_acp_launch(&agents, Some("needs-auth"));
        assert_eq!(result.selected_agent_id(), Some("needs-auth"));
        assert_eq!(
            result.blocker,
            Some(AcpLaunchBlocker::AuthenticationRequired)
        );
    }

    #[test]
    fn setup_title_covers_all_blockers() {
        let titles = vec![
            setup_title_for_resolution(&AcpLaunchResolution {
                selected_agent: None,
                blocker: Some(AcpLaunchBlocker::NoAgentsAvailable),
                catalog_entries: vec![],
            }),
            setup_title_for_resolution(&AcpLaunchResolution {
                selected_agent: None,
                blocker: Some(AcpLaunchBlocker::AgentNotInstalled),
                catalog_entries: vec![],
            }),
            setup_title_for_resolution(&AcpLaunchResolution {
                selected_agent: None,
                blocker: Some(AcpLaunchBlocker::AuthenticationRequired),
                catalog_entries: vec![],
            }),
            setup_title_for_resolution(&AcpLaunchResolution {
                selected_agent: None,
                blocker: Some(AcpLaunchBlocker::AgentMisconfigured),
                catalog_entries: vec![],
            }),
            setup_title_for_resolution(&AcpLaunchResolution {
                selected_agent: None,
                blocker: Some(AcpLaunchBlocker::UnsupportedAgent),
                catalog_entries: vec![],
            }),
            setup_title_for_resolution(&AcpLaunchResolution {
                selected_agent: None,
                blocker: None,
                catalog_entries: vec![],
            }),
        ];
        // Each title is non-empty and unique
        for title in &titles {
            assert!(!title.is_empty());
        }
        let unique: std::collections::HashSet<&str> = titles.iter().map(|t| t.as_ref()).collect();
        assert_eq!(unique.len(), titles.len(), "all titles should be unique");
    }
}
