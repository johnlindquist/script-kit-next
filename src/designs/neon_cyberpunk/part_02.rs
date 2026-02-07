/// Render neon cyberpunk-styled header
///
/// Dark purple background with cyan/magenta neon accents and glow.
pub fn render_neon_cyberpunk_header(title: &str) -> impl IntoElement {
    div()
        .w_full()
        .h(px(48.))
        .px(px(16.))
        .bg(rgb(colors::BACKGROUND_PANEL))
        .border_b_1()
        .border_color(rgb(colors::CYAN))
        .shadow(vec![BoxShadow {
            color: hsla(180.0 / 360.0, 1.0, 0.5, 0.4),
            offset: point(px(0.), px(2.)),
            blur_radius: px(8.),
            spread_radius: px(0.),
        }])
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .font_family("Menlo")
        .child(
            div()
                .text_base()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(colors::CYAN))
                .child(format!("// {}", title.to_uppercase())),
        )
        .child(
            // Animated-looking neon indicator
            div()
                .flex()
                .flex_row()
                .gap(px(4.))
                .child(
                    div()
                        .w(px(6.))
                        .h(px(6.))
                        .rounded_full()
                        .bg(rgb(colors::MAGENTA))
                        .shadow(vec![BoxShadow {
                            color: hsla(300.0 / 360.0, 1.0, 0.5, 0.8),
                            offset: point(px(0.), px(0.)),
                            blur_radius: px(6.),
                            spread_radius: px(1.),
                        }]),
                )
                .child(
                    div()
                        .w(px(6.))
                        .h(px(6.))
                        .rounded_full()
                        .bg(rgb(colors::CYAN))
                        .shadow(vec![BoxShadow {
                            color: hsla(180.0 / 360.0, 1.0, 0.5, 0.8),
                            offset: point(px(0.), px(0.)),
                            blur_radius: px(6.),
                            spread_radius: px(1.),
                        }]),
                ),
        )
}

/// Render neon cyberpunk-styled preview panel
///
/// Dark panel with neon border glow and futuristic styling.
pub fn render_neon_cyberpunk_preview_panel(content: Option<&str>) -> impl IntoElement {
    let display_content = content.unwrap_or("// awaiting input...");
    let text_color = if content.is_some() {
        rgb(colors::CYAN)
    } else {
        rgb(colors::CYAN_DIM)
    };

    div()
        .w_full()
        .h_full()
        .p(px(16.))
        .bg(rgb(colors::BACKGROUND_PANEL))
        .border_1()
        .border_color(rgb(colors::MAGENTA))
        .rounded(px(4.))
        .shadow(vec![BoxShadow {
            color: hsla(300.0 / 360.0, 1.0, 0.5, 0.3),
            offset: point(px(0.), px(0.)),
            blur_radius: px(12.),
            spread_radius: px(0.),
        }])
        .flex()
        .flex_col()
        .font_family("Menlo")
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(colors::MAGENTA_DIM))
                .mb(px(12.))
                .child("/* PREVIEW */"),
        )
        .child(
            div()
                .flex_1()
                .text_sm()
                .text_color(text_color)
                .overflow_hidden()
                .child(display_content.to_string()),
        )
}

/// Render neon cyberpunk-styled log panel
///
/// Terminal-like output with neon green/cyan text on dark background.
pub fn render_neon_cyberpunk_log_panel(logs: &[String]) -> impl IntoElement {
    div()
        .w_full()
        .h(px(150.))
        .p(px(12.))
        .bg(rgb(colors::BACKGROUND))
        .border_1()
        .border_color(rgba(0x00ffff40))
        .rounded(px(4.))
        .flex()
        .flex_col()
        .font_family("Menlo")
        .child(
            div()
                .text_xs()
                .text_color(rgb(colors::YELLOW))
                .mb(px(8.))
                .child("> SYSTEM LOG_"),
        )
        .child(
            div()
                .flex_1()
                .overflow_hidden()
                .flex()
                .flex_col()
                .gap(px(2.))
                .children(logs.iter().enumerate().map(|(i, log)| {
                    let color = if i % 2 == 0 {
                        rgb(colors::CYAN)
                    } else {
                        rgb(colors::CYAN_DIM)
                    };
                    div()
                        .text_xs()
                        .text_color(color)
                        .child(format!("> {}", log))
                })),
        )
}

/// Render neon cyberpunk-styled window container
///
/// Deep dark background with subtle neon border glow.
pub fn render_neon_cyberpunk_window_container(children: impl IntoElement) -> impl IntoElement {
    div()
        .w_full()
        .h_full()
        .bg(rgb(colors::BACKGROUND))
        .border_1()
        .border_color(rgba(0x00ffff30))
        .rounded(px(4.))
        .overflow_hidden()
        .shadow(vec![
            // Outer cyan glow
            BoxShadow {
                color: hsla(180.0 / 360.0, 1.0, 0.5, 0.2),
                offset: point(px(0.), px(0.)),
                blur_radius: px(20.),
                spread_radius: px(0.),
            },
            // Inner magenta accent
            BoxShadow {
                color: hsla(300.0 / 360.0, 1.0, 0.5, 0.1),
                offset: point(px(0.), px(0.)),
                blur_radius: px(40.),
                spread_radius: px(-10.),
            },
        ])
        .child(children)
}

// Note: Tests omitted due to GPUI macro recursion limit issues.
// Neon Cyberpunk colors:
// - text_primary: 0x00ffff (cyan)
// - accent_selected: 0xff00ff (magenta)
