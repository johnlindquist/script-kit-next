        // ── Preview panel with customization controls ─────────────
        let border_rgba = rgba((ui_border << 8) | 0x40);
        let current_opacity_main = opacity.main;
        let vibrancy_enabled = self
            .theme
            .vibrancy
            .as_ref()
            .map(|v| v.enabled)
            .unwrap_or(true);

        // Build accent color swatches (clickable)
        let accent_swatches: Vec<gpui::AnyElement> = Self::ACCENT_PALETTE
            .iter()
            .enumerate()
            .map(|(i, &(color, _name))| {
                let is_current = color == accent_color;
                let click_entity = entity_handle_for_customize.clone();
                let swatch_bg_main = bg_main;
                div()
                    .id(ElementId::NamedInteger("accent-swatch".into(), i as u64))
                    .w(px(20.0))
                    .h(px(20.0))
                    .rounded(px(10.0))
                    .bg(rgb(color))
                    .cursor_pointer()
                    .when(is_current, |d| d.border_2().border_color(rgb(text_primary)))
                    .when(!is_current, |d| {
                        d.border_1()
                            .border_color(border_rgba)
                            .hover(move |s| s.border_color(rgb(text_secondary)))
                    })
                    .on_click(
                        move |_event: &gpui::ClickEvent,
                              _window: &mut Window,
                              cx: &mut gpui::App| {
                            if let Some(app) = click_entity.upgrade() {
                                app.update(cx, |this, cx| {
                                    let mut modified = (*this.theme).clone();
                                    modified.colors.accent.selected = color;
                                    modified.colors.text.on_accent =
                                        Self::accent_on_text_color(color, swatch_bg_main);
                                    this.theme = std::sync::Arc::new(modified);
                                    theme::sync_gpui_component_theme(cx);
                                    cx.notify();
                                });
                            }
                        },
                    )
                    .into_any_element()
            })
            .collect();

        // Build opacity preset buttons (clickable)
        let opacity_buttons: Vec<gpui::AnyElement> = Self::OPACITY_PRESETS
            .iter()
            .enumerate()
            .map(|(i, &(value, label))| {
                let is_current = (value - current_opacity_main).abs() < 0.05;
                let click_entity = entity_handle_for_customize.clone();
                div()
                    .id(ElementId::NamedInteger("opacity-btn".into(), i as u64))
                    .px(px(8.0))
                    .py(px(3.0))
                    .rounded(px(4.0))
                    .cursor_pointer()
                    .text_xs()
                    .when(is_current, |d| {
                        d.bg(rgb(accent_color))
                            .text_color(rgb(text_on_accent))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                    })
                    .when(!is_current, |d| {
                        d.border_1()
                            .border_color(border_rgba)
                            .text_color(rgb(text_secondary))
                            .hover(move |s| s.bg(rgba((selection_bg << 8) | hover_alpha)))
                    })
                    .on_click(
                        move |_event: &gpui::ClickEvent,
                              _window: &mut Window,
                              cx: &mut gpui::App| {
                            if let Some(app) = click_entity.upgrade() {
                                app.update(cx, |this, cx| {
                                    let mut modified = (*this.theme).clone();
                                    if let Some(ref mut op) = modified.opacity {
                                        op.main = value;
                                        op.title_bar = value;
                                    }
                                    this.theme = std::sync::Arc::new(modified);
                                    theme::sync_gpui_component_theme(cx);
                                    cx.notify();
                                });
                            }
                        },
                    )
                    .child(label.to_string())
                    .into_any_element()
            })
            .collect();

        // Build vibrancy toggle (clickable)
        let vibrancy_entity = entity_handle_for_customize.clone();
        let vibrancy_toggle = div()
            .id("vibrancy-toggle")
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.0))
            .cursor_pointer()
            .hover(move |s| s.bg(rgba((selection_bg << 8) | hover_alpha)))
            .rounded(px(4.0))
            .px(px(4.0))
            .py(px(2.0))
            .on_click(
                move |_event: &gpui::ClickEvent,
                      _window: &mut Window,
                      cx: &mut gpui::App| {
                    if let Some(app) = vibrancy_entity.upgrade() {
                        app.update(cx, |this, cx| {
                            let mut modified = (*this.theme).clone();
                            if let Some(ref mut vibrancy) = modified.vibrancy {
                                vibrancy.enabled = !vibrancy.enabled;
                            }
                            this.theme = std::sync::Arc::new(modified);
                            theme::sync_gpui_component_theme(cx);
                            cx.notify();
                        });
                    }
                },
            )
            .child(
                div()
                    .w(px(28.0))
                    .h(px(14.0))
                    .rounded(px(7.0))
                    .when(vibrancy_enabled, |d| d.bg(rgb(accent_color)))
                    .when(!vibrancy_enabled, |d| {
                        d.bg(rgba((ui_border << 8) | 0x80))
                    })
                    .flex()
                    .items_center()
                    .child(
                        div()
                            .w(px(10.0))
                            .h(px(10.0))
                            .rounded(px(5.0))
                            .bg(rgb(0xffffff))
                            .when(vibrancy_enabled, |d| d.ml(px(16.0)))
                            .when(!vibrancy_enabled, |d| d.ml(px(2.0))),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(text_secondary))
                    .child(if vibrancy_enabled { "On" } else { "Off" }),
            );

        // Build vibrancy material buttons (clickable, only shown when vibrancy enabled)
        let current_material = self
            .theme
            .vibrancy
            .as_ref()
            .map(|v| v.material)
            .unwrap_or_default();
        let material_buttons: Vec<gpui::AnyElement> = Self::VIBRANCY_MATERIALS
            .iter()
            .enumerate()
            .map(|(i, &(material, label))| {
                let is_current = material == current_material;
                let click_entity = entity_handle_for_customize.clone();
                div()
                    .id(ElementId::NamedInteger("material-btn".into(), i as u64))
                    .px(px(6.0))
                    .py(px(2.0))
                    .rounded(px(4.0))
                    .cursor_pointer()
                    .text_xs()
                    .when(is_current, |d| {
                        d.bg(rgb(accent_color))
                            .text_color(rgb(text_on_accent))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                    })
                    .when(!is_current, |d| {
                        d.border_1()
                            .border_color(border_rgba)
                            .text_color(rgb(text_secondary))
                            .hover(move |s| s.bg(rgba((selection_bg << 8) | hover_alpha)))
                    })
                    .on_click(
                        move |_event: &gpui::ClickEvent,
                              _window: &mut Window,
                              cx: &mut gpui::App| {
                            if let Some(app) = click_entity.upgrade() {
                                app.update(cx, |this, cx| {
                                    let mut modified = (*this.theme).clone();
                                    if let Some(ref mut vibrancy) = modified.vibrancy {
                                        vibrancy.material = material;
                                    }
                                    this.theme = std::sync::Arc::new(modified);
                                    theme::sync_gpui_component_theme(cx);
                                    cx.notify();
                                });
                            }
                        },
                    )
                    .child(label.to_string())
                    .into_any_element()
            })
            .collect();

        // Build font size preset buttons (clickable)
        let current_ui_font_size = self.theme.get_fonts().ui_size;
        let font_size_buttons: Vec<gpui::AnyElement> = Self::FONT_SIZE_PRESETS
            .iter()
            .enumerate()
            .map(|(i, &(size, label))| {
                let is_current = (size - current_ui_font_size).abs() < 0.5;
                let click_entity = entity_handle_for_customize.clone();
                div()
                    .id(ElementId::NamedInteger("fontsize-btn".into(), i as u64))
                    .px(px(8.0))
                    .py(px(2.0))
                    .rounded(px(4.0))
                    .cursor_pointer()
                    .text_xs()
                    .when(is_current, |d| {
                        d.bg(rgb(accent_color))
                            .text_color(rgb(text_on_accent))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                    })
                    .when(!is_current, |d| {
                        d.border_1()
                            .border_color(border_rgba)
                            .text_color(rgb(text_secondary))
                            .hover(move |s| s.bg(rgba((selection_bg << 8) | hover_alpha)))
                    })
                    .on_click(
                        move |_event: &gpui::ClickEvent,
                              _window: &mut Window,
                              cx: &mut gpui::App| {
                            if let Some(app) = click_entity.upgrade() {
                                app.update(cx, |this, cx| {
                                    let mut modified = (*this.theme).clone();
                                    if let Some(ref mut fonts) = modified.fonts {
                                        fonts.ui_size = size;
                                    } else {
                                        modified.fonts = Some(theme::FontConfig {
                                            ui_size: size,
                                            ..Default::default()
                                        });
                                    }
                                    this.theme = std::sync::Arc::new(modified);
                                    theme::sync_gpui_component_theme(cx);
                                    cx.notify();
                                });
                            }
                        },
                    )
                    .child(label.to_string())
                    .into_any_element()
            })
            .collect();

        // Build reset button (clickable)
        let reset_entity = entity_handle_for_customize.clone();
        let reset_button = div()
            .id("reset-to-preset")
            .px(px(10.0))
            .py(px(4.0))
            .rounded(px(4.0))
            .cursor_pointer()
            .text_xs()
            .border_1()
            .border_color(border_rgba)
            .text_color(rgb(text_secondary))
            .hover(move |s| s.bg(rgba((selection_bg << 8) | hover_alpha)))
            .on_click(
                move |_event: &gpui::ClickEvent,
                      _window: &mut Window,
                      cx: &mut gpui::App| {
                    if let Some(app) = reset_entity.upgrade() {
                        app.update(cx, |this, cx| {
                            let current_filter =
                                if let AppView::ThemeChooserView { ref filter, .. } =
                                    this.current_view
                                {
                                    filter.clone()
                                } else {
                                    return;
                                };
                            let presets = theme::presets::all_presets();
                            let filtered =
                                Self::theme_chooser_filtered_indices(&current_filter);
                            if let AppView::ThemeChooserView {
                                ref selected_index, ..
                            } = this.current_view
                            {
                                if let Some(&pidx) = filtered.get(*selected_index) {
                                    if pidx < presets.len() {
                                        this.theme =
                                            std::sync::Arc::new(presets[pidx].create_theme());
                                        theme::sync_gpui_component_theme(cx);
                                        cx.notify();
                                    }
                                }
                            }
                        });
                    }
                },
            )
            .child("Reset to Defaults");

        let accent_name = Self::accent_color_name(accent_color);

