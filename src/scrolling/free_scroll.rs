use gpui::{ScrollDelta, ScrollWheelEvent};

pub(crate) const FREE_SCROLL_LINE_DELTA_PX: f32 = 72.0;
pub(crate) const FREE_SCROLL_PIXEL_MULTIPLIER: f32 = 1.35;

/// Normalize a wheel event to a signed pixel delta for scroll-direction
/// bookkeeping. Never apply this delta back to a `ListState` that lives
/// inside a gpui `list()` element — the list's own bubble-phase handler
/// already scrolls, and a second absolute offset writer oscillates against
/// it (the ChatPrompt scroll-jank bug).
pub(crate) fn normalized_vertical_delta_px(event: &ScrollWheelEvent) -> f32 {
    match event.delta {
        ScrollDelta::Lines(point) => point.y * FREE_SCROLL_LINE_DELTA_PX,
        ScrollDelta::Pixels(point) => {
            let pixels: f32 = point.y.into();
            pixels * FREE_SCROLL_PIXEL_MULTIPLIER
        }
    }
}
