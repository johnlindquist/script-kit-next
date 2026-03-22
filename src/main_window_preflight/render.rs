use gpui::{div, prelude::*, px, rgb, AnyElement, FontWeight};

pub(crate) fn render_main_window_preflight_receipt(
    app: &crate::ScriptListApp,
    receipt: &crate::main_window_preflight::MainWindowPreflightReceipt,
) -> AnyElement {
    let chrome = crate::theme::AppChromeColors::from_theme(&app.theme);

    let mut warnings_el = div().flex().flex_col().gap(px(4.));
    for warning in &receipt.warnings {
        warnings_el = warnings_el.child(
            div()
                .text_xs()
                .text_color(rgb(chrome.badge_text_hex))
                .child(format!("• {}", warning)),
        );
    }

    div()
        .id("main-window-preflight")
        .w_full()
        .flex()
        .flex_col()
        .gap(px(10.))
        .p(px(12.))
        .child(
            div()
                .text_xs()
                .text_color(rgb(chrome.badge_text_hex))
                .child("Execution Contract"),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(format!("\u{21B5} {}", receipt.enter_action.label)),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(chrome.badge_text_hex))
                        .child(format!(
                            "{} \u{00B7} {}",
                            receipt.enter_action.type_label, receipt.enter_action.subject
                        )),
                ),
        )
        .when_some(receipt.tab_action.as_ref(), |d, tab| {
            d.child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(format!("Tab {}", tab.label)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(chrome.badge_text_hex))
                            .child(tab.subject.clone()),
                    ),
            )
        })
        .when(
            !receipt.warnings.is_empty(),
            |d: gpui::Stateful<gpui::Div>| d.child(warnings_el),
        )
        .into_any_element()
}
