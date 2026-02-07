use alacritty_terminal::term::TermMode;
use anyhow::{Context, Result};
use tracing::{debug, instrument, trace};

use crate::theme::Theme;

use super::*;

impl TerminalHandle {
    /// Processes PTY output through terminal parser.
    ///
    /// Reads available data from the channel (sent by background reader thread)
    /// and processes it through the VTE parser to update the terminal grid.
    /// Returns a tuple of (had_output, events) where had_output is true if any
    /// data was processed (grid may have changed), and events are terminal events
    /// like Bell, Title changes, or Exit.
    ///
    /// This method is non-blocking - it only processes data that's already
    /// been read by the background thread.
    ///
    /// # Returns
    ///
    /// A tuple of (had_output: bool, events: Vec<TerminalEvent>)
    #[instrument(level = "trace", skip(self))]
    pub fn process(&mut self) -> (bool, Vec<TerminalEvent>) {
        let mut had_output = false;

        while let Ok(data) = self.pty_output_rx.try_recv() {
            trace!(bytes = data.len(), "Processing PTY data from channel");
            let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            state.process_bytes(&data);
            had_output = true;
        }

        let events = self.event_proxy.take_events();
        (had_output, events)
    }

    /// Sends keyboard input bytes to the terminal.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Raw bytes to send (e.g., UTF-8 encoded text)
    ///
    /// # Errors
    ///
    /// Returns an error if writing to the PTY fails.
    #[instrument(level = "debug", skip(self, bytes), fields(bytes_len = bytes.len()))]
    pub fn input(&mut self, bytes: &[u8]) -> Result<()> {
        self.pty
            .write_all(bytes)
            .context("Failed to write to PTY")?;
        self.pty.flush().context("Failed to flush PTY")?;
        debug!(bytes_len = bytes.len(), "Sent input to terminal");
        Ok(())
    }

    /// Resizes the terminal grid.
    ///
    /// Content is reflowed according to terminal resize semantics:
    /// - Lines longer than the new width are wrapped
    /// - The cursor position is adjusted to stay visible
    /// - Scrollback content is preserved
    ///
    /// # Arguments
    ///
    /// * `cols` - New number of columns
    /// * `rows` - New number of rows
    ///
    /// # Errors
    ///
    /// Returns an error if the PTY resize fails.
    #[instrument(level = "debug", skip(self), fields(cols, rows))]
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        self.pty
            .resize(cols, rows)
            .context("Failed to resize PTY")?;

        let size = TerminalSize::new(cols, rows);
        {
            let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            state.term.resize(size);
        }

        self.cols = cols;
        self.rows = rows;

        debug!(cols, rows, "Terminal resized");
        Ok(())
    }

    /// Returns the current terminal dimensions as (columns, rows).
    #[inline]
    pub fn size(&self) -> (u16, u16) {
        (self.cols, self.rows)
    }

    /// Checks if the terminal process is still running.
    ///
    /// # Returns
    ///
    /// `true` if the child process is still running, `false` otherwise.
    pub fn is_running(&mut self) -> bool {
        self.pty.is_running()
    }

    /// Gets the configured scrollback buffer size.
    #[inline]
    pub fn scrollback_lines(&self) -> usize {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.term.history_size()
    }

    /// Check if bracketed paste mode is enabled.
    ///
    /// When bracketed paste mode is enabled, pasted text should be wrapped
    /// in escape sequences (`\x1b[200~` before and `\x1b[201~` after) so
    /// the shell/application knows the content is pasted rather than typed.
    ///
    /// # Returns
    ///
    /// `true` if the terminal is in bracketed paste mode.
    pub fn is_bracketed_paste_mode(&self) -> bool {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.term.mode().contains(TermMode::BRACKETED_PASTE)
    }

    /// Check if application cursor mode (DECCKM) is enabled.
    ///
    /// When application cursor mode is enabled, arrow keys send different
    /// escape sequences:
    /// - Normal mode: `\x1b[A` (up), `\x1b[B` (down), `\x1b[C` (right), `\x1b[D` (left)
    /// - Application mode: `\x1bOA` (up), `\x1bOB` (down), `\x1bOC` (right), `\x1bOD` (left)
    ///
    /// Many terminal applications (vim, less, htop, fzf) enable this mode
    /// to properly receive arrow key input.
    ///
    /// # Returns
    ///
    /// `true` if the terminal is in application cursor mode.
    pub fn is_application_cursor_mode(&self) -> bool {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.term.mode().contains(TermMode::APP_CURSOR)
    }

    /// Updates the theme adapter for focus state.
    ///
    /// # Arguments
    ///
    /// * `is_focused` - Whether the terminal window is focused.
    pub fn update_focus(&mut self, is_focused: bool) {
        self.theme.update_for_focus(is_focused);
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.term.is_focused = is_focused;
        debug!(is_focused, "Terminal focus updated");
    }

    /// Gets a reference to the theme adapter.
    pub fn theme(&self) -> &ThemeAdapter {
        &self.theme
    }

    /// Updates the terminal theme from a new Theme.
    ///
    /// This allows updating terminal colors when the application theme changes.
    /// Preserves the current focus state.
    pub fn update_theme(&mut self, theme: &Theme) {
        self.theme.update_from_theme(theme);
        debug!("Terminal theme updated");
    }
}
