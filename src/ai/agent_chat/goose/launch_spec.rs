//! Builds the goose launch for a resolved Agent Chat profile.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::ai::agent_chat::profiles::ResolvedAgentChatProfile;

use super::protocol::GooseLaunchSpec;
use super::recipe::{
    build_recipe, profile_tool_ids, recipe_tools, RecipeInput, RecipeTools,
    UNRESTRICTED_PROFILE_TOOLS,
};

/// goose state (config, sessions, sign-ins) lives under the kit, isolated from
/// any goose installation the user runs directly.
pub(crate) fn goose_root(kit_path: &Path) -> PathBuf {
    kit_path.join("goose")
}

/// Map a provider id from config onto goose's provider id. Profiles written
/// for the previous engine used `openai-codex` for the ChatGPT subscription.
pub(crate) fn goose_provider_id(provider: &str) -> String {
    match provider.trim() {
        "openai-codex" | "codex-subscription" => "chatgpt_codex".to_string(),
        "github-copilot" => "github_copilot".to_string(),
        "gemini" => "google".to_string(),
        "claude-code" => "claude_code".to_string(),
        other => other.to_string(),
    }
}

/// Provider credentials Script Kit manages, exported under the names goose's
/// providers read. Existing process variables win.
fn provider_credential_env() -> HashMap<String, String> {
    let keys = crate::ai::config::DetectedKeys::from_environment();
    let mut env = HashMap::new();
    let mut export = |name: &str, value: Option<&crate::ai::config::SecretString>| {
        if let Some(value) = value {
            if std::env::var_os(name).is_none() {
                env.insert(name.to_string(), value.expose().to_string());
            }
        }
    };
    export("OPENAI_API_KEY", keys.openai.as_ref());
    export("ANTHROPIC_API_KEY", keys.anthropic.as_ref());
    export("GOOGLE_API_KEY", keys.google.as_ref());
    export("GROQ_API_KEY", keys.groq.as_ref());
    export("AI_GATEWAY_API_KEY", keys.vercel.as_ref());
    env
}

/// Engine-level environment shared by every goose process Script Kit runs.
pub(crate) fn goose_process_env(root: &Path) -> HashMap<String, String> {
    let mut env = HashMap::from([
        (
            "GOOSE_PATH_ROOT".to_string(),
            root.to_string_lossy().into_owned(),
        ),
        // Sign-ins and keys stay in Script Kit's goose root, not the keychain
        // entries a separately installed goose would share.
        ("GOOSE_DISABLE_KEYRING".to_string(), "1".to_string()),
        ("GOOSE_TELEMETRY_ENABLED".to_string(), "false".to_string()),
        // The default for processes that never set a session mode (one-shot
        // helpers); chat sessions set theirs from the approval mode.
        ("GOOSE_MODE".to_string(), "smart_approve".to_string()),
    ]);
    env.extend(provider_credential_env());
    env
}

/// Seed a new engine root without replacing existing settings. The effective
/// tool boundary is the running session, not a comment in this file.
pub(crate) fn ensure_goose_root(root: &Path) -> std::io::Result<()> {
    let config_dir = root.join("config");
    std::fs::create_dir_all(&config_dir)?;
    use std::io::{Error, ErrorKind, Write as _};
    if !std::fs::symlink_metadata(&config_dir)?.is_dir() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Invalid engine config directory",
        ));
    }
    let config_path = config_dir.join("config.yaml");
    let existing = || match std::fs::symlink_metadata(&config_path) {
        Ok(meta) if meta.is_file() => Ok(true),
        Ok(_) => Err(Error::new(
            ErrorKind::InvalidInput,
            "Invalid engine config file",
        )),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err),
    };
    if existing()? {
        return Ok(());
    }
    let managed = "# script-kit-managed: developer-disabled\n\
extensions:\n\
\x20 developer:\n\
\x20   enabled: false\n\
\x20   type: platform\n\
\x20   name: developer\n\
\x20   description: ''\n\
\x20   display_name: Developer\n\
\x20   bundled: true\n";
    let mut tmp = tempfile::NamedTempFile::new_in(config_dir)?;
    tmp.write_all(managed.as_bytes())?;
    tmp.as_file().sync_data()?;
    match tmp.persist_noclobber(&config_path) {
        Ok(_) => Ok(()),
        Err(err) if err.error.kind() == ErrorKind::AlreadyExists && existing()? => Ok(()),
        Err(err) => Err(err.error),
    }
}

/// goose runs subagents without approvals, so outside full access a
/// delegation must itself be approved: seed `delegate` as ask-before in
/// goose's permission file unless the person already chose for it.
pub(crate) fn ensure_goose_permission_defaults(root: &Path) -> std::io::Result<()> {
    use serde_yaml::{Mapping, Value};
    let path = root.join("config").join("permission.yaml");
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let mut doc: Value = serde_yaml::from_str(&existing).unwrap_or(Value::Null);
    if !doc.is_mapping() {
        doc = Value::Mapping(Mapping::new());
    }
    let Some(root_map) = doc.as_mapping_mut() else {
        return Ok(());
    };
    let user = root_map
        .entry(Value::from("user"))
        .or_insert_with(|| Value::Mapping(Mapping::new()));
    if !user.is_mapping() {
        *user = Value::Mapping(Mapping::new());
    }
    let Some(user) = user.as_mapping_mut() else {
        return Ok(());
    };
    let decided = ["always_allow", "ask_before", "never_allow"]
        .iter()
        .any(|list| {
            user.get(*list)
                .and_then(Value::as_sequence)
                .is_some_and(|items| items.iter().any(|item| item.as_str() == Some("delegate")))
        });
    if decided {
        return Ok(());
    }
    for list in ["always_allow", "never_allow"] {
        user.entry(Value::from(list))
            .or_insert_with(|| Value::Sequence(Vec::new()));
    }
    let ask = user
        .entry(Value::from("ask_before"))
        .or_insert_with(|| Value::Sequence(Vec::new()));
    if !ask.is_sequence() {
        *ask = Value::Sequence(Vec::new());
    }
    if let Some(ask) = ask.as_sequence_mut() {
        ask.push(Value::from("delegate"));
    }
    std::fs::create_dir_all(root.join("config"))?;
    let text = serde_yaml::to_string(&doc).map_err(std::io::Error::other)?;
    std::fs::write(path, text)
}

/// A profile translated for goose, before per-process tool policy wiring.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GooseProfileLaunch {
    pub spec: GooseLaunchSpec,
    pub tools: RecipeTools,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub profile: ResolvedAgentChatProfile,
}

impl GooseProfileLaunch {
    pub(crate) fn from_profile(
        profile: &ResolvedAgentChatProfile,
        goose_binary: PathBuf,
        cwd: PathBuf,
        kit_path: &Path,
    ) -> Self {
        let tool_ids = profile_tool_ids(profile).unwrap_or_else(|| {
            UNRESTRICTED_PROFILE_TOOLS
                .iter()
                .map(|tool| tool.to_string())
                .collect()
        });
        let tools = recipe_tools(&tool_ids);
        if !tools.unknown.is_empty() {
            tracing::warn!(
                target: "script_kit::agent_chat",
                event = "goose_profile_unknown_tools",
                profile_id = %profile.id,
                tools = ?tools.unknown,
            );
        }
        let provider = profile.provider.as_deref().map(goose_provider_id);
        let model = profile.model.clone();
        let root = goose_root(kit_path);
        let mut spec = GooseLaunchSpec::new(goose_binary, cwd);
        spec.env = goose_process_env(&root);
        spec.session_title = profile.name.clone();
        spec.hidden_session = profile.no_session.unwrap_or(true);
        spec.thinking = profile.thinking.clone();
        spec.surface = crate::ai::phase_trace::AiSurface::from_profile(&profile.id, false);
        spec.recipe = build_recipe(RecipeInput {
            profile,
            provider: provider.as_deref(),
            model: model.as_deref(),
            tools: &tools,
            endpoint: None,
            user_mcp: &[],
        });
        Self {
            spec,
            tools,
            provider,
            model,
            profile: profile.clone(),
        }
    }

    /// Bind all authority before looking for a warm session; the factory uses exactly
    /// this snapshot. Resolving a profile for display never reads external server config.
    pub(crate) fn bind(&self) -> super::bound_launch::BoundLaunch {
        let mcp = if self.tools.user_mcp {
            super::recipe::user_mcp_extensions(&crate::config::load_config().get_mcp())
        } else {
            Vec::new()
        };
        super::bound_launch::BoundLaunch::from_snapshot(
            self,
            crate::ai::agent_chat::approval::AgentApprovalMode::current(),
            crate::mcp_agent_tools::endpoint_generation(),
            &mcp,
        )
    }
}

/// The goose root a launch environment points at.
pub(crate) fn goose_root_from_env(env: &HashMap<String, String>) -> PathBuf {
    env.get("GOOSE_PATH_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| goose_root(&crate::setup::get_kit_path()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::agent_chat::profiles::{built_in_text_profile, AgentChatProfileContext};

    #[test]
    fn previous_engine_provider_ids_map_to_goose_ids() {
        assert_eq!(goose_provider_id("openai-codex"), "chatgpt_codex");
        assert_eq!(goose_provider_id("anthropic"), "anthropic");
        assert_eq!(goose_provider_id("ollama"), "ollama");
    }

    #[test]
    fn goose_root_config_disables_the_default_developer_extension_once() {
        let temp = tempfile::tempdir().unwrap();
        ensure_goose_root(temp.path()).unwrap();
        let config = std::fs::read_to_string(temp.path().join("config/config.yaml")).unwrap();
        assert!(config.contains("enabled: false"));
        // User edits after the first write are preserved.
        // An engine-managed rewrite may drop comments or enable its own defaults.
        let edited = "GOOSE_PROVIDER: ollama\nextensions:\n  developer:\n    enabled: true\n";
        std::fs::write(temp.path().join("config/config.yaml"), edited).unwrap();
        ensure_goose_root(temp.path()).unwrap();
        assert_eq!(
            std::fs::read_to_string(temp.path().join("config/config.yaml")).unwrap(),
            edited
        );
    }

    #[test]
    fn delegation_asks_first_unless_the_person_already_chose() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(temp.path().join("config")).unwrap();
        std::fs::write(
            temp.path().join("config/permission.yaml"),
            "smart_approve:\n  ask_before:\n  - shell\n",
        )
        .unwrap();
        ensure_goose_permission_defaults(temp.path()).unwrap();
        let text = std::fs::read_to_string(temp.path().join("config/permission.yaml")).unwrap();
        let doc: serde_yaml::Value = serde_yaml::from_str(&text).unwrap();
        assert_eq!(doc["user"]["ask_before"][0], "delegate");
        assert_eq!(doc["smart_approve"]["ask_before"][0], "shell");

        let chosen = "user:\n  always_allow:\n  - delegate\n";
        std::fs::write(temp.path().join("config/permission.yaml"), chosen).unwrap();
        ensure_goose_permission_defaults(temp.path()).unwrap();
        assert_eq!(
            std::fs::read_to_string(temp.path().join("config/permission.yaml")).unwrap(),
            chosen
        );
    }

    #[test]
    fn profile_launch_isolates_goose_state_under_the_kit() {
        let ctx = AgentChatProfileContext {
            kit_path: PathBuf::from("/tmp/kit"),
        };
        let launch = GooseProfileLaunch::from_profile(
            &built_in_text_profile(&ctx),
            PathBuf::from("/tmp/goose"),
            PathBuf::from("/tmp/kit/agent-chat/text"),
            &ctx.kit_path,
        );
        assert_eq!(launch.spec.args, vec!["acp".to_string()]);
        assert_eq!(
            launch.spec.env.get("GOOSE_PATH_ROOT").map(String::as_str),
            Some("/tmp/kit/goose")
        );
        assert_eq!(launch.provider.as_deref(), Some("chatgpt_codex"));
        assert!(launch.tools.developer.is_empty());
        assert_eq!(launch.tools.script_kit, vec!["web_search"]);
    }
}
