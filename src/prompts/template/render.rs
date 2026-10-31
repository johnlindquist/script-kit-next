use super::*;
use crate::components::{FocusablePrompt, FocusablePromptInterceptedKey};
use crate::ui_foundation::is_key_enter;
use gpui_component::scroll::ScrollableElement;

const TEMPLATE_PROMPT_BODY_SELECTOR: &str = "template-prompt-body-content";

fn template_prompt_body_insets(
    design_variant: DesignVariant,
) -> crate::components::PromptBodyInsets {
    crate::components::PromptBodyInsets::MainMenu(design_variant)
}

impl Focusable for TemplatePrompt {
    fn focus_handle(&self, cx: &gpui::App) -> FocusHandle {
        self.editors
            .get(self.current_input)
            .map(|e| e.read(cx).get_focus_handle())
            .unwrap_or_else(|| self.focus_handle.clone())
    }
}

impl Render for TemplatePrompt {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tokens = get_tokens(self.design_variant);
        let spacing = tokens.spacing();

        let text = crate::components::prompt_text_palette(&self.theme);
        let text_primary = text.primary;
        let text_secondary = text.label;
        let text_muted = text.help;
        let colors = crate::components::FormFieldColors::from_theme(&self.theme);
        let metrics = crate::components::FormFieldMetrics::from_colors(colors);

        let description = if self.inputs.is_empty() {
            "This template has no editable placeholders. Review the preview and press Enter to submit."
                .to_string()
        } else {
            "Fill in the fields below to complete the preview.".to_string()
        };

        let preview = self.preview_template();
        let preview_style = crate::components::prompt_preview_style(&self.theme);

        let mut content = div()
            .id(gpui::ElementId::Name("window:template".into()))
            .debug_selector(|| TEMPLATE_PROMPT_BODY_SELECTOR.to_string())
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .text_color(text_primary)
            .gap(px(spacing.gap_lg))
            .child(crate::components::prompt_form_intro(
                "Complete the template",
                description,
                text_primary,
                text_muted,
                spacing.gap_sm,
            ));

        if self.inputs.is_empty() {
            content = content.child(crate::components::prompt_form_help(
                "No {{placeholders}} found in template.",
                text_secondary,
            ));
        } else {
            let mut fields = div()
                .id("template-fields-scroll")
                .w_full()
                .flex_1()
                .min_h(px(0.))
                .flex()
                .flex_col()
                .gap(px(metrics.field_stack_gap_px))
                .overflow_y_scroll()
                .track_scroll(&self.scroll);
            for (idx, input) in self.inputs.iter().enumerate() {
                let value = &self.values[idx];
                if self.editors.len() == idx {
                    let label = if input.required {
                        format!("{} (required)", input.label)
                    } else {
                        input.label.clone()
                    };
                    let mut def = crate::protocol::Field::new(input.name.clone())
                        .with_label(label)
                        .with_placeholder(input.placeholder.clone());
                    def.value = Some(value.clone());
                    self.editors
                        .push(cx.new(|cx| crate::components::FormTextField::new(def, colors, cx)));
                }
                let editor = self.editors[idx].clone();
                editor.update(cx, |field, cx| {
                    field.sync_host(
                        value,
                        self.validation_errors[idx].clone().map(Into::into),
                        colors,
                        cx,
                    )
                });
                if idx == self.current_input
                    && !editor.read(cx).get_focus_handle().is_focused(window)
                {
                    editor.read(cx).get_focus_handle().focus(window, cx);
                }
                fields = fields.child(
                    div()
                        .id(SharedString::from(format!("template-field-{idx}")))
                        .flex_none()
                        .w_full()
                        .on_click(cx.listener(move |this, _: &gpui::ClickEvent, _, cx| {
                            this.set_current_input(idx);
                            cx.notify();
                        }))
                        .child(editor),
                );
            }

            if self
                .inputs
                .iter()
                .any(|input| Self::is_name_field(&input.name))
            {
                fields = fields.child(crate::components::prompt_form_help(
                    "Naming tip: use lowercase letters, numbers, and hyphens.",
                    text_muted,
                ));
            }

            content = content.child(fields.vertical_scrollbar(&self.scroll));
        }

        let content = content.child(crate::components::prompt_form_section(
            "Preview",
            text_secondary,
            metrics.field_gap_px,
            crate::components::prompt_text_field(preview, preview_style),
        ));
        let content = crate::components::render_inset_prompt_body(
            "template-prompt-inset-body",
            content,
            template_prompt_body_insets(self.design_variant),
        );

        FocusablePrompt::new(content)
            .key_context("template_prompt")
            .focus_handle(self.focus_handle.clone())
            .build(
                window,
                cx,
                |this, intercepted_key, _event, _window, _cx| match intercepted_key {
                    FocusablePromptInterceptedKey::Escape => {
                        this.submit_cancel();
                        true
                    }
                    _ => false,
                },
                |_this, _event, _window, _cx| {},
            )
            // Keymap actions run before raw key capture; consume traversal here
            // before the ancestor Root can move focus outside the template.
            .on_action(cx.listener(|this, _: &gpui_component::Tab, _window, cx| {
                this.next_input(cx);
                cx.stop_propagation();
            }))
            .on_action(
                cx.listener(|this, _: &gpui_component::TabPrev, _window, cx| {
                    this.prev_input(cx);
                    cx.stop_propagation();
                }),
            )
            .capture_key_down(
                cx.listener(|this, event: &gpui::KeyDownEvent, _window, cx| {
                    if let Some(value) = this.editors.get(this.current_input).and_then(|field| {
                        crate::components::FormTextField::apply_bound_edit(field, event, cx)
                    }) {
                        this.set_input(value, cx);
                        cx.stop_propagation();
                        return;
                    }
                    let modifiers = &event.keystroke.modifiers;
                    if modifiers.platform
                        || modifiers.control
                        || modifiers.alt
                        || modifiers.function
                    {
                        return;
                    }

                    let key = event.keystroke.key.as_str();
                    if is_key_enter(key) {
                        this.submit(cx);
                    } else {
                        return;
                    }
                    cx.stop_propagation();
                }),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = include_str!("render.rs");

    #[test]
    fn of58ab_template_inset_policy_matches_main_menu_content_frame() {
        let resolved = template_prompt_body_insets(DesignVariant::Default).resolve();
        let def = crate::designs::current_main_menu_theme().def();
        let spacing = get_tokens(DesignVariant::Default).spacing();
        assert_eq!(resolved.x_px, def.shell.content_inset_x);
        assert_eq!(resolved.y_px, spacing.padding_sm);
    }

    #[gpui::test]
    fn of58ab_layout_lock_template_body_uses_main_menu_content_insets(
        cx: &mut gpui::TestAppContext,
    ) {
        let def = crate::designs::current_main_menu_theme().def();
        let spacing = get_tokens(DesignVariant::Default).spacing();
        cx.update(crate::theme::gpui_kit_bridge::init_component_libraries);
        let window = cx.update(|cx| {
            let mut options = gpui::WindowOptions::default();
            options.window_bounds = Some(gpui::WindowBounds::Windowed(gpui::Bounds::new(
                gpui::point(px(0.0), px(0.0)),
                gpui::size(px(480.0), px(320.0)),
            )));
            cx.open_window(options, |_, cx| {
                let focus_handle = cx.focus_handle();
                cx.new(|_| {
                    TemplatePrompt::new(
                        "layout-lock".to_string(),
                        "Hello {{name}}".to_string(),
                        focus_handle,
                        Arc::new(|_, _| {}),
                        Arc::new(theme::Theme::default()),
                    )
                })
            })
            .expect("template layout test window should open")
        });

        cx.run_until_parked();

        window
            .update(cx, |_, window, _| {
                let body = window
                    .debug_bounds_entries()
                    .iter()
                    .find(|entry| entry.selector == TEMPLATE_PROMPT_BODY_SELECTOR)
                    .expect("template body should publish debug bounds");
                assert_eq!(body.bounds.origin.x, px(def.shell.content_inset_x));
                assert_eq!(body.bounds.origin.y, px(spacing.padding_sm));
            })
            .expect("template layout test window should update");
    }

    #[test]
    fn template_render_uses_shared_create_flow_helpers() {
        assert!(
            SOURCE.contains("prompt_form_intro("),
            "render.rs should use prompt_form_intro"
        );
        assert!(
            SOURCE.contains("prompt_form_section("),
            "render.rs should use prompt_form_section"
        );
        assert!(
            SOURCE.contains("prompt_form_help("),
            "render.rs should use prompt_form_help"
        );
        assert!(
            SOURCE.contains("prompt_text_field("),
            "render.rs should use prompt_text_field"
        );
        assert!(
            SOURCE.contains("prompt_text_palette("),
            "render.rs should use the shared text palette"
        );
    }

    #[test]
    fn template_render_exposes_scrollable_clickable_field_region() {
        assert!(
            SOURCE.contains("template-fields-scroll"),
            "template fields should have a stable scroll region id"
        );
        assert!(
            SOURCE.contains(".track_scroll(&self.scroll)"),
            "template fields should scroll when placeholders exceed available height"
        );
        assert!(
            SOURCE.contains(".on_click(") && SOURCE.contains("set_current_input(idx)"),
            "template fields should be clickable and move the current input"
        );
    }

    #[test]
    fn template_render_no_longer_renders_inline_shortcut_footer_text() {
        let production_code = SOURCE.split("#[cfg(test)]").next().unwrap_or(SOURCE);

        assert!(
            !production_code
                .contains("Tab: next field | Shift+Tab: previous | Enter: submit | Escape: cancel"),
            "render.rs production code should not contain inline shortcut footer text"
        );
    }
}
