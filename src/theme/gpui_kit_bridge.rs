//! gpui-kit (gpui-component 0.6) theme integration.
//!
//! Script Kit resolves its theme into gpui-component's `Theme` in
//! `gpui_integration.rs`. gpui-kit keeps a second, headless theme layer in
//! gpui-base (TextView defaults, scrollbars, resize handles) that only follows
//! the styled theme through `Theme::sync_base`, and gpui-kit's registry
//! observer re-applies its bundled theme (`Theme::change`) whenever the
//! registry global changes. This module owns both edges so every gpui-kit and
//! gpui-ai component renders the Script Kit palette.

use gpui::App;
use gpui_component::{Theme, ThemeRegistry};

/// Inner inset of multi-line editors (Notes, Day Page, editor prompt).
///
/// The Script Kit gpui-component fork applied `Size::Medium` input padding
/// (12px × 8px) to every `Input`, including code-editor mode. gpui-kit paints
/// multi-line editors with no inner padding, so surfaces that adopted
/// `Editor` apply this inset themselves to keep their text where it was.
pub(crate) const KIT_MULTILINE_EDITOR_INSET_X_PX: f32 = 12.0;
/// Vertical counterpart of [`KIT_MULTILINE_EDITOR_INSET_X_PX`].
pub(crate) const KIT_MULTILINE_EDITOR_INSET_Y_PX: f32 = 8.0;
/// Line box of multi-line editors, in rems. The fork's `Input` used
/// `Rems(1.25)` (20pt at the default 16px rem); gpui-kit's `Editor` defaults
/// to `1.5 ×` the font size, which would reflow Notes and the editor prompt.
pub(crate) const KIT_MULTILINE_EDITOR_LINE_HEIGHT_REMS: f32 = 1.25;

/// Mirror the freshly written styled theme into gpui-base.
pub(crate) fn sync_kit_theme_from_fork(cx: &mut App) {
    if cx.has_global::<Theme>() {
        Theme::sync_base(cx);
    }
}

/// Initialize every GPUI component layer Script Kit renders with (gpui-kit
/// and gpui-ai). Call instead of `gpui_component::init`.
pub fn init_component_libraries(cx: &mut App) {
    // gpui_ai::init also initializes gpui-component (gpui-kit).
    gpui_ai::init(cx);
    // gpui-kit re-applies its registry theme whenever the registry global
    // changes; put the Script Kit palette back afterwards. Registered after
    // gpui-kit's own observer, so it runs second.
    cx.observe_global::<ThemeRegistry>(|cx| {
        let theme = crate::theme::get_cached_theme();
        crate::theme::gpui_integration::PreparedComponentTheme::new(&theme).apply(cx);
    })
    .detach();
}
