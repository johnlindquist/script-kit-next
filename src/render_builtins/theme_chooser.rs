use crate::theme::gpui_integration::{best_contrast_of_two, sync_gpui_component_theme_for_theme};

const ALPHA_BADGE_BORDER: u32 = 0x40;
const ALPHA_FOOTER_BORDER: u32 = 0x30;
const ALPHA_TOGGLE_BG: u32 = 0x80;
const MAX_ALPHA: u32 = 0xFF;
const MIN_HOVER_ALPHA: f32 = 18.0;
const THEME_LIST_PAGE_SIZE: usize = 5;
const OPACITY_MATCH_TOLERANCE: f32 = 0.05;
const FONT_SIZE_MATCH_TOLERANCE: f32 = 0.5;
const LEGACY_THEME_ITEM_HEIGHT: f32 = 48.0;
const THEME_ITEM_MIN_HEIGHT: f32 = 56.0;
const THEME_ITEM_MAX_HEIGHT: f32 = 66.0;
const THEME_ITEM_VERTICAL_PADDING_MIN: f32 = 6.0;
const THEME_ITEM_VERTICAL_PADDING_MAX: f32 = 14.0;
const THEME_ITEM_HORIZONTAL_PADDING_MIN: f32 = 14.0;
const THEME_ITEM_HORIZONTAL_PADDING_MAX: f32 = 20.0;
const THEME_ITEM_CONTENT_GAP_MIN: f32 = 10.0;
const THEME_ITEM_CONTENT_GAP_MAX: f32 = 14.0;
const THEME_ITEM_TEXT_GAP_MIN: f32 = 3.0;
const THEME_ITEM_TEXT_GAP_MAX: f32 = 6.0;
const THEME_ITEM_SWATCH_GAP_MIN: f32 = 2.0;
const THEME_ITEM_SWATCH_GAP_MAX: f32 = 4.0;
const THEME_LIST_VERTICAL_PADDING_MIN: f32 = 2.0;
const THEME_LIST_VERTICAL_PADDING_MAX: f32 = 6.0;

#[derive(Debug, Clone, Copy, PartialEq)]
struct ThemeChooserRowLayout {
    item_height: f32,
    horizontal_padding: f32,
    content_gap: f32,
    text_gap: f32,
    swatch_gap: f32,
    list_vertical_padding: f32,
}

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


    fn theme_chooser_row_layout(spacing: &designs::DesignSpacing) -> ThemeChooserRowLayout {
        let vertical_padding = spacing.item_padding_y.clamp(
            THEME_ITEM_VERTICAL_PADDING_MIN,
            THEME_ITEM_VERTICAL_PADDING_MAX,
        );
        let item_height = (LEGACY_THEME_ITEM_HEIGHT + vertical_padding + spacing.gap_sm / 2.0)
            .clamp(THEME_ITEM_MIN_HEIGHT, THEME_ITEM_MAX_HEIGHT);
        let horizontal_padding = spacing.item_padding_x.clamp(
            THEME_ITEM_HORIZONTAL_PADDING_MIN,
            THEME_ITEM_HORIZONTAL_PADDING_MAX,
        );
        let content_gap = spacing
            .icon_text_gap
            .clamp(THEME_ITEM_CONTENT_GAP_MIN, THEME_ITEM_CONTENT_GAP_MAX);
        let text_gap = spacing
            .gap_sm
            .clamp(THEME_ITEM_TEXT_GAP_MIN, THEME_ITEM_TEXT_GAP_MAX);
        let swatch_gap =
            (spacing.gap_sm / 2.0).clamp(THEME_ITEM_SWATCH_GAP_MIN, THEME_ITEM_SWATCH_GAP_MAX);
        let list_vertical_padding = spacing.margin_sm.clamp(
            THEME_LIST_VERTICAL_PADDING_MIN,
            THEME_LIST_VERTICAL_PADDING_MAX,
        );
        ThemeChooserRowLayout {
            item_height,
            horizontal_padding,
            content_gap,
            text_gap,
            swatch_gap,
            list_vertical_padding,
        }
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
        // Use raw opacity values (matching main menu's ListItem behavior)
        let theme_row_selected_bg = rgba((selection_bg << 8) | selected_alpha);
        let theme_row_hover_bg = rgba((selection_bg << 8) | hover_alpha);

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

        let row_layout = Self::theme_chooser_row_layout(&design_spacing);
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
                            this.theme = original;
                            sync_gpui_component_theme_for_theme(cx, this.theme.as_ref());
                        }
                        this.go_back_or_close(window, cx);
                    }
                    return;
                }
                // Cmd+W: restore and close window
                if has_cmd && key.eq_ignore_ascii_case("w") {
                    if let Some(original) = this.theme_before_chooser.take() {
                        this.theme = original;
                        sync_gpui_component_theme_for_theme(cx, this.theme.as_ref());
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
                    let mut modified = (*this.theme).clone();
                    modified.colors.accent.selected = new_accent;
                    modified.colors.text.on_accent =
                        best_contrast_of_two(new_accent, 0xFFFFFF, modified.colors.background.main);
                    this.theme = std::sync::Arc::new(modified);
                    sync_gpui_component_theme_for_theme(cx, this.theme.as_ref());
                    cx.notify();
                    return;
                }
                if has_cmd && (key == "]" || key.eq_ignore_ascii_case("bracketright")) {
                    let current = this.theme.colors.accent.selected;
                    let idx = Self::find_accent_palette_index(current).unwrap_or(0);
                    let new_idx = (idx + 1) % Self::ACCENT_PALETTE.len();
                    let (new_accent, _) = Self::ACCENT_PALETTE[new_idx];
                    let mut modified = (*this.theme).clone();
                    modified.colors.accent.selected = new_accent;
                    modified.colors.text.on_accent =
                        best_contrast_of_two(new_accent, 0xFFFFFF, modified.colors.background.main);
                    this.theme = std::sync::Arc::new(modified);
                    sync_gpui_component_theme_for_theme(cx, this.theme.as_ref());
                    cx.notify();
                    return;
                }
                // Cmd+- / Cmd+=: adjust opacity
                if has_cmd && key == "-" {
                    let current_main = this
                        .theme
                        .get_opacity()
                        .vibrancy_background
                        .unwrap_or(0.85);
                    let idx = Self::find_opacity_preset_index(current_main);
                    if idx > 0 {
                        let target = Self::OPACITY_PRESETS[idx - 1].0;
                        let mut modified = (*this.theme).clone();
                        if let Some(ref mut op) = modified.opacity {
                            op.vibrancy_background = Some(target);
                        }
                        this.theme = std::sync::Arc::new(modified);
                        sync_gpui_component_theme_for_theme(cx, this.theme.as_ref());
                        cx.notify();
                    }
                    return;
                }
                if has_cmd && (key == "=" || key == "+") {
                    let current_main = this
                        .theme
                        .get_opacity()
                        .vibrancy_background
                        .unwrap_or(0.85);
                    let idx = Self::find_opacity_preset_index(current_main);
                    if idx < Self::OPACITY_PRESETS.len() - 1 {
                        let target = Self::OPACITY_PRESETS[idx + 1].0;
                        let mut modified = (*this.theme).clone();
                        if let Some(ref mut op) = modified.opacity {
                            op.vibrancy_background = Some(target);
                        }
                        this.theme = std::sync::Arc::new(modified);
                        sync_gpui_component_theme_for_theme(cx, this.theme.as_ref());
                        cx.notify();
                    }
                    return;
                }
                // Cmd+B: toggle vibrancy
                if has_cmd && key.eq_ignore_ascii_case("b") {
                    let mut modified = (*this.theme).clone();
                    if let Some(ref mut vibrancy) = modified.vibrancy {
                        vibrancy.enabled = !vibrancy.enabled;
                    }
                    this.theme = std::sync::Arc::new(modified);
                    sync_gpui_component_theme_for_theme(cx, this.theme.as_ref());
                    let is_dark = this.theme.should_use_dark_vibrancy();
                    let material = this.theme.get_vibrancy().material;
                    platform::configure_window_vibrancy_material_for_appearance(
                        is_dark,
                        material,
                    );
                    cx.notify();
                    return;
                }
                // Cmd+M: cycle vibrancy material
                if has_cmd && key.eq_ignore_ascii_case("m") {
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
                    sync_gpui_component_theme_for_theme(cx, this.theme.as_ref());
                    let is_dark = this.theme.should_use_dark_vibrancy();
                    let material = this.theme.get_vibrancy().material;
                    platform::configure_window_vibrancy_material_for_appearance(
                        is_dark,
                        material,
                    );
                    cx.notify();
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
                                this.theme = std::sync::Arc::new(presets[pidx].create_theme());
                                sync_gpui_component_theme_for_theme(cx, this.theme.as_ref());
                                cx.notify();
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
                    sync_gpui_component_theme_for_theme(cx, this.theme.as_ref());
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
                    // Map to actual preset index and apply theme
                    let preset_idx = filtered[*selected_index];
                    let new_theme = std::sync::Arc::new(presets[preset_idx].create_theme());
                    this.theme = new_theme;
                    this.theme_chooser_scroll_handle
                        .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                    sync_gpui_component_theme_for_theme(cx, this.theme.as_ref());
                    cx.notify();
                }
            },
        );

        // ── Pre-compute data for list closure ──────────────────────
        let presets_for_list = presets;
        let selected = selected_index;
        let orig_idx = original_index;
        let first_light_idx = first_light;
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
                        let preset = &presets_for_list[preset_idx];
                        let name = preset.name;
                        let desc = preset.description;
                        let colors = &preview_colors[preset_idx];
                        let is_first_light =
                            filter_is_empty && preset_idx == first_light_idx && first_light_idx > 0;

                        // Compact color bar — thin horizontal strip showing theme palette
                        let color_bar = div()
                            .flex()
                            .flex_row()
                            .w(px(40.0))
                            .h(px(8.0))
                            .rounded(px(4.0))
                            .overflow_hidden()
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

                        let border_rgba = rgba((ui_border << 8) | ALPHA_FOOTER_BORDER);

                        // Section label for light themes (only when unfiltered)
                        let section_label = if is_first_light {
                            Some(
                                div()
                                    .w_full()
                                    .pt(px(row_layout.list_vertical_padding + 2.0))
                                    .pb(px(row_layout.list_vertical_padding))
                                    .px(px(row_layout.horizontal_padding))
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
                                                sync_gpui_component_theme_for_theme(
                                                    cx,
                                                    this.theme.as_ref(),
                                                );
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
                            .gap(px(row_layout.text_gap))
                            .child(
                                div()
                                    .text_sm()
                                    .when(is_original || is_selected, |d| {
                                        d.font_weight(gpui::FontWeight::SEMIBOLD)
                                    })
                                    .text_color(rgb(name_color))
                                    .child(name),
                            )
                            // Description only revealed on focused row
                            .when(is_selected, |d| {
                                d.child(
                                    div().text_xs().text_color(rgb(text_secondary)).child(desc),
                                )
                            });

                        let row = div()
                            .id(ix)
                            .w_full()
                            .h(px(row_layout.item_height))
                            .px(px(row_layout.horizontal_padding))
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(row_layout.content_gap))
                            .cursor_pointer()
                            .border_l(px(3.0))
                            .when(is_selected, |d| {
                                d.bg(theme_row_selected_bg)
                                    .border_color(rgb(accent_color))
                            })
                            .when(!is_selected, |d| {
                                d.border_color(gpui::transparent_black())
                                    .hover(move |s| s.bg(theme_row_hover_bg))
                            })
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
        let list_scrollbar = self.builtin_uniform_list_scrollbar(
            &self.theme_chooser_scroll_handle,
            filtered_count,
            8,
        );

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
        let current_opacity_main = opacity.vibrancy_background.unwrap_or(0.85);
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
            .map(|(i, &(color, name))| {
                let is_current = color == accent_color;
                let click_entity = entity_handle_for_customize.clone();
                let swatch_bg_main = bg_main;
                let tooltip_label = format!("Set accent color to {}", name);
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
                    .tooltip(move |window, cx| {
                        gpui_component::tooltip::Tooltip::new(tooltip_label.clone())
                            .build(window, cx)
                    })
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
                                        best_contrast_of_two(color, 0xFFFFFF, swatch_bg_main);
                                    this.theme = std::sync::Arc::new(modified);
                                    sync_gpui_component_theme_for_theme(cx, this.theme.as_ref());
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
                let tooltip_label = format!("Set opacity to {}", label);
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
                    .tooltip(move |window, cx| {
                        gpui_component::tooltip::Tooltip::new(tooltip_label.clone())
                            .build(window, cx)
                    })
                    .on_click(
                        move |_event: &gpui::ClickEvent,
                              _window: &mut Window,
                              cx: &mut gpui::App| {
                            if let Some(app) = click_entity.upgrade() {
                                app.update(cx, |this, cx| {
                                    let mut modified = (*this.theme).clone();
                                    if let Some(ref mut op) = modified.opacity {
                                        op.vibrancy_background = Some(value);
                                    }
                                    this.theme = std::sync::Arc::new(modified);
                                    sync_gpui_component_theme_for_theme(cx, this.theme.as_ref());
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
            .tooltip(|window, cx| {
                gpui_component::tooltip::Tooltip::new("Toggle vibrancy blur").build(window, cx)
            })
            .on_click(
                move |_event: &gpui::ClickEvent, _window: &mut Window, cx: &mut gpui::App| {
                    if let Some(app) = vibrancy_entity.upgrade() {
                        app.update(cx, |this, cx| {
                            let mut modified = (*this.theme).clone();
                            if let Some(ref mut vibrancy) = modified.vibrancy {
                                vibrancy.enabled = !vibrancy.enabled;
                            }
                            this.theme = std::sync::Arc::new(modified);
                            sync_gpui_component_theme_for_theme(cx, this.theme.as_ref());
                            let is_dark = this.theme.should_use_dark_vibrancy();
                            let material = this.theme.get_vibrancy().material;
                            platform::configure_window_vibrancy_material_for_appearance(
                                is_dark,
                                material,
                            );
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
                        d.bg(rgba((ui_border << 8) | ALPHA_TOGGLE_BG))
                    })
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
                let tooltip_label = format!("Set vibrancy material to {}", label);
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
                    .tooltip(move |window, cx| {
                        gpui_component::tooltip::Tooltip::new(tooltip_label.clone())
                            .build(window, cx)
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
                                    sync_gpui_component_theme_for_theme(cx, this.theme.as_ref());
                                    let is_dark = this.theme.should_use_dark_vibrancy();
                                    let current_material = this.theme.get_vibrancy().material;
                                    platform::configure_window_vibrancy_material_for_appearance(
                                        is_dark,
                                        current_material,
                                    );
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
                let tooltip_label = format!("Set UI font size to {}px", label);
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
                    .tooltip(move |window, cx| {
                        gpui_component::tooltip::Tooltip::new(tooltip_label.clone())
                            .build(window, cx)
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
                                    sync_gpui_component_theme_for_theme(cx, this.theme.as_ref());
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
            .tooltip(|window, cx| {
                gpui_component::tooltip::Tooltip::new("Reset to selected preset defaults")
                    .build(window, cx)
            })
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
                                        sync_gpui_component_theme_for_theme(
                                            cx,
                                            this.theme.as_ref(),
                                        );
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
                // ── Preview section (spacing-only separation per spec) ──
                .child(
                    div()
                        .w_full()
                        .mt(px(8.0))
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

        // ── Footer: canonical three-key hint strip per .impeccable.md ──
        let footer = crate::components::prompt_layout_shell::render_simple_hint_strip(
            vec![
                gpui::SharedString::from("↵ Apply"),
                gpui::SharedString::from("Esc Back"),
            ],
            None,
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
                    .child(
                        div()
                            .w_1_2()
                            .h_full()
                            .py(px(row_layout.list_vertical_padding))
                            .child(
                                div()
                                    .relative()
                                    .w_full()
                                    .h_full()
                                    .child(list)
                                    .child(list_scrollbar),
                            ),
                    )
                    .child(preview_panel),
            )
            .child(footer)
            .into_any_element()
    }
}

#[cfg(test)]
mod theme_chooser_chrome_audit {
    #[test]
    fn theme_chooser_uses_truthful_two_item_footer() {
        let source = include_str!("theme_chooser.rs");
        assert!(
            !source.contains("universal_prompt_hints()"),
            "theme_chooser should not use universal hints (no actions dialog wired)"
        );
        assert!(
            source.contains("render_simple_hint_strip("),
            "theme_chooser should use render_simple_hint_strip"
        );
        assert!(
            source.contains(r#"SharedString::from("↵ Apply")"#),
            "theme_chooser should use truthful '↵ Apply' footer label"
        );
        assert!(
            source.contains(r#"SharedString::from("Esc Back")"#),
            "theme_chooser should use 'Esc Back' footer label"
        );
        assert!(
            !source.contains("⌘K Actions"),
            "theme_chooser should not advertise ⌘K Actions without a working dialog"
        );
    }

    #[test]
    fn theme_chooser_has_no_legacy_multi_shortcut_footer() {
        let source = include_str!("theme_chooser.rs");
        assert!(
            !source.contains(r#".child(shortcut("⌘[]", "Accent"))"#),
            "theme_chooser should not have legacy multi-shortcut footer"
        );
    }

    #[test]
    fn theme_chooser_preview_has_no_decorative_section_dividers() {
        let source = include_str!("theme_chooser.rs");
        // The PREVIEW section header should use spacing-only separation,
        // not a border_t_1 divider.
        let preview_section_start = source.find("spacing-only separation per spec");
        assert!(
            preview_section_start.is_some(),
            "theme_chooser preview section should use spacing-only separation"
        );
    }

    #[test]
    fn theme_chooser_has_no_prompt_footer() {
        let source = include_str!("theme_chooser.rs");
        let legacy = "Prompt".to_owned() + "Footer::new(";
        assert_eq!(
            source.matches(&legacy).count(),
            0,
            "theme_chooser should not use PromptFooter"
        );
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
    fn test_accent_on_text_color_prefers_background_for_bright_accent() {
        let bg_main = 0x1E1E1E;
        assert_eq!(best_contrast_of_two(0xFBBF24, 0xFFFFFF, bg_main), bg_main);
    }

    #[test]
    fn test_accent_on_text_color_prefers_white_for_dark_accent() {
        let bg_main = 0x1E1E1E;
        assert_eq!(best_contrast_of_two(0x312E81, 0xFFFFFF, bg_main), 0xFFFFFF);
    }

    #[test]
    fn test_theme_chooser_row_layout_increases_spacing_when_default_tokens_used() {
        let layout = ScriptListApp::theme_chooser_row_layout(&designs::DesignSpacing::default());

        assert_eq!(layout.item_height, 58.0);
        assert_eq!(layout.horizontal_padding, 16.0);
        assert_eq!(layout.content_gap, 10.0);
        assert_eq!(layout.text_gap, 4.0);
        assert_eq!(layout.swatch_gap, 2.0);
        assert_eq!(layout.list_vertical_padding, 4.0);
    }

    #[test]
    fn test_theme_chooser_row_layout_clamps_extreme_spacing_tokens() {
        let spacing = designs::DesignSpacing {
            padding_xs: 12.0,
            padding_sm: 20.0,
            gap_sm: 20.0,
            margin_sm: 20.0,
            item_padding_x: 40.0,
            item_padding_y: 40.0,
            icon_text_gap: 32.0,
            ..designs::DesignSpacing::default()
        };

        let layout = ScriptListApp::theme_chooser_row_layout(&spacing);

        assert_eq!(layout.item_height, THEME_ITEM_MAX_HEIGHT);
        assert_eq!(layout.horizontal_padding, THEME_ITEM_HORIZONTAL_PADDING_MAX);
        assert_eq!(layout.content_gap, THEME_ITEM_CONTENT_GAP_MAX);
        assert_eq!(layout.text_gap, THEME_ITEM_TEXT_GAP_MAX);
        assert_eq!(layout.swatch_gap, THEME_ITEM_SWATCH_GAP_MAX);
        assert_eq!(
            layout.list_vertical_padding,
            THEME_LIST_VERTICAL_PADDING_MAX
        );
    }
}
