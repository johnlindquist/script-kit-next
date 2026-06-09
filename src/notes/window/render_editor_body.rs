use super::*;

impl NotesApp {
    pub(super) fn render_editor_body(
        &self,
        is_trash: bool,
        has_selection: bool,
        is_preview: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let no_notes = self.get_visible_notes().is_empty();

        if no_notes && !has_selection && is_trash {
            return div()
                .id("notes-empty-trash")
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_4()
                .child(
                    div()
                        .text_base()
                        .text_color(cx.theme().muted_foreground)
                        .child("Trash is empty"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground.opacity(OPACITY_SUBTLE))
                        .child("Deleted notes will appear here"),
                )
                .child(
                    div()
                        .id("back-to-notes-link")
                        .text_xs()
                        .text_color(cx.theme().accent)
                        .cursor_pointer()
                        .hover(|s| s.text_color(cx.theme().foreground))
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.set_view_mode(NotesViewMode::AllNotes, window, cx);
                        }))
                        .child("← Back to Notes"),
                )
                .into_any_element();
        }

        if no_notes && !has_selection {
            return div()
                .id("notes-empty-state")
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_3()
                .child(
                    div()
                        .text_base()
                        .text_color(cx.theme().muted_foreground.opacity(OPACITY_MUTED))
                        .child("No notes yet"),
                )
                .child(
                    div()
                        .id("create-first-note")
                        .text_sm()
                        .text_color(cx.theme().accent)
                        .cursor_pointer()
                        .hover(|s| s.text_color(cx.theme().foreground))
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.create_note(window, cx);
                        }))
                        .child("Create your first note"),
                )
                .child(
                    div().flex().flex_col().items_center().gap_1().pt_2().child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground.opacity(OPACITY_MUTED))
                                    .child("⌘N  new"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground.opacity(OPACITY_MUTED))
                                    .child("⌘⇧N  from clipboard"),
                            ),
                    ),
                )
                .into_any_element();
        }

        if is_preview {
            let content = self.editor_state.read(cx).value().to_string();
            let metrics = style::adopted_metrics();
            return div()
                .id("notes-markdown-preview")
                .flex_1()
                .min_h(px(0.))
                .track_scroll(&self.preview_scroll_handle)
                .overflow_y_scroll()
                .vertical_scrollbar(&self.preview_scroll_handle)
                .px(px(metrics.editor_padding_x))
                .py(px(metrics.editor_padding_y))
                .child(markdown::render_markdown_preview(&content, cx.theme()))
                .into_any_element();
        }

        let editor = Input::new(&self.editor_state)
            .h_full()
            .appearance(false)
            .font_family(cx.theme().mono_font_family.clone())
            .text_size(cx.theme().mono_font_size);

        div()
            .relative()
            .h_full()
            .child(editor)
            .when_some(self.notes_ghost_prediction.as_ref(), |this, prediction| {
                this.child(self.render_notes_ghost_overlay(prediction, cx))
            })
            .into_any_element()
    }

    fn render_notes_ghost_overlay(
        &self,
        prediction: &crate::notes::ghost::NotesGhostPrediction,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let metrics = style::adopted_metrics();
        let prefix_cols = prediction.query_prefix.chars().count() as f32;
        let line_index = self
            .get_cursor_line_info(cx)
            .map(|(line, _)| line.saturating_sub(1))
            .unwrap_or(0) as f32;
        // The editor is monospace, so one measured advance positions any
        // column exactly; measure it from the actual mono font instead of
        // assuming 7.4px.
        let text_system = cx.text_system();
        let mono_font_id =
            text_system.resolve_font(&gpui::font(cx.theme().mono_font_family.clone()));
        let mono_advance = text_system
            .em_advance(mono_font_id, cx.theme().mono_font_size)
            .map(f32::from)
            .ok()
            .filter(|advance| advance.is_finite() && *advance > 0.0)
            .unwrap_or(7.4);
        let x = metrics.editor_padding_x + prefix_cols * mono_advance;
        let y = metrics.editor_padding_y + line_index * metrics.auto_resize_line_height;

        div()
            .id("notes-ghost-autocomplete")
            .absolute()
            .left(px(x))
            .top(px(y))
            .font_family(cx.theme().mono_font_family.clone())
            .text_size(cx.theme().mono_font_size)
            .text_color(cx.theme().muted_foreground.opacity(OPACITY_SUBTLE))
            .child(prediction.suffix.clone())
            .into_any_element()
    }
}
