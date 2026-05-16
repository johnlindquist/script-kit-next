//! Contract for MCP-backed catalog row projection.
//!
//! SDK Reference and Script Template Catalog rows should be projected by
//! named helpers before render, state, elements, and Tab AI read them.

use script_kit_gpui::mcp_resources::{
    self, ScriptTemplateMetadataDefaults, ScriptTemplateRef, SdkFunctionRef, SdkSupport,
};

const MCP_RESOURCES: &str = include_str!("../src/mcp_resources/mod.rs");
const SDK_REFERENCE_RENDER: &str = include_str!("../src/render_builtins/sdk_reference.rs");
const SCRIPT_TEMPLATES_RENDER: &str = include_str!("../src/render_builtins/script_templates.rs");
const COLLECT_ELEMENTS: &str = include_str!("../src/app_layout/collect_elements.rs");
const PROMPT_HANDLER: &str = include_str!("../src/prompt_handler/mod.rs");
const TAB_AI_MODE: &str = include_str!("../src/app_impl/tab_ai_mode/mod.rs");
const UI_WINDOW: &str = include_str!("../src/app_impl/ui_window.rs");

fn source_between<'a>(source: &'a str, start_pat: &str, end_pat: &str) -> &'a str {
    let start = source
        .find(start_pat)
        .unwrap_or_else(|| panic!("missing source start: {start_pat}"));
    let end_rel = source[start..]
        .find(end_pat)
        .unwrap_or_else(|| panic!("missing source end: {end_pat}"));
    &source[start..start + end_rel]
}

fn sdk_entry(name: &str, category: &str, description: &str) -> SdkFunctionRef {
    SdkFunctionRef {
        name: name.to_string(),
        signature: format!("await {name}(): Promise<void>"),
        description: description.to_string(),
        category: category.to_string(),
        support: SdkSupport::Supported,
        unsupported_note: None,
    }
}

fn template(id: &str, title: &str, category: &str, description: &str) -> ScriptTemplateRef {
    ScriptTemplateRef {
        id: id.to_string(),
        title: title.to_string(),
        description: description.to_string(),
        category: category.to_string(),
        filename_hint: format!("{id}.ts"),
        body_template: "console.log({{NAME}});".to_string(),
        metadata_defaults: ScriptTemplateMetadataDefaults::default(),
    }
}

// doc-anchor-removed: [[removed-docs Surface Matrix]]
#[test]
fn sdk_reference_visible_rows_preserve_display_and_source_indices() {
    let entries = vec![
        sdk_entry("arg", "input", "Prompt for text"),
        sdk_entry("notify", "system", "Show a notification"),
        sdk_entry("select", "input", "Prompt with choices"),
    ];

    let rows = mcp_resources::sdk_reference_visible_rows(&entries, "input");
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].display_index, 0);
    assert_eq!(rows[0].source_index, 0);
    assert_eq!(rows[0].entry.name, "arg");
    assert_eq!(rows[1].display_index, 1);
    assert_eq!(rows[1].source_index, 2);
    assert_eq!(rows[1].entry.name, "select");

    assert_eq!(
        mcp_resources::sdk_reference_dataset_and_visible_counts(&entries, "input"),
        (3, 2)
    );
    assert_eq!(
        mcp_resources::sdk_reference_selected_visible_entry(&entries, "input", 1)
            .map(|row| row.entry.name.as_str()),
        Some("select")
    );
}

#[test]
fn script_template_visible_rows_preserve_display_and_source_indices() {
    let templates = vec![
        template("blank", "Blank Starter", "starter", "Empty script"),
        template("choice", "Choice Prompt", "prompts", "Pick an option"),
        template("date", "Today Note", "notes", "Create today note"),
    ];

    let rows = mcp_resources::script_template_catalog_visible_rows(&templates, "prompt");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].display_index, 0);
    assert_eq!(rows[0].source_index, 1);
    assert_eq!(rows[0].template.id, "choice");

    assert_eq!(
        mcp_resources::script_template_catalog_dataset_and_visible_counts(&templates, "prompt"),
        (3, 1)
    );
    assert_eq!(
        mcp_resources::script_template_catalog_selected_visible_template(&templates, "prompt", 0)
            .map(|row| row.template.title.as_str()),
        Some("Choice Prompt")
    );
}

#[test]
fn mcp_resource_declares_catalog_projection_helper_families() {
    for required in [
        "pub struct SdkReferenceVisibleRow",
        "pub fn sdk_reference_visible_rows",
        "pub fn sdk_reference_visible_row_names",
        "pub fn sdk_reference_dataset_and_visible_counts",
        "pub fn sdk_reference_selected_visible_entry",
        "pub fn sdk_reference_visible_target_rows",
        "pub struct ScriptTemplateCatalogVisibleRow",
        "pub fn script_template_catalog_visible_rows",
        "pub fn script_template_catalog_visible_row_names",
        "pub fn script_template_catalog_dataset_and_visible_counts",
        "pub fn script_template_catalog_selected_visible_template",
        "pub fn script_template_catalog_visible_target_rows",
    ] {
        assert!(
            MCP_RESOURCES.contains(required),
            "MCP catalog projection owner must contain: {required}"
        );
    }
}

#[test]
fn catalog_renderers_use_projection_helpers() {
    assert!(
        SDK_REFERENCE_RENDER.contains("sdk_reference_visible_rows(&entries, filter)"),
        "SDK Reference render path must use the named visible-row helper"
    );
    assert!(
        !SDK_REFERENCE_RENDER.contains("filter_sdk_reference_entries("),
        "SDK Reference render path must not call the raw filter directly"
    );

    assert!(
        SCRIPT_TEMPLATES_RENDER
            .contains("script_template_catalog_visible_rows(&templates, filter)"),
        "Script Template render path must use the named visible-row helper"
    );
    assert!(
        !SCRIPT_TEMPLATES_RENDER.contains("filter_script_template_entries("),
        "Script Template render path must not call the raw filter directly"
    );
}

#[test]
fn catalog_state_elements_and_sizing_use_projection_helpers() {
    let elements_body = source_between(
        COLLECT_ELEMENTS,
        "AppView::SdkReferenceView {\n                filter,",
        "\n            AppView::ScriptTemplateCatalogView",
    );
    assert!(elements_body.contains("sdk_reference_visible_row_names(entries, filter)"));

    let template_elements_body = source_between(
        COLLECT_ELEMENTS,
        "AppView::ScriptTemplateCatalogView {\n                filter,",
        "\n            AppView::EmojiPickerView",
    );
    assert!(template_elements_body.contains("script_template_catalog_visible_row_names("));

    let state_body = source_between(
        PROMPT_HANDLER,
        "AppView::SdkReferenceView {\n                        filter,",
        "\n                    AppView::ScriptTemplateCatalogView",
    );
    assert!(state_body.contains("sdk_reference_dataset_and_visible_counts("));
    assert!(state_body.contains("sdk_reference_selected_visible_entry("));

    let template_state_body = source_between(
        PROMPT_HANDLER,
        "AppView::ScriptTemplateCatalogView {\n                        filter,",
        "\n                };",
    );
    assert!(template_state_body.contains("script_template_catalog_dataset_and_visible_counts("));
    assert!(template_state_body.contains("script_template_catalog_selected_visible_template("));

    assert!(UI_WINDOW.contains("sdk_reference_dataset_and_visible_counts(entries, filter)"));
    assert!(
        UI_WINDOW.contains("script_template_catalog_dataset_and_visible_counts(\n                        templates, filter,")
    );
}

#[test]
fn catalog_tab_ai_targets_use_projection_helpers() {
    let sdk_arm = source_between(
        TAB_AI_MODE,
        "AppView::SdkReferenceView {\n                filter,",
        "\n            AppView::ScriptTemplateCatalogView",
    );
    assert!(TAB_AI_MODE.contains("fn tab_ai_target_from_sdk_reference_row("));
    assert!(sdk_arm.contains("sdk_reference_selected_visible_entry("));
    assert!(sdk_arm.contains("sdk_reference_visible_target_rows("));

    let template_arm = source_between(
        TAB_AI_MODE,
        "AppView::ScriptTemplateCatalogView {\n                filter,",
        "\n            AppView::ScriptList =>",
    );
    assert!(TAB_AI_MODE.contains("fn tab_ai_target_from_script_template_catalog_row("));
    assert!(template_arm.contains("script_template_catalog_selected_visible_template("));
    assert!(template_arm.contains("script_template_catalog_visible_target_rows("));
}
