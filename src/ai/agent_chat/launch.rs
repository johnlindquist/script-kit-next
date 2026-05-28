use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use anyhow::{anyhow, Context as _, Result};

use crate::ai::acp::config::AcpModelEntry;
use crate::ai::agent_chat::pi::launch_spec::PiLaunchSpec;
use crate::ai::agent_chat::pi::{PiRpcLaunchSpec, PiRpcRuntime};
use crate::ai::agent_chat::profiles::{
    resolve_effective_profile, AgentChatProfileContext, ResolvedAgentChatProfile,
    BUILTIN_TEXT_PROFILE_ID,
};
use crate::ai::agent_chat::runtime::AgentChatConnection;
use crate::ai::agent_chat::warm_key::pi_warm_key;
use crate::ai::agent_chat::warm_session::{AgentChatWarmSessionManager, AgentChatWarmSessionSpec};
use crate::config::{AgentChatBackend, AiPreferences};

static WARM_SESSION_MANAGER: OnceLock<AgentChatWarmSessionManager> = OnceLock::new();

pub(crate) const INLINE_AGENT_PI_APPEND_SYSTEM_PROMPT: &str = "You are Cue, Script Kit's inline text-editing assistant. You receive focused-field text through the user's prompt and must return only the requested text output. Do not describe system prompts, capture mechanics, tools, sessions, files, or Script Kit internals.";

#[derive(Debug, Clone)]
pub(crate) struct PiAgentChatLaunch {
    pub profile: ResolvedAgentChatProfile,
    pub launch_spec: PiLaunchSpec,
    pub rpc_spec: PiRpcLaunchSpec,
    pub warm_key: String,
    pub cwd: PathBuf,
    pub selected_model_id: Option<String>,
    pub available_models: Vec<AcpModelEntry>,
}

impl PiAgentChatLaunch {
    pub(crate) fn from_profile(profile: ResolvedAgentChatProfile) -> Result<Self> {
        Self::from_profile_with_cwd_override(profile, None)
    }

    /// Resolve a launch, optionally overriding the working directory the Pi
    /// process is spawned in.
    ///
    /// The Pi RPC worker bakes its `current_dir` from the launch spec at spawn
    /// time and ignores per-turn cwd, so the user's chosen working directory
    /// (the Spine cwd chip) must be applied here — before the warm session is
    /// keyed and spawned — not via `AcpThread::set_cwd` afterward. Because
    /// `pi_warm_key` includes the cwd, an overridden cwd produces a distinct
    /// warm-session key so a default-cwd warm session is never reused for a
    /// different directory.
    pub(crate) fn from_profile_with_cwd_override(
        profile: ResolvedAgentChatProfile,
        cwd_override: Option<PathBuf>,
    ) -> Result<Self> {
        let mut launch_spec = PiLaunchSpec::from_profile(&profile)
            .ok_or_else(|| {
                anyhow!(
                    "Pi Agent Chat is selected, but no Pi binary was resolved. Ship Contents/MacOS/pi in the app bundle or configure ai.piBinary / SCRIPT_KIT_PI_BINARY."
                )
            })?;
        if let Some(cwd_override) = cwd_override {
            launch_spec.cwd = Some(cwd_override);
        }
        let cwd = launch_spec
            .cwd
            .clone()
            .unwrap_or_else(crate::setup::get_kit_path);
        ensure_pi_cwd(&cwd)?;
        let warm_key = pi_warm_key(&launch_spec);
        let rpc_spec = PiRpcLaunchSpec::new(launch_spec.pi_binary.clone(), cwd.clone())
            .with_args(launch_spec.argv());
        let selected_model_id = pi_model_selection_id(&profile);
        let available_models = selected_model_id
            .as_ref()
            .map(|id| {
                vec![AcpModelEntry {
                    id: id.clone(),
                    display_name: profile.model.clone().or_else(|| Some(id.clone())),
                    context_window: None,
                }]
            })
            .unwrap_or_default();

        Ok(Self {
            profile,
            launch_spec,
            rpc_spec,
            warm_key,
            cwd,
            selected_model_id,
            available_models,
        })
    }

    pub(crate) fn warm_spec(&self) -> AgentChatWarmSessionSpec {
        let rpc_spec = self.rpc_spec.clone();
        AgentChatWarmSessionSpec {
            key: self.warm_key.clone(),
            cwd: self.cwd.clone(),
            factory: Arc::new(move || {
                let runtime = PiRpcRuntime::spawn(rpc_spec.clone())?;
                Ok(Arc::new(runtime) as Arc<dyn AgentChatConnection>)
            }),
        }
    }
}

pub(crate) fn warm_session_manager() -> &'static AgentChatWarmSessionManager {
    WARM_SESSION_MANAGER.get_or_init(AgentChatWarmSessionManager::new)
}

pub(crate) fn resolve_selected_pi_launch(
    ai: &AiPreferences,
    ctx: &AgentChatProfileContext,
) -> Result<PiAgentChatLaunch> {
    let profile = resolve_effective_profile(ai, ctx);
    PiAgentChatLaunch::from_profile(profile)
}

pub(crate) fn resolve_inline_agent_pi_launch(
    ai: &AiPreferences,
    ctx: &AgentChatProfileContext,
) -> Result<PiAgentChatLaunch> {
    let base = resolve_effective_profile(ai, ctx);
    let inline_profile = inline_agent_pi_profile(base, ctx);
    PiAgentChatLaunch::from_profile(inline_profile)
}

pub(crate) fn resolve_focused_text_pi_launch(
    ai: &AiPreferences,
    ctx: &AgentChatProfileContext,
) -> Result<PiAgentChatLaunch> {
    let text_ai = AiPreferences {
        selected_model_id: ai.selected_model_id.clone(),
        selected_profile_id: Some(BUILTIN_TEXT_PROFILE_ID.to_string()),
        selected_backend: Some(AgentChatBackend::Pi),
        pi_binary: ai.pi_binary.clone(),
        profiles: ai.profiles.clone(),
        selected_profile_name: None,
        cwd: ai.cwd.clone(),
    };

    PiAgentChatLaunch::from_profile(resolve_effective_profile(&text_ai, ctx))
}

fn pi_model_selection_id(profile: &ResolvedAgentChatProfile) -> Option<String> {
    let provider = profile.provider.as_deref()?.trim();
    let model = profile.model.as_deref()?.trim();
    if provider.is_empty() || model.is_empty() {
        return None;
    }
    Some(format!("{provider}/{model}"))
}

fn ensure_pi_cwd(cwd: &PathBuf) -> Result<()> {
    std::fs::create_dir_all(cwd)
        .with_context(|| format!("Failed to prepare Pi Agent Chat cwd {}", cwd.display()))
}

fn inline_agent_pi_profile(
    mut profile: ResolvedAgentChatProfile,
    ctx: &AgentChatProfileContext,
) -> ResolvedAgentChatProfile {
    profile.id = format!("inline-agent:{}", profile.id);
    profile.name = format!("Inline Agent ({})", profile.name);
    profile.backend = AgentChatBackend::Pi;
    profile.agent = None;
    profile.cwd = Some(ctx.kit_path.join("agent-chat").join("inline-agent"));
    profile.tools = Some(Vec::new());
    profile.disable_extensions = Some(true);
    profile.disable_skills = Some(true);
    profile.disable_prompt_templates = Some(true);
    profile.hide_cwd_in_prompt = Some(true);
    profile.session_dir = None;
    profile.no_session = Some(true);
    profile.session_durability = None;
    profile.system_prompt = None;
    profile.append_system_prompt = Some(INLINE_AGENT_PI_APPEND_SYSTEM_PROMPT.to_string());
    profile
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::agent_chat::profiles::{
        built_in_general_profile, built_in_script_kit_profile, built_in_text_profile,
    };

    fn ctx() -> AgentChatProfileContext {
        AgentChatProfileContext {
            kit_path: PathBuf::from("/tmp/kit"),
        }
    }

    /// Returns the value following a `--flag value` pair in an argv vector.
    fn argv_value<'a>(argv: &'a [String], flag: &str) -> Option<&'a str> {
        argv.windows(2)
            .find(|pair| pair[0] == flag)
            .map(|pair| pair[1].as_str())
    }

    #[test]
    fn text_profile_allows_only_web_search_and_no_file_access() {
        let profile = built_in_text_profile(&ctx());
        assert_eq!(profile.tools, Some(vec!["web_search".to_string()]));
        assert_eq!(
            profile.tool_policy.as_ref().and_then(|p| p.allow.clone()),
            Some(vec!["web_search".to_string()])
        );
        let path_policy = profile.path_policy.as_ref().expect("path policy");
        assert_eq!(path_policy.allow_read.as_deref(), Some(&[][..]));
        assert_eq!(path_policy.allow_write.as_deref(), Some(&[][..]));
        assert_eq!(profile.disable_extensions, Some(true));
        assert_eq!(profile.disable_skills, Some(true));
        assert_eq!(profile.disable_prompt_templates, Some(true));
        assert_eq!(profile.no_session, Some(true));
    }

    #[test]
    fn pi_launch_from_profile_builds_rpc_spec_and_warm_key() {
        let ai = AiPreferences {
            pi_binary: Some("/tmp/test-pi".to_string()),
            ..AiPreferences::default()
        };
        let launch = resolve_selected_pi_launch(&ai, &ctx()).unwrap();

        assert_eq!(launch.profile.name, "General");
        assert_eq!(launch.cwd, PathBuf::from("/tmp/kit/agent-chat/general"));
        assert_eq!(launch.rpc_spec.command, PathBuf::from("/tmp/test-pi"));
        assert!(launch.rpc_spec.args.contains(&"--mode".to_string()));
        assert!(launch.rpc_spec.args.contains(&"rpc".to_string()));
        assert!(launch.warm_key.starts_with("pi-warm-v1:"));
    }

    #[test]
    fn pi_launch_exposes_provider_scoped_model_for_selector_and_rpc() {
        let ai = AiPreferences {
            pi_binary: Some("/tmp/test-pi".to_string()),
            selected_profile_id: Some(
                crate::ai::agent_chat::profiles::BUILTIN_SCRIPT_KIT_PROFILE_ID.to_string(),
            ),
            ..AiPreferences::default()
        };
        let launch = resolve_selected_pi_launch(&ai, &ctx()).unwrap();

        assert_eq!(
            launch.selected_model_id.as_deref(),
            Some("openai-codex/gpt-5.4")
        );
        assert_eq!(launch.available_models.len(), 1);
        assert_eq!(launch.available_models[0].id, "openai-codex/gpt-5.4");
        assert_eq!(
            launch.available_models[0].display_name.as_deref(),
            Some("gpt-5.4")
        );
    }

    #[test]
    fn legacy_acp_backend_resolves_to_pi() {
        let ai = AiPreferences {
            pi_binary: Some("/tmp/test-pi".to_string()),
            selected_backend: Some(AgentChatBackend::Pi),
            ..AiPreferences::default()
        };

        let launch = resolve_selected_pi_launch(&ai, &ctx()).unwrap();
        assert_eq!(launch.profile.backend, AgentChatBackend::Pi);
        assert_eq!(launch.profile.id, "general");
    }

    #[test]
    fn focused_text_pi_launch_uses_text_profile_for_isolated_warm_profile() {
        let ai = AiPreferences {
            pi_binary: Some("/tmp/test-pi".to_string()),
            selected_profile_id: Some(
                crate::ai::agent_chat::profiles::BUILTIN_SCRIPT_KIT_PROFILE_ID.to_string(),
            ),
            ..AiPreferences::default()
        };

        let launch = resolve_focused_text_pi_launch(&ai, &ctx()).unwrap();
        let argv = launch.launch_spec.argv();

        assert_eq!(launch.profile.id, BUILTIN_TEXT_PROFILE_ID);
        assert_eq!(launch.profile.name, "Text");
        assert_eq!(launch.cwd, PathBuf::from("/tmp/kit/agent-chat/text"));
        assert_eq!(
            launch.selected_model_id.as_deref(),
            Some("openai-codex/gpt-5.4")
        );
        // The Text/mini profile now ships exactly one read-only network tool so
        // live-info questions can search the web; it must NOT fall back to
        // --no-tools, and must stay otherwise locked down.
        assert!(!argv.contains(&"--no-tools".to_string()));
        assert_eq!(argv_value(&argv, "--tools"), Some("web_search"));
        assert!(argv.contains(&"--no-extensions".to_string()));
        assert!(argv.contains(&"--no-skills".to_string()));
        assert!(argv.contains(&"--no-prompt-templates".to_string()));
        assert!(argv.contains(&"--hide-cwd-in-prompt".to_string()));
        assert!(argv.contains(&"--no-session".to_string()));
        assert_eq!(
            launch.launch_spec.append_system_prompt.as_deref(),
            Some(crate::ai::agent_chat::profiles::TEXT_APPEND_SYSTEM_PROMPT)
        );
        assert_eq!(launch.launch_spec.system_prompt, None);
    }

    #[test]
    fn focused_text_pi_launch_does_not_inherit_agent_chat_prompts() {
        let ai = AiPreferences {
            pi_binary: Some("/tmp/test-pi".to_string()),
            selected_profile_id: Some("custom-pi".to_string()),
            profiles: vec![crate::config::AcpProfile {
                id: Some("custom-pi".to_string()),
                name: "Custom Pi".to_string(),
                backend: Some(AgentChatBackend::Pi),
                provider: Some("openai-codex".to_string()),
                model: Some("gpt-5.4".to_string()),
                system_prompt: Some("normal chat system prompt".to_string()),
                append_system_prompt: Some("normal chat append prompt".to_string()),
                ..Default::default()
            }],
            ..AiPreferences::default()
        };

        let launch = resolve_focused_text_pi_launch(&ai, &ctx()).unwrap();
        let argv = launch.launch_spec.argv();

        assert_eq!(launch.profile.id, BUILTIN_TEXT_PROFILE_ID);
        assert_eq!(launch.launch_spec.system_prompt, None);
        assert_eq!(
            launch.launch_spec.append_system_prompt.as_deref(),
            Some(crate::ai::agent_chat::profiles::TEXT_APPEND_SYSTEM_PROMPT)
        );
        assert!(!argv.contains(&"--system-prompt".to_string()));
        assert!(!argv.iter().any(|arg| arg == "normal chat system prompt"));
        assert!(!argv.iter().any(|arg| arg == "normal chat append prompt"));
    }

    #[test]
    fn focused_text_pi_launch_uses_text_profile_with_model_override() {
        let ai = AiPreferences {
            pi_binary: Some("/tmp/test-pi".to_string()),
            selected_model_id: Some("claude-sonnet".to_string()),
            ..AiPreferences::default()
        };

        let launch = resolve_focused_text_pi_launch(&ai, &ctx()).unwrap();

        assert_eq!(launch.profile.id, BUILTIN_TEXT_PROFILE_ID);
        assert_eq!(launch.profile.backend, AgentChatBackend::Pi);
        assert_eq!(launch.profile.provider.as_deref(), Some("openai-codex"));
        assert_eq!(launch.profile.model.as_deref(), Some("claude-sonnet"));
        assert_eq!(launch.cwd, PathBuf::from("/tmp/kit/agent-chat/text"));
    }

    #[test]
    fn pi_launch_creates_profile_cwd_before_spawn() {
        let temp = tempfile::tempdir().expect("temp dir");
        let kit_path = temp.path().join(".scriptkit");
        let ctx = AgentChatProfileContext {
            kit_path: kit_path.clone(),
        };
        let ai = AiPreferences {
            pi_binary: Some("/tmp/test-pi".to_string()),
            ..AiPreferences::default()
        };

        let launch = resolve_selected_pi_launch(&ai, &ctx).unwrap();

        assert_eq!(launch.cwd, kit_path.join("agent-chat").join("general"));
        assert!(launch.cwd.is_dir());
    }
}
