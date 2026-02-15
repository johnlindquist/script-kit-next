        let tokens = get_tokens(self.current_design);
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();
        let text_primary = self.theme.colors.text.primary;
        let text_dimmed = self.theme.colors.text.dimmed;
        let text_secondary = self.theme.colors.text.secondary;
        let text_muted = self.theme.colors.text.muted;
        let accent_color = self.theme.colors.accent.selected;
        let ui_border = self.theme.colors.ui.border;
        let selection_bg = self.theme.colors.accent.selected_subtle;
        let bg_main = self.theme.colors.background.main;
        let bg_search_box = self.theme.colors.background.search_box;
        let text_on_accent = self.theme.colors.text.on_accent;
        let ui_success = self.theme.colors.ui.success;
        let ui_error = self.theme.colors.ui.error;
        let ui_warning = self.theme.colors.ui.warning;
        let ui_info = self.theme.colors.ui.info;
        let opacity = self.theme.get_opacity();
        let selected_alpha = (opacity.selected * 255.0) as u32;
        let hover_alpha = (opacity.hover * 255.0).max(18.0) as u32;
        let presets = theme::presets::presets_cached();
        let preview_colors = theme::presets::preset_preview_colors_cached();
        let first_light = theme::presets::first_light_theme_index();
        let original_index = self
            .theme_before_chooser
            .as_ref()
            .map(|t| theme::presets::find_current_preset_index(t))
            .unwrap_or(0);

        // Filter presets by name or description
        let filtered_indices = Self::theme_chooser_filtered_indices(filter);
        let filtered_count = filtered_indices.len();
        let filter_is_empty = filter.is_empty();

        // Count dark/light in filtered results
        let dark_count = filtered_indices
            .iter()
            .filter(|&&i| presets[i].is_dark)
            .count();
        let light_count = filtered_count - dark_count;

        // Terminal colors for preview panel
        let terminal = &self.theme.colors.terminal;
        let term_colors: Vec<u32> = vec![
            terminal.red,
            terminal.green,
            terminal.yellow,
            terminal.blue,
            terminal.magenta,
            terminal.cyan,
            terminal.white,
            terminal.black,
        ];
        let term_bright: Vec<u32> = vec![
            terminal.bright_red,
            terminal.bright_green,
            terminal.bright_yellow,
            terminal.bright_blue,
            terminal.bright_magenta,
            terminal.bright_cyan,
            terminal.bright_white,
            terminal.bright_black,
        ];

        let theme_item_height: f32 = 48.0;
        let entity_handle = cx.entity().downgrade();

        // ── Keyboard handler ───────────────────────────────────────
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                this.hide_mouse_cursor(cx);
                let key_str = event.keystroke.key.to_lowercase();
                let has_cmd = event.keystroke.modifiers.platform;

                // Escape: clear filter first if present, otherwise restore original and close
                if key_str == "escape" && !this.show_actions_popup {
                    if !this.clear_builtin_view_filter(cx) {
                        // No filter to clear — restore original theme and go back
                        if let Some(original) = this.theme_before_chooser.take() {
                            this.theme = original;
                            theme::sync_gpui_component_theme(cx);
                        }
                        this.go_back_or_close(window, cx);
                    }
                    return;
                }
                // Cmd+W: restore and close window
                if has_cmd && key_str == "w" {
                    if let Some(original) = this.theme_before_chooser.take() {
                        this.theme = original;
                        theme::sync_gpui_component_theme(cx);
                    }
                    this.close_and_reset_window(cx);
                    return;
                }
                // Cmd+[ / Cmd+]: cycle accent colors
                if has_cmd && (key_str == "[" || key_str == "bracketleft") {
                    let current = this.theme.colors.accent.selected;
                    let idx = Self::find_accent_palette_index(current).unwrap_or(0);
                    let new_idx = if idx == 0 {
                        Self::ACCENT_PALETTE.len() - 1
                    } else {
                        idx - 1
                    };
                    let (new_accent, _) = Self::ACCENT_PALETTE[new_idx];
                    let mut modified = (*this.theme).clone();
                    modified.colors.accent.selected = new_accent;
                    modified.colors.text.on_accent =
                        Self::accent_on_text_color(new_accent, modified.colors.background.main);
                    this.theme = std::sync::Arc::new(modified);
                    theme::sync_gpui_component_theme(cx);
                    cx.notify();
                    return;
                }
                if has_cmd && (key_str == "]" || key_str == "bracketright") {
                    let current = this.theme.colors.accent.selected;
                    let idx = Self::find_accent_palette_index(current).unwrap_or(0);
                    let new_idx = (idx + 1) % Self::ACCENT_PALETTE.len();
                    let (new_accent, _) = Self::ACCENT_PALETTE[new_idx];
                    let mut modified = (*this.theme).clone();
                    modified.colors.accent.selected = new_accent;
                    modified.colors.text.on_accent =
                        Self::accent_on_text_color(new_accent, modified.colors.background.main);
                    this.theme = std::sync::Arc::new(modified);
                    theme::sync_gpui_component_theme(cx);
                    cx.notify();
                    return;
                }
                // Cmd+- / Cmd+=: adjust opacity
                if has_cmd && key_str == "-" {
                    let current_main = this.theme.get_opacity().main;
                    let idx = Self::find_opacity_preset_index(current_main);
                    if idx > 0 {
                        let target = Self::OPACITY_PRESETS[idx - 1].0;
                        let mut modified = (*this.theme).clone();
                        if let Some(ref mut op) = modified.opacity {
                            op.main = target;
                            op.title_bar = target;
                        }
                        this.theme = std::sync::Arc::new(modified);
                        theme::sync_gpui_component_theme(cx);
                        cx.notify();
                    }
                    return;
                }
                if has_cmd && (key_str == "=" || key_str == "+") {
                    let current_main = this.theme.get_opacity().main;
                    let idx = Self::find_opacity_preset_index(current_main);
                    if idx < Self::OPACITY_PRESETS.len() - 1 {
                        let target = Self::OPACITY_PRESETS[idx + 1].0;
                        let mut modified = (*this.theme).clone();
                        if let Some(ref mut op) = modified.opacity {
                            op.main = target;
                            op.title_bar = target;
                        }
                        this.theme = std::sync::Arc::new(modified);
                        theme::sync_gpui_component_theme(cx);
                        cx.notify();
                    }
                    return;
                }
                // Cmd+B: toggle vibrancy
                if has_cmd && key_str == "b" {
                    let mut modified = (*this.theme).clone();
                    if let Some(ref mut vibrancy) = modified.vibrancy {
                        vibrancy.enabled = !vibrancy.enabled;
                    }
                    this.theme = std::sync::Arc::new(modified);
                    theme::sync_gpui_component_theme(cx);
                    cx.notify();
                    return;
                }
                // Cmd+M: cycle vibrancy material
                if has_cmd && key_str == "m" {
                    let current_material = this
                        .theme
                        .vibrancy
                        .as_ref()
                        .map(|v| v.material)
                        .unwrap_or_default();
                    let idx = Self::find_vibrancy_material_index(current_material);
                    let new_idx = (idx + 1) % Self::VIBRANCY_MATERIALS.len();
                    let (new_material, _) = Self::VIBRANCY_MATERIALS[new_idx];
                    let mut modified = (*this.theme).clone();
                    if let Some(ref mut vibrancy) = modified.vibrancy {
                        vibrancy.material = new_material;
                    }
                    this.theme = std::sync::Arc::new(modified);
                    theme::sync_gpui_component_theme(cx);
                    cx.notify();
                    return;
                }
                // Cmd+R: reset customizations to selected preset defaults
                if has_cmd && key_str == "r" {
                    let current_filter =
                        if let AppView::ThemeChooserView { ref filter, .. } = this.current_view {
                            filter.clone()
                        } else {
                            return;
                        };
                    let presets = theme::presets::presets_cached();
                    let filtered = Self::theme_chooser_filtered_indices(&current_filter);
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
                    return;
                }
                // Enter: apply and close
                if key_str == "enter" {
                    this.theme_before_chooser = None;
                    if let Err(e) = theme::presets::write_theme_to_disk(&this.theme) {
                        logging::log("ERROR", &format!("Failed to save theme: {}", e));
                    }
                    theme::sync_gpui_component_theme(cx);
                    this.go_back_or_close(window, cx);
                    return;
                }

                // Compute filtered indices from current filter
                let current_filter =
                    if let AppView::ThemeChooserView { ref filter, .. } = this.current_view {
                        filter.clone()
                    } else {
                        return;
                    };
                let presets = theme::presets::presets_cached();
                let filtered = Self::theme_chooser_filtered_indices(&current_filter);
                let count = filtered.len();
                if count == 0 {
                    return;
                }

                if let AppView::ThemeChooserView {
                    ref mut selected_index,
                    ..
                } = this.current_view
                {
                    let page_size: usize = 5;
                    match key_str.as_str() {
                        "up" | "arrowup" => {
                            if *selected_index > 0 {
                                *selected_index -= 1;
                            }
                        }
                        "down" | "arrowdown" => {
                            if *selected_index < count - 1 {
                                *selected_index += 1;
                            }
                        }
                        "home" => {
                            *selected_index = 0;
                        }
                        "end" => {
                            *selected_index = count - 1;
                        }
                        "pageup" => {
                            *selected_index = selected_index.saturating_sub(page_size);
                        }
                        "pagedown" => {
                            *selected_index = (*selected_index + page_size).min(count - 1);
                        }
                        _ => return,
                    }
                    // Map to actual preset index and apply theme
                    let preset_idx = filtered[*selected_index];
                    let new_theme = std::sync::Arc::new(presets[preset_idx].create_theme());
                    this.theme = new_theme;
                    this.theme_chooser_scroll_handle
                        .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                    theme::sync_gpui_component_theme(cx);
                    cx.notify();
                }
            },
        );

