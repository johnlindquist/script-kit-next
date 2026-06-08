const SCRIPT_TEMPLATES: &str = include_str!("../src/render_builtins/script_templates.rs");

#[test]
fn script_template_catalog_copy_promises_local_creation() {
    assert!(
        SCRIPT_TEMPLATES.contains("Create Local Script"),
        "Script Template Catalog primary action should promise local script creation"
    );
    assert!(
        SCRIPT_TEMPLATES.contains("show_naming_dialog_for_script_template"),
        "Script Template Catalog Enter path should hand off to the naming dialog"
    );
    assert!(
        SCRIPT_TEMPLATES.contains("render_script_template_file"),
        "Script Template Catalog should document the rendered local file owner"
    );
}

#[test]
fn script_template_catalog_keeps_shared_surface_contract() {
    assert!(
        SCRIPT_TEMPLATES.contains("render_expanded_view_scaffold_with_footer"),
        "Script Template Catalog should stay on the shared expanded view scaffold"
    );
    assert!(
        SCRIPT_TEMPLATES.contains("render_simple_hint_strip"),
        "Script Template Catalog should use the shared footer hint strip"
    );
    assert!(
        SCRIPT_TEMPLATES.contains("emit_prompt_chrome_audit"),
        "Script Template Catalog should emit the shared chrome audit"
    );
}

#[test]
fn script_template_empty_state_copy_is_modeled() {
    assert!(
        SCRIPT_TEMPLATES.contains("enum ScriptTemplateCatalogEmptyState")
            && SCRIPT_TEMPLATES.contains("NoTemplatesAvailable")
            && SCRIPT_TEMPLATES.contains("NoFilteredMatches"),
        "Script Template catalog empty-state copy should use named states"
    );
    assert!(
        SCRIPT_TEMPLATES.contains("fn from_filter(filter: &str) -> Self")
            && SCRIPT_TEMPLATES.contains("fn message(self) -> &'static str"),
        "Script Template catalog empty states should own filter classification and visible copy"
    );
    assert!(
        SCRIPT_TEMPLATES.contains("ScriptTemplateCatalogEmptyState::from_filter(filter)")
            && SCRIPT_TEMPLATES.contains("state.message()"),
        "Script Template catalog renderer should derive empty-state copy from the model"
    );
    assert!(
        !SCRIPT_TEMPLATES.contains("child(if filter.trim().is_empty()"),
        "Script Template empty-state copy must not regress to inline filter-empty branching"
    );
}

#[test]
fn script_template_row_description_copy_is_modeled() {
    assert!(
        SCRIPT_TEMPLATES.contains("fn script_template_catalog_row_description("),
        "Script Template row description fallback should have one owner"
    );
    assert!(
        SCRIPT_TEMPLATES.contains("Self::script_template_catalog_row_description(template)"),
        "Script Template row rendering should use the shared description helper"
    );
    assert!(
        !SCRIPT_TEMPLATES.contains("let description = if template.description.is_empty()"),
        "Script Template row description must not regress to inline description-empty branching"
    );
}
