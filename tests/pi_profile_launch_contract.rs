use std::path::{Path, PathBuf};

use script_kit_gpui::ai::agent_chat::pi::binary::{
    bundled_pi_binary_candidate_for_exe, dev_pi_binary_for_home,
};
use script_kit_gpui::ai::agent_chat::pi::launch_spec::PiLaunchSpec;
use script_kit_gpui::ai::agent_chat::profiles::{
    agent_chat_profile_picker_entries, persist_agent_chat_profile_selection,
    resolve_effective_profile, selected_agent_chat_profile_picker_id, AgentChatProfileContext,
    AgentChatProfileSource, BUILTIN_GENERAL_PROFILE_ID, BUILTIN_SCRIPT_KIT_PROFILE_ID,
    BUILTIN_TEXT_PROFILE_ID, DEFAULT_PI_MODEL, DEFAULT_PI_PROVIDER, GENERAL_BLOCKED_ACTION_MESSAGE,
    GENERAL_PI_TOOLS, SCRIPT_KIT_PI_TOOLS, TEXT_APPEND_SYSTEM_PROMPT, TEXT_BLOCKED_ACTION_MESSAGE,
};
use script_kit_gpui::config::{AcpProfile, AgentChatBackend, AiPreferences};

fn context() -> AgentChatProfileContext {
    AgentChatProfileContext {
        kit_path: PathBuf::from("/Users/test/.scriptkit"),
    }
}

fn ai_with_pi_binary(path: &str) -> AiPreferences {
    AiPreferences {
        pi_binary: Some(path.to_string()),
        ..AiPreferences::default()
    }
}

#[test]
fn general_builtin_profile_builds_locked_down_pi_rpc_launch_spec() {
    let ctx = context();
    let profile = resolve_effective_profile(&ai_with_pi_binary("/tmp/test-pi"), &ctx);
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
    assert_eq!(spec.pi_binary, PathBuf::from("/tmp/test-pi"));
    assert_eq!(spec.profile_id.as_deref(), Some(BUILTIN_GENERAL_PROFILE_ID));
    assert_eq!(spec.profile_name.as_deref(), Some("General"));
    assert_eq!(spec.cwd.as_deref(), profile.cwd.as_deref());
    assert!(argv.starts_with(&["--mode".to_string(), "rpc".to_string()]));
    assert!(argv.contains(&"--profile-id".to_string()));
    assert!(argv.contains(&BUILTIN_GENERAL_PROFILE_ID.to_string()));
    assert!(argv.contains(&"--profile-name".to_string()));
    assert!(argv.contains(&"General".to_string()));
    assert!(argv.contains(&"--provider".to_string()));
    assert!(argv.contains(&DEFAULT_PI_PROVIDER.to_string()));
    assert!(argv.contains(&"--model".to_string()));
    assert!(argv.contains(&DEFAULT_PI_MODEL.to_string()));
    assert!(argv.contains(&"--append-system-prompt".to_string()));
    assert_eq!(
        profile.tools.as_deref(),
        Some(
            GENERAL_PI_TOOLS
                .iter()
                .map(|tool| tool.to_string())
                .collect::<Vec<_>>()
                .as_slice()
        )
    );
    assert!(argv.contains(&"--tools".to_string()));
    assert!(argv.contains(&GENERAL_PI_TOOLS.join(",")));
    assert!(argv.contains(&"--path-policy-json".to_string()));
    assert!(spec
        .path_policy_json
        .as_deref()
        .is_some_and(|json| { json.contains("allowRead") && json.contains("allowWrite") }));
    assert!(argv.contains(&"--blocked-action-message".to_string()));
    assert!(argv.contains(&GENERAL_BLOCKED_ACTION_MESSAGE.to_string()));
    assert!(!argv.contains(&"--no-tools".to_string()));
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
    let mut ai = ai_with_pi_binary("/tmp/test-pi");
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
fn text_builtin_profile_builds_focused_text_only_pi_rpc_launch_spec() {
    let ctx = context();
    let mut ai = ai_with_pi_binary("/tmp/test-pi");
    ai.selected_profile_id = Some(BUILTIN_TEXT_PROFILE_ID.to_string());

    let profile = resolve_effective_profile(&ai, &ctx);
    assert_eq!(profile.source, AgentChatProfileSource::BuiltIn);
    assert_eq!(profile.id, BUILTIN_TEXT_PROFILE_ID);
    assert_eq!(profile.name, "Text");
    assert_eq!(profile.backend, AgentChatBackend::Pi);
    assert_eq!(profile.provider.as_deref(), Some(DEFAULT_PI_PROVIDER));
    assert_eq!(profile.model.as_deref(), Some(DEFAULT_PI_MODEL));
    assert_eq!(profile.system_prompt, None);
    assert_eq!(
        profile.append_system_prompt.as_deref(),
        Some(TEXT_APPEND_SYSTEM_PROMPT)
    );
    assert_eq!(
        profile.cwd.as_deref(),
        Some(Path::new("/Users/test/.scriptkit/agent-chat/text"))
    );
    assert!(profile.tools.as_ref().is_some_and(|tools| tools.is_empty()));
    assert_eq!(
        profile.blocked_action_message.as_deref(),
        Some(TEXT_BLOCKED_ACTION_MESSAGE)
    );
    assert_eq!(profile.disable_extensions, Some(true));
    assert_eq!(profile.disable_skills, Some(true));
    assert_eq!(profile.disable_prompt_templates, Some(true));
    assert_eq!(profile.hide_cwd_in_prompt, Some(true));
    assert_eq!(profile.extension_policy.as_deref(), Some("deny"));
    assert_eq!(profile.session_dir, None);
    assert_eq!(profile.no_session, Some(true));

    let spec = PiLaunchSpec::from_profile(&profile).expect("text profile should be Pi");
    let argv = spec.argv();
    assert_eq!(spec.pi_binary, PathBuf::from("/tmp/test-pi"));
    assert_eq!(spec.profile_id.as_deref(), Some(BUILTIN_TEXT_PROFILE_ID));
    assert_eq!(spec.profile_name.as_deref(), Some("Text"));
    assert_eq!(spec.cwd.as_deref(), profile.cwd.as_deref());
    assert!(argv.contains(&"--no-tools".to_string()));
    assert!(argv.contains(&"--no-extensions".to_string()));
    assert!(argv.contains(&"--extension-policy".to_string()));
    assert!(argv.contains(&"deny".to_string()));
    assert!(argv.contains(&"--no-skills".to_string()));
    assert!(argv.contains(&"--no-prompt-templates".to_string()));
    assert!(argv.contains(&"--hide-cwd-in-prompt".to_string()));
    assert!(argv.contains(&"--no-session".to_string()));
    assert!(argv.contains(&"--append-system-prompt".to_string()));
    assert!(argv.contains(&TEXT_APPEND_SYSTEM_PROMPT.to_string()));
    assert!(argv.contains(&"--blocked-action-message".to_string()));
    assert!(argv.contains(&TEXT_BLOCKED_ACTION_MESSAGE.to_string()));
    assert!(!argv.contains(&"--tools".to_string()));
    assert!(!argv.contains(&"--agent".to_string()));
    assert!(!argv.contains(&"--system-prompt".to_string()));
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
    assert_eq!(profile.backend, AgentChatBackend::Pi);
    assert_eq!(profile.system_prompt.as_deref(), Some("legacy prompt"));
}

#[test]
fn selected_model_fills_missing_legacy_profile_fields() {
    let ctx = context();
    let ai = AiPreferences {
        selected_model_id: Some("gpt-5.4".to_string()),
        selected_profile_name: Some("Ops".to_string()),
        profiles: vec![AcpProfile {
            name: "Ops".to_string(),
            ..AcpProfile::default()
        }],
        ..AiPreferences::default()
    };

    let profile = resolve_effective_profile(&ai, &ctx);
    assert_eq!(profile.backend, AgentChatBackend::Pi);
    assert_eq!(profile.model.as_deref(), Some("gpt-5.4"));
}

#[test]
fn selected_model_id_overrides_builtin_profile_default_model() {
    let ctx = context();
    let ai = AiPreferences {
        pi_binary: Some("/tmp/test-pi".to_string()),
        selected_model_id: Some("gpt-5.5-pro".to_string()),
        ..AiPreferences::default()
    };

    let profile = resolve_effective_profile(&ai, &ctx);
    assert_eq!(profile.id, BUILTIN_GENERAL_PROFILE_ID);
    assert_eq!(profile.model.as_deref(), Some("gpt-5.5-pro"));
}

#[test]
fn global_pi_binary_preference_overrides_builtin_profile_binary() {
    let ctx = context();
    let ai = AiPreferences {
        pi_binary: Some("~/dev/pi_agent_rust/target/release/pi".to_string()),
        ..AiPreferences::default()
    };

    let profile = resolve_effective_profile(&ai, &ctx);
    let spec = PiLaunchSpec::from_profile(&profile).expect("general profile should be Pi");
    assert!(spec
        .pi_binary
        .ends_with("dev/pi_agent_rust/target/release/pi"));
}

#[test]
fn profile_pi_binary_overrides_global_pi_binary_preference() {
    let ctx = context();
    let ai = AiPreferences {
        pi_binary: Some("/tmp/global-pi".to_string()),
        selected_profile_id: Some("ops".to_string()),
        profiles: vec![AcpProfile {
            id: Some("ops".to_string()),
            name: "Ops".to_string(),
            backend: Some(AgentChatBackend::Pi),
            pi_binary: Some("/tmp/profile-pi".to_string()),
            ..AcpProfile::default()
        }],
        ..AiPreferences::default()
    };

    let profile = resolve_effective_profile(&ai, &ctx);
    let spec = PiLaunchSpec::from_profile(&profile).expect("custom profile should be Pi");
    assert_eq!(spec.pi_binary, PathBuf::from("/tmp/profile-pi"));
}

#[test]
fn unmatched_profile_selection_falls_back_to_general() {
    let ctx = context();
    let ai = AiPreferences {
        pi_binary: Some("/tmp/test-pi".to_string()),
        selected_profile_id: Some("missing".to_string()),
        selected_profile_name: Some("Also Missing".to_string()),
        ..AiPreferences::default()
    };

    let profile = resolve_effective_profile(&ai, &ctx);
    assert_eq!(profile.id, BUILTIN_GENERAL_PROFILE_ID);
    assert_eq!(profile.backend, AgentChatBackend::Pi);
}

#[test]
fn legacy_acp_backend_selection_falls_back_to_general_pi() {
    let ctx = context();
    let ai = AiPreferences {
        pi_binary: Some("/tmp/test-pi".to_string()),
        selected_model_id: Some("claude-sonnet-4-6".to_string()),
        ..AiPreferences::default()
    };

    let profile = resolve_effective_profile(&ai, &ctx);
    assert_eq!(profile.id, BUILTIN_GENERAL_PROFILE_ID);
    assert_eq!(profile.backend, AgentChatBackend::Pi);
    assert_eq!(profile.model.as_deref(), Some("claude-sonnet-4-6"));
}

#[test]
fn selected_profile_id_still_beats_selected_backend() {
    let ctx = context();
    let ai = AiPreferences {
        pi_binary: Some("/tmp/test-pi".to_string()),
        selected_backend: Some(AgentChatBackend::Pi),
        selected_profile_id: Some(BUILTIN_SCRIPT_KIT_PROFILE_ID.to_string()),
        ..AiPreferences::default()
    };

    let profile = resolve_effective_profile(&ai, &ctx);
    assert_eq!(profile.id, BUILTIN_SCRIPT_KIT_PROFILE_ID);
    assert_eq!(profile.backend, AgentChatBackend::Pi);
}

#[test]
fn profile_picker_lists_builtin_and_custom_profiles() {
    let ctx = context();
    let ai = AiPreferences {
        profiles: vec![AcpProfile {
            id: Some("ops".to_string()),
            name: "Ops".to_string(),
            backend: Some(AgentChatBackend::Pi),
            ..AcpProfile::default()
        }],
        ..AiPreferences::default()
    };

    let entries = agent_chat_profile_picker_entries(&ai, &ctx);
    let ids = entries
        .iter()
        .map(|entry| entry.id.as_str())
        .collect::<Vec<_>>();
    assert!(ids.contains(&BUILTIN_GENERAL_PROFILE_ID));
    assert!(ids.contains(&BUILTIN_TEXT_PROFILE_ID));
    assert!(ids.contains(&BUILTIN_SCRIPT_KIT_PROFILE_ID));
    assert!(ids.contains(&"ops"));
    assert!(entries
        .iter()
        .all(|entry| entry.backend == AgentChatBackend::Pi));
}

#[test]
fn persisting_builtin_profile_uses_stable_profile_id_and_backend() {
    let ctx = context();
    let mut ai = AiPreferences::default();
    let entry = persist_agent_chat_profile_selection(&mut ai, BUILTIN_SCRIPT_KIT_PROFILE_ID, &ctx)
        .expect("script-kit profile should exist");

    assert_eq!(entry.name, "Script Kit");
    assert_eq!(
        ai.selected_profile_id.as_deref(),
        Some(BUILTIN_SCRIPT_KIT_PROFILE_ID)
    );
    assert_eq!(ai.selected_profile_name, None);
    assert_eq!(ai.selected_backend, Some(AgentChatBackend::Pi));
    assert_eq!(
        selected_agent_chat_profile_picker_id(&ai, &ctx),
        BUILTIN_SCRIPT_KIT_PROFILE_ID
    );
}

#[test]
fn provider_scoped_pi_model_selection_splits_provider_and_model_for_launch() {
    let ctx = context();
    let ai = AiPreferences {
        pi_binary: Some("/tmp/test-pi".to_string()),
        selected_profile_id: Some(BUILTIN_SCRIPT_KIT_PROFILE_ID.to_string()),
        selected_model_id: Some("anthropic:claude-3-7-sonnet".to_string()),
        ..AiPreferences::default()
    };

    let profile = resolve_effective_profile(&ai, &ctx);
    assert_eq!(profile.provider.as_deref(), Some("anthropic"));
    assert_eq!(profile.model.as_deref(), Some("claude-3-7-sonnet"));

    let spec = PiLaunchSpec::from_profile(&profile).expect("script-kit profile should be Pi");
    let argv = spec.argv();
    assert!(argv
        .windows(2)
        .any(|pair| pair == ["--provider", "anthropic"]));
    assert!(argv
        .windows(2)
        .any(|pair| pair == ["--model", "claude-3-7-sonnet"]));
    assert!(!argv.contains(&"anthropic:claude-3-7-sonnet".to_string()));
}

#[test]
fn profile_picker_skips_custom_profiles_that_collide_with_builtins() {
    let ctx = context();
    let ai = AiPreferences {
        profiles: vec![AcpProfile {
            id: Some(BUILTIN_GENERAL_PROFILE_ID.to_string()),
            name: "Shadow General".to_string(),
            backend: Some(AgentChatBackend::Pi),
            ..AcpProfile::default()
        }],
        ..AiPreferences::default()
    };

    let general_entries = agent_chat_profile_picker_entries(&ai, &ctx)
        .into_iter()
        .filter(|entry| entry.id == BUILTIN_GENERAL_PROFILE_ID)
        .collect::<Vec<_>>();
    assert_eq!(general_entries.len(), 1);
    assert_eq!(general_entries[0].name, "General");
    assert_eq!(general_entries[0].backend, AgentChatBackend::Pi);
}

#[test]
fn bundled_pi_binary_candidate_resolves_next_to_macos_executable() {
    let exe = Path::new("/Applications/Script Kit.app/Contents/MacOS/script-kit-gpui");

    assert_eq!(
        bundled_pi_binary_candidate_for_exe(exe),
        Some(PathBuf::from(
            "/Applications/Script Kit.app/Contents/MacOS/pi"
        ))
    );
}

#[test]
fn local_pi_rust_path_is_only_an_existing_executable_dev_fallback() {
    assert_eq!(dev_pi_binary_for_home(Some(Path::new("/Users/test"))), None);
    assert_eq!(
        bundled_pi_binary_candidate_for_exe(Path::new("/tmp/script-kit-gpui")),
        None
    );
}
