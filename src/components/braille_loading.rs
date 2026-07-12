// Callers live in the bin target (render_impl/ui_window); the lib compile of
// this shared module would otherwise flag them dead (same as info_state.rs).
#![allow(dead_code)]

//! Braille loading treatment for slow-filling list surfaces.
//!
//! Two pieces share one motif — the 8-frame braille dot rotation:
//! - [`footer_braille_frame`]: the small footer spinner glyph (0.9s cycle)
//!   paired with a status label like "Fetching tabs".
//! - [`constellation_loading_layer`]: the "Constellation" ambient layer —
//!   four large, slow-cycling braille cells hung across the window, each
//!   breathing on its own long clock. Peak opacity stays ≈ 0.10–0.13 so the
//!   field reads *under* live list text and never competes with it.
//!
//! Both are pure functions of elapsed time; the caller owns the clock and
//! the repaint ticker (see `app_impl/main_list_loading.rs`).

use gpui::{div, prelude::*, px, relative, rgb, Div, Hsla};

/// The 8-frame braille dot rotation, in cycle order. Matches the classic
/// spinner sequence (U+280B → U+2807) so the motion reads clockwise.
pub const BRAILLE_SPINNER_FRAMES: [&str; 8] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠇"];

/// Footer spinner speed: one full 8-frame rotation per 0.9s.
pub const FOOTER_BRAILLE_CYCLE_SECS: f32 = 0.9;

/// The frame shown `elapsed_secs` into a rotation of period `cycle_secs`,
/// phase-shifted by `delay_secs` (negative delays advance the phase, like a
/// CSS `animation-delay`). Steps discretely — no cross-fade between frames.
pub fn braille_frame_at(cycle_secs: f32, delay_secs: f32, elapsed_secs: f32) -> &'static str {
    let phase = ((elapsed_secs - delay_secs) / cycle_secs).rem_euclid(1.0);
    let idx = ((phase * BRAILLE_SPINNER_FRAMES.len() as f32) as usize)
        .min(BRAILLE_SPINNER_FRAMES.len() - 1);
    BRAILLE_SPINNER_FRAMES[idx]
}

/// The footer spinner glyph for the given elapsed time.
pub fn footer_braille_frame(elapsed_secs: f32) -> &'static str {
    braille_frame_at(FOOTER_BRAILLE_CYCLE_SECS, 0.0, elapsed_secs)
}

/// One star of the constellation: where it hangs, how large it renders, and
/// the two independent clocks (opacity breath, frame rotation) it lives on.
struct ConstellationCell {
    /// Center position as a fraction of the window (0..1).
    x: f32,
    y: f32,
    /// Glyph size in pixels.
    size: f32,
    /// Peak opacity at the top of the breath.
    peak_alpha: f32,
    /// Opacity breath period / phase offset (seconds).
    breath_secs: f32,
    breath_delay: f32,
    /// Frame rotation period / phase offset (seconds).
    cycle_secs: f32,
    cycle_delay: f32,
}

/// The four-cell constellation. Positions, sizes, opacities, and clocks are
/// deliberately unsynchronized so the field shimmers instead of pulsing as a
/// block; every peak opacity stays within the calm 0.10–0.13 band.
const CONSTELLATION_CELLS: [ConstellationCell; 4] = [
    ConstellationCell {
        x: 0.20,
        y: 0.30,
        size: 44.0,
        peak_alpha: 0.13,
        breath_secs: 8.5,
        breath_delay: 0.0,
        cycle_secs: 2.6,
        cycle_delay: 0.0,
    },
    ConstellationCell {
        x: 0.71,
        y: 0.22,
        size: 52.0,
        peak_alpha: 0.11,
        breath_secs: 10.0,
        breath_delay: -3.2,
        cycle_secs: 3.0,
        cycle_delay: -1.1,
    },
    ConstellationCell {
        x: 0.44,
        y: 0.66,
        size: 60.0,
        peak_alpha: 0.12,
        breath_secs: 9.0,
        breath_delay: -5.5,
        cycle_secs: 2.8,
        cycle_delay: -2.0,
    },
    ConstellationCell {
        x: 0.85,
        y: 0.72,
        size: 40.0,
        peak_alpha: 0.10,
        breath_secs: 11.0,
        breath_delay: -1.7,
        cycle_secs: 3.4,
        cycle_delay: -0.6,
    },
];

/// The breath never fully extinguishes a cell — it bottoms out at this
/// fraction of the peak so the constellation stays legible as one shape.
const BREATH_FLOOR: f32 = 0.18;

/// How long the whole layer takes to fade in after loading starts, so a
/// fast refresh reads as a soft glow instead of a flash.
const LAYER_FADE_IN_SECS: f32 = 0.25;

/// Cosine "breath": peaks at `peak_alpha` mid-period, bottoms at
/// `BREATH_FLOOR * peak_alpha` at the period boundaries.
fn breath_alpha(cell: &ConstellationCell, elapsed_secs: f32) -> f32 {
    let phase = ((elapsed_secs - cell.breath_delay) / cell.breath_secs).rem_euclid(1.0);
    let wave = 0.5 - 0.5 * (phase * std::f32::consts::TAU).cos();
    cell.peak_alpha * (BREATH_FLOOR + (1.0 - BREATH_FLOOR) * wave)
}

/// Accent-derived glyph color: saturation trimmed and lightness adapted to
/// the theme background (glow on dark themes, ink on light ones), mirroring
/// how the background-effect palette keeps ambience behind content.
fn constellation_glyph_color(theme: &crate::theme::Theme) -> Hsla {
    let mut color: Hsla = rgb(theme.colors.accent.selected).into();
    let background: Hsla = rgb(theme.colors.background.main).into();
    let light_theme = background.l > 0.55;
    color.s = (color.s * 0.85).min(0.9);
    color.l = if light_theme {
        (color.l * 0.7).clamp(0.2, 0.55)
    } else {
        color.l.clamp(0.55, 0.8)
    };
    color
}

/// Full-window "Constellation" loading layer: four large braille cells, each
/// cycling the shared 8-frame rotation and breathing on its own clock.
/// Render absolutely positioned behind the list content (next to the
/// background-effect layer) while the slow fill is in flight.
pub fn constellation_loading_layer(theme: &crate::theme::Theme, elapsed_secs: f32) -> Div {
    let base_color = constellation_glyph_color(theme);
    let fade_in = (elapsed_secs / LAYER_FADE_IN_SECS).clamp(0.0, 1.0);

    div()
        .absolute()
        .inset_0()
        .overflow_hidden()
        .children(CONSTELLATION_CELLS.iter().map(|cell| {
            let mut color = base_color;
            color.a = breath_alpha(cell, elapsed_secs) * fade_in;
            div()
                .absolute()
                .left(relative(cell.x))
                .top(relative(cell.y))
                .w(px(cell.size))
                .h(px(cell.size))
                .ml(px(-cell.size / 2.0))
                .mt(px(-cell.size / 2.0))
                .flex()
                .items_center()
                .justify_center()
                .font_family(crate::list_item::FONT_MONO)
                .text_size(px(cell.size))
                .line_height(px(cell.size))
                .text_color(color)
                .child(braille_frame_at(
                    cell.cycle_secs,
                    cell.cycle_delay,
                    elapsed_secs,
                ))
        }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The rotation must step through all 8 frames in order over one period
    /// and wrap back to the first frame at the boundary.
    #[test]
    fn braille_rotation_steps_through_all_frames_in_order() {
        let cycle = 0.9;
        for (i, expected) in BRAILLE_SPINNER_FRAMES.iter().enumerate() {
            let t = (i as f32 + 0.5) / 8.0 * cycle;
            assert_eq!(braille_frame_at(cycle, 0.0, t), *expected, "frame {i}");
        }
        assert_eq!(
            braille_frame_at(cycle, 0.0, cycle),
            BRAILLE_SPINNER_FRAMES[0]
        );
        assert_eq!(
            braille_frame_at(cycle, 0.0, cycle * 2.0 + 0.01),
            BRAILLE_SPINNER_FRAMES[0]
        );
    }

    /// A negative delay advances the phase, exactly like a CSS
    /// `animation-delay`, so unsynchronized cells start mid-rotation.
    #[test]
    fn negative_delay_advances_the_phase() {
        let cycle = 2.0;
        let quarter = cycle / 4.0; // 2 frames into the 8-frame rotation
        assert_eq!(
            braille_frame_at(cycle, -quarter, 0.0),
            BRAILLE_SPINNER_FRAMES[2]
        );
    }

    /// The breath peaks at the cell's peak opacity mid-period and bottoms at
    /// the floor fraction, never reaching zero (the constellation must stay
    /// legible as one shape).
    #[test]
    fn breath_peaks_mid_period_and_never_extinguishes() {
        let cell = &CONSTELLATION_CELLS[0];
        let bottom = breath_alpha(cell, cell.breath_delay);
        let peak = breath_alpha(cell, cell.breath_delay + cell.breath_secs / 2.0);
        assert!((peak - cell.peak_alpha).abs() < 1e-4, "peak={peak}");
        assert!(
            (bottom - cell.peak_alpha * BREATH_FLOOR).abs() < 1e-4,
            "bottom={bottom}"
        );
        assert!(bottom > 0.0);
    }

    /// The calm contract: every cell's peak opacity stays in the ambient
    /// 0.10–0.13 band and every clock is slow (multi-second) so the layer
    /// reads under list text instead of fighting it.
    #[test]
    fn constellation_stays_calm() {
        for cell in &CONSTELLATION_CELLS {
            assert!((0.10..=0.13).contains(&cell.peak_alpha));
            assert!(cell.breath_secs >= 8.0);
            assert!(cell.cycle_secs >= 2.0);
            assert!((0.0..=1.0).contains(&cell.x));
            assert!((0.0..=1.0).contains(&cell.y));
        }
    }
}
