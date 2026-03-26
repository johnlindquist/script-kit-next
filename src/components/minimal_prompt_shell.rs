use gpui::{div, prelude::*, px, AnyElement, App, IntoElement, RenderOnce, Window};

use crate::ui::chrome::{HEADER_PADDING_X, HEADER_PADDING_Y};

#[derive(IntoElement)]
pub struct MinimalPromptShell {
    radius: f32,
    vibrancy_bg: Option<gpui::Rgba>,
    child: AnyElement,
}

impl MinimalPromptShell {
    pub fn new(radius: f32, vibrancy_bg: Option<gpui::Rgba>, content: impl IntoElement) -> Self {
        Self {
            radius,
            vibrancy_bg,
            child: content.into_any_element(),
        }
    }

    #[allow(dead_code)]
    pub fn from_child(child: impl IntoElement) -> Self {
        Self {
            radius: 0.0,
            vibrancy_bg: None,
            child: child.into_any_element(),
        }
    }
}

impl RenderOnce for MinimalPromptShell {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .rounded(px(self.radius))
            .when_some(self.vibrancy_bg, |div, bg| div.bg(bg))
            .px(px(HEADER_PADDING_X))
            .py(px(HEADER_PADDING_Y))
            .child(self.child)
    }
}
