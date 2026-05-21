pub use crate::window_resize::mini_layout::{
    DIVIDER_HEIGHT, HEADER_PADDING_X, HEADER_PADDING_Y, HINT_STRIP_HEIGHT, HINT_STRIP_PADDING_X,
    HINT_STRIP_PADDING_Y, HINT_TEXT_OPACITY,
};

pub const DIVIDER_OPACITY: f32 = crate::theme::opacity::OPACITY_HOVER;

pub fn alpha_from_opacity(opacity: f32) -> u32 {
    (opacity.clamp(0.0, 1.0) * 255.0).round() as u32
}
