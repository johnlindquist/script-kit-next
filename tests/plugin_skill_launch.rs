use std::sync::Arc;

use script_kit_gpui::plugins::PluginSkill;
use script_kit_gpui::scripts::{MatchIndices, SearchResult, SkillMatch};

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
fn skill_search_result_has_correct_type_label() {
    let skill = make_skill("authoring", "Authoring", "scriptlets", "Scriptlets");
    let result = SearchResult::Skill(SkillMatch {
        skill,
        score: 100,
        match_indices: MatchIndices::default(),
    });
    assert_eq!(result.type_label(), "Skill");
}

#[test]
fn skill_search_result_has_correct_action_text() {
    let skill = make_skill("authoring", "Authoring", "scriptlets", "Scriptlets");
    let result = SearchResult::Skill(SkillMatch {
        skill,
        score: 100,
        match_indices: MatchIndices::default(),
    });
    assert_eq!(result.get_default_action_text(), "Open Skill");
}

#[test]
fn skill_search_result_name_is_title() {
    let skill = make_skill(
        "authoring",
        "Authoring",
        "scriptlets",
        "Scriptlet Authoring",
    );
    let result = SearchResult::Skill(SkillMatch {
        skill,
        score: 100,
        match_indices: MatchIndices::default(),
    });
    assert_eq!(result.name(), "Scriptlet Authoring");
}

#[test]
fn skill_search_result_description_from_skill() {
    let skill = make_skill("authoring", "Authoring", "scriptlets", "Scriptlets");
    let result = SearchResult::Skill(SkillMatch {
        skill,
        score: 100,
        match_indices: MatchIndices::default(),
    });
    assert_eq!(
        result.description(),
        Some("Scriptlets skill from Authoring")
    );
}

#[test]
fn skill_search_result_source_name_is_plugin_title() {
    let skill = make_skill("authoring", "Authoring", "scriptlets", "Scriptlets");
    let result = SearchResult::Skill(SkillMatch {
        skill,
        score: 100,
        match_indices: MatchIndices::default(),
    });
    assert_eq!(result.source_name(), Some("Authoring"));
}

#[test]
fn skill_search_result_type_tag_uses_gold_color() {
    let skill = make_skill("authoring", "Authoring", "scriptlets", "Scriptlets");
    let result = SearchResult::Skill(SkillMatch {
        skill,
        score: 100,
        match_indices: MatchIndices::default(),
    });
    let (label, color) = result.type_tag_info();
    assert_eq!(label, "Skill");
    assert_eq!(color, 0xFBBF24, "skill badge should use gold accent");
}

#[test]
fn skill_initial_input_contains_skill_title_and_plugin() {
    let skill = make_skill(
        "authoring",
        "Authoring",
        "scriptlets",
        "Scriptlet Authoring",
    );

    // Simulate the initial input that would be built by open_acp_with_selected_skill
    let initial_input = format!(
        "Use the attached skill \"{}\" from plugin \"{}\" for this session.",
        skill.title, skill.plugin_title
    );
    assert!(initial_input.contains("Scriptlet Authoring"));
    assert!(initial_input.contains("Authoring"));
}
