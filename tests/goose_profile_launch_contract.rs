//! Agent Chat profile resolution contract: which profile, provider, model,
//! reasoning level, tools, and engine binary a launch resolves to.

use std::path::{Path, PathBuf};

use script_kit_gpui::ai::agent_chat::goose::binary::bundled_goose_binary_candidate_for_exe;
use script_kit_gpui::ai::agent_chat::profiles::{
    advanced_provider_model_catalog, agent_chat_profile_picker_entries,
    persist_agent_chat_profile_selection, provider_model_catalog, resolve_effective_profile,
    selected_agent_chat_profile_picker_id, AgentChatProfileContext, AgentChatProfileSource,
    BUILTIN_BRAIN_PROFILE_ID, BUILTIN_GENERAL_PROFILE_ID, BUILTIN_SCRIPT_KIT_PROFILE_ID,
    BUILTIN_TEXT_PROFILE_ID, DEFAULT_MODEL, DEFAULT_PROVIDER, DEFAULT_THINKING, GENERAL_TOOLS,
    SCRIPT_KIT_TOOLS, TEXT_APPEND_SYSTEM_PROMPT, TEXT_BLOCKED_ACTION_MESSAGE, TEXT_MODEL,
};
use script_kit_gpui::config::{AgentChatBackend, AgentChatProfile, AiPreferences};

fn context() -> AgentChatProfileContext {
    AgentChatProfileContext {
        kit_path: PathBuf::from("/Users/test/.scriptkit"),
    }
}

fn ai_with_goose_binary(path: &str) -> AiPreferences {
    AiPreferences {
        goose_binary: Some(path.to_string()),
        ..AiPreferences::default()
    }
}

#[test]
fn general_builtin_profile_resolves_codex_defaults_and_general_tools() {
    let ctx = context();
    // Brain is the no-selection default; select General explicitly.
    let mut ai = ai_with_goose_binary("/tmp/test-goose");
    ai.selected_profile_id = Some(BUILTIN_GENERAL_PROFILE_ID.to_string());
    let profile = resolve_effective_profile(&ai, &ctx);
    assert_eq!(profile.source, AgentChatProfileSource::BuiltIn);
    assert_eq!(profile.id, BUILTIN_GENERAL_PROFILE_ID);
    assert_eq!(profile.name, "General");
    assert_eq!(profile.backend, AgentChatBackend::Goose);
    assert_eq!(profile.provider.as_deref(), Some(DEFAULT_PROVIDER));
    assert_eq!(profile.model.as_deref(), Some(DEFAULT_MODEL));
    assert_eq!(
        profile.cwd.as_deref(),
        Some(Path::new("/Users/test/.scriptkit/agent-chat/general"))
    );

    assert_eq!(profile.goose_binary, Some(PathBuf::from("/tmp/test-goose")));
    assert_eq!(profile.thinking.as_deref(), Some(DEFAULT_THINKING));
    assert_eq!(
        profile.tools.as_deref(),
        Some(
            GENERAL_TOOLS
                .iter()
                .map(|tool| tool.to_string())
                .collect::<Vec<_>>()
                .as_slice()
        )
    );
}

#[test]
fn script_kit_builtin_profile_resolves_the_kit_workspace() {
    let ctx = context();
    let mut ai = ai_with_goose_binary("/tmp/test-goose");
    ai.selected_profile_id = Some(BUILTIN_SCRIPT_KIT_PROFILE_ID.to_string());

    let profile = resolve_effective_profile(&ai, &ctx);
    assert_eq!(profile.id, BUILTIN_SCRIPT_KIT_PROFILE_ID);
    assert_eq!(profile.name, "Script Kit");
    assert_eq!(profile.backend, AgentChatBackend::Goose);
    assert_eq!(
        profile.cwd.as_deref(),
        Some(Path::new("/Users/test/.scriptkit"))
    );
    assert_eq!(
        profile.tools.as_deref(),
        Some(
            SCRIPT_KIT_TOOLS
                .iter()
                .map(|tool| tool.to_string())
                .collect::<Vec<_>>()
                .as_slice()
        )
    );
    assert_eq!(profile.hide_cwd_in_prompt, Some(false));
    assert_eq!(profile.thinking.as_deref(), Some(DEFAULT_THINKING));
}

#[test]
fn text_builtin_profile_is_web_search_only_and_sessionless() {
    let ctx = context();
    let mut ai = ai_with_goose_binary("/tmp/test-goose");
    ai.selected_profile_id = Some(BUILTIN_TEXT_PROFILE_ID.to_string());

    let profile = resolve_effective_profile(&ai, &ctx);
    assert_eq!(profile.source, AgentChatProfileSource::BuiltIn);
    assert_eq!(profile.id, BUILTIN_TEXT_PROFILE_ID);
    assert_eq!(profile.name, "Text");
    assert_eq!(profile.backend, AgentChatBackend::Goose);
    assert_eq!(profile.provider.as_deref(), Some(DEFAULT_PROVIDER));
    // Pinned to the fastest Codex model for the instant rewrite flow.
    assert_eq!(profile.model.as_deref(), Some(TEXT_MODEL));
    assert_eq!(profile.system_prompt, None);
    assert_eq!(
        profile.append_system_prompt.as_deref(),
        Some(TEXT_APPEND_SYSTEM_PROMPT)
    );
    assert_eq!(
        profile.cwd.as_deref(),
        Some(Path::new("/Users/test/.scriptkit/agent-chat/text"))
    );
    // Exactly one read-only network tool: web_search (live-info questions).
    assert_eq!(profile.tools, Some(vec!["web_search".to_string()]));
    assert_eq!(
        profile.tool_policy.as_ref().and_then(|p| p.allow.clone()),
        Some(vec!["web_search".to_string()])
    );
    assert_eq!(
        profile.blocked_action_message.as_deref(),
        Some(TEXT_BLOCKED_ACTION_MESSAGE)
    );
    assert_eq!(profile.disable_extensions, Some(true));
    assert_eq!(profile.disable_skills, Some(true));
    assert_eq!(profile.disable_prompt_templates, Some(true));
    assert_eq!(profile.disable_context_files, Some(true));
    assert_eq!(profile.hide_cwd_in_prompt, Some(true));
    assert_eq!(profile.extension_policy.as_deref(), Some("deny"));
    assert_eq!(profile.session_dir, None);
    assert_eq!(profile.no_session, Some(true));
    assert_eq!(profile.thinking, None);
}

#[test]
fn built_in_non_speed_profiles_launch_gpt_6_luna_with_medium_thinking() {
    let ctx = context();

    for profile_id in [
        BUILTIN_BRAIN_PROFILE_ID,
        BUILTIN_GENERAL_PROFILE_ID,
        BUILTIN_SCRIPT_KIT_PROFILE_ID,
    ] {
        let ai = AiPreferences {
            goose_binary: Some("/tmp/test-goose".to_string()),
            selected_profile_id: Some(profile_id.to_string()),
            ..AiPreferences::default()
        };
        let profile = resolve_effective_profile(&ai, &ctx);
        assert_eq!(profile.provider.as_deref(), Some("chatgpt_codex"));
        assert_eq!(profile.model.as_deref(), Some("gpt-6-luna"));
        assert_eq!(profile.thinking.as_deref(), Some("medium"));
    }
}

#[test]
fn codex_catalog_models_default_missing_thinking_to_medium() {
    let ctx = context();

    for model in ["gpt-6-sol", "gpt-5.6-sol", "gpt-5.6-terra", "gpt-5.6-luna"] {
        let ai = AiPreferences {
            goose_binary: Some("/tmp/test-goose".to_string()),
            selected_profile_id: Some("custom".to_string()),
            selected_model_id: Some(format!("chatgpt_codex/{model}")),
            profiles: vec![AgentChatProfile {
                id: Some("custom".to_string()),
                name: "Custom".to_string(),
                backend: Some(AgentChatBackend::Goose),
                provider: Some("chatgpt_codex".to_string()),
                model: Some("gpt-5.4".to_string()),
                ..AgentChatProfile::default()
            }],
            ..AiPreferences::default()
        };
        let profile = resolve_effective_profile(&ai, &ctx);
        assert_eq!(profile.model.as_deref(), Some(model));
        assert_eq!(profile.thinking.as_deref(), Some("medium"));
    }
}

#[test]
fn codex_catalog_models_preserve_explicit_thinking() {
    let ctx = context();
    let ai = AiPreferences {
        goose_binary: Some("/tmp/test-goose".to_string()),
        selected_profile_id: Some("custom".to_string()),
        profiles: vec![AgentChatProfile {
            id: Some("custom".to_string()),
            name: "Custom".to_string(),
            backend: Some(AgentChatBackend::Goose),
            provider: Some("chatgpt_codex".to_string()),
            model: Some("gpt-5.6-luna".to_string()),
            thinking: Some("high".to_string()),
            ..AgentChatProfile::default()
        }],
        ..AiPreferences::default()
    };
    let profile = resolve_effective_profile(&ai, &ctx);
    assert_eq!(profile.thinking.as_deref(), Some("high"));
}

#[test]
fn provider_catalog_is_codex_first_with_other_providers_in_advanced() {
    let catalog = provider_model_catalog();
    assert_eq!(
        catalog.iter().map(|entry| entry.id).collect::<Vec<_>>(),
        vec!["chatgpt_codex"],
        "the primary picker offers only the Codex subscription"
    );
    let codex = &catalog[0];
    assert_eq!(codex.display_name, "Codex");
    assert_eq!(
        codex.models.first().copied(),
        Some(("gpt-6-sol", "GPT-6 SOL"))
    );
    assert!(codex.models.contains(&("gpt-5.6-sol", "GPT-5.6 SOL")));

    let advanced: Vec<_> = advanced_provider_model_catalog()
        .into_iter()
        .map(|entry| entry.id)
        .collect();
    for provider in ["anthropic", "google", "ollama"] {
        assert!(
            advanced.contains(&provider),
            "missing advanced provider {provider}"
        );
    }
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
fn selected_profile_name_resolves_legacy_agent_chat_profile() {
    let ctx = context();
    let ai = AiPreferences {
        selected_profile_name: Some("Ops".to_string()),
        profiles: vec![AgentChatProfile {
            name: "Ops".to_string(),
            system_prompt: Some("legacy prompt".to_string()),
            ..AgentChatProfile::default()
        }],
        ..AiPreferences::default()
    };

    let profile = resolve_effective_profile(&ai, &ctx);
    assert_eq!(profile.source, AgentChatProfileSource::User);
    assert_eq!(profile.id, "legacy:ops");
    assert_eq!(profile.backend, AgentChatBackend::Goose);
    assert_eq!(profile.system_prompt.as_deref(), Some("legacy prompt"));
}

#[test]
fn selected_model_fills_missing_legacy_profile_fields() {
    let ctx = context();
    let ai = AiPreferences {
        selected_model_id: Some("gpt-5.4".to_string()),
        selected_profile_name: Some("Ops".to_string()),
        profiles: vec![AgentChatProfile {
            name: "Ops".to_string(),
            ..AgentChatProfile::default()
        }],
        ..AiPreferences::default()
    };

    let profile = resolve_effective_profile(&ai, &ctx);
    assert_eq!(profile.backend, AgentChatBackend::Goose);
    assert_eq!(profile.model.as_deref(), Some("gpt-5.4"));
}

#[test]
fn selected_model_id_overrides_builtin_profile_default_model() {
    let ctx = context();
    let ai = AiPreferences {
        goose_binary: Some("/tmp/test-goose".to_string()),
        selected_profile_id: Some(BUILTIN_GENERAL_PROFILE_ID.to_string()),
        selected_model_id: Some("chatgpt_codex/gpt-5.6-terra".to_string()),
        ..AiPreferences::default()
    };

    let profile = resolve_effective_profile(&ai, &ctx);
    assert_eq!(profile.id, BUILTIN_GENERAL_PROFILE_ID);
    assert_eq!(profile.model.as_deref(), Some("gpt-5.6-terra"));
    assert_eq!(profile.thinking.as_deref(), Some("medium"));
}

#[test]
fn global_goose_binary_preference_applies_to_builtin_profiles() {
    let ctx = context();
    let ai = ai_with_goose_binary("~/bin/goose");

    let profile = resolve_effective_profile(&ai, &ctx);
    assert!(profile
        .goose_binary
        .as_deref()
        .is_some_and(|binary| binary.ends_with("bin/goose")));
}

#[test]
fn profile_goose_binary_overrides_global_goose_binary_preference() {
    let ctx = context();
    let ai = AiPreferences {
        goose_binary: Some("/tmp/global-goose".to_string()),
        selected_profile_id: Some("ops".to_string()),
        profiles: vec![AgentChatProfile {
            id: Some("ops".to_string()),
            name: "Ops".to_string(),
            backend: Some(AgentChatBackend::Goose),
            goose_binary: Some("/tmp/profile-goose".to_string()),
            ..AgentChatProfile::default()
        }],
        ..AiPreferences::default()
    };

    let profile = resolve_effective_profile(&ai, &ctx);
    assert_eq!(
        profile.goose_binary,
        Some(PathBuf::from("/tmp/profile-goose"))
    );
}

#[test]
fn unmatched_profile_selection_falls_back_to_brain() {
    let ctx = context();
    let ai = AiPreferences {
        goose_binary: Some("/tmp/test-goose".to_string()),
        selected_profile_id: Some("missing".to_string()),
        selected_profile_name: Some("Also Missing".to_string()),
        ..AiPreferences::default()
    };

    // Brain is the memory-aware default when no selection resolves.
    let profile = resolve_effective_profile(&ai, &ctx);
    assert_eq!(profile.id, BUILTIN_BRAIN_PROFILE_ID);
    assert_eq!(profile.backend, AgentChatBackend::Goose);
}

#[test]
fn legacy_agent_chat_backend_selection_falls_back_to_brain() {
    let ctx = context();
    let ai = AiPreferences {
        goose_binary: Some("/tmp/test-goose".to_string()),
        selected_model_id: Some("claude-sonnet-4-6".to_string()),
        ..AiPreferences::default()
    };

    // Brain is the memory-aware default when no selection resolves.
    let profile = resolve_effective_profile(&ai, &ctx);
    assert_eq!(profile.id, BUILTIN_BRAIN_PROFILE_ID);
    assert_eq!(profile.backend, AgentChatBackend::Goose);
    assert_eq!(profile.model.as_deref(), Some("claude-sonnet-4-6"));
}

#[test]
fn selected_profile_id_still_beats_selected_backend() {
    let ctx = context();
    let ai = AiPreferences {
        goose_binary: Some("/tmp/test-goose".to_string()),
        selected_backend: Some(AgentChatBackend::Goose),
        selected_profile_id: Some(BUILTIN_SCRIPT_KIT_PROFILE_ID.to_string()),
        ..AiPreferences::default()
    };

    let profile = resolve_effective_profile(&ai, &ctx);
    assert_eq!(profile.id, BUILTIN_SCRIPT_KIT_PROFILE_ID);
    assert_eq!(profile.backend, AgentChatBackend::Goose);
}

#[test]
fn profile_picker_lists_builtin_and_custom_profiles() {
    let ctx = context();
    let ai = AiPreferences {
        profiles: vec![AgentChatProfile {
            id: Some("ops".to_string()),
            name: "Ops".to_string(),
            backend: Some(AgentChatBackend::Goose),
            ..AgentChatProfile::default()
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
        .all(|entry| entry.backend == AgentChatBackend::Goose));
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
    assert_eq!(ai.selected_backend, Some(AgentChatBackend::Goose));
    assert_eq!(
        selected_agent_chat_profile_picker_id(&ai, &ctx),
        BUILTIN_SCRIPT_KIT_PROFILE_ID
    );
}

#[test]
fn provider_scoped_model_selection_splits_provider_and_model() {
    let ctx = context();
    let ai = AiPreferences {
        goose_binary: Some("/tmp/test-goose".to_string()),
        selected_profile_id: Some(BUILTIN_SCRIPT_KIT_PROFILE_ID.to_string()),
        selected_model_id: Some("anthropic:claude-3-7-sonnet".to_string()),
        ..AiPreferences::default()
    };

    let profile = resolve_effective_profile(&ai, &ctx);
    assert_eq!(profile.provider.as_deref(), Some("anthropic"));
    assert_eq!(profile.model.as_deref(), Some("claude-3-7-sonnet"));
}
#[test]
fn profile_picker_skips_custom_profiles_that_collide_with_builtins() {
    let ctx = context();
    let ai = AiPreferences {
        profiles: vec![AgentChatProfile {
            id: Some(BUILTIN_GENERAL_PROFILE_ID.to_string()),
            name: "Shadow General".to_string(),
            backend: Some(AgentChatBackend::Goose),
            ..AgentChatProfile::default()
        }],
        ..AiPreferences::default()
    };

    let general_entries = agent_chat_profile_picker_entries(&ai, &ctx)
        .into_iter()
        .filter(|entry| entry.id == BUILTIN_GENERAL_PROFILE_ID)
        .collect::<Vec<_>>();
    assert_eq!(general_entries.len(), 1);
    assert_eq!(general_entries[0].name, "General");
    assert_eq!(general_entries[0].backend, AgentChatBackend::Goose);
}

#[test]
fn bundled_goose_binary_candidate_resolves_next_to_macos_executable() {
    let exe = Path::new("/Applications/Script Kit.app/Contents/MacOS/script-kit-gpui");

    assert_eq!(
        bundled_goose_binary_candidate_for_exe(exe),
        Some(PathBuf::from(
            "/Applications/Script Kit.app/Contents/MacOS/goose"
        ))
    );
    assert_eq!(
        bundled_goose_binary_candidate_for_exe(Path::new("/tmp/script-kit-gpui")),
        None
    );
}
