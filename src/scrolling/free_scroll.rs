use gpui::{point, px, ScrollDelta, ScrollWheelEvent};
use gpui_component::scroll::ScrollbarHandle;

pub(crate) const FREE_SCROLL_LINE_DELTA_PX: f32 = 72.0;
pub(crate) const FREE_SCROLL_PIXEL_MULTIPLIER: f32 = 1.35;

pub(crate) fn normalized_vertical_delta_px(event: &ScrollWheelEvent) -> f32 {
    match event.delta {
        ScrollDelta::Lines(point) => point.y * FREE_SCROLL_LINE_DELTA_PX,
        ScrollDelta::Pixels(point) => {
            let pixels: f32 = point.y.into();
            pixels * FREE_SCROLL_PIXEL_MULTIPLIER
        }
    }
}

pub(crate) fn apply_vertical_wheel_scroll<H: ScrollbarHandle>(
    handle: &H,
    event: &ScrollWheelEvent,
) -> f32 {
    let delta_y = normalized_vertical_delta_px(event);
    let offset = handle.offset();
    handle.set_offset(point(offset.x, offset.y + px(delta_y)));
    delta_y
}
