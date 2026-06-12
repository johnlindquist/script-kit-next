//! Sediment presentation helpers for the Day Page host.

use gpui::{
    div, prelude::*, px, AnyElement, App, IntoElement, ParentElement, SharedString, Styled, Window,
};

use super::sediment::FRAGMENT_BACK_ID;

/// Compact back affordance when the Day Page editor is bound to a fragment file.
pub fn render_fragment_back_bar(
    label: SharedString,
    on_back: impl Fn(&mut Window, &mut App) + 'static,
) -> AnyElement {
    div()
        .id(FRAGMENT_BACK_ID)
        .w_full()
        .flex()
        .items_center()
        .gap(px(8.))
        .pb(px(6.))
        .text_sm()
        .cursor_pointer()
        .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
            on_back(window, cx);
            cx.stop_propagation();
        })
        .child("←")
        .child(label)
        .into_any_element()
}
