use std::{fs, path::Path};

fn read(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
}

#[test]
// @lat: [[lat.md/tests/acp-portal-contract#Plugin skill target-thread contract#Main menu and slash picker are equivalent]]
// @lat: [[lat.md/tests/acp-portal-contract#Plugin skill target-thread contract#Detached reuse stages into detached thread]]
fn main_menu_skill_stages_on_target_acp_view() {
    let tab_ai = read("src/app_impl/tab_ai_mode/mod.rs");
    let body = tab_ai
        .split("pub(crate) fn open_acp_with_selected_skill")
        .nth(1)
        .expect("open_acp_with_selected_skill exists");
    assert!(body.contains("get_detached_acp_view_entity"));
    assert!(body.contains("stage_selected_plugin_skill_from_main_menu"));
    assert!(!body.contains("thread.add_context_part"));
}

#[test]
// @lat: [[lat.md/tests/acp-portal-contract#Plugin skill target-thread contract#Agent switch revalidates staged skill]]
fn skill_context_identity_is_bound_to_thread_and_revalidated_on_agent_switch() {
    let thread = read("src/ai/acp/thread.rs");
    assert!(thread.contains("pub(crate) struct SkillContextIdentity"));
    assert!(thread.contains("pub(crate) fn add_or_replace_skill_context"));
    assert!(thread.contains("pub(crate) fn revalidate_skill_context_for_agent"));

    let view = read("src/ai/acp/view.rs");
    assert!(view.contains("ui_thread_id().to_string()"));
    assert!(view.contains("add_or_replace_skill_context(identity, part, cx)"));
    assert!(view.contains("relaunch_for_agent_switch_preserving_draft"));

    let actions = read("src/app_actions/handle_action/mod.rs");
    assert!(actions.contains("relaunch_for_agent_switch_preserving_draft"));
}
