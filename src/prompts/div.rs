//! DivPrompt - HTML content display
//!
//! Features:
//! - Parse and render HTML elements as native GPUI components
//! - Support for headers, paragraphs, bold, italic, code, lists, blockquotes
//! - Theme-aware styling
//! - Simple keyboard: Enter or Escape to submit

use gpui::{
    div, prelude::*, px, rgb, rgba, Context, Div, FocusHandle, Focusable, FontWeight, Hsla, Render,
    ScrollHandle, Window,
};
use std::sync::Arc;

use crate::designs::{get_tokens, DesignVariant};
use crate::logging;
use crate::theme;
use crate::ui_foundation::{get_vibrancy_background, is_key_enter, is_key_escape};
use crate::utils::{parse_color, parse_html, HtmlElement, TailwindStyles};

use super::SubmitCallback;

/// Options for customizing the div container appearance
#[derive(Debug, Clone, Default)]
pub struct ContainerOptions {
    /// Background color: "transparent", "#RRGGBB", "#RRGGBBAA", or Tailwind color name
    pub background: Option<String>,
    /// Padding in pixels, or None to use default
    pub padding: Option<ContainerPadding>,
    /// Opacity (0-100), applies to the container background color
    pub opacity: Option<u8>,
    /// Tailwind classes for the content container
    pub container_classes: Option<String>,
}

/// Padding options for the container
#[derive(Debug, Clone)]
pub enum ContainerPadding {
    /// No padding
    None,
    /// Custom padding in pixels
    Pixels(f32),
}

impl ContainerOptions {
    /// Parse container background to GPUI color
    pub fn parse_background(&self) -> Option<Hsla> {
        let bg = self.background.as_ref()?;

        // Handle "transparent"
        if bg == "transparent" {
            return Some(Hsla::transparent_black());
        }

        // Handle hex colors: #RGB, #RRGGBB, #RRGGBBAA
        if bg.starts_with('#') {
            return parse_hex_color(bg);
        }

        // Handle Tailwind color names (e.g., "blue-500", "gray-900")
        if let Some(color) = parse_color(bg) {
            return Some(rgb_to_hsla(color, self.opacity));
        }

        None
    }

    /// Get padding value
    pub fn get_padding(&self, default: f32) -> f32 {
        match &self.padding {
            Some(ContainerPadding::None) => 0.0,
            Some(ContainerPadding::Pixels(px)) => *px,
            None => default,
        }
    }
}

/// Parse hex color string to GPUI Hsla
fn parse_hex_color(hex: &str) -> Option<Hsla> {
    let hex = hex.trim_start_matches('#');

    match hex.len() {
        // #RGB -> #RRGGBB
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Some(Hsla::from(gpui::Rgba {
                r: r as f32 / 255.0,
                g: g as f32 / 255.0,
                b: b as f32 / 255.0,
                a: 1.0,
            }))
        }
        // #RRGGBB
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Hsla::from(gpui::Rgba {
                r: r as f32 / 255.0,
                g: g as f32 / 255.0,
                b: b as f32 / 255.0,
                a: 1.0,
            }))
        }
        // #RRGGBBAA
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(Hsla::from(gpui::Rgba {
                r: r as f32 / 255.0,
                g: g as f32 / 255.0,
                b: b as f32 / 255.0,
                a: a as f32 / 255.0,
            }))
        }
        _ => None,
    }
}

#[inline]
fn is_div_submit_key(key: &str) -> bool {
    is_key_enter(key) || is_key_escape(key)
}

/// Convert RGB u32 to Hsla with optional opacity
fn rgb_to_hsla(color: u32, opacity: Option<u8>) -> Hsla {
    let r = ((color >> 16) & 0xFF) as f32 / 255.0;
    let g = ((color >> 8) & 0xFF) as f32 / 255.0;
    let b = (color & 0xFF) as f32 / 255.0;
    let a = opacity.map(|o| o as f32 / 100.0).unwrap_or(1.0);
    Hsla::from(gpui::Rgba { r, g, b, a })
}

#[inline]
fn default_container_padding(variant: DesignVariant) -> f32 {
    get_tokens(variant).spacing().padding_md
}

/// DivPrompt - HTML content display
///
/// Features:
/// - Parse and render HTML elements as native GPUI components
/// - Support for headers, paragraphs, bold, italic, code, lists, blockquotes
/// - Theme-aware styling
/// - Simple keyboard: Enter or Escape to submit
pub struct DivPrompt {
    pub id: String,
    pub html: String,
    pub tailwind: Option<String>,
    pub focus_handle: FocusHandle,
    pub on_submit: SubmitCallback,
    pub theme: Arc<theme::Theme>,
    /// Design variant for styling (defaults to Default for theme-based styling)
    pub design_variant: DesignVariant,
    /// Container customization options
    pub container_options: ContainerOptions,
    /// Scroll handle for tracking scroll position
    pub scroll_handle: ScrollHandle,
    /// Pre-extracted prompt colors for efficient rendering (Copy, 28 bytes)
    /// Avoids re-extracting colors from theme on every render
    prompt_colors: theme::PromptColors,
}

impl DivPrompt {
    pub fn new(
        id: String,
        html: String,
        tailwind: Option<String>,
        focus_handle: FocusHandle,
        on_submit: SubmitCallback,
        theme: Arc<theme::Theme>,
    ) -> Self {
        Self::with_options(
            id,
            html,
            tailwind,
            focus_handle,
            on_submit,
            theme,
            DesignVariant::Default,
            ContainerOptions::default(),
        )
    }

    pub fn with_design(
        id: String,
        html: String,
        tailwind: Option<String>,
        focus_handle: FocusHandle,
        on_submit: SubmitCallback,
        theme: Arc<theme::Theme>,
        design_variant: DesignVariant,
    ) -> Self {
        Self::with_options(
            id,
            html,
            tailwind,
            focus_handle,
            on_submit,
            theme,
            design_variant,
            ContainerOptions::default(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_options(
        id: String,
        html: String,
        tailwind: Option<String>,
        focus_handle: FocusHandle,
        on_submit: SubmitCallback,
        theme: Arc<theme::Theme>,
        design_variant: DesignVariant,
        container_options: ContainerOptions,
    ) -> Self {
        // Extract colors ONCE during construction to avoid re-extraction on every render
        // PromptColors is Copy (28 bytes) - much cheaper than extracting on every frame
        let prompt_colors = theme.colors.prompt_colors();

        logging::log(
            "PROMPTS",
            &format!(
                "DivPrompt::new with theme colors: bg={:#x}, text={:#x}, design: {:?}, container_opts: {:?}",
                theme.colors.background.main, theme.colors.text.primary, design_variant, container_options
            ),
        );
        DivPrompt {
            id,
            html,
            tailwind,
            focus_handle,
            on_submit,
            theme,
            design_variant,
            container_options,
            scroll_handle: ScrollHandle::new(),
            prompt_colors,
        }
    }

    /// Submit - always with None value (just acknowledgment)
    fn submit(&mut self) {
        (self.on_submit)(self.id.clone(), None);
    }

    /// Submit with a specific value (for submit:value links)
    fn submit_with_value(&mut self, value: String) {
        logging::log("DIV", &format!("Submit with value: {}", value));
        (self.on_submit)(self.id.clone(), Some(value));
    }

    /// Handle link click based on href pattern
    pub fn handle_link_click(&mut self, href: &str) {
        logging::log("DIV", &format!("Link clicked: {}", href));

        if let Some(value) = href.strip_prefix("submit:") {
            self.submit_with_value(value.to_string());
        } else if href.starts_with("http://") || href.starts_with("https://") {
            if let Err(e) = open::that(href) {
                logging::log("DIV", &format!("Failed to open URL {}: {}", href, e));
            }
        } else if href.starts_with("file://") {
            if let Err(e) = open::that(href) {
                logging::log("DIV", &format!("Failed to open file {}: {}", href, e));
            }
        } else {
            logging::log("DIV", &format!("Unknown link protocol: {}", href));
        }
    }
}

/// Callback type for link clicks - needs App context to update entity
type LinkClickCallback = Arc<dyn Fn(&str, &mut gpui::App) + Send + Sync>;

/// Style context for rendering HTML elements
#[derive(Clone)]
struct RenderContext {
    /// Primary text color
    text_primary: u32,
    /// Secondary text color (for muted content)
    text_secondary: u32,
    /// Tertiary text color
    text_tertiary: u32,
    /// Accent/link color
    accent_color: u32,
    /// Code background color
    code_bg: u32,
    /// Blockquote border color
    quote_border: u32,
    /// HR color
    hr_color: u32,
    /// Optional link click callback
    on_link_click: Option<LinkClickCallback>,
}

impl RenderContext {
    fn from_theme(colors: &theme::ColorScheme) -> Self {
        Self {
            text_primary: colors.text.primary,
            text_secondary: colors.text.secondary,
            text_tertiary: colors.text.tertiary,
            accent_color: colors.accent.selected,
            code_bg: colors.background.search_box,
            quote_border: colors.ui.border,
            hr_color: colors.ui.border,
            on_link_click: None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct DivInlineStyle {
    bold: bool,
    italic: bool,
    code: bool,
    link_href: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DivInlineSegment {
    text: String,
    style: DivInlineStyle,
}

fn collect_inline_segments(elements: &[HtmlElement]) -> Vec<DivInlineSegment> {
    let mut segments = Vec::new();
    append_inline_segments(elements, &DivInlineStyle::default(), &mut segments);
    segments
}

fn append_inline_segments(
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

fn render_inline_content(elements: &[HtmlElement], ctx: &RenderContext) -> Div {
    render_inline_segments(&collect_inline_segments(elements), ctx)
}

fn render_inline_segments(segments: &[DivInlineSegment], ctx: &RenderContext) -> Div {
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
            .font_family("Menlo")
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

/// Render a vector of HtmlElements as a GPUI Div
fn render_elements(elements: &[HtmlElement], ctx: RenderContext) -> Div {
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
            let mut style = DivInlineStyle::default();
            style.bold = true;
            let mut segments = Vec::new();
            append_inline_segments(children, &style, &mut segments);
            div()
                .w_full()
                .child(render_inline_segments(&segments, &ctx))
        }

        HtmlElement::Italic(children) => {
            let mut style = DivInlineStyle::default();
            style.italic = true;
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
            let mut style = DivInlineStyle::default();
            style.link_href = Some(href.clone());
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

/// Apply Tailwind styles to a div based on a class string
fn apply_tailwind_styles(mut element: Div, class_string: &str) -> Div {
    let styles = TailwindStyles::parse(class_string);

    // Layout
    if styles.flex {
        element = element.flex();
    }
    if styles.flex_col {
        element = element.flex_col();
    }
    if styles.flex_row {
        element = element.flex_row();
    }
    if styles.flex_1 {
        element = element.flex_1();
    }
    if styles.items_center {
        element = element.items_center();
    }
    if styles.items_start {
        element = element.items_start();
    }
    if styles.items_end {
        element = element.items_end();
    }
    if styles.justify_center {
        element = element.justify_center();
    }
    if styles.justify_between {
        element = element.justify_between();
    }
    if styles.justify_start {
        element = element.justify_start();
    }
    if styles.justify_end {
        element = element.justify_end();
    }

    // Sizing
    if styles.w_full {
        element = element.w_full();
    }
    if styles.h_full {
        element = element.h_full();
    }
    if styles.min_w_0 {
        element = element.min_w(px(0.));
    }
    if styles.min_h_0 {
        element = element.min_h(px(0.));
    }

    // Spacing - padding
    if let Some(p) = styles.padding {
        element = element.p(px(p));
    }
    if let Some(px_val) = styles.padding_x {
        element = element.px(px(px_val));
    }
    if let Some(py_val) = styles.padding_y {
        element = element.py(px(py_val));
    }
    if let Some(pt) = styles.padding_top {
        element = element.pt(px(pt));
    }
    if let Some(pb) = styles.padding_bottom {
        element = element.pb(px(pb));
    }
    if let Some(pl) = styles.padding_left {
        element = element.pl(px(pl));
    }
    if let Some(pr) = styles.padding_right {
        element = element.pr(px(pr));
    }

    // Spacing - margin
    if let Some(m) = styles.margin {
        element = element.m(px(m));
    }
    if let Some(mx_val) = styles.margin_x {
        element = element.mx(px(mx_val));
    }
    if let Some(my_val) = styles.margin_y {
        element = element.my(px(my_val));
    }
    if let Some(mt) = styles.margin_top {
        element = element.mt(px(mt));
    }
    if let Some(mb) = styles.margin_bottom {
        element = element.mb(px(mb));
    }
    if let Some(ml) = styles.margin_left {
        element = element.ml(px(ml));
    }
    if let Some(mr) = styles.margin_right {
        element = element.mr(px(mr));
    }

    // Gap
    if let Some(gap_val) = styles.gap {
        element = element.gap(px(gap_val));
    }

    // Colors
    if let Some(color) = styles.bg_color {
        element = element.bg(rgb(color));
    }
    if let Some(color) = styles.text_color {
        element = element.text_color(rgb(color));
    }
    if let Some(color) = styles.border_color {
        element = element.border_color(rgb(color));
    }

    // Typography
    // User-specified pixel size - not converted to rem
    if let Some(size) = styles.font_size {
        element = element.text_size(px(size));
    }
    if styles.font_bold {
        element = element.font_weight(FontWeight::BOLD);
    }
    if styles.font_medium {
        element = element.font_weight(FontWeight::MEDIUM);
    }
    if styles.font_normal {
        element = element.font_weight(FontWeight::NORMAL);
    }

    // Border radius
    if let Some(r) = styles.rounded {
        element = element.rounded(px(r));
    }

    // Border
    if styles.border {
        element = element.border_1();
    }
    if let Some(width) = styles.border_width {
        if width == 0.0 {
            // No border
        } else if width == 2.0 {
            element = element.border_2();
        } else if width == 4.0 {
            element = element.border_4();
        } else if width == 8.0 {
            element = element.border_8();
        }
    }

    element
}

impl Focusable for DivPrompt {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for DivPrompt {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Get design tokens for the current design variant
        let tokens = get_tokens(self.design_variant);
        let colors = tokens.colors();

        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  _window: &mut Window,
                  cx: &mut Context<Self>| {
                let modifiers = &event.keystroke.modifiers;
                if modifiers.platform || modifiers.control || modifiers.alt {
                    return;
                }

                if is_div_submit_key(event.keystroke.key.as_str()) {
                    this.submit();
                    cx.stop_propagation();
                }
            },
        );

        // Parse HTML into elements
        let elements = parse_html(&self.html);

        // Create link click callback using a weak entity handle
        // This allows us to call back into the DivPrompt to handle submit:value links
        let weak_handle = cx.entity().downgrade();
        let on_link_click: LinkClickCallback = Arc::new(move |href: &str, cx: &mut gpui::App| {
            let href_owned = href.to_string();
            if let Some(entity) = weak_handle.upgrade() {
                entity.update(cx, move |this, _cx| {
                    this.handle_link_click(&href_owned);
                });
            }
        });

        // Create render context using pre-extracted colors (avoids extraction on every render)
        let render_ctx = if self.design_variant == DesignVariant::Default {
            // Use pre-extracted prompt_colors instead of extracting from theme
            RenderContext {
                text_primary: self.prompt_colors.text_primary,
                text_secondary: self.prompt_colors.text_secondary,
                text_tertiary: self.prompt_colors.text_tertiary,
                accent_color: self.prompt_colors.accent_color,
                code_bg: self.prompt_colors.code_bg,
                quote_border: self.prompt_colors.quote_border,
                hr_color: self.prompt_colors.hr_color,
                on_link_click: Some(on_link_click),
            }
        } else {
            RenderContext {
                text_primary: colors.text_primary,
                text_secondary: colors.text_secondary,
                text_tertiary: colors.text_muted, // Use text_muted for tertiary
                accent_color: colors.accent,
                code_bg: colors.background_tertiary, // Use background_tertiary for code bg
                quote_border: colors.border,
                hr_color: colors.border,
                on_link_click: Some(on_link_click),
            }
        };

        // Determine container background:
        // 1. If container_options.background is set, use that
        // 2. If container_options.opacity is set, apply that to base color
        // 3. Otherwise use vibrancy foundation (None when vibrancy enabled)
        let container_bg: Option<Hsla> =
            if let Some(custom_bg) = self.container_options.parse_background() {
                // Custom background specified - always use it
                Some(custom_bg)
            } else if let Some(opacity) = self.container_options.opacity {
                // Opacity specified - apply to theme/design color
                let base_color = if self.design_variant == DesignVariant::Default {
                    self.theme.colors.background.main
                } else {
                    colors.background
                };
                Some(rgb_to_hsla(base_color, Some(opacity)))
            } else {
                // No custom background or opacity - use vibrancy foundation
                // Returns None when vibrancy enabled (let Root handle bg)
                get_vibrancy_background(&self.theme).map(Hsla::from)
            };

        // Determine container padding from design tokens for consistent prompt spacing.
        let container_padding = self
            .container_options
            .get_padding(default_container_padding(self.design_variant));

        // Generate semantic IDs for div prompt elements
        let panel_semantic_id = format!("panel:content-{}", self.id);

        // Render the HTML elements with any inline Tailwind classes
        let content = render_elements(&elements, render_ctx);

        // Apply root tailwind classes if provided (legacy support)
        let styled_content = if let Some(tw) = &self.tailwind {
            apply_tailwind_styles(content, tw)
        } else {
            content
        };

        // Build the content container with optional containerClasses
        // Apply containerClasses first (before .id() which makes it Stateful for overflow_y_scroll)
        let content_base = div()
            .flex_1() // Grow to fill available space to bottom
            .min_h(px(0.)) // Allow shrinking
            .w_full()
            .child(styled_content);

        let content_styled = if let Some(ref classes) = self.container_options.container_classes {
            apply_tailwind_styles(content_base, classes)
        } else {
            content_base
        };

        // Add ID to make it Stateful, then enable vertical scrolling with tracked scroll handle
        // overflow_y_scroll requires StatefulInteractiveElement trait (needs .id() first)
        let content_container = content_styled
            .id(gpui::ElementId::Name(panel_semantic_id.into()))
            .overflow_y_scroll()
            .track_scroll(&self.scroll_handle);

        // Main container - fills entire window height with no bottom gap
        // Use relative positioning to overlay scrollbar
        div()
            .id(gpui::ElementId::Name("window:div".into()))
            .relative()
            .flex()
            .flex_col()
            .w_full()
            .h_full() // Fill container height completely
            .min_h(px(0.)) // Allow proper flex behavior
            .when_some(container_bg, |d, bg| d.bg(bg)) // Only apply bg when available
            .p(px(container_padding))
            .key_context("div_prompt")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .child(content_container)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_context_from_theme() {
        let colors = theme::ColorScheme::dark_default();
        let ctx = RenderContext::from_theme(&colors);

        assert_eq!(ctx.text_primary, colors.text.primary);
        assert_eq!(ctx.text_secondary, colors.text.secondary);
        assert_eq!(ctx.accent_color, colors.accent.selected);
    }

    #[test]
    fn test_render_simple_text() {
        let elements = parse_html("Hello World");
        let ctx = RenderContext::from_theme(&theme::ColorScheme::dark_default());

        // Should not panic
        let _ = render_elements(&elements, ctx);
    }

    #[test]
    fn test_render_complex_html() {
        let html = r#"
            <h1>Title</h1>
            <p>A paragraph with <strong>bold</strong> and <em>italic</em> text.</p>
            <ul>
                <li>Item 1</li>
                <li>Item 2</li>
            </ul>
            <blockquote>A quote</blockquote>
            <pre><code>let x = 1;</code></pre>
            <hr>
            <a href="https://example.com">Link</a>
        "#;
        let elements = parse_html(html);
        let ctx = RenderContext::from_theme(&theme::ColorScheme::dark_default());

        // Should not panic
        let _ = render_elements(&elements, ctx);
    }

    #[test]
    fn test_render_headers_different_sizes() {
        for level in 1..=6 {
            let html = format!("<h{}>Header {}</h{}>", level, level, level);
            let elements = parse_html(&html);
            let ctx = RenderContext::from_theme(&theme::ColorScheme::dark_default());

            // Should not panic
            let _ = render_elements(&elements, ctx);
        }
    }

    #[test]
    fn test_render_nested_formatting() {
        let html = "<p><strong><em>Bold and italic</em></strong></p>";
        let elements = parse_html(html);
        let ctx = RenderContext::from_theme(&theme::ColorScheme::dark_default());

        // Should not panic
        let _ = render_elements(&elements, ctx);
    }

    #[test]
    fn test_default_container_padding_follows_design_spacing() {
        for variant in [
            DesignVariant::Default,
            DesignVariant::Minimal,
            DesignVariant::Compact,
        ] {
            let expected = get_tokens(variant).spacing().padding_md;
            assert_eq!(default_container_padding(variant), expected);
        }
    }

    #[test]
    fn test_collect_inline_segments_preserves_nested_inline_styles_when_html_contains_formatting() {
        let elements = parse_html(
            "<p>Hello <strong>Bold</strong> <em>Italic</em> <code>const x = 1;</code></p>",
        );
        let segments = collect_inline_segments(&elements);

        assert!(segments
            .iter()
            .any(|segment| segment.text == "Bold" && segment.style.bold));
        assert!(segments
            .iter()
            .any(|segment| segment.text == "Italic" && segment.style.italic));
        assert!(segments
            .iter()
            .any(|segment| segment.text == "const x = 1;" && segment.style.code));
    }

    #[test]
    fn test_collect_inline_segments_preserves_link_target_when_html_contains_nested_link_text() {
        let elements =
            parse_html("<p>Open <a href=\"submit:continue\"><strong>Continue</strong></a></p>");
        let segments = collect_inline_segments(&elements);

        assert!(segments.iter().any(|segment| {
            segment.text == "Continue"
                && segment.style.bold
                && segment.style.link_href.as_deref() == Some("submit:continue")
        }));
    }

    #[test]
    fn test_is_div_submit_key_handles_enter_return_escape_and_esc() {
        assert!(is_div_submit_key("enter"));
        assert!(is_div_submit_key("return"));
        assert!(is_div_submit_key("escape"));
        assert!(is_div_submit_key("esc"));
        assert!(!is_div_submit_key("tab"));
    }
}
