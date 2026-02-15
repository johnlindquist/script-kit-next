use crate::list_item::FONT_MONO;
use crate::theme::get_cached_theme;
use gpui::*;

/// Container for story content
pub fn story_container() -> Div {
    let theme = get_cached_theme();
    div()
        .flex()
        .flex_col()
        .gap_4()
        .p_4()
        .bg(rgb(theme.colors.background.main))
        .w_full()
        .min_h_full()
}

/// Section with title
pub fn story_section(title: &str) -> Div {
    let theme = get_cached_theme();
    div().flex().flex_col().gap_2().child(
        div()
            .text_sm()
            .text_color(rgb(theme.colors.text.tertiary))
            .child(title.to_string()),
    )
}

/// Item row with label and element
pub fn story_item(label: &str, element: impl IntoElement) -> Div {
    let theme = get_cached_theme();
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_4()
        .child(
            div()
                .w(px(120.))
                .text_sm()
                .text_color(rgb(theme.colors.text.dimmed))
                .child(label.to_string()),
        )
        .child(element)
}

/// Code block for examples
pub fn code_block(code: &str) -> Div {
    let theme = get_cached_theme();
    div()
        .font_family(FONT_MONO)
        .text_sm()
        .p_2()
        .bg(rgb(theme.colors.background.title_bar))
        .rounded_md()
        .overflow_hidden()
        .child(code.to_string())
}

/// Horizontal divider
pub fn story_divider() -> Div {
    let theme = get_cached_theme();
    div()
        .h(px(1.))
        .w_full()
        .bg(rgb(theme.colors.ui.border))
        .my_2()
}
