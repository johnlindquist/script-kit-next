//! Slash-command and skill context models shared by the Agent Chat view.

use crate::ai::context_selector::types::SlashCommandPayload;

/// Parse the `description` field from YAML frontmatter in a SKILL.md file.
pub(super) fn parse_skill_description(content: &str) -> Option<String> {
    if !content.starts_with("---") {
        return None;
    }
    let end = content[3..].find("---")?;
    let frontmatter = &content[3..3 + end];
    for line in frontmatter.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("description:") {
            let desc = rest.trim().trim_matches('"').trim_matches('\'');
            // Truncate long descriptions for the menu
            if desc.len() > 80 {
                return Some(format!("{}\u{2026}", &desc[..77]));
            }
            return Some(desc.to_string());
        }
    }
    None
}

// ── Source-aware slash command model ──────────────────────────────────

/// The origin of a slash command entry discovered during skill enumeration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SlashCommandSource {
    /// A built-in Claude Code command (e.g. `/compact`, `/clear`).
    Default,
    /// A skill owned by a discovered plugin.
    PluginSkill(crate::plugins::PluginSkill),
    /// A user-level Claude Code skill from `~/.scriptkit/.claude/skills/`.
    ClaudeCodeSkill {
        skill_id: String,
        skill_path: std::path::PathBuf,
    },
}

impl SlashCommandSource {
    pub(super) fn owner_label(&self) -> String {
        match self {
            Self::Default => "Built-in".to_string(),
            Self::PluginSkill(skill) => {
                if skill.plugin_title.is_empty() {
                    skill.plugin_id.clone()
                } else {
                    skill.plugin_title.clone()
                }
            }
            Self::ClaudeCodeSkill { .. } => "Claude Code".to_string(),
        }
    }
}

/// A discovered slash command entry with source identity and description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SlashCommandEntry {
    /// The bare slash name (e.g. `"compact"`, `"review"`).
    pub name: String,
    /// Human-readable description for the picker.
    pub description: String,
    /// Where this entry came from.
    pub source: SlashCommandSource,
}

impl SlashCommandEntry {
    pub(crate) fn default_command(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: String::new(),
            source: SlashCommandSource::Default,
        }
    }

    pub(super) fn plugin_skill(skill: &crate::plugins::PluginSkill) -> Self {
        let plugin_title = if skill.plugin_title.is_empty() {
            skill.plugin_id.clone()
        } else {
            skill.plugin_title.clone()
        };

        let raw_desc = if skill.description.is_empty() {
            format!("Plugin: {}", plugin_title)
        } else {
            format!("{} \u{2014} {}", plugin_title, skill.description)
        };

        let desc_chars: Vec<char> = raw_desc.chars().collect();
        let description = if desc_chars.len() > 80 {
            let truncated: String = desc_chars.into_iter().take(77).collect();
            format!("{truncated}\u{2026}")
        } else {
            raw_desc
        };

        Self {
            name: skill.skill_id.clone(),
            description,
            source: SlashCommandSource::PluginSkill(skill.clone()),
        }
    }

    pub(super) fn claude_code_skill(
        name: String,
        description: String,
        skill_path: std::path::PathBuf,
    ) -> Self {
        Self {
            name: name.clone(),
            description,
            source: SlashCommandSource::ClaudeCodeSkill {
                skill_id: name,
                skill_path,
            },
        }
    }

    /// A key that uniquely identifies this entry across sources.
    pub(crate) fn qualified_key(&self) -> String {
        match &self.source {
            SlashCommandSource::Default => format!("default:{}", self.name),
            SlashCommandSource::PluginSkill(skill) => {
                format!("{}:{}", skill.plugin_id, skill.skill_id)
            }
            SlashCommandSource::ClaudeCodeSkill { skill_id, .. } => {
                format!("claude:{skill_id}")
            }
        }
    }

    /// Convert to a `SlashCommandPayload` for the context selector item kind.
    pub(crate) fn to_payload(&self) -> SlashCommandPayload {
        match &self.source {
            SlashCommandSource::Default => SlashCommandPayload::Default {
                name: self.name.clone(),
            },
            SlashCommandSource::PluginSkill(skill) => {
                SlashCommandPayload::PluginSkill(skill.clone())
            }
            SlashCommandSource::ClaudeCodeSkill {
                skill_id,
                skill_path,
            } => SlashCommandPayload::ClaudeCodeSkill {
                skill_id: skill_id.clone(),
                skill_path: skill_path.clone(),
            },
        }
    }
}

/// Build the staged prompt text for a local skill being accepted from
/// the Agent Chat slash picker or main-menu skill launch.  Both entry paths
/// must produce the same deterministic payload so that the Agent Chat agent
/// receives identical context regardless of how the user invoked the skill.
pub(crate) fn build_staged_skill_prompt(
    skill_title: &str,
    owner_label: &str,
    skill_path: &std::path::Path,
) -> String {
    let skill_content = std::fs::read_to_string(skill_path).unwrap_or_default();
    let owner_phrase = if owner_label == "Claude Code" {
        format!("from {owner_label}")
    } else {
        format!("from plugin \"{owner_label}\"")
    };
    if skill_content.is_empty() {
        format!("Use the skill \"{skill_title}\" {owner_phrase} for this session.")
    } else {
        format!(
            "Use the attached skill \"{skill_title}\" {owner_phrase} for this session.\n\n<skill path=\"{}\">\n{}\n</skill>",
            skill_path.display(),
            skill_content
        )
    }
}

/// Build the deterministic slash-prefill text for a selected skill.
pub(crate) fn build_skill_slash_command_text(slash_name: &str) -> String {
    format!("/{slash_name} ")
}

/// Build the attached skill context part shared by Agent Chat skill entry paths.
pub(crate) fn build_skill_context_part(
    skill_title: &str,
    owner_label: &str,
    slash_name: &str,
    skill_path: &std::path::Path,
) -> crate::ai::message_parts::AiContextPart {
    crate::ai::message_parts::AiContextPart::SkillFile {
        path: skill_path.to_string_lossy().to_string(),
        label: format!("/{slash_name}"),
        skill_name: skill_title.to_string(),
        owner_label: owner_label.to_string(),
        slash_name: slash_name.to_string(),
    }
}
