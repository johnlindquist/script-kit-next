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
    Silk,
    Dunes,
    Moonwater,
    Petals,
    ZenGarden,
    InkWash,
    Marble,
    TreeRings,
    SeaGlass,
    KoiPond,
    Bamboo,
    Candlelight,
    Jellyfish,
    Lotus,
    SoftPrism,
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
            BackgroundEffect::Silk,
            BackgroundEffect::Dunes,
            BackgroundEffect::Moonwater,
            BackgroundEffect::Petals,
            BackgroundEffect::ZenGarden,
            BackgroundEffect::InkWash,
            BackgroundEffect::Marble,
            BackgroundEffect::TreeRings,
            BackgroundEffect::SeaGlass,
            BackgroundEffect::KoiPond,
            BackgroundEffect::Bamboo,
            BackgroundEffect::Candlelight,
            BackgroundEffect::Jellyfish,
            BackgroundEffect::Lotus,
            BackgroundEffect::SoftPrism,
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
            BackgroundEffect::Silk => 17,
            BackgroundEffect::Dunes => 18,
            BackgroundEffect::Moonwater => 19,
            BackgroundEffect::Petals => 20,
            BackgroundEffect::ZenGarden => 21,
            BackgroundEffect::InkWash => 22,
            BackgroundEffect::Marble => 23,
            BackgroundEffect::TreeRings => 24,
            BackgroundEffect::SeaGlass => 25,
            BackgroundEffect::KoiPond => 26,
            BackgroundEffect::Bamboo => 27,
            BackgroundEffect::Candlelight => 28,
            BackgroundEffect::Jellyfish => 29,
            BackgroundEffect::Lotus => 30,
            BackgroundEffect::SoftPrism => 31,
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
            BackgroundEffect::Silk => "Silk",
            BackgroundEffect::Dunes => "Sand Dunes",
            BackgroundEffect::Moonwater => "Moonlit Water",
            BackgroundEffect::Petals => "Drifting Petals",
            BackgroundEffect::ZenGarden => "Zen Garden",
            BackgroundEffect::InkWash => "Ink Wash",
            BackgroundEffect::Marble => "Marble",
            BackgroundEffect::TreeRings => "Tree Rings",
            BackgroundEffect::SeaGlass => "Sea Glass",
            BackgroundEffect::KoiPond => "Koi Pond",
            BackgroundEffect::Bamboo => "Bamboo",
            BackgroundEffect::Candlelight => "Candlelight",
            BackgroundEffect::Jellyfish => "Jellyfish",
            BackgroundEffect::Lotus => "Lotus",
            BackgroundEffect::SoftPrism => "Soft Prism",
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
            BackgroundEffect::Silk => "silk",
            BackgroundEffect::Dunes => "dunes",
            BackgroundEffect::Moonwater => "moonwater",
            BackgroundEffect::Petals => "petals",
            BackgroundEffect::ZenGarden => "zen-garden",
            BackgroundEffect::InkWash => "ink-wash",
            BackgroundEffect::Marble => "marble",
            BackgroundEffect::TreeRings => "tree-rings",
            BackgroundEffect::SeaGlass => "sea-glass",
            BackgroundEffect::KoiPond => "koi-pond",
            BackgroundEffect::Bamboo => "bamboo",
            BackgroundEffect::Candlelight => "candlelight",
            BackgroundEffect::Jellyfish => "jellyfish",
            BackgroundEffect::Lotus => "lotus",
            BackgroundEffect::SoftPrism => "soft-prism",
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
            BackgroundEffect::Silk => 0.04,
            BackgroundEffect::Dunes => 0.03,
            BackgroundEffect::Moonwater => 0.03,
            BackgroundEffect::Petals => 0.05,
            BackgroundEffect::ZenGarden => 0.02,
            BackgroundEffect::InkWash => 0.02,
            BackgroundEffect::Marble => 0.03,
            BackgroundEffect::TreeRings => 0.03,
            BackgroundEffect::SeaGlass => 0.05,
            BackgroundEffect::KoiPond => 0.05,
            BackgroundEffect::Bamboo => 0.04,
            BackgroundEffect::Candlelight => 0.04,
            BackgroundEffect::Jellyfish => 0.05,
            BackgroundEffect::Lotus => 0.05,
            BackgroundEffect::SoftPrism => 0.06,
        }
    }
}

/// Sources that can move the effect focus point. Each source remembers its
/// own last position so a stationary source re-reporting every frame (e.g.
/// the selected row during the 30fps effect ticker) neither fights another
/// source for the target nor re-arms the activity-energy envelope.
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
/// Smoothed 0..1 activity energy handed to the shader (f32 bits).
static FOCUS_ENERGY: AtomicU32 = AtomicU32::new(f32::to_bits(0.0));
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
/// and re-arms the activity-energy envelope.
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
/// critically-damped-ish smoothed focus point and a smoothed 0..1 activity
/// energy. Called once per effect-layer render; advances the smoothing.
///
/// The energy is deliberately an envelope, not a raw event timer: each
/// change re-arms a target that decays over ~0.5s, and the reported energy
/// chases that target with a ~300ms time constant. A single change reads as
/// one slow breath; sustained typing or mouse motion reads as one steady
/// gentle glow — never a per-keystroke throb.
fn effect_focus_uniforms() -> ([f32; 2], f32) {
    let now = micros_now();
    let last = LAST_SMOOTH_MICROS.swap(now, Ordering::Relaxed);
    let dt = (now.saturating_sub(last) as f32 / 1_000_000.0).min(0.25);
    // ~170ms time constant: attached to the attention point, but gliding.
    let k = 1.0 - (-dt * 6.0).exp();
    let tx = f32::from_bits(FOCUS_TARGET_X.load(Ordering::Relaxed));
    let ty = f32::from_bits(FOCUS_TARGET_Y.load(Ordering::Relaxed));
    let mut sx = f32::from_bits(FOCUS_SMOOTH_X.load(Ordering::Relaxed));
    let mut sy = f32::from_bits(FOCUS_SMOOTH_Y.load(Ordering::Relaxed));
    sx += (tx - sx) * k;
    sy += (ty - sy) * k;
    FOCUS_SMOOTH_X.store(sx.to_bits(), Ordering::Relaxed);
    FOCUS_SMOOTH_Y.store(sy.to_bits(), Ordering::Relaxed);
    let since_change =
        now.saturating_sub(LAST_CHANGE_MICROS.load(Ordering::Relaxed)) as f32 / 1_000_000.0;
    let target = (-since_change * 2.0).exp();
    let mut energy = f32::from_bits(FOCUS_ENERGY.load(Ordering::Relaxed));
    energy += (target - energy) * (1.0 - (-dt * 3.5).exp());
    energy = energy.clamp(0.0, 1.0);
    FOCUS_ENERGY.store(energy.to_bits(), Ordering::Relaxed);
    ([sx, sy], energy)
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
            BackgroundEffect::resolve_pref(prefs.effects.background.as_deref()),
            prefs.effects.intensity(),
        )
    })
}

/// The effect a fresh install ships with.
pub const DEFAULT_BACKGROUND_EFFECT: BackgroundEffect = BackgroundEffect::Starfield;

/// Sentinel slug persisted when the user explicitly turns effects off.
/// `background` is skipped when `None` on serialize, so absence must stay
/// free to mean "use the default" — off needs its own value.
pub const BACKGROUND_EFFECT_OFF_SLUG: &str = "off";

impl BackgroundEffect {
    /// Resolve the persisted `effects.background` preference:
    /// - absent → the install default ([`DEFAULT_BACKGROUND_EFFECT`])
    /// - "off"/"none" (any case) → no effect
    /// - a known slug → that effect
    /// - anything else → no effect (fail quiet, never fail loud at startup)
    pub fn resolve_pref(pref: Option<&str>) -> Option<BackgroundEffect> {
        match pref {
            None => Some(DEFAULT_BACKGROUND_EFFECT),
            Some(raw) => {
                let slug = raw.trim();
                if slug.is_empty()
                    || slug.eq_ignore_ascii_case(BACKGROUND_EFFECT_OFF_SLUG)
                    || slug.eq_ignore_ascii_case("none")
                {
                    None
                } else {
                    BackgroundEffect::from_slug(slug)
                }
            }
        }
    }

    /// The slug to persist for a runtime effect change. `None` (Effect Off)
    /// persists the explicit off sentinel so it survives the default.
    pub fn persisted_slug(effect: Option<BackgroundEffect>) -> String {
        effect.map_or_else(
            || BACKGROUND_EFFECT_OFF_SLUG.to_string(),
            |e| e.slug().to_string(),
        )
    }
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
/// The two accent-derived colors handed to the background shader. Shared by
/// `background_effect_layer` and the design-contract exporter so HTML mockups
/// reproduce the exact shader palette.
pub fn background_effect_palette(
    theme: &crate::theme::Theme,
    effect: BackgroundEffect,
    intensity: f32,
) -> (Hsla, Hsla) {
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
    (color_a, tune(color_b))
}

pub fn background_effect_layer(
    theme: &crate::theme::Theme,
    effect: BackgroundEffect,
    intensity: f32,
    elapsed_secs: f32,
) -> Stateful<Div> {
    let (color_a, color_b) = background_effect_palette(theme, effect, intensity);
    let (focus, energy) = effect_focus_uniforms();

    div()
        .id("bg-effect-layer")
        .absolute()
        .inset_0()
        .bg(gpui::shader_effect(
            effect.shader_id(),
            elapsed_secs,
            focus,
            energy,
            color_a,
            color_b,
        ))
}

#[cfg(test)]
mod tests {
    use super::BackgroundEffect;

    /// One combined sequence test because the recorder is (intentionally)
    /// process-global state: seeding must not claim the focus target, a real
    /// move must retarget and re-arm the energy envelope, smoothing must walk
    /// toward the target, energy must swell gradually (never spike), and
    /// sub-epsilon jitter must not re-arm the envelope.
    #[test]
    fn focus_recorder_seeds_tracks_and_swells() {
        use super::{effect_focus_uniforms, note_effect_focus, EffectFocusSource};
        use std::sync::atomic::Ordering;

        // First report from a source only seeds its memory.
        note_effect_focus(EffectFocusSource::Mouse, 0.9, 0.9);
        let ([x0, _y0], _) = effect_focus_uniforms();
        assert!(
            (x0 - super::FOCUS_DEFAULT_X).abs() < 0.05,
            "seed report must not claim the focus target (x0={x0})"
        );

        // A real move claims the target and re-arms the energy envelope.
        let armed_before = super::LAST_CHANGE_MICROS.load(Ordering::Relaxed);
        std::thread::sleep(std::time::Duration::from_millis(5));
        note_effect_focus(EffectFocusSource::Mouse, 0.1, 0.2);
        let armed_after = super::LAST_CHANGE_MICROS.load(Ordering::Relaxed);
        assert!(
            armed_after > armed_before,
            "move must re-arm the energy envelope"
        );

        // Energy swells smoothly toward the change instead of spiking: each
        // sampled step moves at most a bounded fraction toward the target.
        let (_, e0) = effect_focus_uniforms();
        std::thread::sleep(std::time::Duration::from_millis(60));
        let ([x1, y1], e1) = effect_focus_uniforms();
        std::thread::sleep(std::time::Duration::from_millis(60));
        let ([x2, y2], e2) = effect_focus_uniforms();
        assert!(e1 >= e0 - 1e-4, "energy must not drop right after a change");
        assert!(
            (e1 - e0) < 0.5 && (e2 - e1) < 0.5,
            "energy must swell gradually, not spike (e0={e0} e1={e1} e2={e2})"
        );

        // Focus smoothing converges toward the new target.
        assert!((x1 - 0.1).abs() <= (x0 - 0.1).abs() + 1e-4);
        assert!((x2 - 0.1).abs() <= (x1 - 0.1).abs() + 1e-4);
        assert!((y2 - 0.2).abs() <= (y1 - 0.2).abs() + 1e-4);

        // Sub-epsilon jitter (paint re-reports, caret blink) is ignored.
        note_effect_focus(EffectFocusSource::Mouse, 0.101, 0.2);
        assert_eq!(
            super::LAST_CHANGE_MICROS.load(Ordering::Relaxed),
            armed_after,
            "jitter must not re-arm the energy envelope"
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
        let ids: Vec<_> = BackgroundEffect::all()
            .iter()
            .map(|e| e.shader_id())
            .collect();
        assert_eq!(ids, (1..=31).collect::<Vec<_>>());
    }

    /// A fresh install (no persisted preference) ships with Starfield, the
    /// explicit "off" sentinel disables effects across relaunches, and the
    /// persisted slug for Effect Off must be that sentinel — `None` is
    /// skipped on serialize, so it can never mean "off" once a default
    /// exists.
    #[test]
    fn background_pref_resolution_defaults_and_off_round_trip() {
        use super::{BACKGROUND_EFFECT_OFF_SLUG, DEFAULT_BACKGROUND_EFFECT};

        assert_eq!(
            BackgroundEffect::resolve_pref(None),
            Some(DEFAULT_BACKGROUND_EFFECT)
        );
        for off in ["off", "Off", "OFF", "none", "None", "", "  "] {
            assert_eq!(BackgroundEffect::resolve_pref(Some(off)), None, "{off:?}");
        }
        assert_eq!(
            BackgroundEffect::resolve_pref(Some("aurora")),
            Some(BackgroundEffect::Aurora)
        );
        assert_eq!(BackgroundEffect::resolve_pref(Some("not-a-shader")), None);

        // Runtime "Effect Off" persists the sentinel; the sentinel resolves
        // back to off after a relaunch.
        let persisted = BackgroundEffect::persisted_slug(None);
        assert_eq!(persisted, BACKGROUND_EFFECT_OFF_SLUG);
        assert_eq!(BackgroundEffect::resolve_pref(Some(&persisted)), None);
        assert_eq!(
            BackgroundEffect::persisted_slug(Some(BackgroundEffect::Starfield)),
            "starfield"
        );
    }
}
