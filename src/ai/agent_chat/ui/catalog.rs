//! Agent Chat agent catalog types.
//!
//! Represents the schema-versioned catalog of available Agent Chat agents,
//! their install/auth/config states, and the file-backed catalog path.

use std::path::PathBuf;

use gpui::SharedString;
use serde::{Deserialize, Serialize};

use super::config::AgentChatAgentConfig;

/// Current schema version for the Agent Chat agent catalog file.
pub(crate) const AGENT_CHAT_AGENT_CATALOG_SCHEMA_VERSION: u32 = 1;

/// File-backed Agent Chat agent catalog.
///
/// Stored at `~/.scriptkit/agent_chat/agents.json` and loaded at Agent Chat launch time.
/// The legacy Claude Code path remains as one catalog source, not the default.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AgentChatAgentCatalogFile {
    pub schema_version: u32,
    #[serde(default)]
    pub agents: Vec<AgentChatAgentConfig>,
}

impl Default for AgentChatAgentCatalogFile {
    fn default() -> Self {
        Self {
            schema_version: AGENT_CHAT_AGENT_CATALOG_SCHEMA_VERSION,
            agents: Vec::new(),
        }
    }
}

/// Where this agent entry was discovered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum AgentChatAgentSource {
    /// Synthesized from the legacy `claudeCode` config block.
    LegacyClaudeCode,
    /// Loaded from `~/.scriptkit/agent_chat/agents.json`.
    ScriptKitCatalog,
    /// Auto-detected built-in agent (e.g., `opencode` on PATH).
    BuiltIn,
}

/// Whether the agent binary is available on this machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum AgentChatAgentInstallState {
    /// Command resolved successfully.
    Ready,
    /// Binary not found but install spec is available.
    NeedsInstall,
    /// Binary not found and no install spec — cannot proceed.
    Unsupported,
}

/// Whether the agent has been authenticated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum AgentChatAgentAuthState {
    /// Auth state not yet determined (pre-initialize).
    Unknown,
    /// Agent confirmed authenticated.
    Authenticated,
    /// Agent requires authentication before use.
    NeedsAuthentication,
}

/// Whether the agent config is valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum AgentChatAgentConfigState {
    /// Config has a non-empty command and parses correctly.
    Valid,
    /// Required config fields are missing.
    Missing,
    /// Config present but malformed.
    Invalid,
}

/// A resolved catalog entry with install/auth/config readiness.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AgentChatAgentCatalogEntry {
    pub id: SharedString,
    pub display_name: SharedString,
    pub source: AgentChatAgentSource,
    pub install_state: AgentChatAgentInstallState,
    pub auth_state: AgentChatAgentAuthState,
    pub config_state: AgentChatAgentConfigState,
    pub install_hint: Option<SharedString>,
    pub config_hint: Option<SharedString>,
    /// Whether this agent supports embedded context blocks (e.g. desktop snapshots).
    /// `None` means unknown — treated as capable for fallback selection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supports_embedded_context: Option<bool>,
    /// Whether this agent supports image/screenshot attachments.
    /// `None` means unknown — treated as capable for fallback selection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supports_image: Option<bool>,
    /// Whether the last Agent Chat session with this agent completed successfully.
    #[serde(default)]
    pub last_session_ok: bool,
    pub config: Option<AgentChatAgentConfig>,
}

impl AgentChatAgentCatalogEntry {
    /// An entry is launchable when installed, authenticated enough to start,
    /// and validly configured.
    pub(crate) fn is_launchable(&self) -> bool {
        self.install_state == AgentChatAgentInstallState::Ready
            && self.auth_state != AgentChatAgentAuthState::NeedsAuthentication
            && self.config_state == AgentChatAgentConfigState::Valid
    }
}

/// Default path for the Agent Chat agent catalog file.
pub(crate) fn default_agent_chat_agents_path() -> PathBuf {
    crate::setup::get_kit_path()
        .join("agent_chat")
        .join("agents.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_file_default_has_correct_schema_version() {
        let file = AgentChatAgentCatalogFile::default();
        assert_eq!(file.schema_version, AGENT_CHAT_AGENT_CATALOG_SCHEMA_VERSION);
        assert!(file.agents.is_empty());
    }

    #[test]
    fn catalog_file_round_trip() {
        let json = r#"{
            "schemaVersion": 1,
            "agents": [
                {
                    "id": "opencode",
                    "displayName": "OpenCode",
                    "command": "opencode",
                    "args": ["agent_chat"]
                }
            ]
        }"#;
        let file: AgentChatAgentCatalogFile = serde_json::from_str(json).expect("should parse");
        assert_eq!(file.schema_version, 1);
        assert_eq!(file.agents.len(), 1);
        assert_eq!(file.agents[0].id, "opencode");
    }

    #[test]
    fn catalog_entry_launchable_when_ready_and_valid() {
        let entry = AgentChatAgentCatalogEntry {
            id: "test".into(),
            display_name: "Test".into(),
            source: AgentChatAgentSource::ScriptKitCatalog,
            install_state: AgentChatAgentInstallState::Ready,
            auth_state: AgentChatAgentAuthState::Unknown,
            config_state: AgentChatAgentConfigState::Valid,
            install_hint: None,
            config_hint: None,
            supports_embedded_context: None,
            supports_image: None,
            last_session_ok: false,
            config: None,
        };
        assert!(entry.is_launchable());
    }

    #[test]
    fn catalog_entry_not_launchable_when_needs_install() {
        let entry = AgentChatAgentCatalogEntry {
            id: "test".into(),
            display_name: "Test".into(),
            source: AgentChatAgentSource::ScriptKitCatalog,
            install_state: AgentChatAgentInstallState::NeedsInstall,
            auth_state: AgentChatAgentAuthState::Unknown,
            config_state: AgentChatAgentConfigState::Valid,
            install_hint: Some("npm i -g test".into()),
            config_hint: None,
            supports_embedded_context: None,
            supports_image: None,
            last_session_ok: false,
            config: None,
        };
        assert!(!entry.is_launchable());
    }

    #[test]
    fn catalog_entry_not_launchable_when_auth_required() {
        let entry = AgentChatAgentCatalogEntry {
            id: "test".into(),
            display_name: "Test".into(),
            source: AgentChatAgentSource::ScriptKitCatalog,
            install_state: AgentChatAgentInstallState::Ready,
            auth_state: AgentChatAgentAuthState::NeedsAuthentication,
            config_state: AgentChatAgentConfigState::Valid,
            install_hint: None,
            config_hint: None,
            supports_embedded_context: None,
            supports_image: None,
            last_session_ok: false,
            config: None,
        };
        assert!(!entry.is_launchable());
    }

    #[test]
    fn catalog_entry_not_launchable_when_config_missing() {
        let entry = AgentChatAgentCatalogEntry {
            id: "test".into(),
            display_name: "Test".into(),
            source: AgentChatAgentSource::ScriptKitCatalog,
            install_state: AgentChatAgentInstallState::Ready,
            auth_state: AgentChatAgentAuthState::Unknown,
            config_state: AgentChatAgentConfigState::Missing,
            install_hint: None,
            config_hint: None,
            supports_embedded_context: None,
            supports_image: None,
            last_session_ok: false,
            config: None,
        };
        assert!(!entry.is_launchable());
    }
}
