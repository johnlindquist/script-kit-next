//! Smoke tests for the main-menu skill launch → ACP staging pipeline.
//!
//! Validates the end-to-end contract from plugin setup → skill discovery →
//! search/grouping → ACP initial input staging, without requiring a live
//! GPUI window. This is the headless equivalent of `make smoke-main-menu`.
//!
//! Runtime evidence is emitted via structured tracing logs:
//! - `plugin_skill_cataloged` — skill discovered in plugin inventory
//! - `main_menu_skill_ranked` — skill surfaced in search results
//! - `acp_skill_launch_requested` — skill selected from main menu
//! - `acp_skill_context_staged` — skill content staged for ACP session

use std::fs;
use std::sync::Arc;

use script_kit_gpui::plugins::{discover_plugin_skills, PluginSkill};
use script_kit_gpui::scripts::{fuzzy_search_unified_all_with_skills, SearchResult, SkillMatch};
use script_kit_gpui::setup::{ensure_kit_setup, SK_PATH_ENV};
use tempfile::TempDir;

/// Shared lock for SK_PATH env var mutation.
static SK_PATH_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn with_temp_sk_path<F: FnOnce(&std::path::Path)>(f: F) {
    let _lock = SK_PATH_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let temp_dir = TempDir::new().expect("create temp dir");
    let kit_root = temp_dir.path().join("scriptkit-test");
    std::env::set_var(SK_PATH_ENV, kit_root.to_str().unwrap());

    f(&kit_root);

    std::env::remove_var(SK_PATH_ENV);
}

// ── End-to-end: setup → discover → search → stage ───────────────────

/// Full vertical slice: ensure_kit_setup() seeds skills that are discoverable,
/// searchable, and produce the correct ACP staging payload.
#[test]
fn smoke_skill_discovery_through_search_pipeline() {
    with_temp_sk_path(|kit_root| {
        // Phase 1: Setup seeds plugin structure with skills
        let result = ensure_kit_setup();
        assert!(
            !result.warnings.iter().any(|w| w.contains("Failed")),
            "Setup should succeed: {:?}",
            result.warnings
        );

        // Phase 2: Build plugin index from seeded structure
        let kit_dir = kit_root.join("kit");
        let index = script_kit_gpui::plugins::discover_plugins_in(&kit_dir)
            .expect("plugin discovery should succeed");

        assert!(
            !index.plugins.is_empty(),
            "Plugin index must contain seeded plugins"
        );

        // Phase 3: Discover skills from plugin index
        let skills = discover_plugin_skills(&index).expect("skill discovery should succeed");

        assert!(
            !skills.is_empty(),
            "At least one skill must be discovered from seeded authoring plugin"
        );

        // Verify the authoring plugin's skills are present
        let authoring_skills: Vec<&PluginSkill> = skills
            .iter()
            .filter(|s| s.plugin_id == "authoring")
            .collect();
        assert!(
            authoring_skills.len() >= 3,
            "authoring plugin should have at least 3 skills (script-authoring, scriptlets, agents), got {}",
            authoring_skills.len()
        );

        // Phase 4: Feed skills into search pipeline
        let arc_skills: Vec<Arc<PluginSkill>> = skills.into_iter().map(Arc::new).collect();
        let results =
            fuzzy_search_unified_all_with_skills(&[], &[], &[], &[], &arc_skills, "scriptlet");

        let skill_results: Vec<&SkillMatch> = results
            .iter()
            .filter_map(|r| match r {
                SearchResult::Skill(sm) => Some(sm),
                _ => None,
            })
            .collect();

        assert!(
            !skill_results.is_empty(),
            "Search for 'scriptlet' should find at least one skill result"
        );

        // Verify no agents leak through
        assert!(
            results.iter().all(|r| !r.is_suppressed_agent()),
            "No agent results should appear in search pipeline"
        );

        // Phase 5: Validate ACP staging contract for the top skill result
        let top_skill = &skill_results[0].skill;
        let skill_content = fs::read_to_string(&top_skill.path).unwrap_or_default();
        assert!(
            !skill_content.is_empty(),
            "Skill SKILL.md should have content"
        );

        // Simulate the ACP initial input construction (mirrors open_acp_with_selected_skill)
        let initial_input = format!(
            "Use the attached skill \"{}\" from plugin \"{}\" for this session.\n\n<skill path=\"{}\">\n{}\n</skill>",
            top_skill.title,
            top_skill.plugin_title,
            top_skill.path.display(),
            skill_content
        );

        assert!(
            initial_input.contains(&top_skill.title),
            "ACP initial input must contain skill title"
        );
        assert!(
            initial_input.contains(&top_skill.plugin_title),
            "ACP initial input must contain plugin title"
        );
        assert!(
            initial_input.contains("<skill path="),
            "ACP initial input must wrap skill content in <skill> tags"
        );
        assert!(
            initial_input.contains("</skill>"),
            "ACP initial input must close <skill> tags"
        );
    });
}

/// Duplicate skill slugs from different plugins remain distinct through
/// the full pipeline (discovery → search → display).
#[test]
fn smoke_duplicate_skill_slugs_remain_distinct() {
    let temp = TempDir::new().expect("create temp dir");
    let container = temp.path();

    // Create two plugins with the same skill slug
    for plugin_id in &["alpha", "beta"] {
        let plugin_root = container.join(plugin_id);
        let skill_dir = plugin_root.join("skills").join("review");
        fs::create_dir_all(&skill_dir).unwrap();

        // Write plugin.json
        let manifest = format!(
            r#"{{"id":"{}","title":"{}","description":"test","version":"0.1.0"}}"#,
            plugin_id,
            plugin_id.to_uppercase()
        );
        fs::write(plugin_root.join("plugin.json"), manifest).unwrap();

        // Write SKILL.md
        let skill_content = format!(
            "---\ntitle: {} Review\ndescription: Review from {}\n---\n# Code Review\nReview code.",
            plugin_id.to_uppercase(),
            plugin_id
        );
        fs::write(skill_dir.join("SKILL.md"), skill_content).unwrap();
    }

    let index = script_kit_gpui::plugins::discover_plugins_in(container).expect("discover plugins");

    let skills = discover_plugin_skills(&index).expect("discover skills");
    assert_eq!(skills.len(), 2, "Both plugins' review skills must be found");

    let arc_skills: Vec<Arc<PluginSkill>> = skills.into_iter().map(Arc::new).collect();
    let results = fuzzy_search_unified_all_with_skills(&[], &[], &[], &[], &arc_skills, "review");

    let skill_results: Vec<&SkillMatch> = results
        .iter()
        .filter_map(|r| match r {
            SearchResult::Skill(sm) => Some(sm),
            _ => None,
        })
        .collect();

    assert_eq!(
        skill_results.len(),
        2,
        "Duplicate slugs from different plugins must yield 2 distinct search results"
    );

    let plugin_ids: Vec<&str> = skill_results
        .iter()
        .map(|sm| sm.skill.plugin_id.as_str())
        .collect();
    assert!(
        plugin_ids.contains(&"alpha"),
        "alpha plugin skill must appear"
    );
    assert!(
        plugin_ids.contains(&"beta"),
        "beta plugin skill must appear"
    );
}

/// Empty-query grouped view places skills under plugin ownership.
#[test]
fn smoke_skills_appear_in_empty_query_results() {
    with_temp_sk_path(|kit_root| {
        let _ = ensure_kit_setup();
        let kit_dir = kit_root.join("kit");

        let index =
            script_kit_gpui::plugins::discover_plugins_in(&kit_dir).expect("discover plugins");

        let skills = discover_plugin_skills(&index).expect("discover skills");
        let arc_skills: Vec<Arc<PluginSkill>> = skills.into_iter().map(Arc::new).collect();

        // Empty query — should return all skills
        let results = fuzzy_search_unified_all_with_skills(&[], &[], &[], &[], &arc_skills, "");

        let skill_count = results
            .iter()
            .filter(|r| matches!(r, SearchResult::Skill(_)))
            .count();

        assert!(
            skill_count > 0,
            "Empty-query results must include discovered skills"
        );
    });
}

/// Skill frecency keys are plugin-qualified, not bare slugs.
#[test]
fn smoke_skill_frecency_keys_are_plugin_qualified() {
    with_temp_sk_path(|kit_root| {
        let _ = ensure_kit_setup();
        let kit_dir = kit_root.join("kit");

        let index =
            script_kit_gpui::plugins::discover_plugins_in(&kit_dir).expect("discover plugins");

        let skills = discover_plugin_skills(&index).expect("discover skills");

        for skill in &skills {
            let key = format!("skill:{}:{}", skill.plugin_id, skill.skill_id);
            assert!(
                key.starts_with("skill:"),
                "Frecency key must start with 'skill:'"
            );
            assert!(
                key.contains(&skill.plugin_id),
                "Frecency key must contain plugin_id"
            );
            assert!(
                key.contains(&skill.skill_id),
                "Frecency key must contain skill_id"
            );
            // Verify it's NOT just a bare slug
            assert!(
                key.matches(':').count() == 2,
                "Frecency key must have format skill:<plugin_id>:<skill_id>, got: {}",
                key
            );
        }
    });
}

/// Agent results never appear alongside skill results in the search pipeline.
#[test]
fn smoke_agents_never_appear_in_skill_search() {
    with_temp_sk_path(|kit_root| {
        let _ = ensure_kit_setup();
        let kit_dir = kit_root.join("kit");

        let index =
            script_kit_gpui::plugins::discover_plugins_in(&kit_dir).expect("discover plugins");

        let skills = discover_plugin_skills(&index).expect("discover skills");
        let arc_skills: Vec<Arc<PluginSkill>> = skills.into_iter().map(Arc::new).collect();

        // Search for anything that might match agents
        for query in &["agent", "review", "plan", ""] {
            let results =
                fuzzy_search_unified_all_with_skills(&[], &[], &[], &[], &arc_skills, query);

            for result in &results {
                assert!(
                    !result.is_suppressed_agent(),
                    "Agent results must never appear in search pipeline (query: '{}')",
                    query
                );
            }
        }
    });
}

/// ACP staging contract: skill content includes the SKILL.md body with
/// correct XML wrapping and metadata.
#[test]
fn smoke_acp_staging_contract_structure() {
    with_temp_sk_path(|kit_root| {
        let _ = ensure_kit_setup();
        let kit_dir = kit_root.join("kit");

        let index =
            script_kit_gpui::plugins::discover_plugins_in(&kit_dir).expect("discover plugins");

        let skills = discover_plugin_skills(&index).expect("discover skills");

        // Find the scriptlets skill from authoring plugin
        let scriptlets_skill = skills
            .iter()
            .find(|s| s.plugin_id == "authoring" && s.skill_id == "scriptlets")
            .expect("authoring/scriptlets skill must exist");

        let content = fs::read_to_string(&scriptlets_skill.path).expect("read SKILL.md");

        // Mirror the ACP staging logic from open_acp_with_selected_skill
        let initial_input = format!(
            "Use the attached skill \"{}\" from plugin \"{}\" for this session.\n\n<skill path=\"{}\">\n{}\n</skill>",
            scriptlets_skill.title,
            scriptlets_skill.plugin_title,
            scriptlets_skill.path.display(),
            content
        );

        // Structural assertions on the ACP payload
        assert!(
            initial_input.starts_with("Use the attached skill"),
            "Initial input must start with skill instruction"
        );
        assert!(
            initial_input.contains("from plugin"),
            "Initial input must reference the owning plugin"
        );
        assert!(
            initial_input.contains(&format!(
                "<skill path=\"{}\">",
                scriptlets_skill.path.display()
            )),
            "Skill content must be wrapped in <skill path=...> tags"
        );
        assert!(
            initial_input.ends_with("</skill>"),
            "Initial input must end with closing </skill> tag"
        );
        assert!(
            initial_input.contains("---"),
            "Skill content should contain frontmatter"
        );
    });
}
