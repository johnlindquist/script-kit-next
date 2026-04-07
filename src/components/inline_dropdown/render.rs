use gpui::{
    div, px, App, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, Window,
};

use super::component::InlineDropdown;
use super::row::{render_compact_synopsis_strip, MUTED_OP};

impl RenderOnce for InlineDropdown {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        tracing::info!(
            target: "ai",
            dropdown_id = %self.id,
            empty = self.empty_state.is_some(),
            has_synopsis = self.synopsis.is_some(),
            "inline_dropdown_rendered"
        );

        let mut surface = div()
            .id(self.id)
            .rounded(px(8.0))
            .overflow_hidden()
            .bg(gpui::rgba(self.colors.surface_rgba))
            .border_1()
            .border_color(gpui::rgba(self.colors.border_rgba))
            .py(px(self.vertical_padding))
            .flex()
            .flex_col();

        if let Some(empty_state) = self.empty_state {
            surface = surface.px(px(self.horizontal_padding)).gap(px(4.0)).child(
                div()
                    .text_xs()
                    .text_color(self.colors.muted_foreground.opacity(MUTED_OP))
                    .child(empty_state.message),
            );

            if !empty_state.hints.is_empty() {
                surface = surface.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(4.0))
                        .children(empty_state.hints),
                );
            }
        } else {
            surface = surface.child(self.body);
        }

        if let Some(synopsis) = self.synopsis {
            surface = surface
                .child(div().h(px(1.0)).bg(gpui::rgba(self.colors.divider_rgba)))
                .child(render_compact_synopsis_strip(
                    synopsis.label,
                    synopsis.meta,
                    synopsis.description,
                    self.colors.foreground,
                    self.colors.muted_foreground,
                ));
        }

        surface
    }
}
