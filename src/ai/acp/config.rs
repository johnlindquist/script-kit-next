use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::Context as _;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::ai::ModelInfo;

/// Cached agent config — avoids spawning bun processes on every Tab press.
static CACHED_AGENT_CONFIG: OnceLock<AcpAgentConfig> = OnceLock::new();

const CLAUDE_MCP_SYNC_SCHEMA_VERSION: u32 = 1;
pub(crate) const CODEX_ACP_AGENT_ID: &str = "codex-acp";

/// Configuration for a generic ACP-compatible AI agent.
///
/// Supports both direct ACP agents (OpenCode) and
/// adapter-wrapped agents (Claude Code via claude-acp, Codex via codex-acp).
/// The `command` + `args` fields let users point at whatever ACP binary
/// is actually installed — no agent name is hardcoded.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcpAgentConfig {
    /// Unique identifier for this agent (e.g., "claude-code", "codex-acp").
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
pub(crate) const CODEX_ACP_NPX_PACKAGE: &str = "@zed-industries/codex-acp";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CodexAcpAdapterSource {
    EnvOverride,
    RepoLocal,
    SiblingRepo,
    Path,
}

impl CodexAcpAdapterSource {
    fn as_str(self) -> &'static str {
        match self {
            Self::EnvOverride => "env_override",
            Self::RepoLocal => "repo_local",
            Self::SiblingRepo => "sibling_repo",
            Self::Path => "path",
        }
    }
}

#[derive(Debug, Clone)]
struct CodexAcpAdapterResolution {
    path: Option<PathBuf>,
    source: Option<CodexAcpAdapterSource>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CodexAcpDefaultProbeState {
    pub codex_cli_ready: bool,
    pub npx_ready: bool,
    pub codex_acp_binary_ready: bool,
    pub adapter_ready: bool,
    pub launch_ready: bool,
    pub should_be_implicit_codex_default: bool,
    pub npx_runtime_fallback_enabled: bool,
    adapter_source: Option<CodexAcpAdapterSource>,
}

fn existing_executable_file(path: PathBuf) -> Option<PathBuf> {
    let metadata = path.metadata().ok()?;
    if !metadata.is_file() {
        return None;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            return None;
        }
    }

    Some(path)
}

fn env_codex_acp_path() -> Option<PathBuf> {
    std::env::var_os("SCRIPT_KIT_CODEX_ACP_PATH")
        .map(PathBuf::from)
        .and_then(existing_executable_file)
}

fn sibling_repo_codex_acp_candidates(root: &Path) -> Vec<PathBuf> {
    let sibling_dev = root.join("codex-acp");
    vec![
        sibling_dev.join("target/release/codex-acp"),
        sibling_dev.join("target/debug/codex-acp"),
    ]
}

fn sibling_repo_codex_acp_search_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        if let Some(parent) = PathBuf::from(manifest_dir).parent() {
            roots.push(parent.to_path_buf());
        }
    }
    if let Ok(current_dir) = std::env::current_dir() {
        if let Some(parent) = current_dir.parent() {
            roots.push(parent.to_path_buf());
        }
    }
    if let Ok(exe_path) = std::env::current_exe() {
        let mut cursor = exe_path.as_path();
        while let Some(parent) = cursor.parent() {
            let parent_name = parent.file_name().and_then(|name| name.to_str());
            if parent_name == Some("target") || parent_name == Some("target-agent") {
                if let Some(repo_root) = parent.parent() {
                    if let Some(dev_root) = repo_root.parent() {
                        roots.push(dev_root.to_path_buf());
                    }
                }
                break;
            }
            cursor = parent;
        }
    }
    roots.sort();
    roots.dedup();
    roots
}

fn sibling_repo_codex_acp_path() -> Option<PathBuf> {
    sibling_repo_codex_acp_search_roots()
        .into_iter()
        .flat_map(|root| sibling_repo_codex_acp_candidates(&root))
        .find_map(existing_executable_file)
}

fn repo_local_codex_acp_path() -> Option<PathBuf> {
    let manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR")?);
    [
        manifest_dir.join("target/debug/codex-acp"),
        manifest_dir.join("target/release/codex-acp"),
    ]
    .into_iter()
    .find_map(existing_executable_file)
}

fn path_codex_acp_path() -> Option<PathBuf> {
    which::which(CODEX_ACP_AGENT_ID)
        .ok()
        .and_then(existing_executable_file)
}

fn resolved_codex_acp_adapter() -> CodexAcpAdapterResolution {
    if let Some(path) = env_codex_acp_path() {
        return CodexAcpAdapterResolution {
            path: Some(path),
            source: Some(CodexAcpAdapterSource::EnvOverride),
        };
    }
    if let Some(path) = sibling_repo_codex_acp_path() {
        return CodexAcpAdapterResolution {
            path: Some(path),
            source: Some(CodexAcpAdapterSource::SiblingRepo),
        };
    }
    if let Some(path) = repo_local_codex_acp_path() {
        return CodexAcpAdapterResolution {
            path: Some(path),
            source: Some(CodexAcpAdapterSource::RepoLocal),
        };
    }
    if let Some(path) = path_codex_acp_path() {
        return CodexAcpAdapterResolution {
            path: Some(path),
            source: Some(CodexAcpAdapterSource::Path),
        };
    }

    CodexAcpAdapterResolution {
        path: None,
        source: None,
    }
}

fn resolved_codex_acp_adapter_path() -> Option<PathBuf> {
    resolved_codex_acp_adapter().path
}

pub(crate) fn codex_acp_default_probe_state() -> CodexAcpDefaultProbeState {
    if codex_acp_disabled_by_env() {
        return CodexAcpDefaultProbeState {
            codex_cli_ready: false,
            npx_ready: false,
            codex_acp_binary_ready: false,
            adapter_ready: false,
            launch_ready: false,
            should_be_implicit_codex_default: false,
            npx_runtime_fallback_enabled: false,
            adapter_source: None,
        };
    }
    let adapter = resolved_codex_acp_adapter();
    codex_acp_default_probe_state_from_parts(
        command_exists("codex"),
        command_exists("npx"),
        adapter.path.is_some(),
        adapter.source,
    )
}

fn codex_acp_default_probe_state_from_parts(
    codex_cli_ready: bool,
    npx_ready: bool,
    codex_acp_binary_ready: bool,
    adapter_source: Option<CodexAcpAdapterSource>,
) -> CodexAcpDefaultProbeState {
    let adapter_ready = codex_acp_binary_ready;
    let launch_ready = adapter_ready && codex_cli_ready;
    CodexAcpDefaultProbeState {
        codex_cli_ready,
        npx_ready,
        codex_acp_binary_ready,
        adapter_ready,
        launch_ready,
        should_be_implicit_codex_default: launch_ready,
        npx_runtime_fallback_enabled: false,
        adapter_source,
    }
}

fn env_flag_enabled(name: &str) -> bool {
    std::env::var(name)
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn codex_acp_disabled_by_env() -> bool {
    env_flag_enabled("SCRIPT_KIT_DISABLE_CODEX_ACP")
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClaudeManagedMcpState {
    schema_version: u32,
    #[serde(default)]
    managed_servers: Vec<String>,
}

impl Default for ClaudeManagedMcpState {
    fn default() -> Self {
        Self {
            schema_version: CLAUDE_MCP_SYNC_SCHEMA_VERSION,
            managed_servers: Vec::new(),
        }
    }
}

fn script_kit_claude_mcp_sync_path() -> PathBuf {
    crate::setup::get_kit_path()
        .join("mcp")
        .join("claude-sync.json")
}

fn default_claude_user_config_path() -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir().context("resolve home directory for Claude MCP sync")?;
    Ok(home.join(".claude.json"))
}

fn build_claude_mcp_server_config(
    server: &crate::config::McpServerConfig,
) -> anyhow::Result<Value> {
    let value = match server {
        crate::config::McpServerConfig::Stdio(config) => {
            if config.command.trim().is_empty() {
                anyhow::bail!("MCP stdio server command cannot be empty");
            }

            let mut object = Map::new();
            object.insert("type".to_string(), Value::String("stdio".to_string()));
            object.insert("command".to_string(), Value::String(config.command.clone()));

            if !config.args.is_empty() {
                object.insert(
                    "args".to_string(),
                    Value::Array(config.args.iter().cloned().map(Value::String).collect()),
                );
            }
            if !config.env.is_empty() {
                object.insert(
                    "env".to_string(),
                    Value::Object(
                        config
                            .env
                            .iter()
                            .map(|(key, value)| (key.clone(), Value::String(value.clone())))
                            .collect(),
                    ),
                );
            }
            if let Some(cwd) = config.cwd.as_ref().filter(|cwd| !cwd.trim().is_empty()) {
                object.insert("cwd".to_string(), Value::String(cwd.clone()));
            }

            Value::Object(object)
        }
        crate::config::McpServerConfig::Http(config) => {
            if config.endpoint.trim().is_empty() {
                anyhow::bail!("MCP HTTP server endpoint cannot be empty");
            }

            let mut object = Map::new();
            object.insert("type".to_string(), Value::String("http".to_string()));
            object.insert("url".to_string(), Value::String(config.endpoint.clone()));

            if !config.headers.is_empty() {
                object.insert(
                    "headers".to_string(),
                    Value::Object(
                        config
                            .headers
                            .iter()
                            .map(|(key, value)| (key.clone(), Value::String(value.clone())))
                            .collect(),
                    ),
                );
            }

            Value::Object(object)
        }
    };

    Ok(value)
}

fn script_kit_managed_claude_mcp_servers(
    config: &crate::config::Config,
) -> anyhow::Result<Vec<(String, Value)>> {
    let mut servers = config
        .get_mcp()
        .enabled_servers()
        .map(|(server_id, server)| {
            build_claude_mcp_server_config(server).map(|value| (server_id.clone(), value))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    servers.sort_by(|(left, _), (right, _)| left.cmp(right));
    Ok(servers)
}

fn load_claude_managed_mcp_state(path: &Path) -> anyhow::Result<ClaudeManagedMcpState> {
    if !path.exists() {
        return Ok(ClaudeManagedMcpState::default());
    }

    let bytes = std::fs::read(path)
        .with_context(|| format!("read Claude MCP sync state {}", path.display()))?;
    let state = serde_json::from_slice::<ClaudeManagedMcpState>(&bytes)
        .with_context(|| format!("parse Claude MCP sync state {}", path.display()))?;
    Ok(state)
}

fn write_claude_managed_mcp_state(path: &Path, managed_servers: &[String]) -> anyhow::Result<()> {
    if managed_servers.is_empty() {
        if path.exists() {
            std::fs::remove_file(path)
                .with_context(|| format!("remove Claude MCP sync state {}", path.display()))?;
        }
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!(
                "create Claude MCP sync state directory {}",
                parent.display()
            )
        })?;
    }

    let state = ClaudeManagedMcpState {
        schema_version: CLAUDE_MCP_SYNC_SCHEMA_VERSION,
        managed_servers: managed_servers.to_vec(),
    };
    let bytes = serde_json::to_vec_pretty(&state)
        .with_context(|| format!("serialize Claude MCP sync state {}", path.display()))?;
    std::fs::write(path, bytes)
        .with_context(|| format!("write Claude MCP sync state {}", path.display()))?;
    Ok(())
}

fn sync_script_kit_mcp_to_claude(config: &crate::config::Config) -> anyhow::Result<()> {
    let desired_servers = script_kit_managed_claude_mcp_servers(config)?;
    let managed_server_names = desired_servers
        .iter()
        .map(|(server_id, _)| server_id.clone())
        .collect::<Vec<_>>();
    let claude_config_path = default_claude_user_config_path()?;
    let state_path = script_kit_claude_mcp_sync_path();

    sync_script_kit_mcp_to_claude_at(
        &desired_servers,
        &managed_server_names,
        &claude_config_path,
        &state_path,
    )
}

fn sync_script_kit_mcp_to_claude_at(
    desired_servers: &[(String, Value)],
    managed_server_names: &[String],
    claude_config_path: &Path,
    state_path: &Path,
) -> anyhow::Result<()> {
    let previous_state = load_claude_managed_mcp_state(state_path)?;
    let mut root = if claude_config_path.exists() {
        let bytes = std::fs::read(claude_config_path)
            .with_context(|| format!("read Claude config {}", claude_config_path.display()))?;
        serde_json::from_slice::<Value>(&bytes)
            .with_context(|| format!("parse Claude config {}", claude_config_path.display()))?
    } else {
        Value::Object(Map::new())
    };

    let root_object = root
        .as_object_mut()
        .context("Claude config root must be a JSON object")?;

    let mut existing_mcp_servers = match root_object.remove("mcpServers") {
        Some(Value::Object(object)) => object,
        Some(_) => anyhow::bail!("Claude config mcpServers must be a JSON object"),
        None => Map::new(),
    };

    for server_name in previous_state.managed_servers {
        existing_mcp_servers.remove(&server_name);
    }

    for (server_name, server_value) in desired_servers {
        existing_mcp_servers.insert(server_name.clone(), server_value.clone());
    }

    if existing_mcp_servers.is_empty() {
        root_object.remove("mcpServers");
    } else {
        root_object.insert(
            "mcpServers".to_string(),
            Value::Object(existing_mcp_servers),
        );
    }

    if let Some(parent) = claude_config_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create Claude config directory {}", parent.display()))?;
    }

    let bytes = serde_json::to_vec_pretty(&root)
        .with_context(|| format!("serialize Claude config {}", claude_config_path.display()))?;
    std::fs::write(claude_config_path, bytes)
        .with_context(|| format!("write Claude config {}", claude_config_path.display()))?;

    write_claude_managed_mcp_state(state_path, managed_server_names)?;
    Ok(())
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
    if let Err(error) = sync_script_kit_mcp_to_claude(&config) {
        tracing::warn!(
            target: "script_kit::tab_ai",
            event = "script_kit_mcp_sync_failed",
            error = %error,
        );
    }
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

fn looks_like_codex_acp_adapter_command(command: &str) -> bool {
    if command == CODEX_ACP_AGENT_ID {
        return true;
    }
    Path::new(command)
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == CODEX_ACP_AGENT_ID)
        .unwrap_or(false)
}

fn is_legacy_codex_acp_npx_config(agent: &AcpAgentConfig) -> bool {
    agent.command == "npx" && agent.args.iter().any(|arg| arg == CODEX_ACP_NPX_PACKAGE)
}

fn codex_acp_direct_args(existing_args: &[String]) -> Vec<String> {
    existing_args
        .iter()
        .filter(|arg| {
            let arg = arg.as_str();
            arg != CODEX_ACP_NPX_PACKAGE && arg != "-y" && arg != "--yes"
        })
        .cloned()
        .collect()
}

fn normalize_codex_acp_agent_config_with_path(
    mut agent: AcpAgentConfig,
    adapter_path: Option<PathBuf>,
) -> AcpAgentConfig {
    if agent.id != CODEX_ACP_AGENT_ID {
        return agent;
    }

    if looks_like_codex_acp_adapter_command(&agent.command)
        || is_legacy_codex_acp_npx_config(&agent)
    {
        agent.command = adapter_path
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_else(|| CODEX_ACP_AGENT_ID.to_string());
        agent.args = codex_acp_direct_args(&agent.args);
    }
    agent.install = None;

    agent
}

fn normalize_well_known_agent_config(agent: AcpAgentConfig) -> AcpAgentConfig {
    if agent.id == CODEX_ACP_AGENT_ID {
        normalize_codex_acp_agent_config_with_path(agent, resolved_codex_acp_adapter_path())
    } else {
        agent
    }
}

fn install_state_from_probe(
    agent: &AcpAgentConfig,
    command_ready: bool,
    adapter_ready: bool,
    codex_cli_ready: bool,
    _agy_cli_ready: bool,
) -> super::catalog::AcpAgentInstallState {
    use super::catalog::AcpAgentInstallState;

    let ready = if agent.id == CODEX_ACP_AGENT_ID {
        adapter_ready && codex_cli_ready
    } else {
        command_ready
    };

    if ready {
        AcpAgentInstallState::Ready
    } else if agent.install.is_some() {
        AcpAgentInstallState::NeedsInstall
    } else {
        AcpAgentInstallState::Unsupported
    }
}

fn install_state_for_agent(agent: &AcpAgentConfig) -> super::catalog::AcpAgentInstallState {
    let is_codex_acp = agent.id == CODEX_ACP_AGENT_ID;
    install_state_from_probe(
        agent,
        command_exists(&agent.command),
        if is_codex_acp {
            resolved_codex_acp_adapter_path().is_some()
        } else {
            false
        },
        !is_codex_acp || command_exists("codex"),
        true,
    )
}

fn opencode_agent_config() -> AcpAgentConfig {
    AcpAgentConfig {
        id: "opencode".to_string(),
        display_name: "OpenCode".to_string(),
        command: "opencode".to_string(),
        args: vec!["acp".to_string()],
        env: HashMap::new(),
        models: Vec::new(),
        install: Some(AcpAgentInstallSpec {
            command: "npm".to_string(),
            args: vec![
                "install".to_string(),
                "-g".to_string(),
                "opencode-ai".to_string(),
            ],
        }),
        auth: None,
    }
}

fn codex_acp_agent_config() -> AcpAgentConfig {
    AcpAgentConfig {
        id: CODEX_ACP_AGENT_ID.to_string(),
        display_name: "Codex".to_string(),
        command: CODEX_ACP_AGENT_ID.to_string(),
        args: Vec::new(),
        env: HashMap::new(),
        models: Vec::new(),
        install: None,
        auth: Some(AcpAgentAuthHint {
            summary: "Authenticate with ChatGPT, CODEX_API_KEY, or OPENAI_API_KEY.".to_string(),
        }),
    }
}

fn starter_acp_agent_configs() -> Vec<AcpAgentConfig> {
    vec![opencode_agent_config(), codex_acp_agent_config()]
}

fn merge_catalog_with_starter_agents(file: &mut super::catalog::AcpAgentCatalogFile) -> usize {
    let mut added = 0;
    for starter in starter_acp_agent_configs() {
        if file.agents.iter().any(|existing| existing.id == starter.id) {
            continue;
        }
        file.agents.push(starter);
        added += 1;
    }
    added
}

fn prune_deprecated_google_cli_agents(file: &mut super::catalog::AcpAgentCatalogFile) -> usize {
    let deprecated_id = ["gemini", "cli"].join("-");
    let deprecated_package = format!("{}/{}", "@google", ["gemini", "cli"].join("-"));
    let before = file.agents.len();
    file.agents.retain(|agent| {
        agent.id != deprecated_id
            && agent.command != "gemini"
            && !agent
                .args
                .iter()
                .any(|arg| arg == &deprecated_package || arg == "--acp")
    });
    before.saturating_sub(file.agents.len())
}

/// Ensure the ACP catalog exists and includes starter entries for common
/// ACP-compatible agents so the user has a concrete file to edit.
pub(crate) fn ensure_acp_agents_catalog_seeded() -> anyhow::Result<PathBuf> {
    let path = super::catalog::default_acp_agents_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create ACP catalog directory {}", parent.display()))?;
    }

    let existed = path.exists();
    let mut file = if existed {
        let bytes = std::fs::read(&path)
            .with_context(|| format!("read ACP agents catalog at {}", path.display()))?;
        serde_json::from_slice::<super::catalog::AcpAgentCatalogFile>(&bytes)
            .with_context(|| format!("parse ACP agents catalog at {}", path.display()))?
    } else {
        super::catalog::AcpAgentCatalogFile::default()
    };

    let pruned_count = prune_deprecated_google_cli_agents(&mut file);
    let starter_count = merge_catalog_with_starter_agents(&mut file);
    if !existed || starter_count > 0 || pruned_count > 0 {
        let bytes = serde_json::to_vec_pretty(&file)
            .with_context(|| format!("serialize ACP agents catalog at {}", path.display()))?;
        std::fs::write(&path, bytes)
            .with_context(|| format!("write ACP agents catalog at {}", path.display()))?;
    }

    tracing::info!(
        target: "script_kit::tab_ai",
        event = "acp_agent_catalog_seeded_for_editing",
        path = %path.display(),
        existed,
        starter_count,
        pruned_count,
        total_agents = file.agents.len(),
    );

    Ok(path)
}

/// Seed the ACP catalog with starter entries and open it in the default editor.
pub(crate) fn open_acp_agents_catalog_in_editor() -> anyhow::Result<PathBuf> {
    let path = ensure_acp_agents_catalog_seeded()?;

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-a")
            .arg("TextEdit")
            .arg(&path)
            .spawn()
            .with_context(|| format!("open ACP agents catalog in TextEdit: {}", path.display()))?;
    }

    tracing::info!(
        target: "script_kit::tab_ai",
        event = "acp_agent_catalog_editor_opened",
        path = %path.display(),
    );

    Ok(path)
}

/// Load all ACP agent configs from every source (legacy + catalog + built-in).
///
/// Sources (in priority order):
/// 1. Legacy Claude Code config (synthesized from `claudeCode` settings).
/// 2. `~/.scriptkit/acp/agents.json` (user-managed catalog file).
/// 3. Built-in auto-detection (`opencode`, `gemini`, local `codex` CLI).
pub(crate) fn load_acp_agent_configs() -> anyhow::Result<Vec<AcpAgentConfig>> {
    let mut agents = Vec::new();

    // 1. Legacy compatibility: synthesize the existing Claude Code entry.
    match claude_code_agent_config_cached() {
        Ok(legacy_claude) => agents.push(normalize_well_known_agent_config(legacy_claude)),
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
    {
        let mut file = match std::fs::read(&catalog_path) {
            Ok(bytes) => serde_json::from_slice::<super::catalog::AcpAgentCatalogFile>(&bytes)
                .with_context(|| {
                    format!("parse ACP agents catalog at {}", catalog_path.display())
                })?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                super::catalog::AcpAgentCatalogFile::default()
            }
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("read ACP agents catalog at {}", catalog_path.display())
                });
            }
        };
        let pruned_count = prune_deprecated_google_cli_agents(&mut file);
        let starter_count = merge_catalog_with_starter_agents(&mut file);
        if starter_count > 0 || pruned_count > 0 {
            tracing::info!(
                target: "script_kit::tab_ai",
                event = "acp_agent_catalog_starters_merged_runtime",
                path = %catalog_path.display(),
                starter_count,
                pruned_count,
            );
        }
        // Deduplicate: skip catalog entries whose id already exists.
        for agent in file.agents {
            if codex_acp_disabled_by_env() && agent.id == CODEX_ACP_AGENT_ID {
                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "acp_codex_agent_skipped",
                    reason = "disabled_by_env",
                );
                continue;
            }
            let agent = normalize_well_known_agent_config(agent);
            if !agents.iter().any(|existing| existing.id == agent.id) {
                agents.push(agent);
            }
        }
    }

    // 3. Built-in OpenCode detection.
    if command_exists("opencode") && !agents.iter().any(|a| a.id == "opencode") {
        agents.push(opencode_agent_config());
    }

    // 4. Built-in Codex ACP detection.
    let codex_probe = codex_acp_default_probe_state();
    if !codex_acp_disabled_by_env()
        && codex_probe.should_be_implicit_codex_default
        && !agents.iter().any(|a| a.id == CODEX_ACP_AGENT_ID)
    {
        agents.push(codex_acp_agent_config());
    }
    tracing::info!(
        target: "script_kit::tab_ai",
        event = "acp_codex_default_probe",
        codex_cli_ready = codex_probe.codex_cli_ready,
        npx_ready = codex_probe.npx_ready,
        codex_acp_binary_ready = codex_probe.codex_acp_binary_ready,
        adapter_ready = codex_probe.adapter_ready,
        launch_ready = codex_probe.launch_ready,
        should_be_implicit_codex_default = codex_probe.should_be_implicit_codex_default,
        npx_runtime_fallback_enabled = codex_probe.npx_runtime_fallback_enabled,
        codex_adapter_source = codex_probe
            .adapter_source
            .map(CodexAcpAdapterSource::as_str)
            .unwrap_or("none"),
    );

    tracing::info!(
        target: "script_kit::tab_ai",
        event = "acp_agent_configs_loaded",
        total_agents = agents.len(),
        ids = ?agents.iter().map(|a| a.id.as_str()).collect::<Vec<_>>(),
    );

    Ok(agents)
}

/// Build resolved catalog entries with install/auth/config state.
///
/// Overlays persisted runtime state (auth state, auth methods) from
/// `~/.scriptkit/acp/agent-runtime-state.json` so preflight sees truthful
/// auth state instead of always starting at `Unknown`.
pub(crate) fn load_acp_agent_catalog_entries(
) -> anyhow::Result<Vec<super::catalog::AcpAgentCatalogEntry>> {
    let agents = load_acp_agent_configs()?;
    let runtime_states = load_acp_agent_runtime_states();

    let entries = agents
        .into_iter()
        .map(|agent| {
            let install_state = install_state_for_agent(&agent);

            let config_state = if agent.command.trim().is_empty() {
                super::catalog::AcpAgentConfigState::Missing
            } else {
                super::catalog::AcpAgentConfigState::Valid
            };

            // Overlay persisted runtime state when available.
            let runtime_state = runtime_states.get(&agent.id);
            let auth_state = runtime_state
                .and_then(|state| state.auth_state)
                .unwrap_or(super::catalog::AcpAgentAuthState::Unknown);
            let supports_embedded_context =
                runtime_state.and_then(|state| state.supports_embedded_context);
            let supports_image = runtime_state.and_then(|state| state.supports_image);
            let last_session_ok = runtime_state
                .map(|state| state.last_session_ok)
                .unwrap_or(false);

            let source = classify_agent_source(&agent.id);

            let install_hint = agent.install.as_ref().map(|spec| {
                if spec.args.is_empty() {
                    gpui::SharedString::from(spec.command.clone())
                } else {
                    gpui::SharedString::from(format!("{} {}", spec.command, spec.args.join(" ")))
                }
            });

            tracing::info!(
                target: "script_kit::tab_ai",
                event = "acp_agent_catalog_entry_built",
                id = %agent.id,
                display_name = %agent.display_name,
                source = ?source,
                install_state = ?install_state,
                auth_state = ?auth_state,
                config_state = ?config_state,
            );
            if agent.id == CODEX_ACP_AGENT_ID {
                let codex_probe = codex_acp_default_probe_state();
                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "acp_codex_default_readiness",
                    codex_cli_ready = codex_probe.codex_cli_ready,
                    npx_ready = codex_probe.npx_ready,
                    codex_acp_binary_ready = codex_probe.codex_acp_binary_ready,
                    adapter_ready = codex_probe.adapter_ready,
                    launch_ready = codex_probe.launch_ready,
                    should_be_implicit_codex_default = codex_probe.should_be_implicit_codex_default,
                    npx_runtime_fallback_enabled = codex_probe.npx_runtime_fallback_enabled,
                    codex_adapter_source = codex_probe
                        .adapter_source
                        .map(CodexAcpAdapterSource::as_str)
                        .unwrap_or("none"),
                    install_state = ?install_state,
                    auth_state = ?auth_state,
                    config_state = ?config_state,
                );
            }

            super::catalog::AcpAgentCatalogEntry {
                id: agent.id.clone().into(),
                display_name: agent.display_name.clone().into(),
                source,
                install_state,
                auth_state,
                config_state,
                install_hint,
                config_hint: Some("Edit ~/.scriptkit/acp/agents.json to add or fix agents.".into()),
                supports_embedded_context,
                supports_image,
                last_session_ok,
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

fn merge_acp_agent_catalog_entries_with_snapshot(
    mut fresh_entries: Vec<super::catalog::AcpAgentCatalogEntry>,
    snapshot_entries: &[super::catalog::AcpAgentCatalogEntry],
) -> Vec<super::catalog::AcpAgentCatalogEntry> {
    for snapshot in snapshot_entries {
        if !fresh_entries.iter().any(|entry| entry.id == snapshot.id) {
            fresh_entries.push(snapshot.clone());
        }
    }
    fresh_entries
}

/// Reload the ACP agent catalog for UI pickers while preserving any live-session
/// snapshot entries that are not present in the current catalog.
pub(crate) fn refresh_acp_agent_catalog_entries_with_snapshot(
    snapshot_entries: &[super::catalog::AcpAgentCatalogEntry],
) -> Vec<super::catalog::AcpAgentCatalogEntry> {
    match load_acp_agent_catalog_entries() {
        Ok(fresh_entries) if !fresh_entries.is_empty() => {
            merge_acp_agent_catalog_entries_with_snapshot(fresh_entries, snapshot_entries)
        }
        Ok(_) => snapshot_entries.to_vec(),
        Err(error) => {
            tracing::warn!(
                target: "script_kit::tab_ai",
                event = "acp_agent_catalog_refresh_failed",
                error = %error,
                snapshot_count = snapshot_entries.len(),
            );
            snapshot_entries.to_vec()
        }
    }
}

/// Classify an agent by its well-known ID into a catalog source.
fn classify_agent_source(agent_id: &str) -> super::catalog::AcpAgentSource {
    match agent_id {
        "claude-code" => super::catalog::AcpAgentSource::LegacyClaudeCode,
        "opencode" | "codex-acp" => super::catalog::AcpAgentSource::BuiltIn,
        _ => super::catalog::AcpAgentSource::ScriptKitCatalog,
    }
}

/// Load the persisted preferred ACP agent ID from config-backed preferences.
pub(crate) fn load_preferred_acp_agent_id() -> Option<String> {
    crate::config::load_user_preferences()
        .ai
        .selected_acp_agent_id
}

/// Resolve the selected profile's non-empty system prompt from loaded
/// preferences.
pub(crate) fn selected_profile_system_prompt_from_preferences(
    ai: &crate::config::AiPreferences,
) -> Option<(String, String)> {
    let selected_name = ai
        .selected_profile_name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())?;

    ai.profiles
        .iter()
        .find(|profile| profile.name == selected_name)
        .and_then(|profile| {
            profile
                .system_prompt
                .as_deref()
                .map(str::trim)
                .filter(|prompt| !prompt.is_empty())
                .map(|prompt| (profile.name.clone(), prompt.to_string()))
        })
}

/// Load the selected profile's non-empty system prompt, if one is active.
pub(crate) fn load_selected_profile_system_prompt() -> Option<(String, String)> {
    let prefs = crate::config::load_user_preferences();
    selected_profile_system_prompt_from_preferences(&prefs.ai)
}

/// Persist the preferred ACP agent ID to `config.ts` synchronously.
///
/// Returns `Ok(())` when the write succeeds, so callers can gate retry
/// logic on a truthful persistence outcome instead of racing an async write.
pub(crate) fn persist_preferred_acp_agent_id_sync(agent_id: Option<String>) -> anyhow::Result<()> {
    let mut prefs = crate::config::load_user_preferences();
    prefs.ai.selected_acp_agent_id = agent_id.clone();
    crate::config::save_user_preferences(&prefs)?;
    tracing::info!(
        target: "script_kit::tab_ai",
        event = "acp_agent_selection_persisted_sync",
        ?agent_id,
    );
    Ok(())
}

/// Persist the preferred ACP agent ID to `config.ts` on a background thread.
///
/// Delegates to the synchronous helper internally. Use this when the caller
/// does not need to gate on persistence success (e.g. initial launch).
pub(crate) fn persist_preferred_acp_agent_id(agent_id: Option<String>) {
    std::thread::Builder::new()
        .name("acp-save-agent".into())
        .spawn(move || {
            if let Err(error) = persist_preferred_acp_agent_id_sync(agent_id.clone()) {
                tracing::warn!(
                    target: "script_kit::tab_ai",
                    event = "acp_agent_selection_persist_failed",
                    error = %error,
                    ?agent_id,
                );
            }
        })
        .ok();
}

// ---------------------------------------------------------------------------
// ACP agent runtime state persistence
// ---------------------------------------------------------------------------

const ACP_AGENT_RUNTIME_STATE_SCHEMA_VERSION: u32 = 1;

/// File-backed ACP agent runtime state cache.
///
/// Persisted at `~/.scriptkit/acp/agent-runtime-state.json` and overlaid onto
/// catalog entries at load time so preflight sees truthful auth state.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AcpAgentRuntimeStateFile {
    pub schema_version: u32,
    #[serde(default)]
    pub agents: HashMap<String, AcpAgentRuntimeState>,
}

impl Default for AcpAgentRuntimeStateFile {
    fn default() -> Self {
        Self {
            schema_version: ACP_AGENT_RUNTIME_STATE_SCHEMA_VERSION,
            agents: HashMap::new(),
        }
    }
}

/// Runtime state for a single ACP agent, cached between sessions.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AcpAgentRuntimeState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_state: Option<super::catalog::AcpAgentAuthState>,
    #[serde(default)]
    pub auth_methods: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supports_embedded_context: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supports_image: Option<bool>,
    #[serde(default)]
    pub last_session_ok: bool,
}

impl AcpAgentRuntimeState {
    fn auth_state_rank(state: Option<super::catalog::AcpAgentAuthState>) -> u8 {
        match state {
            Some(super::catalog::AcpAgentAuthState::Unknown) => 1,
            Some(super::catalog::AcpAgentAuthState::Authenticated) => 2,
            Some(super::catalog::AcpAgentAuthState::NeedsAuthentication) => 3,
            None => 0,
        }
    }

    /// Merge a new runtime snapshot into the existing persisted state without
    /// regressing known auth facts when background writes complete out of order.
    fn merged_with(&self, next: &Self) -> Self {
        let auth_state =
            if Self::auth_state_rank(next.auth_state) >= Self::auth_state_rank(self.auth_state) {
                next.auth_state
            } else {
                self.auth_state
            };

        Self {
            auth_state,
            auth_methods: if next.auth_methods.is_empty() {
                self.auth_methods.clone()
            } else {
                next.auth_methods.clone()
            },
            supports_embedded_context: next
                .supports_embedded_context
                .or(self.supports_embedded_context),
            supports_image: next.supports_image.or(self.supports_image),
            last_session_ok: next.last_session_ok || self.last_session_ok,
        }
    }
}

/// Default path for the ACP agent runtime state file.
pub(crate) fn default_acp_agent_runtime_state_path() -> PathBuf {
    crate::setup::get_kit_path()
        .join("acp")
        .join("agent-runtime-state.json")
}

/// Load all persisted ACP agent runtime states from disk.
pub(crate) fn load_acp_agent_runtime_states() -> HashMap<String, AcpAgentRuntimeState> {
    let path = default_acp_agent_runtime_state_path();
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(_) => return HashMap::new(),
    };
    match serde_json::from_slice::<AcpAgentRuntimeStateFile>(&bytes) {
        Ok(file) => {
            tracing::info!(
                target: "script_kit::tab_ai",
                event = "acp_agent_runtime_state_loaded",
                path = %path.display(),
                agent_count = file.agents.len(),
            );
            file.agents
        }
        Err(error) => {
            tracing::warn!(
                target: "script_kit::tab_ai",
                event = "acp_agent_runtime_state_load_failed",
                path = %path.display(),
                error = %error,
            );
            HashMap::new()
        }
    }
}

/// Persist runtime state for a single agent on a background thread.
pub(crate) fn persist_acp_agent_runtime_state(agent_id: String, next: AcpAgentRuntimeState) {
    std::thread::Builder::new()
        .name("acp-save-runtime-state".into())
        .spawn(move || {
            let path = default_acp_agent_runtime_state_path();

            let mut file = std::fs::read(&path)
                .ok()
                .and_then(|bytes| serde_json::from_slice::<AcpAgentRuntimeStateFile>(&bytes).ok())
                .unwrap_or_default();

            let merged = file
                .agents
                .get(&agent_id)
                .map(|current| current.merged_with(&next))
                .unwrap_or_else(|| next.clone());

            file.agents.insert(agent_id.clone(), merged.clone());

            if let Some(parent) = path.parent() {
                if let Err(error) = std::fs::create_dir_all(parent) {
                    tracing::warn!(
                        target: "script_kit::tab_ai",
                        event = "acp_agent_runtime_state_persist_failed",
                        path = %path.display(),
                        error = %error,
                        agent_id = %agent_id,
                    );
                    return;
                }
            }

            match serde_json::to_vec_pretty(&file) {
                Ok(bytes) => {
                    if let Err(error) = std::fs::write(&path, bytes) {
                        tracing::warn!(
                            target: "script_kit::tab_ai",
                            event = "acp_agent_runtime_state_persist_failed",
                            path = %path.display(),
                            error = %error,
                            agent_id = %agent_id,
                        );
                    } else {
                        tracing::info!(
                            target: "script_kit::tab_ai",
                            event = "acp_agent_runtime_state_persisted",
                            path = %path.display(),
                            agent_id = %agent_id,
                            auth_state = ?merged.auth_state,
                            auth_method_count = merged.auth_methods.len(),
                            last_session_ok = merged.last_session_ok,
                        );
                    }
                }
                Err(error) => {
                    tracing::warn!(
                        target: "script_kit::tab_ai",
                        event = "acp_agent_runtime_state_persist_failed",
                        path = %path.display(),
                        error = %error,
                        agent_id = %agent_id,
                    );
                }
            }
        })
        .ok();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::acp::catalog::{
        AcpAgentAuthState, AcpAgentCatalogEntry, AcpAgentConfigState, AcpAgentInstallState,
        AcpAgentSource,
    };
    use tempfile::tempdir;

    fn catalog_entry(id: &str, display_name: &str) -> AcpAgentCatalogEntry {
        AcpAgentCatalogEntry {
            id: id.to_string().into(),
            display_name: display_name.to_string().into(),
            source: AcpAgentSource::BuiltIn,
            install_state: AcpAgentInstallState::Ready,
            auth_state: AcpAgentAuthState::Unknown,
            config_state: AcpAgentConfigState::Valid,
            install_hint: None,
            config_hint: None,
            supports_embedded_context: None,
            supports_image: None,
            last_session_ok: false,
            config: None,
        }
    }

    #[test]
    fn round_trip_minimal_config() {
        let json = r#"{
            "id": "test-agent",
            "displayName": "Test Agent",
            "command": "test-agent"
        }"#;
        let config: AcpAgentConfig =
            serde_json::from_str(json).expect("minimal config should parse");
        assert_eq!(config.id, "test-agent");
        assert_eq!(config.display_name, "Test Agent");
        assert_eq!(config.command, "test-agent");
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
    fn starter_catalog_entries_include_common_acp_agents() {
        let starters = starter_acp_agent_configs();
        let ids = starters
            .iter()
            .map(|agent| agent.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, vec!["opencode", "codex-acp"]);
    }

    #[test]
    fn codex_starter_uses_direct_adapter_command() {
        let codex = starter_acp_agent_configs()
            .into_iter()
            .find(|agent| agent.id == "codex-acp")
            .expect("codex-acp starter");

        assert_eq!(codex.display_name, "Codex");
        assert_eq!(codex.command, CODEX_ACP_AGENT_ID);
        assert!(codex.args.is_empty());
        assert!(codex.install.is_none());
        assert!(codex
            .auth
            .expect("codex-acp auth hint")
            .summary
            .contains("OPENAI_API_KEY"));
    }

    #[test]
    fn acp_catalog_refresh_merge_keeps_fresh_codex_and_snapshot_selection() {
        let fresh = vec![
            catalog_entry("opencode", "OpenCode"),
            catalog_entry("codex-acp", "Codex"),
        ];
        let snapshot = vec![catalog_entry("opencode", "Stale OpenCode")];

        let merged = merge_acp_agent_catalog_entries_with_snapshot(fresh, &snapshot);
        let ids = merged
            .iter()
            .map(|entry| entry.id.as_ref())
            .collect::<Vec<_>>();

        assert_eq!(ids, vec!["opencode", "codex-acp"]);
        assert_eq!(merged[0].display_name.as_ref(), "OpenCode");
    }

    #[test]
    fn legacy_codex_npx_config_normalizes_to_resolved_adapter_without_npx() {
        let mut codex = codex_acp_agent_config();
        codex.command = "npx".into();
        codex.args = vec![
            "-y".into(),
            CODEX_ACP_NPX_PACKAGE.into(),
            "--verbose".into(),
        ];

        let normalized = normalize_codex_acp_agent_config_with_path(
            codex,
            Some(PathBuf::from(
                "/Applications/Script Kit.app/Contents/MacOS/codex-acp",
            )),
        );

        assert_eq!(
            normalized.command,
            "/Applications/Script Kit.app/Contents/MacOS/codex-acp"
        );
        assert_eq!(normalized.args, vec!["--verbose"]);
        assert!(normalized.install.is_none());
    }

    #[test]
    fn legacy_codex_acp_command_normalizes_to_resolved_adapter_without_npx() {
        let mut codex = codex_acp_agent_config();
        codex.command = "codex-acp".into();
        codex.args = vec!["--verbose".into()];

        let normalized = normalize_codex_acp_agent_config_with_path(
            codex,
            Some(PathBuf::from(
                "/tmp/Script Kit.app/Contents/MacOS/codex-acp",
            )),
        );

        assert_eq!(
            normalized.command,
            "/tmp/Script Kit.app/Contents/MacOS/codex-acp"
        );
        assert_eq!(normalized.args, vec!["--verbose"]);
        assert!(normalized.install.is_none());
    }

    #[test]
    fn missing_adapter_does_not_normalize_to_npx_runtime() {
        let mut codex = codex_acp_agent_config();
        codex.command = "/Users/example/dev/codex-acp/target/release/codex-acp".into();
        codex.args = Vec::new();

        let normalized = normalize_codex_acp_agent_config_with_path(codex, None);

        assert_eq!(normalized.command, CODEX_ACP_AGENT_ID);
        assert!(normalized.args.is_empty());
        assert!(normalized.install.is_none());
    }

    #[test]
    fn codex_acp_install_state_accepts_direct_adapter_only() {
        let codex = codex_acp_agent_config();

        assert_eq!(
            install_state_from_probe(&codex, true, false, true, true),
            crate::ai::acp::catalog::AcpAgentInstallState::Unsupported
        );
        assert_eq!(
            install_state_from_probe(&codex, false, false, true, true),
            crate::ai::acp::catalog::AcpAgentInstallState::Unsupported
        );

        let mut legacy = codex_acp_agent_config();
        legacy.command = "codex-acp".into();
        legacy.args = Vec::new();
        assert_eq!(
            install_state_from_probe(&legacy, false, true, true, true),
            crate::ai::acp::catalog::AcpAgentInstallState::Ready
        );
        assert_eq!(
            install_state_from_probe(&legacy, false, true, false, true),
            crate::ai::acp::catalog::AcpAgentInstallState::Unsupported,
            "Codex ACP adapter alone is not usable without the installed codex CLI"
        );
    }

    #[test]
    fn codex_default_probe_tracks_cli_and_adapter_separately() {
        let ready = codex_acp_default_probe_state_from_parts(true, true, false, None);
        assert!(ready.codex_cli_ready);
        assert!(ready.npx_ready);
        assert!(!ready.codex_acp_binary_ready);
        assert!(!ready.adapter_ready);
        assert!(!ready.launch_ready);
        assert!(!ready.should_be_implicit_codex_default);
        assert!(!ready.npx_runtime_fallback_enabled);

        let adapter_blocked = codex_acp_default_probe_state_from_parts(true, false, false, None);
        assert!(adapter_blocked.codex_cli_ready);
        assert!(!adapter_blocked.adapter_ready);
        assert!(
            !adapter_blocked.should_be_implicit_codex_default,
            "local codex CLI must not own default setup when the ACP adapter is missing"
        );

        let adapter_ready = codex_acp_default_probe_state_from_parts(
            true,
            true,
            true,
            Some(CodexAcpAdapterSource::Path),
        );
        assert!(adapter_ready.codex_cli_ready);
        assert!(adapter_ready.npx_ready);
        assert!(adapter_ready.codex_acp_binary_ready);
        assert!(adapter_ready.adapter_ready);
        assert!(adapter_ready.launch_ready);
        assert!(adapter_ready.should_be_implicit_codex_default);
        assert!(!adapter_ready.npx_runtime_fallback_enabled);

        let missing_cli = codex_acp_default_probe_state_from_parts(
            false,
            true,
            true,
            Some(CodexAcpAdapterSource::Path),
        );
        assert!(missing_cli.adapter_ready);
        assert!(!missing_cli.launch_ready);
        assert!(
            !missing_cli.should_be_implicit_codex_default,
            "adapter discovery must not select Codex by default when the codex CLI is missing"
        );
    }

    #[test]
    fn sibling_codex_acp_candidates_cover_release_before_debug() {
        let root = PathBuf::from("/Users/example/dev");
        let candidates = sibling_repo_codex_acp_candidates(&root);
        assert_eq!(
            candidates,
            vec![
                PathBuf::from("/Users/example/dev/codex-acp/target/release/codex-acp"),
                PathBuf::from("/Users/example/dev/codex-acp/target/debug/codex-acp"),
            ]
        );
    }

    #[test]
    fn merge_catalog_with_starters_preserves_existing_entries() {
        let mut file = crate::ai::acp::catalog::AcpAgentCatalogFile {
            schema_version: crate::ai::acp::catalog::ACP_AGENT_CATALOG_SCHEMA_VERSION,
            agents: vec![AcpAgentConfig {
                id: "opencode".into(),
                display_name: "OpenCode".into(),
                command: "opencode".into(),
                args: vec!["acp".into()],
                env: HashMap::new(),
                models: vec![],
                install: None,
                auth: None,
            }],
        };

        let added = merge_catalog_with_starter_agents(&mut file);
        assert_eq!(added, 1);
        assert_eq!(file.agents[0].id, "opencode");
        assert!(file.agents.iter().any(|agent| agent.id == "codex-acp"));
    }

    #[test]
    fn prune_deprecated_google_cli_agents_removes_old_rows() {
        let deprecated_id = ["gemini", "cli"].join("-");
        let mut file = crate::ai::acp::catalog::AcpAgentCatalogFile {
            schema_version: crate::ai::acp::catalog::ACP_AGENT_CATALOG_SCHEMA_VERSION,
            agents: vec![AcpAgentConfig {
                id: deprecated_id,
                display_name: "Deprecated Google CLI".into(),
                command: "gemini".into(),
                args: vec!["--acp".into()],
                env: HashMap::new(),
                models: vec![],
                install: None,
                auth: None,
            }],
        };

        let pruned = prune_deprecated_google_cli_agents(&mut file);
        assert_eq!(pruned, 1);
        assert!(file.agents.is_empty());
        assert!(file.agents.is_empty());
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
            id: "test-agent".into(),
            display_name: "Test Agent".into(),
            command: "test-agent".into(),
            args: vec![],
            env: HashMap::new(),
            models: vec![AcpModelEntry {
                id: "default".into(),
                display_name: Some("Test Agent Default".into()),
                context_window: Some(1_000_000),
            }],
            install: None,
            auth: None,
        };
        let infos = config.model_infos();
        assert_eq!(infos[0].display_name, "Test Agent Default");
        assert_eq!(infos[0].context_window, 1_000_000);
    }

    #[test]
    fn runtime_state_file_round_trip() {
        let json = r#"{
            "schemaVersion": 1,
            "agents": {
                "codex-acp": {
                    "authState": "needsAuthentication",
                    "authMethods": ["chatgpt-login", "openai-api-key"],
                    "supportsEmbeddedContext": true,
                    "supportsImage": false,
                    "lastSessionOk": false
                }
            }
        }"#;
        let file: AcpAgentRuntimeStateFile =
            serde_json::from_str(json).expect("runtime state should parse");
        assert_eq!(file.schema_version, 1);
        assert!(file.agents.is_empty());
        let codex = file.agents.get("codex-acp").expect("codex-acp entry");
        assert_eq!(
            codex.auth_state,
            Some(crate::ai::acp::catalog::AcpAgentAuthState::NeedsAuthentication)
        );
        assert_eq!(codex.auth_methods, vec!["chatgpt-login", "openai-api-key"]);
        assert_eq!(codex.supports_embedded_context, Some(true));
        assert_eq!(codex.supports_image, Some(false));
        assert!(!codex.last_session_ok);
    }

    #[test]
    fn runtime_state_file_defaults_on_missing_fields() {
        let json = r#"{"schemaVersion": 1, "agents": {"test": {}}}"#;
        let file: AcpAgentRuntimeStateFile =
            serde_json::from_str(json).expect("should parse with defaults");
        let state = file.agents.get("test").expect("test entry");
        assert!(state.auth_state.is_none());
        assert!(state.auth_methods.is_empty());
        assert!(state.supports_embedded_context.is_none());
        assert!(state.supports_image.is_none());
        assert!(!state.last_session_ok);
    }

    #[test]
    fn runtime_state_serialize_skips_none_fields() {
        let state = AcpAgentRuntimeState {
            auth_state: Some(crate::ai::acp::catalog::AcpAgentAuthState::Authenticated),
            auth_methods: vec!["terminal".to_string()],
            supports_embedded_context: None,
            supports_image: None,
            last_session_ok: true,
        };
        let json = serde_json::to_string(&state).expect("should serialize");
        assert!(!json.contains("supportsEmbeddedContext"));
        assert!(!json.contains("supportsImage"));
        assert!(json.contains("authenticated"));
    }

    #[test]
    fn runtime_state_merge_does_not_regress_auth_state() {
        let current = AcpAgentRuntimeState {
            auth_state: Some(crate::ai::acp::catalog::AcpAgentAuthState::Authenticated),
            auth_methods: vec!["chatgpt-login".to_string()],
            supports_embedded_context: Some(true),
            supports_image: Some(true),
            last_session_ok: true,
        };
        let stale_initialize = AcpAgentRuntimeState {
            auth_state: Some(crate::ai::acp::catalog::AcpAgentAuthState::Unknown),
            auth_methods: vec!["chatgpt-login".to_string(), "openai-api-key".to_string()],
            supports_embedded_context: Some(true),
            supports_image: Some(false),
            last_session_ok: false,
        };

        let merged = current.merged_with(&stale_initialize);
        assert_eq!(
            merged.auth_state,
            Some(crate::ai::acp::catalog::AcpAgentAuthState::Authenticated)
        );
        assert_eq!(
            merged.auth_methods,
            vec!["chatgpt-login".to_string(), "openai-api-key".to_string()]
        );
        assert_eq!(merged.supports_embedded_context, Some(true));
        assert_eq!(merged.supports_image, Some(false));
        assert!(merged.last_session_ok);
    }

    #[test]
    fn runtime_state_merge_allows_auth_required_to_override_unknown() {
        let current = AcpAgentRuntimeState {
            auth_state: Some(crate::ai::acp::catalog::AcpAgentAuthState::Unknown),
            auth_methods: vec!["chatgpt-login".to_string()],
            supports_embedded_context: Some(true),
            supports_image: Some(true),
            last_session_ok: false,
        };
        let auth_required = AcpAgentRuntimeState {
            auth_state: Some(crate::ai::acp::catalog::AcpAgentAuthState::NeedsAuthentication),
            auth_methods: vec!["chatgpt-login".to_string()],
            supports_embedded_context: Some(true),
            supports_image: Some(true),
            last_session_ok: false,
        };

        let merged = current.merged_with(&auth_required);
        assert_eq!(
            merged.auth_state,
            Some(crate::ai::acp::catalog::AcpAgentAuthState::NeedsAuthentication)
        );
        assert!(!merged.last_session_ok);
    }

    #[test]
    fn sync_script_kit_mcp_to_claude_preserves_unmanaged_servers() {
        let temp = tempdir().expect("temp dir");
        let claude_config_path = temp.path().join(".claude.json");
        let state_path = temp.path().join("claude-sync.json");

        let existing = serde_json::json!({
            "mcpServers": {
                "user-server": {
                    "type": "http",
                    "url": "https://example.com/mcp"
                },
                "old-script-kit": {
                    "type": "stdio",
                    "command": "old"
                }
            }
        });
        std::fs::write(
            &claude_config_path,
            serde_json::to_vec_pretty(&existing).expect("serialize existing config"),
        )
        .expect("write existing config");

        write_claude_managed_mcp_state(&state_path, &["old-script-kit".to_string()])
            .expect("seed sync state");

        let desired_servers = vec![(
            "linear".to_string(),
            serde_json::json!({
                "type": "http",
                "url": "https://mcp.linear.app/sse"
            }),
        )];

        sync_script_kit_mcp_to_claude_at(
            &desired_servers,
            &["linear".to_string()],
            &claude_config_path,
            &state_path,
        )
        .expect("sync MCP config");

        let synced = serde_json::from_slice::<Value>(
            &std::fs::read(&claude_config_path).expect("read synced config"),
        )
        .expect("parse synced config");
        let servers = synced["mcpServers"]
            .as_object()
            .expect("mcpServers object after sync");
        assert!(servers.contains_key("user-server"));
        assert!(servers.contains_key("linear"));
        assert!(!servers.contains_key("old-script-kit"));
    }

    #[test]
    fn sync_script_kit_mcp_to_claude_removes_state_when_empty() {
        let temp = tempdir().expect("temp dir");
        let claude_config_path = temp.path().join(".claude.json");
        let state_path = temp.path().join("claude-sync.json");

        let existing = serde_json::json!({
            "theme": "dark",
            "mcpServers": {
                "old-script-kit": {
                    "type": "stdio",
                    "command": "old"
                }
            }
        });
        std::fs::write(
            &claude_config_path,
            serde_json::to_vec_pretty(&existing).expect("serialize existing config"),
        )
        .expect("write existing config");

        write_claude_managed_mcp_state(&state_path, &["old-script-kit".to_string()])
            .expect("seed sync state");

        sync_script_kit_mcp_to_claude_at(&[], &[], &claude_config_path, &state_path)
            .expect("clear managed servers");

        let synced = serde_json::from_slice::<Value>(
            &std::fs::read(&claude_config_path).expect("read synced config"),
        )
        .expect("parse synced config");
        assert_eq!(synced["theme"], "dark");
        assert!(synced.get("mcpServers").is_none());
        assert!(!state_path.exists());
    }
}
