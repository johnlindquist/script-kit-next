/// Anchor point for an overlay within the integrated surface shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegratedOverlayAnchor {
    /// Overlay anchored relative to a composer / input region.
    Composer,
    /// Overlay anchored relative to the footer region.
    Footer,
}

/// Discrete overlay states used by compare-mode playgrounds.
///
/// These are intentionally compare-friendly states, not runtime animation handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegratedOverlayState {
    Resting,
    Focused,
    Loading,
    Empty,
    Error,
    Danger,
}

impl IntegratedOverlayState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Resting => "resting",
            Self::Focused => "focused",
            Self::Loading => "loading",
            Self::Empty => "empty",
            Self::Error => "error",
            Self::Danger => "danger",
        }
    }
}

/// Positioning data for an overlay popup within the shell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IntegratedOverlayPlacement {
    pub anchor: IntegratedOverlayAnchor,
    pub left: f32,
    pub top: f32,
    pub width: f32,
}

impl IntegratedOverlayPlacement {
    pub fn new(anchor: IntegratedOverlayAnchor, left: f32, top: f32, width: f32) -> Self {
        Self {
            anchor,
            left,
            top,
            width,
        }
    }
}

/// Configuration for the integrated surface shell scene sizing and chrome polish.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IntegratedSurfaceShellConfig {
    pub width: f32,
    pub height: f32,
    pub corner_radius: f32,
    pub body_padding: f32,
    pub footer_height: f32,
    pub scrim_alpha: f32,
    pub overlay_lift: f32,
    pub overlay_shadow_y: f32,
    pub overlay_shadow_blur: f32,
    pub overlay_bridge_width: f32,
    pub overlay_bridge_height: f32,
}

impl Default for IntegratedSurfaceShellConfig {
    fn default() -> Self {
        Self {
            width: 560.0,
            height: 320.0,
            corner_radius: 12.0,
            body_padding: 16.0,
            footer_height: 36.0,
            scrim_alpha: 0.04,
            overlay_lift: 6.0,
            overlay_shadow_y: 12.0,
            overlay_shadow_blur: 28.0,
            overlay_bridge_width: 54.0,
            overlay_bridge_height: 8.0,
        }
    }
}
