use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

use anyhow::Context as _;
use serde::{Deserialize, Serialize};

use crate::ai::ModelInfo;

/// Cached agent config — avoids spawning bun processes on every Tab press.
static CACHED_AGENT_CONFIG: OnceLock<AcpAgentConfig> = OnceLock::new();

/// Configuration for a generic ACP-compatible AI agent.
///
/// Supports both direct ACP agents (Gemini CLI, OpenCode) and
/// adapter-wrapped agents (Claude Code via claude-acp, Codex via codex-acp).
/// The `command` + `args` fields let users point at whatever ACP binary
/// is actually installed — no agent name is hardcoded.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcpAgentConfig {
    /// Unique identifier for this agent (e.g., "claude-code", "gemini-cli").
    pub id: String,

    /// Human-readable display name shown in the provider selector.
    pub display_name: String,

    /// Path or name of the executable to spawn (resolved via `$PATH`).
    pub command: String,

    /// Extra CLI arguments passed to the agent subprocess.
    #[serde(default)]
    pub args: Vec<String>,

    /// Extra environment variables set on the agent subprocess.
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Serializable model descriptors from the config file.
    /// Converted to `ModelInfo` at registration time via `model_infos()`.
    #[serde(default)]
    pub models: Vec<AcpModelEntry>,

    /// Optional install specification for agents not yet on PATH.
    #[serde(default)]
    pub install: Option<AcpAgentInstallSpec>,

    /// Optional authentication hint shown in the setup surface.
    #[serde(default)]
    pub auth: Option<AcpAgentAuthHint>,
}

/// How to install an ACP agent that is not yet available.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcpAgentInstallSpec {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
}

/// Human-readable authentication guidance for the setup surface.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcpAgentAuthHint {
    pub summary: String,
}

/// A lightweight, serializable model descriptor for ACP agent config files.
/// Converted to `crate::ai::ModelInfo` at provider registration time.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcpModelEntry {
    /// Model identifier sent to the agent (e.g., "claude-sonnet-4-6").
    pub id: String,

    /// Human-readable display name. Defaults to `id` if absent.
    #[serde(default)]
    pub display_name: Option<String>,

    /// Context window size in tokens. Defaults to 128 000 if absent.
    #[serde(default)]
    pub context_window: Option<u32>,
}

const DEFAULT_CONTEXT_WINDOW: u32 = 128_000;

/// Default Claude Code models available via the ACP adapter.
fn default_claude_code_models() -> Vec<AcpModelEntry> {
    vec![
        AcpModelEntry {
            id: "claude-sonnet-4-6".into(),
            display_name: Some("Sonnet 4.6".into()),
            context_window: Some(200_000),
        },
        AcpModelEntry {
            id: "claude-sonnet-4-5".into(),
            display_name: Some("Sonnet 4.5".into()),
            context_window: Some(200_000),
        },
        AcpModelEntry {
            id: "claude-opus-4-6".into(),
            display_name: Some("Opus 4.6".into()),
            context_window: Some(200_000),
        },
        AcpModelEntry {
            id: "claude-haiku-4-5".into(),
            display_name: Some("Haiku 4.5".into()),
            context_window: Some(200_000),
        },
    ]
}

impl AcpAgentConfig {
    /// Provider ID used for `AiProvider::provider_id()`.
    pub(crate) fn provider_id(&self) -> &str {
        &self.id
    }

    /// Display name used for `AiProvider::display_name()`.
    pub(crate) fn display_name(&self) -> &str {
        &self.display_name
    }

    /// Convert the serializable model entries into `ModelInfo` values
    /// suitable for `AiProvider::available_models()`.
    pub(crate) fn model_infos(&self) -> Vec<ModelInfo> {
        self.models
            .iter()
            .map(|entry| {
                ModelInfo::new(
                    &entry.id,
                    entry.display_name.as_deref().unwrap_or(&entry.id),
                    &self.id,
                    true,
                    entry.context_window.unwrap_or(DEFAULT_CONTEXT_WINDOW),
                )
            })
            .collect()
    }
}

/// Return the cached agent config, loading it on first call.
///
/// The first call spawns bun to transpile + extract config (~100-500ms).
/// Subsequent calls return instantly from the `OnceLock` cache.
/// Call `prewarm_agent_config()` at startup to pay the cost early.
pub(crate) fn claude_code_agent_config_cached() -> anyhow::Result<AcpAgentConfig> {
    if let Some(cached) = CACHED_AGENT_CONFIG.get() {
        return Ok(cached.clone());
    }
    let config = claude_code_agent_config()?;
    // Ignore the error if another thread raced us — their value is equivalent.
    let _ = CACHED_AGENT_CONFIG.set(config.clone());
    Ok(config)
}

/// Prewarm the agent config cache on a background thread.
/// Call once at startup so Tab presses never block on bun transpile.
pub(crate) fn prewarm_agent_config() {
    std::thread::Builder::new()
        .name("acp-config-prewarm".into())
        .spawn(|| {
            let started = std::time::Instant::now();
            match claude_code_agent_config() {
                Ok(config) => {
                    let _ = CACHED_AGENT_CONFIG.set(config);
                    tracing::info!(
                        target: "script_kit::tab_ai",
                        event = "acp_config_prewarmed",
                        elapsed_ms = started.elapsed().as_millis() as u64,
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        target: "script_kit::tab_ai",
                        event = "acp_config_prewarm_failed",
                        error = %e,
                    );
                }
            }
        })
        .ok();
}

/// Build an `AcpAgentConfig` for Claude Code from the user's Script Kit config.
///
/// Reads `claudeCode` settings (path, permissionMode, allowedTools, addDirs)
/// and maps them to ACP agent CLI arguments. Does not touch the PTY terminal
/// path — this is only used by the ACP event-driven surface.
///
/// Prefer `claude_code_agent_config_cached()` in hot paths to avoid repeated
/// bun subprocess spawns.
fn claude_code_agent_config() -> anyhow::Result<AcpAgentConfig> {
    let config = crate::config::load_config();
    let claude_code = config.claude_code.unwrap_or_default();

    let mut args = Vec::new();
    let configured_path = claude_code.path;

    if !claude_code.permission_mode.trim().is_empty() {
        args.push("--permission-mode".to_string());
        args.push(claude_code.permission_mode);
    }

    if let Some(allowed_tools) = claude_code
        .allowed_tools
        .filter(|value| !value.trim().is_empty())
    {
        args.push("--allowedTools".to_string());
        args.push(allowed_tools);
    }

    for add_dir in claude_code
        .add_dirs
        .into_iter()
        .filter(|value| !value.trim().is_empty())
    {
        args.push("--add-dir".to_string());
        args.push(add_dir);
    }

    // `claudeCode.path` historically points at the Claude CLI binary, not the
    // ACP adapter. Preserve that contract by defaulting to the ACP wrapper and
    // only using the configured path as the spawned command when it already
    // looks like an ACP adapter executable.
    let configured_path_looks_like_adapter = configured_path
        .as_deref()
        .map(|path| {
            let lowered = path.to_ascii_lowercase();
            lowered.contains("claude-agent-acp")
                || lowered.contains("claude-code-acp")
                || lowered.ends_with("-acp")
        })
        .unwrap_or(false);
    let (command, mut acp_args) = if configured_path_looks_like_adapter {
        (configured_path.unwrap_or_default(), Vec::new())
    } else {
        (
            "npx".to_string(),
            vec!["@agentclientprotocol/claude-agent-acp".to_string()],
        )
    };
    acp_args.extend(args);

    Ok(AcpAgentConfig {
        id: "claude-code".to_string(),
        display_name: "Claude Code".to_string(),
        command,
        args: acp_args,
        env: HashMap::new(),
        models: default_claude_code_models(),
        install: None,
        auth: None,
    })
}

// ---------------------------------------------------------------------------
// Multi-agent catalog loader
// ---------------------------------------------------------------------------

/// Check whether `command` resolves to an executable.
fn command_exists(command: &str) -> bool {
    if command.trim().is_empty() {
        return false;
    }
    if Path::new(command).exists() {
        return true;
    }
    which::which(command).is_ok()
}

/// Load all ACP agent configs from every source (legacy + catalog + built-in).
///
/// Sources (in priority order):
/// 1. Legacy Claude Code config (synthesized from `claudeCode` settings).
/// 2. `~/.scriptkit/acp/agents.json` (user-managed catalog file).
/// 3. Built-in auto-detection (`opencode`, `codex-acp` on PATH).
pub(crate) fn load_acp_agent_configs() -> anyhow::Result<Vec<AcpAgentConfig>> {
    let mut agents = Vec::new();

    // 1. Legacy compatibility: synthesize the existing Claude Code entry.
    match claude_code_agent_config_cached() {
        Ok(legacy_claude) => agents.push(legacy_claude),
        Err(e) => {
            tracing::debug!(
                target: "script_kit::tab_ai",
                event = "acp_legacy_claude_unavailable",
                error = %e,
            );
        }
    }

    // 2. Script Kit native multi-agent catalog.
    let catalog_path = super::catalog::default_acp_agents_path();
    if catalog_path.exists() {
        let bytes = std::fs::read(&catalog_path)
            .with_context(|| format!("read ACP agents catalog at {}", catalog_path.display()))?;
        let file: super::catalog::AcpAgentCatalogFile = serde_json::from_slice(&bytes)
            .with_context(|| {
                format!("parse ACP agents catalog at {}", catalog_path.display())
            })?;
        // Deduplicate: skip catalog entries whose id already exists.
        for agent in file.agents {
            if !agents.iter().any(|existing| existing.id == agent.id) {
                agents.push(agent);
            }
        }
    }

    // 3. Built-in OpenCode detection.
    if command_exists("opencode") && !agents.iter().any(|a| a.id == "opencode") {
        agents.push(AcpAgentConfig {
            id: "opencode".to_string(),
            display_name: "OpenCode".to_string(),
            command: "opencode".to_string(),
            args: vec!["acp".to_string()],
            env: HashMap::new(),
            models: Vec::new(),
            install: None,
            auth: None,
        });
    }

    // 4. Built-in Codex ACP detection.
    if command_exists("codex-acp") && !agents.iter().any(|a| a.id == "codex-acp") {
        agents.push(AcpAgentConfig {
            id: "codex-acp".to_string(),
            display_name: "Codex".to_string(),
            command: "codex-acp".to_string(),
            args: Vec::new(),
            env: HashMap::new(),
            models: Vec::new(),
            install: Some(AcpAgentInstallSpec {
                command: "npx".to_string(),
                args: vec!["@zed-industries/codex-acp".to_string()],
            }),
            auth: Some(AcpAgentAuthHint {
                summary: "Authenticate with ChatGPT, CODEX_API_KEY, or OPENAI_API_KEY."
                    .to_string(),
            }),
        });
    }

    tracing::info!(
        target: "script_kit::tab_ai",
        event = "acp_agent_configs_loaded",
        total_agents = agents.len(),
        ids = ?agents.iter().map(|a| a.id.as_str()).collect::<Vec<_>>(),
    );

    Ok(agents)
}

/// Build resolved catalog entries with install/auth/config state.
pub(crate) fn load_acp_agent_catalog_entries(
) -> anyhow::Result<Vec<super::catalog::AcpAgentCatalogEntry>> {
    let agents = load_acp_agent_configs()?;

    let entries = agents
        .into_iter()
        .map(|agent| {
            let install_state = if command_exists(&agent.command) || agent.command == "npx" {
                super::catalog::AcpAgentInstallState::Ready
            } else if agent.install.is_some() {
                super::catalog::AcpAgentInstallState::NeedsInstall
            } else {
                super::catalog::AcpAgentInstallState::Unsupported
            };

            let config_state = if agent.command.trim().is_empty() {
                super::catalog::AcpAgentConfigState::Missing
            } else {
                super::catalog::AcpAgentConfigState::Valid
            };

            let source = if agent.id == "claude-code" {
                super::catalog::AcpAgentSource::LegacyClaudeCode
            } else {
                super::catalog::AcpAgentSource::ScriptKitCatalog
            };

            let install_hint = agent.install.as_ref().map(|spec| {
                if spec.args.is_empty() {
                    gpui::SharedString::from(spec.command.clone())
                } else {
                    gpui::SharedString::from(format!(
                        "{} {}",
                        spec.command,
                        spec.args.join(" ")
                    ))
                }
            });

            super::catalog::AcpAgentCatalogEntry {
                id: agent.id.clone().into(),
                display_name: agent.display_name.clone().into(),
                source,
                install_state,
                auth_state: super::catalog::AcpAgentAuthState::Unknown,
                config_state,
                install_hint,
                config_hint: Some(
                    "Edit ~/.scriptkit/acp/agents.json to add or fix ACP agents.".into(),
                ),
                config: Some(agent),
            }
        })
        .collect::<Vec<_>>();

    tracing::info!(
        target: "script_kit::tab_ai",
        event = "acp_agent_catalog_built",
        total_entries = entries.len(),
        ready_entries = entries.iter().filter(|e| e.is_launchable()).count(),
    );

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_minimal_config() {
        let json = r#"{
            "id": "gemini-cli",
            "displayName": "Gemini CLI",
            "command": "gemini"
        }"#;
        let config: AcpAgentConfig =
            serde_json::from_str(json).expect("minimal config should parse");
        assert_eq!(config.id, "gemini-cli");
        assert_eq!(config.display_name, "Gemini CLI");
        assert_eq!(config.command, "gemini");
        assert!(config.args.is_empty());
        assert!(config.env.is_empty());
        assert!(config.models.is_empty());
    }

    #[test]
    fn round_trip_full_config() {
        let json = r#"{
            "id": "claude-code",
            "displayName": "Claude Code (ACP)",
            "command": "claude-acp",
            "args": ["--profile", "default"],
            "env": {"CLAUDE_CONFIG_DIR": "/tmp/claude"},
            "models": [
                {"id": "claude-sonnet-4-6", "displayName": "Claude Sonnet 4.6", "contextWindow": 200000}
            ]
        }"#;
        let config: AcpAgentConfig = serde_json::from_str(json).expect("full config should parse");
        assert_eq!(config.command, "claude-acp");
        assert_eq!(config.args, vec!["--profile", "default"]);
        assert_eq!(
            config.env.get("CLAUDE_CONFIG_DIR"),
            Some(&"/tmp/claude".to_string())
        );
        assert_eq!(config.models.len(), 1);
        assert_eq!(config.models[0].id, "claude-sonnet-4-6");
    }

    #[test]
    fn provider_id_and_display_name() {
        let config = AcpAgentConfig {
            id: "opencode".into(),
            display_name: "OpenCode".into(),
            command: "opencode".into(),
            args: vec!["acp".into()],
            env: HashMap::new(),
            models: vec![],
            install: None,
            auth: None,
        };
        assert_eq!(config.provider_id(), "opencode");
        assert_eq!(config.display_name(), "OpenCode");
    }

    #[test]
    fn serialize_round_trip() {
        let config = AcpAgentConfig {
            id: "codex".into(),
            display_name: "Codex (ACP)".into(),
            command: "codex-acp".into(),
            args: vec![],
            env: HashMap::new(),
            models: vec![],
            install: None,
            auth: None,
        };
        let json = serde_json::to_string(&config).expect("should serialize");
        let back: AcpAgentConfig = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(back.id, config.id);
        assert_eq!(back.command, config.command);
    }

    #[test]
    fn model_infos_defaults() {
        let config = AcpAgentConfig {
            id: "test-agent".into(),
            display_name: "Test".into(),
            command: "test".into(),
            args: vec![],
            env: HashMap::new(),
            models: vec![AcpModelEntry {
                id: "model-1".into(),
                display_name: None,
                context_window: None,
            }],
            install: None,
            auth: None,
        };
        let infos = config.model_infos();
        assert_eq!(infos.len(), 1);
        assert_eq!(infos[0].id, "model-1");
        assert_eq!(infos[0].display_name, "model-1");
        assert_eq!(infos[0].provider, "test-agent");
        assert!(infos[0].supports_streaming);
        assert_eq!(infos[0].context_window, DEFAULT_CONTEXT_WINDOW);
    }

    #[test]
    fn model_infos_explicit_values() {
        let config = AcpAgentConfig {
            id: "gemini".into(),
            display_name: "Gemini".into(),
            command: "gemini".into(),
            args: vec![],
            env: HashMap::new(),
            models: vec![AcpModelEntry {
                id: "gemini-2.5-pro".into(),
                display_name: Some("Gemini 2.5 Pro".into()),
                context_window: Some(1_000_000),
            }],
            install: None,
            auth: None,
        };
        let infos = config.model_infos();
        assert_eq!(infos[0].display_name, "Gemini 2.5 Pro");
        assert_eq!(infos[0].context_window, 1_000_000);
    }
}
