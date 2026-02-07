{
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
