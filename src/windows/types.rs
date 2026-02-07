//! Shared, platform-agnostic windowing data contracts.
//!
//! Keeping these types outside `platform` lets state and manager modules depend
//! on stable geometry data without introducing platform back-edges.

/// Display bounds in global top-left coordinate space (Y increases downward).
#[derive(Debug, Clone)]
pub struct DisplayBounds {
    pub origin_x: f64,
    pub origin_y: f64,
    pub width: f64,
    pub height: f64,
}
