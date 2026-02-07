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
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground.opacity(OPACITY_MUTED))
                                    .child("⌘/  shortcuts"),
                            ),
                    ),
                )
                .into_any_element();
        }

        if is_preview {
            let content = self.editor_state.read(cx).value().to_string();
            return div()
                .id("notes-markdown-preview")
                .flex_1()
                .min_h(px(0.))
                .overflow_y_scrollbar()
                .px_4()
                .py_3()
                .child(markdown::render_markdown_preview(&content, cx.theme()))
                .into_any_element();
        }

        Input::new(&self.editor_state)
            .h_full()
            .appearance(false)
            .font_family(cx.theme().mono_font_family.clone())
            .text_size(cx.theme().mono_font_size)
            .into_any_element()
    }
}
