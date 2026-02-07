use super::TerminalCell;

/// Content snapshot for rendering.
///
/// This struct contains a snapshot of the terminal content at a point
/// in time, suitable for rendering in GPUI.
#[derive(Debug, Clone)]
pub struct TerminalContent {
    /// Lines of text in the terminal (plain text, backward compatible).
    pub lines: Vec<String>,
    /// Styled lines with per-cell color and attribute information.
    pub styled_lines: Vec<Vec<TerminalCell>>,
    /// Cursor line position (0-indexed from top).
    pub cursor_line: usize,
    /// Cursor column position (0-indexed from left).
    pub cursor_col: usize,
    /// Selected cells as (column, line) pairs for highlighting.
    /// Empty if no selection is active.
    pub selected_cells: Vec<(usize, usize)>,
}

impl TerminalContent {
    /// Returns `true` if the terminal is empty (no content).
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty() || self.lines.iter().all(|l| l.is_empty())
    }

    /// Returns the number of non-empty lines.
    pub fn line_count(&self) -> usize {
        self.lines.iter().filter(|l| !l.is_empty()).count()
    }

    /// Returns plain text lines (backward compatible accessor).
    ///
    /// This method provides backward compatibility for code that only
    /// needs the plain text content without styling information.
    pub fn lines_plain(&self) -> &[String] {
        &self.lines
    }
}

/// Cursor position in the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorPosition {
    /// Line index (0-indexed from top).
    pub line: usize,
    /// Column index (0-indexed from left).
    pub col: usize,
}

impl From<&TerminalContent> for CursorPosition {
    fn from(content: &TerminalContent) -> Self {
        Self {
            line: content.cursor_line,
            col: content.cursor_col,
        }
    }
}
