//! Window entry motion. Every window keeps its settled frame and fades in:
//! the glass material, the content, and the NSWindow alpha rise together over
//! one short duration. The selected preset is a single atomic Copy value; an
//! entry captures its spec once instead of consulting config.ts mid-flight.

use serde::{Deserialize, Deserializer, Serialize};
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize)]
#[repr(u8)]
#[serde(rename_all = "snake_case")]
pub enum MainWindowAnimationPreset {
    FixedFrame,
    #[default]
    SharpPresence,
}

/// Retired preset ids (the width-travel catalog and `previous_calibration`)
/// load as the default instead of rejecting the whole config.
impl<'de> Deserialize<'de> for MainWindowAnimationPreset {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let id = String::deserialize(deserializer)?;
        Ok(Self::from_id(&id).unwrap_or_default())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MainWindowAnimationSpec {
    /// Material, content, window alpha, and defocus all resolve over this.
    pub duration_ms: u32,
    /// NSWindow alpha multiplies every pixel, so below 0.85 the desktop
    /// shows through; the fade itself lives in the material and content.
    pub window_start_alpha: f64,
    pub material_start_alpha: f64,
    pub content_start_alpha: f64,
    pub blur_radius: f64,
    pub foreground_blur_radius: f64,
}

impl MainWindowAnimationSpec {
    const fn new(
        duration_ms: u32,
        window_start_alpha: f64,
        material_start_alpha: f64,
        content_start_alpha: f64,
        blur_radius: f64,
        foreground_blur_radius: f64,
    ) -> Self {
        Self {
            duration_ms,
            window_start_alpha,
            material_start_alpha,
            content_start_alpha,
            blur_radius,
            foreground_blur_radius,
        }
    }
}

impl MainWindowAnimationPreset {
    pub const ALL: [Self; 2] = [Self::FixedFrame, Self::SharpPresence];

    pub const fn number(self) -> usize {
        self as usize + 1
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|preset| preset.id() == id)
    }

    /// Stable snake_case persistence and catalog identity.
    pub const fn id(self) -> &'static str {
        match self {
            Self::FixedFrame => "fixed_frame",
            Self::SharpPresence => "sharp_presence",
        }
    }

    /// The canonical launcher/triggerBuiltin ID, independent of display text.
    pub const fn command_id(self) -> &'static str {
        match self {
            Self::FixedFrame => "builtin/animation-fixed-frame",
            Self::SharpPresence => "builtin/animation-sharp-presence",
        }
    }

    pub const fn trigger_aliases(self) -> &'static [&'static str] {
        match self {
            Self::FixedFrame => &["animation-fixed-frame"],
            Self::SharpPresence => &["animation-sharp-presence"],
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::FixedFrame => "Soft Focus Fade",
            Self::SharpPresence => "Sharp Presence",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::FixedFrame => "Slower fade that resolves out of a soft focus",
            Self::SharpPresence => "Fast, sharp fade; mostly visible from the first frame",
        }
    }

    pub const fn spec(self) -> MainWindowAnimationSpec {
        use MainWindowAnimationSpec as S;
        match self {
            Self::FixedFrame => S::new(140, 0.85, 0.15, 0.15, 12.0, 4.0),
            Self::SharpPresence => S::new(90, 0.99, 0.8, 0.85, 0.0, 0.0),
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
            "scope": "window_entry",
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

#[cfg(test)]
mod tests {
    use super::MainWindowAnimationPreset;

    #[test]
    fn retired_preset_ids_load_as_the_default() {
        for retired in ["\"coherent\"", "\"grow_in\"", "\"previous_calibration\""] {
            let preset: MainWindowAnimationPreset =
                serde_json::from_str(retired).expect("retired id still parses");
            assert_eq!(preset, MainWindowAnimationPreset::default());
        }
        for preset in MainWindowAnimationPreset::ALL {
            let json = serde_json::to_string(&preset).expect("serialize preset");
            let parsed: MainWindowAnimationPreset =
                serde_json::from_str(&json).expect("round trip");
            assert_eq!(parsed, preset);
            assert_eq!(MainWindowAnimationPreset::ALL[preset as usize], preset);
        }
    }
}
