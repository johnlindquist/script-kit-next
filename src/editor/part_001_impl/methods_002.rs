impl EditorPrompt {
    /// Request focus on next render (useful when called outside of render context)
    #[allow(dead_code)]
    pub fn request_focus(&mut self) {
        self.needs_focus = true;
    }

    // === Choice Popup Methods ===

    /// Check if the choice popup is currently visible
    pub fn is_choice_popup_visible(&self) -> bool {
        self.choices_popup.is_some()
    }

    /// Public wrapper for choice_popup_up (for SimulateKey)
    pub fn choice_popup_up_public(&mut self, cx: &mut Context<Self>) {
        self.choice_popup_up(cx);
    }

    /// Public wrapper for choice_popup_down (for SimulateKey)
    pub fn choice_popup_down_public(&mut self, cx: &mut Context<Self>) {
        self.choice_popup_down(cx);
    }

    /// Public wrapper for choice_popup_confirm (for SimulateKey)
    pub fn choice_popup_confirm_public(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.choice_popup_confirm(window, cx);
    }

    /// Public wrapper for choice_popup_cancel (for SimulateKey)
    pub fn choice_popup_cancel_public(&mut self, cx: &mut Context<Self>) {
        self.choice_popup_cancel(cx);
    }

    /// Move selection up in the choice popup
    fn choice_popup_up(&mut self, cx: &mut Context<Self>) {
        if let Some(ref mut popup) = self.choices_popup {
            if popup.selected_index > 0 {
                popup.selected_index -= 1;
                logging::log(
                    "EDITOR",
                    &format!("Choice popup: moved up to index {}", popup.selected_index),
                );
                cx.notify();
            }
        }
    }

    /// Move selection down in the choice popup
    fn choice_popup_down(&mut self, cx: &mut Context<Self>) {
        if let Some(ref mut popup) = self.choices_popup {
            if popup.selected_index + 1 < popup.choices.len() {
                popup.selected_index += 1;
                logging::log(
                    "EDITOR",
                    &format!("Choice popup: moved down to index {}", popup.selected_index),
                );
                cx.notify();
            }
        }
    }

    /// Confirm the current choice and replace the selection
    fn choice_popup_confirm(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(popup) = self.choices_popup.take() else {
            return;
        };

        let Some(chosen) = popup.choices.get(popup.selected_index).cloned() else {
            return;
        };

        logging::log(
            "EDITOR",
            &format!(
                "Choice popup: confirmed '{}' at index {}",
                chosen, popup.selected_index
            ),
        );

        // Replace the current selection with the chosen text
        // CRITICAL: Use replace() not insert() - insert() only inserts at cursor position
        // (cursor..cursor range), while replace() replaces the current selection (None = use selection)
        if let Some(ref editor_state) = self.editor_state {
            editor_state.update(cx, |input_state, cx| {
                // The current tabstop text should be selected
                // replace() uses the current selection, insert() only inserts at cursor
                input_state.replace(chosen.clone(), window, cx);
            });
        }

        // Update current_values for offset tracking
        if let Some(ref mut state) = self.snippet_state {
            if popup.tabstop_idx < state.current_values.len() {
                state.current_values[popup.tabstop_idx] = chosen.clone();
            }
        }

        cx.notify();
    }

    /// Cancel the choice popup (dismiss without changing selection)
    fn choice_popup_cancel(&mut self, cx: &mut Context<Self>) {
        if self.choices_popup.is_some() {
            logging::log("EDITOR", "Choice popup: cancelled");
            self.choices_popup = None;
            cx.notify();
        }
    }

    /// Render the choice popup overlay
    fn render_choices_popup(&self, _cx: &Context<Self>) -> Option<impl IntoElement> {
        let popup = self.choices_popup.as_ref()?;
        let colors = &self.theme.colors;

        Some(
            div()
                .absolute()
                .top(px(40.)) // Position below the editor toolbar area
                .left(px(16.))
                //.z_index(1000) // Not available in GPUI, using layer order instead
                .min_w(px(200.))
                .max_w(px(400.))
                .bg(rgb(colors.background.main))
                .border_1()
                .border_color(rgb(colors.ui.border))
                .rounded_md()
                // Only apply shadow when vibrancy is disabled - shadows block blur
                .when(!self.theme.is_vibrancy_enabled(), |d| d.shadow_lg())
                .py(px(4.))
                .children(popup.choices.iter().enumerate().map(|(idx, choice)| {
                    let is_selected = idx == popup.selected_index;
                    let bg_color = if is_selected {
                        rgb(colors.accent.selected)
                    } else {
                        rgb(colors.background.main)
                    };
                    // Use contrasting text color for selected item
                    let text_color = if is_selected {
                        // Use on_accent color for text on accent backgrounds
                        rgb(colors.text.on_accent)
                    } else {
                        rgb(colors.text.primary)
                    };

                    div()
                        .px(px(12.))
                        .py(px(6.))
                        .bg(bg_color)
                        .text_color(text_color)
                        .text_sm()
                        .cursor_pointer()
                        .child(choice.clone())
                })),
        )
    }
}
