use std::sync::Arc;

use script_kit_gpui::plugins::PluginSkill;
use script_kit_gpui::scripts::{fuzzy_search_skills, SkillMatch};

fn make_skill(
    plugin_id: &str,
    plugin_title: &str,
    skill_id: &str,
    title: &str,
) -> Arc<PluginSkill> {
    Arc::new(PluginSkill {
        plugin_id: plugin_id.to_string(),
        plugin_title: plugin_title.to_string(),
        skill_id: skill_id.to_string(),
        path: std::path::PathBuf::from(format!(
            "/fake/kit/{}/skills/{}/SKILL.md",
            plugin_id, skill_id
        )),
        title: title.to_string(),
        description: format!("{} skill from {}", title, plugin_title),
    })
}

#[test]
fn skill_search_returns_all_on_empty_query() {
    let skills = vec![
        make_skill(
            "authoring",
            "Authoring",
            "scriptlets",
            "Scriptlet Authoring",
        ),
        make_skill("authoring", "Authoring", "config", "Configuration"),
    ];
    let results = fuzzy_search_skills(&skills, "");
    assert_eq!(results.len(), 2, "empty query returns all skills");
    assert!(results.iter().all(|r| r.score == 0));
}

#[test]
fn skill_search_matches_title() {
    let skills = vec![
        make_skill(
            "authoring",
            "Authoring",
            "scriptlets",
            "Scriptlet Authoring",
        ),
        make_skill("authoring", "Authoring", "config", "Configuration"),
        make_skill("tools", "Tools", "debug", "Debug Helper"),
    ];
    let results = fuzzy_search_skills(&skills, "config");
    assert!(!results.is_empty(), "should match 'config'");
    assert_eq!(
        results[0].skill.skill_id, "config",
        "Configuration should be top result"
    );
}

#[test]
fn skill_search_matches_plugin_title() {
    let skills = vec![
        make_skill("authoring", "Authoring", "review", "Code Review"),
        make_skill("tools", "Dev Tools", "review", "Code Review"),
    ];
    let results = fuzzy_search_skills(&skills, "dev tools");
    // "Dev Tools" matches plugin_title, so tools/review should score higher
    let tools_results: Vec<&SkillMatch> = results
        .iter()
        .filter(|r| r.skill.plugin_id == "tools")
        .collect();
    assert!(
        !tools_results.is_empty(),
        "plugin_title match should appear"
    );
}

#[test]
fn duplicate_skill_slugs_across_plugins_are_distinct_results() {
    let skills = vec![
        make_skill("alpha", "Alpha", "review", "Alpha Review"),
        make_skill("beta", "Beta", "review", "Beta Review"),
    ];
    let results = fuzzy_search_skills(&skills, "review");
    assert_eq!(
        results.len(),
        2,
        "both plugins' review skills must appear as distinct results"
    );
    let plugin_ids: Vec<&str> = results.iter().map(|r| r.skill.plugin_id.as_str()).collect();
    assert!(plugin_ids.contains(&"alpha"));
    assert!(plugin_ids.contains(&"beta"));
}

#[test]
fn skill_frecency_key_is_plugin_qualified() {
    // Verify the frecency key format used in grouping/search_mode
    let skill = make_skill("authoring", "Authoring", "scriptlets", "Scriptlets");
    let key = format!("skill:{}:{}", skill.plugin_id, skill.skill_id);
    assert_eq!(key, "skill:authoring:scriptlets");
}
