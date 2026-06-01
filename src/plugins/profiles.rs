use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use tracing::warn;

use crate::config::{AgentChatBackend, AgentChatPathPolicyConfig, AgentChatToolPolicyConfig};

use super::types::PluginIndex;

const PROFILE_MANIFEST_FILE: &str = "profile.json";
const BUILTIN_PROFILE_IDS: [&str; 3] = ["general", "script-kit", "text"];

#[derive(Debug, Clone, PartialEq)]
pub struct PluginProfile {
    pub plugin_id: String,
    pub plugin_title: String,
    pub profile_id: String,
    pub root: PathBuf,
    pub manifest_path: PathBuf,
    pub artifact: AgentProfileArtifactV1,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentProfileArtifactV1 {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub icon_name: Option<String>,
    #[serde(default)]
    pub backend: Option<AgentChatBackend>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub thinking: Option<String>,
    pub prompt: ProfilePromptSpec,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub tools: Option<Vec<String>>,
    #[serde(default)]
    pub tool_policy: Option<AgentChatToolPolicyConfig>,
    pub path_policy: AgentChatPathPolicyConfig,
    #[serde(default)]
    pub blocked_action_message: Option<String>,
    #[serde(default)]
    pub disable_extensions: Option<bool>,
    #[serde(default)]
    pub disable_skills: Option<bool>,
    #[serde(default)]
    pub disable_prompt_templates: Option<bool>,
    #[serde(default)]
    pub disable_context_files: Option<bool>,
    #[serde(default)]
    pub hide_cwd_in_prompt: Option<bool>,
    #[serde(default)]
    pub extension_policy: Option<String>,
    #[serde(default)]
    pub session_dir: Option<String>,
    #[serde(default)]
    pub no_session: Option<bool>,
    #[serde(default)]
    pub session_durability: Option<String>,
    #[serde(default)]
    pub examples: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfilePromptSpec {
    pub mode: ProfilePromptMode,
    pub file: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ProfilePromptMode {
    Replace,
    Append,
}

pub fn discover_plugin_profiles() -> Result<Vec<PluginProfile>> {
    let index = super::discovery::discover_plugins()?;
    discover_plugin_profiles_in(&index)
}

pub fn discover_plugin_profiles_in(index: &PluginIndex) -> Result<Vec<PluginProfile>> {
    let mut profiles = Vec::new();

    for plugin in &index.plugins {
        let profiles_dir = plugin.root.join("profiles");
        if !profiles_dir.exists() {
            continue;
        }

        let entries = fs::read_dir(&profiles_dir).with_context(|| {
            format!(
                "Failed to read profiles dir for plugin {}: {}",
                plugin.id,
                profiles_dir.display()
            )
        })?;

        for entry in entries.flatten() {
            let root = entry.path();
            if !root.is_dir() {
                continue;
            }

            let manifest_path = root.join(PROFILE_MANIFEST_FILE);
            if !manifest_path.exists() {
                continue;
            }

            match parse_profile_artifact(&manifest_path)
                .and_then(|artifact| validate_artifact(&root, &artifact).map(|_| artifact))
            {
                Ok(artifact) => {
                    profiles.push(PluginProfile {
                        plugin_id: plugin.id.clone(),
                        plugin_title: if plugin.manifest.title.is_empty() {
                            plugin.id.clone()
                        } else {
                            plugin.manifest.title.clone()
                        },
                        profile_id: artifact.id.clone(),
                        root,
                        manifest_path,
                        artifact,
                    });
                }
                Err(error) => {
                    warn!(
                        plugin_id = %plugin.id,
                        path = %manifest_path.display(),
                        error = %error,
                        "plugin_profile_invalid_skipped"
                    );
                }
            }
        }
    }

    profiles.sort_by(|a, b| {
        a.plugin_id
            .cmp(&b.plugin_id)
            .then_with(|| a.profile_id.cmp(&b.profile_id))
    });
    Ok(profiles)
}

pub fn parse_profile_artifact(path: &Path) -> Result<AgentProfileArtifactV1> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("Failed to read profile artifact: {}", path.display()))?;
    let artifact: AgentProfileArtifactV1 = serde_json::from_str(&text)
        .with_context(|| format!("Failed to parse profile artifact: {}", path.display()))?;
    Ok(artifact)
}

pub fn prompt_file_text(profile: &PluginProfile) -> Result<String> {
    let relative = safe_relative_prompt_path(&profile.artifact.prompt.file)?;
    let path = profile.root.join(relative);
    fs::read_to_string(&path)
        .with_context(|| format!("Failed to read profile prompt file: {}", path.display()))
}

pub fn validate_profile(profile: &PluginProfile) -> Result<()> {
    validate_artifact(&profile.root, &profile.artifact)
}

fn validate_artifact(root: &Path, artifact: &AgentProfileArtifactV1) -> Result<()> {
    if artifact.schema_version != 1 {
        bail!(
            "unsupported profile schemaVersion {}",
            artifact.schema_version
        );
    }

    ensure_slug(&artifact.id)?;
    if root
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name != artifact.id)
    {
        bail!(
            "profile directory name must match profile id '{}'",
            artifact.id
        );
    }
    if BUILTIN_PROFILE_IDS.contains(&artifact.id.as_str()) {
        bail!(
            "profile id '{}' is reserved by a built-in profile",
            artifact.id
        );
    }

    if artifact.name.trim().is_empty() {
        bail!("profile name is required");
    }

    let prompt_path = safe_relative_prompt_path(&artifact.prompt.file)?;
    let prompt_text = fs::read_to_string(root.join(prompt_path))
        .context("profile prompt file is required and must be readable")?;
    let prompt_text = prompt_text.trim();
    if prompt_text.is_empty() {
        bail!("profile prompt file must not be empty");
    }
    if prompt_text.len() > 16 * 1024 {
        bail!("profile prompt file must be <= 16k characters");
    }

    require_support_file(root, "README.md")?;
    require_json_file(root, "examples/smoke.json")?;

    let Some(tools) = resolved_artifact_tools(artifact) else {
        bail!("profile artifacts require explicit toolPolicy.allow");
    };
    let uses_mutation = tools.iter().any(|tool| {
        matches!(
            tool.trim().to_ascii_lowercase().as_str(),
            "create_file" | "write" | "edit" | "bash" | "hashline_edit"
        )
    });
    if tools
        .iter()
        .any(|tool| tool.trim().eq_ignore_ascii_case("bash"))
    {
        bail!("bash is not supported in schema v1 profile artifacts");
    }
    let uses_filesystem = tools.iter().any(|tool| {
        matches!(
            tool.trim().to_ascii_lowercase().as_str(),
            "read"
                | "create_file"
                | "write"
                | "edit"
                | "bash"
                | "hashline_edit"
                | "grep"
                | "find"
                | "ls"
        )
    });
    let allow_read = artifact.path_policy.allow_read.as_deref().unwrap_or(&[]);
    for path in allow_read {
        reject_broad_or_secret_path(path, "allowRead")?;
    }
    let allow_write = artifact.path_policy.allow_write.as_deref().unwrap_or(&[]);
    if uses_mutation && allow_write.is_empty() {
        bail!("mutation tools require a non-empty pathPolicy.allowWrite");
    }
    for path in allow_write {
        reject_broad_or_secret_path(path, "allowWrite")?;
    }
    if uses_filesystem {
        require_protected_denies(artifact.path_policy.deny.as_deref().unwrap_or(&[]))?;
    }

    if artifact.disable_extensions == Some(false) {
        bail!("profile artifacts cannot enable extensions in schema v1");
    }
    if artifact.disable_skills == Some(false) {
        bail!("profile artifacts cannot enable ambient skills in schema v1");
    }
    if artifact.disable_prompt_templates == Some(false) {
        bail!("profile artifacts cannot enable prompt templates in schema v1");
    }
    if artifact.disable_context_files == Some(false) {
        bail!("profile artifacts cannot enable ambient context files in schema v1");
    }
    if artifact.no_session == Some(true)
        && artifact
            .session_dir
            .as_deref()
            .is_some_and(|s| !s.trim().is_empty())
    {
        bail!("noSession profiles must not set sessionDir");
    }

    Ok(())
}

fn require_support_file(root: &Path, relative: &str) -> Result<()> {
    let text = fs::read_to_string(root.join(relative))
        .with_context(|| format!("profile {relative} is required and must be readable"))?;
    if text.trim().is_empty() {
        bail!("profile {relative} must not be empty");
    }
    Ok(())
}

fn require_json_file(root: &Path, relative: &str) -> Result<()> {
    let text = fs::read_to_string(root.join(relative))
        .with_context(|| format!("profile {relative} is required and must be readable"))?;
    serde_json::from_str::<serde_json::Value>(&text)
        .with_context(|| format!("profile {relative} must be valid JSON"))?;
    Ok(())
}

pub fn resolved_artifact_tools(artifact: &AgentProfileArtifactV1) -> Option<Vec<String>> {
    artifact
        .tool_policy
        .as_ref()
        .and_then(|policy| policy.allow.as_ref())
        .map(|tools| clean_list(tools))
}

fn clean_list(values: &[String]) -> Vec<String> {
    values
        .iter()
        .filter_map(|value| {
            let value = value.trim();
            (!value.is_empty()).then(|| value.to_string())
        })
        .collect()
}

fn ensure_slug(id: &str) -> Result<()> {
    let id = id.trim();
    if !(2..=63).contains(&id.len()) {
        bail!("profile id must be 2-63 characters");
    }
    let mut chars = id.chars();
    let Some(first) = chars.next() else {
        bail!("profile id is required");
    };
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        bail!("profile id must start with a lowercase letter or digit");
    }
    if !chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-') {
        bail!("profile id must contain only lowercase letters, digits, and hyphens");
    }
    Ok(())
}

fn safe_relative_prompt_path(path: &str) -> Result<&Path> {
    let path = Path::new(path.trim());
    if path.as_os_str().is_empty() || path.is_absolute() {
        bail!("prompt.file must be a relative path");
    }
    if path.components().any(|component| {
        matches!(
            component,
            std::path::Component::ParentDir | std::path::Component::Prefix(_)
        )
    }) {
        bail!("prompt.file must not escape the profile directory");
    }
    Ok(path)
}

fn reject_broad_or_secret_path(path: &str, field: &str) -> Result<()> {
    let normalized = normalize_policy_path(path);
    let forbidden_exact = ["/", "~", "~/.scriptkit", "~/Desktop", "~/Documents"];
    if forbidden_exact.contains(&normalized.as_str()) {
        bail!("{field} '{}' is too broad", path);
    }
    let forbidden_prefixes = [
        "~/.ssh",
        "~/.codex",
        "~/.scriptkit/secrets",
        "~/.zshrc",
        "~/.bashrc",
        "~/.config",
        "~/Library/Application Support",
    ];
    if forbidden_prefixes
        .iter()
        .any(|prefix| normalized == *prefix || normalized.starts_with(&format!("{prefix}/")))
    {
        bail!("{field} '{}' targets a protected path", path);
    }
    Ok(())
}

fn require_protected_denies(paths: &[String]) -> Result<()> {
    let normalized = paths
        .iter()
        .map(|path| normalize_policy_path(path))
        .collect::<Vec<_>>();
    for required in ["~/.ssh", "~/.codex", "~/.scriptkit/secrets"] {
        if !normalized
            .iter()
            .any(|path| path == required || path.starts_with(&format!("{required}/")))
        {
            bail!("filesystem profiles must deny protected path {required}");
        }
    }
    Ok(())
}

fn normalize_policy_path(path: &str) -> String {
    let mut normalized = path.trim().replace('\\', "/");
    while normalized.len() > 1 && normalized.ends_with('/') {
        normalized.pop();
    }
    normalized
}

#[cfg(test)]
mod tests {
    use std::fs;

    use serde_json::json;

    use super::*;
    use crate::plugins::manifest::read_plugin_manifest;
    use crate::plugins::types::{PluginIndex, PluginManifest, PluginRoot};

    fn write_profile(root: &Path, id: &str, overrides: serde_json::Value) -> PathBuf {
        let profile_root = root.join("profiles").join(id);
        fs::create_dir_all(profile_root.join("examples")).expect("create profile root");
        fs::write(
            profile_root.join("PROMPT.md"),
            "You are a narrow test profile.",
        )
        .expect("write prompt");
        fs::write(profile_root.join("README.md"), "# Test Profile\n").expect("write readme");
        fs::write(profile_root.join("examples").join("smoke.json"), "{}\n").expect("write smoke");
        let mut value = json!({
            "schemaVersion": 1,
            "id": id,
            "name": "Test Profile",
            "backend": "pi",
            "provider": "openai-codex",
            "model": "gpt-5.5",
            "prompt": { "mode": "append", "file": "PROMPT.md" },
            "tools": ["read", "write"],
            "toolPolicy": { "allow": ["read", "write"] },
            "pathPolicy": {
                "allowRead": ["~/.scriptkit/plugins/main/profiles"],
                "allowWrite": ["~/.scriptkit/plugins/main/profiles"],
                "deny": ["~/.ssh", "~/.codex", "~/.scriptkit/secrets"]
            },
            "disableExtensions": true,
            "disableSkills": true,
            "disablePromptTemplates": true
        });
        merge_json(&mut value, overrides);
        fs::write(
            profile_root.join(PROFILE_MANIFEST_FILE),
            serde_json::to_string_pretty(&value).expect("serialize profile"),
        )
        .expect("write profile");
        profile_root
    }

    fn merge_json(base: &mut serde_json::Value, overrides: serde_json::Value) {
        let (Some(base), Some(overrides)) = (base.as_object_mut(), overrides.as_object()) else {
            return;
        };
        for (key, value) in overrides {
            base.insert(key.clone(), value.clone());
        }
    }

    fn plugin_index(plugin_root: PathBuf) -> PluginIndex {
        fs::write(
            plugin_root.join("plugin.json"),
            r#"{"id":"main","title":"Main"}"#,
        )
        .expect("write manifest");
        let manifest = read_plugin_manifest(&plugin_root).expect("read manifest");
        assert_eq!(
            manifest,
            PluginManifest {
                id: "main".to_string(),
                title: "Main".to_string(),
                ..PluginManifest::default()
            }
        );
        PluginIndex {
            plugins: vec![PluginRoot {
                id: "main".to_string(),
                root: plugin_root,
                manifest,
            }],
        }
    }

    #[test]
    fn discovers_valid_plugin_profile_artifacts() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_profile(temp.path(), "profile-builder", json!({}));
        let profiles = discover_plugin_profiles_in(&plugin_index(temp.path().to_path_buf()))
            .expect("discover profiles");
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].plugin_id, "main");
        assert_eq!(profiles[0].profile_id, "profile-builder");
        assert_eq!(
            profiles[0].artifact.provider.as_deref(),
            Some("openai-codex")
        );
    }

    #[test]
    fn skips_reserved_builtin_profile_ids() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_profile(temp.path(), "script-kit", json!({}));
        let profiles = discover_plugin_profiles_in(&plugin_index(temp.path().to_path_buf()))
            .expect("discover profiles");
        assert!(profiles.is_empty());
    }

    #[test]
    fn rejects_mutation_tools_without_allow_write() {
        let temp = tempfile::tempdir().expect("tempdir");
        let profile_root = write_profile(
            temp.path(),
            "writer",
            json!({
                "pathPolicy": {
                    "allowRead": ["~/.scriptkit/plugins/main/profiles"],
                    "allowWrite": [],
                    "deny": ["~/.ssh", "~/.codex", "~/.scriptkit/secrets"]
                }
            }),
        );
        let artifact = parse_profile_artifact(&profile_root.join(PROFILE_MANIFEST_FILE))
            .expect("parse artifact");
        let err = validate_artifact(&profile_root, &artifact).expect_err("must reject");
        assert!(err.to_string().contains("allowWrite"));
    }

    #[test]
    fn rejects_prompt_paths_that_escape_profile_root() {
        let temp = tempfile::tempdir().expect("tempdir");
        let profile_root = write_profile(
            temp.path(),
            "bad-prompt",
            json!({ "prompt": { "mode": "append", "file": "../PROMPT.md" } }),
        );
        let artifact = parse_profile_artifact(&profile_root.join(PROFILE_MANIFEST_FILE))
            .expect("parse artifact");
        let err = validate_artifact(&profile_root, &artifact).expect_err("must reject");
        assert!(err.to_string().contains("escape"));
    }

    #[test]
    fn rejects_profile_id_that_does_not_match_directory_name() {
        let temp = tempfile::tempdir().expect("tempdir");
        let profile_root = write_profile(temp.path(), "writer", json!({ "id": "reader" }));
        let artifact = parse_profile_artifact(&profile_root.join(PROFILE_MANIFEST_FILE))
            .expect("parse artifact");
        let err = validate_artifact(&profile_root, &artifact).expect_err("must reject");
        assert!(err.to_string().contains("directory name"));
    }

    #[test]
    fn rejects_broad_allow_read_paths() {
        let temp = tempfile::tempdir().expect("tempdir");
        let profile_root = write_profile(
            temp.path(),
            "reader",
            json!({
                "tools": ["read"],
                "toolPolicy": { "allow": ["read"] },
                "pathPolicy": {
                    "allowRead": ["~/.scriptkit"],
                    "allowWrite": [],
                    "deny": ["~/.ssh", "~/.codex", "~/.scriptkit/secrets"]
                }
            }),
        );
        let artifact = parse_profile_artifact(&profile_root.join(PROFILE_MANIFEST_FILE))
            .expect("parse artifact");
        let err = validate_artifact(&profile_root, &artifact).expect_err("must reject");
        assert!(err.to_string().contains("allowRead"));
    }

    #[test]
    fn rejects_missing_explicit_tool_policy_allow() {
        let temp = tempfile::tempdir().expect("tempdir");
        let profile_root = write_profile(
            temp.path(),
            "implicit-tools",
            json!({
                "toolPolicy": {},
                "tools": ["read"],
                "pathPolicy": {
                    "allowRead": ["~/.scriptkit/plugins/main/profiles"],
                    "allowWrite": [],
                    "deny": ["~/.ssh", "~/.codex", "~/.scriptkit/secrets"]
                }
            }),
        );
        let artifact = parse_profile_artifact(&profile_root.join(PROFILE_MANIFEST_FILE))
            .expect("parse artifact");
        let err = validate_artifact(&profile_root, &artifact).expect_err("must reject");
        assert!(err.to_string().contains("toolPolicy.allow"));
    }

    #[test]
    fn rejects_bash_tool_in_schema_v1() {
        let temp = tempfile::tempdir().expect("tempdir");
        let profile_root = write_profile(
            temp.path(),
            "bash-profile",
            json!({
                "tools": ["read", "bash"],
                "toolPolicy": { "allow": ["read", "bash"] }
            }),
        );
        let artifact = parse_profile_artifact(&profile_root.join(PROFILE_MANIFEST_FILE))
            .expect("parse artifact");
        let err = validate_artifact(&profile_root, &artifact).expect_err("must reject");
        assert!(err.to_string().contains("bash"));
    }

    #[test]
    fn rejects_broad_paths_with_trailing_slashes() {
        for broad_path in ["~/.scriptkit/", "~/Desktop/", "~/Documents/"] {
            let temp = tempfile::tempdir().expect("tempdir");
            let profile_root = write_profile(
                temp.path(),
                "reader",
                json!({
                    "tools": ["read"],
                    "toolPolicy": { "allow": ["read"] },
                    "pathPolicy": {
                        "allowRead": [broad_path],
                        "allowWrite": [],
                        "deny": ["~/.ssh", "~/.codex", "~/.scriptkit/secrets"]
                    }
                }),
            );
            let artifact = parse_profile_artifact(&profile_root.join(PROFILE_MANIFEST_FILE))
                .expect("parse artifact");
            let err = validate_artifact(&profile_root, &artifact).expect_err("must reject");
            assert!(err.to_string().contains("allowRead"));
        }
    }

    #[test]
    fn protected_denies_accept_trailing_slashes() {
        let temp = tempfile::tempdir().expect("tempdir");
        let profile_root = write_profile(
            temp.path(),
            "reader",
            json!({
                "tools": ["read"],
                "toolPolicy": { "allow": ["read"] },
                "pathPolicy": {
                    "allowRead": ["~/.scriptkit/plugins/main/profiles"],
                    "allowWrite": [],
                    "deny": ["~/.ssh/", "~/.codex/", "~/.scriptkit/secrets/"]
                }
            }),
        );
        let artifact = parse_profile_artifact(&profile_root.join(PROFILE_MANIFEST_FILE))
            .expect("parse artifact");
        validate_artifact(&profile_root, &artifact).expect("trailing slash denies normalize");
    }

    #[test]
    fn rejects_filesystem_profiles_without_protected_denies() {
        let temp = tempfile::tempdir().expect("tempdir");
        let profile_root = write_profile(
            temp.path(),
            "reader",
            json!({
                "tools": ["read"],
                "toolPolicy": { "allow": ["read"] },
                "pathPolicy": {
                    "allowRead": ["~/.scriptkit/plugins/main/profiles"],
                    "allowWrite": [],
                    "deny": ["~/.ssh", "~/.codex"]
                }
            }),
        );
        let artifact = parse_profile_artifact(&profile_root.join(PROFILE_MANIFEST_FILE))
            .expect("parse artifact");
        let err = validate_artifact(&profile_root, &artifact).expect_err("must reject");
        assert!(err.to_string().contains("~/.scriptkit/secrets"));
    }

    #[test]
    fn rejects_missing_readme_and_smoke_artifacts() {
        let temp = tempfile::tempdir().expect("tempdir");
        let profile_root = write_profile(temp.path(), "writer", json!({}));
        fs::remove_file(profile_root.join("README.md")).expect("remove readme");
        let artifact = parse_profile_artifact(&profile_root.join(PROFILE_MANIFEST_FILE))
            .expect("parse artifact");
        let err = validate_artifact(&profile_root, &artifact).expect_err("must reject");
        assert!(err.to_string().contains("README.md"));
    }
}
