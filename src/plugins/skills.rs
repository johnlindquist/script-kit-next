use std::fs;

use anyhow::{Context, Result};
use tracing::info;

use super::types::{PluginIndex, PluginSkill};

/// Discover skills across all plugins in the index.
///
/// Each skill lives at `<plugin_root>/skills/<skill_id>/SKILL.md`. Results
/// are sorted by `(plugin_id, skill_id)` for deterministic ordering.
pub fn discover_plugin_skills(index: &PluginIndex) -> Result<Vec<PluginSkill>> {
    let mut skills = Vec::new();

    for plugin in &index.plugins {
        let skills_dir = plugin.root.join("skills");
        if !skills_dir.exists() {
            continue;
        }

        let entries = fs::read_dir(&skills_dir).with_context(|| {
            format!(
                "Failed to read skills dir for plugin {}: {}",
                plugin.id,
                skills_dir.display()
            )
        })?;

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue,
            };

            let skill_root = entry.path();
            let skill_doc = skill_root.join("SKILL.md");
            if !skill_doc.exists() {
                continue;
            }

            let skill_id = skill_root
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            info!(
                plugin_id = %plugin.id,
                skill_id = %skill_id,
                path = %skill_doc.display(),
                "plugin_skill_discovered"
            );

            skills.push(PluginSkill {
                plugin_id: plugin.id.clone(),
                skill_id: skill_id.clone(),
                path: skill_doc,
                title: skill_id,
            });
        }
    }

    skills.sort_by(|a, b| {
        a.plugin_id
            .cmp(&b.plugin_id)
            .then_with(|| a.skill_id.cmp(&b.skill_id))
    });

    Ok(skills)
}
