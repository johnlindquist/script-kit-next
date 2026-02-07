impl Focusable for EditorPrompt {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
impl Render for EditorPrompt {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Ensure InputState is initialized on first render
        self.ensure_initialized(window, cx);

        // Handle deferred focus - focus the editor's InputState after initialization
        if self.needs_focus {
            if let Some(ref editor_state) = self.editor_state {
                editor_state.update(cx, |state, cx| {
                    state.focus(window, cx);
                });
                self.needs_focus = false;
                logging::log("EDITOR", "Editor focused via deferred focus");
            }
        }

        // Handle initial tabstop selection for templates
        if self.needs_initial_tabstop_selection && self.editor_state.is_some() {
            self.needs_initial_tabstop_selection = false;
            self.select_current_tabstop(window, cx);
            logging::log("EDITOR", "Initial tabstop selected");
        }

        let colors = &self.theme.colors;

        // Key handler for submit/cancel, snippet navigation, and choice popup
        // IMPORTANT: We intercept Tab here BEFORE gpui-component's Input processes it,
        // so we don't get tab characters inserted when navigating snippets.
        let handle_key = cx.listener(move |this, event: &gpui::KeyDownEvent, window, cx| {
            // Hide cursor while typing - automatically shows when mouse moves
            crate::platform::hide_cursor_until_mouse_moves();

            if this.suppress_keys {
                return;
            }

            let key = event.keystroke.key.to_lowercase();
            let cmd = event.keystroke.modifiers.platform;
            let shift = event.keystroke.modifiers.shift;

            // Debug logging for key events
            logging::log(
                "EDITOR",
                &format!(
                    "Key event: key='{}', cmd={}, shift={}, in_snippet_mode={}, choice_popup={}",
                    key,
                    cmd,
                    shift,
                    this.in_snippet_mode(),
                    this.is_choice_popup_visible()
                ),
            );

            // Handle choice popup keys first (takes priority)
            if this.is_choice_popup_visible() {
                match key.as_str() {
                    "up" | "arrowup" => {
                        this.choice_popup_up(cx);
                        return; // Don't propagate
                    }
                    "down" | "arrowdown" => {
                        this.choice_popup_down(cx);
                        return; // Don't propagate
                    }
                    "enter" | "return" if !cmd => {
                        this.choice_popup_confirm(window, cx);
                        return; // Don't propagate
                    }
                    "escape" | "esc" => {
                        this.choice_popup_cancel(cx);
                        return; // Don't propagate
                    }
                    "tab" if !shift => {
                        // Tab confirms the choice and moves to next tabstop
                        this.choice_popup_confirm(window, cx);
                        this.next_tabstop(window, cx);
                        return; // Don't propagate
                    }
                    _ => {
                        // Other keys close the popup and propagate
                        this.choice_popup_cancel(cx);
                        // Fall through to normal handling
                    }
                }
            }

            match (key.as_str(), cmd, shift) {
                // Cmd+Enter submits
                ("enter" | "return", true, _) => {
                    this.submit(cx);
                    // Don't propagate - we handled it
                }
                // Cmd+S also submits (save)
                ("s", true, _) => {
                    this.submit(cx);
                    // Don't propagate - we handled it
                }
                // Tab - snippet navigation (when in snippet mode)
                ("tab", false, false) if this.in_snippet_mode() => {
                    logging::log(
                        "EDITOR",
                        "Tab pressed in snippet mode - calling next_tabstop",
                    );
                    this.next_tabstop(window, cx);
                    // Don't propagate - prevents tab character insertion
                }
                // Shift+Tab - snippet navigation backwards (when in snippet mode)
                ("tab", false, true) if this.in_snippet_mode() => {
                    this.prev_tabstop(window, cx);
                    // Don't propagate - prevents tab character insertion
                }
                // Escape - exit snippet mode or let parent handle
                ("escape" | "esc", false, _) => {
                    if this.in_snippet_mode() {
                        this.exit_snippet_mode(window, cx);
                        cx.notify();
                        // Don't propagate when exiting snippet mode
                    } else {
                        // Let parent handle escape for closing the editor
                        cx.propagate();
                    }
                }
                _ => {
                    // Let other keys propagate to the Input component
                    cx.propagate();
                }
            }
        });

        // Calculate height - use the height passed from parent
        let _height = self.content_height.unwrap_or_else(|| px(500.)); // Default height if not specified

        // Get mono font family for code editor
        let fonts = self.theme.get_fonts();
        let mono_font: SharedString = fonts.mono_family.into();

        // Get font size from config (used for inner container inheritance)
        // KEEP as px() because:
        // 1. User explicitly configured a pixel size in config.ts
        // 2. Editor requires precise character sizing for monospace alignment
        let font_size = self.config.get_editor_font_size();

        // Action handlers for snippet Tab navigation
        // GPUI actions bubble up from focused element to parents, but only if the
        // focused element calls cx.propagate(). Since gpui-component's Input handles
        // IndentInline without propagating, we need to intercept at the Input wrapper level.
        let handle_indent = cx.listener(|this, _: &IndentInline, window, cx| {
            logging::log(
                "EDITOR",
                &format!(
                    "IndentInline action received, in_snippet_mode={}",
                    this.in_snippet_mode()
                ),
            );
            if this.in_snippet_mode() {
                this.next_tabstop(window, cx);
                // Don't propagate - we handled it
            } else {
                cx.propagate(); // Let Input handle normal indent
            }
        });

        let handle_outdent = cx.listener(|this, _: &OutdentInline, window, cx| {
            logging::log(
                "EDITOR",
                &format!(
                    "OutdentInline action received, in_snippet_mode={}",
                    this.in_snippet_mode()
                ),
            );
            if this.in_snippet_mode() {
                this.prev_tabstop(window, cx);
                // Don't propagate - we handled it
            } else {
                cx.propagate(); // Let Input handle normal outdent
            }
        });

        // Build the main container - code editor fills the space completely
        // Note: We don't track focus on the container because the InputState
        // has its own focus handle. Key events will be handled by the Input.
        //
        // Use size_full() to fill the parent container (which controls the actual size).
        // The parent wrapper in render_prompts/editor.rs uses flex_1 to allocate space.
        let mut container = div()
            .id("editor-v2")
            .flex()
            .flex_col()
            .size_full() // Fill parent container instead of explicit height
            .text_color(rgb(colors.text.primary))
            .font_family(mono_font.clone()) // Use monospace font for code
            .text_size(px(font_size)) // Apply configured font size
            .on_key_down(handle_key)
            .on_action(handle_indent)
            .on_action(handle_outdent);

        // Add the editor content if initialized
        if let Some(ref editor_state) = self.editor_state {
            container = container.child(
                div()
                    .size_full() // Fill the entire container
                    .overflow_hidden()
                    .text_size(px(font_size)) // Apply font size to inner container for inheritance
                    .font_family(mono_font.clone()) // Also apply mono font
                    // No padding - editor fills the space completely
                    // The Input component from gpui-component
                    // appearance(false) removes border styling for seamless integration
                    .child(Input::new(editor_state).size_full().appearance(false)),
            );
        } else {
            // Show loading placeholder while initializing
            container = container.child(
                div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child("Loading editor..."),
            );
        }

        // NOTE: Footer rendering has been moved to the unified PromptFooter component
        // in render_prompts/editor.rs. The snippet state and language are passed to that footer.

        // Add relative positioning to container for choices popup overlay
        container = container.relative();

        // Add the choices popup overlay if visible
        if let Some(popup_element) = self.render_choices_popup(cx) {
            container = container.child(popup_element);
        }

        container
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_creation() {
        // Basic smoke test - just verify the struct can be created with expected fields
        // Full integration tests require GPUI context
    }

    #[test]
    fn test_char_offset_to_position_single_line() {
        let text = "Hello World";
        let pos0 = char_offset_to_position(text, 0);
        assert_eq!((pos0.line, pos0.character), (0, 0));

        let pos5 = char_offset_to_position(text, 5);
        assert_eq!((pos5.line, pos5.character), (0, 5));

        let pos11 = char_offset_to_position(text, 11);
        assert_eq!((pos11.line, pos11.character), (0, 11));
    }

    #[test]
    fn test_char_offset_to_position_multi_line() {
        let text = "Hello\nWorld\nTest";
        // Line 0: "Hello" (0-4), newline at 5
        // Line 1: "World" (6-10), newline at 11
        // Line 2: "Test" (12-15)
        let pos0 = char_offset_to_position(text, 0);
        assert_eq!((pos0.line, pos0.character), (0, 0)); // 'H'

        let pos5 = char_offset_to_position(text, 5);
        assert_eq!((pos5.line, pos5.character), (0, 5)); // '\n'

        let pos6 = char_offset_to_position(text, 6);
        assert_eq!((pos6.line, pos6.character), (1, 0)); // 'W'

        let pos11 = char_offset_to_position(text, 11);
        assert_eq!((pos11.line, pos11.character), (1, 5)); // '\n'

        let pos12 = char_offset_to_position(text, 12);
        assert_eq!((pos12.line, pos12.character), (2, 0)); // 'T'

        let pos16 = char_offset_to_position(text, 16);
        assert_eq!((pos16.line, pos16.character), (2, 4)); // past end
    }

    #[test]
    fn test_char_offset_to_position_empty() {
        let text = "";
        let pos = char_offset_to_position(text, 0);
        assert_eq!((pos.line, pos.character), (0, 0));
    }

    #[test]
    fn test_snippet_state_creation() {
        // Test that SnippetState is properly initialized from a template
        let snippet = ParsedSnippet::parse("Hello ${1:name}!");

        let current_values = vec!["name".to_string()];
        let last_selection_ranges = vec![Some((6, 10))];

        let state = SnippetState {
            snippet: snippet.clone(),
            current_tabstop_idx: 0,
            current_values,
            last_selection_ranges,
        };

        assert_eq!(state.current_tabstop_idx, 0);
        assert_eq!(state.snippet.tabstops.len(), 1);
        assert_eq!(state.snippet.tabstops[0].index, 1);
        assert_eq!(state.snippet.text, "Hello name!");
    }

    #[test]
    fn test_snippet_state_multiple_tabstops() {
        let snippet = ParsedSnippet::parse("Hello ${1:name}, welcome to ${2:place}!");

        let current_values = vec!["name".to_string(), "place".to_string()];
        let last_selection_ranges = vec![Some((6, 10)), Some((23, 28))];

        let state = SnippetState {
            snippet,
            current_tabstop_idx: 0,
            current_values,
            last_selection_ranges,
        };

        assert_eq!(state.snippet.tabstops.len(), 2);
        assert_eq!(state.snippet.tabstops[0].index, 1);
        assert_eq!(state.snippet.tabstops[1].index, 2);
        assert_eq!(state.snippet.text, "Hello name, welcome to place!");
    }

    #[test]
    fn test_snippet_state_with_final_cursor() {
        let snippet = ParsedSnippet::parse("Hello ${1:name}!$0");

        let current_values = vec!["name".to_string(), "".to_string()];
        let last_selection_ranges = vec![Some((6, 10)), Some((11, 11))];

        let state = SnippetState {
            snippet,
            current_tabstop_idx: 0,
            current_values,
            last_selection_ranges,
        };

        // Should have 2 tabstops: index 1 first, then index 0 ($0) at end
        assert_eq!(state.snippet.tabstops.len(), 2);
        assert_eq!(state.snippet.tabstops[0].index, 1);
        assert_eq!(state.snippet.tabstops[1].index, 0);
    }

    // =========================================================================
    // char_offset_to_byte_offset tests - CRITICAL for correct cursor placement
    // =========================================================================

    #[test]
    fn test_char_to_byte_offset_basic() {
        let text = "Hello";
        assert_eq!(char_offset_to_byte_offset(text, 0), 0); // 'H'
        assert_eq!(char_offset_to_byte_offset(text, 1), 1); // 'e'
        assert_eq!(char_offset_to_byte_offset(text, 5), 5); // end of string (equals len)
    }

    #[test]
    fn test_char_to_byte_offset_at_end_of_document() {
        // CRITICAL: This is the bug fix - offset at end should NOT return 0
        let text = "Hello";
        // Char offset 5 (end of 5-char string) should return byte offset 5, not 0
        assert_eq!(char_offset_to_byte_offset(text, 5), 5);

        // Beyond end should also return text.len()
        assert_eq!(char_offset_to_byte_offset(text, 100), 5);
    }

    #[test]
    fn test_char_to_byte_offset_empty_string() {
        let text = "";
        // Empty string: any offset should return 0 (which equals text.len())
        assert_eq!(char_offset_to_byte_offset(text, 0), 0);
        assert_eq!(char_offset_to_byte_offset(text, 1), 0);
    }

    #[test]
    fn test_char_to_byte_offset_unicode() {
        // "你好" = 2 chars, 6 bytes (3 bytes per CJK char)
        let text = "你好";
        assert_eq!(text.len(), 6); // 6 bytes
        assert_eq!(text.chars().count(), 2); // 2 chars

        assert_eq!(char_offset_to_byte_offset(text, 0), 0); // '你' at byte 0
        assert_eq!(char_offset_to_byte_offset(text, 1), 3); // '好' at byte 3
        assert_eq!(char_offset_to_byte_offset(text, 2), 6); // end = byte length
    }

    #[test]
    fn test_char_to_byte_offset_mixed_unicode() {
        // "Hi你好" = 4 chars, 8 bytes
        let text = "Hi你好";
        assert_eq!(text.len(), 8); // 2 + 3 + 3 = 8 bytes
        assert_eq!(text.chars().count(), 4); // 4 chars

        assert_eq!(char_offset_to_byte_offset(text, 0), 0); // 'H'
        assert_eq!(char_offset_to_byte_offset(text, 1), 1); // 'i'
        assert_eq!(char_offset_to_byte_offset(text, 2), 2); // '你'
        assert_eq!(char_offset_to_byte_offset(text, 3), 5); // '好'
        assert_eq!(char_offset_to_byte_offset(text, 4), 8); // end
    }

    #[test]
    fn test_char_to_byte_offset_emoji() {
        // "Hello 🌍" = 7 chars, but 🌍 is 4 bytes
        let text = "Hello 🌍";
        assert_eq!(text.chars().count(), 7);
        assert!(text.len() > 7); // bytes > chars

        assert_eq!(char_offset_to_byte_offset(text, 0), 0); // 'H'
        assert_eq!(char_offset_to_byte_offset(text, 6), 6); // '🌍' starts at byte 6
        assert_eq!(char_offset_to_byte_offset(text, 7), text.len()); // end
    }

    #[test]
    fn test_char_to_byte_offset_snippet_final_cursor() {
        // Simulate $0 at end of "Hello name!"
        // This is the exact scenario that was broken before the fix
        let text = "Hello name!";
        let text_len = text.chars().count(); // 11

        // $0 range is (11, 11) - cursor at very end
        let start_clamped = 11_usize.min(text_len);
        let end_clamped = 11_usize.min(text_len);

        // Both should be 11 (byte length), NOT 0
        let start_bytes = char_offset_to_byte_offset(text, start_clamped);
        let end_bytes = char_offset_to_byte_offset(text, end_clamped);

        assert_eq!(start_bytes, 11, "start_bytes should be 11, not 0!");
        assert_eq!(end_bytes, 11, "end_bytes should be 11");

        // This is a zero-length selection at the end (cursor, not selection)
        assert_eq!(start_bytes, end_bytes);
    }
}
