use super::*;

impl AiApp {
    pub(super) fn render_presets_dropdown(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let bg_color = theme.background;
        let border_color = theme.border;
        let muted_fg = theme.muted_foreground;
        let accent = theme.accent;
        let accent_fg = theme.accent_foreground;
        let fg = theme.foreground;

        // Build preset items
        let preset_items: Vec<_> = self
            .presets
            .iter()
            .enumerate()
            .map(|(idx, preset)| {
                let is_selected = idx == self.presets_selected_index;
                let icon = preset.icon;
                let name = preset.name.to_string();
                let description = preset.description.to_string();

                div()
                    .id(SharedString::from(format!("preset-{}", idx)))
                    .px_3()
                    .py_2()
                    .mx_1()
                    .rounded_md()
                    .flex()
                    .items_center()
                    .gap_3()
                    .cursor_pointer()
                    .when(is_selected, |el| el.bg(accent))
                    .when(!is_selected, |el| el.hover(|el| el.bg(accent.opacity(0.5))))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.presets_selected_index = idx;
                        this.create_chat_with_preset(window, cx);
                    }))
                    // Icon
                    .child(
                        svg()
                            .external_path(icon.external_path())
                            .size(px(18.))
                            .text_color(if is_selected { accent_fg } else { muted_fg }),
                    )
                    // Name and description
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(if is_selected { accent_fg } else { fg })
                                    .child(name),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(if is_selected {
                                        accent_fg.opacity(0.7)
                                    } else {
                                        muted_fg
                                    })
                                    .child(description),
                            ),
                    )
            })
            .collect();

        // Overlay positioned near the new chat button
        // Theme-aware modal overlay: black for dark mode, white for light mode
        let overlay_bg = Self::get_modal_overlay_background();
        div()
            .id("presets-dropdown-overlay")
            .absolute()
            .inset_0()
            .bg(overlay_bg)
            .flex()
            .items_start()
            .justify_start()
            .pt_12()
            .pl_4()
            .on_click(cx.listener(|this, _, _, cx| {
                this.hide_presets_dropdown(cx);
            }))
            .child(
                div()
                    .id("presets-dropdown-container")
                    .w(px(300.0))
                    .max_h(px(350.0))
                    .bg(bg_color)
                    .border_1()
                    .border_color(border_color)
                    .rounded_lg()
                    // Shadow disabled for vibrancy - shadows on transparent elements cause gray fill
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .on_click(cx.listener(|_, _, _, _| {}))
                    // Header
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .border_b_1()
                            .border_color(border_color)
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(fg)
                            .child("New Chat with Preset"),
                    )
                    // Preset list
                    .child(
                        div()
                            .id("preset-list")
                            .flex_1()
                            .overflow_y_scroll()
                            .p_1()
                            .children(preset_items),
                    )
                    // Footer hint
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .border_t_1()
                            .border_color(border_color)
                            .text_xs()
                            .text_color(muted_fg)
                            .child("Select a preset to start a new chat"),
                    ),
            )
    }

    /// Render the new chat dropdown (Raycast-style with search, last used, presets, models)
    pub(super) fn render_new_chat_dropdown(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let bg_color = theme.background;
        let border_color = theme.border;
        let muted_fg = theme.muted_foreground;
        let accent = theme.accent;
        let accent_fg = theme.accent_foreground;
        let fg = theme.foreground;

        let (filtered_last_used, filtered_presets, filtered_models) =
            self.get_filtered_new_chat_items();

        let current_section = self.new_chat_dropdown_section;
        let current_index = self.new_chat_dropdown_index;

        // Build "Last Used Settings" section items
        let last_used_items: Vec<_> = filtered_last_used
            .iter()
            .enumerate()
            .map(|(idx, setting)| {
                let is_selected = current_section == 0 && idx == current_index;
                let display_name = setting.display_name.clone();
                let provider_name = setting.provider_display_name.clone();
                let model_id = setting.model_id.clone();
                let provider = setting.provider.clone();

                div()
                    .id(SharedString::from(format!("last-used-{}", idx)))
                    .px_3()
                    .py_2()
                    .mx_1()
                    .rounded_md()
                    .flex()
                    .items_center()
                    .justify_between()
                    .cursor_pointer()
                    .when(is_selected, |el| el.bg(accent))
                    .when(!is_selected, |el| el.hover(|el| el.bg(accent.opacity(0.5))))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.hide_new_chat_dropdown(cx);
                        this.create_chat_with_model(&model_id, &provider, window, cx);
                    }))
                    .child(
                        div()
                            .text_sm()
                            .text_color(if is_selected { accent_fg } else { fg })
                            .child(display_name),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(if is_selected {
                                accent_fg.opacity(0.7)
                            } else {
                                muted_fg
                            })
                            .child(provider_name),
                    )
            })
            .collect();

        // Build "Presets" section items
        let preset_items: Vec<_> = filtered_presets
            .iter()
            .enumerate()
            .map(|(idx, preset)| {
                let is_selected = current_section == 1 && idx == current_index;
                let preset_id = preset.id;
                let name = preset.name.to_string();
                let icon = preset.icon;

                // Find the original preset index for create_chat_with_preset
                let original_idx = self
                    .presets
                    .iter()
                    .position(|p| p.id == preset_id)
                    .unwrap_or(0);

                div()
                    .id(SharedString::from(format!("ncd-preset-{}", idx)))
                    .px_3()
                    .py_2()
                    .mx_1()
                    .rounded_md()
                    .flex()
                    .items_center()
                    .gap_2()
                    .cursor_pointer()
                    .when(is_selected, |el| el.bg(accent))
                    .when(!is_selected, |el| el.hover(|el| el.bg(accent.opacity(0.5))))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.presets_selected_index = original_idx;
                        this.hide_new_chat_dropdown(cx);
                        this.create_chat_with_preset(window, cx);
                    }))
                    .child(
                        svg()
                            .external_path(icon.external_path())
                            .size(px(14.))
                            .text_color(if is_selected { accent_fg } else { muted_fg }),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(if is_selected { accent_fg } else { fg })
                            .child(name),
                    )
            })
            .collect();

        // Build "Recently Used" models section items
        let model_items: Vec<_> = filtered_models
            .iter()
            .enumerate()
            .map(|(idx, model)| {
                let is_selected = current_section == 2 && idx == current_index;
                let display_name = model.display_name.clone();
                let provider = model.provider.clone();
                let model_id = model.id.clone();

                // Provider display name
                let provider_display = match provider.as_str() {
                    "anthropic" => "Anthropic",
                    "openai" => "OpenAI",
                    "google" => "Google",
                    "groq" => "Groq",
                    "openrouter" => "OpenRouter",
                    "vercel" => "Vercel",
                    _ => &provider,
                }
                .to_string();

                div()
                    .id(SharedString::from(format!("ncd-model-{}", idx)))
                    .px_3()
                    .py_2()
                    .mx_1()
                    .rounded_md()
                    .flex()
                    .items_center()
                    .justify_between()
                    .cursor_pointer()
                    .when(is_selected, |el| el.bg(accent))
                    .when(!is_selected, |el| el.hover(|el| el.bg(accent.opacity(0.5))))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.hide_new_chat_dropdown(cx);
                        this.create_chat_with_model(&model_id, &provider, window, cx);
                    }))
                    .child(
                        div()
                            .text_sm()
                            .text_color(if is_selected { accent_fg } else { fg })
                            .child(display_name),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(if is_selected {
                                accent_fg.opacity(0.7)
                            } else {
                                muted_fg
                            })
                            .child(provider_display.to_string()),
                    )
            })
            .collect();

        // Build the dropdown overlay - positioned near the header + button
        // Theme-aware modal overlay: black for dark mode, white for light mode
        let overlay_bg = Self::get_modal_overlay_background();
        div()
            .id("new-chat-dropdown-overlay")
            .absolute()
            .inset_0()
            .bg(overlay_bg)
            .flex()
            .items_start()
            .justify_end() // Align to right (near the + button)
            .pt(px(40.)) // Below the titlebar
            .pr_3() // Right padding
            .on_click(cx.listener(|this, _, _, cx| {
                this.hide_new_chat_dropdown(cx);
            }))
            .child(
                div()
                    .id("new-chat-dropdown-container")
                    .w(px(320.0))
                    .max_h(px(450.0))
                    .bg(bg_color)
                    .border_1()
                    .border_color(border_color)
                    .rounded_lg()
                    // Shadow disabled for vibrancy - shadows on transparent elements cause gray fill
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .on_click(cx.listener(|_, _, _, _| {})) // Prevent click-through
                    // Search input header
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .border_b_1()
                            .border_color(border_color)
                            .child(
                                Input::new(&self.new_chat_dropdown_input)
                                    .w_full()
                                    .appearance(false) // Minimal appearance
                                    .bordered(false),
                            ),
                    )
                    // Scrollable sections
                    .child(
                        div()
                            .id("new-chat-dropdown-sections")
                            .flex_1()
                            .overflow_y_scroll()
                            .p_1()
                            // Last Used Settings section (if not empty)
                            .when(!last_used_items.is_empty(), |d| {
                                d.child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .w_full()
                                        .mb_2()
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .text_color(muted_fg)
                                                .px_3()
                                                .py_1()
                                                .child("Last Used Settings"),
                                        )
                                        .children(last_used_items),
                                )
                            })
                            // Presets section (if not empty)
                            .when(!preset_items.is_empty(), |d| {
                                d.child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .w_full()
                                        .mb_2()
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .text_color(muted_fg)
                                                .px_3()
                                                .py_1()
                                                .child("Presets"),
                                        )
                                        .children(preset_items),
                                )
                            })
                            // Recently Used / All Models section (if not empty)
                            .when(!model_items.is_empty(), |d| {
                                d.child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .w_full()
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .text_color(muted_fg)
                                                .px_3()
                                                .py_1()
                                                .child("Models"),
                                        )
                                        .children(model_items),
                                )
                            }),
                    )
                    // Footer with keyboard hint
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .border_t_1()
                            .border_color(border_color)
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(muted_fg)
                                    .child("↑↓ Navigate  ↵ Select  ⎋ Close"),
                            ),
                    ),
            )
    }
}
