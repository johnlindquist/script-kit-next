pub use crate::window_resize::mini_layout::{
    DIVIDER_HEIGHT, HEADER_PADDING_X, HEADER_PADDING_Y, HINT_STRIP_HEIGHT, HINT_STRIP_PADDING_X,
    HINT_STRIP_PADDING_Y, HINT_TEXT_OPACITY,
};

pub const DIVIDER_OPACITY: f32 = crate::theme::opacity::OPACITY_HOVER;

/// Tahoe/Liquid Glass shared geometry and control metrics.
///
/// These tokens deliberately start with conservative Mac-launcher values:
/// shells and controls get softer, while dense content rows keep their scan
/// rhythm. Native AppKit glass can consume the same scale without each surface
/// inventing its own radii.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TahoeChromeMetrics {
    pub control_sm_radius: f32,
    pub control_md_radius: f32,
    pub control_lg_radius: f32,
    pub panel_radius: f32,
    pub popup_shell_radius: f32,
    pub prompt_surface_radius: f32,
    pub keycap_radius: f32,
    pub footer_inset_x: f32,
    pub footer_inset_y: f32,
    pub footer_shadow_blur: f32,
    pub button_height: f32,
    pub action_row_radius: f32,
    pub acp_toolbar_height: f32,
    pub acp_composer_min_height: f32,
}

impl TahoeChromeMetrics {
    pub const fn default() -> Self {
        Self {
            control_sm_radius: 8.0,
            control_md_radius: 12.0,
            control_lg_radius: 16.0,
            panel_radius: 18.0,
            popup_shell_radius: 18.0,
            prompt_surface_radius: 12.0,
            keycap_radius: 6.0,
            footer_inset_x: 8.0,
            footer_inset_y: 4.0,
            footer_shadow_blur: 14.0,
            button_height: 30.0,
            action_row_radius: 10.0,
            acp_toolbar_height: 36.0,
            acp_composer_min_height: 44.0,
        }
    }
}

pub const TAHOE_CHROME_METRICS: TahoeChromeMetrics = TahoeChromeMetrics::default();

pub fn alpha_from_opacity(opacity: f32) -> u32 {
    (opacity.clamp(0.0, 1.0) * 255.0).round() as u32
}
