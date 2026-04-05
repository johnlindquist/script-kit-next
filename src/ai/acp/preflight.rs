//! ACP launch preflight resolution.
//!
//! Resolves a catalog of agents into a deterministic launch decision:
//! either a ready agent or a structured blocker state that the UI
//! can render inline instead of dead-ending with a toast.

use std::cmp::Ordering;

use gpui::SharedString;

use super::catalog::{
    AcpAgentAuthState, AcpAgentCatalogEntry, AcpAgentConfigState, AcpAgentInstallState,
};

/// Capability requirements derived from the current ACP entry path.
///
/// Used by `resolve_acp_launch_with_requirements()` to select an agent
/// that can actually satisfy the request instead of just the first
/// install/auth/config-valid one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct AcpLaunchRequirements {
    /// The entry path includes embedded context blocks (desktop snapshot, etc.)
    pub needs_embedded_context: bool,
    /// The entry path includes image/screenshot attachments.
    pub needs_image: bool,
}

impl AcpAgentCatalogEntry {
    /// Whether this agent can satisfy the given capability requirements.
    ///
    /// `None` (unknown) is treated as capable — optimistic default so that
    /// agents without persisted runtime facts are not excluded prematurely.
    pub(crate) fn satisfies_requirements(&self, requirements: AcpLaunchRequirements) -> bool {
        let embedded_ok =
            !requirements.needs_embedded_context || self.supports_embedded_context.unwrap_or(true);
        let image_ok = !requirements.needs_image || self.supports_image.unwrap_or(true);
        embedded_ok && image_ok
    }
}

/// Compare two launchable candidates for ranking.
///
/// Ordering (best first):
/// 1. `last_session_ok == true` beats `false` (proven-good agent).
/// 2. Non-legacy source beats `LegacyClaudeCode` (generic-first).
/// 3. Stable alphabetical tie-breaker on `display_name`.
fn compare_launchable_candidates(
    a: &AcpAgentCatalogEntry,
    b: &AcpAgentCatalogEntry,
) -> Ordering {
    let a_is_legacy = matches!(a.source, super::catalog::AcpAgentSource::LegacyClaudeCode);
    let b_is_legacy = matches!(b.source, super::catalog::AcpAgentSource::LegacyClaudeCode);

    // Prefer last_session_ok == true (reverse: true > false).
    b.last_session_ok
        .cmp(&a.last_session_ok)
        // Prefer non-legacy (false < true, so legacy sorts after).
        .then_with(|| a_is_legacy.cmp(&b_is_legacy))
        // Stable alphabetical tie-breaker.
        .then_with(|| a.display_name.as_ref().cmp(b.display_name.as_ref()))
}

/// Pick the best launchable candidate from the catalog, optionally
/// filtering by capability requirements.
fn best_launchable_candidate(
    agents: &[AcpAgentCatalogEntry],
    requirements: Option<AcpLaunchRequirements>,
) -> Option<AcpAgentCatalogEntry> {
    let mut candidates: Vec<&AcpAgentCatalogEntry> = agents
        .iter()
        .filter(|entry| {
            entry.is_launchable()
                && requirements
                    .map(|req| entry.satisfies_requirements(req))
                    .unwrap_or(true)
        })
        .collect();

    candidates.sort_by(|a, b| compare_launchable_candidates(a, b));

    tracing::info!(
        target: "script_kit::tab_ai",
        event = "acp_launchable_candidate_ranked",
        needs_embedded_context = requirements.map(|r| r.needs_embedded_context),
        needs_image = requirements.map(|r| r.needs_image),
        candidate_ids = ?candidates
            .iter()
            .map(|entry| entry.id.as_ref())
            .collect::<Vec<_>>(),
    );

    candidates.into_iter().next().cloned()
}

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
    /// Selected agent is launchable but cannot satisfy the derived capability
    /// requirements (e.g. needs embedded context but agent doesn't support it).
    CapabilityMismatch,
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
/// 2. Best ranked launchable agent (last-known-good > non-legacy > alphabetical)
/// 3. First agent in catalog (will have a blocker)
/// 4. None (empty catalog → NoAgentsAvailable)
pub(crate) fn resolve_default_acp_launch(
    agents: &[AcpAgentCatalogEntry],
    preferred_agent_id: Option<&str>,
) -> AcpLaunchResolution {
    let selected = preferred_agent_id
        .and_then(|preferred| agents.iter().find(|entry| entry.id.as_ref() == preferred))
        .cloned()
        .or_else(|| best_launchable_candidate(agents, None))
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

/// Resolve which agent to launch using capability requirements.
///
/// Selection priority:
/// 1. Preferred agent ID (if launchable AND satisfies requirements)
/// 2. Best ranked launchable agent that satisfies requirements
/// 3. Preferred agent ID (if launchable, even without capability match)
/// 4. Best ranked launchable agent (capability mismatch blocker)
/// 5. Preferred agent (even if not launchable — will get install/auth blocker)
/// 6. First agent in catalog (will have a blocker)
/// 7. None (empty catalog → NoAgentsAvailable)
pub(crate) fn resolve_acp_launch_with_requirements(
    agents: &[AcpAgentCatalogEntry],
    preferred_agent_id: Option<&str>,
    requirements: AcpLaunchRequirements,
) -> AcpLaunchResolution {
    let preferred = preferred_agent_id
        .and_then(|preferred| agents.iter().find(|entry| entry.id.as_ref() == preferred));

    // Best case: preferred is launchable and satisfies requirements.
    let selected = preferred
        .filter(|entry| entry.is_launchable() && entry.satisfies_requirements(requirements))
        .cloned()
        // Fallback: best ranked launchable agent that satisfies requirements.
        .or_else(|| best_launchable_candidate(agents, Some(requirements)))
        // Fallback: preferred is launchable but doesn't satisfy requirements.
        .or_else(|| preferred.filter(|entry| entry.is_launchable()).cloned())
        // Fallback: best ranked launchable agent (will get CapabilityMismatch blocker).
        .or_else(|| best_launchable_candidate(agents, None))
        // Fallback: preferred even if not launchable (will get install/auth blocker).
        .or_else(|| preferred.cloned())
        // Last resort: first catalog entry.
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
        _ if !selected_agent.satisfies_requirements(requirements) => {
            Some(AcpLaunchBlocker::CapabilityMismatch)
        }
        _ => None,
    };

    tracing::info!(
        target: "script_kit::tab_ai",
        event = "acp_capability_resolution",
        preferred_agent_id = ?preferred_agent_id,
        needs_embedded_context = requirements.needs_embedded_context,
        needs_image = requirements.needs_image,
        selected_agent_id = selected_agent.id.as_ref(),
        supports_embedded_context = ?selected_agent.supports_embedded_context,
        supports_image = ?selected_agent.supports_image,
        blocker = ?blocker,
    );

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
        Some(AcpLaunchBlocker::CapabilityMismatch) => "ACP capability mismatch".into(),
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
            supports_embedded_context: None,
            supports_image: None,
            last_session_ok: false,
            config: None,
        }
    }

    fn make_entry_with_capabilities(
        id: &str,
        install: AcpAgentInstallState,
        auth: AcpAgentAuthState,
        config: AcpAgentConfigState,
        supports_embedded_context: Option<bool>,
        supports_image: Option<bool>,
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
            supports_embedded_context,
            supports_image,
            last_session_ok: false,
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

    // ---------------------------------------------------------------
    // Capability-aware resolution tests
    // ---------------------------------------------------------------

    #[test]
    fn capability_requirements_default_is_no_requirements() {
        let reqs = AcpLaunchRequirements::default();
        assert!(!reqs.needs_embedded_context);
        assert!(!reqs.needs_image);
    }

    #[test]
    fn satisfies_requirements_unknown_treated_as_capable() {
        let entry = make_entry(
            "unknown-caps",
            AcpAgentInstallState::Ready,
            AcpAgentAuthState::Unknown,
            AcpAgentConfigState::Valid,
        );
        let reqs = AcpLaunchRequirements {
            needs_embedded_context: true,
            needs_image: true,
        };
        assert!(
            entry.satisfies_requirements(reqs),
            "None capabilities should be treated as capable"
        );
    }

    #[test]
    fn satisfies_requirements_explicit_false_rejects() {
        let entry = make_entry_with_capabilities(
            "no-context",
            AcpAgentInstallState::Ready,
            AcpAgentAuthState::Unknown,
            AcpAgentConfigState::Valid,
            Some(false),
            Some(true),
        );
        let reqs = AcpLaunchRequirements {
            needs_embedded_context: true,
            needs_image: false,
        };
        assert!(
            !entry.satisfies_requirements(reqs),
            "explicit false should reject"
        );
    }

    #[test]
    fn satisfies_requirements_no_needs_always_passes() {
        let entry = make_entry_with_capabilities(
            "no-caps",
            AcpAgentInstallState::Ready,
            AcpAgentAuthState::Unknown,
            AcpAgentConfigState::Valid,
            Some(false),
            Some(false),
        );
        let reqs = AcpLaunchRequirements::default();
        assert!(
            entry.satisfies_requirements(reqs),
            "no requirements should always pass"
        );
    }

    #[test]
    fn capability_aware_prefers_capable_over_preferred() {
        let agents = vec![
            make_entry_with_capabilities(
                "claude-code",
                AcpAgentInstallState::Ready,
                AcpAgentAuthState::Unknown,
                AcpAgentConfigState::Valid,
                Some(false), // cannot do embedded context
                Some(true),
            ),
            make_entry_with_capabilities(
                "opencode",
                AcpAgentInstallState::Ready,
                AcpAgentAuthState::Unknown,
                AcpAgentConfigState::Valid,
                Some(true), // can do embedded context
                Some(true),
            ),
        ];
        let reqs = AcpLaunchRequirements {
            needs_embedded_context: true,
            needs_image: false,
        };
        let result = resolve_acp_launch_with_requirements(&agents, Some("claude-code"), reqs);
        assert_eq!(
            result.selected_agent_id(),
            Some("opencode"),
            "should fall back to capable agent"
        );
        assert!(result.is_ready());
    }

    #[test]
    fn capability_aware_uses_preferred_when_capable() {
        let agents = vec![
            make_entry_with_capabilities(
                "claude-code",
                AcpAgentInstallState::Ready,
                AcpAgentAuthState::Unknown,
                AcpAgentConfigState::Valid,
                Some(true),
                Some(true),
            ),
            make_entry_with_capabilities(
                "opencode",
                AcpAgentInstallState::Ready,
                AcpAgentAuthState::Unknown,
                AcpAgentConfigState::Valid,
                Some(true),
                Some(true),
            ),
        ];
        let reqs = AcpLaunchRequirements {
            needs_embedded_context: true,
            needs_image: false,
        };
        let result = resolve_acp_launch_with_requirements(&agents, Some("claude-code"), reqs);
        assert_eq!(result.selected_agent_id(), Some("claude-code"));
        assert!(result.is_ready());
    }

    #[test]
    fn capability_aware_returns_mismatch_when_none_capable() {
        let agents = vec![
            make_entry_with_capabilities(
                "agent-a",
                AcpAgentInstallState::Ready,
                AcpAgentAuthState::Unknown,
                AcpAgentConfigState::Valid,
                Some(false),
                Some(false),
            ),
            make_entry_with_capabilities(
                "agent-b",
                AcpAgentInstallState::Ready,
                AcpAgentAuthState::Unknown,
                AcpAgentConfigState::Valid,
                Some(false),
                Some(false),
            ),
        ];
        let reqs = AcpLaunchRequirements {
            needs_embedded_context: true,
            needs_image: false,
        };
        let result = resolve_acp_launch_with_requirements(&agents, None, reqs);
        assert_eq!(
            result.blocker,
            Some(AcpLaunchBlocker::CapabilityMismatch),
            "should return CapabilityMismatch when no agent satisfies requirements"
        );
    }

    #[test]
    fn capability_aware_no_requirements_uses_preferred() {
        let agents = vec![
            make_entry_with_capabilities(
                "agent-a",
                AcpAgentInstallState::Ready,
                AcpAgentAuthState::Unknown,
                AcpAgentConfigState::Valid,
                Some(false),
                Some(false),
            ),
            make_entry_with_capabilities(
                "agent-b",
                AcpAgentInstallState::Ready,
                AcpAgentAuthState::Unknown,
                AcpAgentConfigState::Valid,
                Some(true),
                Some(true),
            ),
        ];
        let reqs = AcpLaunchRequirements::default();
        let result = resolve_acp_launch_with_requirements(&agents, Some("agent-a"), reqs);
        assert_eq!(
            result.selected_agent_id(),
            Some("agent-a"),
            "with no requirements, preferred should win"
        );
        assert!(result.is_ready());
    }

    #[test]
    fn capability_aware_install_blocker_trumps_capability() {
        let agents = vec![make_entry_with_capabilities(
            "uninstalled",
            AcpAgentInstallState::NeedsInstall,
            AcpAgentAuthState::Unknown,
            AcpAgentConfigState::Valid,
            Some(true),
            Some(true),
        )];
        let reqs = AcpLaunchRequirements {
            needs_embedded_context: true,
            needs_image: false,
        };
        let result = resolve_acp_launch_with_requirements(&agents, None, reqs);
        assert_eq!(
            result.blocker,
            Some(AcpLaunchBlocker::AgentNotInstalled),
            "install blocker should take precedence over capability"
        );
    }

    #[test]
    fn capability_aware_empty_catalog() {
        let reqs = AcpLaunchRequirements {
            needs_embedded_context: true,
            needs_image: true,
        };
        let result = resolve_acp_launch_with_requirements(&[], None, reqs);
        assert_eq!(result.blocker, Some(AcpLaunchBlocker::NoAgentsAvailable));
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
                blocker: Some(AcpLaunchBlocker::CapabilityMismatch),
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

    // ---------------------------------------------------------------
    // Capability-driven ranking tests
    // ---------------------------------------------------------------

    fn make_ranked_entry(
        id: &str,
        source: AcpAgentSource,
        last_session_ok: bool,
    ) -> AcpAgentCatalogEntry {
        AcpAgentCatalogEntry {
            id: id.to_string().into(),
            display_name: id.to_string().into(),
            source,
            install_state: AcpAgentInstallState::Ready,
            auth_state: AcpAgentAuthState::Unknown,
            config_state: AcpAgentConfigState::Valid,
            install_hint: None,
            config_hint: None,
            supports_embedded_context: None,
            supports_image: None,
            last_session_ok,
            config: None,
        }
    }

    #[test]
    fn ranking_prefers_last_session_ok_over_load_order() {
        let agents = vec![
            make_ranked_entry("claude-code", AcpAgentSource::LegacyClaudeCode, false),
            make_ranked_entry("opencode", AcpAgentSource::ScriptKitCatalog, true),
        ];
        let result = resolve_default_acp_launch(&agents, None);
        assert_eq!(
            result.selected_agent_id(),
            Some("opencode"),
            "last_session_ok agent should win over load-order-first"
        );
    }

    #[test]
    fn ranking_prefers_non_legacy_when_equal() {
        let agents = vec![
            make_ranked_entry("claude-code", AcpAgentSource::LegacyClaudeCode, false),
            make_ranked_entry("gemini-cli", AcpAgentSource::ScriptKitCatalog, false),
        ];
        let result = resolve_default_acp_launch(&agents, None);
        assert_eq!(
            result.selected_agent_id(),
            Some("gemini-cli"),
            "non-legacy should rank ahead of legacy when otherwise equal"
        );
    }

    #[test]
    fn ranking_stable_alphabetical_tiebreaker() {
        let agents = vec![
            make_ranked_entry("zeta-agent", AcpAgentSource::ScriptKitCatalog, false),
            make_ranked_entry("alpha-agent", AcpAgentSource::ScriptKitCatalog, false),
        ];
        let result = resolve_default_acp_launch(&agents, None);
        assert_eq!(
            result.selected_agent_id(),
            Some("alpha-agent"),
            "alphabetical tie-breaker should pick alpha before zeta"
        );
    }

    #[test]
    fn ranking_last_session_ok_beats_non_legacy() {
        // Legacy agent that worked last session should beat non-legacy that didn't.
        let agents = vec![
            make_ranked_entry("opencode", AcpAgentSource::ScriptKitCatalog, false),
            make_ranked_entry("claude-code", AcpAgentSource::LegacyClaudeCode, true),
        ];
        let result = resolve_default_acp_launch(&agents, None);
        assert_eq!(
            result.selected_agent_id(),
            Some("claude-code"),
            "last_session_ok should outrank non-legacy preference"
        );
    }

    #[test]
    fn ranking_with_requirements_prefers_capable_non_legacy() {
        let mut opencode =
            make_ranked_entry("opencode", AcpAgentSource::ScriptKitCatalog, false);
        opencode.supports_image = Some(true);

        let mut claude =
            make_ranked_entry("claude-code", AcpAgentSource::LegacyClaudeCode, false);
        claude.supports_image = Some(true);

        let agents = vec![claude, opencode];
        let reqs = AcpLaunchRequirements {
            needs_embedded_context: false,
            needs_image: true,
        };
        let result = resolve_acp_launch_with_requirements(&agents, None, reqs);
        assert_eq!(
            result.selected_agent_id(),
            Some("opencode"),
            "non-legacy capable agent should rank ahead of legacy capable agent"
        );
        assert!(result.is_ready());
    }

    #[test]
    fn ranking_preferred_still_wins_when_valid() {
        let agents = vec![
            make_ranked_entry("opencode", AcpAgentSource::ScriptKitCatalog, true),
            make_ranked_entry("claude-code", AcpAgentSource::LegacyClaudeCode, false),
        ];
        let result = resolve_default_acp_launch(&agents, Some("claude-code"));
        assert_eq!(
            result.selected_agent_id(),
            Some("claude-code"),
            "explicit preferred agent should override ranking"
        );
    }
}
