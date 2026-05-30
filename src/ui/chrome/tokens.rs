pub use crate::window_resize::main_layout::{
    DIVIDER_HEIGHT, HEADER_PADDING_X, HEADER_PADDING_Y, HINT_STRIP_HEIGHT, HINT_STRIP_PADDING_X,
    HINT_STRIP_PADDING_Y, HINT_TEXT_OPACITY,
};

pub const DIVIDER_OPACITY: f32 = crate::theme::opacity::OPACITY_HOVER;

pub const LIQUID_GLASS_MIN_HIT_PX: f32 = 28.0;
pub const LIQUID_GLASS_MIN_VISUAL_PX: f32 = 20.0;
pub const LIQUID_GLASS_PREFERRED_HIT_PX: f32 = 44.0;
pub const LIQUID_GLASS_PREFERRED_CENTER_GAP_PX: f32 = 60.0;

pub const LIQUID_GLASS_WINDOW_RADIUS_PX: f32 = 22.0;
/// Radius for the Tahoe-style floating popup shell (e.g. the actions dialog).
pub const LIQUID_GLASS_POPUP_RADIUS_PX: f32 = 18.0;
pub const LIQUID_GLASS_PANEL_RADIUS_PX: f32 = 16.0;
pub const LIQUID_GLASS_CONTROL_RADIUS_PX: f32 = 14.0;
pub const LIQUID_GLASS_COMPACT_RADIUS_PX: f32 = 10.0;

/// Canonical interior padding for a Liquid Glass panel surface.
pub const LIQUID_GLASS_PANEL_PADDING_PX: f32 = 16.0;
/// Canonical dense gap between stacked controls/labels inside chrome.
pub const LIQUID_GLASS_DENSE_GAP_PX: f32 = 8.0;

/// Internal horizontal text inset for the main search input, matching the
/// MEASURED native regular `NSTextField` content inset on macOS 26.5 (9pt;
/// see `artifacts/liquid-glass/receipts/tahoe-native-baseline.json` and the
/// `control.searchField.textInset.horizontal` guideline metric). Before this
/// token the search text rendered flush (0pt) — the proven "input lacks
/// padding" gap (Oracle-Session tahoe-apple-guideline-metrics, slice 5).
pub const SEARCH_INPUT_TEXT_INSET_X_PX: f32 = 9.0;

pub const CHROME_LAYER_CONTENT: &str = "content";
pub const CHROME_LAYER_FUNCTIONAL: &str = "functionalChrome";
pub const CHROME_LAYER_FLOATING: &str = "floatingTransient";
pub const CHROME_LAYER_WINDOW_BACKDROP: &str = "windowBackdrop";

pub const MATERIAL_SOLID_THEME_TOKEN: &str = "solidThemeToken";
pub const MATERIAL_NS_VISUAL_EFFECT: &str = "NSVisualEffectView";
pub const MATERIAL_NATIVE_WINDOW_BACKDROP: &str = "nativeWindowBackdrop";

pub fn alpha_from_opacity(opacity: f32) -> u32 {
    (opacity.clamp(0.0, 1.0) * 255.0).round() as u32
}
