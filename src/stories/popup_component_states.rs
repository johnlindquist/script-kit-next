use gpui::*;

use crate::storybook::{
    acp_chat_state_story_variants, notes_window_state_story_variants,
    render_acp_chat_state_compare_thumbnail, render_acp_chat_state_preview,
    render_confirm_popup_playground_compare_thumbnail,
    render_confirm_popup_playground_story_preview,
    render_context_picker_popup_playground_story_preview,
    render_notes_window_state_compare_thumbnail, render_notes_window_state_preview,
    render_shortcut_recorder_state_compare_thumbnail, render_shortcut_recorder_state_preview,
    shortcut_recorder_state_specs, ConfirmPopupPlaygroundId, ContextPickerPopupPlaygroundId, Story,
    StoryCatalogRole, StorySurface, StoryVariant,
};

pub struct ConfirmPopupStatesStory;

impl Story for ConfirmPopupStatesStory {
    fn id(&self) -> &'static str {
        "confirm-popup-states"
    }

    fn name(&self) -> &'static str {
        "Confirm Popup States"
    }

    fn category(&self) -> &'static str {
        "Popups"
    }

    fn catalog_role(&self) -> StoryCatalogRole {
        StoryCatalogRole::CanonicalState
    }

    fn surface(&self) -> StorySurface {
        StorySurface::ConfirmPopup
    }

    fn render(&self) -> AnyElement {
        render_confirm_popup_playground_story_preview(ConfirmPopupPlaygroundId::Current.as_str())
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        let source_variant = variant
            .props
            .get("sourceVariant")
            .map(String::as_str)
            .unwrap_or(ConfirmPopupPlaygroundId::Current.as_str());
        render_confirm_popup_playground_story_preview(source_variant)
    }

    fn render_compare_variant(&self, variant: &StoryVariant) -> AnyElement {
        let source_variant = variant
            .props
            .get("sourceVariant")
            .map(String::as_str)
            .unwrap_or(ConfirmPopupPlaygroundId::Current.as_str());
        render_confirm_popup_playground_compare_thumbnail(source_variant)
    }

    fn variants(&self) -> Vec<StoryVariant> {
        crate::storybook::confirm_popup_playground_story_variants()
            .into_iter()
            .map(|variant| {
                let source_variant = variant.stable_id();
                StoryVariant::default_named(source_variant.clone(), variant.name)
                    .description(variant.description.unwrap_or_default())
                    .with_prop("surface", "confirmPopup")
                    .with_prop("representation", "presenterFixture")
                    .with_prop("sourceVariant", source_variant)
            })
            .collect()
    }
}

pub struct ContextPickerPopupStatesStory;

impl Story for ContextPickerPopupStatesStory {
    fn id(&self) -> &'static str {
        "context-picker-popup-states"
    }

    fn name(&self) -> &'static str {
        "Context Picker Popup States"
    }

    fn category(&self) -> &'static str {
        "Popups"
    }

    fn catalog_role(&self) -> StoryCatalogRole {
        StoryCatalogRole::CanonicalState
    }

    fn surface(&self) -> StorySurface {
        StorySurface::ContextPickerPopup
    }

    fn render(&self) -> AnyElement {
        render_context_picker_popup_playground_story_preview(
            ContextPickerPopupPlaygroundId::MentionWhisperDense.as_str(),
        )
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        let source_variant = variant
            .props
            .get("sourceVariant")
            .map(String::as_str)
            .unwrap_or(ContextPickerPopupPlaygroundId::MentionWhisperDense.as_str());
        render_context_picker_popup_playground_story_preview(source_variant)
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            context_picker_variant(
                "mention-results",
                "Mention Results",
                "Mention picker with results and a focused row.",
                "mention",
                ContextPickerPopupPlaygroundId::MentionWhisperDense,
            ),
            context_picker_variant(
                "mention-empty",
                "Mention Empty",
                "Mention picker no-match recovery state.",
                "mention",
                ContextPickerPopupPlaygroundId::MentionEmptyState,
            ),
            context_picker_variant(
                "slash-results",
                "Slash Results",
                "Slash command picker with results and a focused command.",
                "slash",
                ContextPickerPopupPlaygroundId::SlashWhisperDense,
            ),
            context_picker_variant(
                "slash-empty",
                "Slash Empty",
                "Slash command picker no-match recovery state.",
                "slash",
                ContextPickerPopupPlaygroundId::SlashEmptyState,
            ),
        ]
    }
}

pub struct ShortcutRecorderStatesStory;

impl Story for ShortcutRecorderStatesStory {
    fn id(&self) -> &'static str {
        "shortcut-recorder-states"
    }

    fn name(&self) -> &'static str {
        "Shortcut Recorder States"
    }

    fn category(&self) -> &'static str {
        "Popups"
    }

    fn catalog_role(&self) -> StoryCatalogRole {
        StoryCatalogRole::CanonicalState
    }

    fn surface(&self) -> StorySurface {
        StorySurface::ShortcutRecorder
    }

    fn render(&self) -> AnyElement {
        render_shortcut_recorder_state_preview("empty")
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_shortcut_recorder_state_preview(&variant.stable_id())
    }

    fn render_compare_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_shortcut_recorder_state_compare_thumbnail(&variant.stable_id())
    }

    fn variants(&self) -> Vec<StoryVariant> {
        shortcut_recorder_state_specs()
            .into_iter()
            .map(|spec| {
                StoryVariant::default_named(spec.id.as_str(), spec.id.name())
                    .description(spec.id.description())
                    .with_prop("surface", "shortcutRecorder")
                    .with_prop("representation", "presenterFixture")
            })
            .collect()
    }
}

pub struct NotesWindowStatesStory;

impl Story for NotesWindowStatesStory {
    fn id(&self) -> &'static str {
        "notes-window-states"
    }

    fn name(&self) -> &'static str {
        "Notes Window States"
    }

    fn category(&self) -> &'static str {
        "Windows"
    }

    fn catalog_role(&self) -> StoryCatalogRole {
        StoryCatalogRole::CanonicalState
    }

    fn surface(&self) -> StorySurface {
        StorySurface::NotesWindow
    }

    fn render(&self) -> AnyElement {
        render_notes_window_state_preview("editor")
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_notes_window_state_preview(&variant.stable_id())
    }

    fn render_compare_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_notes_window_state_compare_thumbnail(&variant.stable_id())
    }

    fn variants(&self) -> Vec<StoryVariant> {
        notes_window_state_story_variants()
    }
}

pub struct AcpChatStatesStory;

impl Story for AcpChatStatesStory {
    fn id(&self) -> &'static str {
        "acp-chat-states"
    }

    fn name(&self) -> &'static str {
        "Agent Chat States"
    }

    fn category(&self) -> &'static str {
        "AI"
    }

    fn catalog_role(&self) -> StoryCatalogRole {
        StoryCatalogRole::CanonicalState
    }

    fn surface(&self) -> StorySurface {
        StorySurface::AcpChat
    }

    fn render(&self) -> AnyElement {
        render_acp_chat_state_preview("conversation")
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_acp_chat_state_preview(&variant.stable_id())
    }

    fn render_compare_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_acp_chat_state_compare_thumbnail(&variant.stable_id())
    }

    fn variants(&self) -> Vec<StoryVariant> {
        acp_chat_state_story_variants()
    }
}

fn context_picker_variant(
    id: &'static str,
    name: &'static str,
    description: &'static str,
    trigger: &'static str,
    source_variant: ContextPickerPopupPlaygroundId,
) -> StoryVariant {
    StoryVariant::default_named(id, name)
        .description(description)
        .with_prop("surface", "contextPickerPopup")
        .with_prop("representation", "presenterFixture")
        .with_prop("trigger", trigger)
        .with_prop("sourceVariant", source_variant.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popup_component_stories_are_canonical_non_runtime_fixtures() {
        let stories: Vec<&dyn Story> = vec![
            &ConfirmPopupStatesStory,
            &ContextPickerPopupStatesStory,
            &ShortcutRecorderStatesStory,
            &NotesWindowStatesStory,
            &AcpChatStatesStory,
        ];

        for story in stories {
            assert_eq!(story.catalog_role(), StoryCatalogRole::CanonicalState);
            assert!(story.variants().len() >= 2);

            for variant in story.variants() {
                assert_eq!(
                    variant.props.get("representation").map(String::as_str),
                    Some("presenterFixture")
                );
                assert!(!variant.props.contains_key("fixtureImagePresent"));
                assert!(!variant.props.contains_key("fixtureManifestPresent"));
            }
        }
    }
}
