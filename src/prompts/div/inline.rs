use super::*;
use crate::list_item::FONT_MONO;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct DivInlineStyle {
    pub(super) bold: bool,
    pub(super) italic: bool,
    pub(super) code: bool,
    pub(super) link_href: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DivInlineSegment {
    pub(super) text: String,
    pub(super) style: DivInlineStyle,
}

pub(super) fn collect_inline_segments(elements: &[HtmlElement]) -> Vec<DivInlineSegment> {
    let mut segments = Vec::new();
    append_inline_segments(elements, &DivInlineStyle::default(), &mut segments);
    segments
}

pub(super) fn append_inline_segments(
    elements: &[HtmlElement],
    style: &DivInlineStyle,
    out: &mut Vec<DivInlineSegment>,
) {
    for element in elements {
        match element {
            HtmlElement::Text(text) => push_inline_segment(out, text.clone(), style.clone()),
            HtmlElement::Bold(children) => {
                let mut nested = style.clone();
                nested.bold = true;
                append_inline_segments(children, &nested, out);
            }
            HtmlElement::Italic(children) => {
                let mut nested = style.clone();
                nested.italic = true;
                append_inline_segments(children, &nested, out);
            }
            HtmlElement::InlineCode(code) => {
                let mut nested = style.clone();
                nested.code = true;
                push_inline_segment(out, code.clone(), nested);
            }
            HtmlElement::Link { href, children } => {
                let mut nested = style.clone();
                nested.link_href = Some(href.clone());
                append_inline_segments(children, &nested, out);
            }
            HtmlElement::LineBreak => push_inline_segment(out, "\n".to_string(), style.clone()),
            HtmlElement::Header { children, .. }
            | HtmlElement::Paragraph(children)
            | HtmlElement::ListItem(children)
            | HtmlElement::Blockquote(children)
            | HtmlElement::Div { children, .. }
            | HtmlElement::Span { children, .. } => append_inline_segments(children, style, out),
            HtmlElement::UnorderedList(items) | HtmlElement::OrderedList(items) => {
                for (idx, item) in items.iter().enumerate() {
                    if idx > 0 {
                        push_inline_segment(out, "\n".to_string(), style.clone());
                    }
                    if let HtmlElement::ListItem(children) = item {
                        append_inline_segments(children, style, out);
                    }
                }
            }
            HtmlElement::CodeBlock { code, .. } => {
                push_inline_segment(out, code.clone(), style.clone())
            }
            HtmlElement::HorizontalRule => {
                push_inline_segment(out, "---".to_string(), style.clone())
            }
        }
    }
}

fn push_inline_segment(out: &mut Vec<DivInlineSegment>, text: String, style: DivInlineStyle) {
    if text.is_empty() {
        return;
    }

    if let Some(last) = out.last_mut() {
        if last.style == style {
            last.text.push_str(&text);
            return;
        }
    }

    out.push(DivInlineSegment { text, style });
}

pub(super) fn render_inline_content(elements: &[HtmlElement], ctx: &RenderContext) -> Div {
    render_inline_segments(&collect_inline_segments(elements), ctx)
}

pub(super) fn render_inline_segments(segments: &[DivInlineSegment], ctx: &RenderContext) -> Div {
    let mut row = div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_baseline()
        .min_w(px(0.));

    for segment in segments {
        if segment.style.code || segment.style.link_href.is_some() {
            row = row.child(render_inline_segment_piece(
                &segment.text,
                &segment.style,
                ctx,
            ));
            continue;
        }

        for line_segment in segment.text.split_inclusive('\n') {
            let has_break = line_segment.ends_with('\n');
            let body = line_segment.strip_suffix('\n').unwrap_or(line_segment);

            for word in body.split_inclusive(char::is_whitespace) {
                if word.is_empty() {
                    continue;
                }
                row = row.child(render_inline_segment_piece(word, &segment.style, ctx));
            }

            if has_break {
                row = row.child(div().w_full().h(px(0.0)));
            }
        }
    }

    row
}

fn render_inline_segment_piece(text: &str, style: &DivInlineStyle, ctx: &RenderContext) -> Div {
    let mut piece = div().child(text.to_string());

    if style.code {
        piece = piece
            .px(px(4.0))
            .py(px(1.0))
            .bg(rgba((ctx.code_bg << 8) | 0x80))
            .rounded(px(3.0))
            .font_family(FONT_MONO)
            .text_xs()
            .text_color(rgb(ctx.accent_color));
    }

    if let Some(href) = style.link_href.as_ref() {
        piece = piece.text_color(rgb(ctx.accent_color)).cursor_pointer();
        if let Some(callback) = &ctx.on_link_click {
            let cb = callback.clone();
            let href_for_click = href.clone();
            piece = piece.on_mouse_down(
                gpui::MouseButton::Left,
                move |_event, _window, cx: &mut gpui::App| {
                    cb(&href_for_click, cx);
                },
            );
        }
    }

    if style.bold {
        piece = piece.font_weight(FontWeight::BOLD);
    }
    if style.italic {
        piece = piece.italic();
    }

    piece
}
