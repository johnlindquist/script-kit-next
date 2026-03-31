use alacritty_terminal::index::{Column, Direction, Line, Point as AlacPoint};
use alacritty_terminal::selection::{Selection, SelectionType};
use tracing::{debug, trace};

use super::*;

impl TerminalHandle {
    /// Scrolls the terminal display by a number of lines.
    ///
    /// # Arguments
    ///
    /// * `delta` - Number of lines to scroll (positive = up, negative = down)
    pub fn scroll(&mut self, delta: i32) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let scroll = alacritty_terminal::grid::Scroll::Delta(delta);
        state.term.scroll_display(scroll);
        debug!(delta, "Scrolled terminal display");
    }

    /// Scrolls the terminal display by one page up.
    pub fn scroll_page_up(&mut self) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state
            .term
            .scroll_display(alacritty_terminal::grid::Scroll::PageUp);
        debug!("Scrolled terminal page up");
    }

    /// Scrolls the terminal display by one page down.
    pub fn scroll_page_down(&mut self) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state
            .term
            .scroll_display(alacritty_terminal::grid::Scroll::PageDown);
        debug!("Scrolled terminal page down");
    }

    /// Scrolls the terminal display to the top of scrollback.
    pub fn scroll_to_top(&mut self) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state
            .term
            .scroll_display(alacritty_terminal::grid::Scroll::Top);
        debug!("Scrolled terminal to top");
    }

    /// Scrolls the terminal display to the bottom (latest output).
    pub fn scroll_to_bottom(&mut self) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state
            .term
            .scroll_display(alacritty_terminal::grid::Scroll::Bottom);
        debug!("Scrolled terminal to bottom");
    }

    /// Gets the current scroll offset (0 = at bottom).
    pub fn display_offset(&self) -> usize {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.term.grid().display_offset()
    }

    /// Gets the current selection as a string, if any.
    ///
    /// # Returns
    ///
    /// The selected text, or `None` if there is no selection.
    pub fn selection_to_string(&self) -> Option<String> {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.term.selection_to_string()
    }

    /// Clears the current selection.
    pub fn clear_selection(&mut self) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.term.selection = None;
        debug!("Selection cleared");
    }

    /// Start a new selection at the given grid position.
    ///
    /// # Arguments
    ///
    /// * `col` - Column index (0-indexed from left)
    /// * `row` - Row index (0-indexed from top of visible area)
    pub fn start_selection(&mut self, col: usize, row: usize) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let point = AlacPoint::new(Line(row as i32), Column(col));
        state.term.selection = Some(Selection::new(
            SelectionType::Simple,
            point,
            Direction::Left,
        ));
        debug!(col, row, "Selection started");
    }

    /// Start a semantic (word) selection at the given grid position.
    ///
    /// Double-click triggers word selection - selects the word at the clicked position.
    ///
    /// # Arguments
    ///
    /// * `col` - Column index (0-indexed from left)
    /// * `row` - Row index (0-indexed from top of visible area)
    pub fn start_semantic_selection(&mut self, col: usize, row: usize) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let point = AlacPoint::new(Line(row as i32), Column(col));
        state.term.selection = Some(Selection::new(
            SelectionType::Semantic,
            point,
            Direction::Left,
        ));
        debug!(col, row, "Semantic (word) selection started");
    }

    /// Start a line selection at the given grid position.
    ///
    /// Triple-click triggers line selection - selects the entire line at the clicked position.
    ///
    /// # Arguments
    ///
    /// * `col` - Column index (0-indexed from left)
    /// * `row` - Row index (0-indexed from top of visible area)
    pub fn start_line_selection(&mut self, col: usize, row: usize) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let point = AlacPoint::new(Line(row as i32), Column(col));
        state.term.selection = Some(Selection::new(SelectionType::Lines, point, Direction::Left));
        debug!(col, row, "Line selection started");
    }

    /// Update the current selection to extend to the given position.
    ///
    /// # Arguments
    ///
    /// * `col` - Column index (0-indexed from left)
    /// * `row` - Row index (0-indexed from top of visible area)
    pub fn update_selection(&mut self, col: usize, row: usize) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(ref mut selection) = state.term.selection {
            let point = AlacPoint::new(Line(row as i32), Column(col));
            selection.update(point, Direction::Right);
            trace!(col, row, "Selection updated");
        }
    }

    /// Check if there is an active selection.
    pub fn has_selection(&self) -> bool {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.term.selection.is_some()
    }

    /// Send mouse scroll wheel events to the PTY as escape sequences.
    ///
    /// When the terminal is in mouse mode, scroll events must be encoded
    /// as mouse button events and sent to the running application. When on
    /// the alternate screen with `ALTERNATE_SCROLL` enabled (but no mouse
    /// mode), scroll events are converted to arrow key sequences.
    ///
    /// Returns `true` if the scroll was handled by sending to PTY,
    /// `false` if the caller should fall back to display buffer scrolling.
    pub fn scroll_to_pty(&mut self, lines: i32) -> bool {
        use alacritty_terminal::term::TermMode;

        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let mode = *state.term.mode();
        let is_mouse = mode.intersects(TermMode::MOUSE_MODE);
        let is_alt = mode.contains(TermMode::ALT_SCREEN);
        let is_alt_scroll = mode.contains(TermMode::ALTERNATE_SCROLL);
        let is_sgr = mode.contains(TermMode::SGR_MOUSE);
        drop(state);

        if is_mouse {
            // Mouse mode: send SGR or legacy mouse wheel sequences.
            // Button 64 = scroll up, button 65 = scroll down (SGR encoding).
            let (button, count) = if lines > 0 {
                (64u8, lines as u32) // scroll up
            } else {
                (65u8, (-lines) as u32) // scroll down
            };

            for _ in 0..count {
                let seq = if is_sgr {
                    // SGR: \x1b[<button;col;rowM  (col/row 1-based, use 1;1 as position)
                    format!("\x1b[<{button};1;1M")
                } else {
                    // Legacy X10: \x1b[M + (button+32) + (col+33) + (row+33)
                    let cb = (button + 32) as char;
                    let cx = 33u8 as char; // column 1
                    let cy = 33u8 as char; // row 1
                    format!("\x1b[M{cb}{cx}{cy}")
                };
                let _ = self.input(seq.as_bytes());
            }
            debug!(lines, sgr = is_sgr, "Sent mouse wheel to PTY");
            true
        } else if is_alt && is_alt_scroll {
            // Alt screen + alternate scroll: convert to arrow keys.
            let is_app_cursor = {
                let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
                state.term.mode().contains(TermMode::APP_CURSOR)
            };
            let (seq, count) = if lines > 0 {
                // Scroll up → up arrow
                (
                    if is_app_cursor { "\x1bOA" } else { "\x1b[A" },
                    lines as u32,
                )
            } else {
                // Scroll down → down arrow
                (
                    if is_app_cursor { "\x1bOB" } else { "\x1b[B" },
                    (-lines) as u32,
                )
            };
            for _ in 0..count {
                let _ = self.input(seq.as_bytes());
            }
            debug!(
                lines,
                app_cursor = is_app_cursor,
                "Sent alternate scroll arrows to PTY"
            );
            true
        } else {
            // Normal screen, no mouse mode: caller should scroll display buffer.
            false
        }
    }
}
