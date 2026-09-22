use gpui::{div, InteractiveElement, IntoElement, ParentElement, Styled};

#[allow(
    dead_code,
    reason = "separately compiled builtin renderers and paint measurement own this stable selector"
)]
pub(crate) const BUILTIN_LEADING_SEPARATOR_ID: &str = "builtin-leading-separator";

/// Shared persistent first row for builtin list browsers.
///
/// The row is always present. Transient status belongs in its label so status
/// changes cannot add floating chrome or shift the list below it (OF-15).
/// Its slot uses the active launcher's first-section budget, not the legacy
/// section label's intrinsic height, so switching browsers cannot shift rows.
#[allow(
    dead_code,
    reason = "the separately compiled launcher calls this from seven builtin renderers"
)]
pub(crate) fn render_builtin_leading_separator(
    label: &str,
    status: Option<&str>,
    colors: crate::list_item::ListItemColors,
    main_menu_theme: crate::designs::MainMenuThemeVariant,
) -> impl IntoElement {
    let text = status.map_or_else(|| label.to_string(), |status| format!("{label} · {status}"));
    div()
        .debug_selector(|| BUILTIN_LEADING_SEPARATOR_ID.to_string())
        .w_full()
        .h(gpui::px(
            crate::list_item::effective_first_section_header_height_for_theme(main_menu_theme),
        ))
        .flex_shrink_0()
        .child(crate::list_item::render_section_header(
            &text, None, colors, true,
        ))
}

/// Semantic mirror of [`render_builtin_leading_separator`] for getElements.
#[allow(
    dead_code,
    reason = "the separately compiled launcher projects this separator in app_layout/collect_elements.rs"
)]
pub(crate) fn builtin_leading_separator_element(
    surface: &str,
    label: &str,
    status: Option<&str>,
) -> crate::protocol::ElementInfo {
    let mut element = crate::protocol::ElementInfo::panel(&format!("{surface}-leading-separator"));
    element.text =
        Some(status.map_or_else(|| label.to_string(), |status| format!("{label} · {status}")));
    element.role = Some("sectionHeader".to_string());
    element.kind = Some("leadingSeparator".to_string());
    element.selectable = Some(false);
    element.status_kind = status.map(str::to_string);
    element
}

#[cfg(test)]
mod tests {
    use gpui::{div, AppContext, InteractiveElement, ParentElement, Styled};

    struct TestBuiltinLeadingSeparator;

    impl gpui::Render for TestBuiltinLeadingSeparator {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut gpui::Context<Self>,
        ) -> impl gpui::IntoElement {
            let theme = crate::theme::Theme::default();
            div()
                .debug_selector(|| "test-builtin-browser".to_string())
                .w_full()
                .h_full()
                .flex()
                .flex_col()
                .child(super::render_builtin_leading_separator(
                    "Files",
                    Some("Indexing files"),
                    crate::list_item::ListItemColors::from_theme(&theme),
                    crate::designs::MainMenuThemeVariant::default(),
                ))
                .child(
                    div()
                        .debug_selector(|| "test-builtin-results".to_string())
                        .w_full()
                        .h_full()
                        .flex_1()
                        .min_h(gpui::px(0.0)),
                )
        }
    }

    #[test]
    fn semantic_status_stays_inside_the_leading_separator() {
        let element = super::builtin_leading_separator_element(
            "file-search",
            "Files",
            Some("Indexing files"),
        );
        assert_eq!(element.role.as_deref(), Some("sectionHeader"));
        assert_eq!(element.kind.as_deref(), Some("leadingSeparator"));
        assert_eq!(element.text.as_deref(), Some("Files · Indexing files"));
        assert_eq!(element.status_kind.as_deref(), Some("Indexing files"));
        assert_eq!(element.selectable, Some(false));
    }

    #[gpui::test]
    fn leading_section_keeps_launcher_slot_under_results_content_pressure(
        cx: &mut gpui::TestAppContext,
    ) {
        use gpui::px;

        let window = cx.update(|cx| {
            let mut options = gpui::WindowOptions::default();
            options.window_bounds = Some(gpui::WindowBounds::Windowed(gpui::Bounds::new(
                gpui::point(px(0.0), px(0.0)),
                gpui::size(px(480.0), px(120.0)),
            )));
            cx.open_window(options, |_, cx| cx.new(|_| TestBuiltinLeadingSeparator))
                .unwrap()
        });

        cx.run_until_parked();

        window
            .update(cx, |_, window, _| {
                let bounds = window.debug_bounds();
                let browser = bounds.get("test-builtin-browser").unwrap();
                let separator = bounds.get(super::BUILTIN_LEADING_SEPARATOR_ID).unwrap();
                let results = bounds.get("test-builtin-results").unwrap();
                let launcher_slot = px(
                    crate::list_item::effective_first_section_header_height_for_theme(
                        crate::designs::MainMenuThemeVariant::default(),
                    ),
                );

                assert_eq!(separator.size.height, launcher_slot);
                assert_eq!(separator.origin.y, browser.origin.y);
                assert_eq!(results.origin.y - browser.origin.y, launcher_slot);
                assert_eq!(results.bottom(), browser.bottom());
            })
            .unwrap();
    }
}
