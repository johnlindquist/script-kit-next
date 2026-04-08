use gpui::{
    div, point, px, App, BoxShadow, IntoElement, ParentElement, RenderOnce, Styled, Window,
};

use crate::theme::{get_cached_theme, AppChromeColors};
use crate::ui_foundation::HexColorExt;

use super::component::IntegratedSurfaceShell;
use super::types::{IntegratedOverlayAnchor, IntegratedOverlayState, IntegratedSurfaceShellConfig};

impl RenderOnce for IntegratedSurfaceShell {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = get_cached_theme();
        let chrome = AppChromeColors::from_theme(&theme);

        let overlay_state = self
            .overlay
            .as_ref()
            .map(|(_, state, _)| state.as_str())
            .unwrap_or("none");
        let overlay_anchor = self
            .overlay
            .as_ref()
            .map(|(placement, _, _)| match placement.anchor {
                IntegratedOverlayAnchor::Composer => "composer",
                IntegratedOverlayAnchor::Footer => "footer",
            })
            .unwrap_or("none");

        tracing::info!(
            event = "integrated_surface_shell_rendered",
            width = self.config.width,
            height = self.config.height,
            has_footer = self.footer.is_some(),
            has_overlay = self.overlay.is_some(),
            overlay_state = overlay_state,
            overlay_anchor = overlay_anchor,
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
            }])
            .child(
                div()
                    .absolute()
                    .left(px(0.0))
                    .top(px(0.0))
                    .w_full()
                    .h(px(1.0))
                    .bg(theme.colors.text.primary.with_opacity(0.06)),
            );

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

        // Overlay popup with state-driven choreography
        if let Some((placement, state, overlay)) = self.overlay {
            let scrim_alpha = overlay_scrim_alpha(&self.config, state);
            let lift = overlay_lift(&self.config, state);

            // Scrim layer
            if scrim_alpha > 0.0 {
                root = root.child(
                    div()
                        .absolute()
                        .left(px(0.0))
                        .top(px(0.0))
                        .size_full()
                        .bg(theme.colors.background.title_bar.with_opacity(scrim_alpha)),
                );
            }

            // Overlay with lift, shadow, and bridge
            root = root.child(
                div()
                    .absolute()
                    .left(px(placement.left))
                    .top(px((placement.top - lift).max(0.0)))
                    .w(px(placement.width))
                    .relative()
                    .shadow(vec![BoxShadow {
                        color: overlay_shadow_color(&theme, state),
                        offset: point(px(0.0), px(self.config.overlay_shadow_y + lift)),
                        blur_radius: px(self.config.overlay_shadow_blur + (lift * 4.0)),
                        spread_radius: px(0.0),
                    }])
                    .child(overlay)
                    .child(render_overlay_bridge(
                        &theme,
                        &self.config,
                        placement.anchor,
                        state,
                        placement.width,
                    )),
            );
        }

        root
    }
}

fn overlay_lift(config: &IntegratedSurfaceShellConfig, state: IntegratedOverlayState) -> f32 {
    match state {
        IntegratedOverlayState::Resting => 0.0,
        IntegratedOverlayState::Focused => config.overlay_lift,
        IntegratedOverlayState::Loading => config.overlay_lift + 2.0,
        IntegratedOverlayState::Empty => 2.0,
        IntegratedOverlayState::Error | IntegratedOverlayState::Danger => config.overlay_lift + 1.0,
    }
}

fn overlay_scrim_alpha(
    config: &IntegratedSurfaceShellConfig,
    state: IntegratedOverlayState,
) -> f32 {
    match state {
        IntegratedOverlayState::Resting => 0.0,
        IntegratedOverlayState::Focused => config.scrim_alpha,
        IntegratedOverlayState::Loading => (config.scrim_alpha + 0.01).min(0.12),
        IntegratedOverlayState::Empty => config.scrim_alpha * 0.5,
        IntegratedOverlayState::Error | IntegratedOverlayState::Danger => {
            (config.scrim_alpha + 0.02).min(0.12)
        }
    }
}

fn overlay_shadow_color(theme: &crate::theme::Theme, state: IntegratedOverlayState) -> gpui::Hsla {
    match state {
        IntegratedOverlayState::Danger | IntegratedOverlayState::Error => {
            theme.colors.ui.error.with_opacity(0.18)
        }
        IntegratedOverlayState::Focused => theme.colors.accent.selected.with_opacity(0.14),
        IntegratedOverlayState::Loading => theme.colors.ui.border.with_opacity(0.18),
        IntegratedOverlayState::Empty => theme.colors.ui.border.with_opacity(0.10),
        IntegratedOverlayState::Resting => theme.colors.ui.border.with_opacity(0.12),
    }
}

fn render_overlay_bridge(
    theme: &crate::theme::Theme,
    config: &IntegratedSurfaceShellConfig,
    anchor: IntegratedOverlayAnchor,
    state: IntegratedOverlayState,
    overlay_width: f32,
) -> gpui::Div {
    let bridge_left = ((overlay_width - config.overlay_bridge_width).max(0.0)) / 2.0;

    let bridge_color = match state {
        IntegratedOverlayState::Danger | IntegratedOverlayState::Error => {
            theme.colors.ui.error.with_opacity(0.16)
        }
        _ => theme.colors.ui.border.with_opacity(0.22),
    };

    let bridge = div()
        .absolute()
        .left(px(bridge_left))
        .w(px(config.overlay_bridge_width))
        .h(px(config.overlay_bridge_height))
        .rounded(px(config.overlay_bridge_height / 2.0))
        .bg(bridge_color);

    match anchor {
        IntegratedOverlayAnchor::Composer => bridge.top(px(-(config.overlay_bridge_height + 4.0))),
        IntegratedOverlayAnchor::Footer => bridge.bottom(px(-(config.overlay_bridge_height + 4.0))),
    }
}
