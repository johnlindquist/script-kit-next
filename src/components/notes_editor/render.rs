use gpui::{div, prelude::*, px, AnyElement, App, Entity, IntoElement, ParentElement, Styled};
use gpui_component::{
    input::{Editor as Input, EditorState as InputState},
    scroll::ScrollableElement,
    theme::{ActiveTheme, Theme},
};

use crate::notes::markdown;

use super::NotesEditor;

impl NotesEditor {
    /// Render a host-owned accessory on the same horizontal plane as editor
    /// text. The host owns the accessory's contents and vertical budget; the
    /// shared editor remains the sole owner of its horizontal inset.
    pub fn render_content_accessory(&self, accessory: AnyElement) -> AnyElement {
        let layout = self.layout;

        div()
            .w_full()
            .flex_none()
            .px(px(layout.padding_x))
            .pb(px(layout.padding_y))
            .child(accessory)
            .into_any_element()
    }

    /// Render the markdown preview surface.
    pub fn render_preview(
        &self,
        content: &str,
        on_toggle_task: markdown::TaskToggleHandler,
        theme: &Theme,
    ) -> AnyElement {
        let layout = self.layout;

        div()
            .id("notes-markdown-preview")
            .debug_selector(|| "notes-markdown-preview".to_string())
            .flex_1()
            .min_h(px(0.))
            .track_scroll(&self.preview_scroll_handle)
            .overflow_y_scroll()
            .vertical_scrollbar(&self.preview_scroll_handle)
            .px(px(layout.padding_x))
            .py(px(layout.padding_y))
            .child(markdown::render_markdown_preview_interactive(
                content,
                theme,
                on_toggle_task,
            ))
            .into_any_element()
    }

    /// Render the editable markdown input surface.
    pub fn render_input(&self, cx: &App) -> AnyElement {
        let layout = self.layout;

        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h(px(0.))
            .h_full()
            .px(px(layout.padding_x))
            .py(px(layout.padding_y))
            .child(Self::render_input_state(&self.input_state, cx))
            .into_any_element()
    }

    pub fn render_input_state(input_state: &Entity<InputState>, cx: &App) -> AnyElement {
        let editor = Input::new(input_state)
            .h_full()
            .appearance(false)
            // gpui-kit gives multi-line editors no inner padding; keep the
            // Notes text inset the contract publishes.
            .px(px(super::contract::NOTES_EDITOR_INPUT_PADDING_X_PX))
            .py(px(super::contract::NOTES_EDITOR_INPUT_PADDING_Y_PX))
            .font_family(cx.theme().mono_font_family.clone())
            .text_size(cx.theme().mono_font_size)
            // Keep the fork's 20pt line box (gpui-kit Editor defaults to 1.5×).
            .line_height(gpui::rems(
                crate::theme::gpui_kit_bridge::KIT_MULTILINE_EDITOR_LINE_HEIGHT_REMS,
            ));

        div()
            .debug_selector(|| "notes-editor-input-viewport".to_string())
            .h_full()
            .child(editor)
            .into_any_element()
    }
}
