impl TermPrompt {
    /// Get the configured font size
    ///
    /// KEEP as px() because:
    /// 1. User explicitly configured a pixel size in config.ts (terminalFontSize)
    /// 2. Terminal requires precise character sizing for monospace grid alignment
    /// 3. Cell dimensions (width/height) are calculated from this value
    fn font_size(&self) -> f32 {
        self.config.get_terminal_font_size()
    }

    /// Get cell width scaled to configured font size
    fn cell_width(&self) -> f32 {
        BASE_CELL_WIDTH * (self.font_size() / BASE_FONT_SIZE)
    }

    /// Get cell height scaled to configured font size
    fn cell_height(&self) -> f32 {
        self.font_size() * LINE_HEIGHT_MULTIPLIER
    }

    /// Convert pixel position to terminal grid cell (col, row)
    fn pixel_to_cell(&self, position: gpui::Point<Pixels>) -> (usize, usize) {
        let padding = self.config.get_padding();
        let pos_x: f32 = position.x.into();
        let pos_y: f32 = position.y.into();
        let x = (pos_x - padding.left).max(0.0);
        let y = (pos_y - padding.top).max(0.0);

        let col = (x / self.cell_width()) as usize;
        let row = (y / self.cell_height()) as usize;

        (col, row)
    }

    /// Clamp cell coordinates to the visible viewport to prevent out-of-bounds access.
    ///
    /// Mouse clicks can produce coordinates beyond the terminal grid (click far right,
    /// far bottom, or during resize races). This function ensures coordinates are always
    /// within valid bounds before passing to selection APIs.
    fn clamp_to_viewport(&self, col: usize, row: usize) -> (usize, usize) {
        let (cols, rows) = self.last_size;
        // Clamp to last column/row (0-indexed, so max is size - 1)
        let max_col = cols.saturating_sub(1) as usize;
        let max_row = rows.saturating_sub(1) as usize;
        (col.min(max_col), row.min(max_row))
    }

    /// Calculate terminal dimensions from pixel size with padding (uses default cell dimensions)
    /// This version uses the base font size dimensions, suitable for tests and static calculations.
    #[cfg(test)]
    fn calculate_terminal_size(
        width: Pixels,
        height: Pixels,
        padding_left: f32,
        padding_right: f32,
        padding_top: f32,
        padding_bottom: f32,
    ) -> (u16, u16) {
        Self::calculate_terminal_size_with_cells(
            width,
            height,
            padding_left,
            padding_right,
            padding_top,
            padding_bottom,
            CELL_WIDTH,
            CELL_HEIGHT,
        )
    }

    /// Calculate terminal dimensions from pixel size with padding and custom cell dimensions
    #[allow(clippy::too_many_arguments)]
    fn calculate_terminal_size_with_cells(
        width: Pixels,
        height: Pixels,
        padding_left: f32,
        padding_right: f32,
        padding_top: f32,
        padding_bottom: f32,
        cell_width: f32,
        cell_height: f32,
    ) -> (u16, u16) {
        // Subtract padding from available space
        let available_width = f32::from(width) - padding_left - padding_right;
        let available_height = f32::from(height) - padding_top - padding_bottom;

        // Calculate columns and rows
        // Use floor() for cols to ensure we never tell the PTY we have more columns
        // than can actually be rendered. Combined with a conservative cell_width,
        // this prevents the last character from wrapping.
        let cols = (available_width / cell_width).floor() as u16;
        let rows = (available_height / cell_height).floor() as u16;

        // Apply minimum bounds
        let cols = cols.max(MIN_COLS);
        let rows = rows.max(MIN_ROWS);

        (cols, rows)
    }

    /// Resize terminal if needed based on new dimensions
    fn resize_if_needed(&mut self, width: Pixels, height: Pixels) {
        let padding = self.config.get_padding();
        let cell_width = self.cell_width();
        let cell_height = self.cell_height();
        // Note: We use padding.top for bottom padding as well (see render() which uses pb(px(padding.top)))
        let (new_cols, new_rows) = Self::calculate_terminal_size_with_cells(
            width,
            height,
            padding.left,
            padding.right,
            padding.top,
            padding.top,
            cell_width,
            cell_height,
        );

        if (new_cols, new_rows) != self.last_size {
            debug!(
                old_cols = self.last_size.0,
                old_rows = self.last_size.1,
                new_cols,
                new_rows,
                "Resizing terminal"
            );

            if let Err(e) = self.terminal.resize(new_cols, new_rows) {
                warn!(error = %e, "Failed to resize terminal");
            } else {
                self.last_size = (new_cols, new_rows);
            }
        }
    }

    /// Handle terminal exit
    fn handle_exit(&mut self, code: i32) {
        info!(code, "Terminal exited");
        self.exited = true;
        self.exit_code = Some(code);
        // Call submit callback with exit code
        (self.on_submit)(self.id.clone(), Some(code.to_string()));
    }

    /// Submit/cancel
    fn submit_cancel(&mut self) {
        (self.on_submit)(self.id.clone(), None);
    }

    /// Start the refresh timer for periodic terminal output updates
    fn start_refresh_timer(&mut self, cx: &mut Context<Self>) {
        if self.refresh_timer_active || self.exited {
            return;
        }
        self.refresh_timer_active = true;

        cx.spawn(async move |this, cx| {
            loop {
                Timer::after(Duration::from_millis(REFRESH_INTERVAL_MS)).await;

                let should_stop = cx
                    .update(|cx| {
                        this.update(cx, |term_prompt, cx| {
                            if term_prompt.exited {
                                term_prompt.refresh_timer_active = false;
                                return true; // Stop polling
                            }

                            // Process terminal output - 2 iterations catches bursts without excessive overhead
                            // Auto-scroll: Track if we're at the bottom before processing
                            let was_at_bottom = term_prompt.terminal.display_offset() == 0;
                            let mut had_output = false;
                            let mut needs_render = false;

                            for _ in 0..2 {
                                let (processed_data, events) = term_prompt.terminal.process();
                                // CRITICAL: processed_data means the grid changed (characters added)
                                // This is separate from events (Bell, Title, Exit)
                                if processed_data {
                                    had_output = true;
                                    needs_render = true;
                                }
                                for event in events {
                                    match event {
                                        TerminalEvent::Exit(code) => {
                                            term_prompt.handle_exit(code);
                                            return true;
                                        }
                                        TerminalEvent::Bell => {
                                            term_prompt.bell_flash_until = Some(
                                                Instant::now()
                                                    + Duration::from_millis(BELL_FLASH_DURATION_MS),
                                            );
                                            debug!("Terminal bell triggered (timer), flashing border");
                                            needs_render = true;
                                        }
                                        TerminalEvent::Title(title) => {
                                            term_prompt.title =
                                                if title.is_empty() { None } else { Some(title) };
                                            debug!(title = ?term_prompt.title, "Terminal title updated (timer)");
                                            needs_render = true;
                                        }
                                        TerminalEvent::Output(_) => { /* handled by had_output */ }
                                    }
                                }
                            }

                            // CRITICAL: Check if the process has exited but we didn't get an Exit event.
                            // This can happen when:
                            // 1. The shell exits via EOF (Ctrl+D) without explicit exit code
                            // 2. The process is killed externally
                            // 3. The PTY reader thread detected EOF but alacritty didn't emit an Exit event
                            // Without this check, the timer loop would run forever at 60fps causing 100% CPU!
                            if !term_prompt.terminal.is_running() && !term_prompt.exited {
                                // Process exited without explicit Exit event
                                // Use exit code 0 as we don't know the actual code
                                info!("Terminal process exited (detected via is_running check)");
                                term_prompt.handle_exit(0);
                                return true; // Stop polling
                            }

                            // Auto-scroll: If we were at bottom and got new output, stay at bottom
                            if was_at_bottom && had_output {
                                term_prompt.terminal.scroll_to_bottom();
                            }

                            // Check if bell flash period ended - need to clear the border
                            if let Some(until) = term_prompt.bell_flash_until {
                                if Instant::now() >= until {
                                    term_prompt.bell_flash_until = None;
                                    needs_render = true;
                                }
                            }

                            // Only trigger re-render if something actually changed
                            if needs_render {
                                cx.notify();
                            }
                            false
                        })
                        .unwrap_or(true)
                    })
                    .unwrap_or(true);

                if should_stop {
                    break;
                }
            }
        })
        .detach();
    }

    /// Convert a Ctrl+key press to the corresponding control character byte.
    ///
    /// Uses the canonical ASCII control character transform: `byte & 0x1F`.
    /// This works for A-Z (gives 0x01-0x1A) and special chars `[ \ ] ^ _`.
    ///
    /// Control character mapping:
    /// - Ctrl+A = 0x01, Ctrl+B = 0x02, ..., Ctrl+Z = 0x1A
    /// - Ctrl+C = 0x03 (SIGINT), Ctrl+D = 0x04 (EOF), Ctrl+Z = 0x1A (SIGTSTP)
    /// - Ctrl+[ = 0x1B (ESC), Ctrl+\ = 0x1C (SIGQUIT)
    ///
    /// Returns None if the key is not a valid control character.
    fn ctrl_key_to_byte(key: &str) -> Option<u8> {
        // Must be a single ASCII character
        if key.len() != 1 {
            return None;
        }

        let byte = key.as_bytes()[0].to_ascii_uppercase();

        // Valid control chars: @ through _ (0x40-0x5F)
        // This covers A-Z (0x41-0x5A) and [ \ ] ^ _ (0x5B-0x5F)
        // We exclude @ (0x40) which would give 0x00 (NUL)
        match byte {
            b'A'..=b'_' => Some(byte & 0x1F),
            _ => None,
        }
    }

}
