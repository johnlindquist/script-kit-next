use gpui::{
    div, point, px, App, BoxShadow, IntoElement, ParentElement, RenderOnce, Styled, Window,
};

use crate::theme::{get_cached_theme, AppChromeColors};
use crate::ui_foundation::HexColorExt;

use super::component::IntegratedSurfaceShell;

impl RenderOnce for IntegratedSurfaceShell {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = get_cached_theme();
        let chrome = AppChromeColors::from_theme(&theme);

        tracing::info!(
            event = "integrated_surface_shell_rendered",
            width = self.config.width,
            height = self.config.height,
            has_footer = self.footer.is_some(),
            has_overlay = self.overlay.is_some(),
            "Rendered integrated playground shell"
        );

        let footer_height = if self.footer.is_some() {
            self.config.footer_height
        } else {
            0.0
        };

        let mut root = div()
            .w(px(self.config.width))
            .h(px(self.config.height))
            .relative()
            .rounded(px(self.config.corner_radius))
            .overflow_hidden()
            .border_1()
            .border_color(theme.colors.ui.border.with_opacity(0.30))
            .bg(gpui::rgba(chrome.window_surface_rgba))
            .shadow(vec![BoxShadow {
                color: theme.colors.ui.border.with_opacity(0.16),
                offset: point(px(0.0), px(10.0)),
                blur_radius: px(24.0),
                spread_radius: px(0.0),
            }]);

        // Body region fills remaining space above footer
        root = root.child(
            div()
                .w_full()
                .h(px(self.config.height - footer_height))
                .px(px(self.config.body_padding))
                .py(px(self.config.body_padding))
                .child(self.body),
        );

        // Footer docked at bottom
        if let Some(footer) = self.footer {
            root = root.child(
                div()
                    .absolute()
                    .left(px(0.0))
                    .bottom(px(0.0))
                    .w_full()
                    .h(px(self.config.footer_height))
                    .child(footer),
            );
        }

        // Overlay popup positioned absolutely
        if let Some((placement, overlay)) = self.overlay {
            root = root.child(
                div()
                    .absolute()
                    .left(px(placement.left))
                    .top(px(placement.top))
                    .w(px(placement.width))
                    .child(overlay),
            );
        }

        root
    }
}
