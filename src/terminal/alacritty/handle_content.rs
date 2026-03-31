use alacritty_terminal::index::{Column, Line, Point as AlacPoint};
use alacritty_terminal::term::cell::Flags as AlacrittyFlags;
use tracing::instrument;

use super::*;

impl TerminalHandle {
    /// Gets the current terminal content for rendering.
    ///
    /// This method creates a snapshot of the visible terminal content,
    /// including the cursor position, styled cells with colors and attributes.
    /// It's designed to be called from the render loop.
    ///
    /// # Returns
    ///
    /// A `TerminalContent` struct containing lines, styled cells, and cursor info.
    #[instrument(level = "trace", skip(self))]
    pub fn content(&self) -> TerminalContent {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let grid = state.term.grid();
        let display_offset = grid.display_offset() as i32;

        let mut lines = Vec::with_capacity(state.term.screen_lines());
        let mut styled_lines = Vec::with_capacity(state.term.screen_lines());

        let selection_range = state
            .term
            .selection
            .as_ref()
            .and_then(|sel| sel.to_range(&state.term));

        let mut selected_cells = Vec::new();

        for line_idx in 0..state.term.screen_lines() {
            // Negative Line indices go into scrollback history.
            // When display_offset > 0, shift the view upward into history.
            let grid_line = Line(line_idx as i32 - display_offset);
            let row = &grid[grid_line];
            let mut line_str = String::with_capacity(state.term.columns());
            let mut styled_row = Vec::with_capacity(state.term.columns());

            for col_idx in 0..state.term.columns() {
                let cell = &row[Column(col_idx)];
                line_str.push(cell.c);

                let is_bold = cell.flags.contains(AlacrittyFlags::BOLD);
                let fg = resolve_fg_color_with_bold(&cell.fg, is_bold, &self.theme);
                let bg = resolve_color(&cell.bg, &self.theme);
                let attrs = CellAttributes::from_alacritty_flags(cell.flags);

                styled_row.push(TerminalCell {
                    c: cell.c,
                    fg,
                    bg,
                    attrs,
                });

                if let Some(ref range) = selection_range {
                    let point = AlacPoint::new(grid_line, Column(col_idx));
                    if range.contains(point) {
                        selected_cells.push((col_idx, line_idx));
                    }
                }
            }

            let trimmed = line_str.trim_end();
            lines.push(trimmed.to_string());
            styled_lines.push(styled_row);
        }

        let cursor = grid.cursor.point;
        // Adjust cursor position for display offset — if scrolled up,
        // the cursor may be below the visible viewport.
        let cursor_line_in_view = cursor.line.0 + display_offset;

        TerminalContent {
            lines,
            styled_lines,
            cursor_line: cursor_line_in_view.max(0) as usize,
            cursor_col: cursor.column.0,
            selected_cells,
        }
    }
}
