//! Decorative chrome for scrollable list boundaries.
//!
//! The renderer deliberately owns no interaction or scroll state. Callers
//! derive progress from logical geometry and mount this fixed paint layer
//! above the translated row subtree, leaving hit testing and scrollbars alone.

use gpui::{div, linear_color_stop, linear_gradient, prelude::*, px, rgba, AnyElement, Pixels};

use crate::designs::MainMenuListTokens;

#[inline]
#[allow(
    dead_code,
    reason = "the separately compiled root-list renderer computes its fixed boundary occlusion"
)]
pub(crate) fn top_occlusion_alpha(tokens: MainMenuListTokens, progress: f32) -> u32 {
    ((tokens.top_occlusion_peak_alpha as f32 * progress.clamp(0.0, 1.0)).round() as u32).min(0xFF)
}

#[allow(
    dead_code,
    reason = "the separately compiled root-list renderer assigns this stable paint selector"
)]
pub(crate) const MAIN_LIST_TOP_OCCLUSION_ID: &str = "main-list-top-occlusion";

#[allow(
    dead_code,
    reason = "the separately compiled launcher calls this from render_script_list/mod.rs"
)]
pub(crate) fn render_top_occlusion_at(
    theme: &crate::theme::Theme,
    tokens: MainMenuListTokens,
    progress: f32,
    top: Pixels,
) -> AnyElement {
    let alpha = top_occlusion_alpha(tokens, progress);
    let base = theme.colors.background.main;
    let opaque = rgba((base << 8) | alpha);
    let transparent = rgba(base << 8);

    div()
        .id(MAIN_LIST_TOP_OCCLUSION_ID)
        .debug_selector(|| MAIN_LIST_TOP_OCCLUSION_ID.to_string())
        .absolute()
        .top(top)
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

// ── Jump-to-latest affordance ─────────────────────────────────────────────

/// Should a transcript show a "jump to latest" affordance?
///
/// Pure, and deliberately derived from only two facts:
///
/// - `can_scroll_y` — there is somewhere to jump to. A short transcript that
///   fits its viewport must never show the pill, or it sits there permanently
///   pointing at content already fully visible.
/// - `is_following_tail` — the list is NOT parked at the bottom. This comes
///   straight from `ListState::is_following_tail()`, the same authority the
///   list itself uses to decide whether incoming rows scroll into view.
///
/// The important property is that this reads EXISTING scroll state rather than
/// tracking a parallel `user_scrolled_up` flag. A duplicate flag is how a pill
/// starts lying: the list resumes tail-following on its own (new content, a
/// programmatic scroll, a resize) and the shadow flag never hears about it, so
/// the pill lingers over a transcript that is already at the bottom.
#[inline]
pub(crate) fn should_show_jump_to_latest(can_scroll_y: bool, is_following_tail: bool) -> bool {
    can_scroll_y && !is_following_tail
}

#[cfg(test)]
mod jump_to_latest_tests {
    use super::*;

    #[test]
    fn hidden_when_the_transcript_is_not_scrollable() {
        // A short conversation fits its viewport. Showing the pill here would
        // park a permanent control over content already fully visible.
        assert!(!should_show_jump_to_latest(false, true));
        assert!(!should_show_jump_to_latest(false, false));
    }

    #[test]
    fn hidden_while_following_the_tail() {
        assert!(!should_show_jump_to_latest(true, true));
    }

    #[test]
    fn visible_only_when_scrollable_and_scrolled_off_tail() {
        assert!(should_show_jump_to_latest(true, false));
    }

    /// Exhaustive over both inputs: exactly one of four combinations shows it.
    #[test]
    fn exactly_one_combination_shows_the_pill() {
        let shown = [(true, true), (true, false), (false, true), (false, false)]
            .into_iter()
            .filter(|(scrollable, following)| should_show_jump_to_latest(*scrollable, *following))
            .count();
        assert_eq!(shown, 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_occlusion_alpha_clamps_progress_and_uses_peak_token() {
        let tokens = crate::designs::MainMenuThemeVariant::InfoBarBase.def().list;
        assert_eq!(top_occlusion_alpha(tokens, -1.0), 0);
        assert_eq!(top_occlusion_alpha(tokens, 0.5), 0x17);
        assert_eq!(top_occlusion_alpha(tokens, 1.0), 0x2E);
        assert_eq!(top_occlusion_alpha(tokens, 2.0), 0x2E);
    }
}
