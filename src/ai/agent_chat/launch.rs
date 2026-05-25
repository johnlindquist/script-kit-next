use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use anyhow::{anyhow, Context as _, Result};

use crate::ai::acp::config::AcpModelEntry;
use crate::ai::agent_chat::pi::launch_spec::PiLaunchSpec;
use crate::ai::agent_chat::pi::{PiRpcLaunchSpec, PiRpcRuntime};
use crate::ai::agent_chat::profiles::{
    resolve_effective_profile, AgentChatProfileContext, ResolvedAgentChatProfile,
};
use crate::ai::agent_chat::runtime::AgentChatConnection;
use crate::ai::agent_chat::warm_key::pi_warm_key;
use crate::ai::agent_chat::warm_session::{AgentChatWarmSessionManager, AgentChatWarmSessionSpec};
use crate::config::{AgentChatBackend, AiPreferences};

static WARM_SESSION_MANAGER: OnceLock<AgentChatWarmSessionManager> = OnceLock::new();

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
    pub(crate) fn from_profile(profile: ResolvedAgentChatProfile) -> Result<Option<Self>> {
        if profile.backend != AgentChatBackend::Pi {
            return Ok(None);
        }

        let launch_spec = PiLaunchSpec::from_profile(&profile)
            .ok_or_else(|| {
                if profile.backend == AgentChatBackend::Pi {
                    anyhow!(
                        "Pi Agent Chat is selected, but no Pi binary was resolved. Ship Contents/MacOS/pi in the app bundle or configure ai.piBinary / SCRIPT_KIT_PI_BINARY."
                    )
                } else {
                    anyhow!("selected Agent Chat profile is not a Pi profile")
                }
            })?;
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

        Ok(Some(Self {
            profile,
            launch_spec,
            rpc_spec,
            warm_key,
            cwd,
            selected_model_id,
            available_models,
        }))
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
) -> Result<Option<PiAgentChatLaunch>> {
    PiAgentChatLaunch::from_profile(resolve_effective_profile(ai, ctx))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::agent_chat::profiles::{built_in_general_profile, built_in_script_kit_profile};

    fn ctx() -> AgentChatProfileContext {
        AgentChatProfileContext {
            kit_path: PathBuf::from("/tmp/kit"),
        }
    }

    #[test]
    fn pi_launch_from_profile_builds_rpc_spec_and_warm_key() {
        let ai = AiPreferences {
            pi_binary: Some("/tmp/test-pi".to_string()),
            ..AiPreferences::default()
        };
        let launch = resolve_selected_pi_launch(&ai, &ctx()).unwrap().unwrap();

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
        let launch = resolve_selected_pi_launch(&ai, &ctx()).unwrap().unwrap();

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
    fn non_pi_profile_does_not_build_pi_launch() {
        let mut profile = built_in_general_profile(&ctx());
        profile.backend = AgentChatBackend::Acp;

        assert!(PiAgentChatLaunch::from_profile(profile).unwrap().is_none());
    }

    #[test]
    fn selected_backend_acp_does_not_build_pi_launch() {
        let ai = AiPreferences {
            selected_backend: Some(AgentChatBackend::Acp),
            ..AiPreferences::default()
        };

        assert!(resolve_selected_pi_launch(&ai, &ctx()).unwrap().is_none());
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

        let launch = resolve_selected_pi_launch(&ai, &ctx).unwrap().unwrap();

        assert_eq!(launch.cwd, kit_path.join("agent-chat").join("general"));
        assert!(launch.cwd.is_dir());
    }
}
