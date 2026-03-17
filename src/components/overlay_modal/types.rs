use std::time::Duration;

use crate::transitions;

/// Shared constants for overlay modal styling.
///
/// Both `AliasInput` and `ShortcutRecorder` consume these instead of
/// maintaining their own copies.
pub(crate) const MODAL_WIDTH: f32 = 420.0;
pub(crate) const MODAL_PADDING: f32 = 24.0;
pub(crate) const BUTTON_GAP: f32 = 12.0;
pub(crate) const OVERLAY_ANIMATION_DURATION_MS: u64 = 140;
pub(crate) const OVERLAY_MODAL_ENTRY_OFFSET_PX: f32 = 12.0;
pub(crate) const OVERLAY_MODAL_START_OPACITY: f32 = 0.82;

/// Pre-computed animation state for overlay entrance (fade + slide-in).
#[derive(Clone, Copy, Debug)]
pub(crate) struct OverlayAppearStyle {
    pub(crate) backdrop_opacity: f32,
    pub(crate) modal_opacity: f32,
    pub(crate) modal_offset_y: f32,
    pub(crate) complete: bool,
}

/// Compute the overlay entrance animation state from the elapsed time since
/// the overlay was shown.
pub(crate) fn compute_overlay_appear_style(elapsed: Duration) -> OverlayAppearStyle {
    let progress =
        (elapsed.as_secs_f32() / (OVERLAY_ANIMATION_DURATION_MS as f32 / 1000.0)).clamp(0.0, 1.0);
    let eased = transitions::ease_out_quad(progress);
    let modal_opacity = OVERLAY_MODAL_START_OPACITY + ((1.0 - OVERLAY_MODAL_START_OPACITY) * eased);

    OverlayAppearStyle {
        backdrop_opacity: eased,
        modal_opacity,
        modal_offset_y: OVERLAY_MODAL_ENTRY_OFFSET_PX * (1.0 - eased),
        complete: progress >= 1.0,
    }
}

/// Combine a 24-bit RGB color with an 8-bit alpha into a 32-bit RGBA value
/// suitable for `gpui::rgba()`.
pub(crate) fn overlay_color_with_alpha(color: u32, alpha: u8) -> u32 {
    ((color & 0x00ff_ffff) << 8) | (alpha as u32)
}
