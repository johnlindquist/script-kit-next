impl Render for ActionsDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Get design tokens for the current design variant
        let tokens = get_tokens(self.design_variant);
        let colors = tokens.colors();
        let spacing = tokens.spacing();
        let visual = tokens.visual();

        // NOTE: Key handling is done by the parent (ScriptListApp in main.rs)
        // which routes all keyboard events to this dialog's methods.
        // We do NOT attach our own on_key_down handler to avoid double-processing.

        // Render search input - compact version
        let search_display = if self.search_text.is_empty() {
            SharedString::from("Search actions...")
        } else {
            SharedString::from(self.search_text.clone())
        };

        // Use helper method for design/theme color extraction
        let (_search_box_bg, border_color, _muted_text, dimmed_text, _secondary_text) =
            self.get_search_colors(&colors);

        // Get primary text color for cursor (matches main list styling)
        let primary_text = if self.design_variant == DesignVariant::Default {
            rgb(self.theme.colors.text.primary)
        } else {
            rgb(colors.text_primary)
        };

        // Get accent color for the search input focus indicator
        let accent_color_hex = if self.design_variant == DesignVariant::Default {
            self.theme.colors.accent.selected
        } else {
            colors.accent
        };
        let accent_color = rgb(accent_color_hex);

        // Focus border color (accent with theme-aware transparency)
        // Use border_active opacity for focused state, scaled for visibility
        let opacity = self.theme.get_opacity();
        let focus_border_alpha = ((opacity.border_active * 1.5).min(1.0) * 255.0) as u8;
        let _focus_border_color = rgba(hex_with_alpha(accent_color_hex, focus_border_alpha));

        // Raycast-style footer search input: minimal styling, full-width, top separator line
        // No boxed input field - just text on a clean background with a thin top border
        // Use theme colors for both light and dark mode
        // Light mode derives from the same theme tokens as dark mode
        let separator_color = border_color;
        let hint_text_color = dimmed_text;
        let input_text_color = primary_text;

        let input_container = div()
            .w(px(POPUP_WIDTH)) // Match parent width exactly
            .min_w(px(POPUP_WIDTH))
            .max_w(px(POPUP_WIDTH))
            .h(px(SEARCH_INPUT_HEIGHT)) // Fixed height for the input row
            .min_h(px(SEARCH_INPUT_HEIGHT))
            .max_h(px(SEARCH_INPUT_HEIGHT))
            .overflow_hidden() // Prevent any content from causing shifts
            .px(px(spacing.item_padding_x))
            .py(px(spacing.item_padding_y + 2.0)) // Slightly more vertical padding
            // No background - clean/transparent to match Raycast
            .border_t_1() // Top separator line only
            .border_color(separator_color)
            .flex()
            .flex_row()
            .items_center()
            .child(
                // Full-width search input - no box styling, just text
                div()
                    .flex_1() // Take full width
                    .h(px(28.0))
                    .flex()
                    .flex_row()
                    .items_center()
                    .text_sm()
                    // Placeholder or input text color
                    .text_color(if self.search_text.is_empty() {
                        hint_text_color
                    } else {
                        input_text_color
                    })
                    // Cursor at start when empty
                    .when(self.search_text.is_empty(), |d| {
                        d.child(
                            div()
                                .w(px(2.))
                                .h(px(16.))
                                .mr(px(2.))
                                .rounded(px(1.))
                                .when(self.cursor_visible, |d| d.bg(accent_color)),
                        )
                    })
                    .child(search_display.clone())
                    // Cursor at end when has text
                    .when(!self.search_text.is_empty(), |d| {
                        d.child(
                            div()
                                .w(px(2.))
                                .h(px(16.))
                                .ml(px(2.))
                                .rounded(px(1.))
                                .when(self.cursor_visible, |d| d.bg(accent_color)),
                        )
                    }),
            );

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
        let border_height = visual.border_thin * 2.0; // top + bottom border

        // Count items and section headers separately for accurate height calculation
        let mut section_header_count = 0_usize;
        let mut action_item_count = 0_usize;
        for item in &self.grouped_items {
            match item {
                GroupedActionItem::SectionHeader(_) => section_header_count += 1,
                GroupedActionItem::Item(_) => action_item_count += 1,
            }
        }

        // When no actions, still need space for "No actions match" message
        let min_items_height = if action_item_count == 0 {
            ACTION_ITEM_HEIGHT
        } else {
            0.0
        };

        // Calculate content height including both items and section headers
        let content_height = (action_item_count as f32 * ACTION_ITEM_HEIGHT)
            + (section_header_count as f32 * SECTION_HEADER_HEIGHT);
        let items_height = content_height
            .max(min_items_height)
            .min(POPUP_MAX_HEIGHT - search_box_height - header_height);
        let total_height = items_height + search_box_height + header_height + border_height;

        // Build header row (section header style - non-interactive label)
        // Styled to match render_section_header() from list_item.rs:
        // - Smaller font (text_xs)
        // - Semibold weight
        // - Dimmed color (visually distinct from actionable items)
        let header_container = self.context_title.as_ref().map(|title| {
            let header_text = if self.design_variant == DesignVariant::Default {
                rgb(self.theme.colors.text.dimmed)
            } else {
                rgb(colors.text_dimmed)
            };
            let header_border = if self.design_variant == DesignVariant::Default {
                rgba(hex_with_alpha(self.theme.colors.ui.border, 0x40))
            } else {
                rgba(hex_with_alpha(colors.border, 0x40))
            };

            div()
                .w_full()
                .h(px(HEADER_HEIGHT))
                .px(px(16.0)) // Match section header padding from list_item.rs
                .pt(px(8.0)) // Top padding for visual separation
                .pb(px(4.0)) // Bottom padding
                .flex()
                .flex_col()
                .justify_center()
                .border_b_1()
                .border_color(header_border)
                .child(
                    div()
                        .text_xs() // Smaller font like section headers
                        .font_weight(gpui::FontWeight::SEMIBOLD) // Semibold like section headers
                        .text_color(header_text)
                        .child(title.clone()),
                )
        });

        // Main overlay popup container
        // Fixed width, dynamic height based on content, rounded corners, shadow
        // NOTE: Using visual.radius_lg from design tokens for consistency with child item rounding
        //
        // VIBRANCY: Background is handled in get_container_colors() with vibrancy-aware opacity
        // (~50% when vibrancy enabled, ~95% when disabled)

        // Build footer with keyboard hints (if enabled)
        let footer_height = if self.config.show_footer { 32.0 } else { 0.0 };
        let footer_container = if self.config.show_footer {
            let footer_text = if self.design_variant == DesignVariant::Default {
                rgb(self.theme.colors.text.dimmed)
            } else {
                rgb(colors.text_dimmed)
            };
            let footer_border = if self.design_variant == DesignVariant::Default {
                rgba(hex_with_alpha(self.theme.colors.ui.border, 0x40))
            } else {
                rgba(hex_with_alpha(colors.border, 0x40))
            };

            Some(
                div()
                    .w_full()
                    .h(px(32.0))
                    .px(px(16.0))
                    .border_t_1()
                    .border_color(footer_border)
                    .flex()
                    .items_center()
                    .gap(px(16.0))
                    .text_xs()
                    .text_color(footer_text)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(4.0))
                            .child("↑↓")
                            .child("Navigate"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(4.0))
                            .child("↵")
                            .child("Select"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(4.0))
                            .child("esc")
                            .child("Close"),
                    ),
            )
        } else {
            None
        };

        // Recalculate total height including footer
        let total_height = total_height + footer_height;

        // Get search position from config
        let search_at_top = matches!(self.config.search_position, SearchPosition::Top);
        let show_search =
            !matches!(self.config.search_position, SearchPosition::Hidden) && !self.hide_search;

        // Top-positioned search input - clean Raycast-style matching the bottom search
        // No boxed input field, no ⌘K prefix - just text on a clean background with bottom separator
        let input_container_top = if search_at_top && show_search {
            Some(
                div()
                    .w(px(POPUP_WIDTH)) // Match parent width exactly
                    .min_w(px(POPUP_WIDTH))
                    .max_w(px(POPUP_WIDTH))
                    .h(px(SEARCH_INPUT_HEIGHT)) // Fixed height for the input row
                    .min_h(px(SEARCH_INPUT_HEIGHT))
                    .max_h(px(SEARCH_INPUT_HEIGHT))
                    .overflow_hidden() // Prevent any content from causing shifts
                    .px(px(spacing.item_padding_x))
                    .py(px(spacing.item_padding_y + 2.0)) // Slightly more vertical padding
                    // No background - clean/transparent to match Raycast
                    .border_b_1() // Bottom separator line
                    .border_color(separator_color)
                    .flex()
                    .flex_row()
                    .items_center()
                    .child(
                        // Full-width search input - no box styling, just text
                        div()
                            .flex_1() // Take full width
                            .h(px(28.0))
                            .flex()
                            .flex_row()
                            .items_center()
                            .text_sm()
                            // Placeholder or input text color
                            .text_color(if self.search_text.is_empty() {
                                hint_text_color
                            } else {
                                input_text_color
                            })
                            // Cursor at start when empty
                            .when(self.search_text.is_empty(), |d| {
                                d.child(
                                    div()
                                        .w(px(2.))
                                        .h(px(16.))
                                        .mr(px(2.))
                                        .rounded(px(1.))
                                        .when(self.cursor_visible, |d| d.bg(accent_color)),
                                )
                            })
                            .child(search_display.clone())
                            // Cursor at end when has text
                            .when(!self.search_text.is_empty(), |d| {
                                d.child(
                                    div()
                                        .w(px(2.))
                                        .h(px(16.))
                                        .ml(px(2.))
                                        .rounded(px(1.))
                                        .when(self.cursor_visible, |d| d.bg(accent_color)),
                                )
                            }),
                    ),
            )
        } else {
            None
        };

        div()
            .flex()
            .flex_col()
            .w(px(POPUP_WIDTH))
            .h(px(total_height)) // Use calculated height including footer
            .bg(main_bg) // Always apply background with vibrancy-aware opacity
            .rounded(px(visual.radius_lg))
            .shadow(Self::create_popup_shadow())
            .border_1()
            .border_color(container_border)
            .overflow_hidden()
            .text_color(container_text)
            .key_context("actions_dialog")
            // Only track focus if not delegated to parent (ActionsWindow sets skip_track_focus=true)
            .when(!self.skip_track_focus, |d| {
                d.track_focus(&self.focus_handle)
            })
            // NOTE: No on_key_down here - parent handles all keyboard input
            // Search input at top (if config.search_position == Top)
            .when_some(input_container_top, |d, input| d.child(input))
            // Header row (if context_title is set)
            .when_some(header_container, |d, header| d.child(header))
            // Actions list
            .child(actions_container)
            // Search input at bottom (if config.search_position == Bottom)
            .when(show_search && !search_at_top, |d| d.child(input_container))
            // Footer with keyboard hints (if config.show_footer)
            .when_some(footer_container, |d, footer| d.child(footer))
    }
}
