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
