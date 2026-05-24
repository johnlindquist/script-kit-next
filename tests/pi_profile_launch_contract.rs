use std::path::{Path, PathBuf};

use script_kit_gpui::ai::agent_chat::pi::launch_spec::PiLaunchSpec;
use script_kit_gpui::ai::agent_chat::profiles::{
    resolve_effective_profile, AgentChatProfileContext, AgentChatProfileSource,
    BUILTIN_GENERAL_PROFILE_ID, BUILTIN_SCRIPT_KIT_PROFILE_ID, DEFAULT_PI_MODEL,
    DEFAULT_PI_PROVIDER, SCRIPT_KIT_PI_TOOLS,
};
use script_kit_gpui::config::{AcpProfile, AgentChatBackend, AiPreferences};

fn context() -> AgentChatProfileContext {
    AgentChatProfileContext {
        kit_path: PathBuf::from("/Users/test/.scriptkit"),
    }
}

#[test]
fn general_builtin_profile_builds_locked_down_pi_rpc_launch_spec() {
    let ctx = context();
    let profile = resolve_effective_profile(&AiPreferences::default(), &ctx);
    assert_eq!(profile.source, AgentChatProfileSource::BuiltIn);
    assert_eq!(profile.id, BUILTIN_GENERAL_PROFILE_ID);
    assert_eq!(profile.name, "General");
    assert_eq!(profile.backend, AgentChatBackend::Pi);
    assert_eq!(profile.provider.as_deref(), Some(DEFAULT_PI_PROVIDER));
    assert_eq!(profile.model.as_deref(), Some(DEFAULT_PI_MODEL));
    assert_eq!(
        profile.cwd.as_deref(),
        Some(Path::new("/Users/test/.scriptkit/agent-chat/general"))
    );

    let spec = PiLaunchSpec::from_profile(&profile).expect("general profile should be Pi");
    let argv = spec.argv();
    assert_eq!(spec.cwd.as_deref(), profile.cwd.as_deref());
    assert!(argv.starts_with(&["--mode".to_string(), "rpc".to_string()]));
    assert!(argv.contains(&"--provider".to_string()));
    assert!(argv.contains(&DEFAULT_PI_PROVIDER.to_string()));
    assert!(argv.contains(&"--model".to_string()));
    assert!(argv.contains(&DEFAULT_PI_MODEL.to_string()));
    assert!(argv.contains(&"--append-system-prompt".to_string()));
    assert!(argv.contains(&"--no-tools".to_string()));
    assert!(argv.contains(&"--no-extensions".to_string()));
    assert!(argv.contains(&"--no-skills".to_string()));
    assert!(argv.contains(&"--no-prompt-templates".to_string()));
    assert!(argv.contains(&"--hide-cwd-in-prompt".to_string()));
    assert!(!argv.contains(&"--cwd".to_string()));
    assert!(!argv.contains(&"--agent".to_string()));
    assert!(!argv.contains(&"--no-hooks".to_string()));
}

#[test]
fn script_kit_builtin_profile_builds_workspace_pi_rpc_launch_spec() {
    let ctx = context();
    let mut ai = AiPreferences::default();
    ai.selected_profile_id = Some(BUILTIN_SCRIPT_KIT_PROFILE_ID.to_string());

    let profile = resolve_effective_profile(&ai, &ctx);
    assert_eq!(profile.id, BUILTIN_SCRIPT_KIT_PROFILE_ID);
    assert_eq!(profile.name, "Script Kit");
    assert_eq!(profile.backend, AgentChatBackend::Pi);
    assert_eq!(
        profile.cwd.as_deref(),
        Some(Path::new("/Users/test/.scriptkit"))
    );
    assert_eq!(
        profile.tools.as_deref(),
        Some(
            SCRIPT_KIT_PI_TOOLS
                .iter()
                .map(|tool| tool.to_string())
                .collect::<Vec<_>>()
                .as_slice()
        )
    );
    assert_eq!(profile.hide_cwd_in_prompt, Some(false));

    let spec = PiLaunchSpec::from_profile(&profile).expect("script-kit profile should be Pi");
    let argv = spec.argv();
    assert!(argv.contains(&"--tools".to_string()));
    assert!(argv.contains(&SCRIPT_KIT_PI_TOOLS.join(",")));
    assert!(!argv.contains(&"--no-tools".to_string()));
    assert!(!argv.contains(&"--hide-cwd-in-prompt".to_string()));
}

#[test]
fn selected_profile_id_takes_precedence_over_selected_profile_name() {
    let ctx = context();
    let ai = AiPreferences {
        selected_profile_id: Some(BUILTIN_GENERAL_PROFILE_ID.to_string()),
        selected_profile_name: Some("Script Kit".to_string()),
        ..AiPreferences::default()
    };

    let profile = resolve_effective_profile(&ai, &ctx);
    assert_eq!(profile.id, BUILTIN_GENERAL_PROFILE_ID);
}

#[test]
fn selected_profile_name_resolves_legacy_acp_profile() {
    let ctx = context();
    let ai = AiPreferences {
        selected_profile_name: Some("Ops".to_string()),
        profiles: vec![AcpProfile {
            name: "Ops".to_string(),
            system_prompt: Some("legacy prompt".to_string()),
            ..AcpProfile::default()
        }],
        ..AiPreferences::default()
    };

    let profile = resolve_effective_profile(&ai, &ctx);
    assert_eq!(profile.source, AgentChatProfileSource::User);
    assert_eq!(profile.id, "legacy:ops");
    assert_eq!(profile.backend, AgentChatBackend::Acp);
    assert_eq!(profile.system_prompt.as_deref(), Some("legacy prompt"));
}

#[test]
fn selected_acp_agent_and_model_fill_missing_legacy_profile_fields() {
    let ctx = context();
    let ai = AiPreferences {
        selected_acp_agent_id: Some("codex-acp".to_string()),
        selected_model_id: Some("gpt-5.4".to_string()),
        selected_profile_name: Some("Ops".to_string()),
        profiles: vec![AcpProfile {
            name: "Ops".to_string(),
            ..AcpProfile::default()
        }],
        ..AiPreferences::default()
    };

    let profile = resolve_effective_profile(&ai, &ctx);
    assert_eq!(profile.backend, AgentChatBackend::Acp);
    assert_eq!(profile.agent.as_deref(), Some("codex-acp"));
    assert_eq!(profile.model.as_deref(), Some("gpt-5.4"));
}

#[test]
fn selected_model_id_overrides_builtin_profile_default_model() {
    let ctx = context();
    let ai = AiPreferences {
        selected_model_id: Some("gpt-5.5-pro".to_string()),
        ..AiPreferences::default()
    };

    let profile = resolve_effective_profile(&ai, &ctx);
    assert_eq!(profile.id, BUILTIN_GENERAL_PROFILE_ID);
    assert_eq!(profile.model.as_deref(), Some("gpt-5.5-pro"));
}

#[test]
fn unmatched_profile_selection_falls_back_to_general() {
    let ctx = context();
    let ai = AiPreferences {
        selected_profile_id: Some("missing".to_string()),
        selected_profile_name: Some("Also Missing".to_string()),
        ..AiPreferences::default()
    };

    let profile = resolve_effective_profile(&ai, &ctx);
    assert_eq!(profile.id, BUILTIN_GENERAL_PROFILE_ID);
    assert_eq!(profile.backend, AgentChatBackend::Pi);
}
