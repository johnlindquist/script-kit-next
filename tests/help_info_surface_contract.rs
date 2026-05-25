use std::fs;

#[test]
fn acp_empty_guidance_uses_shared_info_state() {
    let source =
        fs::read_to_string("src/ai/acp/view.rs").expect("failed to read src/ai/acp/view.rs");

    assert!(
        source.contains("render_acp_empty_guidance"),
        "ACP empty composer must use shared InfoState guidance"
    );
    assert!(
        !source.contains("Type / for skills"),
        "old weak ACP empty hint copy must not return"
    );
}

#[test]
fn shared_info_state_is_exported() {
    let source =
        fs::read_to_string("src/components/mod.rs").expect("failed to read src/components/mod.rs");

    assert!(source.contains("mod info_state"));
    assert!(source.contains("render_info_state"));
    assert!(source.contains("InfoStateSpec"));
}

#[test]
fn info_state_keeps_context_first_acp_copy() {
    let source = fs::read_to_string("src/components/info_state.rs")
        .expect("failed to read src/components/info_state.rs");

    assert!(source.contains("Ask with context"));
    assert!(source.contains("Use / for skills or @ to attach context"));
    assert!(source.contains("Attach files, scripts, clipboard, or history"));
    assert!(source.contains("⌘K shows every chat action."));
    assert!(!source.contains("⌘N new"));
    assert!(!source.contains("⌘W close"));
}

#[test]
fn launcher_empty_guidance_uses_shared_info_state() {
    let launcher = fs::read_to_string("src/render_script_list/mod.rs")
        .expect("failed to read src/render_script_list/mod.rs");
    let info = fs::read_to_string("src/components/info_state.rs")
        .expect("failed to read src/components/info_state.rs");

    assert!(launcher.contains("render_launcher_empty_or_no_results"));
    assert!(info.contains("launcher_empty_or_no_results_spec"));
    assert!(info.contains("No scripts yet"));
    assert!(info.contains("Tags need a syntax prefix"));
    assert!(info.contains("active filter is narrowing"));
    assert!(info.contains("scripts, scriptlets, snippets, and built-in commands"));
    assert!(!launcher.contains("No scripts or snippets found"));
    assert!(!launcher.contains("Press ⌘N to create a new script"));
}
