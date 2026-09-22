//! Main-window entry experiments. The selected preset is a single atomic Copy
//! value; an entry captures its spec once instead of consulting config.ts mid-flight.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[repr(u8)]
#[serde(rename_all = "snake_case")]
pub enum MainWindowAnimationPreset {
    Coherent,
    Brisk,
    Swift,
    Leisurely,
    Cinematic,
    FixedFrame,
    GrowIn,
    Shallow,
    Bounce,
    DeepSquish,
    Present,
    SoftFade,
    LateContent,
    EarlyContent,
    Luminous,
    Razor,
    SoftFocus,
    Diffuse,
    ForegroundFocus,
    MaterialHaze,
    PreviousCalibration,
    QuickGrow,
    VelvetBounce,
    #[default]
    SharpPresence,
    DreamyGlide,
    BriskMaterialHaze,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MainWindowAnimationSpec {
    pub legacy: bool,
    pub duration_ms: u32,
    pub compression_ms: u32,
    pub start_width_scale: f64,
    pub squish_width_scale: f64,
    pub reveal_ms: u32,
    pub material_start_alpha: f64,
    pub content_start_alpha: f64,
    pub window_start_alpha: f64,
    pub blur_radius: f64,
    pub foreground_blur_radius: f64,
}

impl MainWindowAnimationSpec {
    const fn new(
        duration_ms: u32,
        compression_ms: u32,
        start_width_scale: f64,
        squish_width_scale: f64,
        reveal_ms: u32,
        material_start_alpha: f64,
        content_start_alpha: f64,
        window_start_alpha: f64,
        blur_radius: f64,
        foreground_blur_radius: f64,
    ) -> Self {
        Self {
            legacy: false,
            duration_ms,
            compression_ms,
            start_width_scale,
            squish_width_scale,
            reveal_ms,
            material_start_alpha,
            content_start_alpha,
            window_start_alpha,
            blur_radius,
            foreground_blur_radius,
        }
    }
}

impl MainWindowAnimationPreset {
    pub const ALL: [Self; 26] = [
        Self::Coherent,
        Self::Brisk,
        Self::Swift,
        Self::Leisurely,
        Self::Cinematic,
        Self::FixedFrame,
        Self::GrowIn,
        Self::Shallow,
        Self::Bounce,
        Self::DeepSquish,
        Self::Present,
        Self::SoftFade,
        Self::LateContent,
        Self::EarlyContent,
        Self::Luminous,
        Self::Razor,
        Self::SoftFocus,
        Self::Diffuse,
        Self::ForegroundFocus,
        Self::MaterialHaze,
        Self::PreviousCalibration,
        Self::QuickGrow,
        Self::VelvetBounce,
        Self::SharpPresence,
        Self::DreamyGlide,
        Self::BriskMaterialHaze,
    ];

    pub const fn number(self) -> usize {
        self as usize + 1
    }

    /// Stable snake_case persistence and catalog identity.
    pub const fn id(self) -> &'static str {
        match self {
            Self::Coherent => "coherent",
            Self::Brisk => "brisk",
            Self::Swift => "swift",
            Self::Leisurely => "leisurely",
            Self::Cinematic => "cinematic",
            Self::FixedFrame => "fixed_frame",
            Self::GrowIn => "grow_in",
            Self::Shallow => "shallow",
            Self::Bounce => "bounce",
            Self::DeepSquish => "deep_squish",
            Self::Present => "present",
            Self::SoftFade => "soft_fade",
            Self::LateContent => "late_content",
            Self::EarlyContent => "early_content",
            Self::Luminous => "luminous",
            Self::Razor => "razor",
            Self::SoftFocus => "soft_focus",
            Self::Diffuse => "diffuse",
            Self::ForegroundFocus => "foreground_focus",
            Self::MaterialHaze => "material_haze",
            Self::PreviousCalibration => "previous_calibration",
            Self::QuickGrow => "quick_grow",
            Self::VelvetBounce => "velvet_bounce",
            Self::SharpPresence => "sharp_presence",
            Self::DreamyGlide => "dreamy_glide",
            Self::BriskMaterialHaze => "brisk_material_haze",
        }
    }

    /// The canonical launcher/triggerBuiltin ID, independent of display text.
    pub const fn command_id(self) -> &'static str {
        match self {
            Self::Coherent => "builtin/animation-coherent",
            Self::Brisk => "builtin/animation-brisk",
            Self::Swift => "builtin/animation-swift",
            Self::Leisurely => "builtin/animation-leisurely",
            Self::Cinematic => "builtin/animation-cinematic",
            Self::FixedFrame => "builtin/animation-fixed-frame",
            Self::GrowIn => "builtin/animation-grow-in",
            Self::Shallow => "builtin/animation-shallow",
            Self::Bounce => "builtin/animation-bounce",
            Self::DeepSquish => "builtin/animation-deep-squish",
            Self::Present => "builtin/animation-present",
            Self::SoftFade => "builtin/animation-soft-fade",
            Self::LateContent => "builtin/animation-late-content",
            Self::EarlyContent => "builtin/animation-early-content",
            Self::Luminous => "builtin/animation-luminous",
            Self::Razor => "builtin/animation-razor",
            Self::SoftFocus => "builtin/animation-soft-focus",
            Self::Diffuse => "builtin/animation-diffuse",
            Self::ForegroundFocus => "builtin/animation-foreground-focus",
            Self::MaterialHaze => "builtin/animation-material-haze",
            Self::PreviousCalibration => "builtin/animation-previous-calibration",
            Self::QuickGrow => "builtin/animation-quick-grow",
            Self::VelvetBounce => "builtin/animation-velvet-bounce",
            Self::SharpPresence => "builtin/animation-sharp-presence",
            Self::DreamyGlide => "builtin/animation-dreamy-glide",
            Self::BriskMaterialHaze => "builtin/animation-brisk-material-haze",
        }
    }

    pub const fn trigger_aliases(self) -> &'static [&'static str] {
        match self {
            Self::Coherent => &["animation-coherent"],
            Self::Brisk => &["animation-brisk"],
            Self::Swift => &["animation-swift"],
            Self::Leisurely => &["animation-leisurely"],
            Self::Cinematic => &["animation-cinematic"],
            Self::FixedFrame => &["animation-fixed-frame"],
            Self::GrowIn => &["animation-grow-in"],
            Self::Shallow => &["animation-shallow"],
            Self::Bounce => &["animation-bounce"],
            Self::DeepSquish => &["animation-deep-squish"],
            Self::Present => &["animation-present"],
            Self::SoftFade => &["animation-soft-fade"],
            Self::LateContent => &["animation-late-content"],
            Self::EarlyContent => &["animation-early-content"],
            Self::Luminous => &["animation-luminous"],
            Self::Razor => &["animation-razor"],
            Self::SoftFocus => &["animation-soft-focus"],
            Self::Diffuse => &["animation-diffuse"],
            Self::ForegroundFocus => &["animation-foreground-focus"],
            Self::MaterialHaze => &["animation-material-haze"],
            Self::PreviousCalibration => &["animation-previous-calibration"],
            Self::QuickGrow => &["animation-quick-grow"],
            Self::VelvetBounce => &["animation-velvet-bounce"],
            Self::SharpPresence => &["animation-sharp-presence"],
            Self::DreamyGlide => &["animation-dreamy-glide"],
            Self::BriskMaterialHaze => &["animation-brisk-material-haze"],
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Coherent => "Coherent",
            Self::Brisk => "Brisk",
            Self::Swift => "Swift",
            Self::Leisurely => "Leisurely",
            Self::Cinematic => "Cinematic",
            Self::FixedFrame => "Fixed Frame",
            Self::GrowIn => "Grow In",
            Self::Shallow => "Shallow",
            Self::Bounce => "Bounce",
            Self::DeepSquish => "Deep Squish",
            Self::Present => "Present",
            Self::SoftFade => "Soft Fade",
            Self::LateContent => "Late Content",
            Self::EarlyContent => "Early Content",
            Self::Luminous => "Luminous",
            Self::Razor => "Razor",
            Self::SoftFocus => "Soft Focus",
            Self::Diffuse => "Diffuse",
            Self::ForegroundFocus => "Foreground Focus",
            Self::MaterialHaze => "Material Haze",
            Self::PreviousCalibration => "Previous Calibration",
            Self::QuickGrow => "Quick Grow",
            Self::VelvetBounce => "Velvet Bounce",
            Self::SharpPresence => "Sharp Presence",
            Self::DreamyGlide => "Dreamy Glide",
            Self::BriskMaterialHaze => "Brisk + Material Haze",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::Coherent => {
                "Uninterrupted compression with overlapping material and content reveal"
            }
            Self::Brisk => "Quick entry with a short reveal and snap-back",
            Self::Swift => "Accelerated entry without sacrificing the material transition",
            Self::Leisurely => "Longer, relaxed material and content entry",
            Self::Cinematic => "Slow, deliberate wide-to-narrow entrance",
            Self::FixedFrame => "No width motion: fade and focus alone",
            Self::GrowIn => "Start narrow and grow toward the settled width",
            Self::Shallow => "Almost stationary width with a barely visible squish",
            Self::Bounce => "Pronounced wide start and deeper rebound",
            Self::DeepSquish => "More emphatic width compression before returning to rest",
            Self::Present => "Mostly present from the outset, with restrained reveals",
            Self::SoftFade => "Dimmer material and content for a stronger arrival",
            Self::LateContent => {
                "Material establishes first; content joins near the compression end"
            }
            Self::EarlyContent => "Content is already legible as the frame forms",
            Self::Luminous => "Strong frame presence against a slow material reveal",
            Self::Razor => "No defocus: crisp material and foreground throughout",
            Self::SoftFocus => "Small focus change with a lightly blurred foreground",
            Self::Diffuse => "Strong material defocus that resolves on reveal",
            Self::ForegroundFocus => "Sharper material, foreground-focused reveal",
            Self::MaterialHaze => "More material haze, lighter foreground defocus",
            Self::PreviousCalibration => "Exact prior 298 ms main-entry behavior (rollback)",
            Self::QuickGrow => "Quick narrow-to-wide arrival with a clear foreground",
            Self::VelvetBounce => "Deep rebound, diffuse material and slow soft fade",
            Self::SharpPresence => "Stationary, sharp and mostly visible from the start",
            Self::DreamyGlide => "Unhurried broad glide with layered focus and fade",
            Self::BriskMaterialHaze => "Brisk compression and reveal with Material Haze defocus",
        }
    }

    pub const fn spec(self) -> MainWindowAnimationSpec {
        use MainWindowAnimationSpec as S;
        match self {
            Self::Coherent => S::new(298, 158, 1.0305, 0.987, 140, 0.15, 0.15, 0.85, 12.0, 4.0),
            Self::Brisk => S::new(165, 90, 1.021, 0.991, 77, 0.32, 0.35, 0.91, 8.0, 2.0),
            Self::Swift => S::new(225, 120, 1.0305, 0.987, 105, 0.15, 0.15, 0.85, 12.0, 4.0),
            Self::Leisurely => S::new(425, 235, 1.0305, 0.987, 205, 0.15, 0.15, 0.85, 12.0, 4.0),
            Self::Cinematic => S::new(620, 335, 1.039, 0.981, 290, 0.12, 0.12, 0.85, 14.0, 5.0),
            Self::FixedFrame => S::new(298, 158, 1.0, 1.0, 140, 0.15, 0.15, 0.85, 12.0, 4.0),
            Self::GrowIn => S::new(310, 164, 0.964, 1.013, 145, 0.15, 0.15, 0.85, 12.0, 4.0),
            Self::Shallow => S::new(298, 158, 1.009, 0.997, 140, 0.15, 0.15, 0.85, 12.0, 4.0),
            Self::Bounce => S::new(355, 185, 1.063, 0.965, 160, 0.15, 0.15, 0.85, 12.0, 4.0),
            Self::DeepSquish => S::new(335, 175, 1.041, 0.953, 152, 0.15, 0.15, 0.85, 12.0, 4.0),
            Self::Present => S::new(298, 158, 1.0305, 0.987, 110, 0.78, 0.78, 0.98, 12.0, 4.0),
            Self::SoftFade => S::new(298, 158, 1.0305, 0.987, 155, 0.06, 0.06, 0.85, 12.0, 4.0),
            Self::LateContent => S::new(298, 158, 1.0305, 0.987, 155, 0.46, 0.05, 0.87, 12.0, 4.0),
            Self::EarlyContent => S::new(298, 158, 1.0305, 0.987, 105, 0.12, 0.68, 0.87, 12.0, 4.0),
            Self::Luminous => S::new(345, 185, 1.0305, 0.987, 170, 0.05, 0.26, 0.99, 12.0, 4.0),
            Self::Razor => S::new(298, 158, 1.0305, 0.987, 140, 0.15, 0.15, 0.85, 0.0, 0.0),
            Self::SoftFocus => S::new(298, 158, 1.0305, 0.987, 140, 0.15, 0.15, 0.85, 4.0, 1.5),
            Self::Diffuse => S::new(298, 158, 1.0305, 0.987, 140, 0.15, 0.15, 0.85, 22.0, 6.0),
            Self::ForegroundFocus => {
                S::new(298, 158, 1.0305, 0.987, 140, 0.15, 0.15, 0.85, 5.0, 8.0)
            }
            Self::MaterialHaze => S::new(298, 158, 1.0305, 0.987, 140, 0.15, 0.15, 0.85, 26.0, 2.0),
            Self::PreviousCalibration => MainWindowAnimationSpec {
                legacy: true,
                ..S::new(298, 158, 1.0305, 0.987, 88, 1.0, 0.21, 0.85, 12.0, 0.0)
            },
            Self::QuickGrow => S::new(190, 105, 0.957, 0.982, 87, 0.28, 0.34, 0.9, 4.0, 1.0),
            Self::VelvetBounce => S::new(510, 265, 1.067, 0.952, 250, 0.07, 0.08, 0.85, 21.0, 7.0),
            Self::SharpPresence => S::new(235, 115, 1.0, 1.0, 90, 0.8, 0.85, 0.99, 0.0, 0.0),
            Self::DreamyGlide => S::new(560, 310, 1.056, 0.978, 275, 0.07, 0.11, 0.86, 23.0, 7.0),
            Self::BriskMaterialHaze => {
                S::new(165, 90, 1.021, 0.991, 77, 0.15, 0.15, 0.85, 26.0, 2.0)
            }
        }
    }
    /// Catalog and selected entry derived from this instance's own preference.
    pub fn automation_snapshot(self) -> serde_json::Value {
        let presets: Vec<_> = Self::ALL
            .iter()
            .map(|&preset| {
                serde_json::json!({
                    "id": preset.id(),
                    "name": preset.name(),
                    "number": preset.number(),
                    "description": preset.description(),
                    "commandId": preset.command_id(),
                    "spec": preset.spec(),
                })
            })
            .collect();
        serde_json::json!({
            "scope": "main_window_entry",
            "selectedId": self.id(),
            "selectedName": self.name(),
            "selectedNumber": self.number(),
            "selectedSpec": self.spec(),
            "presets": presets,
        })
    }
}
static CURRENT_PRESET: AtomicU8 = AtomicU8::new(MainWindowAnimationPreset::SharpPresence as u8);

pub fn current_preset() -> MainWindowAnimationPreset {
    MainWindowAnimationPreset::ALL[CURRENT_PRESET.load(Ordering::Relaxed) as usize]
}

pub fn set_current_preset(preset: MainWindowAnimationPreset) {
    CURRENT_PRESET.store(preset as u8, Ordering::Relaxed);
}
