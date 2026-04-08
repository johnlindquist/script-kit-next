/// Anchor point for an overlay within the integrated surface shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegratedOverlayAnchor {
    /// Overlay anchored relative to a composer / input region.
    Composer,
    /// Overlay anchored relative to the footer region.
    Footer,
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

/// Configuration for the integrated surface shell scene sizing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IntegratedSurfaceShellConfig {
    pub width: f32,
    pub height: f32,
    pub corner_radius: f32,
    pub body_padding: f32,
    pub footer_height: f32,
}

impl Default for IntegratedSurfaceShellConfig {
    fn default() -> Self {
        Self {
            width: 560.0,
            height: 320.0,
            corner_radius: 12.0,
            body_padding: 16.0,
            footer_height: 36.0,
        }
    }
}
