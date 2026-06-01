use std::fs;
use std::path::{Path, PathBuf};

use script_kit_gpui::ai::agent_chat::pi::launch_spec::PiLaunchSpec;
use script_kit_gpui::ai::agent_chat::profiles::{
    apply_ai_fallbacks, resolve_plugin_profile_entries, AgentChatProfileContext,
    AgentChatProfileSource,
};
use script_kit_gpui::config::AiPreferences;
use script_kit_gpui::plugins::profiles::discover_plugin_profiles_in;
use script_kit_gpui::plugins::{PluginIndex, PluginManifest, PluginRoot};

fn copy_dir(source: &Path, target: &Path) {
    fs::create_dir_all(target).expect("create target dir");
    for entry in fs::read_dir(source).expect("read source dir") {
        let entry = entry.expect("read dir entry");
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir(&source_path, &target_path);
        } else {
            fs::copy(&source_path, &target_path).expect("copy file");
        }
    }
}

fn examples_index(root: &Path) -> PluginIndex {
    PluginIndex {
        plugins: vec![PluginRoot {
            id: "examples".to_string(),
            root: root.to_path_buf(),
            manifest: PluginManifest {
                id: "examples".to_string(),
                title: "Examples".to_string(),
                ..PluginManifest::default()
            },
        }],
    }
}

#[test]
fn shipped_example_profiles_parse_resolve_and_launch_with_supported_pi_flags() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let source_profiles = manifest_dir.join("kit-init").join("profiles");
    let temp = tempfile::tempdir().expect("tempdir");
    let plugin_root = temp.path().join("examples");
    fs::create_dir_all(plugin_root.join("profiles")).expect("create profiles dir");
    copy_dir(&source_profiles, &plugin_root.join("profiles"));

    let profiles = discover_plugin_profiles_in(&examples_index(&plugin_root))
        .expect("discover shipped examples");
    let ids = profiles
        .iter()
        .map(|profile| profile.profile_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        ids,
        vec![
            "ambient-leakage-stress",
            "codebase-scout",
            "docs-researcher",
            "invalid-schema-collision",
            "legacy-agent-import",
            "package-manager-plan-only",
            "plugin-sandbox-builder",
            "profile-builder",
            "project-docs-maintainer",
            "text-polisher"
        ]
    );

    let resolved = resolve_plugin_profile_entries(
        profiles,
        &AgentChatProfileContext {
            kit_path: temp.path().join("kit"),
        },
    );
    assert_eq!(resolved.len(), 10);

    for profile in resolved {
        assert_eq!(profile.source, AgentChatProfileSource::Plugin);
        assert!(profile.id.starts_with("plugin:examples/"));
        assert_eq!(profile.disable_extensions, Some(true));
        assert_eq!(profile.disable_skills, Some(true));
        assert_eq!(profile.disable_prompt_templates, Some(true));
        assert_eq!(profile.disable_context_files, Some(true));
        assert!(
            profile
                .system_prompt
                .as_deref()
                .or(profile.append_system_prompt.as_deref())
                .is_some_and(|prompt| prompt.contains("[Script Kit profile contract]")),
            "plugin profile prompts must include the policy appendix"
        );

        let with_fallbacks = apply_ai_fallbacks(
            profile,
            &AiPreferences {
                pi_binary: Some("/usr/local/bin/pi".to_string()),
                ..AiPreferences::default()
            },
        );
        let argv = PiLaunchSpec::from_profile(&with_fallbacks)
            .expect("pi launch spec")
            .argv();
        assert!(argv.contains(&"--no-extensions".to_string()));
        assert!(argv.contains(&"--no-skills".to_string()));
        assert!(argv.contains(&"--no-prompt-templates".to_string()));
        assert!(argv.contains(&"--no-context-files".to_string()));
        for unsupported in [
            "--profile-id",
            "--profile-name",
            "--path-policy-json",
            "--blocked-action-message",
            "--extension-policy",
            "--hide-cwd-in-prompt",
            "--session-durability",
        ] {
            assert!(!argv.contains(&unsupported.to_string()));
        }
    }
}
