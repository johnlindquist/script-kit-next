        use crate::ui_foundation::{is_key_down, is_key_enter, is_key_escape, is_key_up};

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
                let key = event.keystroke.key.as_str();
                let has_cmd = event.keystroke.modifiers.platform;

                // Escape: clear filter first if present, otherwise restore original and close
                if is_key_escape(key) && !this.show_actions_popup {
                    if !this.clear_builtin_view_filter(cx) {
                        // No filter to clear — restore original theme and go back
                        if let Some(original) = this.theme_before_chooser.take() {
                            this.restore_theme_chooser_theme(
                                original,
                                "theme_chooser_restore_escape",
                                cx,
                            );
                        }
                        this.go_back_or_close(window, cx);
                    }
                    return;
                }
                // Cmd+W: restore and close window
                if has_cmd && key.eq_ignore_ascii_case("w") {
                    if let Some(original) = this.theme_before_chooser.take() {
                        this.restore_theme_chooser_theme(
                            original,
                            "theme_chooser_restore_cmd_w",
                            cx,
                        );
                    }
                    this.close_and_reset_window(cx);
                    return;
                }
                // Cmd+[ / Cmd+]: cycle accent colors
                if has_cmd && (key == "[" || key.eq_ignore_ascii_case("bracketleft")) {
                    let current = this.theme.colors.accent.selected;
                    let idx = Self::find_accent_palette_index(current).unwrap_or(0);
                    let new_idx = if idx == 0 {
                        Self::ACCENT_PALETTE.len() - 1
                    } else {
                        idx - 1
                    };
                    let (new_accent, _) = Self::ACCENT_PALETTE[new_idx];
                    this.mutate_theme_chooser_theme(
                        "theme_chooser_cycle_accent_prev",
                        cx,
                        |theme| {
                            theme.colors.accent.selected = new_accent;
                            theme.colors.text.on_accent = Self::accent_on_text_color(
                                new_accent,
                                theme.colors.background.main,
                            );
                        },
                    );
                    return;
                }
                if has_cmd && (key == "]" || key.eq_ignore_ascii_case("bracketright")) {
                    let current = this.theme.colors.accent.selected;
                    let idx = Self::find_accent_palette_index(current).unwrap_or(0);
                    let new_idx = (idx + 1) % Self::ACCENT_PALETTE.len();
                    let (new_accent, _) = Self::ACCENT_PALETTE[new_idx];
                    this.mutate_theme_chooser_theme(
                        "theme_chooser_cycle_accent_next",
                        cx,
                        |theme| {
                            theme.colors.accent.selected = new_accent;
                            theme.colors.text.on_accent = Self::accent_on_text_color(
                                new_accent,
                                theme.colors.background.main,
                            );
                        },
                    );
                    return;
                }
                // Cmd+- / Cmd+=: adjust opacity (whole-shell preset)
                if has_cmd && key == "-" {
                    let current_main = this.theme.get_opacity().main;
                    let idx = Self::find_opacity_preset_index(current_main);
                    if idx > 0 {
                        let target = Self::OPACITY_PRESETS[idx - 1].0;
                        let modified =
                            Self::apply_surface_opacity_preset(this.theme.as_ref(), target);
                        this.apply_theme_chooser_theme(
                            modified,
                            "theme_chooser_opacity_prev",
                            cx,
                        );
                    }
                    return;
                }
                if has_cmd && (key == "=" || key == "+") {
                    let current_main = this.theme.get_opacity().main;
                    let idx = Self::find_opacity_preset_index(current_main);
                    if idx < Self::OPACITY_PRESETS.len() - 1 {
                        let target = Self::OPACITY_PRESETS[idx + 1].0;
                        let modified =
                            Self::apply_surface_opacity_preset(this.theme.as_ref(), target);
                        this.apply_theme_chooser_theme(
                            modified,
                            "theme_chooser_opacity_next",
                            cx,
                        );
                    }
                    return;
                }
                // Cmd+B: toggle vibrancy
                if has_cmd && key.eq_ignore_ascii_case("b") {
                    this.mutate_theme_chooser_theme(
                        "theme_chooser_toggle_vibrancy",
                        cx,
                        |theme| {
                            let mut vibrancy = theme.get_vibrancy();
                            vibrancy.enabled = !vibrancy.enabled;
                            theme.vibrancy = Some(vibrancy);
                        },
                    );
                    return;
                }
                // Cmd+M: cycle vibrancy material
                if has_cmd && key.eq_ignore_ascii_case("m") {
                    let current_material = this.theme.get_vibrancy().material;
                    let idx = Self::find_vibrancy_material_index(current_material);
                    let new_idx = (idx + 1) % Self::VIBRANCY_MATERIALS.len();
                    let (new_material, _) = Self::VIBRANCY_MATERIALS[new_idx];
                    this.mutate_theme_chooser_theme(
                        "theme_chooser_cycle_material",
                        cx,
                        |theme| {
                            let mut vibrancy = theme.get_vibrancy();
                            vibrancy.enabled = true;
                            vibrancy.material = new_material;
                            theme.vibrancy = Some(vibrancy);
                        },
                    );
                    return;
                }
                // Cmd+J: surprise me / remix
                if has_cmd && key.eq_ignore_ascii_case("j") {
                    let remixed = Self::build_theme_chooser_remix(
                        this.theme.as_ref(),
                        theme_chooser_remix_seed(),
                    );
                    this.apply_theme_chooser_theme(remixed, "theme_chooser_remix", cx);
                    return;
                }
                // Cmd+R: reset customizations to selected preset defaults
                if has_cmd && key.eq_ignore_ascii_case("r") {
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
                                this.apply_theme_chooser_theme(
                                    presets[pidx].create_theme(),
                                    "theme_chooser_reset_preset",
                                    cx,
                                );
                            }
                        }
                    }
                    return;
                }
                // Enter: apply and close
                if is_key_enter(key) {
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
                    match key {
                        _ if is_key_up(key) => {
                            if *selected_index > 0 {
                                *selected_index -= 1;
                            }
                        }
                        _ if is_key_down(key) => {
                            if *selected_index < count - 1 {
                                *selected_index += 1;
                            }
                        }
                        _ if key.eq_ignore_ascii_case("home") => {
                            *selected_index = 0;
                        }
                        _ if key.eq_ignore_ascii_case("end") => {
                            *selected_index = count - 1;
                        }
                        _ if key.eq_ignore_ascii_case("pageup") => {
                            *selected_index = selected_index.saturating_sub(page_size);
                        }
                        _ if key.eq_ignore_ascii_case("pagedown") => {
                            *selected_index = (*selected_index + page_size).min(count - 1);
                        }
                        _ => return,
                    }
                    // Map to actual preset index and apply theme via pipeline
                    this.preview_theme_chooser_preset(
                        &filtered,
                        *selected_index,
                        "theme_chooser_select_keyboard",
                        cx,
                    );
                }
            },
        );
