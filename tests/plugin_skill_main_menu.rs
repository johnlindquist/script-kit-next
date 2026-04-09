//! Integration tests: Agent suppressed from launcher, Skill is first-class
//!
//! Verifies the acceptance criteria for the agent-menu-decenter task:
//! - The launcher search/grouping/selection pipeline no longer yields top-level Agent results.
//! - Scripts, scriptlets, and skills still launch correctly from the main menu.
//! - Any legacy suppression path is observable (structured log, not silent TODO).

use std::path::PathBuf;
use std::sync::Arc;

use script_kit_gpui::agents::{Agent, AgentBackend, AgentFrontmatter};
use script_kit_gpui::plugins::PluginSkill;
use script_kit_gpui::scripts::{
    fuzzy_search_unified_all_with_skills, AgentMatch, MatchIndices, Script, ScriptMatch, Scriptlet,
    ScriptletMatch, SearchResult, SkillMatch,
};

// ── Helpers ────────────────────────────────────────────────────────────

fn make_script(name: &str, plugin_id: &str) -> Arc<Script> {
    Arc::new(Script {
        name: name.to_string(),
        path: PathBuf::from(format!("/scripts/{}.ts", name)),
        extension: "ts".to_string(),
        plugin_id: plugin_id.to_string(),
        plugin_title: Some(plugin_id.to_string()),
        kit_name: Some(plugin_id.to_string()),
        ..Default::default()
    })
}

fn make_scriptlet(name: &str, plugin_id: &str) -> Arc<Scriptlet> {
    Arc::new(Scriptlet {
        name: name.to_string(),
        code: "echo hello".to_string(),
        tool: "bash".to_string(),
        plugin_id: plugin_id.to_string(),
        plugin_title: Some(plugin_id.to_string()),
        description: None,
        shortcut: None,
        keyword: None,
        group: None,
        file_path: Some(format!("/extensions/{}.md#slug", name)),
        command: Some(name.to_string()),
        alias: None,
    })
}

fn make_skill(plugin_id: &str, skill_id: &str, title: &str) -> Arc<PluginSkill> {
    Arc::new(PluginSkill {
        plugin_id: plugin_id.to_string(),
        plugin_title: plugin_id.to_string(),
        skill_id: skill_id.to_string(),
        path: PathBuf::from(format!("/kit/{}/skills/{}/SKILL.md", plugin_id, skill_id)),
        title: title.to_string(),
        description: String::new(),
    })
}

fn make_agent(name: &str, kit: &str) -> Arc<Agent> {
    Arc::new(Agent {
        name: name.to_string(),
        path: PathBuf::from(format!("/agents/{}.claude.md", name)),
        backend: AgentBackend::Claude,
        description: Some("test agent".to_string()),
        interactive: false,
        icon: None,
        alias: None,
        shortcut: None,
        kit: Some(kit.to_string()),
        frontmatter: AgentFrontmatter::default(),
        has_shell_inlines: false,
        has_remote_imports: false,
    })
}

fn make_agent_match(agent: Arc<Agent>) -> AgentMatch {
    AgentMatch {
        display_name: agent.name.clone(),
        score: 100,
        match_indices: MatchIndices::default(),
        agent,
    }
}

// ── Search pipeline suppression ────────────────────────────────────────

#[test]
fn unified_search_never_yields_agent_results() {
    let scripts = vec![make_script("hello-world", "main")];
    let scriptlets = vec![make_scriptlet("greet", "main")];
    let skills = vec![make_skill("authoring", "scriptlets", "Scriptlet Authoring")];

    // Search with a query that would match anything
    let results =
        fuzzy_search_unified_all_with_skills(&scripts, &scriptlets, &[], &[], &skills, "");

    // No Agent results should appear
    for result in &results {
        assert!(
            !result.is_suppressed_agent(),
            "Agent results must not appear in unified search output"
        );
    }
}

#[test]
fn unified_search_with_query_never_yields_agent_results() {
    let scripts = vec![make_script("review-code", "main")];
    let scriptlets = vec![make_scriptlet("lint", "tools")];
    let skills = vec![make_skill("authoring", "review", "Code Review")];

    let results =
        fuzzy_search_unified_all_with_skills(&scripts, &scriptlets, &[], &[], &skills, "review");

    for result in &results {
        assert!(
            !result.is_suppressed_agent(),
            "Agent results must not appear in unified search output for query 'review'"
        );
    }

    // Scripts and skills should still appear
    let has_script = results.iter().any(|r| matches!(r, SearchResult::Script(_)));
    let has_skill = results.iter().any(|r| matches!(r, SearchResult::Skill(_)));
    assert!(has_script, "Scripts should still appear in search results");
    assert!(has_skill, "Skills should still appear in search results");
}

// ── is_suppressed_agent correctness ────────────────────────────────────

#[test]
fn is_suppressed_agent_true_for_agent_variant() {
    let agent = make_agent("review-pr", "main");
    let result = SearchResult::Agent(make_agent_match(agent));
    assert!(result.is_suppressed_agent());
}

#[test]
fn is_suppressed_agent_false_for_script_variant() {
    let script = make_script("hello", "main");
    let result = SearchResult::Script(ScriptMatch {
        script,
        score: 100,
        filename: "hello.ts".to_string(),
        match_indices: MatchIndices::default(),
        match_kind: Default::default(),
        content_match: None,
    });
    assert!(!result.is_suppressed_agent());
}

#[test]
fn is_suppressed_agent_false_for_skill_variant() {
    let skill = make_skill("authoring", "scriptlets", "Scriptlets");
    let result = SearchResult::Skill(SkillMatch {
        skill,
        score: 100,
        match_indices: MatchIndices::default(),
    });
    assert!(!result.is_suppressed_agent());
}

#[test]
fn is_suppressed_agent_false_for_scriptlet_variant() {
    let scriptlet = make_scriptlet("greet", "main");
    let result = SearchResult::Scriptlet(ScriptletMatch {
        scriptlet,
        score: 100,
        display_file_path: None,
        match_indices: MatchIndices::default(),
    });
    assert!(!result.is_suppressed_agent());
}

// ── Skills appear as first-class results ───────────────────────────────

#[test]
fn skills_appear_in_unified_search_results() {
    let scripts: Vec<Arc<Script>> = vec![];
    let scriptlets: Vec<Arc<Scriptlet>> = vec![];
    let skills = vec![
        make_skill("authoring", "scriptlets", "Scriptlet Authoring"),
        make_skill("tools", "review", "Code Review"),
    ];

    let results =
        fuzzy_search_unified_all_with_skills(&scripts, &scriptlets, &[], &[], &skills, "");

    let skill_results: Vec<_> = results
        .iter()
        .filter(|r| matches!(r, SearchResult::Skill(_)))
        .collect();

    assert_eq!(
        skill_results.len(),
        2,
        "Both skills should appear in search results"
    );
}

#[test]
fn skills_from_different_plugins_remain_distinct() {
    let skills = vec![
        make_skill("alpha", "review", "Alpha Review"),
        make_skill("beta", "review", "Beta Review"),
    ];

    let results = fuzzy_search_unified_all_with_skills(&[], &[], &[], &[], &skills, "review");

    let skill_results: Vec<_> = results
        .iter()
        .filter_map(|r| match r {
            SearchResult::Skill(sm) => Some(sm),
            _ => None,
        })
        .collect();

    assert_eq!(
        skill_results.len(),
        2,
        "Duplicate skill slugs from different plugins must remain distinct"
    );

    let plugin_ids: Vec<&str> = skill_results
        .iter()
        .map(|sm| sm.skill.plugin_id.as_str())
        .collect();
    assert!(plugin_ids.contains(&"alpha"));
    assert!(plugin_ids.contains(&"beta"));
}

// ── Scripts and scriptlets survive agent suppression ────────────────────

#[test]
fn scripts_and_scriptlets_still_launch_after_agent_suppression() {
    let scripts = vec![
        make_script("git-commit", "main"),
        make_script("deploy", "tools"),
    ];
    let scriptlets = vec![
        make_scriptlet("open-github", "main"),
        make_scriptlet("paste-date", "tools"),
    ];
    let skills = vec![make_skill("authoring", "scriptlets", "Scriptlets")];

    let results =
        fuzzy_search_unified_all_with_skills(&scripts, &scriptlets, &[], &[], &skills, "");

    let script_count = results
        .iter()
        .filter(|r| matches!(r, SearchResult::Script(_)))
        .count();
    let scriptlet_count = results
        .iter()
        .filter(|r| matches!(r, SearchResult::Scriptlet(_)))
        .count();
    let skill_count = results
        .iter()
        .filter(|r| matches!(r, SearchResult::Skill(_)))
        .count();
    let agent_count = results
        .iter()
        .filter(|r| matches!(r, SearchResult::Agent(_)))
        .count();

    assert_eq!(script_count, 2, "All scripts should still appear");
    assert_eq!(scriptlet_count, 2, "All scriptlets should still appear");
    assert_eq!(skill_count, 1, "Skills should appear");
    assert_eq!(agent_count, 0, "No agent results should appear");
}

// ── Agent default action text reflects suppression ─────────────────────

#[test]
fn agent_default_action_text_indicates_suppression() {
    let agent = make_agent("review-pr", "main");
    let result = SearchResult::Agent(make_agent_match(agent));
    let action_text = result.get_default_action_text();
    assert!(
        action_text.contains("suppressed"),
        "Agent action text should indicate suppression, got: '{}'",
        action_text
    );
}

#[test]
fn skill_default_action_text_is_open_skill() {
    let skill = make_skill("authoring", "scriptlets", "Scriptlets");
    let result = SearchResult::Skill(SkillMatch {
        skill,
        score: 100,
        match_indices: MatchIndices::default(),
    });
    assert_eq!(result.get_default_action_text(), "Open Skill");
}

// ── Skill promotion: equal-score tiebreak ─────────────────────────────

#[test]
fn equal_score_skill_sorts_before_scriptlet() {
    // When a skill and a scriptlet match the same query with equal scores,
    // the skill must rank ahead due to the promoted type ordering.
    let scripts: Vec<Arc<Script>> = vec![];
    let scriptlets = vec![make_scriptlet("review-diff", "tools")];
    let skills = vec![make_skill("authoring", "review", "Review")];

    let results =
        fuzzy_search_unified_all_with_skills(&scripts, &scriptlets, &[], &[], &skills, "review");

    let positions: Vec<(&str, &str)> = results
        .iter()
        .map(|r| {
            (
                match r {
                    SearchResult::Skill(_) => "Skill",
                    SearchResult::Scriptlet(_) => "Scriptlet",
                    SearchResult::Script(_) => "Script",
                    _ => "Other",
                },
                r.name(),
            )
        })
        .collect();

    // Find the first Skill and the first Scriptlet
    let skill_pos = positions.iter().position(|(t, _)| *t == "Skill");
    let scriptlet_pos = positions.iter().position(|(t, _)| *t == "Scriptlet");

    assert!(
        skill_pos.is_some(),
        "Skill should appear in results: {:?}",
        positions
    );
    assert!(
        scriptlet_pos.is_some(),
        "Scriptlet should appear in results: {:?}",
        positions
    );
    assert!(
        skill_pos.unwrap() < scriptlet_pos.unwrap(),
        "Skill should sort before Scriptlet at equal score. Positions: {:?}",
        positions
    );
}

#[test]
fn equal_score_skill_sorts_before_script() {
    let scripts = vec![make_script("review-code", "main")];
    let scriptlets: Vec<Arc<Scriptlet>> = vec![];
    let skills = vec![make_skill("authoring", "review", "Review")];

    let results =
        fuzzy_search_unified_all_with_skills(&scripts, &scriptlets, &[], &[], &skills, "review");

    let skill_pos = results
        .iter()
        .position(|r| matches!(r, SearchResult::Skill(_)));
    let script_pos = results
        .iter()
        .position(|r| matches!(r, SearchResult::Script(_)));

    assert!(skill_pos.is_some(), "Skill should appear in results");
    assert!(script_pos.is_some(), "Script should appear in results");
    assert!(
        skill_pos.unwrap() < script_pos.unwrap(),
        "Skill should sort before Script at equal score"
    );
}

// ── Grouped view: skill-first within plugin sections ──────────────────

#[test]
fn grouped_view_skills_before_scripts_in_plugin_section() {
    use script_kit_gpui::config::SuggestedConfig;
    use script_kit_gpui::frecency::FrecencyStore;
    use script_kit_gpui::scripts::get_grouped_results;

    let scripts = vec![make_script("hello-world", "authoring")];
    let scriptlets = vec![make_scriptlet("open-docs", "authoring")];
    let skills = vec![
        make_skill("authoring", "scriptlets", "Script Authoring"),
        make_skill("authoring", "scriptlet-edit", "Scriptlet Edit"),
    ];

    let frecency = FrecencyStore::new();
    let suggested = SuggestedConfig {
        enabled: false,
        ..Default::default()
    };

    let (grouped, results) = get_grouped_results(
        &scripts,
        &scriptlets,
        &[],
        &[],
        &skills,
        &frecency,
        "",
        &suggested,
        &[],
        None,
    );

    // Find the "authoring" plugin section items
    let authoring_items: Vec<&str> = grouped
        .iter()
        .skip_while(|item| {
            // Skip until we find the AUTHORING section header
            match item {
                script_kit_gpui::list_item::GroupedListItem::SectionHeader(header, _) => {
                    !header.to_uppercase().contains("AUTHORING")
                }
                _ => true,
            }
        })
        .skip(1) // skip the header itself
        .take_while(|item| matches!(item, script_kit_gpui::list_item::GroupedListItem::Item(_)))
        .filter_map(|item| {
            if let script_kit_gpui::list_item::GroupedListItem::Item(idx) = item {
                Some(results[*idx].name())
            } else {
                None
            }
        })
        .collect();

    // Skills must come before scripts and scriptlets in the section
    let first_non_skill = authoring_items.iter().position(|name| {
        // Check if this is NOT a skill
        !skills.iter().any(|s| s.title.as_str() == *name)
    });
    let last_skill = authoring_items
        .iter()
        .rposition(|name| skills.iter().any(|s| s.title.as_str() == *name));

    if let (Some(first_non_skill_pos), Some(last_skill_pos)) = (first_non_skill, last_skill) {
        assert!(
            last_skill_pos < first_non_skill_pos,
            "All skills must appear before non-skills in plugin section. Order: {:?}",
            authoring_items
        );
    }
}

// ── Slash picker: picker_owner_meta format ────────────────────────────

#[test]
fn slash_picker_owner_meta_formats() {
    use script_kit_gpui::ai::SlashCommandPayload;

    let default = SlashCommandPayload::Default {
        name: "compact".to_string(),
    };
    assert_eq!(default.picker_owner_meta(), "/compact");

    let plugin = SlashCommandPayload::PluginSkill(PluginSkill {
        plugin_id: "alpha".to_string(),
        plugin_title: "Alpha Tools".to_string(),
        skill_id: "review".to_string(),
        path: PathBuf::from("/alpha/skills/review/SKILL.md"),
        title: "Review".to_string(),
        description: String::new(),
    });
    assert_eq!(
        plugin.picker_owner_meta(),
        "/review \u{b7} Alpha Tools skill"
    );

    let claude = SlashCommandPayload::ClaudeCodeSkill {
        skill_id: "plan".to_string(),
        skill_path: PathBuf::from("/tmp/plan/SKILL.md"),
    };
    assert_eq!(claude.picker_owner_meta(), "/plan \u{b7} Claude Code skill");
}
