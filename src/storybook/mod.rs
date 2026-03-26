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

mod browser;
mod diagnostics;
pub mod footer_variations;
pub mod input_variations;
pub mod main_menu_variations;
mod layout;
mod registry;
mod selection;
mod story;

pub use browser::StoryBrowser;
pub use diagnostics::{
    build_story_catalog_snapshot, load_story_catalog_snapshot, StoryCatalogEntry,
    StoryCatalogSnapshot, StorySurfaceSummary, StoryVariantSummary,
};
pub use footer_variations::{
    config_from_footer_variation_spec,
    config_from_storybook_footer_selection, config_from_storybook_footer_selection_value,
    footer_story_variants, footer_variation_specs, render_footer_story_preview,
    render_footer_slot_text, resolve_footer_selection, resolve_footer_selection_spec,
    FooterSelectionResolution, FooterVariationId, FooterVariationSpec,
};
pub use input_variations::{
    adopted_input_variation, adopted_input_variation_id, input_story_variants,
    input_variation_specs, render_input_story_preview, InputVariationId, InputVariationSpec,
};
pub use main_menu_variations::{
    main_menu_story_variants, render_main_menu_story_preview, MainMenuVariationId,
};
pub use layout::{code_block, story_container, story_divider, story_item, story_section};
pub use registry::{
    all_categories, all_stories, first_story_with_multiple_variants, stories_by_category,
    stories_by_surface, StoryEntry,
};
pub use selection::{
    load_selected_story_variant, load_story_selections, save_selected_story_variant,
    save_story_selections, StorySelectionStore, StorySelectionWriteResult,
};
pub(crate) use selection::selection_store_path;
pub use story::{Story, StorySurface, StoryVariant};

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
        let value: serde_json::Value =
            serde_json::from_str(&json).expect("parse error payload");

        assert_eq!(value["schema_version"], 1);
        assert_eq!(value["ok"], false);
        assert_eq!(value["error"]["kind"], "catalog_load_failed");
        assert_eq!(value["error"]["message"], "boom");
    }
}
