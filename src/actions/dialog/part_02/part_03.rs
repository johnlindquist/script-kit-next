impl ActionsDialog {

    /// Move selection up, skipping section headers
    ///
    /// When moving up and landing on a section header, we must search UPWARD
    /// (not downward) to find the previous selectable item. This ensures
    /// navigation past section headers works correctly.
    pub fn move_up(&mut self, cx: &mut Context<Self>) {
        if self.selected_index == 0 {
            return;
        }

        // Search backwards from current position to find the previous selectable item
        // This correctly skips section headers when moving up
        for i in (0..self.selected_index).rev() {
            if matches!(self.grouped_items.get(i), Some(GroupedActionItem::Item(_))) {
                self.selected_index = i;
                self.list_state.scroll_to_reveal_item(self.selected_index);
                logging::log_debug(
                    "ACTIONS_SCROLL",
                    &format!("Up: selected_index={}", self.selected_index),
                );
                cx.notify();
                return;
            }
        }
    }

    /// Move selection down, skipping section headers
    pub fn move_down(&mut self, cx: &mut Context<Self>) {
        if self.selected_index < self.grouped_items.len().saturating_sub(1) {
            let new_index = self.selected_index + 1;
            // Skip section headers - search forward
            for i in new_index..self.grouped_items.len() {
                if matches!(self.grouped_items.get(i), Some(GroupedActionItem::Item(_))) {
                    self.selected_index = i;
                    self.list_state.scroll_to_reveal_item(self.selected_index);
                    logging::log_debug(
                        "ACTIONS_SCROLL",
                        &format!("Down: selected_index={}", self.selected_index),
                    );
                    cx.notify();
                    break;
                }
            }
        }
    }

    /// Get the currently selected action ID (for external handling)
    pub fn get_selected_action_id(&self) -> Option<String> {
        self.get_selected_action().map(|action| action.id.clone())
    }

    /// Get the currently selected ProtocolAction (for checking close behavior)
    /// Returns the original ProtocolAction from sdk_actions if this is an SDK action,
    /// or None for built-in actions.
    pub fn get_selected_protocol_action(&self) -> Option<&ProtocolAction> {
        let protocol_action_index = resolve_selected_protocol_action_index(
            self.selected_action_index(),
            &self.sdk_action_indices,
        )?;
        self.sdk_actions.as_ref()?.get(protocol_action_index)
    }

    /// Check if the currently selected action should close the dialog
    /// Returns true if the action has close: true (or no close field, which defaults to true)
    /// Returns true for built-in actions (they always close)
    pub fn selected_action_should_close(&self) -> bool {
        if let Some(protocol_action) = self.get_selected_protocol_action() {
            protocol_action.should_close()
        } else {
            // Built-in actions always close
            true
        }
    }

    /// Submit the selected action
    pub fn submit_selected(&mut self) {
        // Get action from grouped_items -> filtered_actions -> actions chain
        if let Some(action) = self.get_selected_action() {
            let action_id = action.id.clone();
            logging::log("ACTIONS", &format!("Action selected: {}", action_id));
            (self.on_select)(action_id);
        }
    }

    /// Cancel - close the dialog
    pub fn submit_cancel(&mut self) {
        logging::log("ACTIONS", "Actions dialog cancelled");
        (self.on_select)("__cancel__".to_string());
    }

    /// Dismiss the dialog when user clicks outside its bounds.
    /// This is a public method called from the parent container's click-outside handler.
    /// Logs the event and triggers the cancel callback.
    pub fn dismiss_on_click_outside(&mut self) {
        tracing::info!(
            target: "script_kit::actions",
            "ActionsDialog dismiss-on-click-outside triggered"
        );
        logging::log("ACTIONS", "Actions dialog dismissed (click outside)");
        self.submit_cancel();
    }

    /// Create box shadow for the overlay popup
    /// When rendered in a separate vibrancy window, no shadow is needed
    /// (the window vibrancy provides visual separation)
    pub(super) fn create_popup_shadow() -> Vec<BoxShadow> {
        // No shadow - vibrancy window provides visual separation
        vec![]
    }

    /// Get colors for the search box based on design variant
    /// Returns: (search_box_bg, border_color, muted_text, dimmed_text, secondary_text)
    pub(super) fn get_search_colors(
        &self,
        colors: &crate::designs::DesignColors,
    ) -> (gpui::Rgba, gpui::Rgba, gpui::Rgba, gpui::Rgba, gpui::Rgba) {
        // Use theme opacity for input background to support vibrancy
        let opacity = self.theme.get_opacity();
        let input_alpha = (opacity.input * 255.0) as u8;
        // Use theme-aware border opacity for proper light/dark mode support
        // Light mode: ~30% opacity, Dark mode: ~50% opacity (via border_inactive)
        let border_alpha = ((opacity.border_inactive * 2.0).min(1.0) * 255.0) as u8;

        if self.design_variant == DesignVariant::Default {
            (
                rgba(hex_with_alpha(
                    self.theme.colors.background.search_box,
                    input_alpha,
                )),
                rgba(hex_with_alpha(self.theme.colors.ui.border, border_alpha)),
                rgb(self.theme.colors.text.muted),
                rgb(self.theme.colors.text.dimmed),
                rgb(self.theme.colors.text.secondary),
            )
        } else {
            (
                rgba(hex_with_alpha(colors.background_secondary, input_alpha)),
                rgba(hex_with_alpha(colors.border, border_alpha)),
                rgb(colors.text_muted),
                rgb(colors.text_dimmed),
                rgb(colors.text_secondary),
            )
        }
    }

    /// Get colors for the main container based on design variant
    /// Returns: (main_bg, container_border, container_text)
    pub(super) fn get_container_colors(
        &self,
        colors: &crate::designs::DesignColors,
    ) -> (gpui::Rgba, gpui::Rgba, gpui::Rgba) {
        // Vibrancy-aware dialog background:
        // - When vibrancy enabled: ~50% opacity to show blur but remain visible
        // - When vibrancy disabled: ~95% opacity for near-solid appearance
        let use_vibrancy = self.theme.is_vibrancy_enabled();
        let dialog_alpha = if use_vibrancy {
            // Dialogs need higher opacity than main window (0.37) to stand out
            (0.50 * 255.0) as u8
        } else {
            // Near-opaque when vibrancy disabled
            (0.95 * 255.0) as u8
        };

        if self.design_variant == DesignVariant::Default {
            (
                rgba(hex_with_alpha(
                    self.theme.colors.background.main,
                    dialog_alpha,
                )),
                rgba(hex_with_alpha(self.theme.colors.ui.border, 0x80)),
                rgb(self.theme.colors.text.secondary),
            )
        } else {
            (
                rgba(hex_with_alpha(colors.background, dialog_alpha)),
                rgba(hex_with_alpha(colors.border, 0x80)),
                rgb(colors.text_secondary),
            )
        }
    }
}
