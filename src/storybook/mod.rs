//! Storybook - Component preview system for script-kit-gpui
//!
//! This module provides a component preview system for GPUI components.
//!
//! # Components
//!
//! - [`Story`] - Trait for defining previewable stories
//! - [`StoryBrowser`] - Main UI for browsing stories
//! - [`story_container`], [`story_section`], etc. - Layout helpers
//!

pub mod actions_dialog_presenter;
pub mod actions_dialog_variations;
pub mod adoption;
pub mod audit_report;
mod browser;
pub mod confirm_popup_playground;
pub mod context_picker_popup_playground;
mod diagnostics;
pub mod footer_variations;
pub mod input_variations;
pub mod integrated_surface_shell;
mod layout;
pub mod main_menu_variations;
pub mod mini_ai_chat_presenter;
pub mod mini_ai_chat_variations;
pub(crate) mod playground_overlay_metrics;
mod registry;
mod selection;
mod story;

pub use actions_dialog_presenter::{
    render_actions_dialog_presentation, ActionsDialogPresentationAction,
    ActionsDialogPresentationItem, ActionsDialogPresentationModel,
};
pub use actions_dialog_variations::{
    actions_dialog_story_variants, adopted_actions_dialog_style, resolve_actions_dialog_style,
    ActionsDialogStyle, ActionsDialogSurface, ActionsDialogVariationId, ActionsDialogVariationSpec,
    SPECS as ACTIONS_DIALOG_VARIATION_SPECS,
};
pub use adoption::{
    adopted_surface_live, resolve_surface_live, AdoptableSurface, SurfaceSelectionResolution,
    VariationId,
};
pub use browser::StoryBrowser;
pub use confirm_popup_playground::{
    confirm_popup_playground_story_variants, render_confirm_popup_playground_story_preview,
    ConfirmPopupPlaygroundId,
};
pub use context_picker_popup_playground::{
    context_picker_popup_playground_story_variants,
    render_context_picker_popup_playground_story_preview, ContextPickerPopupPlaygroundId,
    ContextPickerPopupSceneState, ContextPickerPopupTrigger,
};
pub use diagnostics::{
    build_story_catalog_snapshot, load_story_catalog_snapshot, StoryCatalogEntry,
    StoryCatalogSnapshot, StorySurfaceSummary, StoryVariantSummary,
};
pub use footer_variations::{
    config_from_footer_variation_spec, config_from_storybook_footer_selection,
    config_from_storybook_footer_selection_value, footer_story_variants, footer_variation_specs,
    render_footer_slot_text, render_footer_story_preview, resolve_footer_selection,
    resolve_footer_selection_spec, FooterSelectionResolution, FooterVariationId,
    FooterVariationSpec,
};
pub use input_variations::{
    adopted_input_variation, adopted_input_variation_id, input_story_variants,
    input_variation_specs, render_input_story_preview, InputVariationId, InputVariationSpec,
};
pub use integrated_surface_shell::{
    IntegratedOverlayAnchor, IntegratedOverlayPlacement, IntegratedSurfaceShell,
    IntegratedSurfaceShellConfig,
};
pub use layout::{code_block, story_container, story_divider, story_item, story_section};
pub use main_menu_variations::{
    main_menu_story_variants, render_main_menu_compare_thumbnail, render_main_menu_story_preview,
    MainMenuVariationId,
};
pub use mini_ai_chat_presenter::{
    render_mini_ai_chat_presentation, MiniAiChatPresentationMessage, MiniAiChatPresentationModel,
    MiniAiChatRole, MiniAiChatSuggestion,
};
pub use mini_ai_chat_variations::{
    adopted_mini_ai_chat_style, mini_ai_chat_story_variants, resolve_mini_ai_chat_style,
    MiniAiChatStyle, MiniAiChatSurface, MiniAiChatVariationId, MiniAiChatVariationSpec,
    SPECS as MINI_AI_CHAT_VARIATION_SPECS,
};
pub use registry::{
    all_categories, all_stories, first_story_with_multiple_variants, stories_by_category,
    stories_by_surface, StoryEntry,
};
pub(crate) use selection::selection_store_path;
pub use selection::{
    load_selected_story_variant, load_story_selections, save_selected_story_variant,
    save_story_selections, StorySelectionStore, StorySelectionWriteResult,
};
pub use story::{Story, StorySurface, StoryVariant};

pub use audit_report::{
    build_command_bar_consistency_report, build_prompt_chrome_consistency_report,
    build_workflow_affordance_consistency_report, render_command_bar_consistency_markdown,
    render_prompt_chrome_consistency_markdown, render_workflow_affordance_consistency_markdown,
    write_command_bar_consistency_report, write_prompt_chrome_consistency_report,
    write_standard_audit_reports, write_workflow_affordance_consistency_report, AuditFinding,
    AuditReport, AuditSeverity, AuditSurfaceResult,
};

/// Machine-readable error payload for `--catalog-json` failures.
#[derive(Debug, serde::Serialize)]
pub struct StorybookJsonError<'a> {
    pub schema_version: u8,
    pub ok: bool,
    pub error: StorybookJsonErrorBody<'a>,
}

/// Structured body within [`StorybookJsonError`].
#[derive(Debug, serde::Serialize)]
pub struct StorybookJsonErrorBody<'a> {
    pub kind: &'a str,
    pub message: String,
    pub hint: &'a str,
}

#[cfg(test)]
mod catalog_json_tests {
    use super::*;

    #[test]
    fn storybook_json_error_payload_is_machine_readable() {
        let payload = StorybookJsonError {
            schema_version: 1,
            ok: false,
            error: StorybookJsonErrorBody {
                kind: "catalog_load_failed",
                message: "boom".to_string(),
                hint: "Run cargo check.",
            },
        };

        let json = serde_json::to_string(&payload).expect("serialize error payload");
        let value: serde_json::Value = serde_json::from_str(&json).expect("parse error payload");

        assert_eq!(value["schema_version"], 1);
        assert_eq!(value["ok"], false);
        assert_eq!(value["error"]["kind"], "catalog_load_failed");
        assert_eq!(value["error"]["message"], "boom");
    }
}
