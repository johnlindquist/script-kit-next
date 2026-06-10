use std::{fs, path::Path};

fn read(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
}

#[test]
fn main_menu_skill_stages_on_target_agent_chat_view() {
    let tab_ai = read("src/app_impl/agent_handoff/mod.rs");
    let body = tab_ai
        .split("pub(crate) fn open_agent_chat_with_selected_skill")
        .nth(1)
        .expect("open_agent_chat_with_selected_skill exists");
    assert!(body.contains("get_detached_agent_chat_view_entity"));
    assert!(body.contains("stage_selected_plugin_skill_from_main_menu"));
    assert!(!body.contains("thread.add_context_part"));
}

#[test]
fn skill_context_identity_is_bound_to_thread() {
    let thread = read("src/ai/agent_chat/ui/thread.rs");
    assert!(thread.contains("pub(crate) struct SkillContextIdentity"));
    assert!(thread.contains("pub(crate) fn add_or_replace_skill_context"));

    let view = read("src/ai/agent_chat/ui/view.rs");
    assert!(view.contains("ui_thread_id().to_string()"));
    assert!(view.contains("add_or_replace_skill_context(identity, part, cx)"));
}
