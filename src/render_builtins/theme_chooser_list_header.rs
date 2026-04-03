        // ── Pre-compute data for list closure ──────────────────────
        let preset_names: Vec<String> = presets.iter().map(|p| p.name.to_string()).collect();
        let preset_descs: Vec<String> = presets.iter().map(|p| p.description.to_string()).collect();
        let selected = selected_index;
        let orig_idx = original_index;
        let first_light_idx = first_light;
        let hover_bg = rgba((selection_bg << 8) | hover_alpha);
        let filtered_indices_for_list = filtered_indices.clone();
        let entity_handle_for_customize = entity_handle.clone();

        // ── Theme list ─────────────────────────────────────────────
        let list = uniform_list(
            "theme-chooser",
            filtered_count,
            move |visible_range, _window, _cx| {
                visible_range
                    .map(|ix| {
                        let preset_idx = filtered_indices_for_list[ix];
                        let is_selected = ix == selected;
                        let is_original = preset_idx == orig_idx;
                        let name = &preset_names[preset_idx];
                        let desc = &preset_descs[preset_idx];
                        let colors = &preview_colors[preset_idx];
                        let is_first_light = filter_is_empty
                            && preset_idx == first_light_idx
                            && first_light_idx > 0;

                        // Compact color bar — thin horizontal strip showing theme palette
                        let color_bar = div()
                            .flex()
                            .flex_row()
                            .w(px(40.0))
                            .h(px(8.0))
                            .rounded(px(4.0))
                            .overflow_hidden()
                            .mr(px(8.0))
                            .child(div().flex_1().bg(rgb(colors.bg)))
                            .child(div().flex_1().bg(rgb(colors.accent)))
                            .child(div().flex_1().bg(rgb(colors.text)))
                            .child(div().flex_1().bg(rgb(colors.secondary)))
                            .child(div().flex_1().bg(rgb(colors.border)));

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

                        // (dark/light badge removed — section headers provide this info)

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
                                    let presets = theme::presets::presets_cached();
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
                                            let preset = &presets[pidx];

                                            tracing::debug!(
                                                target: "script_kit::theme_chooser",
                                                event = "theme_chooser_select",
                                                trigger = "mouse",
                                                filtered_index = ix,
                                                preset_index = pidx,
                                                preset_id = %preset.id,
                                                preset_name = %preset.name,
                                                filtered_count = filtered.len(),
                                            );

                                            this.theme_chooser_scroll_handle
                                                .scroll_to_item(ix, ScrollStrategy::Nearest);
                                            this.theme = std::sync::Arc::new(
                                                preset.create_theme(),
                                            );
                                            theme::sync_gpui_component_theme(cx);
                                            cx.notify();
                                        }
                                    }
                                });
                            }
                        };

                        // Build item row
                        let text_col = div()
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
                            // Description only revealed on focused row
                            .when(is_selected, |d| {
                                d.child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_secondary))
                                        .child(desc.clone()),
                                )
                            });

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
                            .when(!is_selected, |d| d.hover(move |s| s.bg(hover_bg)))
                            .on_click(click_handler)
                            .child(indicator)
                            .child(color_bar)
                            .child(text_col);

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

