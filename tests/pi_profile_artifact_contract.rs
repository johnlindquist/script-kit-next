use std::fs;
use std::path::{Path, PathBuf};

use script_kit_gpui::ai::agent_chat::pi::launch_spec::PiLaunchSpec;
use script_kit_gpui::ai::agent_chat::profiles::{
    apply_ai_fallbacks, resolve_plugin_profile_entries, AgentChatProfileContext,
    AgentChatProfileSource,
};
use script_kit_gpui::config::{AgentChatBackend, AiPreferences};
use script_kit_gpui::plugins::profiles::discover_plugin_profiles_in;
use script_kit_gpui::plugins::{PluginIndex, PluginManifest, PluginRoot};
use serde_json::json;

fn write_plugin(root: &Path) -> PluginIndex {
    fs::create_dir_all(root).expect("create plugin root");
    fs::write(
        root.join("plugin.json"),
        r#"{"id":"main","title":"Main Plugin"}"#,
    )
    .expect("write plugin manifest");
    PluginIndex {
        plugins: vec![PluginRoot {
            id: "main".to_string(),
            root: root.to_path_buf(),
            manifest: PluginManifest {
                id: "main".to_string(),
                title: "Main Plugin".to_string(),
                ..PluginManifest::default()
            },
        }],
    }
}

fn write_profile(plugin_root: &Path, id: &str, body: serde_json::Value, prompt: &str) -> PathBuf {
    let profile_root = plugin_root.join("profiles").join(id);
    fs::create_dir_all(profile_root.join("examples")).expect("create profile root");
    fs::write(profile_root.join("PROMPT.md"), prompt).expect("write prompt");
    fs::write(profile_root.join("README.md"), "# Codebase Scout\n").expect("write readme");
    fs::write(profile_root.join("examples").join("smoke.json"), "{}\n").expect("write smoke");
    fs::write(
        profile_root.join("profile.json"),
        serde_json::to_string_pretty(&body).expect("serialize profile"),
    )
    .expect("write profile");
    profile_root
}

fn scout_profile(id: &str) -> serde_json::Value {
    json!({
        "schemaVersion": 1,
        "id": id,
        "name": "Codebase Scout",
        "description": "Read-only repo search",
        "iconName": "search",
        "backend": "pi",
        "provider": "openai-codex",
        "model": "gpt-5.5",
        "thinking": "low",
        "prompt": { "mode": "append", "file": "PROMPT.md" },
        "cwd": "~/.scriptkit/agent-chat/profiles/codebase-scout",
        "tools": ["read", "grep", "find", "ls"],
        "toolPolicy": { "allow": ["read", "grep", "find", "ls"] },
        "pathPolicy": {
            "allowRead": ["~/dev/script-kit-gpui"],
            "allowWrite": [],
            "deny": ["~/.ssh", "~/.codex", "~/.scriptkit/secrets"]
        },
        "blockedActionMessage": "This profile is read-only.",
        "disableExtensions": true,
        "disableSkills": true,
        "disablePromptTemplates": true,
        "disableContextFiles": true,
        "hideCwdInPrompt": true,
        "extensionPolicy": "deny",
        "noSession": true,
        "sessionDurability": "sync"
    })
}

#[test]
fn plugin_profile_artifact_resolves_to_pi_launch_spec() {
    let temp = tempfile::tempdir().expect("tempdir");
    let plugin_root = temp.path().join("main");
    let index = write_plugin(&plugin_root);
    write_profile(
        &plugin_root,
        "codebase-scout",
        scout_profile("codebase-scout"),
        "You are Codebase Scout.",
    );

    let plugin_profiles = discover_plugin_profiles_in(&index).expect("discover plugin profiles");
    let ctx = AgentChatProfileContext {
        kit_path: temp.path().join("kit"),
    };
    let resolved = resolve_plugin_profile_entries(plugin_profiles, &ctx);
    assert_eq!(resolved.len(), 1);
    let profile = &resolved[0];
    assert_eq!(profile.id, "plugin:main/codebase-scout");
    assert_eq!(profile.source, AgentChatProfileSource::Plugin);
    assert_eq!(profile.backend, AgentChatBackend::Pi);
    let append_prompt = profile
        .append_system_prompt
        .as_deref()
        .expect("append prompt");
    assert!(append_prompt.starts_with("You are Codebase Scout."));
    assert!(append_prompt.contains("[Script Kit profile contract]"));
    assert!(append_prompt.contains("Allowed tools: read, grep, find, ls"));
    assert!(append_prompt.contains("Denied paths: ~/.ssh, ~/.codex, ~/.scriptkit/secrets"));
    assert_eq!(profile.disable_extensions, Some(true));
    assert_eq!(profile.disable_skills, Some(true));
    assert_eq!(profile.disable_prompt_templates, Some(true));
    assert_eq!(profile.disable_context_files, Some(true));

    let with_fallbacks = apply_ai_fallbacks(
        profile.clone(),
        &AiPreferences {
            pi_binary: Some("/usr/local/bin/pi".to_string()),
            ..AiPreferences::default()
        },
    );
    let spec = PiLaunchSpec::from_profile(&with_fallbacks).expect("launch spec");
    let argv = spec.argv();
    assert!(!argv.contains(&"--profile-id".to_string()));
    assert!(!argv.contains(&"--path-policy-json".to_string()));
    assert!(!argv.contains(&"--blocked-action-message".to_string()));
    assert!(argv
        .windows(2)
        .any(|pair| pair == ["--provider", "openai-codex"]));
    assert!(argv.windows(2).any(|pair| pair == ["--model", "gpt-5.5"]));
    assert!(argv
        .windows(2)
        .any(|pair| pair[0] == "--append-system-prompt"
            && pair[1].starts_with("You are Codebase Scout.")
            && pair[1].contains("[Script Kit profile contract]")));
    assert!(argv
        .windows(2)
        .any(|pair| pair == ["--tools", "read,grep,find,ls"]));
    assert!(argv.contains(&"--no-extensions".to_string()));
    assert!(argv.contains(&"--no-skills".to_string()));
    assert!(argv.contains(&"--no-prompt-templates".to_string()));
    assert!(argv.contains(&"--no-context-files".to_string()));
    assert!(argv.contains(&"--no-session".to_string()));
}

#[test]
fn reserved_plugin_profile_ids_are_not_discovered() {
    let temp = tempfile::tempdir().expect("tempdir");
    let plugin_root = temp.path().join("main");
    let index = write_plugin(&plugin_root);
    write_profile(
        &plugin_root,
        "script-kit",
        scout_profile("script-kit"),
        "You cannot shadow the built-in Script Kit profile.",
    );

    let plugin_profiles = discover_plugin_profiles_in(&index).expect("discover plugin profiles");
    assert!(plugin_profiles.is_empty());
}

#[test]
fn empty_tool_allowlist_launches_with_no_tools() {
    let temp = tempfile::tempdir().expect("tempdir");
    let plugin_root = temp.path().join("main");
    let index = write_plugin(&plugin_root);
    let mut body = scout_profile("no-tools");
    body["tools"] = json!([]);
    body["toolPolicy"] = json!({ "allow": [] });
    body["pathPolicy"] = json!({
        "allowRead": [],
        "allowWrite": [],
        "deny": ["~/.ssh", "~/.codex", "~/.scriptkit/secrets"]
    });
    body["noSession"] = json!(true);
    write_profile(
        &plugin_root,
        "no-tools",
        body,
        "You are a no-tool test profile.",
    );

    let plugin_profiles = discover_plugin_profiles_in(&index).expect("discover plugin profiles");
    let resolved = resolve_plugin_profile_entries(
        plugin_profiles,
        &AgentChatProfileContext {
            kit_path: temp.path().join("kit"),
        },
    );
    assert_eq!(resolved.len(), 1);
    assert_eq!(resolved[0].tools, Some(Vec::new()));

    let with_fallbacks = apply_ai_fallbacks(
        resolved[0].clone(),
        &AiPreferences {
            pi_binary: Some("/usr/local/bin/pi".to_string()),
            ..AiPreferences::default()
        },
    );
    let argv = PiLaunchSpec::from_profile(&with_fallbacks)
        .expect("launch spec")
        .argv();
    assert!(argv.contains(&"--no-tools".to_string()));
    assert!(!argv.contains(&"--tools".to_string()));
}
