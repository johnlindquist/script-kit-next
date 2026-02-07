/// Render glassmorphism-styled header
///
/// Features a frosted glass bar with translucent background and subtle border.
pub fn render_glassmorphism_header(title: &str, colors: GlassColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(48.))
        .px(px(20.))
        .bg(rgba(colors.card_bg))
        .border_b_1()
        .border_color(rgba(colors.border))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .font_family(".AppleSystemUIFont")
        .shadow(vec![BoxShadow {
            color: hsla(0., 0., 0., 0.05),
            offset: point(px(0.), px(2.)),
            blur_radius: px(8.),
            spread_radius: px(0.),
        }])
        .child(
            div()
                .text_base()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(colors.text_primary))
                .child(title.to_string()),
        )
        .child(
            // Subtle accent indicator
            div()
                .w(px(8.))
                .h(px(8.))
                .rounded_full()
                .bg(rgb(colors.accent)),
        )
}

/// Render glassmorphism-styled preview panel
///
/// A frosted glass panel for showing script previews with layered transparency.
pub fn render_glassmorphism_preview_panel(
    content: Option<&str>,
    colors: GlassColors,
) -> impl IntoElement {
    let display_content = content.unwrap_or("Select a script to preview");
    let text_color = if content.is_some() {
        rgb(colors.text_primary)
    } else {
        rgba(colors.text_muted)
    };

    div()
        .w_full()
        .h_full()
        .p(px(16.))
        .bg(rgba(colors.card_bg))
        .border_1()
        .border_color(rgba(colors.border))
        .rounded(px(16.))
        .shadow(vec![BoxShadow {
            color: hsla(0., 0., 0., 0.1),
            offset: point(px(0.), px(4.)),
            blur_radius: px(12.),
            spread_radius: px(0.),
        }])
        .flex()
        .flex_col()
        .font_family(".AppleSystemUIFont")
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgba(colors.text_muted))
                .mb(px(12.))
                .child("PREVIEW"),
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

/// Render glassmorphism-styled log panel
///
/// A translucent panel for displaying logs with frosted glass effect.
pub fn render_glassmorphism_log_panel(logs: &[String], colors: GlassColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(150.))
        .p(px(12.))
        .bg(rgba(colors.overlay_subtle)) // Theme-aware subtle glass
        .border_1()
        .border_color(rgba(colors.border))
        .rounded(px(12.))
        .flex()
        .flex_col()
        .font_family("Menlo")
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgba(colors.text_muted))
                .mb(px(8.))
                .child("LOGS"),
        )
        .child(
            div()
                .flex_1()
                .overflow_hidden()
                .flex()
                .flex_col()
                .gap(px(2.))
                .children(logs.iter().map(|log| {
                    div()
                        .text_xs()
                        .text_color(rgba(colors.text_secondary))
                        .child(log.clone())
                })),
        )
}

/// Render glassmorphism-styled window container
///
/// The main window wrapper with frosted glass background and soft edges.
pub fn render_glassmorphism_window_container(
    colors: GlassColors,
    children: impl IntoElement,
) -> impl IntoElement {
    div()
        .w_full()
        .h_full()
        .bg(rgba(colors.background_main))
        .rounded(px(16.))
        .overflow_hidden()
        .shadow(vec![
            BoxShadow {
                color: hsla(0., 0., 0., 0.15),
                offset: point(px(0.), px(8.)),
                blur_radius: px(32.),
                spread_radius: px(0.),
            },
            BoxShadow {
                color: hsla(0., 0., 1., 0.1),
                offset: point(px(0.), px(1.)),
                blur_radius: px(0.),
                spread_radius: px(1.),
            },
        ])
        .child(children)
}

// Note: Tests omitted due to GPUI macro recursion limit issues.
// The GlassColors and GlassmorphismRenderer are verified through integration tests.
