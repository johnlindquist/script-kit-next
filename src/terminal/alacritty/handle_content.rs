use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::index::{Column, Line, Point as AlacPoint};
use alacritty_terminal::term::cell::Flags as AlacrittyFlags;
use tracing::instrument;

use super::*;

fn terminal_line_text(
    row: &alacritty_terminal::grid::Row<alacritty_terminal::term::cell::Cell>,
    columns: usize,
) -> String {
    let mut line = String::with_capacity(columns);
    for col_idx in 0..columns {
        line.push(row[Column(col_idx)].c);
    }
    line.trim_end().to_string()
}

fn truncate_to_char_boundary(value: &mut String, max_bytes: usize) {
    if value.len() <= max_bytes {
        return;
    }

    let mut end = max_bytes;
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    value.truncate(end);
}

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

    /// Captures bounded plain terminal text from scrollback plus the visible grid.
    #[instrument(level = "trace", skip(self))]
    pub fn text_snapshot(&self, max_lines: usize, max_bytes: usize) -> TerminalTextSnapshot {
        if max_lines == 0 || max_bytes == 0 {
            return TerminalTextSnapshot {
                text: String::new(),
                line_count: 0,
                truncated: true,
            };
        }

        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let grid = state.term.grid();
        let columns = state.term.columns();
        let history_size = grid.history_size();
        let screen_lines = state.term.screen_lines();
        let total_available_lines = history_size + screen_lines;
        let captured_lines = total_available_lines.min(max_lines);
        let end_line = screen_lines as i32 - 1;
        let start_line = end_line - captured_lines as i32 + 1;

        let mut lines = Vec::with_capacity(captured_lines);
        for line_idx in start_line..=end_line {
            let row = &grid[Line(line_idx)];
            lines.push(terminal_line_text(row, columns));
        }

        let leading_empty = lines
            .iter()
            .position(|line| !line.is_empty())
            .unwrap_or(lines.len());
        if leading_empty > 0 {
            lines.drain(0..leading_empty);
        }

        let line_count_before_byte_cap = lines.len();
        let mut text = lines.join("\n").trim_end().to_string();
        let mut truncated = total_available_lines > max_lines;
        if text.len() > max_bytes {
            truncate_to_char_boundary(&mut text, max_bytes);
            truncated = true;
        }

        TerminalTextSnapshot {
            line_count: text.lines().count().min(line_count_before_byte_cap),
            text,
            truncated,
        }
    }
}

#[cfg(test)]
mod text_snapshot_tests {
    use super::*;

    #[test]
    fn truncate_to_char_boundary_preserves_utf8() {
        let mut value = "abc😀def".to_string();
        truncate_to_char_boundary(&mut value, 5);
        assert_eq!(value, "abc");
    }
}
