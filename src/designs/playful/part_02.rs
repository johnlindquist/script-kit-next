/// Render playful-styled header
///
/// Colorful header with emoji and rounded badge.
pub fn render_playful_header(title: &str, colors: PlayfulColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(56.))
        .px(px(HORIZONTAL_PADDING))
        .bg(rgb(colors.cream))
        .border_b(px(BORDER_WIDTH))
        .border_color(rgb(colors.coral))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                .child(div().text_xl().child("🎉"))
                .child(
                    div()
                        .text_lg()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(colors.coral))
                        .child(title.to_string()),
                ),
        )
        .child(
            // Playful badge
            div()
                .px(px(12.))
                .py(px(4.))
                .bg(rgb(colors.mint))
                .rounded(px(PILL_RADIUS))
                .text_sm()
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(colors.white))
                .child("✨ Fun Mode"),
        )
}

/// Render playful-styled preview panel
///
/// Colorful card with big rounded corners and playful shadow.
pub fn render_playful_preview_panel(
    content: Option<&str>,
    colors: PlayfulColors,
) -> impl IntoElement {
    let emoji = get_emoji_for_name(content.unwrap_or("preview"));
    let display_content = content.unwrap_or("Pick something fun!");
    let text_color = if content.is_some() {
        rgb(colors.dark_coral)
    } else {
        rgb(colors.soft_gray)
    };

    div().w_full().h_full().p(px(16.)).child(
        div()
            .w_full()
            .h_full()
            .p(px(20.))
            .bg(rgb(colors.white))
            .border(px(BORDER_WIDTH))
            .border_color(rgb(colors.mint))
            .rounded(px(CARD_RADIUS))
            .shadow_md()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(8.))
                    .mb(px(16.))
                    .child(div().text_lg().child(emoji))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(colors.lavender))
                            .child("Preview"),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .text_base()
                    .text_color(text_color)
                    .overflow_hidden()
                    .child(display_content.to_string()),
            ),
    )
}

/// Render playful-styled log panel
///
/// Colorful console with fun styling.
pub fn render_playful_log_panel(logs: &[String], colors: PlayfulColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(140.))
        .px(px(HORIZONTAL_PADDING))
        .pb(px(16.))
        .child(
            div()
                .w_full()
                .h_full()
                .p(px(12.))
                .bg(rgb(colors.white))
                .border(px(BORDER_WIDTH))
                .border_color(rgb(colors.lavender))
                .rounded(px(ELEMENT_RADIUS))
                .shadow_sm()
                .flex()
                .flex_col()
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(6.))
                        .mb(px(8.))
                        .child(div().text_sm().child("📋"))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(colors.lavender))
                                .child("Activity Log"),
                        ),
                )
                .child(
                    div()
                        .flex_1()
                        .overflow_hidden()
                        .flex()
                        .flex_col()
                        .gap(px(4.))
                        .children(logs.iter().enumerate().map(|(i, log)| {
                            let color = match i % 3 {
                                0 => colors.coral,
                                1 => colors.mint,
                                _ => colors.lavender,
                            };
                            div()
                                .text_xs()
                                .text_color(rgb(color))
                                .font_weight(FontWeight::MEDIUM)
                                .child(format!("→ {}", log))
                        })),
                ),
        )
}

/// Render playful-styled window container
///
/// Warm cream background with playful border and shadow.
pub fn render_playful_window_container(
    colors: PlayfulColors,
    children: impl IntoElement,
) -> impl IntoElement {
    div()
        .w_full()
        .h_full()
        .bg(rgb(colors.cream))
        .border(px(BORDER_WIDTH))
        .border_color(rgb(colors.coral))
        .rounded(px(CARD_RADIUS))
        .overflow_hidden()
        .shadow(vec![
            BoxShadow {
                color: hsla(0.0, 0.7, 0.7, 0.2),
                offset: point(px(0.), px(8.)),
                blur_radius: px(24.),
                spread_radius: px(-4.),
            },
            BoxShadow {
                color: hsla(0.5, 0.7, 0.7, 0.15),
                offset: point(px(4.), px(4.)),
                blur_radius: px(0.),
                spread_radius: px(0.),
            },
        ])
        .child(children)
}

// Note: Tests omitted due to GPUI macro recursion limit issues.
// Playful design constants:
// - CARD_RADIUS >= 24.0 (very rounded corners)
// - coral: 0xff6b6b
// - mint: 0x4ecdc4
// - lavender: 0xa29bfe
// Emoji mapping based on first letter of name
