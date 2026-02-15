//! Icon rendering component
//!
//! Provides IconView for rendering any IconRef with styling.
//! Uses type erasure (AnyElement) to unify vector and raster icons.

use gpui::{
    img, px, svg, AnyElement, App, Hsla, IntoElement, ObjectFit, RenderImage, SharedString, Styled,
    StyledImage, Window,
};
use std::sync::Arc;

use super::{ColorToken, EmbeddedIcon, IconColor, IconRef, IconSize, IconStyle};

/// Render an IconRef with the given style
///
/// Returns an AnyElement for type erasure, allowing uniform handling
/// of vector icons (SVG) and raster icons (images).
pub fn render_icon(
    icon: &IconRef,
    style: &IconStyle,
    theme_colors: &dyn ThemeColorProvider,
    window: &Window,
) -> AnyElement {
    let size_px = style.size.to_px();
    let color = resolve_color(&style.color, theme_colors, window);

    match icon {
        IconRef::Lucide(lucide_icon) => {
            render_lucide(lucide_icon.clone(), size_px, color, style.opacity)
        }
        IconRef::Embedded(embedded) => render_embedded(*embedded, size_px, color, style.opacity),
        IconRef::AssetSvg(path) => render_asset_svg(path, size_px, color, style.opacity),
        IconRef::SFSymbol(_) | IconRef::File(_) | IconRef::Url(_) | IconRef::AppBundle(_) => {
            // These require async loading or platform APIs
            // For now, render the fallback
            if let Some(fallback) = icon.fallback() {
                render_icon(&fallback, style, theme_colors, window)
            } else {
                render_placeholder(size_px)
            }
        }
    }
}

/// Trait for providing theme colors
///
/// This allows decoupling from the specific theme implementation
pub trait ThemeColorProvider {
    fn foreground(&self) -> Hsla;
    fn muted(&self) -> Hsla;
    fn accent(&self) -> Hsla;
    fn danger(&self) -> Hsla;
    fn success(&self) -> Hsla;
    fn warning(&self) -> Hsla;
}

/// Resolve an IconColor to an actual HSLA color
fn resolve_color(
    color: &IconColor,
    theme: &dyn ThemeColorProvider,
    window: &Window,
) -> Option<Hsla> {
    match color {
        IconColor::Inherit => Some(window.text_style().color),
        IconColor::Token(token) => Some(match token {
            ColorToken::Primary => theme.foreground(),
            ColorToken::Muted => theme.muted(),
            ColorToken::Accent => theme.accent(),
            ColorToken::Danger => theme.danger(),
            ColorToken::Success => theme.success(),
            ColorToken::Warning => theme.warning(),
        }),
        IconColor::Fixed(hsla) => Some(*hsla),
        IconColor::None => None,
    }
}

/// Render a Lucide icon from gpui_component
fn render_lucide(
    icon: gpui_component::IconName,
    size_px: f32,
    color: Option<Hsla>,
    opacity: f32,
) -> AnyElement {
    use gpui_component::IconNamed;

    let path = icon.path();

    let mut element = svg().path(path).size(px(size_px)).flex_shrink_0();

    if let Some(c) = color {
        element = element.text_color(c);
    }

    if (opacity - 1.0).abs() > f32::EPSILON {
        element = element.opacity(opacity);
    }

    element.into_any_element()
}

/// Render an embedded Script Kit icon
fn render_embedded(
    icon: EmbeddedIcon,
    size_px: f32,
    color: Option<Hsla>,
    opacity: f32,
) -> AnyElement {
    let path = icon.asset_path();

    let mut element = svg().path(path).size(px(size_px)).flex_shrink_0();

    if let Some(c) = color {
        element = element.text_color(c);
    }

    if (opacity - 1.0).abs() > f32::EPSILON {
        element = element.opacity(opacity);
    }

    element.into_any_element()
}

/// Render an SVG from the assets folder
fn render_asset_svg(
    path: &SharedString,
    size_px: f32,
    color: Option<Hsla>,
    opacity: f32,
) -> AnyElement {
    let mut element = svg().path(path.clone()).size(px(size_px)).flex_shrink_0();

    if let Some(c) = color {
        element = element.text_color(c);
    }

    if (opacity - 1.0).abs() > f32::EPSILON {
        element = element.opacity(opacity);
    }

    element.into_any_element()
}

/// Render a placeholder for icons that couldn't be loaded
fn render_placeholder(size_px: f32) -> AnyElement {
    // Just render an empty box of the right size
    gpui::div()
        .size(px(size_px))
        .flex_shrink_0()
        .into_any_element()
}

/// Render a pre-decoded image (for app icons, etc.)
pub fn render_image(image: Arc<RenderImage>, size_px: f32, opacity: f32) -> AnyElement {
    let image_clone = image.clone();

    let mut element = img(move |_window: &mut Window, _cx: &mut App| Some(Ok(image_clone.clone())))
        .size(px(size_px))
        .object_fit(ObjectFit::Contain)
        .flex_shrink_0();

    if (opacity - 1.0).abs() > f32::EPSILON {
        element = element.opacity(opacity);
    }

    element.into_any_element()
}

/// Builder for rendering icons with fluent API
pub struct IconView {
    icon: IconRef,
    style: IconStyle,
}

impl IconView {
    /// Create a new IconView
    pub fn new(icon: impl Into<IconRef>) -> Self {
        Self {
            icon: icon.into(),
            style: IconStyle::default(),
        }
    }

    /// Set the size
    pub fn size(mut self, size: IconSize) -> Self {
        self.style.size = size;
        self
    }

    /// Set the color
    pub fn color(mut self, color: IconColor) -> Self {
        self.style.color = color;
        self
    }

    /// Set the opacity
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.style.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Render using the provided theme
    pub fn render(self, theme: &dyn ThemeColorProvider, window: &Window) -> AnyElement {
        render_icon(&self.icon, &self.style, theme, window)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::rgb;

    const TEST_ICON_FOREGROUND: u32 = 0xffffff;
    const TEST_ICON_MUTED: u32 = 0x888888;
    const TEST_ICON_ACCENT: u32 = 0x0066ff;
    const TEST_ICON_DANGER: u32 = 0xff0000;
    const TEST_ICON_SUCCESS: u32 = 0x00ff00;
    const TEST_ICON_WARNING: u32 = 0xffaa00;

    /// Simple theme for testing
    #[allow(dead_code)]
    struct TestTheme;

    impl ThemeColorProvider for TestTheme {
        fn foreground(&self) -> Hsla {
            rgb(TEST_ICON_FOREGROUND).into()
        }
        fn muted(&self) -> Hsla {
            rgb(TEST_ICON_MUTED).into()
        }
        fn accent(&self) -> Hsla {
            rgb(TEST_ICON_ACCENT).into()
        }
        fn danger(&self) -> Hsla {
            rgb(TEST_ICON_DANGER).into()
        }
        fn success(&self) -> Hsla {
            rgb(TEST_ICON_SUCCESS).into()
        }
        fn warning(&self) -> Hsla {
            rgb(TEST_ICON_WARNING).into()
        }
    }

    #[test]
    fn icon_view_builder() {
        let view = IconView::new(gpui_component::IconName::Check)
            .size(IconSize::Large)
            .color(IconColor::Token(ColorToken::Accent))
            .opacity(0.8);

        assert_eq!(view.style.size, IconSize::Large);
        assert!((view.style.opacity - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn icon_view_from_embedded() {
        let view = IconView::new(EmbeddedIcon::Terminal).size(IconSize::Medium);

        assert_eq!(view.style.size, IconSize::Medium);
    }
}
