use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Canonical plugin manifest stored as `plugin.json` in each plugin root.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PluginManifest {
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub repo_url: String,
}

/// A discovered plugin root directory with its resolved manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginRoot {
    pub id: String,
    pub root: PathBuf,
    pub manifest: PluginManifest,
}

/// The complete set of discovered plugins, sorted by id.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PluginIndex {
    pub plugins: Vec<PluginRoot>,
}

/// A skill discovered under a plugin's `skills/` directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginSkill {
    pub plugin_id: String,
    /// Human-readable plugin title for display (e.g., "Authoring", "Tools")
    pub plugin_title: String,
    pub skill_id: String,
    pub path: PathBuf,
    /// Display title parsed from SKILL.md frontmatter or first H1, falls back to skill_id
    pub title: String,
    /// Description parsed from SKILL.md frontmatter, empty if absent
    pub description: String,
}

impl PluginSkill {
    /// Returns the best human-readable source label for this skill's plugin.
    /// Prefers `plugin_title`; falls back to `plugin_id` when the title is empty.
    pub fn display_source(&self) -> &str {
        if self.plugin_title.is_empty() {
            &self.plugin_id
        } else {
            &self.plugin_title
        }
    }

    /// Build the ACP entry intent text for launching this skill from the launcher.
    ///
    /// When `skill_content` is non-empty, the SKILL.md body is embedded in a
    /// `<skill>` tag. When empty (unreadable or genuinely blank), the intent
    /// omits the attachment and asks the agent to proceed without it.
    pub fn acp_entry_intent(&self, skill_content: &str) -> String {
        let source = self.display_source();
        if skill_content.trim().is_empty() {
            format!(
                "Skill selected from the Script Kit launcher.\n\n\
                 Plugin: {source}\n\
                 Skill: {}\n\n\
                 Use this skill for this session.",
                self.title,
            )
        } else {
            format!(
                "Skill selected from the Script Kit launcher.\n\n\
                 Plugin: {source}\n\
                 Skill: {}\n\
                 Path: {}\n\n\
                 <skill path=\"{}\">\n{}\n</skill>\n\n\
                 Use the attached skill for this session.",
                self.title,
                self.path.display(),
                self.path.display(),
                skill_content,
            )
        }
    }
}
