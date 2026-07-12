//! Decorative chrome for scrollable list boundaries.
//!
//! The renderer deliberately owns no interaction or scroll state. Callers
//! derive progress from logical geometry and mount this fixed paint layer
//! above the translated row subtree, leaving hit testing and scrollbars alone.

use gpui::{div, linear_color_stop, linear_gradient, prelude::*, px, rgba, AnyElement};

use crate::designs::MainMenuListTokens;

#[inline]
pub(crate) fn top_occlusion_alpha(tokens: MainMenuListTokens, progress: f32) -> u32 {
    ((tokens.top_occlusion_peak_alpha as f32 * progress.clamp(0.0, 1.0)).round() as u32).min(0xFF)
}

/// Paint a fixed top-to-transparent occlusion plane over scrolled rows.
///
/// This element intentionally has no id, listeners, or occlusion behavior. It
/// stops before the scrollbar so the overlay cannot steal pointer/wheel input.
pub(crate) fn render_top_occlusion(
    theme: &crate::theme::Theme,
    tokens: MainMenuListTokens,
    progress: f32,
) -> AnyElement {
    let alpha = top_occlusion_alpha(tokens, progress);
    let base = theme.colors.background.main;
    let opaque = rgba((base << 8) | alpha);
    let transparent = rgba(base << 8);

    div()
        .absolute()
        .top_0()
        .left_0()
        .right(px(tokens.scrollbar_width))
        .h(px(tokens.top_occlusion_height))
        .bg(linear_gradient(
            180.0,
            linear_color_stop(opaque, 0.0),
            linear_color_stop(transparent, 1.0),
        ))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_occlusion_alpha_clamps_progress_and_uses_peak_token() {
        let tokens = crate::designs::MainMenuThemeVariant::InfoBarBase.def().list;
        assert_eq!(top_occlusion_alpha(tokens, -1.0), 0);
        assert_eq!(top_occlusion_alpha(tokens, 0.5), 0x66);
        assert_eq!(top_occlusion_alpha(tokens, 1.0), 0xCC);
        assert_eq!(top_occlusion_alpha(tokens, 2.0), 0xCC);
    }
}
