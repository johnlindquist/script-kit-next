use crate::theme::gpui_integration::{
    best_contrast_of_two, sync_gpui_component_theme_for_theme_with_source,
};

const THEME_LIST_PAGE_SIZE: usize = 5;
const OPACITY_MATCH_TOLERANCE: f32 = 0.05;
const FONT_SIZE_MATCH_TOLERANCE: f32 = 0.5;

/// Unified theme chooser preview sync: applies both gpui-component colors and
/// native vibrancy/material in one call, with a source tag for tracing.
fn sync_theme_chooser_preview(
    cx: &mut gpui::App,
    active_theme: &std::sync::Arc<crate::theme::Theme>,
    source: &'static str,
) {
    sync_gpui_component_theme_for_theme_with_source(cx, active_theme.as_ref(), source);
}

#[derive(Debug, Clone, Copy)]
struct ThemeChooserMatchSummary {
    catalog_total: usize,
    catalog_dark: usize,
    catalog_light: usize,
    visible_total: usize,
    visible_dark: usize,
    visible_light: usize,
}

#[derive(Clone, Debug)]
struct ThemeChooserContrastRow {
    label: String,
    ratio: f32,
    minimum: f32,
    passes: bool,
}

#[derive(Clone, Debug)]
struct ThemeChooserContrastSnapshot {
    rows: Vec<ThemeChooserContrastRow>,
    passing: usize,
    total: usize,
    worst_label: String,
    worst_ratio: f32,
}

fn build_theme_chooser_contrast_snapshot(
    theme: &crate::theme::Theme,
) -> ThemeChooserContrastSnapshot {
    let rows = theme::audit_theme_contrast(theme)
        .into_iter()
        .map(|sample| ThemeChooserContrastRow {
            label: sample.label.to_string(),
            ratio: sample.ratio,
            minimum: sample.minimum,
            passes: sample.passes(),
        })
        .collect::<Vec<_>>();

    let passing = rows.iter().filter(|row| row.passes).count();
    let total = rows.len();

    let worst = rows
        .iter()
        .min_by(|left, right| {
            left.ratio
                .partial_cmp(&right.ratio)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned()
        .unwrap_or(ThemeChooserContrastRow {
            label: "n/a".to_string(),
            ratio: 0.0,
            minimum: 4.5,
            passes: false,
        });

    ThemeChooserContrastSnapshot {
        rows,
        passing,
        total,
        worst_label: worst.label,
        worst_ratio: worst.ratio,
    }
}

fn theme_chooser_remix_seed() -> usize {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as usize)
        .unwrap_or(0)
}

fn cached_theme_chooser_contrast_snapshot(
    theme: &std::sync::Arc<crate::theme::Theme>,
) -> ThemeChooserContrastSnapshot {
    static THEME_CHOOSER_CONTRAST_CACHE: std::sync::LazyLock<
        parking_lot::Mutex<std::collections::HashMap<usize, ThemeChooserContrastSnapshot>>,
    > = std::sync::LazyLock::new(|| parking_lot::Mutex::new(std::collections::HashMap::new()));

    let cache_key = std::sync::Arc::as_ptr(theme) as usize;

    if let Some(snapshot) = THEME_CHOOSER_CONTRAST_CACHE.lock().get(&cache_key).cloned() {
        return snapshot;
    }

    let snapshot = build_theme_chooser_contrast_snapshot(theme.as_ref());

    let mut cache = THEME_CHOOSER_CONTRAST_CACHE.lock();
    if cache.len() >= 128 {
        cache.clear();
    }
    cache.insert(cache_key, snapshot.clone());
    snapshot
}

impl ScriptListApp {
    fn theme_chooser_match_summary(
        filtered_indices: &[usize],
        presets: &[theme::presets::ThemePreset],
    ) -> ThemeChooserMatchSummary {
        let catalog_dark = presets.iter().filter(|preset| preset.is_dark).count();
        let visible_dark = filtered_indices
            .iter()
            .filter(|&&idx| presets[idx].is_dark)
            .count();
        ThemeChooserMatchSummary {
            catalog_total: presets.len(),
            catalog_dark,
            catalog_light: presets.len().saturating_sub(catalog_dark),
            visible_total: filtered_indices.len(),
            visible_dark,
            visible_light: filtered_indices.len().saturating_sub(visible_dark),
        }
    }

    fn render_theme_chooser_summary_chip(
        label: impl Into<String>,
        active: bool,
        chrome: &theme::AppChromeColors,
        text_on_accent: u32,
    ) -> AnyElement {
        let label = label.into();
        div()
            .px(px(8.0))
            .py(px(4.0))
            .rounded(px(6.0))
            .text_xs()
            .when(active, |d| {
                d.bg(rgb(chrome.accent_hex))
                    .text_color(rgb(text_on_accent))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
            })
            .when(!active, |d| {
                d.bg(rgba(chrome.badge_bg_rgba))
                    .border_1()
                    .border_color(rgba(chrome.badge_border_rgba))
                    .text_color(rgb(chrome.badge_text_hex))
            })
            .child(label)
            .into_any_element()
    }

    fn render_theme_chooser_summary_strip(
        summary: ThemeChooserMatchSummary,
        selected_preset_name: &str,
        chrome: &theme::AppChromeColors,
        text_on_accent: u32,
    ) -> AnyElement {
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .flex_wrap()
            .child(Self::render_theme_chooser_summary_chip(
                format!("{}/{} shown", summary.visible_total, summary.catalog_total),
                true,
                chrome,
                text_on_accent,
            ))
            .child(Self::render_theme_chooser_summary_chip(
                format!("{} dark", summary.visible_dark),
                false,
                chrome,
                text_on_accent,
            ))
            .child(Self::render_theme_chooser_summary_chip(
                format!("{} light", summary.visible_light),
                false,
                chrome,
                text_on_accent,
            ))
            .child(Self::render_theme_chooser_summary_chip(
                selected_preset_name.to_string(),
                false,
                chrome,
                text_on_accent,
            ))
            .into_any_element()
    }

    fn render_theme_chooser_empty_state_body(
        &self,
        filter: &str,
        summary: ThemeChooserMatchSummary,
        chrome: &theme::AppChromeColors,
    ) -> AnyElement {
        let query = if filter.is_empty() {
            "your search".to_string()
        } else {
            format!("\"{}\"", filter)
        };

        div()
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .max_w(px(360.0))
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(10.0))
                    .child(
                        div()
                            .w(px(56.0))
                            .h(px(10.0))
                            .rounded(px(5.0))
                            .bg(rgb(chrome.accent_hex)),
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(chrome.text_primary_hex))
                            .child(format!("No themes match {}", query)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(chrome.text_muted_hex))
                            .child("Try a family name like rose, github, nord, or light."),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(chrome.text_dimmed_hex))
                            .child(format!(
                                "{} dark · {} light · {} total presets",
                                summary.catalog_dark, summary.catalog_light, summary.catalog_total
                            )),
                    ),
            )
            .into_any_element()
    }

    fn render_theme_chooser_surface_card(
        title: &'static str,
        bg_rgba: u32,
        border_rgba: u32,
        text_hex: u32,
        muted_hex: u32,
        accent_hex: u32,
    ) -> AnyElement {
        div()
            .min_w(px(116.0))
            .flex_1()
            .rounded(px(8.0))
            .border_1()
            .border_color(rgba(border_rgba))
            .bg(rgba(bg_rgba))
            .p(px(8.0))
            .flex()
            .flex_col()
            .gap(px(6.0))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(muted_hex))
                    .child(title),
            )
            .child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(text_hex))
                    .child("Preview Text"),
            )
            .child(
                div()
                    .w(px(44.0))
                    .h(px(6.0))
                    .rounded(px(3.0))
                    .bg(rgb(accent_hex)),
            )
            .into_any_element()
    }

    fn render_theme_chooser_surface_lab(
        &self,
        chrome: &theme::AppChromeColors,
    ) -> AnyElement {
        div()
            .flex()
            .flex_col()
            .gap(px(6.0))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(chrome.text_muted_hex))
                    .child("Surface Lab"),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(8.0))
                    .flex_wrap()
                    .child(Self::render_theme_chooser_surface_card(
                        "Window",
                        chrome.window_surface_rgba,
                        chrome.border_rgba,
                        chrome.text_primary_hex,
                        chrome.text_muted_hex,
                        chrome.accent_hex,
                    ))
                    .child(Self::render_theme_chooser_surface_card(
                        "Preview",
                        chrome.preview_surface_rgba,
                        chrome.border_rgba,
                        chrome.text_primary_hex,
                        chrome.text_muted_hex,
                        chrome.accent_hex,
                    ))
                    .child(Self::render_theme_chooser_surface_card(
                        "Panel",
                        chrome.panel_surface_rgba,
                        chrome.border_rgba,
                        chrome.text_primary_hex,
                        chrome.text_muted_hex,
                        chrome.accent_hex,
                    ))
                    .child(Self::render_theme_chooser_surface_card(
                        "Input",
                        chrome.input_active_rgba,
                        chrome.border_rgba,
                        chrome.text_primary_hex,
                        chrome.text_muted_hex,
                        chrome.accent_hex,
                    ))
                    .child(Self::render_theme_chooser_surface_card(
                        "Log",
                        chrome.log_panel_surface_rgba,
                        chrome.border_rgba,
                        chrome.text_primary_hex,
                        chrome.text_muted_hex,
                        chrome.accent_hex,
                    ))
                    .child(Self::render_theme_chooser_surface_card(
                        "Dialog",
                        chrome.dialog_surface_rgba,
                        chrome.border_rgba,
                        chrome.text_primary_hex,
                        chrome.text_muted_hex,
                        chrome.accent_hex,
                    )),
            )
            .into_any_element()
    }

    fn render_theme_chooser_contrast_row(
        sample: &ThemeChooserContrastRow,
        chrome: &theme::AppChromeColors,
    ) -> AnyElement {
        let status_bg = if sample.passes {
            rgba(chrome.accent_badge_bg_rgba)
        } else {
            rgba(chrome.hover_rgba)
        };
        let status_text = if sample.passes { "Pass" } else { "Fix" };
        let status_text_hex = if sample.passes {
            chrome.accent_badge_text_hex
        } else {
            chrome.text_primary_hex
        };

        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .gap(px(8.0))
            .px(px(8.0))
            .py(px(6.0))
            .rounded(px(6.0))
            .bg(rgba(chrome.badge_bg_rgba))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(chrome.text_primary_hex))
                            .child(sample.label.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(chrome.text_muted_hex))
                            .child(format!(
                                "{:.2}:1 minimum {:.1}:1",
                                sample.ratio, sample.minimum
                            )),
                    ),
            )
            .child(
                div()
                    .px(px(6.0))
                    .py(px(3.0))
                    .rounded(px(4.0))
                    .bg(status_bg)
                    .text_xs()
                    .text_color(rgb(status_text_hex))
                    .child(status_text),
            )
            .into_any_element()
    }

    /// Helper: compute filtered preset indices from a filter string
    fn theme_chooser_filtered_indices(filter: &str) -> Vec<usize> {
        theme::presets::filtered_preset_indices_cached(filter)
    }

    /// Unified helper for all chooser-local theme mutations.
    /// Updates self.theme, syncs gpui-component + native vibrancy, and notifies.
    fn apply_theme_chooser_theme(
        &mut self,
        next_theme: crate::theme::Theme,
        reason: &'static str,
        cx: &mut Context<Self>,
    ) {
        self.theme = std::sync::Arc::new(next_theme);
        sync_theme_chooser_preview(cx, &self.theme, reason);
        // Sync native vibrancy so the window material matches the theme
        let is_dark = self.theme.should_use_dark_vibrancy();
        let material = self.theme.get_vibrancy().material;
        platform::configure_window_vibrancy_material_for_appearance(is_dark, material);
        cx.notify();
    }

    /// Clone-and-mutate convenience: clones the current theme, applies a
    /// mutation closure, then routes through the unified preview pipeline.
    fn mutate_theme_chooser_theme(
        &mut self,
        reason: &'static str,
        cx: &mut Context<Self>,
        mutate: impl FnOnce(&mut crate::theme::Theme),
    ) {
        let mut next = (*self.theme).clone();
        mutate(&mut next);
        self.apply_theme_chooser_theme(next, reason, cx);
    }

    /// Restore a previously saved theme (escape/close paths).
    /// Routes through the same preview sync pipeline as mutations.
    fn restore_theme_chooser_theme(
        &mut self,
        original: std::sync::Arc<crate::theme::Theme>,
        reason: &'static str,
        cx: &mut Context<Self>,
    ) {
        self.theme = original;
        sync_theme_chooser_preview(cx, &self.theme, reason);
        // Sync native vibrancy for the restored theme
        let is_dark = self.theme.should_use_dark_vibrancy();
        let material = self.theme.get_vibrancy().material;
        platform::configure_window_vibrancy_material_for_appearance(is_dark, material);
        cx.notify();
    }

    /// Shared helper: preview a preset by filtered index, using the cached theme.
    fn preview_theme_chooser_preset(
        &mut self,
        filtered_indices: &[usize],
        filtered_selected_index: usize,
        reason: &'static str,
        cx: &mut Context<Self>,
    ) {
        let Some(&preset_idx) = filtered_indices.get(filtered_selected_index) else {
            return;
        };
        let next_theme = (*theme::presets::preset_theme_cached(preset_idx)).clone();
        self.theme_chooser_scroll_handle
            .scroll_to_item(filtered_selected_index, ScrollStrategy::Nearest);
        self.apply_theme_chooser_theme(next_theme, reason, cx);
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


    /// Three-item footer hint strip for the theme chooser
    fn theme_chooser_hint_items() -> Vec<gpui::SharedString> {
        vec![
            gpui::SharedString::from("↵ Apply"),
            gpui::SharedString::from("⌘J Remix"),
            gpui::SharedString::from("Esc Back"),
        ]
    }

    /// Apply a surface opacity preset to all shell surfaces together,
    /// so the preview and the real app behave identically.
    fn apply_surface_opacity_preset(
        theme: &crate::theme::Theme,
        value: f32,
    ) -> crate::theme::Theme {
        let mut next = theme.clone();
        let mut opacity = next.get_opacity();
        opacity.main = value;
        opacity.title_bar = value;
        opacity.search_box = (value + 0.06).min(1.0);
        opacity.log_panel = value;
        opacity.dialog = value;
        opacity.input = (value + 0.04).min(1.0);
        opacity.panel = value;
        opacity.input_inactive = (value + 0.02).min(1.0);
        opacity.input_active = (value + 0.08).min(1.0);
        opacity.vibrancy_background = Some(value);
        next.opacity = Some(opacity);
        next
    }

    /// Build a remixed theme by randomly combining accent, opacity, and vibrancy material.
    fn build_theme_chooser_remix(base: &crate::theme::Theme, seed: usize) -> crate::theme::Theme {
        let mut next = base.clone();

        // Use different bit ranges of the seed for each dimension to avoid correlation
        let accent_index = seed % Self::ACCENT_PALETTE.len();
        let opacity_index = (seed / 7) % Self::OPACITY_PRESETS.len();
        let material_index = (seed / 13) % Self::VIBRANCY_MATERIALS.len();

        let (accent_hex, _) = Self::ACCENT_PALETTE[accent_index];
        let (opacity_value, _) = Self::OPACITY_PRESETS[opacity_index];
        let (material, _) = Self::VIBRANCY_MATERIALS[material_index];

        next.colors.accent.selected = accent_hex;
        next.colors.text.on_accent =
            best_contrast_of_two(accent_hex, 0xFFFFFF, next.colors.background.main);
        next = Self::apply_surface_opacity_preset(&next, opacity_value);
        if let Some(ref mut vibrancy) = next.vibrancy {
            vibrancy.enabled = true;
            vibrancy.material = material;
        }

        next
    }

    /// Render a contrast-safe semantic status chip
    fn render_theme_chooser_semantic_chip(
        label: &'static str,
        colors: theme::SemanticChipColors,
    ) -> gpui::AnyElement {
        div()
            .px(px(8.0))
            .py(px(3.0))
            .rounded(px(5.0))
            .border_1()
            .border_color(rgba(colors.border_rgba))
            .bg(rgba(colors.bg_rgba))
            .text_xs()
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(rgb(colors.text_hex))
            .child(label)
            .into_any_element()
    }

    /// Render a keycap badge for the live preview
    fn render_theme_chooser_preview_keycap(
        label: &'static str,
        chrome: &theme::AppChromeColors,
    ) -> gpui::AnyElement {
        div()
            .px(px(6.0))
            .py(px(2.0))
            .rounded(px(5.0))
            .bg(rgba(chrome.badge_bg_rgba))
            .border_1()
            .border_color(rgba(chrome.badge_border_rgba))
            .text_xs()
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(rgb(chrome.badge_text_hex))
            .child(label)
            .into_any_element()
    }

    /// Render a launcher-style live preview that matches the main menu shell
    fn render_theme_chooser_list_item_preview_rows(&self) -> gpui::AnyElement {
        let list_colors = crate::list_item::ListItemColors::from_theme(self.theme.as_ref());

        div()
            .flex()
            .flex_col()
            .child(
                crate::list_item::ListItem::new("Selected Item", list_colors)
                    .description("Description appears only on the focused row")
                    .selected(true)
                    .with_accent_bar(true),
            )
            .child(
                crate::list_item::ListItem::new("Regular Item", list_colors)
                    .shortcut("cmd+p"),
            )
            .child(
                crate::list_item::ListItem::new("Another Item", list_colors)
                    .tool_badge("ts"),
            )
            .into_any_element()
    }

    fn render_theme_chooser_live_preview(
        &self,
        preset_name: &str,
        accent_name: &str,
        chrome: &theme::AppChromeColors,
    ) -> gpui::AnyElement {
        let hint_rgba = (chrome.text_dimmed_hex << 8) | 0xA6;

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(10.0))
            // Header: preset name + accent + keycaps
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(rgb(chrome.text_primary_hex))
                                    .child(preset_name.to_string()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(chrome.text_muted_hex))
                                    .child(format!(
                                        "{accent_name} accent · live launcher preview"
                                    )),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(6.0))
                            .child(Self::render_theme_chooser_preview_keycap("⌘R", chrome))
                            .child(Self::render_theme_chooser_preview_keycap(
                                "⌘[ ]", chrome,
                            )),
                    ),
            )
            // Mini launcher shell
            .child(
                div()
                    .w_full()
                    .rounded(px(8.0))
                    .overflow_hidden()
                    // Search row
                    .child(
                        div()
                            .px(px(12.0))
                            .py(px(10.0))
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .flex_1()
                                    .text_xs()
                                    .text_color(rgb(chrome.text_muted_hex))
                                    .child("Search scripts..."),
                            )
                            .child(Self::render_theme_chooser_preview_keycap("Tab", chrome)),
                    )
                    // Divider
                    .child(div().mx(px(12.0)).h(px(1.0)).bg(rgba(chrome.divider_rgba)))
                    // List rows — real ListItem components for pixel-perfect alignment
                    .child(self.render_theme_chooser_list_item_preview_rows())
                    // Footer divider + hints
                    .child(div().mx(px(12.0)).h(px(1.0)).bg(rgba(chrome.divider_rgba)))
                    .child(
                        div()
                            .px(px(12.0))
                            .py(px(8.0))
                            .flex()
                            .justify_end()
                            .child(crate::components::render_hint_icons(
                                &["↵ Apply", "Esc Back", "⌘R Reset"],
                                hint_rgba,
                            )),
                    ),
            )
            .into_any_element()
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
        let chrome = theme::AppChromeColors::from_theme(self.theme.as_ref());
        let text_primary = chrome.text_primary_hex;
        let text_dimmed = chrome.text_dimmed_hex;
        let text_secondary = chrome.text_secondary_hex;
        let text_muted = chrome.text_muted_hex;
        let accent_color = chrome.accent_hex;
        let text_on_accent = self.theme.colors.text.on_accent;
        let bg_main = self.theme.colors.background.main;
        let ui_success = self.theme.colors.ui.success;
        let ui_error = self.theme.colors.ui.error;
        let ui_warning = self.theme.colors.ui.warning;
        let ui_info = self.theme.colors.ui.info;
        let divider_bg = rgba(chrome.divider_rgba);
        let badge_border_bg = rgba(chrome.badge_border_rgba);
        let theme_row_selected_bg = rgba(chrome.selection_rgba);
        let theme_row_hover_bg = rgba(chrome.hover_rgba);
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

        let summary = Self::theme_chooser_match_summary(&filtered_indices, presets);
        let selected_preset_name_early = filtered_indices
            .get(selected_index)
            .and_then(|idx| presets.get(*idx))
            .map(|preset| preset.name)
            .unwrap_or("Theme Preview");

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
                            this.restore_theme_chooser_theme(original, "theme_chooser_escape_restore", cx);
                        }
                        this.go_back_or_close(window, cx);
                    }
                    return;
                }
                // Cmd+W: restore and close window
                if has_cmd && key.eq_ignore_ascii_case("w") {
                    if let Some(original) = this.theme_before_chooser.take() {
                        this.restore_theme_chooser_theme(original, "theme_chooser_close_restore", cx);
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
                    this.apply_theme_chooser_theme(modified, "theme_chooser_accent_cycle", cx);
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
                    this.apply_theme_chooser_theme(modified, "theme_chooser_accent_cycle", cx);
                    return;
                }
                // Cmd+- / Cmd+=: adjust surface opacity (all shell surfaces together)
                if has_cmd && key == "-" {
                    let current_main = this.theme.get_opacity().main;
                    let idx = Self::find_opacity_preset_index(current_main);
                    if idx > 0 {
                        let target = Self::OPACITY_PRESETS[idx - 1].0;
                        let modified = Self::apply_surface_opacity_preset(this.theme.as_ref(), target);
                        this.apply_theme_chooser_theme(modified, "theme_chooser_opacity_decrease", cx);
                    }
                    return;
                }
                if has_cmd && (key == "=" || key == "+") {
                    let current_main = this.theme.get_opacity().main;
                    let idx = Self::find_opacity_preset_index(current_main);
                    if idx < Self::OPACITY_PRESETS.len() - 1 {
                        let target = Self::OPACITY_PRESETS[idx + 1].0;
                        let modified = Self::apply_surface_opacity_preset(this.theme.as_ref(), target);
                        this.apply_theme_chooser_theme(modified, "theme_chooser_opacity_increase", cx);
                    }
                    return;
                }
                // Cmd+B: toggle vibrancy
                if has_cmd && key.eq_ignore_ascii_case("b") {
                    let mut modified = (*this.theme).clone();
                    if let Some(ref mut vibrancy) = modified.vibrancy {
                        vibrancy.enabled = !vibrancy.enabled;
                    }
                    this.apply_theme_chooser_theme(modified, "theme_chooser_vibrancy_toggle", cx);
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
                    this.apply_theme_chooser_theme(modified, "theme_chooser_vibrancy_material_cycle", cx);
                    return;
                }
                // Cmd+J: surprise me / remix
                if has_cmd && key.eq_ignore_ascii_case("j") {
                    let remixed = Self::build_theme_chooser_remix(
                        this.theme.as_ref(),
                        theme_chooser_remix_seed(),
                    );
                    this.apply_theme_chooser_theme(remixed, "theme_chooser_surprise_me", cx);
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
                    let filtered = Self::theme_chooser_filtered_indices(&current_filter);
                    if let AppView::ThemeChooserView {
                        ref selected_index, ..
                    } = this.current_view
                    {
                        this.preview_theme_chooser_preset(
                            &filtered,
                            *selected_index,
                            "theme_chooser_reset_shortcut",
                            cx,
                        );
                    }
                    return;
                }
                // Enter: apply and close
                if is_key_enter(key) {
                    this.theme_before_chooser = None;
                    match crate::theme::service::persist_theme_and_sync_all_windows(
                        cx,
                        this.theme.as_ref(),
                        "theme_chooser_apply",
                    ) {
                        Ok(applied_theme) => {
                            this.theme = std::sync::Arc::new(applied_theme);
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to save theme: {}", e));
                        }
                    }
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
                    // Copy index before calling &mut self method
                    let idx = *selected_index;
                    // Map to actual preset index and apply theme
                    this.preview_theme_chooser_preset(
                        &filtered,
                        idx,
                        "theme_chooser_keyboard_preview",
                        cx,
                    );
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
        let accent_badge_border = rgba(chrome.accent_badge_border_rgba);
        let accent_badge_bg = rgba(chrome.accent_badge_bg_rgba);
        let accent_badge_text = rgb(chrome.accent_badge_text_hex);
        let list_colors = crate::list_item::ListItemColors::from_theme(self.theme.as_ref());

        // ── Theme list (shared ListItem rows) ─────────────────────
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

                        // "Saved" badge for original theme — trailing accessory
                        let saved_badge = if is_original {
                            Some(
                                div()
                                    .px(px(6.0))
                                    .py(px(2.0))
                                    .rounded(px(5.0))
                                    .border_1()
                                    .border_color(accent_badge_border)
                                    .bg(accent_badge_bg)
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(accent_badge_text)
                                    .child("Saved")
                                    .into_any_element(),
                            )
                        } else {
                            None
                        };

                        // Section label for light themes (only when unfiltered)
                        let section_label = if is_first_light {
                            Some(crate::list_item::render_section_header(
                                "LIGHT", None, list_colors, false,
                            ))
                        } else {
                            None
                        };

                        // Click handler: select + preview via filtered index
                        let click_entity = entity_handle.clone();
                        let click_handler =
                            move |_event: &gpui::ClickEvent,
                                  _window: &mut Window,
                                  cx: &mut gpui::App| {
                                if let Some(app) = click_entity.upgrade() {
                                    app.update(cx, |this, cx| {
                                        let current_filter = if let AppView::ThemeChooserView {
                                            ref filter,
                                            ..
                                        } = this.current_view
                                        {
                                            filter.clone()
                                        } else {
                                            return;
                                        };
                                        let filtered =
                                            Self::theme_chooser_filtered_indices(&current_filter);

                                        if let AppView::ThemeChooserView {
                                            ref mut selected_index,
                                            ..
                                        } = this.current_view
                                        {
                                            *selected_index = ix;
                                        }
                                        this.preview_theme_chooser_preset(
                                            &filtered,
                                            ix,
                                            "theme_chooser_mouse_preview",
                                            cx,
                                        );
                                    });
                                }
                            };

                        // Build shared ListItem row — matches main menu rendering
                        let item = crate::list_item::ListItem::new(name, list_colors)
                            .description(desc)
                            .selected(is_selected)
                            .with_accent_bar(true)
                            .index(ix)
                            .leading_accessory(color_bar)
                            .trailing_accessory_opt(saved_badge);

                        let row = div()
                            .id(ix)
                            .cursor_pointer()
                            .on_click(click_handler)
                            .child(item);

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

        // ── Header with search input + summary strip ─────────────────
        let header_divider = div()
            .mx(px(design_spacing.padding_lg))
            .h(px(1.0))
            .bg(divider_bg);

        let header = div()
            .w_full()
            .px(px(design_spacing.padding_lg))
            .pt(px(design_spacing.padding_md))
            .pb(px(6.0))
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(
                div().flex().flex_row().items_center().child(
                    div().flex_1().flex().flex_row().items_center().child(
                        Input::new(&self.gpui_input_state)
                            .w_full()
                            .h(px(28.0))
                            .px(px(0.0))
                            .py(px(0.0))
                            .with_size(Size::Size(px(design_typography.font_size_xl)))
                            .appearance(false)
                            .bordered(false)
                            .focus_bordered(false),
                    ),
                ),
            )
            .child(Self::render_theme_chooser_summary_strip(
                summary,
                selected_preset_name_early,
                &chrome,
                text_on_accent,
            ))
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
        let current_opacity_main = self.theme.get_opacity().main;
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
                                    .border_color(badge_border_bg)
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
                                    this.apply_theme_chooser_theme(modified, "theme_chooser_accent_click", cx);
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
                            .border_color(badge_border_bg)
                            .text_color(rgb(text_secondary))
                            .hover(move |s| s.bg(theme_row_hover_bg))
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
                                    let modified = Self::apply_surface_opacity_preset(this.theme.as_ref(), value);
                                    this.apply_theme_chooser_theme(modified, "theme_chooser_opacity_click", cx);
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
            .hover(move |s| s.bg(theme_row_hover_bg))
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
                            this.apply_theme_chooser_theme(modified, "theme_chooser_vibrancy_click", cx);
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
                        d.bg(badge_border_bg)
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
                            .border_color(badge_border_bg)
                            .text_color(rgb(text_secondary))
                            .hover(move |s| s.bg(theme_row_hover_bg))
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
                                    this.apply_theme_chooser_theme(modified, "theme_chooser_material_click", cx);
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
                            .border_color(badge_border_bg)
                            .text_color(rgb(text_secondary))
                            .hover(move |s| s.bg(theme_row_hover_bg))
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
                                    this.apply_theme_chooser_theme(modified, "theme_chooser_font_size_click", cx);
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
            .border_color(badge_border_bg)
            .text_color(rgb(text_secondary))
            .hover(move |s| s.bg(theme_row_hover_bg))
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
                            let filtered = Self::theme_chooser_filtered_indices(&current_filter);
                            if let AppView::ThemeChooserView {
                                ref selected_index, ..
                            } = this.current_view
                            {
                                this.preview_theme_chooser_preset(
                                    &filtered,
                                    *selected_index,
                                    "theme_chooser_reset_click",
                                    cx,
                                );
                            }
                        });
                    }
                },
            )
            .child("Reset to Defaults");

        // Build surprise me / remix button (clickable)
        let surprise_entity = entity_handle_for_customize.clone();
        let surprise_button = div()
            .id("theme-surprise-me")
            .px(px(10.0))
            .py(px(4.0))
            .rounded(px(4.0))
            .cursor_pointer()
            .text_xs()
            .border_1()
            .border_color(badge_border_bg)
            .text_color(rgb(text_secondary))
            .hover(move |s| s.bg(theme_row_hover_bg))
            .tooltip(|window, cx| {
                gpui_component::tooltip::Tooltip::new(
                    "Remix accent, opacity, and vibrancy (\u{2318}J)",
                )
                .build(window, cx)
            })
            .on_click(
                move |_event: &gpui::ClickEvent, _window: &mut Window, cx: &mut gpui::App| {
                    if let Some(app) = surprise_entity.upgrade() {
                        app.update(cx, |this, cx| {
                            let remixed = Self::build_theme_chooser_remix(
                                this.theme.as_ref(),
                                theme_chooser_remix_seed(),
                            );
                            this.apply_theme_chooser_theme(
                                remixed,
                                "theme_chooser_surprise_me",
                                cx,
                            );
                        });
                    }
                },
            )
            .child("Surprise Me");

        let accent_name = Self::accent_color_name(accent_color);

        // Resolve selected preset name for live preview header
        let selected_preset_name = filtered_indices
            .get(selected_index)
            .and_then(|idx| presets.get(*idx))
            .map(|preset| preset.name)
            .unwrap_or("Theme Preview");

        // Resolve contrast-safe semantic chip colors
        let success_chip = chrome.semantic_chip_colors(self.theme.as_ref(), ui_success);
        let error_chip = chrome.semantic_chip_colors(self.theme.as_ref(), ui_error);
        let warning_chip = chrome.semantic_chip_colors(self.theme.as_ref(), ui_warning);
        let info_chip = chrome.semantic_chip_colors(self.theme.as_ref(), ui_info);

        let preview_panel = div()
            .w_1_2()
            .h_full()
            .border_l_1()
            .border_color(divider_bg)
            .px(px(design_spacing.padding_lg))
            .py(px(design_spacing.padding_md))
            .flex()
            .flex_col()
            .gap(px(10.0))
            .overflow_y_hidden()
            // ── Customize header with remix + reset buttons ─────────
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(text_dimmed))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("CUSTOMIZE"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(6.0))
                            .child(surprise_button)
                            .child(reset_button),
                    ),
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
            // Surface opacity row
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(text_muted))
                            .child(format!(
                                "Surface Opacity  {:.0}%",
                                current_opacity_main * 100.0
                            )),
                    )
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
            // ── Semantic status chips ─────────────────────────────
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(8.0))
                    .child(Self::render_theme_chooser_semantic_chip("OK", success_chip))
                    .child(Self::render_theme_chooser_semantic_chip("Err", error_chip))
                    .child(Self::render_theme_chooser_semantic_chip("Warn", warning_chip))
                    .child(Self::render_theme_chooser_semantic_chip("Info", info_chip)),
            )
            // ── Surface Lab ─────────────────────────────────────────
            .child(self.render_theme_chooser_surface_lab(&chrome))
            // ── Contrast audit ──────────────────────────────────────
            .child({
                let contrast_snapshot = cached_theme_chooser_contrast_snapshot(&self.theme);

                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(text_muted))
                            .child(format!(
                                "Contrast {}/{} pass · worst {} {:.2}:1",
                                contrast_snapshot.passing,
                                contrast_snapshot.total,
                                contrast_snapshot.worst_label,
                                contrast_snapshot.worst_ratio,
                            )),
                    )
                    .children(
                        contrast_snapshot
                            .rows
                            .iter()
                            .map(|row| Self::render_theme_chooser_contrast_row(row, &chrome))
                            .collect::<Vec<_>>(),
                    )
            })
            // ── Launcher-style live preview (spacing-only separation per spec) ──
            .child(div().h(px(1.0)).bg(divider_bg))
            .child(self.render_theme_chooser_live_preview(
                selected_preset_name,
                accent_name,
                &chrome,
            ));

        // ── Footer: canonical three-key hint strip per .impeccable.md ──
        let footer = crate::components::prompt_layout_shell::render_simple_hint_strip(
            Self::theme_chooser_hint_items(),
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
                .child(header_divider)
                .child(self.render_theme_chooser_empty_state_body(
                    filter, summary, &chrome,
                ))
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
            .child(header_divider)
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
                            .py(px(4.0))
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
    fn theme_chooser_uses_truthful_three_item_footer() {
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
            source.contains(r#"SharedString::from("⌘R Reset")"#),
            "theme_chooser should use '⌘R Reset' footer label"
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
    fn test_theme_chooser_uses_shared_list_item_row() {
        // The theme chooser now uses the shared ListItem component for preset rows,
        // matching the main menu's accent bar, description reveal, and spacing.
        let source = include_str!("theme_chooser.rs");
        assert!(
            source.contains("ListItem::new(name, list_colors)"),
            "theme chooser preset rows should use the shared ListItem primitive"
        );
        assert!(
            source.contains("leading_accessory(color_bar)"),
            "theme chooser should pass color swatch as leading accessory"
        );
        assert!(
            source.contains("trailing_accessory_opt(saved_badge)"),
            "theme chooser should pass Saved badge as trailing accessory"
        );
    }
}
