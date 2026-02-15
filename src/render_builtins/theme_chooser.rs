const ALPHA_BADGE_BORDER: u32 = 0x40;
const ALPHA_FOOTER_BORDER: u32 = 0x30;
const ALPHA_TOGGLE_BG: u32 = 0x80;
const MAX_ALPHA: u32 = 0xFF;
const SELECTION_HOVER_GAP: u32 = 16;
const HOVER_SELECTION_GAP: u32 = 8;
const MIN_HOVER_ALPHA: f32 = 18.0;
const THEME_LIST_PAGE_SIZE: usize = 5;
const OPACITY_MATCH_TOLERANCE: f32 = 0.05;
const FONT_SIZE_MATCH_TOLERANCE: f32 = 0.5;

impl ScriptListApp {
    fn contains_ascii_case_insensitive(haystack: &str, needle: &str) -> bool {
        if needle.is_empty() {
            return true;
        }

        let haystack = haystack.as_bytes();
        let needle = needle.as_bytes();
        if needle.len() > haystack.len() {
            return false;
        }

        haystack
            .windows(needle.len())
            .any(|window| window.eq_ignore_ascii_case(needle))
    }

    /// Helper: compute filtered preset indices from a filter string
    fn theme_chooser_filtered_indices(filter: &str) -> Vec<usize> {
        let presets = theme::presets::presets_cached();
        if filter.is_empty() {
            (0..presets.len()).collect()
        } else if filter.is_ascii() {
            presets
                .iter()
                .enumerate()
                .filter(|(_, p)| {
                    Self::contains_ascii_case_insensitive(p.name, filter)
                        || Self::contains_ascii_case_insensitive(p.description, filter)
                })
                .map(|(i, _)| i)
                .collect()
        } else {
            let f = filter.to_lowercase();
            presets
                .iter()
                .enumerate()
                .filter(|(_, p)| {
                    p.name.to_lowercase().contains(&f) || p.description.to_lowercase().contains(&f)
                })
                .map(|(i, _)| i)
                .collect()
        }
    }

    /// Accent color palette for theme customization
    const ACCENT_PALETTE: &'static [(u32, &'static str)] = theme::ACCENT_PALETTE;

    /// Opacity presets for quick selection
    const OPACITY_PRESETS: &'static [(f32, &'static str)] = &[
        (0.10, "10%"),
        (0.30, "30%"),
        (0.50, "50%"),
        (0.80, "80%"),
        (1.00, "100%"),
    ];

    /// Compute on-accent text color based on accent luminance
    fn accent_on_text_color(accent: u32, bg_main: u32) -> u32 {
        let contrast_ratio = |a: f32, b: f32| {
            let (lighter, darker) = if a >= b { (a, b) } else { (b, a) };
            (lighter + 0.05) / (darker + 0.05)
        };

        let accent_luminance = theme::relative_luminance_srgb(accent);
        let bg_luminance = theme::relative_luminance_srgb(bg_main);
        let white_luminance = theme::relative_luminance_srgb(0xFFFFFF);

        let contrast_with_background = contrast_ratio(accent_luminance, bg_luminance);
        let contrast_with_white = contrast_ratio(accent_luminance, white_luminance);

        if contrast_with_background >= contrast_with_white {
            bg_main
        } else {
            0xFFFFFF
        }
    }

    /// Find the closest accent palette index for a given accent color
    fn find_accent_palette_index(accent: u32) -> Option<usize> {
        Self::ACCENT_PALETTE.iter().position(|&(c, _)| c == accent)
    }

    /// Find the closest opacity preset index for a given opacity value
    fn find_opacity_preset_index(opacity: f32) -> usize {
        Self::OPACITY_PRESETS
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                (a.0 - opacity)
                    .abs()
                    .partial_cmp(&(b.0 - opacity).abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// Keep selected/active alpha visually stronger than hover alpha.
    fn resolve_interactive_alphas(selected_alpha: u32, hover_alpha: u32) -> (u32, u32) {
        let mut selected = selected_alpha.min(MAX_ALPHA);
        let mut hover = hover_alpha.min(MAX_ALPHA);

        if selected <= hover {
            selected = (hover + SELECTION_HOVER_GAP).min(MAX_ALPHA);
        }

        if hover >= selected {
            hover = selected.saturating_sub(HOVER_SELECTION_GAP);
        }

        (selected, hover)
    }

    fn interactive_state_backgrounds(
        base_color: u32,
        selected_alpha: u32,
        hover_alpha: u32,
    ) -> (gpui::Rgba, gpui::Rgba) {
        let (selected_alpha, hover_alpha) =
            Self::resolve_interactive_alphas(selected_alpha, hover_alpha);
        (
            rgba((base_color << 8) | selected_alpha),
            rgba((base_color << 8) | hover_alpha),
        )
    }

    /// Render the theme chooser with search, live preview, and preview panel
    pub(crate) fn render_theme_chooser(
        &mut self,
        filter: &str,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
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
        let selected_alpha = (opacity.selected * MAX_ALPHA as f32) as u32;
        let hover_alpha = (opacity.hover * MAX_ALPHA as f32).max(MIN_HOVER_ALPHA) as u32;
        let presets = theme::presets::presets_cached();
        let preview_colors = theme::presets::preset_preview_colors_cached();
        let first_light = theme::presets::first_light_theme_index();
        let original_index = self
            .theme_before_chooser
            .as_ref()
            .map(|t| theme::presets::find_current_preset_index(t))
            .unwrap_or(0);
        let (theme_row_selected_bg, theme_row_hover_bg) =
            Self::interactive_state_backgrounds(selection_bg, selected_alpha, hover_alpha);

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
        let term_colors = [
            terminal.red,
            terminal.green,
            terminal.yellow,
            terminal.blue,
            terminal.magenta,
            terminal.cyan,
            terminal.white,
            terminal.black,
        ];
        let term_bright = [
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
                                this.theme = std::sync::Arc::new(presets[pidx].create_theme());
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
                    let page_size: usize = THEME_LIST_PAGE_SIZE;
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

        // ── Pre-compute data for list closure ──────────────────────
        let presets_for_list = presets;
        let selected = selected_index;
        let hovered = self.hovered_index;
        let current_input_mode = self.input_mode;
        let orig_idx = original_index;
        let first_light_idx = first_light;
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
                        let is_hovered = !is_selected
                            && hovered == Some(ix)
                            && current_input_mode == InputMode::Mouse;
                        let is_original = preset_idx == orig_idx;
                        let preset = &presets_for_list[preset_idx];
                        let name = preset.name;
                        let desc = preset.description;
                        let is_dark = preset.is_dark;
                        let colors = &preview_colors[preset_idx];
                        let is_first_light =
                            filter_is_empty && preset_idx == first_light_idx && first_light_idx > 0;

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
                        let badge_border = rgba((ui_border << 8) | ALPHA_BADGE_BORDER);
                        let badge = div()
                            .text_xs()
                            .text_color(rgb(text_dimmed))
                            .ml_auto()
                            .px(px(6.0))
                            .py(px(2.0))
                            .rounded(px(4.0))
                            .border_1()
                            .border_color(badge_border)
                            .child(badge_text);

                        let border_rgba = rgba((ui_border << 8) | ALPHA_FOOTER_BORDER);

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
                        let click_handler =
                            move |_event: &gpui::ClickEvent,
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
                        let hover_handler =
                            move |hov: &bool, _window: &mut Window, cx: &mut gpui::App| {
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
                                d.bg(theme_row_selected_bg)
                                    .border_l_2()
                                    .border_color(rgb(accent_color))
                            })
                            .when(is_hovered, |d| d.bg(theme_row_hover_bg))
                            .when(!is_selected && is_mouse_mode, |d| {
                                d.hover(move |s| s.bg(theme_row_hover_bg))
                            })
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
                                            .child(name),
                                    )
                                    .child(
                                        div().text_xs().text_color(rgb(text_secondary)).child(desc),
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
                        div()
                            .text_xs()
                            .text_color(rgb(text_dimmed))
                            .child(format!("{} dark · {} light", dark_count, light_count)),
                    ),
            )
            // Search input
            .child(
                div().flex().flex_row().items_center().child(
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
                    div().w_full().child(
                        div()
                            .text_xs()
                            .text_color(rgb(text_dimmed))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("DARK"),
                    ),
                )
            });

        // ── Preview panel with customization controls ─────────────
        let border_rgba = rgba((ui_border << 8) | ALPHA_BADGE_BORDER);
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
                    .w(px(22.0))
                    .h(px(22.0))
                    .rounded(px(11.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .when(is_current, |d| d.bg(theme_row_selected_bg))
                    .when(!is_current, |d| d.hover(move |s| s.bg(theme_row_hover_bg)))
                    .child(
                        div()
                            .w(px(18.0))
                            .h(px(18.0))
                            .rounded(px(9.0))
                            .bg(rgb(color))
                            .when(is_current, |d| d.border_2().border_color(rgb(text_primary)))
                            .when(!is_current, |d| {
                                d.border_1()
                                    .border_color(rgba((ui_border << 8) | ALPHA_BADGE_BORDER))
                            }),
                    )
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
                let is_current = (value - current_opacity_main).abs() < OPACITY_MATCH_TOLERANCE;
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
                move |_event: &gpui::ClickEvent, _window: &mut Window, cx: &mut gpui::App| {
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
                    .when(!vibrancy_enabled, |d| d.bg(rgba((ui_border << 8) | ALPHA_TOGGLE_BG)))
                    .flex()
                    .items_center()
                    .child(
                        div()
                            .w(px(10.0))
                            .h(px(10.0))
                            .rounded(px(5.0))
                            .when(vibrancy_enabled, |d| d.bg(rgb(text_on_accent)))
                            .when(!vibrancy_enabled, |d| d.bg(rgb(text_primary)))
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
                let is_current = (size - current_ui_font_size).abs() < FONT_SIZE_MATCH_TOLERANCE;
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
                move |_event: &gpui::ClickEvent, _window: &mut Window, cx: &mut gpui::App| {
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
                        });
                    }
                },
            )
            .child("Reset to Defaults");

        let accent_name = Self::accent_color_name(accent_color);

        let preview_panel =
            div()
                .w_1_2()
                .h_full()
                .border_l_1()
                .border_color(border_rgba)
                .px(px(design_spacing.padding_lg))
                .py(px(design_spacing.padding_md))
                .flex()
                .flex_col()
                .gap(px(10.0))
                .overflow_y_hidden()
                // ── Customize section ──────────────────────────────────
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(text_dimmed))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child("CUSTOMIZE"),
                )
                // Accent color row (with name)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(6.0))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .child("Accent Color"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(accent_color))
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .child(accent_name.to_string()),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .gap(px(4.0))
                                .flex_wrap()
                                .children(accent_swatches),
                        ),
                )
                // Opacity row (10 steps)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(div().text_xs().text_color(rgb(text_muted)).child(format!(
                            "Window Opacity  {:.0}%",
                            current_opacity_main * 100.0
                        )))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .gap(px(2.0))
                                .flex_wrap()
                                .children(opacity_buttons),
                        ),
                )
                // Vibrancy toggle + material row
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .child("Vibrancy Blur"),
                        )
                        .child(vibrancy_toggle)
                        .when(vibrancy_enabled, |d| {
                            d.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .mt(px(4.0))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted))
                                            .child("Material"),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .gap(px(3.0))
                                            .flex_wrap()
                                            .children(material_buttons),
                                    ),
                            )
                        }),
                )
                // Font size row
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .child(format!("UI Font Size  {:.0}px", current_ui_font_size)),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .gap(px(4.0))
                                .children(font_size_buttons),
                        ),
                )
                // Reset button
                .child(reset_button)
                // ── Preview section ────────────────────────────────────
                .child(
                    div()
                        .w_full()
                        .mt(px(4.0))
                        .pt(px(8.0))
                        .border_t_1()
                        .border_color(border_rgba)
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_dimmed))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .child("PREVIEW"),
                        ),
                )
                // Mock search box
                .child(
                    div()
                        .w_full()
                        .h(px(28.0))
                        .rounded(px(6.0))
                        .bg(rgb(bg_search_box))
                        .border_1()
                        .border_color(border_rgba)
                        .px(px(10.0))
                        .flex()
                        .flex_row()
                        .items_center()
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .child("Search scripts..."),
                        ),
                )
                // Mock list items
                .child(
                    div()
                        .w_full()
                        .rounded(px(6.0))
                        .border_1()
                        .border_color(border_rgba)
                        .overflow_hidden()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .w_full()
                                .h(px(28.0))
                                .bg(rgb(accent_color))
                                .px(px(10.0))
                                .flex()
                                .flex_row()
                                .items_center()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(rgb(text_on_accent))
                                        .child("Selected Item"),
                                ),
                        )
                        .child(
                            div()
                                .w_full()
                                .h(px(28.0))
                                .bg(rgb(bg_main))
                                .px(px(10.0))
                                .flex()
                                .flex_row()
                                .items_center()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_primary))
                                        .child("Regular Item"),
                                ),
                        )
                        .child(
                            div()
                                .w_full()
                                .h(px(28.0))
                                .bg(rgb(bg_main))
                                .px(px(10.0))
                                .flex()
                                .flex_row()
                                .items_center()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_secondary))
                                        .child("Another Item"),
                                ),
                        ),
                )
                // Terminal + semantic colors
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_dimmed))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .child("TERMINAL"),
                        )
                        .child(div().flex().flex_row().gap(px(2.0)).children(
                            term_colors.iter().map(|&c| {
                                div().w(px(16.0)).h(px(12.0)).rounded(px(2.0)).bg(rgb(c))
                            }),
                        ))
                        .child(div().flex().flex_row().gap(px(2.0)).children(
                            term_bright.iter().map(|&c| {
                                div().w(px(16.0)).h(px(12.0)).rounded(px(2.0)).bg(rgb(c))
                            }),
                        )),
                )
                // Semantic colors
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap(px(8.0))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(3.0))
                                .child(
                                    div()
                                        .w(px(7.0))
                                        .h(px(7.0))
                                        .rounded(px(4.0))
                                        .bg(rgb(ui_success)),
                                )
                                .child(div().text_xs().text_color(rgb(ui_success)).child("OK")),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(3.0))
                                .child(
                                    div()
                                        .w(px(7.0))
                                        .h(px(7.0))
                                        .rounded(px(4.0))
                                        .bg(rgb(ui_error)),
                                )
                                .child(div().text_xs().text_color(rgb(ui_error)).child("Err")),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(3.0))
                                .child(
                                    div()
                                        .w(px(7.0))
                                        .h(px(7.0))
                                        .rounded(px(4.0))
                                        .bg(rgb(ui_warning)),
                                )
                                .child(div().text_xs().text_color(rgb(ui_warning)).child("Warn")),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(3.0))
                                .child(
                                    div()
                                        .w(px(7.0))
                                        .h(px(7.0))
                                        .rounded(px(4.0))
                                        .bg(rgb(ui_info)),
                                )
                                .child(div().text_xs().text_color(rgb(ui_info)).child("Info")),
                        ),
                );

        // ── Footer with keyboard shortcuts ─────────────────────────
        let shortcut = |key: &str, label: &str| {
            div()
                .flex()
                .flex_row()
                .gap(px(4.0))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(text_secondary))
                        .child(key.to_string()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(text_dimmed))
                        .child(label.to_string()),
                )
        };
        let footer_border = rgba((ui_border << 8) | ALPHA_FOOTER_BORDER);
        let footer = div()
            .w_full()
            .px(px(design_spacing.padding_lg))
            .py(px(design_spacing.padding_sm))
            .border_t_1()
            .border_color(footer_border)
            .flex()
            .flex_col()
            .gap(px(2.0))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_center()
                    .gap(px(12.0))
                    .child(shortcut("↑↓", "Preview"))
                    .child(shortcut("Enter", "Apply"))
                    .child(shortcut("Esc", "Cancel"))
                    .child(shortcut("PgUp/Dn", "Jump"))
                    .child(shortcut("Type", "Search")),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_center()
                    .gap(px(12.0))
                    .child(shortcut("⌘[]", "Accent"))
                    .child(shortcut("⌘-/=", "Opacity"))
                    .child(shortcut("⌘B", "Vibrancy"))
                    .child(shortcut("⌘M", "Material"))
                    .child(shortcut("⌘R", "Reset")),
            );

        // ── Empty state when filter has no matches ─────────────────
        if filtered_count == 0 {
            return div()
                .flex()
                .flex_col()
                .w_full()
                .h_full()
                .rounded(px(design_visual.radius_lg))
                .text_color(rgb(text_primary))
                .font_family(design_typography.font_family)
                .key_context("theme_chooser")
                .track_focus(&self.focus_handle)
                .on_key_down(handle_key)
                .child(header)
                .child(
                    div().flex_1().flex().items_center().justify_center().child(
                        div()
                            .text_sm()
                            .text_color(rgb(text_muted))
                            .child("No matching themes"),
                    ),
                )
                .child(footer)
                .into_any_element();
        }

        // ── Main layout: list + preview panel ──────────────────────
        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .rounded(px(design_visual.radius_lg))
            .text_color(rgb(text_primary))
            .font_family(design_typography.font_family)
            .key_context("theme_chooser")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .child(header)
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .flex()
                    .flex_row()
                    .child(div().w_1_2().h_full().child(list))
                    .child(preview_panel),
            )
            .child(footer)
            .into_any_element()
    }
}

#[cfg(test)]
mod theme_chooser_filter_tests {
    use super::*;

    #[test]
    fn test_theme_chooser_filtered_indices_returns_all_presets_when_filter_empty() {
        let expected_count = theme::presets::presets_cached().len();
        let filtered = ScriptListApp::theme_chooser_filtered_indices("");
        assert_eq!(filtered.len(), expected_count);
    }

    #[test]
    fn test_theme_chooser_filtered_indices_matches_ascii_filter_case_insensitively() {
        let presets = theme::presets::presets_cached();
        let dracula_index = presets
            .iter()
            .position(|preset| preset.id == "dracula")
            .expect("dracula preset should exist");

        let filtered = ScriptListApp::theme_chooser_filtered_indices("DRAC");
        assert!(filtered.contains(&dracula_index));
    }

    #[test]
    fn test_resolve_interactive_alphas_makes_selected_more_prominent_when_equal() {
        let (selected, hover) = ScriptListApp::resolve_interactive_alphas(32, 32);
        assert!(selected > hover);
    }

    #[test]
    fn test_resolve_interactive_alphas_preserves_distinct_values() {
        let (selected, hover) = ScriptListApp::resolve_interactive_alphas(80, 32);
        assert_eq!(selected, 80);
        assert_eq!(hover, 32);
    }

    #[test]
    fn test_accent_on_text_color_prefers_background_for_bright_accent() {
        let bg_main = 0x1E1E1E;
        assert_eq!(
            ScriptListApp::accent_on_text_color(0xFBBF24, bg_main),
            bg_main
        );
    }

    #[test]
    fn test_accent_on_text_color_prefers_white_for_dark_accent() {
        let bg_main = 0x1E1E1E;
        assert_eq!(
            ScriptListApp::accent_on_text_color(0x312E81, bg_main),
            0xFFFFFF
        );
    }
}
