//! Chrome specification for shell frame styling
//!
//! Controls the visual frame around content: background, shadow, radius, padding.

use gpui::{px, Pixels};

/// Chrome mode controls how much frame styling is applied
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ChromeMode {
    /// Full frame with rounded background, shadow, padding, header/footer, divider
    /// Used for: ScriptList, ArgPrompt with choices, EditorPrompt
    #[default]
    FullFrame,

    /// Minimal frame with rounded background and shadow, but tighter padding
    /// No divider. Used for: HUD notifications, compact prompts
    MinimalFrame,

    /// No background or shadow, just pass-through layout
    /// Used for: Overlays that provide their own styling
    ContentOnly,
}

impl ChromeMode {
    /// Check if this mode shows a header divider
    pub fn shows_divider(&self) -> bool {
        matches!(self, ChromeMode::FullFrame)
    }

    /// Check if this mode has background styling
    pub fn has_background(&self) -> bool {
        !matches!(self, ChromeMode::ContentOnly)
    }

    /// Check if this mode has shadow
    pub fn has_shadow(&self) -> bool {
        !matches!(self, ChromeMode::ContentOnly)
    }
}

/// Divider specification between header and content
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DividerSpec {
    /// No divider
    None,
    /// 1px hairline divider (most common)
    #[default]
    Hairline,
}

impl DividerSpec {
    /// Get the height in pixels
    pub fn height(&self) -> Pixels {
        match self {
            DividerSpec::None => px(0.0),
            DividerSpec::Hairline => px(1.0),
        }
    }
}

/// Full chrome specification for the shell frame
#[derive(Clone, Copy, Debug)]
pub struct ChromeSpec {
    /// Chrome mode (determines overall styling)
    pub mode: ChromeMode,
    /// Border radius in pixels (default: 12)
    pub border_radius: f32,
    /// Padding in pixels (default: 0, content handles own padding)
    pub padding: f32,
    /// Divider between header and content
    pub divider: DividerSpec,
    /// Background opacity (0.0 - 1.0, for vibrancy support)
    pub background_opacity: f32,
}

impl Default for ChromeSpec {
    fn default() -> Self {
        Self {
            mode: ChromeMode::FullFrame,
            border_radius: 12.0,
            padding: 0.0,
            divider: DividerSpec::Hairline,
            background_opacity: 0.85, // 85% opacity for vibrancy
        }
    }
}

impl ChromeSpec {
    /// Create a full frame chrome spec
    pub fn full_frame() -> Self {
        Self::default()
    }

    /// Create a minimal frame chrome spec (no divider, tighter styling)
    pub fn minimal() -> Self {
        Self {
            mode: ChromeMode::MinimalFrame,
            border_radius: 8.0,
            padding: 0.0,
            divider: DividerSpec::None,
            background_opacity: 0.80,
        }
    }

    /// Create a content-only chrome spec (no background/shadow)
    pub fn content_only() -> Self {
        Self {
            mode: ChromeMode::ContentOnly,
            border_radius: 0.0,
            padding: 0.0,
            divider: DividerSpec::None,
            background_opacity: 0.0,
        }
    }

    /// Set the border radius
    pub fn radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    /// Set the padding
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// Set the divider style
    pub fn divider(mut self, divider: DividerSpec) -> Self {
        self.divider = divider;
        self
    }

    /// Set background opacity (0.0 - 1.0)
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.background_opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Check if divider should be shown (based on mode and spec)
    pub fn should_show_divider(&self) -> bool {
        self.mode.shows_divider() && self.divider != DividerSpec::None
    }
}
