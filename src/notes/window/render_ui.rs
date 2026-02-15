use super::*;

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

    /// Render the formatting toolbar
    pub(super) fn render_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_1()
            .py_1()
            .px_3() // Align horizontally with titlebar & footer
            .border_b_1() // Subtle bottom border — mirrors footer top border
            .border_color(cx.theme().border.opacity(OPACITY_SECTION_BORDER))
            .child(
                Button::new("bold")
                    .ghost()
                    .xsmall()
                    .label("B")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.insert_formatting("**", "**", window, cx);
                    })),
            )
            .child(
                Button::new("italic")
                    .ghost()
                    .xsmall()
                    .label("I")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.insert_formatting("_", "_", window, cx);
                    })),
            )
            .child(
                Button::new("heading")
                    .ghost()
                    .xsmall()
                    .label("H")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.cycle_heading(window, cx);
                    })),
            )
            .child(
                Button::new("list")
                    .ghost()
                    .xsmall()
                    .label("•")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.toggle_bullet_list(window, cx);
                    })),
            )
            .child(
                Button::new("numbered-list")
                    .ghost()
                    .xsmall()
                    .label("1.")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.toggle_numbered_list(window, cx);
                    })),
            )
            .child(
                Button::new("code")
                    .ghost()
                    .xsmall()
                    .label("</>")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.insert_formatting("`", "`", window, cx);
                    })),
            )
            .child(
                Button::new("codeblock")
                    .ghost()
                    .xsmall()
                    .label("```")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.insert_formatting("\n```\n", "\n```", window, cx);
                    })),
            )
            .child(
                Button::new("strikethrough")
                    .ghost()
                    .xsmall()
                    .label("S\u{0336}")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.insert_formatting("~~", "~~", window, cx);
                    })),
            )
            .child(
                Button::new("checklist")
                    .ghost()
                    .xsmall()
                    .label("\u{2610}")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.toggle_checklist(window, cx);
                    })),
            )
            .child(
                Button::new("link")
                    .ghost()
                    .xsmall()
                    .label("\u{1F517}")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.insert_formatting("[", "](url)", window, cx);
                    })),
            )
            .child(
                Button::new("rule")
                    .ghost()
                    .xsmall()
                    .label("\u{2015}")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.insert_horizontal_rule(window, cx);
                    })),
            )
            .child(
                Button::new("blockquote")
                    .ghost()
                    .xsmall()
                    .label(">")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.insert_formatting("\n> ", "", window, cx);
                    })),
            )
    }

    /// Render the export menu
    pub(super) fn render_export_menu(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .gap_1()
            .child(
                Button::new("export-txt")
                    .ghost()
                    .xsmall()
                    .label("TXT")
                    .on_click(cx.listener(|this, _, _, _cx| {
                        this.export_note(ExportFormat::PlainText);
                    })),
            )
            .child(
                Button::new("export-md")
                    .ghost()
                    .xsmall()
                    .label("MD")
                    .on_click(cx.listener(|this, _, _, _cx| {
                        this.export_note(ExportFormat::Markdown);
                    })),
            )
            .child(
                Button::new("export-html")
                    .ghost()
                    .xsmall()
                    .label("HTML")
                    .on_click(cx.listener(|this, _, _, _cx| {
                        this.export_note(ExportFormat::Html);
                    })),
            )
    }
}
