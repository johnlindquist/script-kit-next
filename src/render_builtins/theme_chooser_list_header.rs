        // ── Pre-compute data for list closure ──────────────────────
        let preset_names: Vec<String> = presets.iter().map(|p| p.name.to_string()).collect();
        let preset_descs: Vec<String> = presets.iter().map(|p| p.description.to_string()).collect();
        let preset_is_dark: Vec<bool> = presets.iter().map(|p| p.is_dark).collect();
        let selected = selected_index;
        let hovered = self.hovered_index;
        let current_input_mode = self.input_mode;
        let orig_idx = original_index;
        let first_light_idx = first_light;
        let hover_bg = rgba((selection_bg << 8) | hover_alpha);
        let filtered_indices_for_list = filtered_indices.clone();
        let entity_handle_for_customize = entity_handle.clone();
        let hover_entity_handle = entity_handle.clone();

        // ── Theme list ─────────────────────────────────────────────
        let list = uniform_list(
            "theme-chooser",
            filtered_count,
            move |visible_range, _window, _cx| {
                visible_range
                    .map(|ix| {
                        let preset_idx = filtered_indices_for_list[ix];
                        let is_selected = ix == selected;
                        let is_hovered = !is_selected && hovered == Some(ix) && current_input_mode == InputMode::Mouse;
                        let is_original = preset_idx == orig_idx;
                        let name = &preset_names[preset_idx];
                        let desc = &preset_descs[preset_idx];
                        let is_dark = preset_is_dark[preset_idx];
                        let colors = &preview_colors[preset_idx];
                        let is_first_light = filter_is_empty
                            && preset_idx == first_light_idx
                            && first_light_idx > 0;

                        // Color swatches
                        let swatch = |color: u32| {
                            div()
                                .w(px(14.0))
                                .h(px(24.0))
                                .rounded(px(3.0))
                                .bg(rgb(color))
                        };
                        let palette = div()
                            .flex()
                            .flex_row()
                            .gap(px(2.0))
                            .mr(px(10.0))
                            .child(swatch(colors.bg))
                            .child(swatch(colors.accent))
                            .child(swatch(colors.text))
                            .child(swatch(colors.secondary))
                            .child(swatch(colors.border));

                        // Checkmark for original (saved) theme
                        let indicator = if is_original {
                            div()
                                .text_sm()
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(rgb(accent_color))
                                .w(px(16.0))
                                .child("✓")
                        } else {
                            div().w(px(16.0))
                        };

                        // Dark/light badge
                        let badge_text = if is_dark { "dark" } else { "light" };
                        let badge_border = rgba((ui_border << 8) | 0x40);
                        let badge = div()
                            .text_xs()
                            .text_color(rgb(text_dimmed))
                            .ml_auto()
                            .px(px(6.0))
                            .py(px(2.0))
                            .rounded(px(4.0))
                            .border_1()
                            .border_color(badge_border)
                            .child(badge_text.to_string());

                        let sel_bg = rgba((selection_bg << 8) | selected_alpha);
                        let border_rgba = rgba((ui_border << 8) | 0x30);

                        // Section label for light themes (only when unfiltered)
                        let section_label = if is_first_light {
                            Some(
                                div()
                                    .w_full()
                                    .pt(px(8.0))
                                    .pb(px(4.0))
                                    .px(px(16.0))
                                    .border_color(border_rgba)
                                    .border_t_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_dimmed))
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .child("LIGHT"),
                                    ),
                            )
                        } else {
                            None
                        };

                        let name_color = if is_selected {
                            accent_color
                        } else {
                            text_primary
                        };

                        // Click handler: select + preview via filtered index
                        let click_entity = entity_handle.clone();
                        let click_handler = move |_event: &gpui::ClickEvent,
                                                   _window: &mut Window,
                                                   cx: &mut gpui::App| {
                            if let Some(app) = click_entity.upgrade() {
                                app.update(cx, |this, cx| {
                                    // Recompute filtered indices from current filter
                                    let current_filter = if let AppView::ThemeChooserView {
                                        ref filter,
                                        ..
                                    } = this.current_view
                                    {
                                        filter.clone()
                                    } else {
                                        return;
                                    };
                                    let presets = theme::presets::all_presets();
                                    let filtered =
                                        Self::theme_chooser_filtered_indices(&current_filter);

                                    if let AppView::ThemeChooserView {
                                        ref mut selected_index,
                                        ..
                                    } = this.current_view
                                    {
                                        *selected_index = ix;
                                    }
                                    if let Some(&pidx) = filtered.get(ix) {
                                        if pidx < presets.len() {
                                            this.theme = std::sync::Arc::new(
                                                presets[pidx].create_theme(),
                                            );
                                            theme::sync_gpui_component_theme(cx);
                                            cx.notify();
                                        }
                                    }
                                });
                            }
                        };

                        // Hover handler for mouse tracking
                        let hover_entity = hover_entity_handle.clone();
                        let hover_handler = move |hov: &bool, _window: &mut Window, cx: &mut gpui::App| {
                            if let Some(app) = hover_entity.upgrade() {
                                app.update(cx, |this, cx| {
                                    if *hov {
                                        this.input_mode = InputMode::Mouse;
                                        if this.hovered_index != Some(ix) {
                                            this.hovered_index = Some(ix);
                                            cx.notify();
                                        }
                                    } else if this.hovered_index == Some(ix) {
                                        this.hovered_index = None;
                                        cx.notify();
                                    }
                                });
                            }
                        };

                        // Build item row
                        let is_mouse_mode = current_input_mode == InputMode::Mouse;
                        let row = div()
                            .id(ix)
                            .w_full()
                            .h(px(theme_item_height))
                            .px(px(12.0))
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(8.0))
                            .cursor_pointer()
                            .when(is_selected, |d| {
                                d.bg(sel_bg).border_l_2().border_color(rgb(accent_color))
                            })
                            .when(is_hovered, |d| d.bg(hover_bg))
                            .when(!is_selected && is_mouse_mode, |d| d.hover(move |s| s.bg(hover_bg)))
                            .on_click(click_handler)
                            .on_hover(hover_handler)
                            .child(indicator)
                            .child(palette)
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .overflow_hidden()
                                    .gap(px(1.0))
                                    .child(
                                        div()
                                            .text_sm()
                                            .when(is_original || is_selected, |d| {
                                                d.font_weight(gpui::FontWeight::SEMIBOLD)
                                            })
                                            .text_color(rgb(name_color))
                                            .child(name.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_secondary))
                                            .child(desc.clone()),
                                    ),
                            )
                            .child(badge);

                        if let Some(label) = section_label {
                            div()
                                .w_full()
                                .flex()
                                .flex_col()
                                .child(label)
                                .child(row)
                                .into_any_element()
                        } else {
                            row.into_any_element()
                        }
                    })
                    .collect()
            },
        )
        .h_full()
        .track_scroll(&self.theme_chooser_scroll_handle)
        .into_any_element();

        // ── Header with search input ───────────────────────────────
        let header = div()
            .w_full()
            .px(px(design_spacing.padding_lg))
            .pt(px(design_spacing.padding_md))
            .pb(px(4.0))
            .flex()
            .flex_col()
            .gap(px(6.0))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(text_primary))
                            .child("Themes"),
                    )
                    .child(
                        div().text_xs().text_color(rgb(text_dimmed)).child(format!(
                            "{} dark · {} light",
                            dark_count, light_count
                        )),
                    ),
            )
            // Search input
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .child(
                        div().flex_1().flex().flex_row().items_center().child(
                            Input::new(&self.gpui_input_state)
                                .w_full()
                                .h(px(28.))
                                .px(px(0.))
                                .py(px(0.))
                                .with_size(Size::Size(px(design_typography.font_size_xl)))
                                .appearance(false)
                                .bordered(false)
                                .focus_bordered(false),
                        ),
                    ),
            )
            // "DARK" section label only when unfiltered
            .when(filter_is_empty, |d| {
                d.child(
                    div()
                        .w_full()
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_dimmed))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .child("DARK"),
                        ),
                )
            });

