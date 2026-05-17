const AI_PRESETS: &str = include_str!("../src/render_builtins/ai_presets.rs");

#[test]
fn ai_preset_search_empty_state_copy_is_modeled() {
    assert!(
        AI_PRESETS.contains("enum AiPresetSearchEmptyState")
            && AI_PRESETS.contains("NoPresetsAvailable")
            && AI_PRESETS.contains("NoFilteredMatches"),
        "AI Presets empty-state copy should use named states"
    );
    assert!(
        AI_PRESETS.contains("fn from_filter(filter: &str) -> Self")
            && AI_PRESETS.contains("fn message(self) -> &'static str"),
        "AI Presets empty states should own filter classification and visible copy"
    );
    assert!(
        AI_PRESETS.contains("AiPresetSearchEmptyState::from_filter(&filter).message()"),
        "AI Presets renderer should derive empty-state copy from the model"
    );
    assert!(
        !AI_PRESETS.contains("child(if filter.is_empty()"),
        "AI Presets empty-state copy must not regress to inline filter-empty branching"
    );
}

#[test]
fn ai_preset_model_selection_is_modeled() {
    assert!(
        AI_PRESETS.contains("enum AiPresetModelSelection<'a>")
            && AI_PRESETS.contains("ProviderDefault")
            && AI_PRESETS.contains("Explicit(&'a str)"),
        "AI preset model input should use named model-selection states"
    );
    assert!(
        AI_PRESETS.contains("fn from_input(model: &'a str) -> Self")
            && AI_PRESETS.contains("fn as_create_arg(self) -> Option<&'a str>"),
        "AI preset model-selection state should own create_preset argument conversion"
    );
    assert!(
        AI_PRESETS.contains("AiPresetModelSelection::from_input(model.as_str()).as_create_arg()"),
        "create preset form should derive model argument through the named state"
    );
    assert!(
        !AI_PRESETS.contains("let model_val = if model.trim().is_empty()"),
        "AI preset create form must not regress to inline blank-model branching"
    );
}
