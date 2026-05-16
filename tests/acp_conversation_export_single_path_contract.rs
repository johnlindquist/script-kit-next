use std::{fs, path::Path};

fn read(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
}

#[test]
// doc-anchor-removed: [[removed-docs export dedupe#Single export path]]
// doc-anchor-removed: [[removed-docs export dedupe#Stable context part dedupe]]
// doc-anchor-removed: [[removed-docs export dedupe#No duplicate seeded user message]]
fn conversation_export_builder_is_single_public_path() {
    let source = read("src/ai/acp/conversation_export.rs");
    assert!(source.contains("pub(crate) struct AcpConversationExport"));
    assert!(source.contains("pub(crate) enum AcpExportPurpose"));
    assert_eq!(
        source.matches("pub(crate) fn export_conversation").count(),
        1
    );
    assert!(source.contains("stable_export_id"));
    assert!(source.contains("HashSet"));
}

#[test]
fn markdown_export_uses_conversation_export_from_thread() {
    let export = read("src/ai/acp/export.rs");
    assert!(export.contains("build_acp_conversation_markdown_from_thread"));
    assert!(export.contains("thread.export_conversation(AcpExportPurpose::CopyTranscript)"));

    for path in [
        "src/app_actions/handle_action/mod.rs",
        "src/ai/acp/chat_window.rs",
        "src/notes/window/acp_host.rs",
    ] {
        let source = read(path);
        assert!(
            source.contains("build_acp_conversation_markdown_from_thread"),
            "{path} must export via the single ACP conversation builder"
        );
    }
}
