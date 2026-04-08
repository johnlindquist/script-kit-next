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
