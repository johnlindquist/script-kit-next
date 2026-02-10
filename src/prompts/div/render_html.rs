use super::*;

/// Render a vector of HtmlElements as a GPUI Div
pub(super) fn render_elements(elements: &[HtmlElement], ctx: RenderContext) -> Div {
    let mut container = div().flex().flex_col().gap_2().w_full();

    for element in elements {
        container = container.child(render_element(element, ctx.clone()));
    }

    container
}

/// Render a single HtmlElement as a GPUI element
fn render_element(element: &HtmlElement, ctx: RenderContext) -> Div {
    match element {
        HtmlElement::Text(text) => {
            // Text is a block with the text content
            div()
                .w_full()
                .text_color(rgb(ctx.text_secondary))
                .child(text.clone())
        }

        HtmlElement::Header { level, children } => {
            let font_size = match level {
                1 => 28.0,
                2 => 24.0,
                3 => 20.0,
                4 => 18.0,
                5 => 16.0,
                _ => 14.0,
            };

            // User-specified pixel size - not converted to rem
            div()
                .w_full()
                .text_size(px(font_size))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(ctx.text_primary))
                .mb(px(8.0))
                .child(render_inline_content(children, &ctx))
        }

        HtmlElement::Paragraph(children) => div()
            .w_full()
            .text_sm()
            .text_color(rgb(ctx.text_secondary))
            .mb(px(8.0))
            .child(render_inline_content(children, &ctx)),

        HtmlElement::Bold(children) => {
            let style = DivInlineStyle {
                bold: true,
                ..Default::default()
            };
            let mut segments = Vec::new();
            append_inline_segments(children, &style, &mut segments);
            div()
                .w_full()
                .child(render_inline_segments(&segments, &ctx))
        }

        HtmlElement::Italic(children) => {
            let style = DivInlineStyle {
                italic: true,
                ..Default::default()
            };
            let mut segments = Vec::new();
            append_inline_segments(children, &style, &mut segments);
            div()
                .w_full()
                .child(render_inline_segments(&segments, &ctx))
        }

        HtmlElement::InlineCode(code) => div()
            .px(px(6.0))
            .py(px(2.0))
            .bg(rgba((ctx.code_bg << 8) | 0x80))
            .rounded(px(4.0))
            .font_family("Menlo")
            .text_sm()
            .text_color(rgb(ctx.accent_color))
            .child(code.clone()),

        HtmlElement::CodeBlock { language, code } => {
            let mut block = div()
                .w_full()
                .p(px(12.0))
                .mb(px(8.0))
                .bg(rgba((ctx.code_bg << 8) | 0xC0))
                .rounded(px(6.0))
                .flex()
                .flex_col()
                .gap_1();

            if let Some(lang) = language.as_ref().filter(|lang| !lang.is_empty()) {
                block = block.child(
                    div()
                        .text_xs()
                        .text_color(rgb(ctx.text_tertiary))
                        .font_weight(FontWeight::MEDIUM)
                        .child(lang.clone()),
                );
            }

            block.child(
                div()
                    .font_family("Menlo")
                    .text_sm()
                    .text_color(rgb(ctx.text_primary))
                    .child(code.clone()),
            )
        }

        HtmlElement::UnorderedList(items) => {
            let mut list = div()
                .flex()
                .flex_col()
                .gap_1()
                .mb(px(8.0))
                .pl(px(16.0))
                .w_full();

            for item in items {
                if let HtmlElement::ListItem(children) = item {
                    list = list.child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_2()
                            .w_full()
                            .child(
                                div().text_color(rgb(ctx.text_tertiary)).child("\u{2022}"), // Bullet point
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .text_color(rgb(ctx.text_secondary))
                                    .child(render_inline_content(children, &ctx)),
                            ),
                    );
                }
            }

            list
        }

        HtmlElement::OrderedList(items) => {
            let mut list = div()
                .flex()
                .flex_col()
                .gap_1()
                .mb(px(8.0))
                .pl(px(16.0))
                .w_full();

            for (index, item) in items.iter().enumerate() {
                if let HtmlElement::ListItem(children) = item {
                    list = list.child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_2()
                            .w_full()
                            .child(
                                div()
                                    .text_color(rgb(ctx.text_tertiary))
                                    .min_w(px(20.0))
                                    .child(format!("{}.", index + 1)),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .text_color(rgb(ctx.text_secondary))
                                    .child(render_inline_content(children, &ctx)),
                            ),
                    );
                }
            }

            list
        }

        HtmlElement::ListItem(children) => {
            // Standalone list item (shouldn't normally happen, but handle gracefully)
            div()
                .w_full()
                .text_color(rgb(ctx.text_secondary))
                .child(render_inline_content(children, &ctx))
        }

        HtmlElement::Blockquote(children) => div()
            .w_full()
            .pl(px(16.0))
            .py(px(8.0))
            .mb(px(8.0))
            .border_l_4()
            .border_color(rgb(ctx.quote_border))
            .text_color(rgb(ctx.text_tertiary))
            .child(render_inline_content(children, &ctx)),

        HtmlElement::HorizontalRule => div().w_full().h(px(1.0)).my(px(12.0)).bg(rgb(ctx.hr_color)),

        HtmlElement::Link { href, children } => {
            let style = DivInlineStyle {
                link_href: Some(href.clone()),
                ..Default::default()
            };
            let mut segments = Vec::new();
            append_inline_segments(children, &style, &mut segments);
            div()
                .w_full()
                .child(render_inline_segments(&segments, &ctx))
        }

        HtmlElement::LineBreak => {
            div().h(px(8.0)) // Line break spacing
        }

        HtmlElement::Div { classes, children } => {
            let base = render_elements(children, ctx.clone());
            if let Some(class_str) = classes {
                apply_tailwind_styles(base, class_str)
            } else {
                base
            }
        }

        HtmlElement::Span { classes, children } => {
            let base = render_elements(children, ctx.clone());
            if let Some(class_str) = classes {
                apply_tailwind_styles(base, class_str)
            } else {
                base
            }
        }
    }
}
