const ACTIONS_DIALOG_COLOR_ALPHA_MAX: f32 = 255.0;
const ACTIONS_DIALOG_SEARCH_BORDER_ALPHA_SCALE: f32 = 2.0;
const ACTIONS_DIALOG_CONTAINER_BORDER_MIN_ALPHA: u8 = 0x80;
const ACTIONS_DIALOG_VIBRANT_DIALOG_MIN_OPACITY: f32 = 0.50;
const ACTIONS_DIALOG_OPAQUE_DIALOG_MIN_OPACITY: f32 = 0.95;

fn actions_dialog_alpha_u8(opacity: f32) -> u8 {
    (opacity.clamp(0.0, 1.0) * ACTIONS_DIALOG_COLOR_ALPHA_MAX) as u8
}

fn actions_dialog_search_border_alpha(border_inactive_opacity: f32) -> u8 {
    let scaled_border_opacity =
        (border_inactive_opacity * ACTIONS_DIALOG_SEARCH_BORDER_ALPHA_SCALE).min(1.0);
    actions_dialog_alpha_u8(scaled_border_opacity)
}

fn actions_dialog_container_border_alpha(border_inactive_opacity: f32) -> u8 {
    actions_dialog_search_border_alpha(border_inactive_opacity)
        .max(ACTIONS_DIALOG_CONTAINER_BORDER_MIN_ALPHA)
}

fn actions_dialog_container_background_alpha(dialog_opacity: f32, use_vibrancy: bool) -> u8 {
    let minimum_dialog_opacity = if use_vibrancy {
        ACTIONS_DIALOG_VIBRANT_DIALOG_MIN_OPACITY
    } else {
        ACTIONS_DIALOG_OPAQUE_DIALOG_MIN_OPACITY
    };
    actions_dialog_alpha_u8(dialog_opacity.max(minimum_dialog_opacity))
}

fn actions_dialog_rgba_with_alpha(hex: u32, alpha: u8) -> gpui::Rgba {
    rgba(hex_with_alpha(hex, alpha))
}

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
        let input_alpha = actions_dialog_alpha_u8(opacity.input);
        // Keep search and container borders on the same opacity scaling path.
        let border_alpha = actions_dialog_search_border_alpha(opacity.border_inactive);
        let (search_box_background, search_box_border, muted_text, dimmed_text, secondary_text) =
            if self.design_variant == DesignVariant::Default {
                (
                    self.theme.colors.background.search_box,
                    self.theme.colors.ui.border,
                    self.theme.colors.text.muted,
                    self.theme.colors.text.dimmed,
                    self.theme.colors.text.secondary,
                )
            } else {
                (
                    colors.background_secondary,
                    colors.border,
                    colors.text_muted,
                    colors.text_dimmed,
                    colors.text_secondary,
                )
            };

        (
            actions_dialog_rgba_with_alpha(search_box_background, input_alpha),
            actions_dialog_rgba_with_alpha(search_box_border, border_alpha),
            rgb(muted_text),
            rgb(dimmed_text),
            rgb(secondary_text),
        )
    }

    /// Get colors for the main container based on design variant
    /// Returns: (main_bg, container_border, container_text)
    pub(super) fn get_container_colors(
        &self,
        colors: &crate::designs::DesignColors,
    ) -> (gpui::Rgba, gpui::Rgba, gpui::Rgba) {
        let opacity = self.theme.get_opacity();
        let use_vibrancy = self.theme.is_vibrancy_enabled();
        // Keep the current readability floors while allowing stronger user-configured dialog
        // opacity values to take effect.
        let dialog_alpha = actions_dialog_container_background_alpha(opacity.dialog, use_vibrancy);
        let border_alpha = actions_dialog_container_border_alpha(opacity.border_inactive);
        let (main_background, container_border, container_text) =
            if self.design_variant == DesignVariant::Default {
                (
                    self.theme.colors.background.main,
                    self.theme.colors.ui.border,
                    self.theme.colors.text.secondary,
                )
            } else {
                (colors.background, colors.border, colors.text_secondary)
            };

        (
            actions_dialog_rgba_with_alpha(main_background, dialog_alpha),
            actions_dialog_rgba_with_alpha(container_border, border_alpha),
            rgb(container_text),
        )
    }
}

#[cfg(test)]
mod actions_dialog_opacity_consistency_tests {
    use super::{
        actions_dialog_container_background_alpha, actions_dialog_container_border_alpha,
        actions_dialog_rgba_with_alpha, actions_dialog_search_border_alpha,
        ACTIONS_DIALOG_CONTAINER_BORDER_MIN_ALPHA,
    };
    use gpui::rgba;

    #[test]
    fn test_actions_dialog_search_border_alpha_scales_border_inactive_opacity() {
        assert_eq!(actions_dialog_search_border_alpha(0.20), 102);
    }

    #[test]
    fn test_actions_dialog_container_border_alpha_enforces_minimum_contrast() {
        assert_eq!(
            actions_dialog_container_border_alpha(0.10),
            ACTIONS_DIALOG_CONTAINER_BORDER_MIN_ALPHA
        );
    }

    #[test]
    fn test_actions_dialog_container_background_alpha_keeps_vibrancy_floor() {
        assert_eq!(actions_dialog_container_background_alpha(0.15, true), 127);
    }

    #[test]
    fn test_actions_dialog_container_background_alpha_keeps_non_vibrancy_floor() {
        assert_eq!(actions_dialog_container_background_alpha(0.15, false), 242);
    }

    #[test]
    fn test_actions_dialog_container_background_alpha_uses_higher_theme_value() {
        assert_eq!(actions_dialog_container_background_alpha(0.80, true), 204);
    }

    #[test]
    fn test_actions_dialog_rgba_with_alpha_combines_hex_and_alpha_channels() {
        assert_eq!(
            actions_dialog_rgba_with_alpha(0x112233, 0x44),
            rgba(0x11223344)
        );
    }
}
