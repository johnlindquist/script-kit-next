//! ACP agent catalog types.
//!
//! Represents the schema-versioned catalog of available ACP agents,
//! their install/auth/config states, and the file-backed catalog path.

use std::path::PathBuf;

use gpui::SharedString;
use serde::{Deserialize, Serialize};

use super::config::AcpAgentConfig;

/// Current schema version for the ACP agent catalog file.
pub(crate) const ACP_AGENT_CATALOG_SCHEMA_VERSION: u32 = 1;

/// File-backed ACP agent catalog.
///
/// Stored at `~/.scriptkit/acp/agents.json` and loaded at ACP launch time.
/// The legacy Claude Code path remains as one catalog source, not the default.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AcpAgentCatalogFile {
    pub schema_version: u32,
    #[serde(default)]
    pub agents: Vec<AcpAgentConfig>,
}

impl Default for AcpAgentCatalogFile {
    fn default() -> Self {
        Self {
            schema_version: ACP_AGENT_CATALOG_SCHEMA_VERSION,
            agents: Vec::new(),
        }
    }
}

/// Where this agent entry was discovered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum AcpAgentSource {
    /// Synthesized from the legacy `claudeCode` config block.
    LegacyClaudeCode,
    /// Loaded from `~/.scriptkit/acp/agents.json`.
    ScriptKitCatalog,
    /// Auto-detected built-in agent (e.g., `opencode` on PATH).
    BuiltIn,
}

/// Whether the agent binary is available on this machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum AcpAgentInstallState {
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
pub(crate) enum AcpAgentAuthState {
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
pub(crate) enum AcpAgentConfigState {
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
pub(crate) struct AcpAgentCatalogEntry {
    pub id: SharedString,
    pub display_name: SharedString,
    pub source: AcpAgentSource,
    pub install_state: AcpAgentInstallState,
    pub auth_state: AcpAgentAuthState,
    pub config_state: AcpAgentConfigState,
    pub install_hint: Option<SharedString>,
    pub config_hint: Option<SharedString>,
    pub config: Option<AcpAgentConfig>,
}

impl AcpAgentCatalogEntry {
    /// An entry is launchable when installed, authenticated enough to start,
    /// and validly configured.
    pub(crate) fn is_launchable(&self) -> bool {
        self.install_state == AcpAgentInstallState::Ready
            && self.auth_state != AcpAgentAuthState::NeedsAuthentication
            && self.config_state == AcpAgentConfigState::Valid
    }
}

/// Default path for the ACP agent catalog file.
pub(crate) fn default_acp_agents_path() -> PathBuf {
    crate::setup::get_kit_path().join("acp").join("agents.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_file_default_has_correct_schema_version() {
        let file = AcpAgentCatalogFile::default();
        assert_eq!(file.schema_version, ACP_AGENT_CATALOG_SCHEMA_VERSION);
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
                    "args": ["acp"]
                }
            ]
        }"#;
        let file: AcpAgentCatalogFile = serde_json::from_str(json).expect("should parse");
        assert_eq!(file.schema_version, 1);
        assert_eq!(file.agents.len(), 1);
        assert_eq!(file.agents[0].id, "opencode");
    }

    #[test]
    fn catalog_entry_launchable_when_ready_and_valid() {
        let entry = AcpAgentCatalogEntry {
            id: "test".into(),
            display_name: "Test".into(),
            source: AcpAgentSource::ScriptKitCatalog,
            install_state: AcpAgentInstallState::Ready,
            auth_state: AcpAgentAuthState::Unknown,
            config_state: AcpAgentConfigState::Valid,
            install_hint: None,
            config_hint: None,
            config: None,
        };
        assert!(entry.is_launchable());
    }

    #[test]
    fn catalog_entry_not_launchable_when_needs_install() {
        let entry = AcpAgentCatalogEntry {
            id: "test".into(),
            display_name: "Test".into(),
            source: AcpAgentSource::ScriptKitCatalog,
            install_state: AcpAgentInstallState::NeedsInstall,
            auth_state: AcpAgentAuthState::Unknown,
            config_state: AcpAgentConfigState::Valid,
            install_hint: Some("npm i -g test".into()),
            config_hint: None,
            config: None,
        };
        assert!(!entry.is_launchable());
    }

    #[test]
    fn catalog_entry_not_launchable_when_auth_required() {
        let entry = AcpAgentCatalogEntry {
            id: "test".into(),
            display_name: "Test".into(),
            source: AcpAgentSource::ScriptKitCatalog,
            install_state: AcpAgentInstallState::Ready,
            auth_state: AcpAgentAuthState::NeedsAuthentication,
            config_state: AcpAgentConfigState::Valid,
            install_hint: None,
            config_hint: None,
            config: None,
        };
        assert!(!entry.is_launchable());
    }

    #[test]
    fn catalog_entry_not_launchable_when_config_missing() {
        let entry = AcpAgentCatalogEntry {
            id: "test".into(),
            display_name: "Test".into(),
            source: AcpAgentSource::ScriptKitCatalog,
            install_state: AcpAgentInstallState::Ready,
            auth_state: AcpAgentAuthState::Unknown,
            config_state: AcpAgentConfigState::Missing,
            install_hint: None,
            config_hint: None,
            config: None,
        };
        assert!(!entry.is_launchable());
    }
}
