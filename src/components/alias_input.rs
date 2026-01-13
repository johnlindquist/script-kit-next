//! Alias Input Component
//!
//! A modal overlay for entering command aliases with proper keyboard focus handling.
//! Follows the ShortcutRecorder pattern for GPUI entity + Focusable trait.
//!
//! ## Features
//! - Text input with full keyboard support (typing, backspace, arrows, etc.)
//! - Selection support (shift+arrows, cmd+a)
//! - Clipboard operations (cmd+c, cmd+v, cmd+x)
//! - Cancel with Escape, Save with Enter
//!
//! ## Usage
//! ```rust,ignore
//! let alias_input = cx.new(|cx| {
//!     AliasInput::new(cx, theme)
//!         .with_command_name("My Script")
//!         .with_command_id("my-script-id")
//! });
//! ```

#![allow(dead_code)]

use crate::components::button::{Button, ButtonColors, ButtonVariant};
use crate::components::text_input::TextInputState;
use crate::logging;
use crate::theme::Theme;
use gpui::{
    div, prelude::*, px, rgb, rgba, App, Context, FocusHandle, Focusable, IntoElement, Render,
    Window,
};
use std::sync::Arc;

/// Constants for alias input styling
const MODAL_WIDTH: f32 = 420.0;
const MODAL_PADDING: f32 = 24.0;
const INPUT_HEIGHT: f32 = 44.0;
const INPUT_PADDING: f32 = 12.0;
const BUTTON_GAP: f32 = 12.0;

/// Pre-computed colors for AliasInput rendering
#[derive(Clone, Copy, Debug)]
pub struct AliasInputColors {
    /// Background color for the modal overlay
    pub overlay_bg: u32,
    /// Background color for the modal itself
    pub modal_bg: u32,
    /// Border color for the modal
    pub border: u32,
    /// Primary text color
    pub text_primary: u32,
    /// Secondary text color (for descriptions)
    pub text_secondary: u32,
    /// Muted text color (for placeholders)
    pub text_muted: u32,
    /// Accent color for highlights
    pub accent: u32,
    /// Input field background
    pub input_bg: u32,
    /// Input field border
    pub input_border: u32,
    /// Selection highlight color
    pub selection_bg: u32,
}

impl AliasInputColors {
    /// Create colors from theme reference
    pub fn from_theme(theme: &Theme) -> Self {
        Self {
            overlay_bg: 0x000000,
            modal_bg: theme.colors.background.main,
            border: theme.colors.ui.border,
            text_primary: theme.colors.text.primary,
            text_secondary: theme.colors.text.secondary,
            text_muted: theme.colors.text.muted,
            accent: theme.colors.accent.selected,
            input_bg: theme.colors.background.search_box,
            input_border: theme.colors.ui.border,
            selection_bg: theme.colors.accent.selected,
        }
    }
}

impl Default for AliasInputColors {
    fn default() -> Self {
        Self {
            overlay_bg: 0x000000,
            modal_bg: 0x1e1e1e,
            border: 0x464647,
            text_primary: 0xffffff,
            text_secondary: 0xcccccc,
            text_muted: 0x808080,
            accent: 0xfbbf24,
            input_bg: 0x3c3c3c,
            input_border: 0x464647,
            selection_bg: 0x3b82f6,
        }
    }
}

/// Actions that can be triggered by the alias input
#[derive(Clone, Debug, PartialEq)]
pub enum AliasInputAction {
    /// User wants to save the alias
    Save(String),
    /// User wants to cancel
    Cancel,
    /// User wants to clear the current alias
    Clear,
}

/// Alias Input Modal Component
///
/// A modal dialog for entering command aliases with full keyboard support.
pub struct AliasInput {
    /// Focus handle for keyboard input - CRITICAL for keyboard events
    pub focus_handle: FocusHandle,
    /// Theme for styling
    pub theme: Arc<Theme>,
    /// Pre-computed colors
    pub colors: AliasInputColors,
    /// Name of the command being configured
    pub command_name: String,
    /// ID of the command being configured
    pub command_id: String,
    /// Text input state (handles selection, cursor, etc.)
    pub input: TextInputState,
    /// Current alias (if editing an existing one)
    pub current_alias: Option<String>,
    /// Pending action for the parent to handle (polled after render)
    pub pending_action: Option<AliasInputAction>,
}

impl AliasInput {
    /// Create a new alias input
    /// The focus_handle is created from the entity's own context (cx.focus_handle())
    /// for keyboard events to work properly.
    pub fn new(cx: &mut Context<Self>, theme: Arc<Theme>) -> Self {
        let colors = AliasInputColors::from_theme(&theme);
        // Create focus handle from THIS entity's context - critical for keyboard events
        let focus_handle = cx.focus_handle();
        logging::log("ALIAS_INPUT", "Created AliasInput with new focus handle");
        Self {
            focus_handle,
            theme,
            colors,
            command_name: String::new(),
            command_id: String::new(),
            input: TextInputState::new(),
            current_alias: None,
            pending_action: None,
        }
    }

    /// Set the command name
    pub fn with_command_name(mut self, name: impl Into<String>) -> Self {
        self.command_name = name.into();
        self
    }

    /// Set the command ID
    pub fn with_command_id(mut self, id: impl Into<String>) -> Self {
        self.command_id = id.into();
        self
    }

    /// Set the current alias (for editing)
    pub fn with_current_alias(mut self, alias: Option<String>) -> Self {
        if let Some(ref a) = alias {
            self.input.set_text(a.clone());
        }
        self.current_alias = alias;
        self
    }

    /// Set command name (mutable version)
    pub fn set_command_name(&mut self, name: impl Into<String>) {
        self.command_name = name.into();
    }

    /// Set command ID (mutable version)
    pub fn set_command_id(&mut self, id: impl Into<String>) {
        self.command_id = id.into();
    }

    /// Set current alias (mutable version)
    pub fn set_current_alias(&mut self, alias: Option<String>) {
        if let Some(ref a) = alias {
            self.input.set_text(a.clone());
        }
        self.current_alias = alias;
    }

    /// Get the current input text
    pub fn text(&self) -> &str {
        self.input.text()
    }

    /// Clear the input
    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.input.clear();
        logging::log("ALIAS_INPUT", "Input cleared");
        cx.notify();
    }

    /// Handle save action
    pub fn save(&mut self) {
        let alias = self.input.text().trim().to_string();
        if !alias.is_empty() {
            logging::log("ALIAS_INPUT", &format!("Saving alias: {}", alias));
            self.pending_action = Some(AliasInputAction::Save(alias));
        }
    }

    /// Handle cancel action
    pub fn cancel(&mut self) {
        logging::log("ALIAS_INPUT", "Alias input cancelled");
        self.pending_action = Some(AliasInputAction::Cancel);
    }

    /// Handle clear action (remove existing alias)
    pub fn clear_alias(&mut self) {
        logging::log("ALIAS_INPUT", "Clearing existing alias");
        self.pending_action = Some(AliasInputAction::Clear);
    }

    /// Take the pending action (returns it and clears the field)
    pub fn take_pending_action(&mut self) -> Option<AliasInputAction> {
        self.pending_action.take()
    }

    /// Update theme
    pub fn update_theme(&mut self, theme: Arc<Theme>) {
        self.colors = AliasInputColors::from_theme(&theme);
        self.theme = theme;
    }

    /// Render the text input field with cursor and selection
    fn render_input_field(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.colors;
        let text = self.input.text();
        let cursor_pos = self.input.cursor();
        let selection = self.input.selection();

        // Build the input content with cursor indicator
        let display_text = if text.is_empty() {
            // Placeholder
            div()
                .text_base()
                .text_color(rgb(colors.text_muted))
                .child("Enter alias (e.g., 'ch' for Clipboard History)")
        } else {
            // Show text with cursor position indicator
            let (before, after) = {
                let chars: Vec<char> = text.chars().collect();
                let before: String = chars[..cursor_pos.min(chars.len())].iter().collect();
                let after: String = chars[cursor_pos.min(chars.len())..].iter().collect();
                (before, after)
            };

            // Determine if we have a selection
            let has_selection = !selection.is_empty();
            let (sel_start, sel_end) = selection.range();

            if has_selection {
                // Render with selection highlight
                let chars: Vec<char> = text.chars().collect();
                let before_sel: String = chars[..sel_start].iter().collect();
                let selected: String = chars[sel_start..sel_end].iter().collect();
                let after_sel: String = chars[sel_end..].iter().collect();

                div()
                    .flex()
                    .flex_row()
                    .text_base()
                    .text_color(rgb(colors.text_primary))
                    .child(before_sel)
                    .child(
                        div()
                            .bg(rgba((colors.selection_bg << 8) | 0x80))
                            .rounded(px(2.))
                            .child(selected),
                    )
                    .child(after_sel)
            } else {
                // Render with cursor indicator
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .text_base()
                    .text_color(rgb(colors.text_primary))
                    .child(before)
                    .child(
                        // Cursor
                        div()
                            .w(px(2.))
                            .h(px(18.))
                            .bg(rgb(colors.accent))
                            .rounded(px(1.)),
                    )
                    .child(after)
            }
        };

        div()
            .id("alias-input-field")
            .w_full()
            .h(px(INPUT_HEIGHT))
            .px(px(INPUT_PADDING))
            .flex()
            .items_center()
            .bg(rgba((colors.input_bg << 8) | 0xFF))
            .border_1()
            .border_color(rgba((colors.input_border << 8) | 0x80))
            .rounded(px(8.))
            .child(display_text)
    }
}

impl Focusable for AliasInput {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for AliasInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.colors;
        let button_colors = ButtonColors::from_theme(&self.theme);

        // Determine button states
        let can_save = !self.input.text().trim().is_empty();
        let can_clear = self.current_alias.is_some();

        // Build header with command info
        let header = div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(4.))
            .child(
                div()
                    .text_lg()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(colors.text_primary))
                    .child(format!("Set Alias for \"{}\"", self.command_name)),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(colors.text_muted))
                    .child("Type the alias + space in the main menu to run this command"),
            );

        // Build button row
        let clear_handler = cx.listener(|this, _: &gpui::ClickEvent, _window, cx| {
            this.clear_alias();
            cx.notify();
        });

        let cancel_handler = cx.listener(|this, _: &gpui::ClickEvent, _window, cx| {
            this.cancel();
            cx.notify();
        });

        let save_handler = cx.listener(|this, _: &gpui::ClickEvent, _window, cx| {
            this.save();
            cx.notify();
        });

        let buttons = div()
            .w_full()
            .mt(px(16.))
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .child(
                // Left side: Clear button (only if editing existing alias)
                div().when(can_clear, |d| {
                    d.child(
                        Button::new("Clear", button_colors)
                            .variant(ButtonVariant::Ghost)
                            .on_click(Box::new(move |event, window, cx| {
                                clear_handler(event, window, cx);
                            })),
                    )
                }),
            )
            .child(
                // Right side: Cancel and Save
                div()
                    .flex()
                    .flex_row()
                    .gap(px(BUTTON_GAP))
                    .child(
                        Button::new("Cancel", button_colors)
                            .variant(ButtonVariant::Ghost)
                            .shortcut("Esc")
                            .on_click(Box::new(move |event, window, cx| {
                                cancel_handler(event, window, cx);
                            })),
                    )
                    .child(
                        Button::new("Save", button_colors)
                            .variant(ButtonVariant::Primary)
                            .shortcut("↵")
                            .disabled(!can_save)
                            .on_click(Box::new(move |event, window, cx| {
                                save_handler(event, window, cx);
                            })),
                    ),
            );

        // Key down event handler - captures all key events for text input
        let handle_key_down = cx.listener(move |this, event: &gpui::KeyDownEvent, _window, cx| {
            let key = event.keystroke.key.as_str();
            let mods = event.keystroke.modifiers;
            let key_char = event.keystroke.key_char.as_deref();

            logging::log(
                "ALIAS_INPUT",
                &format!(
                    "KeyDown: key='{}' cmd={} ctrl={} alt={} shift={}",
                    key, mods.platform, mods.control, mods.alt, mods.shift
                ),
            );

            // Handle special keys first
            match key.to_lowercase().as_str() {
                "escape" => {
                    this.cancel();
                    cx.notify();
                    return;
                }
                "enter" if !this.input.text().trim().is_empty() => {
                    this.save();
                    cx.notify();
                    return;
                }
                _ => {}
            }

            // Pass to text input handler for all other keys
            let handled = this.input.handle_key(
                key,
                key_char,
                mods.platform, // cmd
                mods.alt,
                mods.shift,
                cx,
            );

            if handled {
                cx.notify();
            }
        });

        // Cancel handler for backdrop clicks
        let backdrop_cancel = cx.listener(|this, _: &gpui::ClickEvent, _window, cx| {
            logging::log("ALIAS_INPUT", "Backdrop clicked - cancelling");
            this.cancel();
            cx.notify();
        });

        // Modal content - with stop propagation to prevent backdrop dismiss
        let modal = div()
            .id("alias-input-modal-content")
            .w(px(MODAL_WIDTH))
            .p(px(MODAL_PADDING))
            .bg(rgba((colors.modal_bg << 8) | 0xF0))
            .border_1()
            .border_color(rgba((colors.border << 8) | 0x80))
            .rounded(px(12.))
            .flex()
            .flex_col()
            // Stop propagation - clicks inside modal shouldn't dismiss it
            .on_mouse_down(gpui::MouseButton::Left, |_, _, _| {
                // Empty handler stops propagation to backdrop
            })
            .child(header)
            .child(div().h(px(16.))) // Spacer
            .child(self.render_input_field(cx))
            .child(buttons);

        // Full-screen overlay with backdrop and centered modal
        // The overlay captures ALL keyboard events while open
        div()
            .id("alias-input-overlay")
            .absolute()
            .inset_0()
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key_down)
            // Backdrop layer - semi-transparent, captures clicks to dismiss
            .child(
                div()
                    .id("alias-input-backdrop")
                    .absolute()
                    .inset_0()
                    .bg(rgba((colors.overlay_bg << 8) | 0x80)) // 50% opacity
                    .on_click(backdrop_cancel),
            )
            // Modal container - centered on top of backdrop
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(modal),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alias_input_colors_default() {
        let colors = AliasInputColors::default();
        assert_eq!(colors.accent, 0xfbbf24);
        assert_eq!(colors.text_primary, 0xffffff);
    }

    #[test]
    fn test_alias_input_action_variants() {
        let save_action = AliasInputAction::Save("test".to_string());
        let cancel_action = AliasInputAction::Cancel;
        let clear_action = AliasInputAction::Clear;

        assert!(matches!(save_action, AliasInputAction::Save(_)));
        assert!(matches!(cancel_action, AliasInputAction::Cancel));
        assert!(matches!(clear_action, AliasInputAction::Clear));
    }
}
