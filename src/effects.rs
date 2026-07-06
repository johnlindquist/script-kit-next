//! Procedural background shader effects.
//!
//! The heavy lifting happens in the vendored renderer: a
//! `BackgroundTag::ShaderEffect` background (see `vendor/gpui/src/color.rs`
//! and the `fx_*` functions in `vendor/gpui_macos/src/shaders.metal`) turns a
//! quad fill into a procedural effect selected by id and animated by a time
//! uniform. This module owns the app-side catalog: the effect roster in cycle
//! order, its theme-derived palette, and the full-window layer div the main
//! window renders behind its content.

use gpui::{div, prelude::*, rgb, Div, Hsla, Stateful};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

/// Built-in background effects, in cycle order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundEffect {
    Aurora,
    Plasma,
    Starfield,
    LavaLamp,
    Nebula,
    Rain,
    Waves,
    Fireflies,
    HueDrift,
    Grain,
    Scanlines,
    DotGrid,
    Caustics,
    Matrix,
    Breath,
    Confetti,
}

impl BackgroundEffect {
    /// All effects in cycle order.
    pub fn all() -> &'static [BackgroundEffect] {
        &[
            BackgroundEffect::Aurora,
            BackgroundEffect::Plasma,
            BackgroundEffect::Starfield,
            BackgroundEffect::LavaLamp,
            BackgroundEffect::Nebula,
            BackgroundEffect::Rain,
            BackgroundEffect::Waves,
            BackgroundEffect::Fireflies,
            BackgroundEffect::HueDrift,
            BackgroundEffect::Grain,
            BackgroundEffect::Scanlines,
            BackgroundEffect::DotGrid,
            BackgroundEffect::Caustics,
            BackgroundEffect::Matrix,
            BackgroundEffect::Breath,
            BackgroundEffect::Confetti,
        ]
    }

    /// The `case` this effect selects in the renderer's shader dispatch.
    pub fn shader_id(self) -> u32 {
        match self {
            BackgroundEffect::Aurora => 1,
            BackgroundEffect::Plasma => 2,
            BackgroundEffect::Starfield => 3,
            BackgroundEffect::LavaLamp => 4,
            BackgroundEffect::Nebula => 5,
            BackgroundEffect::Rain => 6,
            BackgroundEffect::Waves => 7,
            BackgroundEffect::Fireflies => 8,
            BackgroundEffect::HueDrift => 9,
            BackgroundEffect::Grain => 10,
            BackgroundEffect::Scanlines => 11,
            BackgroundEffect::DotGrid => 12,
            BackgroundEffect::Caustics => 13,
            BackgroundEffect::Matrix => 14,
            BackgroundEffect::Breath => 15,
            BackgroundEffect::Confetti => 16,
        }
    }

    /// Human-readable name shown in HUDs and list entries.
    pub fn name(self) -> &'static str {
        match self {
            BackgroundEffect::Aurora => "Aurora",
            BackgroundEffect::Plasma => "Plasma",
            BackgroundEffect::Starfield => "Starfield",
            BackgroundEffect::LavaLamp => "Lava Lamp",
            BackgroundEffect::Nebula => "Nebula",
            BackgroundEffect::Rain => "Rain",
            BackgroundEffect::Waves => "Ocean Waves",
            BackgroundEffect::Fireflies => "Fireflies",
            BackgroundEffect::HueDrift => "Hue Drift",
            BackgroundEffect::Grain => "Film Grain",
            BackgroundEffect::Scanlines => "Scanlines",
            BackgroundEffect::DotGrid => "Dot Grid",
            BackgroundEffect::Caustics => "Caustics",
            BackgroundEffect::Matrix => "Matrix",
            BackgroundEffect::Breath => "Breath",
            BackgroundEffect::Confetti => "Confetti",
        }
    }

    /// Stable identifier persisted in preferences.
    pub fn slug(self) -> &'static str {
        match self {
            BackgroundEffect::Aurora => "aurora",
            BackgroundEffect::Plasma => "plasma",
            BackgroundEffect::Starfield => "starfield",
            BackgroundEffect::LavaLamp => "lava-lamp",
            BackgroundEffect::Nebula => "nebula",
            BackgroundEffect::Rain => "rain",
            BackgroundEffect::Waves => "waves",
            BackgroundEffect::Fireflies => "fireflies",
            BackgroundEffect::HueDrift => "hue-drift",
            BackgroundEffect::Grain => "grain",
            BackgroundEffect::Scanlines => "scanlines",
            BackgroundEffect::DotGrid => "dot-grid",
            BackgroundEffect::Caustics => "caustics",
            BackgroundEffect::Matrix => "matrix",
            BackgroundEffect::Breath => "breath",
            BackgroundEffect::Confetti => "confetti",
        }
    }

    /// Resolve a persisted slug back to an effect.
    pub fn from_slug(slug: &str) -> Option<BackgroundEffect> {
        BackgroundEffect::all()
            .iter()
            .copied()
            .find(|effect| effect.slug() == slug)
    }

    /// The next effect in cycle order, wrapping.
    pub fn next(self) -> BackgroundEffect {
        let all = Self::all();
        let idx = all.iter().position(|&e| e == self).unwrap_or(0);
        all[(idx + 1) % all.len()]
    }

    /// The previous effect in cycle order, wrapping.
    pub fn prev(self) -> BackgroundEffect {
        let all = Self::all();
        let idx = all.iter().position(|&e| e == self).unwrap_or(0);
        all[(idx + all.len() - 1) % all.len()]
    }

    /// Hue offset (0..1) between the effect's two palette colors. Tiny and
    /// rotated toward the warm side of the accent: anything past ~0.06
    /// stops reading as "the theme accent" (a gold accent lands in green),
    /// so the second color is mostly a warmth/depth variation, not a new hue.
    fn hue_shift(self) -> f32 {
        match self {
            BackgroundEffect::Aurora => 0.05,
            BackgroundEffect::Plasma => 0.06,
            BackgroundEffect::Starfield => 0.04,
            BackgroundEffect::LavaLamp => 0.04,
            BackgroundEffect::Nebula => 0.05,
            BackgroundEffect::Rain => 0.03,
            BackgroundEffect::Waves => 0.04,
            BackgroundEffect::Fireflies => 0.04,
            BackgroundEffect::HueDrift => 0.06,
            BackgroundEffect::Grain => 0.0,
            BackgroundEffect::Scanlines => 0.0,
            BackgroundEffect::DotGrid => 0.05,
            BackgroundEffect::Caustics => 0.04,
            BackgroundEffect::Matrix => 0.05,
            BackgroundEffect::Breath => 0.05,
            BackgroundEffect::Confetti => 0.06,
        }
    }
}

/// Sources that can move the effect focus point. Each source remembers its
/// own last position so a stationary source re-reporting every frame (e.g.
/// the selected row during the 30fps effect ticker) neither fights another
/// source for the target nor re-fires the change pulse.
#[derive(Clone, Copy)]
pub enum EffectFocusSource {
    /// The selected/focused list row (center of its painted bounds).
    FocusedItem = 0,
    /// The text caret in a focused input.
    TextCursor = 1,
    /// The mouse pointer while it moves over the main window.
    Mouse = 2,
}

/// Movement (in normalized window units) below which a re-report is treated
/// as "no change" — filters paint jitter and caret blinks.
const FOCUS_EPSILON: f32 = 0.004;

const FOCUS_DEFAULT_X: f32 = 0.5;
const FOCUS_DEFAULT_Y: f32 = 0.35;

/// Per-source last-reported positions (f32 bits; index = source * 2 + axis).
static SOURCE_LAST: [AtomicU32; 6] = [
    AtomicU32::new(f32::to_bits(-1.0)),
    AtomicU32::new(f32::to_bits(-1.0)),
    AtomicU32::new(f32::to_bits(-1.0)),
    AtomicU32::new(f32::to_bits(-1.0)),
    AtomicU32::new(f32::to_bits(-1.0)),
    AtomicU32::new(f32::to_bits(-1.0)),
];
/// Where the shader focus is headed (latest change wins).
static FOCUS_TARGET_X: AtomicU32 = AtomicU32::new(f32::to_bits(FOCUS_DEFAULT_X));
static FOCUS_TARGET_Y: AtomicU32 = AtomicU32::new(f32::to_bits(FOCUS_DEFAULT_Y));
/// Smoothed focus actually handed to the shader.
static FOCUS_SMOOTH_X: AtomicU32 = AtomicU32::new(f32::to_bits(FOCUS_DEFAULT_X));
static FOCUS_SMOOTH_Y: AtomicU32 = AtomicU32::new(f32::to_bits(FOCUS_DEFAULT_Y));
/// Micros since `effect_epoch` of the last change / last smoothing step.
static LAST_CHANGE_MICROS: AtomicU64 = AtomicU64::new(0);
static LAST_SMOOTH_MICROS: AtomicU64 = AtomicU64::new(0);

fn effect_epoch() -> Instant {
    static EPOCH: OnceLock<Instant> = OnceLock::new();
    *EPOCH.get_or_init(Instant::now)
}

fn micros_now() -> u64 {
    effect_epoch().elapsed().as_micros() as u64
}

/// Report where a focus source currently is, in normalized (0..1) main-window
/// coordinates. Only an actual move (per source) retargets the shader focus
/// and re-fires the change pulse.
pub fn note_effect_focus(source: EffectFocusSource, x: f32, y: f32) {
    let x = x.clamp(0.0, 1.0);
    let y = y.clamp(0.0, 1.0);
    let base = (source as usize) * 2;
    let last_x = f32::from_bits(SOURCE_LAST[base].load(Ordering::Relaxed));
    let last_y = f32::from_bits(SOURCE_LAST[base + 1].load(Ordering::Relaxed));
    if (x - last_x).abs() + (y - last_y).abs() <= FOCUS_EPSILON {
        return;
    }
    SOURCE_LAST[base].store(x.to_bits(), Ordering::Relaxed);
    SOURCE_LAST[base + 1].store(y.to_bits(), Ordering::Relaxed);
    // A source's first report seeds its memory without claiming the focus,
    // so e.g. mounting a probe doesn't yank the target across the window.
    if last_x < 0.0 {
        return;
    }
    FOCUS_TARGET_X.store(x.to_bits(), Ordering::Relaxed);
    FOCUS_TARGET_Y.store(y.to_bits(), Ordering::Relaxed);
    let now = micros_now();
    LAST_CHANGE_MICROS.store(now, Ordering::Relaxed);

    // Throttled diagnostic so devtools probes can confirm which source is
    // steering the effect focus without spamming the log ring.
    static LAST_LOG_MICROS: AtomicU64 = AtomicU64::new(0);
    let last_log = LAST_LOG_MICROS.load(Ordering::Relaxed);
    if now.saturating_sub(last_log) >= 500_000 {
        LAST_LOG_MICROS.store(now, Ordering::Relaxed);
        crate::logging::log(
            "EFFECT_FOCUS",
            &format!(
                "source={} x={x:.3} y={y:.3}",
                match source {
                    EffectFocusSource::FocusedItem => "focused-item",
                    EffectFocusSource::TextCursor => "text-cursor",
                    EffectFocusSource::Mouse => "mouse",
                }
            ),
        );
    }
}

/// The shader uniforms derived from the recorded focus/change state: a
/// critically-damped-ish smoothed focus point and the seconds since the last
/// change. Called once per effect-layer render; advances the smoothing.
fn effect_focus_uniforms() -> ([f32; 2], f32) {
    let now = micros_now();
    let last = LAST_SMOOTH_MICROS.swap(now, Ordering::Relaxed);
    let dt = (now.saturating_sub(last) as f32 / 1_000_000.0).min(0.25);
    // ~120ms time constant: quick enough to feel attached, slow enough to glide.
    let k = 1.0 - (-dt * 8.0).exp();
    let tx = f32::from_bits(FOCUS_TARGET_X.load(Ordering::Relaxed));
    let ty = f32::from_bits(FOCUS_TARGET_Y.load(Ordering::Relaxed));
    let mut sx = f32::from_bits(FOCUS_SMOOTH_X.load(Ordering::Relaxed));
    let mut sy = f32::from_bits(FOCUS_SMOOTH_Y.load(Ordering::Relaxed));
    sx += (tx - sx) * k;
    sy += (ty - sy) * k;
    FOCUS_SMOOTH_X.store(sx.to_bits(), Ordering::Relaxed);
    FOCUS_SMOOTH_Y.store(sy.to_bits(), Ordering::Relaxed);
    let pulse = now.saturating_sub(LAST_CHANGE_MICROS.load(Ordering::Relaxed)) as f32 / 1_000_000.0;
    ([sx, sy], pulse)
}

/// A zero-cost overlay that records the center of its painted bounds as the
/// effect focus point. Mount it absolutely inside the element that carries
/// the app's attention (selected row, caret). Records only in the main
/// window so popups and secondary windows don't steer the main background.
pub fn effect_focus_probe(source: EffectFocusSource) -> impl IntoElement {
    gpui::canvas(
        move |bounds, window, _cx| {
            if crate::get_main_window_handle() != Some(window.window_handle()) {
                return;
            }
            let viewport = window.viewport_size();
            let (vw, vh) = (f32::from(viewport.width), f32::from(viewport.height));
            if vw <= 0.0 || vh <= 0.0 {
                return;
            }
            let center = bounds.center();
            note_effect_focus(source, f32::from(center.x) / vw, f32::from(center.y) / vh);
        },
        |_, _, _, _| {},
    )
    .absolute()
    .inset_0()
}

/// Startup snapshot of the persisted effect prefs, read once so both
/// `ScriptListApp` constructors can initialize from it without re-reading
/// disk. Runtime changes go through `set_background_effect`, which updates
/// entity state directly.
fn startup_prefs() -> &'static (Option<BackgroundEffect>, f32) {
    static PREFS: std::sync::OnceLock<(Option<BackgroundEffect>, f32)> = std::sync::OnceLock::new();
    PREFS.get_or_init(|| {
        let prefs = crate::config::load_user_preferences();
        (
            prefs
                .effects
                .background
                .as_deref()
                .and_then(BackgroundEffect::from_slug),
            prefs.effects.intensity(),
        )
    })
}

/// The background effect persisted in preferences, if any.
pub fn initial_background_effect() -> Option<BackgroundEffect> {
    startup_prefs().0
}

/// The effect intensity persisted in preferences.
pub fn initial_background_effect_intensity() -> f32 {
    startup_prefs().1
}

/// Full-window layer carrying the shader-effect background. Rendered
/// absolutely positioned behind the main window content, clipped by the
/// window's rounded container.
///
/// The palette derives from the user's current theme: both colors anchor to
/// the theme accent (analogous hue offset only), get a slight saturation
/// trim so they sit behind content instead of competing with it, and adapt
/// their lightness to the theme background so the effect stays a quiet glow
/// on dark themes and a soft tint on light ones.
pub fn background_effect_layer(
    theme: &crate::theme::Theme,
    effect: BackgroundEffect,
    intensity: f32,
    elapsed_secs: f32,
) -> Stateful<Div> {
    let accent: Hsla = rgb(theme.colors.accent.selected).into();
    let background: Hsla = rgb(theme.colors.background.main).into();
    let light_theme = background.l > 0.55;

    let tune = |mut color: Hsla| -> Hsla {
        color.s = (color.s * 0.85).min(0.9);
        color.l = if light_theme {
            // Darker, inkier tones read as a soft tint over light surfaces.
            (color.l * 0.7).clamp(0.2, 0.55)
        } else {
            // Lifted tones glow gently over dark surfaces.
            color.l.clamp(0.55, 0.8)
        };
        color.a = intensity;
        color
    };

    let color_a = tune(accent);
    let mut color_b = accent;
    // Rotate toward the warm side so gold stays gold (never green) and the
    // pair reads as one accent with depth; wrap via +1.0 to keep h in 0..1.
    color_b.h = (accent.h - effect.hue_shift() + 1.0).fract();
    let color_b = tune(color_b);

    let (focus, pulse) = effect_focus_uniforms();

    div()
        .id("bg-effect-layer")
        .absolute()
        .inset_0()
        .bg(gpui::shader_effect(
            effect.shader_id(),
            elapsed_secs,
            focus,
            pulse,
            color_a,
            color_b,
        ))
}

#[cfg(test)]
mod tests {
    use super::BackgroundEffect;

    /// One combined sequence test because the recorder is (intentionally)
    /// process-global state: seeding must not claim the focus target, a real
    /// move must retarget and re-arm the change pulse, smoothing must walk
    /// toward the target, and sub-epsilon jitter must not re-fire the pulse.
    #[test]
    fn focus_recorder_seeds_tracks_and_pulses() {
        use super::{effect_focus_uniforms, note_effect_focus, EffectFocusSource};

        // First report from a source only seeds its memory.
        note_effect_focus(EffectFocusSource::Mouse, 0.9, 0.9);
        let ([x0, _y0], _) = effect_focus_uniforms();
        assert!(
            (x0 - super::FOCUS_DEFAULT_X).abs() < 0.05,
            "seed report must not claim the focus target (x0={x0})"
        );

        // A real move claims the target and re-arms the pulse.
        note_effect_focus(EffectFocusSource::Mouse, 0.1, 0.2);
        let (_, pulse) = effect_focus_uniforms();
        assert!(pulse < 0.5, "move must re-arm the change pulse ({pulse})");

        // Smoothing converges toward the new target.
        std::thread::sleep(std::time::Duration::from_millis(60));
        let ([x1, y1], _) = effect_focus_uniforms();
        std::thread::sleep(std::time::Duration::from_millis(60));
        let ([x2, y2], _) = effect_focus_uniforms();
        assert!((x1 - 0.1).abs() <= (x0 - 0.1).abs() + 1e-4);
        assert!((x2 - 0.1).abs() <= (x1 - 0.1).abs() + 1e-4);
        assert!((y2 - 0.2).abs() <= (y1 - 0.2).abs() + 1e-4);

        // Sub-epsilon jitter (paint re-reports, caret blink) is ignored.
        std::thread::sleep(std::time::Duration::from_millis(40));
        note_effect_focus(EffectFocusSource::Mouse, 0.101, 0.2);
        let (_, pulse_after_jitter) = effect_focus_uniforms();
        assert!(
            pulse_after_jitter >= 0.1,
            "jitter must not re-fire the pulse ({pulse_after_jitter})"
        );
    }

    #[test]
    fn cycle_wraps_in_both_directions() {
        let all = BackgroundEffect::all();
        assert_eq!(all.last().unwrap().next(), all[0]);
        assert_eq!(all[0].prev(), *all.last().unwrap());
        for window in all.windows(2) {
            assert_eq!(window[0].next(), window[1]);
            assert_eq!(window[1].prev(), window[0]);
        }
    }

    #[test]
    fn slugs_round_trip_and_are_unique() {
        let all = BackgroundEffect::all();
        for effect in all {
            assert_eq!(BackgroundEffect::from_slug(effect.slug()), Some(*effect));
        }
        let mut slugs: Vec<_> = all.iter().map(|e| e.slug()).collect();
        slugs.sort();
        slugs.dedup();
        assert_eq!(slugs.len(), all.len());
    }

    #[test]
    fn shader_ids_are_unique_and_match_the_metal_dispatch_range() {
        let mut ids: Vec<_> = BackgroundEffect::all()
            .iter()
            .map(|e| e.shader_id())
            .collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), BackgroundEffect::all().len());
        assert!(ids.iter().all(|&id| (1..=16).contains(&id)));
    }
}
