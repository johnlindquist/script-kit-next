impl Focusable for TermPrompt {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
impl Render for TermPrompt {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let start = Instant::now();

        // Start refresh timer if not already active
        self.start_refresh_timer(cx);

        // Get window bounds and resize terminal if needed
        // Use content_height if set (for constrained layouts), otherwise use window height
        let window_bounds = window.bounds();
        let effective_height = self.content_height.unwrap_or(window_bounds.size.height);
        self.resize_if_needed(window_bounds.size.width, effective_height);

        // NOTE: Terminal event processing is centralized in the refresh timer.
        // We do NOT call terminal.process() here to avoid:
        // 1. Processing the same data twice (timer already handles it)
        // 2. State changes during selection (causes selection bugs)
        // 3. Wasted CPU cycles
        //
        // The timer runs at 30fps and calls process() with event handling.
        // Render just reads the current terminal state.

        // Get terminal content
        let content = self.terminal.content();

        // Handle keyboard with Ctrl+key support
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  _window: &mut Window,
                  _cx: &mut Context<Self>| {
                // Hide cursor while typing - automatically shows when mouse moves
                crate::platform::hide_cursor_until_mouse_moves();

                // When actions panel is open, ignore all key events
                if this.suppress_keys {
                    return;
                }

                let key_str = event.keystroke.key.to_lowercase();
                let has_ctrl = event.keystroke.modifiers.control;
                let has_meta = event.keystroke.modifiers.platform;
                let has_shift = event.keystroke.modifiers.shift;

                // Escape always cancels
                if key_str == "escape" {
                    this.submit_cancel();
                    return;
                }

                // Handle Shift+PageUp/PageDown/Home/End for scrollback navigation
                // These work even after terminal exits to review output
                if has_shift {
                    match key_str.as_str() {
                        "pageup" => {
                            this.terminal.scroll_page_up();
                            debug!("Shift+PageUp: scrolling terminal page up");
                            return;
                        }
                        "pagedown" => {
                            this.terminal.scroll_page_down();
                            debug!("Shift+PageDown: scrolling terminal page down");
                            return;
                        }
                        "home" => {
                            this.terminal.scroll_to_top();
                            debug!("Shift+Home: scrolling terminal to top");
                            return;
                        }
                        "end" => {
                            this.terminal.scroll_to_bottom();
                            debug!("Shift+End: scrolling terminal to bottom");
                            return;
                        }
                        _ => {}
                    }
                }

                // Handle Cmd+C copy BEFORE the "terminal running" check
                // Copy should work even after terminal exits (for reviewing scrollback)
                // MUST always return to prevent inserting 'c' character
                if has_meta && key_str == "c" {
                    // Try to copy selection if one exists
                    if let Some(selected_text) = this
                        .terminal
                        .selection_to_string()
                        .filter(|t| !t.is_empty())
                    {
                        use arboard::Clipboard;
                        if let Ok(mut clipboard) = Clipboard::new() {
                            if clipboard.set_text(&selected_text).is_ok() {
                                debug!(
                                    text_len = selected_text.len(),
                                    "Copied selection to clipboard"
                                );
                            }
                        }
                        // Clear selection after copy (common terminal behavior)
                        this.terminal.clear_selection();
                        return;
                    }

                    // No selection - send SIGINT (Ctrl+C) if terminal is still running
                    if this.terminal.is_running() && !this.exited {
                        debug!("Cmd+C with no selection - sending SIGINT");
                        let _ = this.terminal.input(&[0x03]); // ETX / SIGINT
                    }
                    // Always return to prevent inserting 'c' character
                    return;
                }

                // Check if terminal is still running before sending other input
                if this.exited || !this.terminal.is_running() {
                    trace!(key = %key_str, "Terminal exited, ignoring key input");
                    return;
                }

                // Handle Cmd+V paste (macOS: platform modifier = Command key)
                if has_meta && key_str == "v" {
                    use arboard::Clipboard;
                    if let Ok(mut clipboard) = Clipboard::new() {
                        if let Ok(text) = clipboard.get_text() {
                            // Check if bracketed paste mode is enabled
                            // When enabled, wrap paste in escape sequences so apps know it's pasted
                            let paste_data = if this.terminal.is_bracketed_paste_mode() {
                                debug!(text_len = text.len(), "Pasting with bracketed paste mode");
                                format!("\x1b[200~{}\x1b[201~", text)
                            } else {
                                debug!(text_len = text.len(), "Pasting clipboard text to terminal");
                                text
                            };

                            if let Err(e) = this.terminal.input(paste_data.as_bytes()) {
                                if !this.exited {
                                    warn!(error = %e, "Failed to paste clipboard to terminal");
                                }
                            }
                        }
                    }
                    return;
                }

                // Handle Ctrl+key combinations first
                if has_ctrl {
                    if let Some(ctrl_byte) = Self::ctrl_key_to_byte(&key_str) {
                        debug!(key = %key_str, byte = ctrl_byte, "Sending Ctrl+key");
                        if let Err(e) = this.terminal.input(&[ctrl_byte]) {
                            // Only warn if unexpected error
                            if !this.exited {
                                warn!(error = %e, "Failed to send Ctrl+key to terminal");
                            }
                        }
                        // No cx.notify() needed - timer handles refresh at 30fps
                        return;
                    }
                }

                // Forward regular input to terminal
                if let Some(key_char) = &event.keystroke.key_char {
                    if let Err(e) = this.terminal.input(key_char.as_bytes()) {
                        if !this.exited {
                            warn!(error = %e, "Failed to send input to terminal");
                        }
                    }
                    // No cx.notify() needed - timer handles refresh at 30fps
                } else {
                    // Handle special keys
                    // Check if terminal is in application cursor mode (DECCKM)
                    // Many apps (vim, less, htop, fzf) enable this mode for arrow keys
                    let app_cursor = this.terminal.is_application_cursor_mode();

                    // Arrow keys and Home/End have different sequences in application mode:
                    // Normal mode: \x1b[A (CSI A)
                    // Application mode: \x1bOA (SS3 A)
                    let bytes: Option<&[u8]> = match key_str.as_str() {
                        "enter" => Some(b"\r"),
                        "backspace" => Some(b"\x7f"),
                        "tab" => Some(b"\t"),
                        // Arrow keys: use application mode sequences when DECCKM is set
                        "up" | "arrowup" => Some(if app_cursor { b"\x1bOA" } else { b"\x1b[A" }),
                        "down" | "arrowdown" => {
                            Some(if app_cursor { b"\x1bOB" } else { b"\x1b[B" })
                        }
                        "right" | "arrowright" => {
                            Some(if app_cursor { b"\x1bOC" } else { b"\x1b[C" })
                        }
                        "left" | "arrowleft" => {
                            Some(if app_cursor { b"\x1bOD" } else { b"\x1b[D" })
                        }
                        // Home/End also have application mode variants
                        "home" => Some(if app_cursor { b"\x1bOH" } else { b"\x1b[H" }),
                        "end" => Some(if app_cursor { b"\x1bOF" } else { b"\x1b[F" }),
                        "pageup" => Some(b"\x1b[5~"),
                        "pagedown" => Some(b"\x1b[6~"),
                        "delete" => Some(b"\x1b[3~"),
                        "insert" => Some(b"\x1b[2~"),
                        "f1" => Some(b"\x1bOP"),
                        "f2" => Some(b"\x1bOQ"),
                        "f3" => Some(b"\x1bOR"),
                        "f4" => Some(b"\x1bOS"),
                        "f5" => Some(b"\x1b[15~"),
                        "f6" => Some(b"\x1b[17~"),
                        "f7" => Some(b"\x1b[18~"),
                        "f8" => Some(b"\x1b[19~"),
                        "f9" => Some(b"\x1b[20~"),
                        "f10" => Some(b"\x1b[21~"),
                        "f11" => Some(b"\x1b[23~"),
                        "f12" => Some(b"\x1b[24~"),
                        _ => None,
                    };

                    if let Some(bytes) = bytes {
                        if let Err(e) = this.terminal.input(bytes) {
                            if !this.exited {
                                warn!(error = %e, "Failed to send special key to terminal");
                            }
                        }
                        // No cx.notify() needed - timer handles refresh at 30fps
                    }
                }
            },
        );

        // Render terminal content with styled cells
        let colors = &self.theme.colors;
        let terminal_content = self.render_content(&content);

        // Get padding from config
        let padding = self.config.get_padding();

        // Check if bell is flashing and clear expired state
        // This ensures bell flash doesn't stick if timer has stopped (e.g., after terminal exit)
        let is_bell_flashing = match self.bell_flash_until {
            Some(until) if Instant::now() < until => true,
            Some(_) => {
                // Flash expired - clear the state
                self.bell_flash_until = None;
                false
            }
            None => false,
        };

        // Log slow renders
        let elapsed = start.elapsed().as_millis();
        if elapsed > SLOW_RENDER_THRESHOLD_MS {
            warn!(elapsed_ms = elapsed, "Slow terminal render");
        } else {
            debug!(elapsed_ms = elapsed, "Terminal render");
        }

        // Main container with terminal styling
        // Use explicit height if available, otherwise fall back to size_full
        // Apply padding from config settings (top/left/right/bottom)
        // No background - let vibrancy show through from parent (render_prompts/term.rs handles bg)
        let container = div()
            .flex()
            .flex_col()
            .w_full()
            .pl(px(padding.left))
            .pr(px(padding.right))
            .pt(px(padding.top))
            .pb(px(padding.top)) // Use same as top for consistent spacing
            // No .bg() - vibrancy support
            .text_color(rgb(colors.text.primary))
            .overflow_hidden() // Clip any overflow
            .key_context("term_prompt")
            .track_focus(&self.focus_handle)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, _window, _cx| {
                    let (col, row) = this.pixel_to_cell(event.position);
                    // Clamp to viewport to prevent out-of-bounds access
                    let (col, row) = this.clamp_to_viewport(col, row);
                    let now = Instant::now();
                    let multi_click_threshold = Duration::from_millis(500);

                    // Check if this is a multi-click (same position, within time window)
                    let is_same_position = this.last_click_position == Some((col, row));
                    let is_quick_click = this
                        .last_click_time
                        .map(|t| now.duration_since(t) < multi_click_threshold)
                        .unwrap_or(false);

                    if is_same_position && is_quick_click {
                        this.click_count = (this.click_count + 1).min(3);
                    } else {
                        this.click_count = 1;
                    }

                    this.last_click_time = Some(now);
                    this.last_click_position = Some((col, row));
                    this.is_selecting = true;
                    this.selection_start = Some((col, row));

                    // Start selection based on click count
                    match this.click_count {
                        1 => {
                            // Simple click-drag selection
                            debug!(col, row, "Mouse down at cell - starting simple selection");
                            this.terminal.start_selection(col, row);
                        }
                        2 => {
                            // Double-click: word selection
                            debug!(col, row, "Double-click at cell - starting word selection");
                            this.terminal.start_semantic_selection(col, row);
                        }
                        3 => {
                            // Triple-click: line selection
                            debug!(col, row, "Triple-click at cell - starting line selection");
                            this.terminal.start_line_selection(col, row);
                        }
                        _ => {}
                    }
                }),
            )
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _window, _cx| {
                if this.is_selecting {
                    let (col, row) = this.pixel_to_cell(event.position);
                    // Clamp to viewport to prevent out-of-bounds access
                    let (col, row) = this.clamp_to_viewport(col, row);
                    this.terminal.update_selection(col, row);
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, event: &MouseUpEvent, _window, _cx| {
                    if this.is_selecting {
                        let (col, row) = this.pixel_to_cell(event.position);
                        // Clamp to viewport to prevent out-of-bounds access
                        let (col, row) = this.clamp_to_viewport(col, row);
                        debug!(col, row, "Mouse up at cell - finalizing selection");
                        this.terminal.update_selection(col, row);
                        this.is_selecting = false;

                        // Clear selection if single-click without drag (clicked and released at same position)
                        // For double/triple click, we keep the word/line selection
                        if this.click_count == 1 {
                            if let Some((start_col, start_row)) = this.selection_start {
                                if start_col == col && start_row == row {
                                    // Single click at same position = clear any previous selection
                                    debug!(
                                        col,
                                        row, "Single click without drag - clearing selection"
                                    );
                                    this.terminal.clear_selection();
                                    return;
                                }
                            }
                        }

                        // Log the selected text if any
                        if let Some(text) = this.terminal.selection_to_string() {
                            let preview = if text.len() > 50 {
                                format!("{}...", truncate_str(&text, 50))
                            } else {
                                text.clone()
                            };
                            debug!(text_len = text.len(), "Selection complete: {:?}", preview);
                        }
                    }
                }),
            )
            .on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, _window, cx| {
                // Get scroll direction from delta
                // Lines: direct line count, Pixels: convert based on cell height
                let lines = match event.delta {
                    ScrollDelta::Lines(point) => point.y,
                    ScrollDelta::Pixels(point) => {
                        // Convert pixels to lines by dividing by cell height
                        // Pixels implements Div<Pixels> -> f32
                        let cell_height = px(this.cell_height());
                        point.y / cell_height
                    }
                };

                // Convert to integer lines (positive = scroll down, negative = scroll up)
                // In terminal scrollback: negative delta scrolls up into history
                // We invert because terminal scroll() uses positive = scroll up (into history)
                let scroll_lines = -lines.round() as i32;

                if scroll_lines != 0 {
                    this.terminal.scroll(scroll_lines);
                    trace!(delta = scroll_lines, "Mouse wheel scroll");
                    cx.notify();
                }
            }))
            .on_key_down(handle_key);

        // Apply bell flash border if active
        let container = if is_bell_flashing {
            container
                .border_2()
                .border_color(rgb(colors.accent.selected))
        } else {
            container
        };

        // Apply height - use explicit if set, otherwise use h_full (may not work in all contexts)
        let container = if let Some(h) = self.content_height {
            debug!(content_height = ?h, "TermPrompt using explicit height");
            container.h(h)
        } else {
            container.h_full().min_h(px(0.))
        };

        // Check if scrolled up from bottom - if so, show indicator
        let scroll_offset = self.terminal.display_offset();

        if scroll_offset > 0 {
            // Create scroll position indicator overlay
            let indicator = div()
                .absolute()
                .bottom_2()
                .right_2()
                .px_2()
                .py_1()
                .bg(rgb(colors.background.title_bar))
                .text_color(rgb(colors.text.secondary))
                .text_xs()
                .rounded_sm()
                .child(format!("↑{}", scroll_offset));

            // Wrap container in a relative positioned div to enable absolute positioning
            div()
                .relative()
                .w_full()
                .h_full()
                .child(container.child(terminal_content))
                .child(indicator)
        } else {
            container.child(terminal_content)
        }
    }
}
