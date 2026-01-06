use gpui::*;

/// Container for story content
pub fn story_container() -> Div {
    div()
        .flex()
        .flex_col()
        .gap_4()
        .p_4()
        .bg(rgb(0x1e1e1e))
        .w_full()
        .min_h_full()
}

/// Section with title
pub fn story_section(title: &str) -> Div {
    div().flex().flex_col().gap_2().child(
        div()
            .text_sm()
            .text_color(rgb(0x888888))
            .child(title.to_string()),
    )
}

/// Item row with label and element
pub fn story_item(label: &str, element: impl IntoElement) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_4()
        .child(
            div()
                .w(px(120.))
                .text_sm()
                .text_color(rgb(0x666666))
                .child(label.to_string()),
        )
        .child(element)
}

/// Code block for examples
pub fn code_block(code: &str) -> Div {
    div()
        .font_family("Menlo")
        .text_sm()
        .p_2()
        .bg(rgb(0x2d2d2d))
        .rounded_md()
        .overflow_hidden()
        .child(code.to_string())
}

/// Horizontal divider
pub fn story_divider() -> Div {
    div().h(px(1.)).w_full().bg(rgb(0x3d3d3d)).my_2()
}
