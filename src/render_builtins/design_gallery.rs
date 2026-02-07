impl ScriptListApp {
    /// Render design gallery view with group header and icon variations
    fn render_design_gallery(
        &mut self,
        filter: String,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        use designs::group_header_variations::{GroupHeaderCategory, GroupHeaderStyle};
        use designs::icon_variations::{IconCategory, IconName, IconStyle};

        // Use design tokens for GLOBAL theming
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();

        // Use design tokens for global theming
        let opacity = self.theme.get_opacity();
        let bg_hex = self.theme.colors.background.main;
        let _bg_with_alpha = crate::ui_foundation::hex_to_rgba_with_opacity(bg_hex, opacity.main);
        // Removed: box_shadows - shadows on transparent elements block vibrancy
        let _box_shadows = self.create_box_shadows();

        // Build gallery items: group headers grouped by category, then icons grouped by category
        #[derive(Clone)]
        enum GalleryItem {
            GroupHeaderCategory(GroupHeaderCategory),
            GroupHeader(GroupHeaderStyle),
            IconCategoryHeader(IconCategory),
            Icon(IconName, IconStyle),
        }

        let mut gallery_items: Vec<GalleryItem> = Vec::new();

        // Add group headers by category
        for category in GroupHeaderCategory::all() {
            gallery_items.push(GalleryItem::GroupHeaderCategory(*category));
            for style in category.styles() {
                gallery_items.push(GalleryItem::GroupHeader(*style));
            }
        }

        // Add icons by category, showing each icon with default style
        for category in IconCategory::all() {
            gallery_items.push(GalleryItem::IconCategoryHeader(*category));
            for icon in category.icons() {
                gallery_items.push(GalleryItem::Icon(icon, IconStyle::Default));
            }
        }

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
                .filter(|(_, item)| match item {
                    GalleryItem::GroupHeaderCategory(cat) => {
                        cat.name().to_lowercase().contains(&filter_lower)
                    }
                    GalleryItem::GroupHeader(style) => {
                        style.name().to_lowercase().contains(&filter_lower)
                            || style.description().to_lowercase().contains(&filter_lower)
                    }
                    GalleryItem::IconCategoryHeader(cat) => {
                        cat.name().to_lowercase().contains(&filter_lower)
                    }
                    GalleryItem::Icon(icon, _) => {
                        icon.name().to_lowercase().contains(&filter_lower)
                            || icon.description().to_lowercase().contains(&filter_lower)
                    }
                })
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

                let key_str = event.keystroke.key.to_lowercase();
                let has_cmd = event.keystroke.modifiers.platform;

                // ESC: Clear filter first if present, otherwise go back/close
                if key_str == "escape" && !this.show_actions_popup {
                    if !this.clear_builtin_view_filter(cx) {
                        this.go_back_or_close(window, cx);
                    }
                    return;
                }

                // Cmd+W always closes window
                if has_cmd && key_str == "w" {
                    logging::log("KEY", "Cmd+W - closing window");
                    this.close_and_reset_window(cx);
                    return;
                }

                logging::log("KEY", &format!("DesignGallery key: '{}'", key_str));

                if let AppView::DesignGalleryView {
                    filter,
                    selected_index,
                } = &mut this.current_view
                {
                    // Use the filtered count captured at render time (not total count)
                    // so arrow keys respect the visible items after filtering
                    let current_filtered_len = filtered_len;

                    match key_str.as_str() {
                        "up" | "arrowup" => {
                            if *selected_index > 0 {
                                *selected_index -= 1;
                                this.design_gallery_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                                cx.notify();
                            }
                        }
                        "down" | "arrowdown" => {
                            if *selected_index < current_filtered_len.saturating_sub(1) {
                                *selected_index += 1;
                                this.design_gallery_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                                cx.notify();
                            }
                        }
                        // Note: "escape" is handled by handle_global_shortcut_with_options above
                        "backspace" => {
                            if !filter.is_empty() {
                                filter.pop();
                                *selected_index = 0;
                                this.design_gallery_scroll_handle
                                    .scroll_to_item(0, ScrollStrategy::Top);
                                cx.notify();
                            }
                        }
                        _ => {
                            if let Some(ref key_char) = event.keystroke.key_char {
                                if let Some(ch) = key_char.chars().next() {
                                    if !ch.is_control() {
                                        filter.push(ch);
                                        *selected_index = 0;
                                        this.design_gallery_scroll_handle
                                            .scroll_to_item(0, ScrollStrategy::Top);
                                        cx.notify();
                                    }
                                }
                            }
                        }
                    }
                }
            },
        );

        let input_display = if filter.is_empty() {
            SharedString::from("Search design variations...")
        } else {
            SharedString::from(filter.clone())
        };
        let input_is_empty = filter.is_empty();

        // Pre-compute colors - use theme for consistency with main menu
        let list_colors = ListItemColors::from_theme(&self.theme);
        let text_primary = self.theme.colors.text.primary;
        #[allow(unused_variables)]
        let text_muted = self.theme.colors.text.muted;
        let text_dimmed = self.theme.colors.text.dimmed;
        let ui_border = self.theme.colors.ui.border;

        // Build virtualized list
        let list_element: AnyElement = if filtered_len == 0 {
            div()
                .w_full()
                .py(px(design_spacing.padding_xl))
                .text_center()
                .text_color(rgb(self.theme.colors.text.muted))
                .font_family(design_typography.font_family)
                .child("No items match your filter")
                .into_any_element()
        } else {
            // Clone data for the closure
            let items_for_closure = filtered_items.clone();
            let selected = selected_index;
            let _list_colors_clone = list_colors; // Kept for future use
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

        // Build the full view
        div()
            .flex()
            .flex_col()
            // Removed: .bg(rgba(bg_with_alpha)) - let vibrancy show through from Root
            // Removed: .shadow(box_shadows) - shadows on transparent elements block vibrancy
            .w_full()
            .h_full()
            .rounded(px(design_visual.radius_lg))
            .text_color(rgb(text_primary))
            .font_family(design_typography.font_family)
            .key_context("design_gallery")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            // Header with input
            .child(
                div()
                    .w_full()
                    .px(px(design_spacing.padding_lg))
                    .py(px(design_spacing.padding_md))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    // Gallery icon
                    .child(div().text_xl().child("🎨"))
                    // Search input with blinking cursor
                    // ALIGNMENT FIX: Uses canonical cursor constants and negative margin for placeholder
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_row()
                            .items_center()
                            .text_lg()
                            .text_color(if input_is_empty {
                                rgb(text_muted)
                            } else {
                                rgb(text_primary)
                            })
                            .when(input_is_empty, |d| {
                                d.child(
                                    div()
                                        .w(px(CURSOR_WIDTH))
                                        .h(px(CURSOR_HEIGHT_LG))
                                        .my(px(CURSOR_MARGIN_Y))
                                        .mr(px(CURSOR_GAP_X))
                                        .when(self.cursor_visible, |d| d.bg(rgb(text_primary))),
                                )
                            })
                            .when(input_is_empty, |d| {
                                d.child(
                                    div()
                                        .ml(px(-(CURSOR_WIDTH + CURSOR_GAP_X)))
                                        .child(input_display.clone()),
                                )
                            })
                            .when(!input_is_empty, |d| d.child(input_display.clone()))
                            .when(!input_is_empty, |d| {
                                d.child(
                                    div()
                                        .w(px(CURSOR_WIDTH))
                                        .h(px(CURSOR_HEIGHT_LG))
                                        .my(px(CURSOR_MARGIN_Y))
                                        .ml(px(CURSOR_GAP_X))
                                        .when(self.cursor_visible, |d| d.bg(rgb(text_primary))),
                                )
                            }),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(text_dimmed))
                            .child(format!("{} items", filtered_len)),
                    ),
            )
            // Divider
            .child(
                div()
                    .mx(px(design_spacing.padding_lg))
                    .h(px(design_visual.border_thin))
                    .bg(rgba((ui_border << 8) | 0x60)),
            )
            // Main content area - just the list (no preview panel for gallery)
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .h_full()
                    .min_h(px(0.))
                    .overflow_hidden()
                    .py(px(design_spacing.padding_xs))
                    .child(list_element),
            )
            // Footer
            .child(PromptFooter::new(
                PromptFooterConfig::new()
                    .primary_label("Select")
                    .primary_shortcut("↵")
                    .show_secondary(false),
                PromptFooterColors::from_theme(&self.theme),
            ))
            .into_any_element()
    }
}
