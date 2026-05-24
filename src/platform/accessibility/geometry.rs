#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectPx {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Default for RectPx {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DisplayBounds {
    pub visible: RectPx,
}

impl Default for DisplayBounds {
    fn default() -> Self {
        Self {
            visible: RectPx {
                x: 0.0,
                y: 0.0,
                width: 1440.0,
                height: 900.0,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FocusedFieldGeometry {
    pub caret_bounds: Option<RectPx>,
    pub selection_bounds: Option<RectPx>,
    pub field_bounds: Option<RectPx>,
    pub window_bounds: Option<RectPx>,
    pub display_bounds: DisplayBounds,
}

pub fn preferred_anchor_geometry(snapshot: &FocusedFieldGeometry) -> RectPx {
    snapshot
        .caret_bounds
        .or(snapshot.selection_bounds)
        .or(snapshot.field_bounds)
        .or(snapshot.window_bounds)
        .unwrap_or(snapshot.display_bounds.visible)
}
