impl RenderOnce for ListItem {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let colors = self.colors;
        let index = self.index;
        let on_hover_callback = self.on_hover;
        let semantic_id = self.semantic_id;

        // Selection colors with alpha from theme opacity settings
        // This allows vibrancy blur to show through selected/hovered items
        // Use rgba8() helper (same pattern as footer) to ensure consistent Hsla conversion
        let selected_alpha = (colors.selected_opacity * 255.0) as u8;
        let hover_alpha = (colors.hover_opacity * 255.0) as u8;
        let selected_bg = colors.accent_selected_subtle.rgba8(selected_alpha);
        let hover_bg = colors.accent_selected_subtle.rgba8(hover_alpha);

        // Icon element (if present) - displayed on the left
        // Supports both emoji strings and PNG image data
        // Icons use slightly muted color to maintain text hierarchy
        let icon_text_color = if self.selected {
            rgb(colors.text_primary)
        } else {
            rgba((colors.text_primary << 8) | ALPHA_ICON_QUIET) // Quiet icons let names lead
        };
        let icon_size = px(ICON_CONTAINER_SIZE);
        let svg_size = px(ICON_SVG_SIZE);
        let icon_element = match &self.icon {
            Some(IconKind::Emoji(emoji)) => div()
                .w(icon_size)
                .h(icon_size)
                .flex()
                .items_center()
                .justify_center()
                .text_sm()
                .text_color(icon_text_color)
                .flex_shrink_0()
                .child(emoji.clone()),
            Some(IconKind::Image(render_image)) => {
                // Render pre-decoded image directly (no decoding on render - critical for perf)
                let image = render_image.clone();
                div()
                    .w(icon_size)
                    .h(icon_size)
                    .flex()
                    .items_center()
                    .justify_center()
                    .flex_shrink_0()
                    .child(
                        img(move |_window: &mut Window, _cx: &mut App| Some(Ok(image.clone())))
                            .w(icon_size)
                            .h(icon_size)
                            .object_fit(ObjectFit::Contain),
                    )
            }
            Some(IconKind::Svg(name)) => {
                // Convert string to IconName and render SVG
                // Use external_path() for file system SVGs (not path() which is for embedded assets)
                if let Some(icon_name) = icon_name_from_str(name) {
                    let svg_path = icon_name.external_path();
                    div()
                        .w(icon_size)
                        .h(icon_size)
                        .flex()
                        .items_center()
                        .justify_center()
                        .flex_shrink_0()
                        .child(
                            svg()
                                .external_path(svg_path)
                                .size(svg_size)
                                .text_color(icon_text_color),
                        )
                } else {
                    // Fallback to Code icon if name not recognized
                    let svg_path = IconName::Code.external_path();
                    div()
                        .w(icon_size)
                        .h(icon_size)
                        .flex()
                        .items_center()
                        .justify_center()
                        .flex_shrink_0()
                        .child(
                            svg()
                                .external_path(svg_path)
                                .size(svg_size)
                                .text_color(icon_text_color),
                        )
                }
            }
            None => {
                div().w(px(0.)).h(px(0.)) // No space if no icon
            }
        };

        // Progressive disclosure: detect if search/filter is active
        // Used to conditionally show descriptions and accessories
        let is_filtering =
            self.highlight_indices.is_some() || self.description_highlight_indices.is_some();

        // Build content with name + description (compact with small gap)
        let mut item_content = div()
            .flex_1()
            .min_w(px(0.))
            .overflow_hidden()
            .flex()
            .flex_col()
            .gap(px(ITEM_NAME_DESC_GAP))
            .justify_center();

        // Name rendering - 14px font size for better balance with description
        // Medium weight for unselected, semibold when selected for clear emphasis
        // When highlight_indices are present, use StyledText to highlight matched characters
        // Otherwise, render as plain text
        let name_weight = if self.selected {
            FontWeight::MEDIUM // Subtle emphasis — launchers rely on background, not weight
        } else {
            FontWeight::NORMAL // Lighter weight reduces visual density
        };
        let name_element = if let Some(ref indices) = self.highlight_indices {
            // Build StyledText with highlighted matched characters
            let index_set: HashSet<usize> = indices.iter().copied().collect();
            let highlight_color = if self.selected {
                rgb(colors.text_primary)
            } else {
                rgba((colors.text_primary << 8) | ALPHA_MATCH_HIGHLIGHT)
            };
            let highlight_style = HighlightStyle {
                color: Some(highlight_color.into()),
                ..Default::default()
            };

            // Convert character indices to byte ranges for StyledText
            let mut highlights: Vec<(std::ops::Range<usize>, HighlightStyle)> = Vec::new();
            for (char_idx, (byte_offset, ch)) in self.name.char_indices().enumerate() {
                if index_set.contains(&char_idx) {
                    highlights.push((byte_offset..byte_offset + ch.len_utf8(), highlight_style));
                }
            }

            // Base text color is more muted when highlighting to create contrast
            let base_color = if self.selected {
                rgba((colors.text_secondary << 8) | ALPHA_HINT)
            } else {
                rgba((colors.text_muted << 8) | ALPHA_NAME_QUIET)
            };

            let styled = StyledText::new(self.name.to_string()).with_highlights(highlights);

            div()
                .text_size(px(NAME_FONT_SIZE))
                .font_weight(name_weight)
                .overflow_hidden()
                .text_ellipsis()
                .whitespace_nowrap()
                .line_height(px(NAME_LINE_HEIGHT))
                .text_color(base_color)
                .child(styled)
        } else {
            // Plain text rendering (no search active)
            // Mute non-selected names to let selected item stand out
            let name_color = if self.selected {
                rgb(colors.text_primary)
            } else {
                rgba((colors.text_primary << 8) | ALPHA_NAME_QUIET)
            };
            div()
                .text_size(px(NAME_FONT_SIZE))
                .font_weight(name_weight)
                .overflow_hidden()
                .text_ellipsis()
                .whitespace_nowrap()
                .line_height(px(NAME_LINE_HEIGHT))
                .text_color(name_color)
                .child(self.name)
        };

        item_content = item_content.child(name_element);

        // Description - progressive disclosure pattern (Spotlight/Raycast style)
        // Search mode keeps rows quieter by showing descriptions only when focused
        // or when the description itself contains a search match.
        if let Some(desc) = self.description {
            let has_description_match = self.description_highlight_indices.is_some();
            let show_description = if is_filtering {
                should_show_search_description(self.selected, self.hovered, has_description_match)
            } else {
                self.selected || self.hovered
            };

            if show_description {
                let desc_alpha = if self.selected {
                    ALPHA_DESC_SELECTED
                } else {
                    ALPHA_DESC_QUIET
                };
                let desc_color = rgba((colors.text_secondary << 8) | desc_alpha);
                let desc_element = if let Some(ref desc_indices) =
                    self.description_highlight_indices
                {
                    // Build StyledText with highlighted matched characters in description
                    let index_set: HashSet<usize> = desc_indices.iter().copied().collect();
                    let highlight_color = if self.selected {
                        rgba((colors.text_primary << 8) | ALPHA_MATCH_HIGHLIGHT)
                    } else {
                        rgba((colors.text_secondary << 8) | ALPHA_HINT)
                    };
                    let highlight_style = HighlightStyle {
                        color: Some(highlight_color.into()),
                        ..Default::default()
                    };

                    // Convert character indices to byte ranges for StyledText
                    let mut highlights: Vec<(std::ops::Range<usize>, HighlightStyle)> = Vec::new();
                    for (char_idx, (byte_offset, ch)) in desc.char_indices().enumerate() {
                        if index_set.contains(&char_idx) {
                            highlights
                                .push((byte_offset..byte_offset + ch.len_utf8(), highlight_style));
                        }
                    }

                    let base_alpha = if self.selected {
                        ALPHA_DESC_SELECTED
                    } else {
                        ALPHA_DESC_QUIET
                    };
                    let base_color = rgba((colors.text_secondary << 8) | base_alpha);
                    let styled = StyledText::new(desc.clone()).with_highlights(highlights);

                    div()
                        .text_size(px(DESC_FONT_SIZE))
                        .line_height(px(DESC_LINE_HEIGHT))
                        .text_color(base_color)
                        .overflow_hidden()
                        .text_ellipsis()
                        .whitespace_nowrap()
                        .child(styled)
                } else {
                    div()
                        .text_size(px(DESC_FONT_SIZE))
                        .line_height(px(DESC_LINE_HEIGHT))
                        .text_color(desc_color)
                        .overflow_hidden()
                        .text_ellipsis()
                        .whitespace_nowrap()
                        .child(desc)
                };
                item_content = item_content.child(desc_element);
            }
        }

        // Shortcut badge (if present) - right-aligned with kbd-style rendering
        // Uses macOS-native modifier symbols (⌘, ⇧, ⌥, ⌃) for a native feel
        let shortcut_element = if let Some(sc) = self.shortcut {
            let show_shortcut =
                should_show_search_shortcut(is_filtering, self.selected, self.hovered);
            if show_shortcut {
                let display_text = format_shortcut_display(&sc);
                if is_filtering {
                    div()
                        .text_size(px(SEARCH_SHORTCUT_FONT_SIZE))
                        .font_family(FONT_MONO)
                        .text_color(rgba((colors.text_dimmed << 8) | ALPHA_HINT))
                        .child(display_text)
                } else {
                    let badge_border = (colors.text_dimmed << 8) | ALPHA_BORDER;
                    div()
                        .text_size(px(BADGE_FONT_SIZE))
                        .font_family(FONT_MONO)
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgba((colors.text_muted << 8) | ALPHA_READABLE))
                        .px(px(BADGE_PADDING_X))
                        .py(px(BADGE_PADDING_Y))
                        .rounded(px(BADGE_RADIUS))
                        .bg(rgba((colors.text_dimmed << 8) | ALPHA_TINT_LIGHT))
                        .border_1()
                        .border_color(rgba(badge_border))
                        .child(display_text)
                }
            } else {
                div()
            }
        } else {
            div()
        };

        // Determine background color based on selection/hover state
        // Priority: selected (full focus styling) > hovered (subtle feedback) > transparent
        // Note: For non-selected items, we ALSO apply GPUI's .hover() modifier for instant feedback
        let bg_color: Hsla = if self.selected {
            selected_bg // 15% opacity - subtle selection with vibrancy
        } else if self.hovered {
            hover_bg // 10% opacity - subtle hover feedback (state-based)
        } else {
            hsla(0.0, 0.0, 0.0, 0.0) // fully transparent
        };

        // Build the inner content div with all styling
        // Horizontal padding ITEM_PADDING_X and vertical padding ITEM_PADDING_Y
        //
        // HOVER TRANSITIONS: We use GPUI's built-in .hover() modifier for instant visual
        // feedback on non-selected items. This provides CSS-like instant hover effects
        // without waiting for state updates via cx.notify().
        //
        // For selected items, we don't apply hover styles (they already have full focus styling).
        let mut inner_content = div()
            .w_full()
            .h_full()
            .px(px(ITEM_PADDING_X))
            .py(px(ITEM_PADDING_Y))
            .bg(bg_color)
            .text_color(rgb(colors.text_primary))
            .font_family(FONT_SYSTEM_UI)
            .cursor_pointer()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(ITEM_ICON_TEXT_GAP))
            .child(icon_element)
            .child(item_content)
            .child({
                // Right-side accessories: [source hint] [type tag] [shortcut badge]
                let mut accessories = div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .flex_shrink_0()
                    .gap(px(ITEM_ACCESSORIES_GAP));

                // Tool badge, source hint, and type tag use progressive disclosure.
                // Search mode intentionally strips noisy metadata to keep rows calm.
                let show_accessories = self.selected || self.hovered || is_filtering;

                // Tool/language badge for scriptlets (e.g., "ts", "bash")
                if show_accessories && !is_filtering {
                    if let Some(ref badge) = self.tool_badge {
                        let badge_bg = (colors.text_dimmed << 8) | ALPHA_TINT_MEDIUM;
                        accessories = accessories.child(
                            div()
                                .text_size(px(TOOL_BADGE_FONT_SIZE))
                                .font_family(FONT_MONO)
                                .text_color(rgba((colors.text_dimmed << 8) | ALPHA_READABLE))
                                .px(px(TOOL_BADGE_PADDING_X))
                                .py(px(TOOL_BADGE_PADDING_Y))
                                .rounded(px(TOOL_BADGE_RADIUS))
                                .bg(rgba(badge_bg))
                                .child(badge.clone()),
                        );
                    }
                }

                // Source/kit hint (e.g., "main", "cleanshot") - very subtle
                if show_accessories && !is_filtering {
                    if let Some(ref hint) = self.source_hint {
                        accessories = accessories.child(
                            div()
                                .text_size(px(SOURCE_HINT_FONT_SIZE))
                                .text_color(rgba((colors.text_dimmed << 8) | ALPHA_HINT))
                                .child(hint.clone()),
                        );
                    }
                }

                // Type tag stays visible during search, but as quiet text instead of a pill badge.
                if let Some(ref tag) = self.type_tag {
                    accessories = accessories.child(
                        div()
                            .text_size(px(TYPE_TAG_FONT_SIZE))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgba((tag.color << 8) | ALPHA_TYPE_LABEL))
                            .child(tag.label),
                    );
                }

                accessories = accessories.child(shortcut_element);
                accessories
            });

        // Apply instant hover effect for non-selected items when hover effects are enabled
        // This provides immediate visual feedback without state updates
        // Hover effects are disabled during keyboard navigation to prevent dual-highlight
        if !self.selected && self.enable_hover_effect {
            inner_content = inner_content.hover(move |s| s.bg(hover_bg));
        }

        // Use semantic_id for element ID if available, otherwise fall back to index
        // This allows AI agents to target elements by their semantic meaning
        let element_id = if let Some(ref sem_id) = semantic_id {
            // Use semantic ID as the element ID for better targeting
            ElementId::Name(sem_id.clone().into())
        } else {
            // Fall back to index-based ID
            let element_idx = index.unwrap_or(0);
            ElementId::NamedInteger("list-item".into(), element_idx as u64)
        };

        // Accent bar: Use LEFT BORDER instead of child div because:
        // 1. GPUI clamps corner radii to ≤ half the shortest side
        // 2. A 3px-wide child with 12px radius gets clamped to ~1.5px (invisible)
        // 3. A border on the container follows rounded corners naturally
        let accent_color = rgb(colors.accent_selected);

        // Base container with ID for stateful interactivity
        // Use left border for accent indicator - always reserve space, toggle color
        let mut container = div()
            .w_full()
            .h(px(LIST_ITEM_HEIGHT))
            .pr(px(ITEM_CONTAINER_PADDING_R))
            .flex()
            .flex_row()
            .items_center()
            .id(element_id);

        // Apply accent bar as left border (only when enabled)
        if self.show_accent_bar {
            container = container
                .border_l(px(ACCENT_BAR_WIDTH))
                .border_color(if self.selected {
                    accent_color
                } else {
                    rgba(0x00000000)
                });
        }

        // Add hover handler if we have both index and callback
        if let (Some(idx), Some(callback)) = (index, on_hover_callback) {
            // Use Rc to allow sharing the callback in the closure
            let callback = std::rc::Rc::new(callback);

            container = container.on_hover(move |hovered: &bool, _window, _cx| {
                // Log the mouse enter/leave event
                if *hovered {
                    logging::log_mouse_enter(idx, None);
                } else {
                    logging::log_mouse_leave(idx, None);
                }
                // Call the user-provided callback
                callback(idx, *hovered);
            });
        }

        // Add content (no separate accent bar child needed)
        container.child(inner_content)
    }
}
/// Decode PNG bytes to GPUI RenderImage
///
/// Decode PNG bytes to a GPUI RenderImage
///
/// Uses the `image` crate to decode PNG data and creates a GPUI-compatible
/// RenderImage for display. Returns an Arc<RenderImage> for caching.
///
/// **IMPORTANT**: Call this ONCE when loading icons, NOT during rendering.
/// Decoding PNGs on every render frame causes severe performance issues.
pub fn decode_png_to_render_image(png_data: &[u8]) -> Result<Arc<RenderImage>, image::ImageError> {
    decode_png_to_render_image_internal(png_data, false)
}
/// Decode PNG bytes to GPUI RenderImage with RGBA→BGRA conversion for Metal
///
/// GPUI/Metal expects BGRA pixel format. When creating RenderImage directly
/// from image::Frame (bypassing GPUI's internal loaders), we must do the
/// RGBA→BGRA conversion ourselves. This matches what GPUI does internally
/// in platform.rs for loaded images.
///
/// **IMPORTANT**: Call this ONCE when loading icons, NOT during rendering.
pub fn decode_png_to_render_image_with_bgra_conversion(
    png_data: &[u8],
) -> Result<Arc<RenderImage>, image::ImageError> {
    decode_png_to_render_image_internal(png_data, true)
}
