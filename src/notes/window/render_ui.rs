use super::*;
use crate::ui_foundation::{compact_action_row, log_ui_action, UiActionSpec, UiSurface};

#[derive(Clone, Copy)]
struct NotesToolbarAction {
    spec: UiActionSpec,
    run: fn(&mut NotesApp, &mut Window, &mut Context<NotesApp>),
}

#[derive(Clone, Copy)]
struct NotesExportAction {
    spec: UiActionSpec,
    run: fn(&mut NotesApp, &mut Context<NotesApp>),
}

fn notes_apply_bold(app: &mut NotesApp, window: &mut Window, cx: &mut Context<NotesApp>) {
    app.insert_formatting("**", "**", window, cx);
}

fn notes_apply_italic(app: &mut NotesApp, window: &mut Window, cx: &mut Context<NotesApp>) {
    app.insert_formatting("_", "_", window, cx);
}

fn notes_apply_heading(app: &mut NotesApp, window: &mut Window, cx: &mut Context<NotesApp>) {
    app.cycle_heading(window, cx);
}

fn notes_apply_bullet_list(app: &mut NotesApp, window: &mut Window, cx: &mut Context<NotesApp>) {
    app.toggle_bullet_list(window, cx);
}

fn notes_apply_numbered_list(
    app: &mut NotesApp,
    window: &mut Window,
    cx: &mut Context<NotesApp>,
) {
    app.toggle_numbered_list(window, cx);
}

fn notes_apply_inline_code(app: &mut NotesApp, window: &mut Window, cx: &mut Context<NotesApp>) {
    app.insert_formatting("`", "`", window, cx);
}

fn notes_apply_code_block(app: &mut NotesApp, window: &mut Window, cx: &mut Context<NotesApp>) {
    app.insert_formatting("\n```\n", "\n```", window, cx);
}

fn notes_apply_strikethrough(
    app: &mut NotesApp,
    window: &mut Window,
    cx: &mut Context<NotesApp>,
) {
    app.insert_formatting("~~", "~~", window, cx);
}

fn notes_apply_checklist(app: &mut NotesApp, window: &mut Window, cx: &mut Context<NotesApp>) {
    app.toggle_checklist(window, cx);
}

fn notes_apply_link(app: &mut NotesApp, window: &mut Window, cx: &mut Context<NotesApp>) {
    app.insert_formatting("[", "](url)", window, cx);
}

fn notes_apply_rule(app: &mut NotesApp, window: &mut Window, cx: &mut Context<NotesApp>) {
    app.insert_horizontal_rule(window, cx);
}

fn notes_apply_blockquote(app: &mut NotesApp, window: &mut Window, cx: &mut Context<NotesApp>) {
    app.insert_formatting("\n> ", "", window, cx);
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

const NOTES_TOOLBAR_ACTIONS: [NotesToolbarAction; 12] = [
    NotesToolbarAction {
        spec: UiActionSpec {
            id: "bold",
            label: "B",
            shortcut: None,
        },
        run: notes_apply_bold,
    },
    NotesToolbarAction {
        spec: UiActionSpec {
            id: "italic",
            label: "I",
            shortcut: None,
        },
        run: notes_apply_italic,
    },
    NotesToolbarAction {
        spec: UiActionSpec {
            id: "heading",
            label: "H",
            shortcut: None,
        },
        run: notes_apply_heading,
    },
    NotesToolbarAction {
        spec: UiActionSpec {
            id: "list",
            label: "\u{2022}",
            shortcut: None,
        },
        run: notes_apply_bullet_list,
    },
    NotesToolbarAction {
        spec: UiActionSpec {
            id: "numbered-list",
            label: "1.",
            shortcut: None,
        },
        run: notes_apply_numbered_list,
    },
    NotesToolbarAction {
        spec: UiActionSpec {
            id: "code",
            label: "</>",
            shortcut: None,
        },
        run: notes_apply_inline_code,
    },
    NotesToolbarAction {
        spec: UiActionSpec {
            id: "codeblock",
            label: "```",
            shortcut: None,
        },
        run: notes_apply_code_block,
    },
    NotesToolbarAction {
        spec: UiActionSpec {
            id: "strikethrough",
            label: "S\u{0336}",
            shortcut: None,
        },
        run: notes_apply_strikethrough,
    },
    NotesToolbarAction {
        spec: UiActionSpec {
            id: "checklist",
            label: "\u{2610}",
            shortcut: None,
        },
        run: notes_apply_checklist,
    },
    NotesToolbarAction {
        spec: UiActionSpec {
            id: "link",
            label: "\u{1F517}",
            shortcut: None,
        },
        run: notes_apply_link,
    },
    NotesToolbarAction {
        spec: UiActionSpec {
            id: "rule",
            label: "\u{2015}",
            shortcut: None,
        },
        run: notes_apply_rule,
    },
    NotesToolbarAction {
        spec: UiActionSpec {
            id: "blockquote",
            label: ">",
            shortcut: None,
        },
        run: notes_apply_blockquote,
    },
];

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

    fn render_toolbar_button(
        &self,
        item: NotesToolbarAction,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        Button::new(item.spec.id)
            .ghost()
            .xsmall()
            .label(item.spec.label)
            .on_click(cx.listener(move |this, _, window, cx| {
                log_ui_action(UiSurface::NotesToolbar, item.spec, "click");
                (item.run)(this, window, cx);
            }))
    }

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

    /// Render the formatting toolbar
    pub(super) fn render_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        compact_action_row()
            .py_1()
            .px_3()
            .border_b_1()
            .border_color(cx.theme().border.opacity(OPACITY_SECTION_BORDER))
            .children(
                NOTES_TOOLBAR_ACTIONS
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

#[cfg(test)]
mod action_spec_tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn toolbar_action_ids_are_unique() {
        let mut ids = BTreeSet::new();
        for item in NOTES_TOOLBAR_ACTIONS.iter() {
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
