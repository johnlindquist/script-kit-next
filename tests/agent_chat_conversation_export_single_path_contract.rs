use std::{fs, path::Path};

fn read(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
}

#[test]
fn conversation_export_builder_is_single_public_path() {
    let source = read("src/ai/agent_chat/ui/conversation_export.rs");
    assert!(source.contains("pub(crate) struct AgentChatConversationExport"));
    assert!(source.contains("pub(crate) enum AgentChatExportPurpose"));
    assert_eq!(
        source.matches("pub(crate) fn export_conversation").count(),
        1
    );
    assert!(source.contains("stable_export_id"));
    assert!(source.contains("HashSet"));
}
