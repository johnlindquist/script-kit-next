// Shared gallery-item enumeration for DesignGallery — used by both the renderer
// (`render_design_gallery`) and the state-introspection path
// (`collect_state::AppView::DesignGalleryView` in `src/prompt_handler/mod.rs`)
// so `stateResult.visibleChoiceCount` reflects the same filtered count the UI
// actually renders. Pinned by `tests/design_gallery_state_choice_count_asymmetry_contract.rs`.
#[derive(Clone, Debug)]
pub(crate) enum GalleryItem {
    GroupHeaderCategory(designs::group_header_variations::GroupHeaderCategory),
    GroupHeader(designs::group_header_variations::GroupHeaderStyle),
    IconCategoryHeader(designs::icon_variations::IconCategory),
    Icon(
        designs::icon_variations::IconName,
        designs::icon_variations::IconStyle,
    ),
}

pub(crate) fn build_gallery_items() -> Vec<GalleryItem> {
    use designs::group_header_variations::GroupHeaderCategory;
    use designs::icon_variations::{IconCategory, IconStyle};

    let mut items: Vec<GalleryItem> = Vec::new();
    for category in GroupHeaderCategory::all() {
        items.push(GalleryItem::GroupHeaderCategory(*category));
        for style in category.styles() {
            items.push(GalleryItem::GroupHeader(*style));
        }
    }
    for category in IconCategory::all() {
        items.push(GalleryItem::IconCategoryHeader(*category));
        for icon in category.icons() {
            items.push(GalleryItem::Icon(icon, IconStyle::Default));
        }
    }
    items
}

pub(crate) fn gallery_item_matches(item: &GalleryItem, filter_lower: &str) -> bool {
    match item {
        GalleryItem::GroupHeaderCategory(cat) => cat.name().to_lowercase().contains(filter_lower),
        GalleryItem::GroupHeader(style) => {
            style.name().to_lowercase().contains(filter_lower)
                || style.description().to_lowercase().contains(filter_lower)
        }
        GalleryItem::IconCategoryHeader(cat) => cat.name().to_lowercase().contains(filter_lower),
        GalleryItem::Icon(icon, _) => {
            icon.name().to_lowercase().contains(filter_lower)
                || icon.description().to_lowercase().contains(filter_lower)
        }
    }
}

pub(crate) fn design_gallery_total_items() -> usize {
    build_gallery_items().len()
}

/// Single display label per [`GalleryItem`], shared by the renderer and
/// the `collect_visible_elements::DesignGalleryView` arm so getElements
/// row strings match what the user sees. Uses the same `.name()` field
/// `gallery_item_matches` filters on, so filter hits produce matching
/// row text without drift.
pub(crate) fn design_gallery_item_label(item: &GalleryItem) -> String {
    match item {
        GalleryItem::GroupHeaderCategory(cat) => cat.name().to_string(),
        GalleryItem::GroupHeader(style) => style.name().to_string(),
        GalleryItem::IconCategoryHeader(cat) => cat.name().to_string(),
        GalleryItem::Icon(icon, _) => icon.name().to_string(),
    }
}

pub(crate) fn design_gallery_filtered_len(filter: &str) -> usize {
    if filter.is_empty() {
        return design_gallery_total_items();
    }
    let filter_lower = filter.to_lowercase();
    build_gallery_items()
        .iter()
        .filter(|item| gallery_item_matches(item, &filter_lower))
        .count()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DesignGalleryEmptyState {
    EmptyCatalog,
    NoFilterMatches,
}

impl DesignGalleryEmptyState {
    fn from_filter(filter: &str) -> Self {
        if filter.trim().is_empty() {
            Self::EmptyCatalog
        } else {
            Self::NoFilterMatches
        }
    }

    fn message(self) -> &'static str {
        match self {
            Self::EmptyCatalog => "No design variations available",
            Self::NoFilterMatches => "No designs match your filter",
        }
    }
}

impl ScriptListApp {
    fn design_gallery_visible_rows(filter: &str) -> Vec<GalleryItem> {
        let items = build_gallery_items();
        if filter.is_empty() {
            items
        } else {
            let filter_lower = filter.to_lowercase();
            items
                .into_iter()
                .filter(|item| gallery_item_matches(item, &filter_lower))
                .collect()
        }
    }

    fn design_gallery_selected_visible_row(
        filter: &str,
        selected_index: usize,
    ) -> Option<GalleryItem> {
        Self::design_gallery_visible_rows(filter)
            .get(selected_index)
            .cloned()
    }

    fn design_gallery_dataset_and_visible_counts(filter: &str) -> (usize, usize) {
        (
            design_gallery_total_items(),
            Self::design_gallery_visible_rows(filter).len(),
        )
    }

    fn design_gallery_visible_row_labels(filter: &str) -> Vec<String> {
        Self::design_gallery_visible_rows(filter)
            .iter()
            .map(design_gallery_item_label)
            .collect()
    }

    fn design_gallery_count_label(filtered_len: usize) -> String {
        let suffix = if filtered_len == 1 { "" } else { "s" };
        format!("{} item{}", filtered_len, suffix)
    }

    /// Render design gallery view with group header and icon variations
    fn render_design_gallery(
        &mut self,
        filter: String,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        // Use design tokens for GLOBAL theming
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();
        let color_resolver =
            crate::theme::ColorResolver::new_for_shell(&self.theme, self.current_design);
        let typography_resolver =
            crate::theme::TypographyResolver::new_theme_first(&self.theme, self.current_design);
        let empty_text_color = color_resolver.empty_text_color();
        let empty_font_family = typography_resolver.primary_font().to_string();

        // Use design tokens for global theming
        let opacity = self.theme.get_opacity();
        let bg_hex = self.theme.colors.background.main;
        let _bg_with_alpha = crate::ui_foundation::hex_to_rgba_with_opacity(bg_hex, opacity.main);
        // Removed: box_shadows - shadows on transparent elements block vibrancy
        let _box_shadows = self.create_box_shadows();

        // Build gallery items via the shared helper so collect_state (the
        // `stateResult` receipt path) and this renderer stay in lock-step.
        let gallery_items = build_gallery_items();

        // Filter items based on current filter
        let filtered_items: Vec<(usize, GalleryItem)> = if filter.is_empty() {
            gallery_items
                .iter()
                .enumerate()
                .map(|(i, item)| (i, item.clone()))
                .collect()
        } else {
            let filter_lower = filter.to_lowercase();
            gallery_items
                .iter()
                .enumerate()
                .filter(|(_, item)| gallery_item_matches(item, &filter_lower))
                .map(|(i, item)| (i, item.clone()))
                .collect()
        };
        let filtered_len = filtered_items.len();

        // Key handler for design gallery
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                // Hide cursor while typing - automatically shows when mouse moves
                this.hide_mouse_cursor(cx);

                // If the shortcut recorder is active, don't process any key events.
                // The recorder has its own key handlers and should receive all key events.
                if this.shortcut_recorder_state.is_some() {
                    return;
                }

                let key = event.keystroke.key.as_str();
                let has_cmd = event.keystroke.modifiers.platform;

                // ESC: Clear filter first if present, otherwise go back/close
                if is_key_escape(key) && !this.show_actions_popup {
                    if !this.clear_builtin_view_filter(cx) {
                        this.go_back_or_close(window, cx);
                    }
                    return;
                }

                // Cmd+W always closes window
                if has_cmd && key.eq_ignore_ascii_case("w") {
                    logging::log("KEY", "Cmd+W - closing window");
                    this.close_and_reset_window(cx);
                    return;
                }

                logging::log("KEY", &format!("DesignGallery key: '{}'", key));

                if let AppView::DesignGalleryView { selected_index, .. } = &mut this.current_view {
                    // Use the filtered count captured at render time (not total count)
                    // so arrow keys respect the visible items after filtering
                    let current_filtered_len = filtered_len;

                    match key {
                        _ if is_key_up(key) => {
                            if *selected_index > 0 {
                                *selected_index -= 1;
                                this.design_gallery_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                                cx.notify();
                            }
                        }
                        _ if is_key_down(key) => {
                            if *selected_index < current_filtered_len.saturating_sub(1) {
                                *selected_index += 1;
                                this.design_gallery_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                                cx.notify();
                            }
                        }
                        // Text editing is owned by the shared GPUI input and
                        // routed through InputEvent::Change in startup.rs.
                        _ => {}
                    }
                }
            },
        );

        // Pre-compute colors - use theme for consistency with main menu
        let text_primary = self.theme.colors.text.primary;

        // Build virtualized list
        let list_element: AnyElement = if filtered_len == 0 {
            let empty_state = DesignGalleryEmptyState::from_filter(&filter);
            crate::list_item::EmptyState::new(
                empty_state.message(),
                empty_text_color,
                &empty_font_family,
            )
            .icon(crate::designs::icon_variations::IconName::StarFilled)
            .into_element()
        } else {
            // Clone data for the closure
            let items_for_closure = filtered_items.clone();
            let selected = selected_index;
            let design_spacing_clone = design_spacing;
            let design_typography_clone = design_typography;
            let design_visual_clone = design_visual;
            let design_colors_clone = design_colors;

            uniform_list(
                "design-gallery",
                filtered_len,
                move |visible_range, _window, _cx| {
                    visible_range
                        .map(|ix| {
                            if let Some((_, item)) = items_for_closure.get(ix) {
                                let is_selected = ix == selected;

                                let element: AnyElement = match item {
                                    GalleryItem::GroupHeaderCategory(category) => {
                                        // Category header - styled as section header
                                        div()
                                            .id(ElementId::NamedInteger(
                                                "gallery-header-cat".into(),
                                                ix as u64,
                                            ))
                                            .w_full()
                                            .h(px(32.0))
                                            .px(px(design_spacing_clone.padding_lg))
                                            .flex()
                                            .items_center()
                                            .bg(rgba(
                                                (design_colors_clone.background_secondary << 8)
                                                    | 0x80,
                                            ))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::BOLD)
                                                    .text_color(rgb(design_colors_clone.accent))
                                                    .child(format!(
                                                        "── Group Headers: {} ──",
                                                        category.name()
                                                    )),
                                            )
                                            .into_any_element()
                                    }
                                    GalleryItem::GroupHeader(style) => render_group_header_item(
                                        ix,
                                        is_selected,
                                        style,
                                        &design_spacing_clone,
                                        &design_typography_clone,
                                        &design_visual_clone,
                                        &design_colors_clone,
                                    ),
                                    GalleryItem::IconCategoryHeader(category) => {
                                        // Icon category header
                                        div()
                                            .id(ElementId::NamedInteger(
                                                "gallery-icon-cat".into(),
                                                ix as u64,
                                            ))
                                            .w_full()
                                            .h(px(32.0))
                                            .px(px(design_spacing_clone.padding_lg))
                                            .flex()
                                            .items_center()
                                            .bg(rgba(
                                                (design_colors_clone.background_secondary << 8)
                                                    | 0x80,
                                            ))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::BOLD)
                                                    .text_color(rgb(design_colors_clone.accent))
                                                    .child(format!(
                                                        "── Icons: {} ──",
                                                        category.name()
                                                    )),
                                            )
                                            .into_any_element()
                                    }
                                    GalleryItem::Icon(icon, _style) => {
                                        // Render icon item with SVG
                                        let icon_path = icon.external_path();
                                        let name_owned = icon.name().to_string();
                                        let desc_owned = icon.description().to_string();

                                        let mut item_div = div()
                                            .id(ElementId::NamedInteger(
                                                "gallery-icon".into(),
                                                ix as u64,
                                            ))
                                            .w_full()
                                            .h(px(LIST_ITEM_HEIGHT))
                                            .px(px(design_spacing_clone.padding_lg))
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap(px(design_spacing_clone.gap_md));

                                        if is_selected {
                                            // Use low-opacity for vibrancy support (see VIBRANCY.md)
                                            item_div = item_div.bg(rgba(
                                                (design_colors_clone.background_selected << 8)
                                                    | 0x0f,
                                            )); // ~6% opacity
                                        }

                                        item_div
                                            // Icon preview with SVG
                                            .child(
                                                div()
                                                    .w(px(32.0))
                                                    .h(px(32.0))
                                                    .rounded(px(4.0))
                                                    .bg(rgba(
                                                        (design_colors_clone.background_secondary
                                                            << 8)
                                                            | 0x60,
                                                    ))
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .child(
                                                        svg()
                                                            .external_path(icon_path)
                                                            .size(px(16.0))
                                                            .text_color(rgb(
                                                                design_colors_clone.text_primary
                                                            )),
                                                    ),
                                            )
                                            // Name and description
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .flex()
                                                    .flex_col()
                                                    .gap(px(2.0))
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .font_weight(gpui::FontWeight::MEDIUM)
                                                            .text_color(rgb(
                                                                design_colors_clone.text_primary
                                                            ))
                                                            .child(name_owned),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(rgb(
                                                                design_colors_clone.text_muted
                                                            ))
                                                            .overflow_x_hidden()
                                                            .child(desc_owned),
                                                    ),
                                            )
                                            .into_any_element()
                                    }
                                };
                                element
                            } else {
                                div()
                                    .id(ElementId::NamedInteger("gallery-empty".into(), ix as u64))
                                    .h(px(LIST_ITEM_HEIGHT))
                                    .into_any_element()
                            }
                        })
                        .collect()
                },
            )
            .w_full()
            .h_full()
            .track_scroll(&self.design_gallery_scroll_handle)
            .into_any_element()
        };

        let footer_hints: Vec<SharedString> = vec!["↵ Select".into()];
        crate::components::emit_surface_prompt_hint_audit(
            "design_gallery",
            &footer_hints,
            "design_gallery_footer",
        );
        let footer = div()
            .id("design-gallery-footer-tooltip")
            .tooltip(|window, cx| {
                gpui_component::tooltip::Tooltip::new("Select highlighted design")
                    .key_binding(
                        gpui::Keystroke::parse("enter")
                            .ok()
                            .map(gpui_component::kbd::Kbd::new),
                    )
                    .build(window, cx)
            })
            .child(crate::components::render_simple_hint_strip(
                footer_hints,
                None,
            ))
            .into_any_element();
        let footer = self.main_window_footer_slot(footer);

        let content = div()
            .flex_1()
            .w_full()
            .h_full()
            .min_h(px(0.))
            .overflow_hidden()
            .py(px(design_spacing.padding_xs))
            .text_color(rgb(text_primary))
            .font_family(design_typography.font_family)
            .child(list_element);

        let menu_def = self.current_main_menu_theme.def();
        let shell = menu_def.shell;

        crate::components::main_view_chrome::render_main_view_chrome(
            crate::components::main_view_chrome::render_main_view_shell()
                .key_context("design_gallery")
                .track_focus(&self.focus_handle)
                .on_key_down(handle_key),
            &self.theme,
            menu_def,
            crate::components::main_view_chrome::MainViewChrome {
                header: self.render_builtin_main_input_header(vec![
                    self.render_builtin_main_input_count_label(Self::design_gallery_count_label(
                        filtered_len,
                    )),
                ], cx),
                divider: crate::components::main_view_chrome::MainViewDividerChrome {
                    margin_x: shell.divider_margin_x,
                    height: shell.divider_height,
                    visible: shell.divider_height > 0.0,
                },
                main: content.into_any_element(),
                footer,
                overlays: Vec::new(),
            },
        )
    }
}
