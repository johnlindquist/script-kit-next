use super::*;

impl NotesApp {
    /// Render the search input bar (shown when Cmd+F is pressed)
    pub(super) fn render_search(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let has_query = !self.search_query.is_empty();
        let result_count = if has_query {
            self.notes
                .iter()
                .filter(|n| {
                    n.content
                        .to_lowercase()
                        .contains(&self.search_query.to_lowercase())
                })
                .count()
        } else {
            self.notes.len()
        };

        div()
            .w_full()
            .px_3()
            .py_1() // 4px — tighter to match toolbar density
            .flex()
            .items_center()
            .gap_2()
            .border_b_1()
            .border_color(theme.border.opacity(OPACITY_SECTION_BORDER))
            .child(
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground.opacity(OPACITY_MUTED))
                    .child("\u{2315}"), // ⌕ magnifying glass text char
            )
            .child(
                div().flex_1().child(
                    Input::new(&self.search_state)
                        .w_full()
                        .small()
                        .appearance(false),
                ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground.opacity(OPACITY_MUTED))
                    .child(format!("{} notes", result_count)),
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
