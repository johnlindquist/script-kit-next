use gpui::{div, prelude::*, px, AnyElement, App, IntoElement, ParentElement, Styled};
use gpui_component::{
    input::Input,
    scroll::ScrollableElement,
    theme::{ActiveTheme, Theme},
    Sizable,
};

use crate::notes::markdown;

use super::types::NotesEditorLayout;
use super::NotesEditor;

impl NotesEditor {
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
        let editor = Input::new(&self.input_state)
            .h_full()
            .appearance(false)
            .font_family(cx.theme().mono_font_family.clone())
            .text_size(cx.theme().mono_font_size);

        div().h_full().child(editor).into_any_element()
    }

    pub(crate) fn sync_layout_from_metrics(&mut self, layout: NotesEditorLayout) {
        self.layout = layout;
    }
}
