//! Confirm Dialog
//!
//! A simple confirmation dialog with a message and two buttons (Cancel/Confirm).
//! Supports keyboard shortcuts: Enter = confirm, Escape = cancel.
//! Tab/Arrow keys navigate between buttons with visual focus indication.

use crate::logging;
use crate::theme;
use gpui::{
    div, prelude::*, px, rgb, rgba, App, Context, FocusHandle, Focusable, FontWeight, Render,
    SharedString, Window,
};
use std::sync::Arc;

use super::constants::{BUTTON_GAP, CONFIRM_PADDING, CONFIRM_WIDTH, DIALOG_RADIUS};

/// Callback for confirm/cancel selection
/// Signature: (confirmed: bool)
pub type ConfirmCallback = Arc<dyn Fn(bool) + Send + Sync>;

/// Helper function to combine a hex color with an alpha value
#[inline]
fn hex_with_alpha(hex: u32, alpha: u8) -> u32 {
    (hex << 8) | (alpha as u32)
}

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
        // Log render with focus state for debugging
        logging::log(
            "CONFIRM",
            &format!(
                "ConfirmDialog::render() called, focused_button={}",
                self.focused_button
            ),
        );

        // Get theme colors
        let colors = &self.theme.colors;

        // Background with vibrancy support
        // Use the theme's opacity setting to match the main window's vibrancy look
        let use_vibrancy = self.theme.is_vibrancy_enabled();
        let dialog_alpha = if use_vibrancy {
            // Use theme's opacity setting (typically 0.30-0.37) for consistent vibrancy
            // This allows the blur to show through the dialog
            let opacity = self
                .theme
                .opacity
                .as_ref()
                .map(|o| o.main)
                .unwrap_or(0.37)
                .clamp(0.25, 0.50);
            (opacity * 255.0) as u8
        } else {
            // Near-opaque when vibrancy disabled
            (0.95 * 255.0) as u8
        };
        let main_bg = rgba(hex_with_alpha(colors.background.main, dialog_alpha));

        // Text colors
        let primary_text = rgb(colors.text.primary);

        // Border color for dialog
        let border_color = rgba(hex_with_alpha(colors.ui.border, 0x60));

        let message_str: SharedString = self.message.clone().into();

        let is_cancel_focused = self.focused_button == 0;
        let is_confirm_focused = self.focused_button == 1;

        // Button styling with OBVIOUS focus indication
        // Focused button gets accent background + bright border (like macOS default button)
        // Unfocused button is subtle/transparent
        let accent_hex = colors.accent.selected;
        let accent_color = rgb(accent_hex);

        // Focused: accent background at 40% opacity (very visible), white text
        let focused_bg = rgba(hex_with_alpha(accent_hex, 0x66)); // 40% accent
        let focused_text = rgb(0xFFFFFF); // White text on accent

        // Unfocused: barely visible, accent text
        let unfocused_bg = rgba(0xffffff10); // 6% white
        let hover_bg = rgba(0xffffff20); // 12% white on hover

        // Focus ring: bright accent border, unfocused gets subtle border
        let focus_border = rgba(hex_with_alpha(accent_hex, 0xFF)); // 100% accent
        let unfocused_border = rgba(0xffffff30); // 19% white

        // Cancel button
        let cancel_button = div()
            .id("cancel-btn")
            .flex_1()
            .h(px(44.0))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(8.0))
            .cursor_pointer()
            .font_weight(FontWeight::MEDIUM)
            // Focused: accent bg + white text, Unfocused: transparent + accent text
            .bg(if is_cancel_focused {
                focused_bg
            } else {
                unfocused_bg
            })
            .text_color(if is_cancel_focused {
                focused_text
            } else {
                accent_color
            })
            .border_2() // Thick border for visibility
            .border_color(if is_cancel_focused {
                focus_border
            } else {
                unfocused_border
            })
            .hover(|style| style.bg(hover_bg))
            .on_click(cx.listener(|this, _e, _window, _cx| {
                this.cancel();
            }))
            .child(self.cancel_text.clone());

        // Confirm button - same styling pattern
        let confirm_button = div()
            .id("confirm-btn")
            .flex_1()
            .h(px(44.0))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(8.0))
            .cursor_pointer()
            .font_weight(FontWeight::MEDIUM)
            // Focused: accent bg + white text, Unfocused: transparent + accent text
            .bg(if is_confirm_focused {
                focused_bg
            } else {
                unfocused_bg
            })
            .text_color(if is_confirm_focused {
                focused_text
            } else {
                accent_color
            })
            .border_2() // Thick border for visibility
            .border_color(if is_confirm_focused {
                focus_border
            } else {
                unfocused_border
            })
            .hover(|style| style.bg(hover_bg))
            .on_click(cx.listener(|this, _e, _window, _cx| {
                this.confirm();
            }))
            .child(self.confirm_text.clone());

        // Button row
        let button_row = div()
            .w_full()
            .flex()
            .flex_row()
            .gap(px(BUTTON_GAP))
            .child(cancel_button)
            .child(confirm_button);

        // Main dialog container
        // NOTE: No background - let window vibrancy show through
        let _ = main_bg; // Suppress unused warning
        div()
            .w(px(CONFIRM_WIDTH))
            .flex()
            .flex_col()
            .p(px(CONFIRM_PADDING))
            .gap(px(CONFIRM_PADDING))
            // No .bg() - vibrancy comes from the window
            .rounded(px(DIALOG_RADIUS))
            .border_1()
            .border_color(border_color)
            .overflow_hidden()
            // NOTE: Key handling is done by ConfirmWindow, not here
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
