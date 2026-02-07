    /// Render the arg input text with cursor and selection highlight
    fn render_arg_input_text(&self, text_primary: u32, accent_color: u32) -> gpui::Div {
        let text = self.arg_input.text();
        let chars: Vec<char> = text.chars().collect();
        let cursor_pos = self.arg_input.cursor();
        let has_selection = self.arg_input.has_selection();
        // Separate focus state from blink state to avoid layout shift
        let is_focused = self.focused_input == FocusedInput::ArgPrompt;
        let is_cursor_visible = is_focused && self.cursor_visible;

        if text.is_empty() {
            // Empty - always reserve cursor space, only show bg when visible
            // Note: height matches the fixed input_height (22px = CURSOR_HEIGHT_LG + 2*CURSOR_MARGIN_Y)
            return div()
                .flex()
                .flex_row()
                .items_center()
                .h(px(CURSOR_HEIGHT_LG + (CURSOR_MARGIN_Y * 2.0)))
                .child(
                    div()
                        .w(px(CURSOR_WIDTH))
                        .h(px(CURSOR_HEIGHT_LG))
                        .when(is_cursor_visible, |d: gpui::Div| d.bg(rgb(text_primary))),
                );
        }

        if has_selection {
            // With selection: before | selected | after (no cursor shown during selection)
            // Use fixed height matching the input container for consistent centering
            let selection = self.arg_input.selection();
            let (start, end) = selection.range();

            let before: String = chars[..start].iter().collect();
            let selected: String = chars[start..end].iter().collect();
            let after: String = chars[end..].iter().collect();

            div()
                .flex()
                .flex_row()
                .items_center()
                .h(px(CURSOR_HEIGHT_LG + (CURSOR_MARGIN_Y * 2.0)))
                .overflow_x_hidden()
                .when(!before.is_empty(), |d: gpui::Div| {
                    d.child(div().child(before))
                })
                .child(
                    div()
                        .bg(rgba((accent_color << 8) | 0x60))
                        // Use primary text color for selection - already set from theme
                        .text_color(rgb(text_primary))
                        .child(selected),
                )
                .when(!after.is_empty(), |d: gpui::Div| {
                    d.child(div().child(after))
                })
        } else {
            // No selection: before cursor | cursor | after cursor
            // Always reserve cursor space to prevent layout shift during blink
            // Use fixed height matching the input container for consistent centering
            let before: String = chars[..cursor_pos].iter().collect();
            let after: String = chars[cursor_pos..].iter().collect();

            div()
                .flex()
                .flex_row()
                .items_center()
                .h(px(CURSOR_HEIGHT_LG + (CURSOR_MARGIN_Y * 2.0)))
                .overflow_x_hidden()
                .when(!before.is_empty(), |d: gpui::Div| {
                    d.child(div().child(before))
                })
                // Always render cursor element, only show bg when visible
                .child(
                    div()
                        .w(px(CURSOR_WIDTH))
                        .h(px(CURSOR_HEIGHT_LG))
                        .when(is_cursor_visible, |d: gpui::Div| d.bg(rgb(text_primary))),
                )
                .when(!after.is_empty(), |d: gpui::Div| {
                    d.child(div().child(after))
                })
        }
    }
