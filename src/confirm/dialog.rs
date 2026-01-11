//! Confirm Dialog
//!
//! A simple confirmation dialog with a message and two buttons (Cancel/Confirm).
//! Supports keyboard shortcuts: Enter = confirm, Escape = cancel.
//! Tab/Arrow keys navigate between buttons with visual focus indication.

use crate::logging;
use crate::theme;
use gpui::{
    div, prelude::*, px, rgb, rgba, App, Context, FocusHandle, Focusable, Render, SharedString,
    Window,
};
use std::sync::Arc;

use super::constants::{
    BUTTON_GAP, BUTTON_PADDING_X, BUTTON_PADDING_Y, BUTTON_RADIUS, BUTTON_ROW_HEIGHT,
    CONFIRM_PADDING, CONFIRM_WIDTH, DIALOG_RADIUS,
};

/// Callback for confirm/cancel selection
/// Signature: (confirmed: bool)
pub type ConfirmCallback = Arc<dyn Fn(bool) + Send + Sync>;

/// Helper function to combine a hex color with an alpha value
#[inline]
fn hex_with_alpha(hex: u32, alpha: u8) -> u32 {
    (hex << 8) | (alpha as u32)
}

/// Focus ring border width for focused buttons
const FOCUS_BORDER_WIDTH: f32 = 2.0;

/// ConfirmDialog - Simple confirmation modal with message and two buttons
pub struct ConfirmDialog {
    /// The message to display
    pub message: String,
    /// Text for the confirm button (default: "OK")
    pub confirm_text: String,
    /// Text for the cancel button (default: "Cancel")
    pub cancel_text: String,
    /// Focus handle for keyboard events
    pub focus_handle: FocusHandle,
    /// Callback when user makes a choice
    pub on_choice: ConfirmCallback,
    /// Theme for consistent styling
    pub theme: Arc<theme::Theme>,
    /// Which button is currently focused (0 = cancel, 1 = confirm)
    pub focused_button: usize,
}

impl ConfirmDialog {
    pub fn new(
        message: impl Into<String>,
        confirm_text: Option<String>,
        cancel_text: Option<String>,
        focus_handle: FocusHandle,
        on_choice: ConfirmCallback,
        theme: Arc<theme::Theme>,
    ) -> Self {
        let message_str = message.into();
        logging::log(
            "CONFIRM",
            &format!("ConfirmDialog created: {:?}", message_str),
        );

        Self {
            message: message_str,
            confirm_text: confirm_text.unwrap_or_else(|| "OK".to_string()),
            cancel_text: cancel_text.unwrap_or_else(|| "Cancel".to_string()),
            focus_handle,
            on_choice,
            theme,
            focused_button: 1, // Default focus on confirm button
        }
    }

    /// Handle left arrow key - move focus to cancel button
    pub fn focus_cancel(&mut self, cx: &mut Context<Self>) {
        if self.focused_button != 0 {
            self.focused_button = 0;
            cx.notify();
        }
    }

    /// Handle right arrow key - move focus to confirm button
    pub fn focus_confirm(&mut self, cx: &mut Context<Self>) {
        if self.focused_button != 1 {
            self.focused_button = 1;
            cx.notify();
        }
    }

    /// Handle Tab key - toggle between buttons
    pub fn toggle_focus(&mut self, cx: &mut Context<Self>) {
        self.focused_button = 1 - self.focused_button;
        cx.notify();
    }

    /// Submit the current selection (Enter key or clicking focused button)
    pub fn submit(&mut self) {
        let confirmed = self.focused_button == 1;
        logging::log(
            "CONFIRM",
            &format!(
                "User chose: {}",
                if confirmed { "confirm" } else { "cancel" }
            ),
        );
        (self.on_choice)(confirmed);
    }

    /// Cancel the dialog (Escape key)
    pub fn cancel(&mut self) {
        logging::log("CONFIRM", "User cancelled");
        (self.on_choice)(false);
    }

    /// Direct confirm (clicking confirm button)
    pub fn confirm(&mut self) {
        logging::log("CONFIRM", "User confirmed");
        (self.on_choice)(true);
    }
}

impl Focusable for ConfirmDialog {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ConfirmDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Get theme colors
        let colors = &self.theme.colors;
        let opacity = self.theme.get_opacity();

        // Background with vibrancy support
        let use_vibrancy = self.theme.is_vibrancy_enabled();
        let dialog_alpha = (opacity.dialog * 255.0) as u8;
        let main_bg = rgba(hex_with_alpha(colors.background.main, dialog_alpha));

        // Text colors
        let primary_text = rgb(colors.text.primary);
        let secondary_text = rgb(colors.text.secondary);

        // Button colors - both buttons now use consistent semi-transparent styling
        let border_color = rgba(hex_with_alpha(colors.ui.border, 0x60));
        let button_bg = rgba(hex_with_alpha(colors.background.search_box, 0x40));

        // Confirm button uses accent color with reduced opacity for vibrancy consistency
        // 0x60 = 37.5% opacity - visible but not harsh like solid yellow
        let confirm_bg = rgba(hex_with_alpha(colors.accent.selected, 0x50));
        let confirm_hover = rgba(hex_with_alpha(colors.accent.selected, 0x70));
        let cancel_hover = rgba(hex_with_alpha(colors.accent.selected_subtle, 0x30));

        // Focus colors - visible ring around focused button
        // Higher alpha (0xA0 = 62.5%) for clear focus indication
        let focus_ring_color = rgba(hex_with_alpha(colors.accent.selected, 0xA0));
        let unfocused_border = rgba(hex_with_alpha(colors.ui.border, 0x40));

        // Focus background tint - subtle background change when button is focused
        let focus_tint = rgba(hex_with_alpha(colors.accent.selected_subtle, 0x20));

        let message_str: SharedString = self.message.clone().into();
        let cancel_str: SharedString = self.cancel_text.clone().into();
        let confirm_str: SharedString = self.confirm_text.clone().into();

        let is_cancel_focused = self.focused_button == 0;
        let is_confirm_focused = self.focused_button == 1;

        // Cancel button - secondary action with subtle styling
        // When focused: accent border ring + slight background tint
        let cancel_button = div()
            .id("cancel-button")
            .flex_1()
            .h(px(BUTTON_ROW_HEIGHT))
            .px(px(BUTTON_PADDING_X))
            .py(px(BUTTON_PADDING_Y))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(BUTTON_RADIUS))
            .bg(if is_cancel_focused {
                // Add subtle tint when focused
                focus_tint
            } else {
                button_bg
            })
            .border_color(if is_cancel_focused {
                focus_ring_color
            } else {
                unfocused_border
            })
            .when(is_cancel_focused, |d| d.border(px(FOCUS_BORDER_WIDTH)))
            .when(!is_cancel_focused, |d| d.border_1())
            .hover(|s| s.bg(cancel_hover))
            .cursor_pointer()
            .text_color(secondary_text)
            .text_sm()
            .child(cancel_str)
            .on_click(cx.listener(|this, _e, window, _cx| {
                this.cancel();
                window.remove_window();
            }));

        // Confirm button - primary action with accent color at reduced opacity
        // Matches vibrancy theme while still standing out as the primary action
        // When focused: brighter accent border ring
        let confirm_button = div()
            .id("confirm-button")
            .flex_1()
            .h(px(BUTTON_ROW_HEIGHT))
            .px(px(BUTTON_PADDING_X))
            .py(px(BUTTON_PADDING_Y))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(BUTTON_RADIUS))
            .bg(if is_confirm_focused {
                // Slightly brighter when focused
                rgba(hex_with_alpha(colors.accent.selected, 0x60))
            } else {
                confirm_bg
            })
            .border_color(if is_confirm_focused {
                focus_ring_color
            } else {
                // Subtle accent border even when unfocused to indicate primary action
                rgba(hex_with_alpha(colors.accent.selected, 0x40))
            })
            .when(is_confirm_focused, |d| d.border(px(FOCUS_BORDER_WIDTH)))
            .when(!is_confirm_focused, |d| d.border_1())
            .hover(|s| s.bg(confirm_hover))
            .cursor_pointer()
            // Use primary text color for better readability on semi-transparent bg
            .text_color(primary_text)
            .text_sm()
            .font_weight(gpui::FontWeight::MEDIUM)
            .child(confirm_str)
            .on_click(cx.listener(|this, _e, window, _cx| {
                this.confirm();
                window.remove_window();
            }));

        // Button row
        let button_row = div()
            .w_full()
            .flex()
            .flex_row()
            .gap(px(BUTTON_GAP))
            .child(cancel_button)
            .child(confirm_button);

        // Main dialog container
        div()
            .w(px(CONFIRM_WIDTH))
            .flex()
            .flex_col()
            .p(px(CONFIRM_PADDING))
            .gap(px(CONFIRM_PADDING))
            .when(!use_vibrancy, |d| d.bg(main_bg))
            .rounded(px(DIALOG_RADIUS))
            .border_1()
            .border_color(border_color)
            .overflow_hidden()
            .track_focus(&self.focus_handle)
            .key_context("confirm_dialog")
            // Keyboard event handling
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                let key = event.keystroke.key.as_str();
                logging::log("CONFIRM", &format!("Key pressed: {}", key));
                match key {
                    // Enter = submit current selection and close
                    "enter" | "Enter" => {
                        this.submit();
                        window.remove_window();
                    }
                    // Escape = cancel and close
                    "escape" | "Escape" => {
                        this.cancel();
                        window.remove_window();
                    }
                    // Tab = toggle focus between buttons
                    "tab" | "Tab" => {
                        this.toggle_focus(cx);
                    }
                    // Left arrow = focus cancel button
                    "left" | "arrowleft" | "Left" | "ArrowLeft" => {
                        this.focus_cancel(cx);
                    }
                    // Right arrow = focus confirm button
                    "right" | "arrowright" | "Right" | "ArrowRight" => {
                        this.focus_confirm(cx);
                    }
                    _ => {}
                }
            }))
            // Message
            .child(
                div()
                    .w_full()
                    .text_color(primary_text)
                    .text_base()
                    .text_center()
                    .child(message_str),
            )
            // Buttons
            .child(button_row)
    }
}
