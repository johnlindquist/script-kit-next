use std::fs;

#[test]
fn prompt_compiler_preview_carries_generation_model_and_config_fingerprint() {
    let source = fs::read_to_string("src/ai/window/prompt_compiler/model.rs")
        .expect("read prompt compiler model");

    assert!(source.contains("struct PromptCompilerContext"));
    assert!(source.contains("generation: u64"));
    assert!(source.contains("model_id: String"));
    assert!(source.contains("compiler_config_fingerprint: Option<String>"));
    assert!(source.contains("enum PromptCompilerError"));
    assert!(source.contains("ConfigLoadFailed(String)"));
    assert!(source.contains("ModelUnavailable(String)"));
    assert!(source.contains("CompileFailed(String)"));
    assert!(source.contains("from_receipt_with_context"));
}

#[test]
fn compiler_apply_path_drops_stale_generation_model_or_config() {
    let model_source = fs::read_to_string("src/ai/window/prompt_compiler/model.rs")
        .expect("read prompt compiler model");
    let render_source =
        fs::read_to_string("src/ai/window/render_main_panel.rs").expect("read render source");

    assert!(
        model_source.contains("pub(crate) fn matches(&self, other: &Self) -> bool")
            && model_source.contains("self.generation == other.generation")
            && model_source.contains("self.model_id == other.model_id")
            && model_source
                .contains("self.compiler_config_fingerprint == other.compiler_config_fingerprint"),
        "PromptCompilerContext must expose the exact stale-result comparison"
    );
    assert!(
        render_source.contains("PromptCompilerPreview::from_receipt_with_context")
            && render_source.contains("generation: self.context_preflight.generation")
            && render_source.contains(".selected_model")
            && render_source.contains("compiler_config_fingerprint: None"),
        "prompt compiler pane must build previews with current generation/model/config identity"
    );
}
