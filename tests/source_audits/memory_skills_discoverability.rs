//! Source audit for the de-scoped OpenClicky "memory/skills drawer" idea.
//!
//! The chosen direction is to keep skills/files discoverable through main input
//! sigils and existing pickers, not add a separate drawer UI.

use super::read_source as read;

#[test]
fn context_subsearch_keeps_files_and_skills_in_main_input_path() {
    let context = read("src/spine/catalog_context.rs");
    let subsearch = read("src/spine/catalog_subsearch.rs");

    for expected in [
        r#"prefix: "file""#,
        r#"title: "Files""#,
        r#"prefix: "skills""#,
        r#"title: "Skills""#,
        r#"subtitle: "Search plugin skills""#,
    ] {
        assert!(
            context.contains(expected),
            "context catalog must keep `{expected}` discoverable"
        );
    }

    assert!(
        subsearch.contains(r#""file" => Some(Self::File)"#)
            && subsearch.contains(r#""skills" => Some(Self::Skills)"#)
            && subsearch.contains(r#"Self::File => "file""#)
            && subsearch.contains(r#"Self::Skills => "skills""#),
        "context subsearch prefixes must route file and skills through the same main-input mechanism"
    );
}

#[test]
fn root_filter_keeps_type_skill_qualifier_without_new_drawer() {
    let filter = read("src/spine/catalog_filter.rs");

    assert!(
        filter.contains(r#"token: "type:skill""#)
            && filter.contains(r#"title: "Skills only""#)
            && filter.contains(r#"subtitle: "Find agent skills""#),
        "root search must keep skill filtering discoverable through typed qualifiers"
    );
}

#[test]
fn acp_slash_skills_stage_context_parts_instead_of_opening_drawer() {
    let view = read("src/ai/acp/view.rs");

    assert!(
        view.contains("discover_plugin_skills(&index)")
            && view.contains("acp_slash_skill_cataloged")
            && view.contains("SlashCommandPayload::PluginSkill(skill)")
            && view.contains("build_skill_slash_command_text(&skill.skill_id)")
            && view.contains("build_skill_context_part(")
            && view.contains("thread.add_context_part(part, cx)"),
        "ACP slash skill acceptance must stay in the slash/context-part path"
    );
}

#[test]
fn existing_skill_contract_tests_cover_duplicate_slugs_and_staging() {
    let plugin_skill_search = read("tests/plugin_skill_search.rs");
    let acp_tests = read("src/ai/acp/tests.rs");

    assert!(
        plugin_skill_search.contains("duplicate_skill_slugs_across_plugins_are_distinct_results")
            && plugin_skill_search.contains("skill_frecency_key_is_plugin_qualified"),
        "plugin skill search tests must keep duplicate skill slugs distinct"
    );
    assert!(
        acp_tests.contains("acp_plugin_slash_accept_stages_selected_skill_prompt")
            && acp_tests.contains("acp_claude_skill_staged_prompt_uses_claude_owner_phrase"),
        "ACP tests must keep skill slash acceptance staged as context parts"
    );
}
