impl TermPrompt {
    /// Render terminal content efficiently by batching consecutive cells with same style.
    /// Instead of creating 2400+ divs (80x30), we batch runs of same-styled text,
    /// typically reducing to ~50-100 elements per frame.
    fn render_content(&self, content: &TerminalContent) -> impl IntoElement {
        let colors = &self.theme.colors;
        // Colors for special cells (cursor, selection) - default cells are transparent for vibrancy
        let cursor_bg = rgb(colors.accent.selected);
        // Use low-opacity for vibrancy support (see VIBRANCY.md)
        let selection_bg = rgba((colors.accent.selected_subtle << 8) | 0x0f); // ~6% opacity
        let default_fg = rgb(colors.text.primary);

        // Convert theme defaults to u32 for comparison with cell colors.
        // This fixes the "default vs explicit black" bug: we compare against
        // actual theme colors instead of hardcoded 0x000000.
        let theme_default_fg = colors.text.primary;
        let theme_default_bg = colors.background.main;

        // Get dynamic font sizing
        let font_size = self.font_size();
        let cell_height = self.cell_height();
        let cell_width = self.cell_width();

        // Build HashSet for O(1) selection lookup
        let selected: HashSet<(usize, usize)> = content.selected_cells.iter().cloned().collect();

        let mut lines_container = div()
            .flex()
            .flex_col()
            .flex_1()
            .size_full() // Both w_full and h_full
            .min_h(px(0.)) // Critical for flex children sizing
            .overflow_hidden()
            // No background - let vibrancy show through from parent
            .font_family("Menlo")
            .text_size(px(font_size))
            .line_height(px(cell_height)); // Use calculated line height for proper descender room

        for (line_idx, cells) in content.styled_lines.iter().enumerate() {
            let is_cursor_line = line_idx == content.cursor_line;

            // Build a row - we'll batch consecutive cells with same styling
            let mut row = div().flex().flex_row().w_full().h(px(cell_height));

            // Batch consecutive cells with same styling
            let mut batch_start = 0;
            while batch_start < cells.len() {
                let first_cell = &cells[batch_start];
                let is_cursor_start = is_cursor_line && batch_start == content.cursor_col;
                let is_selected_start = selected.contains(&(batch_start, line_idx));

                // Get styling for this batch
                let fg_u32 = (first_cell.fg.r as u32) << 16
                    | (first_cell.fg.g as u32) << 8
                    | (first_cell.fg.b as u32);
                let bg_u32 = (first_cell.bg.r as u32) << 16
                    | (first_cell.bg.g as u32) << 8
                    | (first_cell.bg.b as u32);
                let attrs = first_cell.attrs;

                // Find how many consecutive cells have the same styling
                // (excluding cursor position and selection boundaries)
                let mut batch_end = batch_start + 1;

                // If this is the cursor cell, it's always its own batch
                if !is_cursor_start {
                    while batch_end < cells.len() {
                        let cell = &cells[batch_end];
                        let is_cursor_here = is_cursor_line && batch_end == content.cursor_col;
                        let is_selected_here = selected.contains(&(batch_end, line_idx));

                        // Stop if cursor, selection boundary change, or different styling
                        if is_cursor_here || is_selected_here != is_selected_start {
                            break;
                        }

                        let cell_fg =
                            (cell.fg.r as u32) << 16 | (cell.fg.g as u32) << 8 | (cell.fg.b as u32);
                        let cell_bg =
                            (cell.bg.r as u32) << 16 | (cell.bg.g as u32) << 8 | (cell.bg.b as u32);

                        if cell_fg != fg_u32 || cell_bg != bg_u32 || cell.attrs != attrs {
                            break;
                        }

                        batch_end += 1;
                    }
                }

                // Build the text for this batch
                let batch_text: String = cells[batch_start..batch_end]
                    .iter()
                    .map(|c| if c.c == '\0' { ' ' } else { c.c })
                    .collect();

                let batch_width = (batch_end - batch_start) as f32 * cell_width;

                // Check if colors match theme defaults (proper default detection)
                // This fixes the bug where 0x000000 was used as a sentinel,
                // breaking light themes and explicit black colors.
                let is_default_fg = fg_u32 == theme_default_fg;
                let is_default_bg = bg_u32 == theme_default_bg;

                // Determine colors - priority: cursor > selection > custom bg > default (transparent)
                // For vibrancy support, default cells have no background (transparent)
                let (fg_color, bg_color) = if is_cursor_start {
                    // Cursor inverts colors
                    (rgb(bg_u32), Some(cursor_bg))
                } else if is_selected_start {
                    // Selection uses selection background with original foreground
                    (
                        if is_default_fg {
                            default_fg
                        } else {
                            rgb(fg_u32)
                        },
                        Some(selection_bg),
                    )
                } else if !is_default_bg {
                    // Custom background (cell has explicit non-default background)
                    (
                        if is_default_fg {
                            default_fg
                        } else {
                            rgb(fg_u32)
                        },
                        Some(rgb(bg_u32)),
                    )
                } else {
                    // Default colors - no background for vibrancy support
                    (
                        if is_default_fg {
                            default_fg
                        } else {
                            rgb(fg_u32)
                        },
                        None, // Transparent for vibrancy
                    )
                };

                let mut span = div()
                    .w(px(batch_width))
                    .h(px(cell_height))
                    .flex_shrink_0()
                    .when_some(bg_color, |d, bg| d.bg(bg)) // Only apply bg when needed
                    .text_color(fg_color)
                    .child(SharedString::from(batch_text));

                // Apply text attributes
                if attrs.contains(CellAttributes::BOLD) {
                    span = span.font_weight(gpui::FontWeight::BOLD);
                }
                if attrs.contains(CellAttributes::UNDERLINE) {
                    span = span.text_decoration_1();
                }

                row = row.child(span);
                batch_start = batch_end;
            }

            lines_container = lines_container.child(row);
        }

        lines_container
    }
}
