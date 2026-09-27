//! ⇥ made the agent the Owner (S04): a pill ahead of the words in the input says so, in the
//! Owner's color, until Enter starts the agent, Esc or ⇥ takes it back, or the input empties.

use gpui::{div, prelude::*, px, rgb, rgba, AnyElement};
use sk_one_window::contract::{marks, words};

use crate::designs::MainMenuThemeDef;

pub(crate) const OWNER_PILL_ID: &str = "main-input-owner-pill";

const PILL_HEIGHT_PX: f32 = 22.0;
const FILL_ALPHA: u32 = 0x24;
const EDGE_ALPHA: u32 = 0x66;

pub(crate) fn render_owner_pill(theme: &crate::theme::Theme, def: MainMenuThemeDef) -> AnyElement {
    let tokens = def.slot_line;
    div()
        .id(OWNER_PILL_ID)
        .debug_selector(|| OWNER_PILL_ID.to_string())
        .flex_none()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(5.0))
        .h(px(PILL_HEIGHT_PX))
        .px(px(8.0))
        .mr(px(8.0))
        .rounded(px(PILL_HEIGHT_PX / 2.0))
        .bg(rgba((marks::OWNER << 8) | FILL_ALPHA))
        .border_1()
        .border_color(rgba((marks::OWNER << 8) | EDGE_ALPHA))
        .whitespace_nowrap()
        .text_size(px(tokens.hint_font_size))
        .child(
            div()
                .text_color(rgb(theme.colors.text.muted))
                .child("OWNER"),
        )
        .child(
            div()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(rgb(marks::OWNER))
                .child(words::OWNER_AGENT),
        )
        .child(
            crate::components::footer_chrome::render_footer_row_shortcut_keycaps_from_tokens(
                ["⇥"],
                theme,
                (theme.colors.text.muted << 8) | 0xFF,
            ),
        )
        .into_any_element()
}
