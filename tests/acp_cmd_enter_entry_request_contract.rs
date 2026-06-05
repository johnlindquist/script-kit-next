use std::{fs, path::Path};

fn read(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
}

fn rust_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

#[test]
fn acp_entry_request_is_the_cmd_enter_handoff_choke_point() {
    let source = read("src/app_impl/tab_ai_mode/acp_entry.rs");
    for needle in [
        "pub(crate) enum AcpEntryOrigin",
        "pub(crate) enum AcpThreadTarget",
        "pub(crate) enum AcpSeedPolicy",
        "pub(crate) struct AcpEntryRequest",
        "pub(crate) fn open_acp_chat_from_entry_request",
        "blocks_launcher_ai_entry()",
    ] {
        assert!(source.contains(needle), "missing {needle}");
    }
    let body_start = source
        .find("pub(crate) fn open_acp_chat_from_entry_request")
        .expect("entry request opener should exist");
    let body = &source[body_start..];
    assert!(
        body.contains("self.seed_acp_return_origin_for_view(&source_view)")
            && body.contains("event = \"acp_entry_request_open\""),
        "entry request opener must seed return origin and log the handoff before launch dispatch"
    );
    for forbidden in [
        "self.current_view = AppView::AcpChatView",
        "ensure_embedded_ai_window(",
        "transition_acp_surface(",
    ] {
        assert!(
            !body.contains(forbidden),
            "entry request opener must not mutate the ACP surface directly: {forbidden}"
        );
    }
}

#[test]
fn cmd_enter_origins_have_real_source_callers() {
    let tab_ai = read("src/app_impl/tab_ai_mode/mod.rs");
    let notes = read("src/notes/window/keyboard.rs");
    for needle in [
        "AcpEntryOrigin::MainLauncher",
        "AcpEntryOrigin::LauncherTab",
        "AcpEntryOrigin::FileSearch",
        "AcpEntryOrigin::ActionsDialog",
        "AcpEntryOrigin::PluginSkill",
    ] {
        assert!(tab_ai.contains(needle), "missing caller for {needle}");
    }
    assert!(
        notes.contains("open_selected_note_cart_in_embedded_acp"),
        "Notes Cmd+Enter must stay on the Notes-owned embedded ACP path"
    );
}

#[test]
fn file_search_and_actions_do_not_stage_context_parts_directly() {
    for dir in ["src/file_search", "src/actions"] {
        let mut files = Vec::new();
        rust_files(&Path::new(env!("CARGO_MANIFEST_DIR")).join(dir), &mut files);
        for path in files {
            let src = fs::read_to_string(&path).unwrap();
            assert!(
                !src.contains("thread.add_context_part"),
                "{} must route ACP context through AcpEntryRequest/staging helpers",
                path.display()
            );
        }
    }
}
