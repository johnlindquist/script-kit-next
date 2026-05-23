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

use std::sync::atomic::{AtomicUsize, Ordering};

static OPEN_STORYBOOK_WINDOWS: AtomicUsize = AtomicUsize::new(0);
static OPEN_STORYBOOK_CHILDREN: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorybookWindowRegistry {
    pub open_storybook_windows: usize,
    pub open_storybook_children: usize,
}

impl StorybookWindowRegistry {
    pub fn snapshot() -> Self {
        Self {
            open_storybook_windows: OPEN_STORYBOOK_WINDOWS.load(Ordering::SeqCst),
            open_storybook_children: OPEN_STORYBOOK_CHILDREN.load(Ordering::SeqCst),
        }
    }

    pub fn register_primary() -> Self {
        OPEN_STORYBOOK_WINDOWS.fetch_add(1, Ordering::SeqCst);
        Self::snapshot()
    }

    pub fn register_child() -> Self {
        OPEN_STORYBOOK_CHILDREN.fetch_add(1, Ordering::SeqCst);
        Self::snapshot()
    }

    pub fn unregister_primary() -> Self {
        let _ = OPEN_STORYBOOK_WINDOWS.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |count| {
            Some(count.saturating_sub(1))
        });
        Self::snapshot()
    }

    pub fn unregister_child() -> Self {
        let _ = OPEN_STORYBOOK_CHILDREN.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |count| {
            Some(count.saturating_sub(1))
        });
        Self::snapshot()
    }

    pub fn should_quit_after_close(&self) -> bool {
        self.open_storybook_windows == 0 && self.open_storybook_children == 0
    }
}

pub mod acp_chat_raycast_weight_studies;
pub mod acp_chat_states;
pub mod actions_dialog_presenter;
pub mod actions_dialog_states;
pub mod actions_dialog_variations;
pub mod adoption;
pub mod audit_report;
mod browser;
pub mod built_in_browser_states;
pub mod component_primitives_states;
pub mod confirm_popup_playground;
pub mod context_picker_popup_playground;
mod diagnostics;
pub mod dictation_states;
pub mod dictation_ui_variations;
pub mod footer_variations;
pub mod input_variations;
pub mod integrated_surface_shell;
mod layout;
pub mod main_menu_raycast_weight_studies;
pub mod main_menu_variations;
pub mod mini_ai_chat_presenter;
pub mod mini_ai_chat_states;
pub mod mini_ai_chat_variations;
pub mod non_list_state_showcase;
pub mod notes_window_states;
pub mod notes_window_variations;
pub(crate) mod playground_overlay_metrics;
pub mod quick_terminal_states;
mod registry;
mod selection;
pub mod shortcut_recorder_states;
mod story;
pub mod utility_builtin_states;

pub use acp_chat_states::{
    acp_chat_state_story_variants, render_acp_chat_state_compare_thumbnail,
    render_acp_chat_state_preview, AcpChatStateId,
};
pub use actions_dialog_presenter::{
    render_actions_dialog_presentation, ActionsDialogPresentationAction,
    ActionsDialogPresentationItem, ActionsDialogPresentationModel,
};
pub use actions_dialog_states::{
    actions_dialog_state_story_variants, render_actions_dialog_state_compare_thumbnail,
    render_actions_dialog_state_preview, ActionsDialogStateId,
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
    confirm_popup_playground_story_variants, render_confirm_popup_playground_compare_thumbnail,
    render_confirm_popup_playground_story_preview, ConfirmPopupPlaygroundId,
};
pub use context_picker_popup_playground::{
    context_picker_popup_playground_story_variants,
    render_context_picker_popup_playground_story_preview, ContextPickerPopupPlaygroundId,
    ContextPickerPopupSceneState, ContextPickerPopupTrigger,
};
pub use diagnostics::{
    build_adopted_surface_resolution_snapshot, build_story_catalog_snapshot,
    load_adopted_surface_resolution_snapshot, load_story_catalog_snapshot,
    AdoptedSurfaceResolutionEntry, AdoptedSurfaceResolutionSnapshot, StoryCatalogEntry,
    StoryCatalogSnapshot, StorySurfaceSummary, StoryVariantSummary,
};
pub use dictation_states::{
    dictation_state_story_variants, render_dictation_state_compare_thumbnail,
    render_dictation_state_gallery, render_dictation_state_story_preview,
};
pub use dictation_ui_variations::{
    dictation_ui_story_variants, render_dictation_ui_compare_thumbnail,
    render_dictation_ui_gallery, render_dictation_ui_story_preview,
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
pub use main_menu_raycast_weight_studies::adopted_main_menu_list_study_metrics;
pub use main_menu_variations::{
    adopted_main_menu_live_spec, adopted_main_menu_variant, main_menu_story_variants,
    render_main_menu_compare_thumbnail, render_main_menu_story_preview, resolve_main_menu_variant,
    MainMenuLiveSpec, MainMenuSurface, MainMenuVariationId, MainMenuVariationSpec,
    SPECS as MAIN_MENU_VARIATION_SPECS,
};
pub use mini_ai_chat_presenter::{
    render_mini_ai_chat_presentation, MiniAiChatPresentationMessage, MiniAiChatPresentationModel,
    MiniAiChatRole, MiniAiChatSuggestion,
};
pub use mini_ai_chat_states::{
    mini_ai_chat_state_story_variants, render_mini_ai_chat_state_compare_thumbnail,
    render_mini_ai_chat_state_preview, MiniAiChatStateId,
};
pub use mini_ai_chat_variations::{
    adopted_mini_ai_chat_style, mini_ai_chat_story_variants, render_mini_ai_chat_compare_thumbnail,
    render_mini_ai_chat_story_preview, resolve_mini_ai_chat_style, MiniAiChatStyle,
    MiniAiChatSurface, MiniAiChatVariationId, MiniAiChatVariationSpec,
    SPECS as MINI_AI_CHAT_VARIATION_SPECS,
};
pub use non_list_state_showcase::{
    non_list_state_showcase_story_variants, render_non_list_state_showcase_compare_thumbnail,
    render_non_list_state_showcase_preview, NonListStateShowcaseId,
};
pub use notes_window_states::{
    notes_window_state_story_variants, render_notes_window_state_compare_thumbnail,
    render_notes_window_state_preview, NotesWindowStateId,
};
pub use notes_window_variations::{
    adopted_notes_window_style, resolve_notes_window_style, NotesWindowSurface,
    NotesWindowVariationId, NotesWindowVariationSpec, SPECS as NOTES_WINDOW_VARIATION_SPECS,
};
pub use quick_terminal_states::{
    quick_terminal_state_story_variants, render_quick_terminal_state_compare_thumbnail,
    render_quick_terminal_state_preview, QuickTerminalStateId,
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
pub use shortcut_recorder_states::{
    render_shortcut_recorder_state_compare_thumbnail, render_shortcut_recorder_state_preview,
    shortcut_recorder_state_spec, shortcut_recorder_state_specs, ShortcutRecorderStateId,
    ShortcutRecorderStateSpec,
};
pub use story::{Story, StoryCatalogRole, StorySurface, StoryVariant};

pub use audit_report::{
    build_command_bar_consistency_report, build_prompt_chrome_consistency_report,
    build_workflow_affordance_consistency_report, render_command_bar_consistency_markdown,
    render_prompt_chrome_consistency_markdown, render_workflow_affordance_consistency_markdown,
    write_command_bar_consistency_report, write_prompt_chrome_consistency_report,
    write_standard_audit_reports, write_workflow_affordance_consistency_report, AuditFinding,
    AuditReport, AuditSeverity, AuditSurfaceResult,
};
pub use built_in_browser_states::{
    built_in_browser_state_story_variants, render_built_in_browser_state_compare_thumbnail,
    render_built_in_browser_state_preview, BuiltInBrowserStateId,
};
pub use component_primitives_states::{
    component_primitive_state_story_variants, render_component_primitive_state_compare_thumbnail,
    render_component_primitive_state_preview, ComponentPrimitiveStateId,
};
pub use utility_builtin_states::{
    render_utility_builtin_state_compare_thumbnail, render_utility_builtin_state_preview,
    utility_builtin_state_story_variants, UtilityBuiltinStateId,
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
