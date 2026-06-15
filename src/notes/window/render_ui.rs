use super::*;

use crate::components::notes_editor::{NotesEditorToolbarAction, NOTES_EDITOR_TOOLBAR_ACTIONS};
use crate::list_item::FONT_MONO;
use crate::ui_foundation::{compact_action_row, log_ui_action, UiActionSpec, UiSurface};
use gpui::FontWeight;

/// Opacity for toolbar section borders — matches Notes window token.
const OPACITY_SECTION_BORDER: f32 = 0.2;

impl NotesApp {
    pub(super) fn render_kit_resource_preview(&self, cx: &mut Context<Self>) -> AnyElement {
        let Some(preview) = self.kit_resource_preview.as_ref() else {
            return div().into_any_element();
        };

        let title = preview.title.clone();
        let uri = preview.uri.clone();
        let copy_uri = uri.clone();
        let mime_type = preview.mime_type.clone();
        let text = preview.text.clone();
        let truncated = preview.truncated;

        div()
            .id("notes-kit-resource-preview")
            .flex_1()
            .min_h(px(0.))
            .flex()
            .flex_col()
            .gap_3()
            .p_3()
            .child(
                div()
                    .flex()
                    .items_start()
                    .justify_between()
                    .gap_3()
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.))
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .id("notes-kit-resource-preview-title")
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(title),
                            )
                            .child(
                                div()
                                    .id("notes-kit-resource-preview-uri")
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(uri),
                            )
                            .child(
                                div()
                                    .id("notes-kit-resource-preview-meta")
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground.opacity(OPACITY_MUTED))
                                    .child(format!(
                                        "{mime_type} · read-only{}",
                                        if truncated { " · truncated" } else { "" }
                                    )),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .id("notes-kit-resource-preview-copy-uri")
                                    .text_xs()
                                    .text_color(cx.theme().accent)
                                    .cursor_pointer()
                                    .hover(|s| s.text_color(cx.theme().foreground))
                                    .on_click(cx.listener(move |_this, _, _window, cx| {
                                        cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                                            copy_uri.clone(),
                                        ));
                                    }))
                                    .child("Copy URI"),
                            )
                            .child(
                                div()
                                    .id("notes-kit-resource-preview-close")
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .cursor_pointer()
                                    .hover(|s| s.text_color(cx.theme().foreground))
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.close_kit_resource_preview(window, cx);
                                    }))
                                    .child("Close"),
                            ),
                    ),
            )
            .child(
                div()
                    .id("notes-kit-resource-preview-body")
                    .flex_1()
                    .min_h(px(0.))
                    .overflow_y_scroll()
                    .rounded(px(6.))
                    .border_1()
                    .border_color(cx.theme().border.opacity(OPACITY_SECTION_BORDER))
                    .p_3()
                    .text_xs()
                    .font_family(FONT_MONO)
                    .text_color(cx.theme().foreground)
                    .child(text),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground.opacity(OPACITY_MUTED))
                    .child("Esc to return"),
            )
            .into_any_element()
    }

    fn render_toolbar_button(
        &self,
        item: NotesEditorToolbarAction,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        Button::new(item.spec.id)
            .ghost()
            .xsmall()
            .label(item.spec.label)
            .on_click(cx.listener(move |this, _, window, cx| {
                log_ui_action(UiSurface::NotesToolbar, item.spec, "click");
                this.notes_editor.update(cx, |editor, cx| {
                    (item.run)(editor, window, cx);
                });
                this.has_unsaved_changes = true;
            }))
    }

    /// Render the formatting toolbar backed by the shared notes editor component.
    pub(super) fn render_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        compact_action_row()
            .py_1()
            .px_3()
            .border_b_1()
            .border_color(cx.theme().border.opacity(OPACITY_SECTION_BORDER))
            .children(
                NOTES_EDITOR_TOOLBAR_ACTIONS
                    .iter()
                    .copied()
                    .map(|item| self.render_toolbar_button(item, cx)),
            )
    }

    /// Render the export menu
    pub(super) fn render_export_menu(&self, cx: &mut Context<Self>) -> impl IntoElement {
        compact_action_row().children(
            NOTES_EXPORT_ACTIONS
                .iter()
                .copied()
                .map(|item| self.render_export_button(item, cx)),
        )
    }
}

#[derive(Clone, Copy)]
struct NotesExportAction {
    spec: UiActionSpec,
    run: fn(&mut NotesApp, &mut Context<NotesApp>),
}

fn notes_export_plain_text(app: &mut NotesApp, cx: &mut Context<NotesApp>) {
    app.export_note(ExportFormat::PlainText, cx);
}

fn notes_export_markdown(app: &mut NotesApp, cx: &mut Context<NotesApp>) {
    app.export_note(ExportFormat::Markdown, cx);
}

fn notes_export_html(app: &mut NotesApp, cx: &mut Context<NotesApp>) {
    app.export_note(ExportFormat::Html, cx);
}

const NOTES_EXPORT_ACTIONS: [NotesExportAction; 3] = [
    NotesExportAction {
        spec: UiActionSpec {
            id: "export-txt",
            label: "TXT",
            shortcut: None,
        },
        run: notes_export_plain_text,
    },
    NotesExportAction {
        spec: UiActionSpec {
            id: "export-md",
            label: "MD",
            shortcut: None,
        },
        run: notes_export_markdown,
    },
    NotesExportAction {
        spec: UiActionSpec {
            id: "export-html",
            label: "HTML",
            shortcut: None,
        },
        run: notes_export_html,
    },
];

impl NotesApp {
    fn render_export_button(
        &self,
        item: NotesExportAction,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        Button::new(item.spec.id)
            .ghost()
            .xsmall()
            .label(item.spec.label)
            .on_click(cx.listener(move |this, _, _, cx| {
                log_ui_action(UiSurface::NotesExportMenu, item.spec, "click");
                (item.run)(this, cx);
            }))
    }

    pub(super) fn format_search_match_counter(
        note_position: Option<(usize, usize)>,
        total_matches: usize,
    ) -> String {
        let current_match = note_position.map(|(position, _)| position).unwrap_or(0);
        format!("{current_match}/{total_matches}")
    }

    /// Render the search input bar (shown when Cmd+F is pressed)
    pub(super) fn render_search(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let sk_theme = crate::theme::get_cached_theme();
        let opacity = sk_theme.get_opacity();
        let total_matches = self.get_visible_notes().len();
        let counter_text =
            Self::format_search_match_counter(self.get_note_position(), total_matches);
        let search_surface = rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
            sk_theme.colors.background.search_box,
            opacity.search_box,
        ));
        let counter_surface = rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
            sk_theme.colors.accent.selected_subtle,
            OPACITY_SUBTLE,
        ));

        div().w_full().px_3().pt_2().pb_2().child(
            Input::new(&self.search_state)
                .w_full()
                .small()
                .prefix(IconName::Search)
                .suffix(
                    div()
                        .h(px(18.))
                        .px_2()
                        .rounded_full()
                        .bg(counter_surface)
                        .flex()
                        .items_center()
                        .text_xs()
                        .text_color(theme.muted_foreground.opacity(OPACITY_MUTED))
                        .child(counter_text),
                )
                .bg(search_surface)
                .border_color(theme.border.opacity(OPACITY_SECTION_BORDER)),
        )
    }
}

#[cfg(test)]
mod action_spec_tests {
    use super::*;
    use crate::components::notes_editor::NOTES_EDITOR_TOOLBAR_ACTIONS;
    use std::collections::BTreeSet;

    #[test]
    fn toolbar_action_ids_are_unique() {
        let mut ids = BTreeSet::new();
        for item in NOTES_EDITOR_TOOLBAR_ACTIONS.iter() {
            assert!(
                ids.insert(item.spec.id),
                "duplicate toolbar action id: {}",
                item.spec.id
            );
        }
    }

    #[test]
    fn export_action_ids_are_unique() {
        let mut ids = BTreeSet::new();
        for item in NOTES_EXPORT_ACTIONS.iter() {
            assert!(
                ids.insert(item.spec.id),
                "duplicate export action id: {}",
                item.spec.id
            );
        }
    }
}
