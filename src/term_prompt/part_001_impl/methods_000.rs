impl TermPrompt {
    /// Create new terminal prompt
    #[allow(dead_code)]
    pub fn new(
        id: String,
        command: Option<String>,
        focus_handle: FocusHandle,
        on_submit: SubmitCallback,
        theme: Arc<Theme>,
        config: Arc<Config>,
    ) -> anyhow::Result<Self> {
        Self::with_height(id, command, focus_handle, on_submit, theme, config, None)
    }

    /// Create new terminal prompt with explicit height
    ///
    /// This is necessary because GPUI entities don't inherit parent flex sizing.
    /// When rendered as a child of a sized container, h_full() doesn't resolve
    /// to the parent's height. We must pass an explicit height.
    pub fn with_height(
        id: String,
        command: Option<String>,
        focus_handle: FocusHandle,
        on_submit: SubmitCallback,
        theme: Arc<Theme>,
        config: Arc<Config>,
        content_height: Option<Pixels>,
    ) -> anyhow::Result<Self> {
        // Start with a reasonable default size; will be resized dynamically
        let initial_cols = 80;
        let initial_rows = 24;

        let terminal = match command {
            Some(cmd) => TerminalHandle::with_command(&cmd, initial_cols, initial_rows)?,
            None => TerminalHandle::new(initial_cols, initial_rows)?,
        };

        info!(
            id = %id,
            content_height = ?content_height,
            "TermPrompt::with_height created"
        );

        Ok(Self {
            id,
            terminal,
            focus_handle,
            on_submit,
            theme,
            config,
            exited: false,
            exit_code: None,
            refresh_timer_active: false,
            last_size: (initial_cols, initial_rows),
            content_height,
            bell_flash_until: None,
            title: None,
            is_selecting: false,
            selection_start: None,
            last_click_time: None,
            last_click_position: None,
            click_count: 0,
            suppress_keys: false,
        })
    }

    /// Set the content height (for dynamic resizing)
    #[allow(dead_code)]
    pub fn set_height(&mut self, height: Pixels) {
        self.content_height = Some(height);
    }

    /// Execute a terminal action.
    ///
    /// This method handles all terminal actions from the command bar,
    /// including clipboard operations, scrolling, and control signals.
    #[allow(dead_code)]
    pub fn execute_action(&mut self, action: TerminalAction, cx: &mut Context<Self>) {
        use arboard::Clipboard;

        match action {
            TerminalAction::Clear => {
                // Send clear screen and home cursor escape sequences
                let _ = self.terminal.input(b"\x1b[2J\x1b[H");
                cx.notify();
            }
            TerminalAction::Copy => {
                // Copy selection to clipboard
                if let Some(text) = self
                    .terminal
                    .selection_to_string()
                    .filter(|t| !t.is_empty())
                {
                    if let Ok(mut clipboard) = Clipboard::new() {
                        if clipboard.set_text(&text).is_ok() {
                            debug!(
                                text_len = text.len(),
                                "Copied selection to clipboard via action"
                            );
                        }
                    }
                    // Clear selection after copy
                    self.terminal.clear_selection();
                    cx.notify();
                }
            }
            TerminalAction::CopyAll => {
                // Copy all visible terminal content
                let terminal_content = self.terminal.content();
                let all_text = terminal_content.lines.join("\n");
                let trimmed = all_text.trim_end().to_string();
                if !trimmed.is_empty() {
                    if let Ok(mut clipboard) = Clipboard::new() {
                        let _ = clipboard.set_text(&trimmed);
                        debug!(text_len = trimmed.len(), "Copied all terminal content");
                    }
                }
            }
            TerminalAction::CopyLastCommand => {
                // Copy the last command entered (basic heuristic)
                let terminal_content = self.terminal.content();
                let prompt_patterns = ["$ ", "% ", "> "];
                for line in terminal_content.lines.iter().rev() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    for pattern in &prompt_patterns {
                        if let Some(pos) = trimmed.rfind(pattern) {
                            let cmd_start = pos + pattern.len();
                            if cmd_start < trimmed.len() {
                                let cmd = trimmed[cmd_start..].trim().to_string();
                                if !cmd.is_empty() {
                                    if let Ok(mut clipboard) = Clipboard::new() {
                                        let _ = clipboard.set_text(&cmd);
                                        debug!(command = %cmd, "Copied last command");
                                    }
                                    return;
                                }
                            }
                        }
                    }
                }
            }
            TerminalAction::CopyLastOutput => {
                // Copy output of the last command (basic heuristic)
                let terminal_content = self.terminal.content();
                let prompt_patterns = ["$ ", "% ", "> "];
                let mut prompt_indices: Vec<usize> = Vec::new();

                for (idx, line) in terminal_content.lines.iter().enumerate() {
                    let trimmed = line.trim();
                    for pattern in &prompt_patterns {
                        if trimmed.contains(pattern) {
                            prompt_indices.push(idx);
                            break;
                        }
                    }
                }

                if prompt_indices.len() >= 2 {
                    let last_prompt = prompt_indices[prompt_indices.len() - 1];
                    let second_last = prompt_indices[prompt_indices.len() - 2];
                    if second_last + 1 < last_prompt {
                        let output: String = terminal_content.lines[second_last + 1..last_prompt]
                            .join("\n")
                            .trim()
                            .to_string();
                        if !output.is_empty() {
                            if let Ok(mut clipboard) = Clipboard::new() {
                                let _ = clipboard.set_text(&output);
                                debug!(text_len = output.len(), "Copied last output");
                            }
                        }
                    }
                }
            }
            TerminalAction::Paste => {
                // Paste from clipboard with bracketed paste support
                if let Ok(mut clipboard) = Clipboard::new() {
                    if let Ok(text) = clipboard.get_text() {
                        let paste_data = if self.terminal.is_bracketed_paste_mode() {
                            debug!(
                                text_len = text.len(),
                                "Pasting with bracketed paste mode via action"
                            );
                            format!("\x1b[200~{}\x1b[201~", text)
                        } else {
                            debug!(text_len = text.len(), "Pasting clipboard text via action");
                            text
                        };
                        if let Err(e) = self.terminal.input(paste_data.as_bytes()) {
                            if !self.exited {
                                warn!(error = %e, "Failed to paste clipboard to terminal");
                            }
                        }
                    }
                }
            }
            TerminalAction::SelectAll => {
                // Select all visible content
                let (cols, rows) = self.last_size;
                self.terminal.start_line_selection(0, 0);
                self.terminal.update_selection(cols as usize, rows as usize);
                cx.notify();
            }
            TerminalAction::ScrollToTop => {
                self.terminal.scroll_to_top();
                cx.notify();
            }
            TerminalAction::ScrollToBottom => {
                self.terminal.scroll_to_bottom();
                cx.notify();
            }
            TerminalAction::ScrollPageUp => {
                self.terminal.scroll_page_up();
                cx.notify();
            }
            TerminalAction::ScrollPageDown => {
                self.terminal.scroll_page_down();
                cx.notify();
            }
            TerminalAction::Interrupt => {
                // Send SIGINT (Ctrl+C)
                let _ = self.terminal.input(&[0x03]);
            }
            TerminalAction::Kill => {
                // Send SIGTERM - for now just send SIGINT twice as a stronger signal
                let _ = self.terminal.input(&[0x03]);
                let _ = self.terminal.input(&[0x03]);
            }
            TerminalAction::Suspend => {
                // Send SIGTSTP (Ctrl+Z)
                let _ = self.terminal.input(&[0x1A]);
            }
            TerminalAction::Quit => {
                // Send SIGQUIT (Ctrl+\)
                let _ = self.terminal.input(&[0x1C]);
            }
            TerminalAction::SendEOF => {
                // Send EOF (Ctrl+D)
                let _ = self.terminal.input(&[0x04]);
            }
            TerminalAction::Reset => {
                // Reset terminal (RIS - Reset to Initial State)
                let _ = self.terminal.input(b"\x1bc");
                cx.notify();
            }
            TerminalAction::Restart => {
                // Reset terminal as a basic restart
                let _ = self.terminal.input(b"\x1bc");
                cx.notify();
            }
            // Search and other features - placeholder
            TerminalAction::Find
            | TerminalAction::NewShell
            | TerminalAction::Custom(_)
            | TerminalAction::ZoomIn
            | TerminalAction::ZoomOut
            | TerminalAction::ResetZoom => {
                debug!(action = ?action, "Terminal action not yet implemented");
            }
        }
    }

}
