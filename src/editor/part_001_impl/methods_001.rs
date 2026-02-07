impl EditorPrompt {
    /// Set the content and position cursor at end (below last content line)
    ///
    /// If content exists and doesn't end with a newline, appends one so the cursor
    /// starts on a fresh line below the existing content.
    #[allow(dead_code)]
    pub fn set_content(&mut self, content: String, window: &mut Window, cx: &mut Context<Self>) {
        // Ensure content ends with newline so cursor is on line below content
        let content_with_newline = if !content.is_empty() && !content.ends_with('\n') {
            format!("{}\n", content)
        } else {
            content
        };

        if let Some(ref editor_state) = self.editor_state {
            let content_len = content_with_newline.len();
            editor_state.update(cx, |state, cx| {
                state.set_value(content_with_newline, window, cx);
                // Move cursor to end (set selection to end..end = no selection, cursor at end)
                state.set_selection(content_len, content_len, window, cx);
            });
        } else {
            // Update pending content if not yet initialized
            if let Some(ref mut pending) = self.pending_init {
                pending.content = content_with_newline;
            }
        }
    }

    /// Set the language for syntax highlighting
    #[allow(dead_code)]
    pub fn set_language(&mut self, language: String, cx: &mut Context<Self>) {
        self.language = language.clone();
        if let Some(ref editor_state) = self.editor_state {
            editor_state.update(cx, |state, cx| {
                state.set_highlighter(language, cx);
            });
        } else {
            // Update pending language if not yet initialized
            if let Some(ref mut pending) = self.pending_init {
                pending.language = language;
            }
        }
    }

    /// Set the content height (for dynamic resizing)
    #[allow(dead_code)]
    pub fn set_height(&mut self, height: gpui::Pixels) {
        self.content_height = Some(height);
    }

    // -------------------------------------------------------------------------
    // Snippet/Template Navigation
    // -------------------------------------------------------------------------

    /// Check if we're currently in snippet/template navigation mode
    pub fn in_snippet_mode(&self) -> bool {
        self.snippet_state.is_some()
    }

    /// Get the current tabstop index (0-based index into tabstops array)
    #[allow(dead_code)]
    pub fn current_tabstop_index(&self) -> Option<usize> {
        self.snippet_state.as_ref().map(|s| s.current_tabstop_idx)
    }

    /// Get a reference to the snippet state (for footer display)
    pub fn snippet_state(&self) -> Option<&SnippetState> {
        self.snippet_state.as_ref()
    }

    /// Move to the next tabstop (public wrapper for testing via stdin commands)
    pub fn next_tabstop_public(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        self.next_tabstop(window, cx)
    }

    /// Move to the next tabstop. Returns true if we moved, false if we exited snippet mode.
    fn next_tabstop(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        logging::log("EDITOR", "next_tabstop called");

        // Guard: don't mutate snippet state until editor is ready
        // This prevents advancing tabstop index before we can actually select the text
        if self.editor_state.is_none() {
            logging::log("EDITOR", "next_tabstop: editor not initialized yet");
            return false;
        }

        // First, capture what the user typed at the current tabstop
        self.capture_current_tabstop_value(cx);

        let Some(ref mut state) = self.snippet_state else {
            logging::log("EDITOR", "next_tabstop: no snippet_state!");
            return false;
        };
        logging::log(
            "EDITOR",
            &format!(
                "next_tabstop: current_idx={}, total_tabstops={}",
                state.current_tabstop_idx,
                state.snippet.tabstops.len()
            ),
        );

        let tabstop_count = state.snippet.tabstops.len();
        if tabstop_count == 0 {
            self.exit_snippet_mode(window, cx);
            return false;
        }

        // Move to next tabstop
        let next_idx = state.current_tabstop_idx + 1;

        if next_idx >= tabstop_count {
            // We've gone past the last tabstop - check if there's a $0 final cursor
            let last_tabstop = &state.snippet.tabstops[tabstop_count - 1];
            if last_tabstop.index == 0 {
                // We were on the $0 tabstop, exit snippet mode
                logging::log("EDITOR", "Snippet: exiting after $0");
                self.exit_snippet_mode(window, cx);
                return false;
            } else {
                // No $0 tabstop - exit snippet mode
                logging::log("EDITOR", "Snippet: exiting after last tabstop");
                self.exit_snippet_mode(window, cx);
                return false;
            }
        }

        state.current_tabstop_idx = next_idx;
        logging::log(
            "EDITOR",
            &format!(
                "Snippet: moved to tabstop {} (index {})",
                state.snippet.tabstops[next_idx].index, next_idx
            ),
        );

        self.select_current_tabstop(window, cx);
        true
    }

    /// Move to the previous tabstop. Returns true if we moved, false if we're at the start.
    fn prev_tabstop(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        // Guard: don't mutate snippet state until editor is ready
        if self.editor_state.is_none() {
            logging::log("EDITOR", "prev_tabstop: editor not initialized yet");
            return false;
        }

        // First, capture what the user typed at the current tabstop
        self.capture_current_tabstop_value(cx);

        let Some(ref mut state) = self.snippet_state else {
            return false;
        };

        if state.current_tabstop_idx == 0 {
            // Already at first tabstop
            return false;
        }

        state.current_tabstop_idx -= 1;
        logging::log(
            "EDITOR",
            &format!(
                "Snippet: moved to tabstop {} (index {})",
                state.snippet.tabstops[state.current_tabstop_idx].index, state.current_tabstop_idx
            ),
        );

        self.select_current_tabstop(window, cx);
        true
    }

    /// Capture the current tabstop's edited value before moving to another tabstop
    ///
    /// This is called before next_tabstop/prev_tabstop to record what the user typed,
    /// so we can calculate correct offsets for subsequent tabstops.
    ///
    /// The key insight: when the user types to replace a selected placeholder,
    /// the selection disappears and the cursor ends up at the end of what they typed.
    /// We need to read from the ORIGINAL start position of this tabstop to the
    /// current cursor position to capture what they actually typed.
    fn capture_current_tabstop_value(&mut self, cx: &mut Context<Self>) {
        // First, gather all the info we need with immutable borrows
        let (current_idx, tabstop_start_char, old_value) = {
            let Some(ref state) = self.snippet_state else {
                return;
            };
            let current_idx = state.current_tabstop_idx;
            if current_idx >= state.current_values.len() {
                return;
            }

            // Get the last known start position for this tabstop
            let tabstop_start_char = state
                .last_selection_ranges
                .get(current_idx)
                .and_then(|r| r.map(|(start, _)| start));

            let old_value = state.current_values[current_idx].clone();

            (current_idx, tabstop_start_char, old_value)
        };

        let Some(ref editor_state) = self.editor_state else {
            return;
        };

        // Get current editor state
        let (cursor_pos_char, selection_start_char, selection_end_char, full_text): (
            usize,
            usize,
            usize,
            String,
        ) = editor_state.update(cx, |input_state, _cx| {
            let selection = input_state.selection();
            let text = input_state.value();

            // Use cursor position (selection.end when collapsed, or end of selection)
            let cursor_byte = selection.end;
            let cursor_char = text
                .get(..cursor_byte)
                .map(|s| s.chars().count())
                .unwrap_or(0);

            let sel_start_char = text
                .get(..selection.start)
                .map(|s| s.chars().count())
                .unwrap_or(0);
            let sel_end_char = cursor_char;

            (cursor_char, sel_start_char, sel_end_char, text.to_string())
        });

        logging::log(
            "EDITOR",
            &format!(
                "Snippet capture: tabstop_idx={}, tabstop_start={:?}, cursor={}, selection=[{},{}), text='{}'",
                current_idx, tabstop_start_char, cursor_pos_char, selection_start_char, selection_end_char, full_text
            ),
        );

        // Determine the range to capture
        let (capture_start, capture_end) = if let Some(start) = tabstop_start_char {
            // We have a known start position - read from there to cursor
            // This handles the case where user typed to replace the placeholder
            (start, cursor_pos_char)
        } else {
            // Fallback: use original tabstop range adjusted for previous edits
            if let Some((adj_start, adj_end)) = self.calculate_adjusted_offset(current_idx) {
                (adj_start, adj_end)
            } else {
                return;
            }
        };

        // Extract the text at this range (convert char offsets to byte offsets)
        let captured_value: String = {
            let chars: Vec<char> = full_text.chars().collect();
            let start = capture_start.min(chars.len());
            let end = capture_end.min(chars.len());
            if start <= end {
                chars[start..end].iter().collect()
            } else {
                String::new()
            }
        };

        // Only update if we actually have something (could be empty if user deleted all)
        if captured_value != old_value {
            logging::log(
                "EDITOR",
                &format!(
                    "Snippet: captured tabstop {} value '{}' -> '{}' (range [{}, {}))",
                    current_idx, old_value, captured_value, capture_start, capture_end
                ),
            );
            // Now we can mutably borrow
            if let Some(ref mut state) = self.snippet_state {
                state.current_values[current_idx] = captured_value;
                state.last_selection_ranges[current_idx] = Some((capture_start, capture_end));
            }
        }
    }

    /// Calculate the adjusted offset for a tabstop based on edits to previous tabstops
    ///
    /// When a user edits tabstop 1 from "name" (4 chars) to "John Doe" (8 chars),
    /// tabstop 2's offset needs to shift by +4 characters.
    fn calculate_adjusted_offset(&self, tabstop_idx: usize) -> Option<(usize, usize)> {
        let state = self.snippet_state.as_ref()?;

        // Get the original range for this tabstop
        let original_range = state.snippet.tabstops.get(tabstop_idx)?.ranges.first()?;
        let (mut start, mut end) = *original_range;

        // Calculate cumulative offset adjustment from all previous tabstops
        for i in 0..tabstop_idx {
            let original_ts = state.snippet.tabstops.get(i)?;
            let original_placeholder = original_ts
                .placeholder
                .as_deref()
                .or_else(|| {
                    original_ts
                        .choices
                        .as_ref()
                        .and_then(|c| c.first().map(|s| s.as_str()))
                })
                .unwrap_or("");

            let current_value = state
                .current_values
                .get(i)
                .map(|s| s.as_str())
                .unwrap_or("");

            // Calculate the difference in character length
            let original_len = original_placeholder.chars().count();
            let current_len = current_value.chars().count();
            let diff = current_len as isize - original_len as isize;

            // Adjust if this tabstop was before our target (compare start positions)
            if let Some(&(ts_start, _)) = original_ts.ranges.first() {
                let (original_start, _) = *original_range;
                if ts_start < original_start {
                    start = (start as isize + diff).max(0) as usize;
                    end = (end as isize + diff).max(0) as usize;
                }
            }
        }

        Some((start, end))
    }

    /// Select the current tabstop placeholder text using gpui-component's set_selection API
    ///
    /// This method calculates the correct offset based on any edits the user has made
    /// to previous tabstops.
    fn select_current_tabstop(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Always clear any existing choice popup before moving to a new tabstop
        // This prevents stale popups from persisting when navigating tabstops
        self.choices_popup = None;

        // First, calculate the adjusted offset (needs immutable borrow)
        let adjusted_range = self.calculate_adjusted_offset_for_current();
        let Some((start, end, tabstop_index)) = adjusted_range else {
            logging::log("EDITOR", "Snippet: could not calculate adjusted offset");
            return;
        };

        let Some(ref editor_state) = self.editor_state else {
            return;
        };

        logging::log(
            "EDITOR",
            &format!(
                "Snippet: selecting tabstop {} adjusted range [{}, {})",
                tabstop_index, start, end
            ),
        );

        // Use gpui-component's set_selection to select the tabstop text
        editor_state.update(cx, |input_state, cx| {
            let text = input_state.value();
            let text_len = text.chars().count();

            // Clamp to valid range
            let start_clamped = start.min(text_len);
            let end_clamped = end.min(text_len);

            // Convert char offsets to byte offsets using the helper function
            // CRITICAL: This correctly handles end-of-document positions (e.g., $0)
            let start_bytes = char_offset_to_byte_offset(&text, start_clamped);
            let end_bytes = char_offset_to_byte_offset(&text, end_clamped);

            // Log what text we're actually selecting
            let selected_text = if start_bytes < end_bytes && end_bytes <= text.len() {
                &text[start_bytes..end_bytes]
            } else {
                ""
            };
            logging::log(
                "EDITOR",
                &format!(
                    "Snippet: setting selection bytes [{}, {}) = '{}' in text len={}, full_text='{}'",
                    start_bytes,
                    end_bytes,
                    selected_text,
                    text.len(),
                    text
                ),
            );

            input_state.set_selection(start_bytes, end_bytes, window, cx);
        });

        // Update the last selection range and check for choices
        if let Some(ref mut state) = self.snippet_state {
            let current_idx = state.current_tabstop_idx;
            if current_idx < state.last_selection_ranges.len() {
                state.last_selection_ranges[current_idx] = Some((start, end));
            }

            // Check if this tabstop has choices - if so, show the dropdown
            if let Some(tabstop) = state.snippet.tabstops.get(current_idx) {
                if let Some(ref choices) = tabstop.choices {
                    if choices.len() > 1 {
                        logging::log(
                            "EDITOR",
                            &format!(
                                "Snippet: tabstop {} has {} choices, showing popup",
                                current_idx,
                                choices.len()
                            ),
                        );
                        self.choices_popup = Some(ChoicesPopupState {
                            choices: choices.clone(),
                            selected_index: 0,
                            tabstop_idx: current_idx,
                        });
                    }
                }
            }
        }

        cx.notify();
    }

    /// Helper to calculate adjusted offset for the current tabstop
    /// Returns (start, end, tabstop_index) or None
    fn calculate_adjusted_offset_for_current(&self) -> Option<(usize, usize, usize)> {
        let state = self.snippet_state.as_ref()?;
        let current_idx = state.current_tabstop_idx;

        if current_idx >= state.snippet.tabstops.len() {
            return None;
        }

        let tabstop_index = state.snippet.tabstops[current_idx].index;
        let (start, end) = self.calculate_adjusted_offset(current_idx)?;
        Some((start, end, tabstop_index))
    }

    /// Exit snippet mode and restore normal Tab behavior
    fn exit_snippet_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.snippet_state.is_some() {
            logging::log("EDITOR", "Exiting snippet mode");
            self.snippet_state = None;
            // Always clear choice popup when exiting snippet mode
            self.choices_popup = None;

            // Disable tab navigation mode so Tab inserts tabs again
            if let Some(ref editor_state) = self.editor_state {
                editor_state.update(cx, |state, cx| {
                    state.set_tab_navigation(false, window, cx);
                });
            }
        }
    }

    /// Submit the current content
    fn submit(&self, cx: &Context<Self>) {
        let content = self.content(cx);
        logging::log("EDITOR", &format!("Submit id={}", self.id));
        (self.on_submit)(self.id.clone(), Some(content));
    }

    /// Cancel - submit None
    #[allow(dead_code)]
    fn cancel(&self) {
        logging::log("EDITOR", &format!("Cancel id={}", self.id));
        (self.on_submit)(self.id.clone(), None);
    }

    /// Focus the editor
    #[allow(dead_code)]
    pub fn focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(ref editor_state) = self.editor_state {
            editor_state.update(cx, |state, cx| {
                state.focus(window, cx);
            });
        }
    }

}
