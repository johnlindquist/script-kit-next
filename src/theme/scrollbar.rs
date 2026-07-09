use crate::theme::Theme;

pub(crate) const SCROLLBAR_TRACK_DARK_OPACITY: f32 = 0.0;
pub(crate) const SCROLLBAR_TRACK_LIGHT_OPACITY: f32 = 0.0;
pub(crate) const SCROLLBAR_THUMB_MIN_OPACITY: f32 = 0.62;
pub(crate) const SCROLLBAR_THUMB_HOVER_MIN_OPACITY: f32 = 0.82;
pub(crate) const SCROLLBAR_THUMB_HOVER_DELTA: f32 = 0.18;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ScrollbarTheme {
    pub track: u32,
    pub track_opacity: f32,
    pub thumb: u32,
    pub thumb_opacity: f32,
    pub thumb_hover: u32,
    pub thumb_hover_opacity: f32,
}

#[inline]
fn clamp_alpha(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

pub(crate) fn scrollbar_theme(theme: &Theme) -> ScrollbarTheme {
    let opacity = theme.get_opacity();
    scrollbar_theme_from_tokens(
        theme.is_dark_mode(),
        theme.colors.ui.border,
        theme.colors.accent.selected,
        opacity.selected,
    )
}

pub(crate) fn scrollbar_theme_from_tokens(
    is_dark: bool,
    track: u32,
    thumb: u32,
    selected_opacity: f32,
) -> ScrollbarTheme {
    let track_opacity = if is_dark {
        SCROLLBAR_TRACK_DARK_OPACITY
    } else {
        SCROLLBAR_TRACK_LIGHT_OPACITY
    };

    let thumb_opacity = clamp_alpha(selected_opacity.max(SCROLLBAR_THUMB_MIN_OPACITY));
    let thumb_hover_opacity = clamp_alpha(
        (thumb_opacity + SCROLLBAR_THUMB_HOVER_DELTA).max(SCROLLBAR_THUMB_HOVER_MIN_OPACITY),
    );

    ScrollbarTheme {
        track,
        track_opacity,
        thumb,
        thumb_opacity,
        thumb_hover: thumb,
        thumb_hover_opacity,
    }
}

#[cfg(test)]
mod tests {
    use super::{scrollbar_theme_from_tokens, SCROLLBAR_THUMB_MIN_OPACITY};

    #[test]
    fn scrollbar_track_is_transparent_while_thumb_remains_visible() {
        for is_dark in [true, false] {
            let scrollbar = scrollbar_theme_from_tokens(is_dark, 0x112233, 0x445566, 0.4);

            assert_eq!(scrollbar.track_opacity, 0.0);
            assert!(scrollbar.thumb_opacity >= SCROLLBAR_THUMB_MIN_OPACITY);
            assert!(scrollbar.thumb_hover_opacity >= scrollbar.thumb_opacity);
        }
    }
}
