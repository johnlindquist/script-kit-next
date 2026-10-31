use super::*;

const NAMING_PROMPT_BODY_SELECTOR: &str = "naming-prompt-body-content";

impl Focusable for NamingPrompt {
    fn focus_handle(&self, cx: &gpui::App) -> FocusHandle {
        self.editor
            .as_ref()
            .map(|e| e.read(cx).get_focus_handle())
            .unwrap_or_else(|| self.focus_handle.clone())
    }
}

impl Render for NamingPrompt {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tokens = get_tokens(self.design_variant);
        let spacing = tokens.spacing();

        let palette = crate::components::prompt_text_palette(&self.theme);
        let (text_primary, text_secondary, text_muted) =
            (palette.primary, palette.label, palette.help);
        let colors = crate::components::FormFieldColors::from_theme(&self.theme);

        let title = match self.target {
            NamingTarget::Script => "Name your script",
            NamingTarget::Extension => "Name your scriptlet bundle",
        };

        let placeholder = self
            .placeholder
            .clone()
            .unwrap_or_else(|| match self.target {
                NamingTarget::Script => "Daily notes".to_string(),
                NamingTarget::Extension => "Writing tools".to_string(),
            });

        let editor = self
            .editor
            .get_or_insert_with(|| {
                let mut field = crate::protocol::Field::new("friendly-name".into())
                    .with_label("Display name".into())
                    .with_placeholder(placeholder);
                field.value = Some(self.friendly_name.clone());
                cx.new(|cx| crate::components::FormTextField::new(field, colors, cx))
            })
            .clone();
        let issue = self
            .visible_issue()
            .map(|error| SharedString::from(error.message()));
        editor.update(cx, |field, cx| {
            field.sync_host(&self.friendly_name, issue, colors, cx)
        });
        if !editor.read(cx).get_focus_handle().is_focused(window) {
            editor.read(cx).get_focus_handle().focus(window, cx);
        }
        let preview_style = crate::components::prompt_preview_style(&self.theme);

        let preview_filename = self.filename_preview();
        let preview_path_text = SharedString::from(
            self.target_directory
                .join(&preview_filename)
                .display()
                .to_string(),
        );

        let content = div()
            .id(gpui::ElementId::Name("window:naming".into()))
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .text_color(text_primary)
            .child(
                div()
                    .w_full()
                    .flex_1()
                    .debug_selector(|| NAMING_PROMPT_BODY_SELECTOR.to_string())
                    .flex()
                    .flex_col()
                    .gap(px(spacing.gap_lg))
                    .child(crate::components::prompt_form_intro(
                        title,
                        "Choose a display name. Script Kit generates the filename automatically.",
                        text_primary,
                        text_muted,
                        spacing.gap_sm,
                    ))
                    .child(editor)
                    .when_some(
                        self.template.as_ref().map(|t| t.label.clone()),
                        |d, label| {
                            d.child(crate::components::prompt_form_section(
                                "Template",
                                text_secondary,
                                spacing.gap_sm,
                                crate::components::prompt_surface(
                                    preview_style.background,
                                    preview_style.border,
                                )
                                .child(
                                    div()
                                        .w_full()
                                        .text_sm()
                                        .text_color(text_primary)
                                        .child(SharedString::from(label)),
                                ),
                            ))
                        },
                    )
                    .child(crate::components::prompt_form_section(
                        "File to create",
                        text_secondary,
                        spacing.gap_sm,
                        crate::components::prompt_surface(
                            preview_style.background,
                            preview_style.border,
                        )
                        .child(
                            div()
                                .w_full()
                                .flex()
                                .flex_col()
                                .gap(px(spacing.gap_sm))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(text_primary)
                                        .child(SharedString::from(preview_filename)),
                                )
                                .child(crate::components::prompt_scroll_value_with_id(
                                    "naming-preview-path",
                                    preview_path_text,
                                    text_muted,
                                )),
                        ),
                    )),
            );

        let content = crate::components::render_inset_prompt_body(
            "naming-prompt-inset-body",
            content,
            crate::components::PromptBodyInsets::MainMenu(self.design_variant),
        );

        FocusablePrompt::new(content)
            .key_context("naming_prompt")
            .focus_handle(self.focus_handle.clone())
            .build(
                window,
                cx,
                |this, intercepted_key, _event, _window, _cx| match intercepted_key {
                    FocusablePromptInterceptedKey::Escape => {
                        this.submit_cancel();
                        true
                    }
                    FocusablePromptInterceptedKey::CmdW | FocusablePromptInterceptedKey::CmdK => {
                        false
                    }
                },
                |this, event, _window, cx| {
                    let key = event.keystroke.key.as_str();

                    if is_key_enter(key) {
                        this.submit(cx);
                        cx.stop_propagation();
                    }
                },
            )
            .capture_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, _, cx| {
                if let Some(value) = this
                    .editor
                    .as_ref()
                    .and_then(|e| crate::components::FormTextField::apply_bound_edit(e, event, cx))
                {
                    this.set_input(value, cx);
                    cx.stop_propagation();
                }
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = include_str!("render.rs");

    #[gpui::test]
    fn of58ab_layout_lock_naming_body_uses_main_menu_content_insets(cx: &mut gpui::TestAppContext) {
        let def = crate::designs::current_main_menu_theme().def();
        let spacing = get_tokens(DesignVariant::Default).spacing();
        let window = cx.update(|cx| {
            let mut options = gpui::WindowOptions::default();
            options.window_bounds = Some(gpui::WindowBounds::Windowed(gpui::Bounds::new(
                gpui::point(px(0.0), px(0.0)),
                gpui::size(px(480.0), px(360.0)),
            )));
            cx.open_window(options, |_, cx| {
                let focus_handle = cx.focus_handle();
                cx.new(|_| {
                    NamingPrompt::new(
                        "layout-lock".to_string(),
                        NamingPromptConfig::new(NamingTarget::Script, PathBuf::from("/tmp"), "ts"),
                        focus_handle,
                        Arc::new(|_, _| {}),
                        Arc::new(theme::Theme::default()),
                    )
                })
            })
            .expect("naming layout test window should open")
        });

        cx.run_until_parked();

        window
            .update(cx, |_, window, _| {
                let body = window
                    .debug_bounds_entries()
                    .iter()
                    .find(|entry| entry.selector == NAMING_PROMPT_BODY_SELECTOR)
                    .expect("naming body should publish debug bounds");
                assert_eq!(body.bounds.origin.x, px(def.shell.content_inset_x));
                assert_eq!(body.bounds.origin.y, px(spacing.padding_sm));
            })
            .expect("naming layout test window should update");
    }

    #[test]
    fn naming_render_uses_shared_create_flow_helpers() {
        assert!(
            SOURCE.contains("prompt_form_intro("),
            "render.rs should use prompt_form_intro"
        );
        assert!(
            SOURCE.contains("prompt_form_section("),
            "render.rs should use prompt_form_section"
        );
    }

    #[test]
    fn naming_render_leaves_submit_hint_to_footer() {
        let production_code = SOURCE.split("#[cfg(test)]").next().unwrap_or(SOURCE);
        assert!(
            !production_code.contains("Press Enter to create"),
            "render.rs production code should not contain submit hint — the footer handles that"
        );
    }

    #[test]
    fn naming_render_no_longer_duplicates_footer_cancel_hint() {
        // Check that the non-test portion of the file doesn't contain the hint.
        // Split at the test module boundary to avoid matching our own assertion.
        let production_code = SOURCE.split("#[cfg(test)]").next().unwrap_or(SOURCE);
        let needle = ["Esc", " to ", "cancel"].concat();
        assert!(
            !production_code.contains(&needle),
            "render.rs production code should not contain cancel hint — the footer handles that"
        );
    }
}
