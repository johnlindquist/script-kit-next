{

        // Render action list using list() for variable-height items
        // Section headers are 24px, action items are 44px
        let actions_container = if self.grouped_items.is_empty() {
            // Empty state: fixed height matching one action item row
            div()
                .w_full()
                .h(px(ACTION_ITEM_HEIGHT))
                .flex()
                .items_center()
                .px(px(spacing.item_padding_x))
                .text_color(dimmed_text)
                .text_sm()
                .child("No actions match your search")
                .into_any_element()
        } else {
            // Clone data needed for the list closure
            let grouped_items_clone = self.grouped_items.clone();
            let design_variant = self.design_variant;

            // Calculate scrollbar parameters
            // Container height for actions (excluding search box)
            let search_box_height = if self.hide_search {
                0.0
            } else {
                SEARCH_INPUT_HEIGHT
            };

            // Count section headers and items for accurate height calculation
            let mut header_count = 0_usize;
            let mut item_count = 0_usize;
            for item in &self.grouped_items {
                match item {
                    GroupedActionItem::SectionHeader(_) => header_count += 1,
                    GroupedActionItem::Item(_) => item_count += 1,
                }
            }
            let total_content_height = (header_count as f32 * SECTION_HEADER_HEIGHT)
                + (item_count as f32 * ACTION_ITEM_HEIGHT);
            let container_height = total_content_height.min(POPUP_MAX_HEIGHT - search_box_height);

            // Estimate visible items based on average item height
            let avg_item_height = if self.grouped_items.is_empty() {
                ACTION_ITEM_HEIGHT
            } else {
                total_content_height / self.grouped_items.len() as f32
            };
            let visible_items = (container_height / avg_item_height).ceil() as usize;

            // Get scroll offset from list state
            let scroll_offset = self.list_state.logical_scroll_top().item_ix;

            // Get scrollbar colors from theme for consistent styling
            let scrollbar_colors = ScrollbarColors::from_theme(&self.theme);

            // Create scrollbar (only visible if content overflows)
            let scrollbar = Scrollbar::new(
                self.grouped_items.len(),
                visible_items,
                scroll_offset,
                scrollbar_colors,
            )
            .container_height(container_height);

            // Capture entity handle for use in the render closure
            let entity = cx.entity();

            let variable_height_list = list(self.list_state.clone(), move |ix, _window, cx| {
                // Access entity state inside the closure
                entity.update(cx, |this, _cx| {
                    let current_selected = this.selected_index;

                    if let Some(grouped_item) = grouped_items_clone.get(ix) {
                        match grouped_item {
                            GroupedActionItem::SectionHeader(label) => {
                                // Section header at 24px height
                                let header_text = if this.design_variant == DesignVariant::Default {
                                    rgb(this.theme.colors.text.dimmed)
                                } else {
                                    let tokens = get_tokens(this.design_variant);
                                    rgb(tokens.colors().text_dimmed)
                                };
                                let border_color = if this.design_variant == DesignVariant::Default
                                {
                                    rgba(hex_with_alpha(this.theme.colors.ui.border, 0x40))
                                } else {
                                    let tokens = get_tokens(this.design_variant);
                                    rgba(hex_with_alpha(tokens.colors().border, 0x40))
                                };

                                div()
                                    .id(ElementId::NamedInteger("section-header".into(), ix as u64))
                                    .h(px(SECTION_HEADER_HEIGHT))
                                    .w_full()
                                    .px(px(16.0))
                                    .flex()
                                    .items_center()
                                    .when(ix > 0, |d| d.border_t_1().border_color(border_color))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(header_text)
                                            .child(label.clone()),
                                    )
                                    .into_any_element()
                            }
                            GroupedActionItem::Item(filter_idx) => {
                                // Action item at 44px height
                                if let Some(&action_idx) = this.filtered_actions.get(*filter_idx) {
                                    if let Some(action) = this.actions.get(action_idx) {
                                        let is_selected = ix == current_selected;
                                        let filter_ix = *filter_idx;
                                        let show_section_separator = matches!(
                                            this.config.section_style,
                                            SectionStyle::Separators
                                        )
                                            && should_render_section_separator(
                                                &this.actions,
                                                &this.filtered_actions,
                                                filter_ix,
                                            );
                                        let is_destructive = is_destructive_action(action);

                                        // Get tokens for styling
                                        let item_tokens = get_tokens(design_variant);
                                        let item_colors = item_tokens.colors();
                                        let item_spacing = item_tokens.spacing();

                                        // Extract colors for list items - theme-aware selection
                                        // Light mode: Use light gray (like POC: 0xE8E8E8 at 80%)
                                        // Dark mode: Use white at low opacity for subtle brightening
                                        let is_dark_mode = this.theme.should_use_dark_vibrancy();

                                        let (
                                            selected_bg,
                                            hover_bg,
                                            primary_text,
                                            secondary_text,
                                            dimmed_text,
                                        ) = if design_variant == DesignVariant::Default {
                                            // Use theme opacity for both light and dark mode
                                            // Light mode uses same derivation pattern as dark mode
                                            let theme_opacity = this.theme.get_opacity();
                                            let selected_alpha =
                                                (theme_opacity.selected * 255.0) as u32;
                                            let hover_alpha = (theme_opacity.hover * 255.0) as u32;
                                            (
                                                rgba(
                                                    (this.theme.colors.accent.selected_subtle << 8)
                                                        | selected_alpha,
                                                ),
                                                rgba(
                                                    (this.theme.colors.accent.selected_subtle << 8)
                                                        | hover_alpha,
                                                ),
                                                rgb(this.theme.colors.text.primary),
                                                rgb(this.theme.colors.text.secondary),
                                                rgb(this.theme.colors.text.dimmed),
                                            )
                                        } else {
                                            let theme_opacity = this.theme.get_opacity();
                                            let selected_alpha =
                                                (theme_opacity.selected * 255.0) as u32;
                                            let hover_alpha = (theme_opacity.hover * 255.0) as u32;
                                            (
                                                rgba(
                                                    (item_colors.background_selected << 8)
                                                        | selected_alpha,
                                                ),
                                                rgba(
                                                    (item_colors.background_selected << 8)
                                                        | hover_alpha,
                                                ),
                                                rgb(item_colors.text_primary),
                                                rgb(item_colors.text_secondary),
                                                rgb(item_colors.text_dimmed),
                                            )
                                        };

                                        let destructive_text =
                                            if design_variant == DesignVariant::Default {
                                                rgb(this.theme.colors.ui.error)
                                            } else {
                                                rgb(item_colors.error)
                                            };
                                        let destructive_selected_bg =
                                            if design_variant == DesignVariant::Default {
                                                rgba(hex_with_alpha(
                                                    this.theme.colors.ui.error,
                                                    if is_dark_mode { 0x45 } else { 0x2A },
                                                ))
                                            } else {
                                                rgba(hex_with_alpha(
                                                    item_colors.error,
                                                    if is_dark_mode { 0x45 } else { 0x2A },
                                                ))
                                            };
                                        let destructive_hover_bg =
                                            if design_variant == DesignVariant::Default {
                                                rgba(hex_with_alpha(
                                                    this.theme.colors.ui.error,
                                                    if is_dark_mode { 0x2E } else { 0x1F },
                                                ))
                                            } else {
                                                rgba(hex_with_alpha(
                                                    item_colors.error,
                                                    if is_dark_mode { 0x2E } else { 0x1F },
                                                ))
                                            };
                                        let section_separator_color = if design_variant
                                            == DesignVariant::Default
                                        {
                                            rgba(hex_with_alpha(this.theme.colors.ui.border, 0x60))
                                        } else {
                                            rgba(hex_with_alpha(item_colors.border, 0x60))
                                        };

                                        // Title color: bright when selected, secondary when not
                                        let title_color = if is_selected {
                                            primary_text
                                        } else {
                                            secondary_text
                                        };
                                        // Keycap colors: derive from theme for both light and dark mode
                                        // Uses theme border color with appropriate alpha values
                                        let (mut keycap_bg, mut keycap_border, mut shortcut_color) =
                                            if design_variant == DesignVariant::Default {
                                                // Use theme-derived colors for both modes
                                                // Light mode: higher alpha for visibility on light bg
                                                // Dark mode: lower alpha for subtlety on dark bg
                                                let bg_alpha: u8 =
                                                    if is_dark_mode { 0x80 } else { 0xCC };
                                                let border_alpha: u8 =
                                                    if is_dark_mode { 0xA0 } else { 0xDD };
                                                (
                                                    rgba(hex_with_alpha(
                                                        this.theme.colors.ui.border,
                                                        bg_alpha,
                                                    )),
                                                    rgba(hex_with_alpha(
                                                        this.theme.colors.ui.border,
                                                        border_alpha,
                                                    )),
                                                    rgb(this.theme.colors.text.secondary),
                                                )
                                            } else {
                                                (
                                                    rgba(hex_with_alpha(item_colors.border, 0x80)),
                                                    rgba(hex_with_alpha(item_colors.border, 0xA0)),
                                                    dimmed_text,
                                                )
                                            };

                                        let title_color = if is_destructive {
                                            destructive_text
                                        } else {
                                            title_color
                                        };
                                        if is_destructive {
                                            keycap_bg = if design_variant == DesignVariant::Default
                                            {
                                                rgba(hex_with_alpha(
                                                    this.theme.colors.ui.error,
                                                    if is_dark_mode { 0x40 } else { 0x2A },
                                                ))
                                            } else {
                                                rgba(hex_with_alpha(
                                                    item_colors.error,
                                                    if is_dark_mode { 0x40 } else { 0x2A },
                                                ))
                                            };
                                            keycap_border =
                                                if design_variant == DesignVariant::Default {
                                                    rgba(hex_with_alpha(
                                                        this.theme.colors.ui.error,
                                                        if is_dark_mode { 0x90 } else { 0xB0 },
                                                    ))
                                                } else {
                                                    rgba(hex_with_alpha(
                                                        item_colors.error,
                                                        if is_dark_mode { 0x90 } else { 0xB0 },
                                                    ))
                                                };
                                            shortcut_color = destructive_text;
                                        }

                                        // Inner row with pill-style selection
                                        let inner_row = div()
                                            .w_full()
                                            .flex_1()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .px(px(item_spacing.item_padding_x))
                                            .rounded(px(SELECTION_RADIUS))
                                            .bg(if is_selected {
                                                if is_destructive {
                                                    destructive_selected_bg
                                                } else {
                                                    selected_bg
                                                }
                                            } else {
                                                rgba(0x00000000)
                                            })
                                            .hover(move |s| {
                                                s.bg(if is_destructive {
                                                    destructive_hover_bg
                                                } else {
                                                    hover_bg
                                                })
                                            })
                                            .cursor_pointer();

                                        // Content: optional icon + title + shortcuts
                                        let show_icons = this.config.show_icons;
                                        let action_icon = action.icon;

                                        let mut left_side =
                                            div().flex().flex_row().items_center().gap(px(12.0));

                                        // Add icon if enabled and present
                                        if show_icons {
                                            if let Some(icon) = action_icon {
                                                left_side = left_side.child(
                                                    svg()
                                                        .external_path(icon.external_path())
                                                        .size(px(16.0))
                                                        .text_color(if is_destructive {
                                                            destructive_text
                                                        } else if is_selected {
                                                            primary_text
                                                        } else {
                                                            dimmed_text
                                                        }),
                                                );
                                            }
                                        }

                                        // Add title + optional description stack
                                        let mut text_stack =
                                            div().flex().flex_col().justify_center().gap(px(1.0));
                                        text_stack = text_stack.child(
                                            div()
                                                .text_color(title_color)
                                                .text_sm()
                                                .font_weight(if is_selected {
                                                    gpui::FontWeight::MEDIUM
                                                } else {
                                                    gpui::FontWeight::NORMAL
                                                })
                                                .child(action.title.clone()),
                                        );

                                        if let Some(description) =
                                            action_subtitle_for_display(action)
                                        {
                                            text_stack = text_stack.child(
                                                div()
                                                    .text_xs()
                                                    .text_color(if is_selected {
                                                        secondary_text
                                                    } else {
                                                        dimmed_text
                                                    })
                                                    .text_ellipsis()
                                                    .child(description.to_string()),
                                            );
                                        }

                                        left_side = left_side.child(text_stack);

                                        let mut content = div()
                                            .flex_1()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .justify_between()
                                            .child(left_side);

                                        // Right side: keyboard shortcuts as keycaps
                                        if let Some(ref shortcut) = action.shortcut {
                                            let keycaps =
                                                ActionsDialog::parse_shortcut_keycaps(shortcut);
                                            let mut keycap_row =
                                                div().flex().flex_row().items_center().gap(px(3.));

                                            for keycap in keycaps {
                                                keycap_row = keycap_row.child(
                                                    div()
                                                        .min_w(px(KEYCAP_MIN_WIDTH))
                                                        .h(px(KEYCAP_HEIGHT))
                                                        .px(px(6.))
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .bg(keycap_bg)
                                                        .border_1()
                                                        .border_color(keycap_border)
                                                        .rounded(px(5.))
                                                        .text_xs()
                                                        .text_color(shortcut_color)
                                                        .child(keycap),
                                                );
                                            }

                                            content = content.child(keycap_row);
                                        }

                                        div()
                                            .id(ElementId::NamedInteger(
                                                "action-item".into(),
                                                ix as u64,
                                            ))
                                            .h(px(ACTION_ITEM_HEIGHT))
                                            .w_full()
                                            .px(px(ACTION_ROW_INSET))
                                            .py(px(2.0))
                                            .flex()
                                            .flex_col()
                                            .justify_center()
                                            .when(show_section_separator, |d| {
                                                d.border_t_1().border_color(section_separator_color)
                                            })
                                            .child(inner_row.child(content))
                                            .into_any_element()
                                    } else {
                                        // Fallback for missing action
                                        div().h(px(ACTION_ITEM_HEIGHT)).into_any_element()
                                    }
                                } else {
                                    // Fallback for missing filtered index
                                    div().h(px(ACTION_ITEM_HEIGHT)).into_any_element()
                                }
                            }
                        }
                    } else {
                        // Fallback for out-of-bounds index
                        div().h(px(ACTION_ITEM_HEIGHT)).into_any_element()
                    }
                })
            })
            .flex_1()
            .w_full();

            // Wrap list in a relative container with scrollbar overlay
            // Note: Using flex_1() to fill remaining space in flex column.
            // Do NOT use h_full() here as it can conflict with flex layout
            // and cause the search bar to be pushed off-screen.
            div()
                .relative()
                .flex()
                .flex_col()
                .flex_1()
                .w_full()
                .overflow_hidden()
                .child(variable_height_list)
                .child(scrollbar)
                .into_any_element()
        };

        // Use helper method for container colors
        let (main_bg, container_border, container_text) = self.get_container_colors(&colors);

        // Calculate dynamic height based on number of items AND section headers
        // Items are ACTION_ITEM_HEIGHT (44px), section headers are SECTION_HEADER_HEIGHT (24px)
        // Plus search box height (SEARCH_INPUT_HEIGHT), header height, and border
        // NOTE: Must count from grouped_items which includes section headers, not just filtered_actions
        let search_box_height = if self.hide_search {
            0.0
        } else {
            SEARCH_INPUT_HEIGHT
        };
        let header_height = if self.context_title.is_some() {
            HEADER_HEIGHT
        } else {
            0.0
        };
}
