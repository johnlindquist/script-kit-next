#![allow(dead_code)]

use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, Div, FontWeight, Rgba, SharedString};

use crate::theme::{self, AppChromeColors};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct InfoTextMetric {
    pub size: f32,
    pub line: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct InfoTypeScale {
    pub micro: InfoTextMetric,
    pub caption: InfoTextMetric,
    pub body: InfoTextMetric,
    pub subhead: InfoTextMetric,
    pub title: InfoTextMetric,
    pub hero: InfoTextMetric,
    pub brand: InfoTextMetric,
}

pub(crate) const INFO_TYPE_SCALE: InfoTypeScale = InfoTypeScale {
    micro: InfoTextMetric {
        size: 11.0,
        line: 14.0,
    },
    caption: InfoTextMetric {
        size: 12.0,
        line: 16.0,
    },
    body: InfoTextMetric {
        size: 13.0,
        line: 18.0,
    },
    subhead: InfoTextMetric {
        size: 14.0,
        line: 20.0,
    },
    title: InfoTextMetric {
        size: 16.0,
        line: 22.0,
    },
    hero: InfoTextMetric {
        size: 20.0,
        line: 26.0,
    },
    brand: InfoTextMetric {
        size: 22.0,
        line: 28.0,
    },
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct InfoSpacing {
    pub xxs: f32,
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
}

pub(crate) const INFO_SPACING: InfoSpacing = InfoSpacing {
    xxs: 4.0,
    xs: 8.0,
    sm: 12.0,
    md: 16.0,
    lg: 20.0,
    xl: 24.0,
    xxl: 32.0,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InfoStateDensity {
    Compact,
    Comfortable,
    Hero,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InfoStateLayout {
    Centered,
    AnchoredTop,
    ComposerEmpty,
    InlinePanel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InfoStateTone {
    Neutral,
    Help,
    Setup,
    Permission,
    Recovery,
    About,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct InfoMetrics {
    pub max_width: f32,
    pub icon_size: f32,
    pub row_min_h: f32,
    pub radius: f32,
    pub block_gap: f32,
    pub item_gap: f32,
    pub pad_x: f32,
    pub pad_y: f32,
}

pub(crate) fn info_metrics(density: InfoStateDensity) -> InfoMetrics {
    match density {
        InfoStateDensity::Compact => InfoMetrics {
            max_width: 380.0,
            icon_size: 28.0,
            row_min_h: 28.0,
            radius: 8.0,
            block_gap: 12.0,
            item_gap: 6.0,
            pad_x: 12.0,
            pad_y: 10.0,
        },
        InfoStateDensity::Comfortable => InfoMetrics {
            max_width: 500.0,
            icon_size: 36.0,
            row_min_h: 34.0,
            radius: 9.0,
            block_gap: 16.0,
            item_gap: 8.0,
            pad_x: 16.0,
            pad_y: 14.0,
        },
        InfoStateDensity::Hero => InfoMetrics {
            max_width: 560.0,
            icon_size: 44.0,
            row_min_h: 36.0,
            radius: 10.0,
            block_gap: 20.0,
            item_gap: 10.0,
            pad_x: 20.0,
            pad_y: 16.0,
        },
    }
}

#[derive(Clone, Copy)]
pub(crate) struct InfoPalette {
    pub title: Rgba,
    pub body: Rgba,
    pub hint: Rgba,
    pub strong: Rgba,
    pub placeholder: Rgba,
    pub icon: Rgba,
    pub accent: Rgba,
    pub hover: Rgba,
    pub selected: Rgba,
    pub border: Rgba,
    pub whisper: Rgba,
    pub panel: Rgba,
}

pub(crate) fn info_palette(theme: &theme::Theme) -> InfoPalette {
    let chrome = AppChromeColors::from_theme(theme);
    InfoPalette {
        title: rgb(chrome.text_primary_hex),
        body: rgba(chrome.text_muted_rgba),
        hint: rgba(chrome.text_hint_rgba),
        strong: rgba(chrome.text_strong_rgba),
        placeholder: rgba(chrome.placeholder_text_rgba),
        icon: rgba(chrome.text_icon_rgba),
        accent: rgb(chrome.accent_hex),
        hover: rgba(chrome.hover_rgba),
        selected: rgba(chrome.selection_rgba),
        border: rgba(chrome.whisper_border_rgba),
        whisper: rgba(chrome.whisper_surface_rgba),
        panel: rgba(chrome.panel_surface_rgba),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct InfoGuidanceItem {
    pub shortcut: Option<&'static str>,
    pub label: SharedString,
    pub detail: Option<SharedString>,
}

impl InfoGuidanceItem {
    pub(crate) fn new(shortcut: Option<&'static str>, label: impl Into<SharedString>) -> Self {
        Self {
            shortcut,
            label: label.into(),
            detail: None,
        }
    }

    pub(crate) fn detail(mut self, detail: impl Into<SharedString>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct InfoSection {
    pub title: Option<SharedString>,
    pub items: Vec<InfoGuidanceItem>,
}

impl InfoSection {
    pub(crate) fn new(items: Vec<InfoGuidanceItem>) -> Self {
        Self { title: None, items }
    }

    pub(crate) fn titled(title: impl Into<SharedString>, items: Vec<InfoGuidanceItem>) -> Self {
        Self {
            title: Some(title.into()),
            items,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct InfoStateSpec {
    pub id: &'static str,
    pub layout: InfoStateLayout,
    pub density: InfoStateDensity,
    pub tone: InfoStateTone,
    pub eyebrow: Option<SharedString>,
    pub title: Option<SharedString>,
    pub body: Option<SharedString>,
    pub sections: Vec<InfoSection>,
    pub footer_note: Option<SharedString>,
}

impl InfoStateSpec {
    pub(crate) fn new(id: &'static str) -> Self {
        Self {
            id,
            layout: InfoStateLayout::Centered,
            density: InfoStateDensity::Compact,
            tone: InfoStateTone::Neutral,
            eyebrow: None,
            title: None,
            body: None,
            sections: Vec::new(),
            footer_note: None,
        }
    }

    pub(crate) fn layout(mut self, layout: InfoStateLayout) -> Self {
        self.layout = layout;
        self
    }

    pub(crate) fn density(mut self, density: InfoStateDensity) -> Self {
        self.density = density;
        self
    }

    pub(crate) fn tone(mut self, tone: InfoStateTone) -> Self {
        self.tone = tone;
        self
    }

    pub(crate) fn eyebrow(mut self, eyebrow: impl Into<SharedString>) -> Self {
        self.eyebrow = Some(eyebrow.into());
        self
    }

    pub(crate) fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub(crate) fn body(mut self, body: impl Into<SharedString>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub(crate) fn section(mut self, section: InfoSection) -> Self {
        self.sections.push(section);
        self
    }

    pub(crate) fn footer_note(mut self, note: impl Into<SharedString>) -> Self {
        self.footer_note = Some(note.into());
        self
    }
}

pub(crate) fn acp_empty_guidance_spec() -> InfoStateSpec {
    InfoStateSpec::new("acp-empty-composer-guidance")
        .layout(InfoStateLayout::ComposerEmpty)
        .density(InfoStateDensity::Compact)
        .tone(InfoStateTone::Help)
        .title("Ask with context")
        .body("Describe the result you want. Use / for skills or @ to attach context before you send.")
        .section(InfoSection::new(vec![
            InfoGuidanceItem::new(Some("/"), "Use a skill or agent command"),
            InfoGuidanceItem::new(Some("@"), "Attach files, scripts, clipboard, or history"),
            InfoGuidanceItem::new(Some("⇧↵"), "Add a newline"),
            InfoGuidanceItem::new(Some("⌘P"), "Open previous chats"),
        ]))
        .footer_note("⌘K shows every chat action.")
}

pub(crate) fn render_acp_empty_guidance(theme: &theme::Theme) -> AnyElement {
    render_info_state(acp_empty_guidance_spec(), theme)
}

pub(crate) fn render_info_state(spec: InfoStateSpec, theme: &theme::Theme) -> AnyElement {
    let palette = info_palette(theme);
    let metrics = info_metrics(spec.density);
    let content = render_info_content(&spec, palette, metrics);

    match spec.layout {
        InfoStateLayout::Centered | InfoStateLayout::ComposerEmpty => div()
            .id(spec.id)
            .w_full()
            .h_full()
            .min_h(px(0.0))
            .flex()
            .items_center()
            .justify_center()
            .px(px(INFO_SPACING.xl))
            .child(content)
            .into_any_element(),
        InfoStateLayout::AnchoredTop => div()
            .id(spec.id)
            .w_full()
            .h_full()
            .min_h(px(0.0))
            .flex()
            .items_start()
            .justify_center()
            .px(px(INFO_SPACING.xl))
            .py(px(INFO_SPACING.xl))
            .child(content)
            .into_any_element(),
        InfoStateLayout::InlinePanel => div()
            .id(spec.id)
            .w_full()
            .child(
                content
                    .rounded(px(metrics.radius))
                    .border_1()
                    .border_color(palette.border)
                    .bg(palette.whisper)
                    .px(px(metrics.pad_x))
                    .py(px(metrics.pad_y)),
            )
            .into_any_element(),
    }
}

fn render_info_content(spec: &InfoStateSpec, palette: InfoPalette, metrics: InfoMetrics) -> Div {
    let title_metric = match spec.density {
        InfoStateDensity::Compact => INFO_TYPE_SCALE.subhead,
        InfoStateDensity::Comfortable => INFO_TYPE_SCALE.title,
        InfoStateDensity::Hero => INFO_TYPE_SCALE.hero,
    };
    let body_metric = INFO_TYPE_SCALE.body;

    let mut stack = div()
        .w_full()
        .max_w(px(metrics.max_width))
        .flex()
        .flex_col()
        .gap(px(metrics.block_gap));

    if let Some(eyebrow) = spec.eyebrow.clone() {
        stack = stack.child(
            div()
                .text_size(px(INFO_TYPE_SCALE.micro.size))
                .line_height(px(INFO_TYPE_SCALE.micro.line))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.strong)
                .child(eyebrow),
        );
    }

    if spec.title.is_some() || spec.body.is_some() {
        let mut intro = div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(INFO_SPACING.xs * 0.5));
        if let Some(title) = spec.title.clone() {
            intro = intro.child(
                div()
                    .text_size(px(title_metric.size))
                    .line_height(px(title_metric.line))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(palette.title)
                    .child(title),
            );
        }
        if let Some(body) = spec.body.clone() {
            intro = intro.child(
                div()
                    .text_size(px(body_metric.size))
                    .line_height(px(body_metric.line))
                    .text_color(palette.body)
                    .child(body),
            );
        }
        stack = stack.child(intro);
    }

    for (index, section) in spec.sections.iter().enumerate() {
        stack = stack.child(render_info_section(
            section,
            format!("{}-section-{index}", spec.id),
            spec.density,
            palette,
            metrics,
        ));
    }

    if let Some(note) = spec.footer_note.clone() {
        stack = stack.child(
            div()
                .text_size(px(INFO_TYPE_SCALE.caption.size))
                .line_height(px(INFO_TYPE_SCALE.caption.line))
                .text_color(palette.hint)
                .child(note),
        );
    }

    stack
}

fn render_info_section(
    section: &InfoSection,
    id: String,
    density: InfoStateDensity,
    palette: InfoPalette,
    metrics: InfoMetrics,
) -> AnyElement {
    let mut stack = div()
        .id(id)
        .w_full()
        .flex()
        .flex_col()
        .gap(px(metrics.item_gap));

    if let Some(title) = section.title.clone() {
        stack = stack.child(
            div()
                .text_size(px(INFO_TYPE_SCALE.micro.size))
                .line_height(px(INFO_TYPE_SCALE.micro.line))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.strong)
                .child(title),
        );
    }

    stack
        .child(render_info_guidance_items(
            "info-guidance-items",
            &section.items,
            density,
            palette,
        ))
        .into_any_element()
}

pub(crate) fn render_info_guidance_items(
    id: &'static str,
    items: &[InfoGuidanceItem],
    density: InfoStateDensity,
    palette: InfoPalette,
) -> AnyElement {
    let metrics = info_metrics(density);
    div()
        .id(id)
        .w_full()
        .flex()
        .flex_col()
        .gap(px(INFO_SPACING.xs * 0.5))
        .children(
            items
                .iter()
                .map(|item| render_guidance_row(item, metrics, palette).into_any_element()),
        )
        .into_any_element()
}

fn render_guidance_row(item: &InfoGuidanceItem, metrics: InfoMetrics, palette: InfoPalette) -> Div {
    let mut row = div()
        .w_full()
        .min_h(px(metrics.row_min_h))
        .flex()
        .items_center()
        .gap(px(INFO_SPACING.sm));

    if let Some(shortcut) = item.shortcut {
        let tokens = crate::components::hint_strip::shortcut_tokens_from_hint(shortcut);
        row = row.child(div().w(px(42.0)).flex().items_center().child(
            crate::components::hint_strip::render_inline_shortcut_keys(
                tokens.iter().map(String::as_str),
                crate::components::hint_strip::whisper_inline_shortcut_colors(
                    palette.strong.into(),
                    palette.title.into(),
                    true,
                ),
            ),
        ));
    }

    let mut text = div()
        .flex_1()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .gap(px(2.0))
        .child(
            div()
                .text_size(px(INFO_TYPE_SCALE.caption.size))
                .line_height(px(INFO_TYPE_SCALE.caption.line))
                .text_color(palette.body)
                .child(item.label.clone()),
        );

    if let Some(detail) = item.detail.clone() {
        text = text.child(
            div()
                .text_size(px(INFO_TYPE_SCALE.micro.size))
                .line_height(px(INFO_TYPE_SCALE.micro.line))
                .text_color(palette.hint)
                .child(detail),
        );
    }

    row.child(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn info_type_scale_matches_compact_palette_math() {
        assert_eq!(
            INFO_TYPE_SCALE.micro,
            InfoTextMetric {
                size: 11.0,
                line: 14.0
            }
        );
        assert_eq!(
            INFO_TYPE_SCALE.caption,
            InfoTextMetric {
                size: 12.0,
                line: 16.0
            }
        );
        assert_eq!(
            INFO_TYPE_SCALE.body,
            InfoTextMetric {
                size: 13.0,
                line: 18.0
            }
        );
        assert_eq!(
            INFO_TYPE_SCALE.title,
            InfoTextMetric {
                size: 16.0,
                line: 22.0
            }
        );
        assert!(INFO_TYPE_SCALE.brand.size <= 22.0);
    }

    #[test]
    fn info_metrics_use_four_pixel_rhythm_where_visible() {
        for metrics in [
            info_metrics(InfoStateDensity::Compact),
            info_metrics(InfoStateDensity::Comfortable),
            info_metrics(InfoStateDensity::Hero),
        ] {
            for value in [
                metrics.icon_size,
                metrics.row_min_h,
                metrics.block_gap,
                metrics.pad_x,
                metrics.pad_y,
            ] {
                assert_eq!(value.rem_euclid(2.0), 0.0);
            }
        }
    }

    #[test]
    fn acp_empty_guidance_teaches_starting_context_not_window_management() {
        let spec = acp_empty_guidance_spec();
        let copy = format!("{spec:?}");
        assert!(copy.contains("Ask with context"));
        assert!(copy.contains("Use a skill or agent command"));
        assert!(copy.contains("Attach files, scripts, clipboard, or history"));
        assert!(copy.contains("Add a newline"));
        assert!(copy.contains("Open previous chats"));
        assert!(!copy.contains("Type / for skills"));
        assert!(!copy.contains(&format!("{} new", "⌘N")));
        assert!(!copy.contains(&format!("{} close", "⌘W")));
    }
}
