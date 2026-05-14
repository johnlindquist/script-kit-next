use std::{fs, path::Path};

fn read(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
}

#[test]
fn acp_popup_registration_facade_registers_and_removes() {
    let source = read("src/ai/acp/popup_registry.rs");
    assert!(source.contains("pub(crate) struct AcpPopupRegistration"));
    assert!(source.contains("pub(crate) fn register"));
    assert!(source.contains("upsert_runtime_window_handle"));
    assert!(source.contains("remove_runtime_window_handle"));
    assert!(source.contains("remove_automation_window"));
    assert!(source.contains("impl Drop for AcpPopupRegistration"));
}

#[test]
fn all_acp_prompt_popups_hold_scoped_registration() {
    for (path, id) in [
        ("src/ai/acp/picker_popup.rs", "acp-mention-popup"),
        (
            "src/ai/acp/model_selector_popup.rs",
            "acp-model-selector-popup",
        ),
        ("src/ai/acp/history_popup.rs", "acp-history-popup"),
    ] {
        let source = read(path);
        assert!(source.contains(id), "{path} missing {id}");
        assert!(
            source.contains("AcpPopupRegistration::register"),
            "{path} must register through ACP popup facade"
        );
        assert!(
            source.contains("_registration: super::popup_registry::AcpPopupRegistration"),
            "{path} must keep the Drop guard in the popup slot"
        );
    }
}
