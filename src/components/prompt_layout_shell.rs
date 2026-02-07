use gpui::{div, prelude::*, px, Div, Rgba};

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

    const OTHER_RENDERERS_SOURCE: &str = include_str!("../render_prompts/other.rs");

    fn fn_source(name: &str) -> &'static str {
        let marker = format!("fn {}(", name);
        let start = OTHER_RENDERERS_SOURCE
            .find(&marker)
            .unwrap_or_else(|| panic!("missing function: {}", name));
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
            "render_template_prompt",
            "render_chat_prompt",
        ] {
            let body = fn_source(fn_name);
            assert!(
                body.contains("prompt_shell_container("),
                "{fn_name} should use prompt_shell_container"
            );
            assert!(
                body.contains("prompt_shell_content("),
                "{fn_name} should use prompt_shell_content"
            );
        }
    }
}
