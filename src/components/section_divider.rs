use gpui::{div, prelude::*, px, rgba, App, IntoElement, RenderOnce, SharedString, Window};

use crate::ui::chrome::{alpha_from_opacity, DIVIDER_HEIGHT, DIVIDER_OPACITY};

#[derive(IntoElement)]
pub struct SectionDivider {
    id: Option<SharedString>,
}

impl SectionDivider {
    pub fn new() -> Self {
        Self { id: None }
    }

    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }
}

impl Default for SectionDivider {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for SectionDivider {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = crate::theme::get_cached_theme();
        let border_rgba = (theme.colors.ui.border << 8) | alpha_from_opacity(DIVIDER_OPACITY);
        let divider = div().w_full().h(px(DIVIDER_HEIGHT)).bg(rgba(border_rgba));

        if let Some(id) = self.id {
            divider.id(id).into_any_element()
        } else {
            divider.into_any_element()
        }
    }
}
