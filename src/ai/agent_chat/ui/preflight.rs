//! Agent Chat launch preflight resolution.
//!
//! Resolves a catalog of agents into a deterministic launch decision:
//! either a ready agent or a structured blocker state that the UI
//! can render inline instead of dead-ending with a toast.

use std::cmp::Ordering;

use gpui::SharedString;

use super::catalog::{
    AgentChatAgentAuthState, AgentChatAgentCatalogEntry, AgentChatAgentConfigState,
    AgentChatAgentInstallState,
};

/// Capability requirements derived from the current Agent Chat entry path.
///
/// Used by `resolve_agent_chat_launch_with_requirements()` to select an agent
/// that can actually satisfy the request instead of just the first
/// install/auth/config-valid one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct AgentChatLaunchRequirements {
    /// The entry path includes embedded context blocks (desktop snapshot, etc.)
    pub needs_embedded_context: bool,
    /// The entry path includes image/screenshot attachments.
    pub needs_image: bool,
}

impl AgentChatAgentCatalogEntry {
    /// Whether this agent can satisfy the given capability requirements.
    ///
    /// `None` (unknown) is treated as capable — optimistic default so that
    /// agents without persisted runtime facts are not excluded prematurely.
    pub(crate) fn satisfies_requirements(&self, requirements: AgentChatLaunchRequirements) -> bool {
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
    a: &AgentChatAgentCatalogEntry,
    b: &AgentChatAgentCatalogEntry,
) -> Ordering {
    let a_is_legacy = matches!(
        a.source,
        super::catalog::AgentChatAgentSource::LegacyClaudeCode
    );
    let b_is_legacy = matches!(
        b.source,
        super::catalog::AgentChatAgentSource::LegacyClaudeCode
    );

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
    agents: &[AgentChatAgentCatalogEntry],
    requirements: Option<AgentChatLaunchRequirements>,
) -> Option<AgentChatAgentCatalogEntry> {
    let mut candidates: Vec<&AgentChatAgentCatalogEntry> = agents
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
        event = "agent_chat_launchable_candidate_ranked",
        needs_embedded_context = requirements.map(|r| r.needs_embedded_context),
        needs_image = requirements.map(|r| r.needs_image),
        candidate_ids = ?candidates
            .iter()
            .map(|entry| entry.id.as_ref())
            .collect::<Vec<_>>(),
    );

    candidates.into_iter().next().cloned()
}

fn implicit_codex_default_candidate(
    agents: &[AgentChatAgentCatalogEntry],
    preferred_agent_id: Option<&str>,
    codex_launch_ready: bool,
) -> Option<AgentChatAgentCatalogEntry> {
    if preferred_agent_id.is_some() || !codex_launch_ready {
        return None;
    }
    agents
        .iter()
        .find(|entry| entry.id.as_ref() == super::config::CODEX_AGENT_CHAT_AGENT_ID)
        .cloned()
}

/// Why an Agent Chat launch cannot proceed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AgentChatLaunchBlocker {
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
pub(crate) struct AgentChatLaunchResolution {
    pub selected_agent: Option<AgentChatAgentCatalogEntry>,
    pub blocker: Option<AgentChatLaunchBlocker>,
    /// The full catalog used for this resolution, carried so setup UIs can
    /// render the same agent list without re-loading from disk.
    pub catalog_entries: Vec<AgentChatAgentCatalogEntry>,
}

impl AgentChatLaunchResolution {
    /// Ready to launch: has an agent and no blockers.
    pub(crate) fn is_ready(&self) -> bool {
        self.selected_agent.is_some() && self.blocker.is_none()
    }

    /// The selected agent's ID, if any.
    pub(crate) fn selected_agent_id(&self) -> Option<&str> {
        self.selected_agent.as_ref().map(|entry| entry.id.as_ref())
    }
}

fn blocker_for_selected_agent(
    selected_agent: &AgentChatAgentCatalogEntry,
    requirements: AgentChatLaunchRequirements,
) -> Option<AgentChatLaunchBlocker> {
    match (
        selected_agent.install_state,
        selected_agent.auth_state,
        selected_agent.config_state,
    ) {
        (AgentChatAgentInstallState::Unsupported, _, _) => {
            Some(AgentChatLaunchBlocker::UnsupportedAgent)
        }
        (AgentChatAgentInstallState::NeedsInstall, _, _) => {
            Some(AgentChatLaunchBlocker::AgentNotInstalled)
        }
        (_, AgentChatAgentAuthState::NeedsAuthentication, _) => {
            Some(AgentChatLaunchBlocker::AuthenticationRequired)
        }
        (_, _, AgentChatAgentConfigState::Missing | AgentChatAgentConfigState::Invalid) => {
            Some(AgentChatLaunchBlocker::AgentMisconfigured)
        }
        _ if !selected_agent.satisfies_requirements(requirements) => {
            Some(AgentChatLaunchBlocker::CapabilityMismatch)
        }
        _ => None,
    }
}

/// Resolve which agent to launch and whether anything blocks it.
///
/// Selection priority:
/// 1. Preferred agent ID (if provided and found in catalog)
/// 2. Best ranked launchable agent (last-known-good > non-legacy > alphabetical)
/// 3. First agent in catalog (will have a blocker)
/// 4. None (empty catalog → NoAgentsAvailable)
pub(crate) fn resolve_default_agent_chat_launch(
    agents: &[AgentChatAgentCatalogEntry],
    preferred_agent_id: Option<&str>,
) -> AgentChatLaunchResolution {
    let codex_probe = super::config::codex_agent_chat_default_probe_state();
    resolve_default_agent_chat_launch_with_codex_probe(
        agents,
        preferred_agent_id,
        codex_probe.should_be_implicit_codex_default,
    )
}

fn resolve_default_agent_chat_launch_with_codex_probe(
    agents: &[AgentChatAgentCatalogEntry],
    preferred_agent_id: Option<&str>,
    implicit_codex_default: bool,
) -> AgentChatLaunchResolution {
    let selected = preferred_agent_id
        .and_then(|preferred| agents.iter().find(|entry| entry.id.as_ref() == preferred))
        .cloned()
        .or_else(|| {
            implicit_codex_default_candidate(agents, preferred_agent_id, implicit_codex_default)
        })
        .or_else(|| best_launchable_candidate(agents, None))
        .or_else(|| agents.first().cloned());

    let Some(selected_agent) = selected else {
        return AgentChatLaunchResolution {
            selected_agent: None,
            blocker: Some(AgentChatLaunchBlocker::NoAgentsAvailable),
            catalog_entries: agents.to_vec(),
        };
    };

    let blocker = match (
        selected_agent.install_state,
        selected_agent.auth_state,
        selected_agent.config_state,
    ) {
        (AgentChatAgentInstallState::Unsupported, _, _) => {
            Some(AgentChatLaunchBlocker::UnsupportedAgent)
        }
        (AgentChatAgentInstallState::NeedsInstall, _, _) => {
            Some(AgentChatLaunchBlocker::AgentNotInstalled)
        }
        (_, AgentChatAgentAuthState::NeedsAuthentication, _) => {
            Some(AgentChatLaunchBlocker::AuthenticationRequired)
        }
        (_, _, AgentChatAgentConfigState::Missing | AgentChatAgentConfigState::Invalid) => {
            Some(AgentChatLaunchBlocker::AgentMisconfigured)
        }
        _ => None,
    };

    if selected_agent.id.as_ref() == super::config::CODEX_AGENT_CHAT_AGENT_ID
        && preferred_agent_id.is_none()
        && implicit_codex_default
    {
        if blocker.is_none() {
            tracing::info!(
                target: "script_kit::tab_ai",
                event = "agent_chat_codex_default_selected",
                selected_agent_id = selected_agent.id.as_ref(),
            );
        } else {
            tracing::info!(
                target: "script_kit::tab_ai",
                event = "agent_chat_codex_default_setup_blocked",
                selected_agent_id = selected_agent.id.as_ref(),
                blocker = ?blocker,
            );
        }
    }

    AgentChatLaunchResolution {
        selected_agent: Some(selected_agent),
        blocker,
        catalog_entries: agents.to_vec(),
    }
}

/// Resolve which agent to launch using capability requirements.
///
/// Selection priority:
/// 1. Preferred agent ID (if provided and found in catalog)
/// 2. Implicit Codex default when no preference exists
/// 3. Best ranked launchable agent that satisfies requirements
/// 4. Best ranked launchable agent (capability mismatch blocker)
/// 5. First agent in catalog (will have a blocker)
/// 6. None (empty catalog → NoAgentsAvailable)
pub(crate) fn resolve_agent_chat_launch_with_requirements(
    agents: &[AgentChatAgentCatalogEntry],
    preferred_agent_id: Option<&str>,
    requirements: AgentChatLaunchRequirements,
) -> AgentChatLaunchResolution {
    let codex_probe = super::config::codex_agent_chat_default_probe_state();
    resolve_agent_chat_launch_with_requirements_and_codex_probe(
        agents,
        preferred_agent_id,
        requirements,
        codex_probe.should_be_implicit_codex_default,
    )
}

fn resolve_agent_chat_launch_with_requirements_and_codex_probe(
    agents: &[AgentChatAgentCatalogEntry],
    preferred_agent_id: Option<&str>,
    requirements: AgentChatLaunchRequirements,
    implicit_codex_default: bool,
) -> AgentChatLaunchResolution {
    let preferred = preferred_agent_id
        .and_then(|preferred| agents.iter().find(|entry| entry.id.as_ref() == preferred));

    let selected = preferred
        .cloned()
        // Implicit Codex default: without an explicit preference, keep Codex
        // selected even when setup blocks it.
        .or_else(|| {
            implicit_codex_default_candidate(agents, preferred_agent_id, implicit_codex_default)
        })
        // Fallback: best ranked launchable agent that satisfies requirements.
        .or_else(|| best_launchable_candidate(agents, Some(requirements)))
        // Fallback: best ranked launchable agent (will get CapabilityMismatch blocker).
        .or_else(|| best_launchable_candidate(agents, None))
        // Last resort: first catalog entry.
        .or_else(|| agents.first().cloned());

    let Some(selected_agent) = selected else {
        return AgentChatLaunchResolution {
            selected_agent: None,
            blocker: Some(AgentChatLaunchBlocker::NoAgentsAvailable),
            catalog_entries: agents.to_vec(),
        };
    };

    let blocker = blocker_for_selected_agent(&selected_agent, requirements);
    let implicit_codex_selected = selected_agent.id.as_ref()
        == super::config::CODEX_AGENT_CHAT_AGENT_ID
        && preferred_agent_id.is_none()
        && implicit_codex_default;

    tracing::info!(
        target: "script_kit::tab_ai",
        event = "agent_chat_capability_resolution",
        preferred_agent_id = ?preferred_agent_id,
        needs_embedded_context = requirements.needs_embedded_context,
        needs_image = requirements.needs_image,
        selected_agent_id = selected_agent.id.as_ref(),
        supports_embedded_context = ?selected_agent.supports_embedded_context,
        supports_image = ?selected_agent.supports_image,
        implicit_codex_default,
        implicit_codex_selected,
        blocker = ?blocker,
    );
    if implicit_codex_selected {
        if blocker.is_none() {
            tracing::info!(
                target: "script_kit::tab_ai",
                event = "agent_chat_codex_default_selected",
                selected_agent_id = selected_agent.id.as_ref(),
                needs_embedded_context = requirements.needs_embedded_context,
                needs_image = requirements.needs_image,
            );
        } else {
            tracing::info!(
                target: "script_kit::tab_ai",
                event = "agent_chat_codex_default_setup_blocked",
                selected_agent_id = selected_agent.id.as_ref(),
                blocker = ?blocker,
                needs_embedded_context = requirements.needs_embedded_context,
                needs_image = requirements.needs_image,
            );
        }
    }

    AgentChatLaunchResolution {
        selected_agent: Some(selected_agent),
        blocker,
        catalog_entries: agents.to_vec(),
    }
}

/// Resolve an explicit user-selected agent without silently falling back.
///
/// Agent switches and setup retries are direct user choices. If the selected
/// agent is not installed, authenticated, configured, or capable enough, the
/// caller should render that setup blocker instead of launching a different
/// ready agent and making the selector appear to reset.
pub(crate) fn resolve_explicit_agent_chat_launch_with_requirements(
    agents: &[AgentChatAgentCatalogEntry],
    preferred_agent_id: Option<&str>,
    requirements: AgentChatLaunchRequirements,
) -> AgentChatLaunchResolution {
    let selected = preferred_agent_id
        .and_then(|preferred| agents.iter().find(|entry| entry.id.as_ref() == preferred))
        .cloned();

    let Some(selected_agent) = selected else {
        return resolve_agent_chat_launch_with_requirements(
            agents,
            preferred_agent_id,
            requirements,
        );
    };

    let blocker = blocker_for_selected_agent(&selected_agent, requirements);

    tracing::info!(
        target: "script_kit::tab_ai",
        event = "agent_chat_explicit_resolution",
        preferred_agent_id = ?preferred_agent_id,
        needs_embedded_context = requirements.needs_embedded_context,
        needs_image = requirements.needs_image,
        selected_agent_id = selected_agent.id.as_ref(),
        supports_embedded_context = ?selected_agent.supports_embedded_context,
        supports_image = ?selected_agent.supports_image,
        blocker = ?blocker,
    );

    AgentChatLaunchResolution {
        selected_agent: Some(selected_agent),
        blocker,
        catalog_entries: agents.to_vec(),
    }
}

/// Human-readable title for the resolution state.
pub(crate) fn setup_title_for_resolution(resolution: &AgentChatLaunchResolution) -> SharedString {
    match resolution.blocker {
        Some(AgentChatLaunchBlocker::NoAgentsAvailable) => "No agents available".into(),
        Some(AgentChatLaunchBlocker::AgentNotInstalled) => "Agent install required".into(),
        Some(AgentChatLaunchBlocker::AuthenticationRequired) => {
            "Agent authentication required".into()
        }
        Some(AgentChatLaunchBlocker::AgentMisconfigured) => "Agent configuration required".into(),
        Some(AgentChatLaunchBlocker::UnsupportedAgent) => "Agent unsupported".into(),
        Some(AgentChatLaunchBlocker::CapabilityMismatch) => "Agent capability mismatch".into(),
        None => "Agent ready".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::agent_chat::ui::catalog::{AgentChatAgentAuthState, AgentChatAgentSource};

    fn make_entry(
        id: &str,
        install: AgentChatAgentInstallState,
        auth: AgentChatAgentAuthState,
        config: AgentChatAgentConfigState,
    ) -> AgentChatAgentCatalogEntry {
        AgentChatAgentCatalogEntry {
            id: id.to_string().into(),
            display_name: id.to_string().into(),
            source: AgentChatAgentSource::ScriptKitCatalog,
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
        install: AgentChatAgentInstallState,
        auth: AgentChatAgentAuthState,
        config: AgentChatAgentConfigState,
        supports_embedded_context: Option<bool>,
        supports_image: Option<bool>,
    ) -> AgentChatAgentCatalogEntry {
        AgentChatAgentCatalogEntry {
            id: id.to_string().into(),
            display_name: id.to_string().into(),
            source: AgentChatAgentSource::ScriptKitCatalog,
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
        let result = resolve_default_agent_chat_launch(&[], None);
        assert_eq!(
            result.blocker,
            Some(AgentChatLaunchBlocker::NoAgentsAvailable)
        );
        assert!(!result.is_ready());
        assert!(result.selected_agent.is_none());
    }

    #[test]
    fn ready_agent_returns_no_blocker() {
        let agents = vec![make_entry(
            "ready",
            AgentChatAgentInstallState::Ready,
            AgentChatAgentAuthState::Unknown,
            AgentChatAgentConfigState::Valid,
        )];
        let result = resolve_default_agent_chat_launch(&agents, None);
        assert!(result.is_ready());
        assert_eq!(result.selected_agent_id(), Some("ready"));
        assert!(result.blocker.is_none());
    }

    #[test]
    fn prefers_ready_agent_over_blocked() {
        let agents = vec![
            make_entry(
                "blocked",
                AgentChatAgentInstallState::NeedsInstall,
                AgentChatAgentAuthState::Unknown,
                AgentChatAgentConfigState::Valid,
            ),
            make_entry(
                "ready",
                AgentChatAgentInstallState::Ready,
                AgentChatAgentAuthState::Unknown,
                AgentChatAgentConfigState::Valid,
            ),
        ];
        let result = resolve_default_agent_chat_launch(&agents, None);
        assert_eq!(result.selected_agent_id(), Some("ready"));
        assert!(result.is_ready());
    }

    #[test]
    fn preferred_agent_id_overrides_auto_selection() {
        let agents = vec![
            make_entry(
                "auto-ready",
                AgentChatAgentInstallState::Ready,
                AgentChatAgentAuthState::Unknown,
                AgentChatAgentConfigState::Valid,
            ),
            make_entry(
                "preferred-blocked",
                AgentChatAgentInstallState::NeedsInstall,
                AgentChatAgentAuthState::Unknown,
                AgentChatAgentConfigState::Valid,
            ),
        ];
        let result = resolve_default_agent_chat_launch(&agents, Some("preferred-blocked"));
        assert_eq!(result.selected_agent_id(), Some("preferred-blocked"));
        assert_eq!(
            result.blocker,
            Some(AgentChatLaunchBlocker::AgentNotInstalled)
        );
    }

    #[test]
    fn needs_install_blocker() {
        let agents = vec![make_entry(
            "missing",
            AgentChatAgentInstallState::NeedsInstall,
            AgentChatAgentAuthState::Unknown,
            AgentChatAgentConfigState::Valid,
        )];
        let result = resolve_default_agent_chat_launch(&agents, None);
        assert_eq!(
            result.blocker,
            Some(AgentChatLaunchBlocker::AgentNotInstalled)
        );
    }

    #[test]
    fn unsupported_blocker() {
        let agents = vec![make_entry(
            "unsupported",
            AgentChatAgentInstallState::Unsupported,
            AgentChatAgentAuthState::Unknown,
            AgentChatAgentConfigState::Valid,
        )];
        let result = resolve_default_agent_chat_launch(&agents, None);
        assert_eq!(
            result.blocker,
            Some(AgentChatLaunchBlocker::UnsupportedAgent)
        );
    }

    #[test]
    fn auth_required_blocker() {
        let agents = vec![make_entry(
            "needs-auth",
            AgentChatAgentInstallState::Ready,
            AgentChatAgentAuthState::NeedsAuthentication,
            AgentChatAgentConfigState::Valid,
        )];
        let result = resolve_default_agent_chat_launch(&agents, None);
        assert_eq!(
            result.blocker,
            Some(AgentChatLaunchBlocker::AuthenticationRequired)
        );
    }

    #[test]
    fn misconfigured_blocker() {
        let agents = vec![make_entry(
            "bad-config",
            AgentChatAgentInstallState::Ready,
            AgentChatAgentAuthState::Unknown,
            AgentChatAgentConfigState::Invalid,
        )];
        let result = resolve_default_agent_chat_launch(&agents, None);
        assert_eq!(
            result.blocker,
            Some(AgentChatLaunchBlocker::AgentMisconfigured)
        );
    }

    #[test]
    fn auth_required_agent_is_skipped_for_auto_selection() {
        let agents = vec![
            make_entry(
                "needs-auth",
                AgentChatAgentInstallState::Ready,
                AgentChatAgentAuthState::NeedsAuthentication,
                AgentChatAgentConfigState::Valid,
            ),
            make_entry(
                "ready",
                AgentChatAgentInstallState::Ready,
                AgentChatAgentAuthState::Authenticated,
                AgentChatAgentConfigState::Valid,
            ),
        ];
        let result = resolve_default_agent_chat_launch(&agents, None);
        assert_eq!(result.selected_agent_id(), Some("ready"));
        assert!(result.blocker.is_none());
    }

    #[test]
    fn preferred_auth_required_agent_gets_blocker() {
        let agents = vec![
            make_entry(
                "needs-auth",
                AgentChatAgentInstallState::Ready,
                AgentChatAgentAuthState::NeedsAuthentication,
                AgentChatAgentConfigState::Valid,
            ),
            make_entry(
                "ready",
                AgentChatAgentInstallState::Ready,
                AgentChatAgentAuthState::Authenticated,
                AgentChatAgentConfigState::Valid,
            ),
        ];
        let result = resolve_default_agent_chat_launch(&agents, Some("needs-auth"));
        assert_eq!(result.selected_agent_id(), Some("needs-auth"));
        assert_eq!(
            result.blocker,
            Some(AgentChatLaunchBlocker::AuthenticationRequired)
        );
    }

    // ---------------------------------------------------------------
    // Capability-aware resolution tests
    // ---------------------------------------------------------------

    #[test]
    fn capability_requirements_default_is_no_requirements() {
        let reqs = AgentChatLaunchRequirements::default();
        assert!(!reqs.needs_embedded_context);
        assert!(!reqs.needs_image);
    }

    #[test]
    fn satisfies_requirements_unknown_treated_as_capable() {
        let entry = make_entry(
            "unknown-caps",
            AgentChatAgentInstallState::Ready,
            AgentChatAgentAuthState::Unknown,
            AgentChatAgentConfigState::Valid,
        );
        let reqs = AgentChatLaunchRequirements {
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
            AgentChatAgentInstallState::Ready,
            AgentChatAgentAuthState::Unknown,
            AgentChatAgentConfigState::Valid,
            Some(false),
            Some(true),
        );
        let reqs = AgentChatLaunchRequirements {
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
            AgentChatAgentInstallState::Ready,
            AgentChatAgentAuthState::Unknown,
            AgentChatAgentConfigState::Valid,
            Some(false),
            Some(false),
        );
        let reqs = AgentChatLaunchRequirements::default();
        assert!(
            entry.satisfies_requirements(reqs),
            "no requirements should always pass"
        );
    }

    #[test]
    fn capability_aware_keeps_preferred_with_capability_blocker() {
        let agents = vec![
            make_entry_with_capabilities(
                "claude-code",
                AgentChatAgentInstallState::Ready,
                AgentChatAgentAuthState::Unknown,
                AgentChatAgentConfigState::Valid,
                Some(false), // cannot do embedded context
                Some(true),
            ),
            make_entry_with_capabilities(
                "opencode",
                AgentChatAgentInstallState::Ready,
                AgentChatAgentAuthState::Unknown,
                AgentChatAgentConfigState::Valid,
                Some(true), // can do embedded context
                Some(true),
            ),
        ];
        let reqs = AgentChatLaunchRequirements {
            needs_embedded_context: true,
            needs_image: false,
        };
        let result =
            resolve_agent_chat_launch_with_requirements(&agents, Some("claude-code"), reqs);
        assert_eq!(
            result.selected_agent_id(),
            Some("claude-code"),
            "explicit preferred agent should win even when setup must explain the blocker"
        );
        assert_eq!(
            result.blocker,
            Some(AgentChatLaunchBlocker::CapabilityMismatch)
        );
        assert!(!result.is_ready());
    }

    #[test]
    fn capability_aware_uses_preferred_when_capable() {
        let agents = vec![
            make_entry_with_capabilities(
                "claude-code",
                AgentChatAgentInstallState::Ready,
                AgentChatAgentAuthState::Unknown,
                AgentChatAgentConfigState::Valid,
                Some(true),
                Some(true),
            ),
            make_entry_with_capabilities(
                "opencode",
                AgentChatAgentInstallState::Ready,
                AgentChatAgentAuthState::Unknown,
                AgentChatAgentConfigState::Valid,
                Some(true),
                Some(true),
            ),
        ];
        let reqs = AgentChatLaunchRequirements {
            needs_embedded_context: true,
            needs_image: false,
        };
        let result =
            resolve_agent_chat_launch_with_requirements(&agents, Some("claude-code"), reqs);
        assert_eq!(result.selected_agent_id(), Some("claude-code"));
        assert!(result.is_ready());
    }

    #[test]
    fn capability_aware_returns_mismatch_when_none_capable() {
        let agents = vec![
            make_entry_with_capabilities(
                "agent-a",
                AgentChatAgentInstallState::Ready,
                AgentChatAgentAuthState::Unknown,
                AgentChatAgentConfigState::Valid,
                Some(false),
                Some(false),
            ),
            make_entry_with_capabilities(
                "agent-b",
                AgentChatAgentInstallState::Ready,
                AgentChatAgentAuthState::Unknown,
                AgentChatAgentConfigState::Valid,
                Some(false),
                Some(false),
            ),
        ];
        let reqs = AgentChatLaunchRequirements {
            needs_embedded_context: true,
            needs_image: false,
        };
        let result = resolve_agent_chat_launch_with_requirements(&agents, None, reqs);
        assert_eq!(
            result.blocker,
            Some(AgentChatLaunchBlocker::CapabilityMismatch),
            "should return CapabilityMismatch when no agent satisfies requirements"
        );
    }

    #[test]
    fn capability_aware_no_requirements_uses_preferred() {
        let agents = vec![
            make_entry_with_capabilities(
                "agent-a",
                AgentChatAgentInstallState::Ready,
                AgentChatAgentAuthState::Unknown,
                AgentChatAgentConfigState::Valid,
                Some(false),
                Some(false),
            ),
            make_entry_with_capabilities(
                "agent-b",
                AgentChatAgentInstallState::Ready,
                AgentChatAgentAuthState::Unknown,
                AgentChatAgentConfigState::Valid,
                Some(true),
                Some(true),
            ),
        ];
        let reqs = AgentChatLaunchRequirements::default();
        let result = resolve_agent_chat_launch_with_requirements(&agents, Some("agent-a"), reqs);
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
            AgentChatAgentInstallState::NeedsInstall,
            AgentChatAgentAuthState::Unknown,
            AgentChatAgentConfigState::Valid,
            Some(true),
            Some(true),
        )];
        let reqs = AgentChatLaunchRequirements {
            needs_embedded_context: true,
            needs_image: false,
        };
        let result = resolve_agent_chat_launch_with_requirements(&agents, None, reqs);
        assert_eq!(
            result.blocker,
            Some(AgentChatLaunchBlocker::AgentNotInstalled),
            "install blocker should take precedence over capability"
        );
    }

    #[test]
    fn default_resolution_prefers_ready_codex_when_no_explicit_preference() {
        let agents = vec![
            make_entry(
                "opencode",
                AgentChatAgentInstallState::Ready,
                AgentChatAgentAuthState::Authenticated,
                AgentChatAgentConfigState::Valid,
            ),
            make_entry(
                "codex-agent_chat",
                AgentChatAgentInstallState::Ready,
                AgentChatAgentAuthState::Unknown,
                AgentChatAgentConfigState::Valid,
            ),
        ];

        let result = resolve_agent_chat_launch_with_requirements_and_codex_probe(
            &agents,
            None,
            AgentChatLaunchRequirements::default(),
            true,
        );

        assert_eq!(result.selected_agent_id(), Some("codex-agent_chat"));
        assert!(result.is_ready());
    }

    #[test]
    fn default_resolution_keeps_codex_auth_blocker_when_codex_installed() {
        let agents = vec![
            make_entry(
                "opencode",
                AgentChatAgentInstallState::Ready,
                AgentChatAgentAuthState::Authenticated,
                AgentChatAgentConfigState::Valid,
            ),
            make_entry(
                "codex-agent_chat",
                AgentChatAgentInstallState::Ready,
                AgentChatAgentAuthState::NeedsAuthentication,
                AgentChatAgentConfigState::Valid,
            ),
        ];

        let result = resolve_agent_chat_launch_with_requirements_and_codex_probe(
            &agents,
            None,
            AgentChatLaunchRequirements::default(),
            true,
        );

        assert_eq!(result.selected_agent_id(), Some("codex-agent_chat"));
        assert_eq!(
            result.blocker,
            Some(AgentChatLaunchBlocker::AuthenticationRequired)
        );
        assert!(!result.is_ready());
    }

    #[test]
    fn implicit_codex_default_does_not_select_missing_codex_adapter() {
        let agents = vec![
            make_entry(
                "opencode",
                AgentChatAgentInstallState::Ready,
                AgentChatAgentAuthState::Authenticated,
                AgentChatAgentConfigState::Valid,
            ),
            make_entry(
                "codex-agent_chat",
                AgentChatAgentInstallState::Unsupported,
                AgentChatAgentAuthState::Authenticated,
                AgentChatAgentConfigState::Valid,
            ),
        ];

        let result = resolve_agent_chat_launch_with_requirements_and_codex_probe(
            &agents,
            None,
            AgentChatLaunchRequirements::default(),
            false,
        );

        assert_eq!(result.selected_agent_id(), Some("opencode"));
        assert!(result.is_ready());
    }

    #[test]
    fn explicit_preference_overrides_installed_codex_default() {
        let agents = vec![
            make_entry(
                "opencode",
                AgentChatAgentInstallState::Ready,
                AgentChatAgentAuthState::Authenticated,
                AgentChatAgentConfigState::Valid,
            ),
            make_entry(
                "codex-agent_chat",
                AgentChatAgentInstallState::Ready,
                AgentChatAgentAuthState::Unknown,
                AgentChatAgentConfigState::Valid,
            ),
        ];

        let result = resolve_agent_chat_launch_with_requirements_and_codex_probe(
            &agents,
            Some("opencode"),
            AgentChatLaunchRequirements::default(),
            true,
        );

        assert_eq!(result.selected_agent_id(), Some("opencode"));
        assert!(result.is_ready());
    }

    #[test]
    fn default_resolution_prefers_codex_even_when_probe_is_not_ready() {
        let agents = vec![
            make_entry(
                "opencode",
                AgentChatAgentInstallState::Ready,
                AgentChatAgentAuthState::Authenticated,
                AgentChatAgentConfigState::Valid,
            ),
            make_entry(
                "codex-agent_chat",
                AgentChatAgentInstallState::Ready,
                AgentChatAgentAuthState::Unknown,
                AgentChatAgentConfigState::Valid,
            ),
        ];

        let result = resolve_agent_chat_launch_with_requirements_and_codex_probe(
            &agents,
            None,
            AgentChatLaunchRequirements::default(),
            true,
        );

        assert_eq!(result.selected_agent_id(), Some("codex-agent_chat"));
    }

    #[test]
    fn explicit_codex_preference_keeps_codex_when_adapter_missing_without_install_prompt() {
        let agents = vec![
            make_entry(
                "opencode",
                AgentChatAgentInstallState::Ready,
                AgentChatAgentAuthState::Authenticated,
                AgentChatAgentConfigState::Valid,
            ),
            make_entry(
                "codex-agent_chat",
                AgentChatAgentInstallState::Unsupported,
                AgentChatAgentAuthState::Authenticated,
                AgentChatAgentConfigState::Valid,
            ),
        ];

        let result = resolve_agent_chat_launch_with_requirements_and_codex_probe(
            &agents,
            Some("codex-agent_chat"),
            AgentChatLaunchRequirements::default(),
            false,
        );

        assert_eq!(result.selected_agent_id(), Some("codex-agent_chat"));
        assert_eq!(
            result.blocker,
            Some(AgentChatLaunchBlocker::UnsupportedAgent)
        );
    }

    #[test]
    fn explicit_resolution_keeps_uninstalled_preferred_agent() {
        let agents = vec![
            make_entry_with_capabilities(
                "opencode",
                AgentChatAgentInstallState::Ready,
                AgentChatAgentAuthState::Unknown,
                AgentChatAgentConfigState::Valid,
                Some(true),
                Some(true),
            ),
            make_entry_with_capabilities(
                "codex-agent_chat",
                AgentChatAgentInstallState::NeedsInstall,
                AgentChatAgentAuthState::Unknown,
                AgentChatAgentConfigState::Valid,
                Some(true),
                Some(true),
            ),
        ];
        let result = resolve_explicit_agent_chat_launch_with_requirements(
            &agents,
            Some("codex-agent_chat"),
            AgentChatLaunchRequirements::default(),
        );
        assert_eq!(result.selected_agent_id(), Some("codex-agent_chat"));
        assert_eq!(
            result.blocker,
            Some(AgentChatLaunchBlocker::AgentNotInstalled)
        );
        assert!(!result.is_ready());
    }

    #[test]
    fn explicit_resolution_keeps_capability_mismatched_preferred_agent() {
        let agents = vec![
            make_entry_with_capabilities(
                "opencode",
                AgentChatAgentInstallState::Ready,
                AgentChatAgentAuthState::Unknown,
                AgentChatAgentConfigState::Valid,
                Some(true),
                Some(true),
            ),
            make_entry_with_capabilities(
                "codex-agent_chat",
                AgentChatAgentInstallState::Ready,
                AgentChatAgentAuthState::Unknown,
                AgentChatAgentConfigState::Valid,
                Some(false),
                Some(false),
            ),
        ];
        let result = resolve_explicit_agent_chat_launch_with_requirements(
            &agents,
            Some("codex-agent_chat"),
            AgentChatLaunchRequirements {
                needs_embedded_context: true,
                needs_image: false,
            },
        );
        assert_eq!(result.selected_agent_id(), Some("codex-agent_chat"));
        assert_eq!(
            result.blocker,
            Some(AgentChatLaunchBlocker::CapabilityMismatch)
        );
        assert!(!result.is_ready());
    }

    #[test]
    fn capability_aware_empty_catalog() {
        let reqs = AgentChatLaunchRequirements {
            needs_embedded_context: true,
            needs_image: true,
        };
        let result = resolve_agent_chat_launch_with_requirements(&[], None, reqs);
        assert_eq!(
            result.blocker,
            Some(AgentChatLaunchBlocker::NoAgentsAvailable)
        );
    }

    #[test]
    fn setup_title_covers_all_blockers() {
        let titles = vec![
            setup_title_for_resolution(&AgentChatLaunchResolution {
                selected_agent: None,
                blocker: Some(AgentChatLaunchBlocker::NoAgentsAvailable),
                catalog_entries: vec![],
            }),
            setup_title_for_resolution(&AgentChatLaunchResolution {
                selected_agent: None,
                blocker: Some(AgentChatLaunchBlocker::AgentNotInstalled),
                catalog_entries: vec![],
            }),
            setup_title_for_resolution(&AgentChatLaunchResolution {
                selected_agent: None,
                blocker: Some(AgentChatLaunchBlocker::AuthenticationRequired),
                catalog_entries: vec![],
            }),
            setup_title_for_resolution(&AgentChatLaunchResolution {
                selected_agent: None,
                blocker: Some(AgentChatLaunchBlocker::AgentMisconfigured),
                catalog_entries: vec![],
            }),
            setup_title_for_resolution(&AgentChatLaunchResolution {
                selected_agent: None,
                blocker: Some(AgentChatLaunchBlocker::UnsupportedAgent),
                catalog_entries: vec![],
            }),
            setup_title_for_resolution(&AgentChatLaunchResolution {
                selected_agent: None,
                blocker: Some(AgentChatLaunchBlocker::CapabilityMismatch),
                catalog_entries: vec![],
            }),
            setup_title_for_resolution(&AgentChatLaunchResolution {
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
        source: AgentChatAgentSource,
        last_session_ok: bool,
    ) -> AgentChatAgentCatalogEntry {
        AgentChatAgentCatalogEntry {
            id: id.to_string().into(),
            display_name: id.to_string().into(),
            source,
            install_state: AgentChatAgentInstallState::Ready,
            auth_state: AgentChatAgentAuthState::Unknown,
            config_state: AgentChatAgentConfigState::Valid,
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
            make_ranked_entry("claude-code", AgentChatAgentSource::LegacyClaudeCode, false),
            make_ranked_entry("opencode", AgentChatAgentSource::ScriptKitCatalog, true),
        ];
        let result = resolve_default_agent_chat_launch(&agents, None);
        assert_eq!(
            result.selected_agent_id(),
            Some("opencode"),
            "last_session_ok agent should win over load-order-first"
        );
    }

    #[test]
    fn ranking_prefers_non_legacy_when_equal() {
        let agents = vec![
            make_ranked_entry("claude-code", AgentChatAgentSource::LegacyClaudeCode, false),
            make_ranked_entry("test-agent", AgentChatAgentSource::ScriptKitCatalog, false),
        ];
        let result = resolve_default_agent_chat_launch(&agents, None);
        assert_eq!(
            result.selected_agent_id(),
            Some("test-agent"),
            "non-legacy should rank ahead of legacy when otherwise equal"
        );
    }

    #[test]
    fn ranking_stable_alphabetical_tiebreaker() {
        let agents = vec![
            make_ranked_entry("zeta-agent", AgentChatAgentSource::ScriptKitCatalog, false),
            make_ranked_entry("alpha-agent", AgentChatAgentSource::ScriptKitCatalog, false),
        ];
        let result = resolve_default_agent_chat_launch(&agents, None);
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
            make_ranked_entry("opencode", AgentChatAgentSource::ScriptKitCatalog, false),
            make_ranked_entry("claude-code", AgentChatAgentSource::LegacyClaudeCode, true),
        ];
        let result = resolve_default_agent_chat_launch(&agents, None);
        assert_eq!(
            result.selected_agent_id(),
            Some("claude-code"),
            "last_session_ok should outrank non-legacy preference"
        );
    }

    #[test]
    fn ranking_with_requirements_prefers_capable_non_legacy() {
        let mut opencode =
            make_ranked_entry("opencode", AgentChatAgentSource::ScriptKitCatalog, false);
        opencode.supports_image = Some(true);

        let mut claude =
            make_ranked_entry("claude-code", AgentChatAgentSource::LegacyClaudeCode, false);
        claude.supports_image = Some(true);

        let agents = vec![claude, opencode];
        let reqs = AgentChatLaunchRequirements {
            needs_embedded_context: false,
            needs_image: true,
        };
        let result = resolve_agent_chat_launch_with_requirements(&agents, None, reqs);
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
            make_ranked_entry("opencode", AgentChatAgentSource::ScriptKitCatalog, true),
            make_ranked_entry("claude-code", AgentChatAgentSource::LegacyClaudeCode, false),
        ];
        let result = resolve_default_agent_chat_launch(&agents, Some("claude-code"));
        assert_eq!(
            result.selected_agent_id(),
            Some("claude-code"),
            "explicit preferred agent should override ranking"
        );
    }
}
