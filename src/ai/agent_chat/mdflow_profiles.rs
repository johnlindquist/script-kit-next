//! Markdown Agent Chat profiles in the mdflow file format.
//!
//! One profile per `~/.scriptkit/profiles/<id>.md`: YAML frontmatter plus a
//! markdown body. The frontmatter keys are the same kebab-case flags the pi
//! CLI accepts (mdflow's passthrough convention — `model:`, `tools:`,
//! `thinking:`, `no-session:` …), so a profile file is also a valid
//! [mdflow](https://mdflow.dev) agent. The body becomes the profile's
//! instructions (`--append-system-prompt`).
//!
//! This replaces the retired `plugins/*/profiles/*/profile.json` pipeline:
//! creating a profile is now "drop one markdown file in `~/.scriptkit/profiles`"
//! (or press the Create action in the Shift+Tab Profile Search).

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::profiles::{AgentChatProfileContext, AgentChatProfileSource, ResolvedAgentChatProfile};
use crate::config::{AgentChatBackend, AgentChatToolPolicyConfig};

/// Directory holding markdown profiles: `<kit>/profiles`.
pub fn mdflow_profiles_dir(ctx: &AgentChatProfileContext) -> PathBuf {
    ctx.kit_path.join("profiles")
}

/// Frontmatter template used by the "Create New Profile" action. Kept in the
/// pi-flag passthrough shape so the file doubles as a runnable mdflow agent.
pub const MDFLOW_PROFILE_TEMPLATE: &str = r#"---
name: My Profile
model: openai-codex/gpt-5.3-codex-spark
tools: web_search
no-session: true
---

You are a focused Agent Chat profile. Describe the job, the tone, and the
boundaries here — this body is the profile's instructions.
"#;

const CACHE_TTL: Duration = Duration::from_secs(2);

struct MdflowProfileCacheEntry {
    dir: PathBuf,
    refreshed_at: Instant,
    profiles: Vec<ResolvedAgentChatProfile>,
}

static MDFLOW_PROFILE_CACHE: Mutex<Option<MdflowProfileCacheEntry>> = Mutex::new(None);

/// Drop the memoized profile list so the next lookup re-reads the directory
/// (used after the Create action writes a new file).
pub fn invalidate_mdflow_profile_cache() {
    if let Ok(mut cache) = MDFLOW_PROFILE_CACHE.lock() {
        *cache = None;
    }
}

/// Load all markdown profiles, memoized for a couple of seconds — Profile
/// Search and the composer picker resolve profiles several times per
/// keystroke and must not re-walk the directory each time.
pub fn resolved_mdflow_profiles(ctx: &AgentChatProfileContext) -> Vec<ResolvedAgentChatProfile> {
    let dir = mdflow_profiles_dir(ctx);
    let now = Instant::now();
    if let Ok(cache) = MDFLOW_PROFILE_CACHE.lock() {
        if let Some(entry) = cache.as_ref() {
            if entry.dir == dir && now.duration_since(entry.refreshed_at) < CACHE_TTL {
                return entry.profiles.clone();
            }
        }
    }

    let profiles = resolved_mdflow_profiles_uncached(&dir);
    if let Ok(mut cache) = MDFLOW_PROFILE_CACHE.lock() {
        *cache = Some(MdflowProfileCacheEntry {
            dir,
            refreshed_at: now,
            profiles: profiles.clone(),
        });
    }
    profiles
}

fn resolved_mdflow_profiles_uncached(dir: &PathBuf) -> Vec<ResolvedAgentChatProfile> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut profiles: Vec<ResolvedAgentChatProfile> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                return None;
            }
            let stem = path.file_stem()?.to_str()?.to_string();
            let content = match std::fs::read_to_string(&path) {
                Ok(content) => content,
                Err(error) => {
                    tracing::warn!(
                        target: "script_kit::agent_chat",
                        event = "mdflow_profile_read_failed",
                        path = %path.display(),
                        %error,
                    );
                    return None;
                }
            };
            match parse_mdflow_profile(&stem, &content) {
                Ok(profile) => Some(profile),
                Err(error) => {
                    tracing::warn!(
                        target: "script_kit::agent_chat",
                        event = "mdflow_profile_parse_failed",
                        path = %path.display(),
                        %error,
                    );
                    None
                }
            }
        })
        .collect();
    profiles.sort_by(|a, b| {
        a.name
            .to_ascii_lowercase()
            .cmp(&b.name.to_ascii_lowercase())
    });
    profiles
}

/// Parse one markdown profile. `stem` (the filename without `.md`) is the
/// profile id. Unknown frontmatter keys are ignored — they may be mdflow
/// engine flags that only matter when the file is run with `md`.
pub fn parse_mdflow_profile(stem: &str, content: &str) -> Result<ResolvedAgentChatProfile, String> {
    let (frontmatter, body) = split_frontmatter(content)?;
    let mapping: serde_yaml::Value = if frontmatter.trim().is_empty() {
        serde_yaml::Value::Mapping(Default::default())
    } else {
        serde_yaml::from_str(frontmatter).map_err(|error| error.to_string())?
    };
    let mapping = mapping
        .as_mapping()
        .ok_or_else(|| "frontmatter must be a YAML mapping".to_string())?;

    let get_str = |key: &str| -> Option<String> {
        mapping
            .get(serde_yaml::Value::String(key.to_string()))
            .and_then(value_to_string)
            .filter(|value| !value.trim().is_empty())
    };
    let get_bool = |key: &str| -> Option<bool> {
        mapping
            .get(serde_yaml::Value::String(key.to_string()))
            .and_then(serde_yaml::Value::as_bool)
    };

    let name = get_str("name").unwrap_or_else(|| title_case_stem(stem));

    // `model` accepts mdflow/pi's "provider/id" shorthand; an explicit
    // `provider:` key wins over the shorthand prefix.
    let raw_model = get_str("model");
    let (shorthand_provider, model) = match raw_model {
        Some(raw) => match raw.split_once('/') {
            Some((provider, model)) if !provider.trim().is_empty() && !model.trim().is_empty() => (
                Some(provider.trim().to_string()),
                Some(model.trim().to_string()),
            ),
            _ => (None, Some(raw)),
        },
        None => (None, None),
    };
    let provider = get_str("provider").or(shorthand_provider);

    let tools = mapping
        .get(serde_yaml::Value::String("tools".to_string()))
        .map(value_to_string_list);

    let body = body.trim();
    let append_system_prompt = if body.is_empty() {
        None
    } else {
        Some(body.to_string())
    };

    Ok(ResolvedAgentChatProfile {
        source: AgentChatProfileSource::Mdflow,
        id: stem.to_string(),
        name,
        icon_name: get_str("icon"),
        backend: AgentChatBackend::Pi,
        pi_binary: None,
        agent: None,
        provider,
        model,
        system_prompt: get_str("system-prompt"),
        append_system_prompt,
        cwd: get_str("cwd")
            .or_else(|| get_str("_cwd"))
            .map(|value| crate::ai::agent_chat::pi::binary::expand_tilde_path(&value)),
        tool_policy: tools.as_ref().map(|tools| AgentChatToolPolicyConfig {
            allow: Some(tools.clone()),
        }),
        tools,
        path_policy: None,
        blocked_action_message: None,
        disable_extensions: get_bool("no-extensions"),
        disable_skills: get_bool("no-skills"),
        disable_prompt_templates: get_bool("no-prompt-templates"),
        disable_context_files: get_bool("no-context-files"),
        hide_cwd_in_prompt: None,
        thinking: get_str("thinking"),
        extension_policy: None,
        session_dir: None,
        no_session: get_bool("no-session"),
        session_durability: None,
    })
}

/// Split `---` frontmatter from the markdown body. Files without frontmatter
/// are all body (instructions-only profiles are valid).
fn split_frontmatter(content: &str) -> Result<(&str, &str), String> {
    let trimmed = content.trim_start_matches(|c: char| c.is_whitespace() || c == '\u{feff}');
    let Some(rest) = trimmed.strip_prefix("---") else {
        return Ok(("", trimmed));
    };
    let rest = rest
        .strip_prefix('\n')
        .or_else(|| rest.strip_prefix("\r\n"))
        .ok_or_else(|| "frontmatter opening `---` must be on its own line".to_string())?;
    for (offset, _) in rest.match_indices("\n---") {
        let after = &rest[offset + 4..];
        let after_line_end = after
            .strip_prefix('\n')
            .or_else(|| after.strip_prefix("\r\n"))
            .or(if after.is_empty() { Some("") } else { None });
        if let Some(body) = after_line_end {
            return Ok((&rest[..offset], body));
        }
    }
    Err("frontmatter is missing its closing `---`".to_string())
}

fn value_to_string(value: &serde_yaml::Value) -> Option<String> {
    match value {
        serde_yaml::Value::String(text) => Some(text.clone()),
        serde_yaml::Value::Number(number) => Some(number.to_string()),
        serde_yaml::Value::Bool(flag) => Some(flag.to_string()),
        _ => None,
    }
}

/// Tools accept both a YAML list and pi's comma-separated string form.
fn value_to_string_list(value: &serde_yaml::Value) -> Vec<String> {
    match value {
        serde_yaml::Value::Sequence(items) => items
            .iter()
            .filter_map(value_to_string)
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect(),
        other => value_to_string(other)
            .map(|text| {
                text.split(',')
                    .map(|item| item.trim().to_string())
                    .filter(|item| !item.is_empty())
                    .collect()
            })
            .unwrap_or_default(),
    }
}

fn title_case_stem(stem: &str) -> String {
    stem.split(['-', '_'])
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Create a new profile file from [`MDFLOW_PROFILE_TEMPLATE`], picking a
/// filename that does not collide with an existing profile. Returns the path.
pub fn create_mdflow_profile_from_template(
    ctx: &AgentChatProfileContext,
) -> std::io::Result<PathBuf> {
    let dir = mdflow_profiles_dir(ctx);
    std::fs::create_dir_all(&dir)?;
    let mut path = dir.join("my-profile.md");
    let mut counter = 2u32;
    while path.exists() {
        path = dir.join(format!("my-profile-{counter}.md"));
        counter += 1;
    }
    std::fs::write(&path, MDFLOW_PROFILE_TEMPLATE)?;
    invalidate_mdflow_profile_cache();
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_frontmatter_and_body() {
        let profile = parse_mdflow_profile(
            "docs-researcher",
            "---\nname: Docs Researcher\nmodel: openai-codex/gpt-5.5\ntools: web_search, read\nthinking: low\nno-session: true\ncwd: ~/notes\n---\n\nResearch docs and cite sources.\n",
        )
        .expect("profile parses");

        assert_eq!(profile.id, "docs-researcher");
        assert_eq!(profile.name, "Docs Researcher");
        assert_eq!(profile.source, AgentChatProfileSource::Mdflow);
        assert_eq!(profile.provider.as_deref(), Some("openai-codex"));
        assert_eq!(profile.model.as_deref(), Some("gpt-5.5"));
        assert_eq!(
            profile.tools,
            Some(vec!["web_search".to_string(), "read".to_string()])
        );
        assert_eq!(profile.thinking.as_deref(), Some("low"));
        assert_eq!(profile.no_session, Some(true));
        assert!(profile
            .cwd
            .as_ref()
            .is_some_and(|cwd| cwd.ends_with("notes")));
        assert_eq!(
            profile.append_system_prompt.as_deref(),
            Some("Research docs and cite sources.")
        );
    }

    #[test]
    fn tools_accept_yaml_list_form() {
        let profile = parse_mdflow_profile(
            "lister",
            "---\ntools:\n  - web_search\n  - grep\n---\nBody.\n",
        )
        .expect("profile parses");
        assert_eq!(
            profile.tools,
            Some(vec!["web_search".to_string(), "grep".to_string()])
        );
        assert_eq!(
            profile
                .tool_policy
                .and_then(|policy| policy.allow)
                .unwrap_or_default()
                .len(),
            2
        );
    }

    #[test]
    fn body_only_file_is_an_instructions_profile_named_from_stem() {
        let profile = parse_mdflow_profile("code-review-buddy", "Be a strict reviewer.\n")
            .expect("profile parses");
        assert_eq!(profile.name, "Code Review Buddy");
        assert_eq!(profile.model, None);
        assert_eq!(
            profile.append_system_prompt.as_deref(),
            Some("Be a strict reviewer.")
        );
    }

    #[test]
    fn explicit_provider_key_beats_model_shorthand() {
        let profile = parse_mdflow_profile(
            "p",
            "---\nprovider: google-antigravity\nmodel: openai-codex/gpt-5.4\n---\n",
        )
        .expect("profile parses");
        assert_eq!(profile.provider.as_deref(), Some("google-antigravity"));
        assert_eq!(profile.model.as_deref(), Some("gpt-5.4"));
    }

    #[test]
    fn unclosed_frontmatter_is_an_error_not_a_silent_profile() {
        assert!(parse_mdflow_profile("bad", "---\nname: Broken\n").is_err());
    }

    #[test]
    fn template_parses_into_a_valid_profile() {
        let profile =
            parse_mdflow_profile("my-profile", MDFLOW_PROFILE_TEMPLATE).expect("template parses");
        assert_eq!(profile.name, "My Profile");
        assert_eq!(profile.tools, Some(vec!["web_search".to_string()]));
        assert_eq!(profile.no_session, Some(true));
        assert!(profile.append_system_prompt.is_some());
    }
}
