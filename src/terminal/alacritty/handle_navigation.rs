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
}
