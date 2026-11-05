use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use anyhow::{anyhow, Context as _, Result};

use crate::ai::agent_chat::goose::{GooseProfileLaunch, GooseRuntime};
use crate::ai::agent_chat::profiles::{
    resolve_effective_profile, resolve_effective_profile_decision,
    resolve_quick_ai_profile_decision, AgentChatProfileContext, ResolvedAgentChatProfile,
    BUILTIN_TEXT_PROFILE_ID,
};
use crate::ai::agent_chat::runtime::AgentChatConnection;
use crate::ai::agent_chat::ui::config::AgentChatModelEntry;
use crate::ai::agent_chat::warm_session::{AgentChatWarmSessionManager, AgentChatWarmSessionSpec};
use crate::config::{AgentChatBackend, AiPreferences};

static WARM_SESSION_MANAGER: OnceLock<AgentChatWarmSessionManager> = OnceLock::new();

/// A resolved Agent Chat launch: the profile, its goose session setup, and
/// the warm-session identity that lets launches share a prepared process.
#[derive(Debug, Clone)]
pub(crate) struct AgentChatLaunch {
    pub profile: ResolvedAgentChatProfile,
    pub goose: GooseProfileLaunch,
    pub cwd: PathBuf,
    pub selected_model_id: Option<String>,
    pub available_models: Vec<AgentChatModelEntry>,
}

impl AgentChatLaunch {
    pub(crate) fn from_profile(profile: ResolvedAgentChatProfile) -> Result<Self> {
        Self::from_profile_with_cwd_override(profile, None)
    }

    /// Resolve a launch, optionally overriding the working directory the
    /// goose session starts in.
    ///
    /// goose fixes a session's working directory at `session/new`, so the
    /// user's chosen directory (the Spine cwd chip) must be applied here —
    /// before the warm session is keyed and spawned. Because the warm key
    /// includes the cwd, an overridden cwd never reuses a session started in a
    /// different directory.
    pub(crate) fn from_profile_with_cwd_override(
        profile: ResolvedAgentChatProfile,
        cwd_override: Option<PathBuf>,
    ) -> Result<Self> {
        let goose_binary = profile.goose_binary.clone().ok_or_else(|| {
            anyhow!(
                "The goose AI engine is missing. Ship Contents/MacOS/goose in the app bundle or configure ai.gooseBinary / SCRIPT_KIT_GOOSE_BINARY."
            )
        })?;
        let dedicated_author = crate::ai::agent_chat::goose::recipe::profile_tool_ids(&profile)
            .is_some_and(|ids| {
                crate::ai::agent_chat::goose::recipe::recipe_tools(&ids)
                    .launcher_author()
                    .is_some()
            });
        let cwd = if dedicated_author {
            // The target is the current Kit workspace, never a project directory.
            crate::setup::get_kit_path()
                .join("agent-chat")
                .join("launcher-author")
        } else {
            cwd_override
                .or_else(|| profile.cwd.clone())
                .unwrap_or_else(crate::setup::get_kit_path)
        };
        std::fs::create_dir_all(&cwd)
            .with_context(|| format!("Failed to prepare Agent Chat cwd {}", cwd.display()))?;
        let goose = GooseProfileLaunch::from_profile(
            &profile,
            goose_binary,
            cwd.clone(),
            &crate::setup::get_kit_path(),
        );
        tracing::info!(
            target: "script_kit::agent_chat",
            event = "agent_chat_profile_launch_resolved",
            profile_id = %profile.id,
            profile_name = %profile.name,
            profile_source = ?profile.source,
            cwd = %cwd.display(),
            provider = ?goose.provider,
            model = ?goose.model,
            script_kit_tools = ?goose.tools.script_kit,
            developer_tools = ?goose.tools.developer,
        );
        let selected_model_id =
            model_selection_id(goose.provider.as_deref(), goose.model.as_deref());
        let available_models = selected_model_id
            .as_ref()
            .map(|id| {
                vec![AgentChatModelEntry {
                    id: id.clone(),
                    display_name: profile.model.clone().or_else(|| Some(id.clone())),
                    context_window: None,
                }]
            })
            .unwrap_or_default();

        Ok(Self {
            profile,
            goose,
            cwd,
            selected_model_id,
            available_models,
        })
    }

    pub(crate) fn warm_spec(&self) -> AgentChatWarmSessionSpec {
        let goose = self.goose.bind();
        AgentChatWarmSessionSpec {
            profile: self.profile.id.clone(),
            key: goose.key(),
            cwd: self.cwd.clone(),
            factory: Arc::new(move || {
                let spec = goose.spec_for_spawn()?;
                let policy_id = spec.agent_policy_id.clone();
                let runtime = GooseRuntime::spawn(spec).inspect_err(|_| {
                    if let Some(id) = &policy_id {
                        crate::mcp_agent_tools::unregister_policy(id);
                    }
                })?;
                Ok(Arc::new(runtime) as Arc<dyn AgentChatConnection>)
            }),
        }
    }
}

pub(crate) fn warm_session_manager() -> &'static AgentChatWarmSessionManager {
    WARM_SESSION_MANAGER.get_or_init(AgentChatWarmSessionManager::new)
}

pub(crate) fn resolve_selected_launch(
    ai: &AiPreferences,
    ctx: &AgentChatProfileContext,
) -> Result<AgentChatLaunch> {
    resolve_selected_launch_with_cwd_override(ai, ctx, None)
}

pub(crate) fn resolve_selected_launch_with_cwd_override(
    ai: &AiPreferences,
    ctx: &AgentChatProfileContext,
    cwd_override: Option<PathBuf>,
) -> Result<AgentChatLaunch> {
    let profile = resolve_effective_profile_decision(ai, ctx)
        .into_result()
        .map_err(anyhow::Error::msg)?;
    AgentChatLaunch::from_profile_with_cwd_override(profile, cwd_override)
}

/// Resolve only the working directory the currently selected profile would
/// launch in, mirroring `from_profile_with_cwd_override`'s fallback.
///
/// Performs no filesystem writes, builds no launch, requires no goose binary,
/// and emits no per-call log — safe for hot paths such as building the
/// main-menu `>` cwd picker rows on every keystroke.
pub(crate) fn resolve_selected_launch_cwd(
    ai: &AiPreferences,
    ctx: &AgentChatProfileContext,
) -> PathBuf {
    let profile = resolve_effective_profile(ai, ctx);
    profile.cwd.unwrap_or_else(crate::setup::get_kit_path)
}

pub(crate) fn resolve_focused_text_launch(
    ai: &AiPreferences,
    ctx: &AgentChatProfileContext,
) -> Result<AgentChatLaunch> {
    let text_ai = AiPreferences {
        // Deliberately drop the global model selection: the Text/rewrite mini
        // is pinned to `TEXT_MODEL` so variations stream back with minimal
        // latency regardless of the Agent Chat model.
        selected_model_id: None,
        selected_profile_id: Some(BUILTIN_TEXT_PROFILE_ID.to_string()),
        selected_backend: Some(AgentChatBackend::Goose),
        goose_binary: ai.goose_binary.clone(),
        profiles: ai.profiles.clone(),
        selected_profile_name: None,
        agent_chat_notify_when_hidden: ai.agent_chat_notify_when_hidden,
        cwd: ai.cwd.clone(),
        quick_ai_profile_id: None,
        agent_approval_mode: ai.agent_approval_mode.clone(),
    };

    AgentChatLaunch::from_profile(resolve_effective_profile(&text_ai, ctx))
}

/// Resolve the Quick AI launch (launcher Tab-with-text).
///
/// The profile comes from `resolve_quick_ai_profile_decision`: the user's
/// Shift+Tab picker choice (`ai.quick_ai_profile_id`) when set, else the
/// built-in zero-context default. The built-in default deliberately bypasses
/// `apply_ai_fallbacks`: its model is pinned to `QUICK_AI_MODEL` and must not
/// be replaced by `ai.selected_model_id`. Only the goose binary fallback from
/// preferences applies to it.
pub(crate) fn resolve_quick_ai_launch(
    ai: &AiPreferences,
    ctx: &AgentChatProfileContext,
) -> Result<AgentChatLaunch> {
    let mut profile = resolve_quick_ai_profile_decision(ai, ctx)
        .into_result()
        .map_err(anyhow::Error::msg)?;
    if profile.goose_binary.is_none() {
        profile.goose_binary =
            crate::ai::agent_chat::profiles::clean_opt(ai.goose_binary.as_deref())
                .map(crate::ai::agent_chat::goose::binary::expand_tilde_path)
                .or_else(crate::ai::agent_chat::goose::binary::default_goose_binary);
    }
    AgentChatLaunch::from_profile(profile)
}

fn model_selection_id(provider: Option<&str>, model: Option<&str>) -> Option<String> {
    let provider = provider?.trim();
    let model = model?.trim();
    (!provider.is_empty() && !model.is_empty()).then(|| format!("{provider}/{model}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::agent_chat::profiles::{
        built_in_text_profile, BUILTIN_BRAIN_PROFILE_ID, BUILTIN_QUICK_AI_PROFILE_ID,
        BUILTIN_SCRIPT_KIT_PROFILE_ID, QUICK_AI_MODEL, TEXT_MODEL,
    };

    fn ctx() -> AgentChatProfileContext {
        AgentChatProfileContext {
            kit_path: PathBuf::from("/tmp/kit"),
        }
    }

    fn test_warm_key(launch: &AgentChatLaunch) -> String {
        crate::ai::agent_chat::goose::bound_launch::BoundLaunch::from_snapshot(
            &launch.goose,
            Default::default(),
            None,
            &[],
        )
        .key()
    }

    fn ai() -> AiPreferences {
        AiPreferences {
            goose_binary: Some("/tmp/test-goose".to_string()),
            ..AiPreferences::default()
        }
    }

    #[test]
    fn text_profile_allows_only_web_search_and_no_file_access() {
        let profile = built_in_text_profile(&ctx());
        assert_eq!(profile.tools, Some(vec!["web_search".to_string()]));
        let path_policy = profile.path_policy.as_ref().expect("path policy");
        assert_eq!(path_policy.allow_read.as_deref(), Some(&[][..]));
        assert_eq!(path_policy.allow_write.as_deref(), Some(&[][..]));
        assert_eq!(profile.no_session, Some(true));
    }

    #[test]
    fn selected_launch_builds_a_goose_acp_spec_and_warm_key() {
        let launch = resolve_selected_launch(&ai(), &ctx()).unwrap();
        assert_eq!(launch.profile.name, "Brain");
        assert_eq!(launch.cwd, PathBuf::from("/tmp/kit/agent-chat/brain"));
        assert_eq!(launch.goose.spec.command, PathBuf::from("/tmp/test-goose"));
        assert_eq!(launch.goose.spec.args, vec!["acp".to_string()]);
        assert!(test_warm_key(&launch).starts_with("goose-warm-v2:"));
    }

    #[test]
    fn cwd_override_changes_the_session_directory_and_warm_key() {
        let ai = AiPreferences {
            selected_profile_id: Some(BUILTIN_SCRIPT_KIT_PROFILE_ID.to_string()),
            ..ai()
        };
        let default = resolve_selected_launch(&ai, &ctx()).unwrap();
        let cwd = PathBuf::from("/tmp/script-kit-work");
        let moved =
            resolve_selected_launch_with_cwd_override(&ai, &ctx(), Some(cwd.clone())).unwrap();
        assert_eq!(moved.profile.id, "script-kit");
        assert_eq!(moved.cwd, cwd);
        assert_eq!(moved.goose.spec.cwd, cwd);
        assert_ne!(test_warm_key(&moved), test_warm_key(&default));
    }

    #[test]
    fn launch_exposes_the_goose_provider_scoped_model() {
        let ai = AiPreferences {
            selected_profile_id: Some(BUILTIN_SCRIPT_KIT_PROFILE_ID.to_string()),
            ..ai()
        };
        let launch = resolve_selected_launch(&ai, &ctx()).unwrap();
        assert_eq!(
            launch.selected_model_id.as_deref(),
            Some("chatgpt_codex/gpt-6-luna")
        );
        assert_eq!(launch.goose.spec.thinking.as_deref(), Some("medium"));
    }

    #[test]
    fn previous_engine_model_selections_are_translated_for_goose() {
        let ai = AiPreferences {
            selected_model_id: Some("openai-codex/gpt-5.6-terra".to_string()),
            ..ai()
        };
        let launch = resolve_selected_launch(&ai, &ctx()).unwrap();
        assert_eq!(
            launch.selected_model_id.as_deref(),
            Some("chatgpt_codex/gpt-5.6-terra")
        );
    }

    #[test]
    fn focused_text_launch_is_pinned_and_locked_down() {
        let ai = AiPreferences {
            selected_model_id: Some("anthropic/claude-sonnet-5".to_string()),
            selected_profile_id: Some(BUILTIN_SCRIPT_KIT_PROFILE_ID.to_string()),
            ..ai()
        };
        let launch = resolve_focused_text_launch(&ai, &ctx()).unwrap();
        assert_eq!(launch.profile.id, BUILTIN_TEXT_PROFILE_ID);
        assert_eq!(launch.cwd, PathBuf::from("/tmp/kit/agent-chat/text"));
        assert_eq!(
            launch.selected_model_id,
            Some(format!("chatgpt_codex/{TEXT_MODEL}"))
        );
        assert_eq!(launch.goose.tools.script_kit, vec!["web_search"]);
        assert!(launch.goose.tools.developer.is_empty());
        assert!(launch.goose.spec.hidden_session);
    }

    #[test]
    fn focused_text_launch_does_not_inherit_agent_chat_prompts() {
        let ai = AiPreferences {
            selected_profile_id: Some("custom".to_string()),
            profiles: vec![crate::config::AgentChatProfile {
                id: Some("custom".to_string()),
                name: "Custom".to_string(),
                system_prompt: Some("normal chat system prompt".to_string()),
                append_system_prompt: Some("normal chat append prompt".to_string()),
                ..Default::default()
            }],
            ..ai()
        };
        let launch = resolve_focused_text_launch(&ai, &ctx()).unwrap();
        let instructions = launch.goose.spec.recipe["instructions"].as_str().unwrap();
        assert!(instructions.contains(crate::ai::agent_chat::profiles::TEXT_APPEND_SYSTEM_PROMPT));
        assert!(!instructions.contains("normal chat"));
    }

    #[test]
    fn quick_ai_launch_is_zero_context_and_pins_its_model() {
        let ai = AiPreferences {
            selected_model_id: Some("chatgpt_codex/gpt-5.6-terra".to_string()),
            selected_profile_id: Some(BUILTIN_SCRIPT_KIT_PROFILE_ID.to_string()),
            ..ai()
        };
        let launch = resolve_quick_ai_launch(&ai, &ctx()).unwrap();
        assert_eq!(launch.profile.id, BUILTIN_QUICK_AI_PROFILE_ID);
        assert_eq!(
            launch.selected_model_id,
            Some(format!("chatgpt_codex/{QUICK_AI_MODEL}"))
        );
        assert_eq!(launch.cwd, PathBuf::from("/tmp/kit/agent-chat/quick-ai"));
        assert_eq!(launch.goose.tools.script_kit, vec!["web_search"]);
        assert!(launch.goose.tools.developer.is_empty());
        assert_eq!(launch.goose.spec.thinking, None);
    }

    #[test]
    fn quick_ai_honors_the_shift_tab_profile_pick() {
        let ai = AiPreferences {
            quick_ai_profile_id: Some(BUILTIN_BRAIN_PROFILE_ID.to_string()),
            ..ai()
        };
        let launch = resolve_quick_ai_launch(&ai, &ctx()).unwrap();
        assert_eq!(launch.profile.id, BUILTIN_BRAIN_PROFILE_ID);
        assert_eq!(launch.cwd, PathBuf::from("/tmp/kit/agent-chat/brain"));
    }

    #[test]
    fn quick_ai_stale_profile_blocks_before_any_launch() {
        let ai = AiPreferences {
            quick_ai_profile_id: Some("deleted-profile".to_string()),
            ..ai()
        };
        let error = resolve_quick_ai_launch(&ai, &ctx()).expect_err("stale selection must block");
        assert_eq!(
            error.to_string(),
            "ai_profile_selection_unavailable:deleted-profile"
        );
    }

    #[test]
    fn launch_creates_the_profile_cwd_before_spawn() {
        let temp = tempfile::tempdir().expect("temp dir");
        let kit_path = temp.path().join(".scriptkit");
        let ctx = AgentChatProfileContext {
            kit_path: kit_path.clone(),
        };
        let launch = resolve_selected_launch(&ai(), &ctx).unwrap();
        assert_eq!(launch.cwd, kit_path.join("agent-chat").join("brain"));
        assert!(launch.cwd.is_dir());
    }
}
