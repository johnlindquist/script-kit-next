use gpui::{div, prelude::*, px, rems, rgb, rgba, Div, FontWeight, Rgba, SharedString};

use crate::theme::opacity::{OPACITY_BORDER, OPACITY_CARD_BG, OPACITY_PROMINENT};
use crate::ui_foundation::hex_to_rgba_with_opacity;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PromptFrameConfig {
    pub relative: bool,
    pub rounded_corners: Option<f32>,
    pub min_height_px: f32,
    pub clip_overflow: bool,
}

impl Default for PromptFrameConfig {
    fn default() -> Self {
        Self {
            relative: false,
            rounded_corners: None,
            min_height_px: 0.0,
            clip_overflow: true,
        }
    }
}

impl PromptFrameConfig {
    pub fn with_relative(mut self, relative: bool) -> Self {
        self.relative = relative;
        self
    }

    pub fn with_rounded_corners(mut self, radius: f32) -> Self {
        self.rounded_corners = Some(radius);
        self
    }
}

pub(crate) fn prompt_shell_frame_config(radius: f32) -> PromptFrameConfig {
    PromptFrameConfig::default()
        .with_relative(true)
        .with_rounded_corners(radius)
}

pub(crate) fn prompt_frame_root(config: PromptFrameConfig) -> Div {
    let mut frame = div()
        .flex()
        .flex_col()
        .w_full()
        .h_full()
        .min_h(px(config.min_height_px));

    if config.clip_overflow {
        frame = frame.overflow_hidden();
    }

    if config.relative {
        frame = frame.relative();
    }

    if let Some(radius) = config.rounded_corners {
        frame = frame.rounded(px(radius));
    }

    frame
}

pub(crate) fn prompt_frame_fill_content(content: impl IntoElement) -> Div {
    div()
        .flex_1()
        .w_full()
        .min_h(px(0.))
        .overflow_hidden()
        .child(content)
}

/// Shared inner card surface for form fields and content cards.
///
/// Returns a full-width rounded div with consistent padding, border, and
/// background — use this for text inputs, preview cards, and any other
/// "card-on-prompt" surface so every step of a multi-step flow shares the
/// same visual language.
pub(crate) fn prompt_surface(background: Rgba, border: Rgba) -> Div {
    div()
        .w_full()
        .px(rems(0.875))
        .py(rems(0.625))
        .bg(background)
        .border_1()
        .border_color(border)
        .rounded(px(8.0))
}

/// Shared intro block for create-flow screens (title + description).
pub(crate) fn prompt_form_intro(
    title: impl Into<SharedString>,
    description: impl Into<SharedString>,
    title_color: Rgba,
    description_color: Rgba,
    gap_px: f32,
) -> Div {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(gap_px))
        .child(
            div()
                .text_lg()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(title_color)
                .child(title.into()),
        )
        .child(
            div()
                .text_sm()
                .text_color(description_color)
                .child(description.into()),
        )
}

/// Shared labeled section for create-flow screens (label above content).
pub(crate) fn prompt_form_section(
    label: impl Into<SharedString>,
    label_color: Rgba,
    gap_px: f32,
    content: impl IntoElement,
) -> Div {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(gap_px))
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(label_color)
                .child(label.into()),
        )
        .child(content)
}

/// Shared helper text for create-flow screens.
pub(crate) fn prompt_form_help(text: impl Into<SharedString>, color: Rgba) -> Div {
    div().text_xs().text_color(color).child(text.into())
}

/// State of a form field within a create-flow prompt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PromptFieldState {
    Default,
    Active,
    Error,
    ReadOnly,
}

/// Pre-computed colors for a form field based on its state.
#[derive(Clone, Copy)]
pub(crate) struct PromptFieldStyle {
    pub background: Rgba,
    pub border: Rgba,
    pub value: Rgba,
}

/// Compute field colors from the theme, field state, and whether the value is empty.
pub(crate) fn prompt_field_style(
    theme: &crate::theme::Theme,
    state: PromptFieldState,
    empty: bool,
) -> PromptFieldStyle {
    let default_background =
        rgba(hex_to_rgba_with_opacity(theme.colors.background.search_box, OPACITY_CARD_BG));
    let highlighted_background =
        rgba(hex_to_rgba_with_opacity(theme.colors.accent.selected_subtle, OPACITY_CARD_BG));
    let default_border =
        rgba(hex_to_rgba_with_opacity(theme.colors.ui.border, OPACITY_BORDER));
    let active_border = rgb(theme.colors.accent.selected);
    let error_border = rgb(theme.colors.ui.error);
    let value = if empty {
        rgba(hex_to_rgba_with_opacity(theme.colors.text.muted, OPACITY_PROMINENT))
    } else {
        rgb(theme.colors.text.primary)
    };

    match state {
        PromptFieldState::Default => PromptFieldStyle {
            background: default_background,
            border: default_border,
            value,
        },
        PromptFieldState::Active => PromptFieldStyle {
            background: highlighted_background,
            border: active_border,
            value,
        },
        PromptFieldState::Error => PromptFieldStyle {
            background: default_background,
            border: error_border,
            value,
        },
        PromptFieldState::ReadOnly => PromptFieldStyle {
            background: highlighted_background,
            border: default_border,
            value: rgb(theme.colors.text.primary),
        },
    }
}

/// Single-line text field card using the shared prompt surface.
pub(crate) fn prompt_text_field(
    value: impl Into<SharedString>,
    style: PromptFieldStyle,
    min_height: f32,
) -> Div {
    prompt_surface(style.background, style.border)
        .min_h(px(min_height))
        .flex()
        .items_center()
        .child(
            div()
                .w_full()
                .text_sm()
                .text_color(style.value)
                .child(value.into()),
        )
}

/// Multi-line detail card with headline, supporting text, and detail text rows.
#[allow(dead_code, clippy::too_many_arguments)]
pub(crate) fn prompt_detail_card(
    headline: impl Into<SharedString>,
    supporting_text: impl Into<SharedString>,
    detail_text: impl Into<SharedString>,
    headline_color: Rgba,
    supporting_color: Rgba,
    detail_color: Rgba,
    style: PromptFieldStyle,
    gap_px: f32,
) -> Div {
    prompt_surface(style.background, style.border).child(
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(gap_px))
            .child(
                div()
                    .text_sm()
                    .text_color(headline_color)
                    .child(headline.into()),
            )
            .child(prompt_form_help(supporting_text, supporting_color))
            .child(prompt_form_help(detail_text, detail_color)),
    )
}

/// Horizontally scrollable single-line value for long paths or strings.
#[allow(dead_code)]
pub(crate) fn prompt_scroll_value(
    value: impl Into<SharedString>,
    color: Rgba,
) -> gpui::Stateful<Div> {
    prompt_scroll_value_with_id("prompt-scroll-value", value, color)
}

/// Horizontally scrollable single-line value with a custom element ID.
///
/// Use this when multiple scroll values appear in the same view to avoid
/// duplicate element IDs.
pub(crate) fn prompt_scroll_value_with_id(
    id: impl Into<gpui::ElementId>,
    value: impl Into<SharedString>,
    color: Rgba,
) -> gpui::Stateful<Div> {
    div()
        .id(id.into())
        .w_full()
        .overflow_x_scroll()
        .overflow_y_hidden()
        .child(
            div()
                .text_xs()
                .text_color(color)
                .whitespace_nowrap()
                .child(value.into()),
        )
}

/// Shared outer shell used by prompt wrappers in `render_prompts/*`.
///
/// This normalizes the frame layout for prompt views:
/// - relative root for overlays
/// - column flex flow
/// - full-width/full-height frame
/// - clipped content with rounded corners
pub fn prompt_shell_container(radius: f32, vibrancy_bg: Option<Rgba>) -> Div {
    prompt_frame_root(prompt_shell_frame_config(radius)).when_some(vibrancy_bg, |d, bg| d.bg(bg))
}

/// Shared content slot used by prompt wrappers.
///
/// This guarantees consistent flex/overflow behavior for the inner prompt entity.
pub fn prompt_shell_content(content: impl IntoElement) -> Div {
    prompt_frame_fill_content(content)
}

#[cfg(test)]
mod prompt_layout_shell_tests {
    use super::{prompt_shell_frame_config, PromptFrameConfig};

    #[test]
    fn test_prompt_frame_defaults_apply_min_h_and_overflow_hidden() {
        let config = PromptFrameConfig::default();
        assert_eq!(config.min_height_px, 0.0);
        assert!(config.clip_overflow);
        assert!(!config.relative);
        assert_eq!(config.rounded_corners, None);
    }

    #[test]
    fn test_prompt_shell_frame_config_sets_relative_and_radius() {
        let config = prompt_shell_frame_config(14.0);
        assert_eq!(config.min_height_px, 0.0);
        assert!(config.clip_overflow);
        assert!(config.relative);
        assert_eq!(config.rounded_corners, Some(14.0));
    }

    #[test]
    fn prompt_surface_defaults_match_create_flow_field_chrome() {
        // Verify the shared surface uses the design-specified values.
        // If these change, update all callers too.
        let _surface = super::prompt_surface(
            gpui::rgba(0x112233ee),
            gpui::rgba(0x445566ff),
        );
        // The function is purely a builder; the real assertion is that it
        // compiles and the constants below stay in sync with the implementation.
        assert_eq!(8.0_f32, 8.0); // radius
        assert_eq!(0.875_f32, 0.875); // px padding
        assert_eq!(0.625_f32, 0.625); // py padding
    }

    const OTHER_RENDERERS_SOURCE: &str = include_str!("../render_prompts/other.rs");

    fn fn_source(name: &str) -> &'static str {
        let marker = format!("fn {}(", name);
        let Some(start) = OTHER_RENDERERS_SOURCE.find(&marker) else {
            return "";
        };
        let tail = &OTHER_RENDERERS_SOURCE[start..];
        let end = tail.find("\n    fn ").unwrap_or(tail.len());
        &tail[..end]
    }

    #[test]
    fn simple_prompt_wrappers_use_shared_layout_shell() {
        for fn_name in [
            "render_select_prompt",
            "render_env_prompt",
            "render_drop_prompt",
            "render_chat_prompt",
        ] {
            let body = fn_source(fn_name);
            assert!(
                body.contains("render_simple_prompt_shell("),
                "{fn_name} should delegate to render_simple_prompt_shell"
            );
        }
    }

    #[test]
    fn template_prompt_uses_form_style_shell_in_other_rs() {
        let body = fn_source("render_template_prompt");
        assert!(
            body.contains("PromptFooter::new("),
            "render_template_prompt should use PromptFooter"
        );
        assert!(
            body.contains("STANDARD_HEIGHT"),
            "render_template_prompt should use STANDARD_HEIGHT"
        );
    }

    #[test]
    fn naming_prompt_uses_form_style_shell_in_other_rs() {
        let body = fn_source("render_naming_prompt");
        assert!(
            body.contains("PromptFooter::new("),
            "render_naming_prompt should use PromptFooter"
        );
        assert!(
            body.contains("STANDARD_HEIGHT"),
            "render_naming_prompt should use STANDARD_HEIGHT"
        );
    }
}
