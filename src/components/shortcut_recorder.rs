//! Shortcut Recorder Component
//!
//! A modal overlay for recording keyboard shortcuts. Captures key combinations
//! and displays them using macOS-style symbols (⌘⇧K format).
//!
//! ## Features
//! - Captures modifier keys (Cmd, Ctrl, Alt, Shift) + a single key
//! - Displays shortcuts visually using symbols
//! - Shows conflict warnings when shortcuts are already assigned
//! - Clear, Cancel, and Save buttons
//!
//! ## Usage
//! ```rust,ignore
//! let recorder = ShortcutRecorder::new(focus_handle, theme)
//!     .with_command_name("My Script")
//!     .with_command_description("Does something useful")
//!     .on_save(|shortcut| { /* handle save */ })
//!     .on_cancel(|| { /* handle cancel */ });
//! ```

#![allow(dead_code)]

use crate::components::button::{Button, ButtonColors, ButtonVariant};
use crate::logging;
use crate::theme::Theme;
use gpui::{
    div, prelude::*, px, rgb, rgba, App, Context, FocusHandle, Focusable, IntoElement, Render,
    Window,
};
use std::sync::Arc;

/// Constants for shortcut recorder styling
const MODAL_WIDTH: f32 = 420.0;
const MODAL_PADDING: f32 = 24.0;
const KEY_DISPLAY_HEIGHT: f32 = 64.0;
const KEY_DISPLAY_PADDING: f32 = 16.0;
const KEYCAP_SIZE: f32 = 44.0;
const KEYCAP_GAP: f32 = 8.0;
const BUTTON_GAP: f32 = 12.0;

/// Pre-computed colors for ShortcutRecorder rendering
#[derive(Clone, Copy, Debug)]
pub struct ShortcutRecorderColors {
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
    /// Muted text color (for hints)
    pub text_muted: u32,
    /// Accent color for highlights
    pub accent: u32,
    /// Warning color for conflicts
    pub warning: u32,
    /// Key display area background
    pub key_display_bg: u32,
    /// Keycap background color
    pub keycap_bg: u32,
    /// Keycap border color
    pub keycap_border: u32,
}

impl ShortcutRecorderColors {
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
            warning: theme.colors.ui.warning,
            key_display_bg: theme.colors.background.search_box,
            keycap_bg: theme.colors.background.title_bar,
            keycap_border: theme.colors.ui.border,
        }
    }
}

impl Default for ShortcutRecorderColors {
    fn default() -> Self {
        Self {
            overlay_bg: 0x000000,
            modal_bg: 0x1e1e1e,
            border: 0x464647,
            text_primary: 0xffffff,
            text_secondary: 0xcccccc,
            text_muted: 0x808080,
            accent: 0xfbbf24,
            warning: 0xf59e0b,
            key_display_bg: 0x3c3c3c,
            keycap_bg: 0x2d2d30,
            keycap_border: 0x464647,
        }
    }
}

/// Represents a recorded keyboard shortcut
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RecordedShortcut {
    /// Command key (macOS) / Super key
    pub cmd: bool,
    /// Control key
    pub ctrl: bool,
    /// Option/Alt key
    pub alt: bool,
    /// Shift key
    pub shift: bool,
    /// The actual key pressed (single character or key name)
    pub key: Option<String>,
}

impl RecordedShortcut {
    /// Create a new empty shortcut
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the shortcut has any content
    pub fn is_empty(&self) -> bool {
        !self.cmd && !self.ctrl && !self.alt && !self.shift && self.key.is_none()
    }

    /// Check if only modifiers are set (no key yet)
    pub fn has_only_modifiers(&self) -> bool {
        (self.cmd || self.ctrl || self.alt || self.shift) && self.key.is_none()
    }

    /// Check if the shortcut is complete (has modifiers + key)
    pub fn is_complete(&self) -> bool {
        (self.cmd || self.ctrl || self.alt || self.shift) && self.key.is_some()
    }

    /// Format as a display string using macOS symbols
    pub fn to_display_string(&self) -> String {
        let mut parts = Vec::new();

        if self.ctrl {
            parts.push("⌃");
        }
        if self.alt {
            parts.push("⌥");
        }
        if self.shift {
            parts.push("⇧");
        }
        if self.cmd {
            parts.push("⌘");
        }

        if let Some(ref key) = self.key {
            parts.push(key.as_str());
        }

        parts.join("")
    }

    /// Format as a config string (e.g., "cmd+shift+k")
    pub fn to_config_string(&self) -> String {
        let mut parts = Vec::new();

        if self.ctrl {
            parts.push("ctrl".to_string());
        }
        if self.alt {
            parts.push("alt".to_string());
        }
        if self.shift {
            parts.push("shift".to_string());
        }
        if self.cmd {
            parts.push("cmd".to_string());
        }

        if let Some(ref key) = self.key {
            parts.push(key.to_lowercase());
        }

        parts.join("+")
    }

    /// Get individual keycaps for display
    pub fn to_keycaps(&self) -> Vec<String> {
        let mut keycaps = Vec::new();

        if self.ctrl {
            keycaps.push("⌃".to_string());
        }
        if self.alt {
            keycaps.push("⌥".to_string());
        }
        if self.shift {
            keycaps.push("⇧".to_string());
        }
        if self.cmd {
            keycaps.push("⌘".to_string());
        }

        if let Some(ref key) = self.key {
            keycaps.push(Self::format_key_display(key));
        }

        keycaps
    }

    /// Format a key for display (uppercase letters, special key names)
    fn format_key_display(key: &str) -> String {
        match key.to_lowercase().as_str() {
            "enter" | "return" => "↵".to_string(),
            "escape" | "esc" => "⎋".to_string(),
            "tab" => "⇥".to_string(),
            "backspace" | "delete" => "⌫".to_string(),
            "space" => "␣".to_string(),
            "up" | "arrowup" => "↑".to_string(),
            "down" | "arrowdown" => "↓".to_string(),
            "left" | "arrowleft" => "←".to_string(),
            "right" | "arrowright" => "→".to_string(),
            _ => key.to_uppercase(),
        }
    }
}

/// Conflict information for a shortcut
#[derive(Clone, Debug)]
pub struct ShortcutConflict {
    /// Name of the command that has this shortcut
    pub command_name: String,
    /// The conflicting shortcut string
    pub shortcut: String,
}

/// Callback types for shortcut recorder
pub type OnSaveCallback = Box<dyn Fn(RecordedShortcut) + 'static>;
pub type OnCancelCallback = Box<dyn Fn() + 'static>;
pub type ConflictChecker = Box<dyn Fn(&RecordedShortcut) -> Option<ShortcutConflict> + 'static>;

/// Actions that can be triggered by the recorder
#[derive(Clone, Debug, PartialEq)]
pub enum RecorderAction {
    /// User wants to save the shortcut
    Save(RecordedShortcut),
    /// User wants to cancel
    Cancel,
}

/// Shortcut Recorder Modal Component
///
/// A modal dialog for recording keyboard shortcuts with visual feedback.
pub struct ShortcutRecorder {
    /// Focus handle for keyboard input
    pub focus_handle: FocusHandle,
    /// Theme for styling
    pub theme: Arc<Theme>,
    /// Pre-computed colors
    pub colors: ShortcutRecorderColors,
    /// Name of the command being configured
    pub command_name: Option<String>,
    /// Description of the command
    pub command_description: Option<String>,
    /// Currently recorded shortcut (final result with key)
    pub shortcut: RecordedShortcut,
    /// Currently held modifiers (for live display before final key)
    pub current_modifiers: gpui::Modifiers,
    /// Current conflict if any
    pub conflict: Option<ShortcutConflict>,
    /// Callback when save is pressed
    pub on_save: Option<OnSaveCallback>,
    /// Callback when cancel is pressed
    pub on_cancel: Option<OnCancelCallback>,
    /// Function to check for conflicts
    pub conflict_checker: Option<ConflictChecker>,
    /// Whether recording is active (listening for keys)
    pub is_recording: bool,
    /// Pending action for the parent to handle (polled after render)
    pub pending_action: Option<RecorderAction>,
}

impl ShortcutRecorder {
    /// Create a new shortcut recorder
    /// The focus_handle MUST be created from the entity's own context (cx.focus_handle())
    /// for keyboard events to work properly.
    pub fn new(cx: &mut Context<Self>, theme: Arc<Theme>) -> Self {
        let colors = ShortcutRecorderColors::from_theme(&theme);
        // Create focus handle from THIS entity's context - critical for keyboard events
        let focus_handle = cx.focus_handle();
        logging::log("SHORTCUT", "Created ShortcutRecorder with new focus handle");
        Self {
            focus_handle,
            theme,
            colors,
            command_name: None,
            command_description: None,
            shortcut: RecordedShortcut::new(),
            current_modifiers: gpui::Modifiers::default(),
            conflict: None,
            on_save: None,
            on_cancel: None,
            conflict_checker: None,
            is_recording: true,
            pending_action: None,
        }
    }

    /// Set the command name
    pub fn with_command_name(mut self, name: impl Into<String>) -> Self {
        self.command_name = Some(name.into());
        self
    }

    /// Set the command description
    pub fn with_command_description(mut self, description: impl Into<String>) -> Self {
        self.command_description = Some(description.into());
        self
    }

    /// Set the save callback
    pub fn on_save(mut self, callback: impl Fn(RecordedShortcut) + 'static) -> Self {
        self.on_save = Some(Box::new(callback));
        self
    }

    /// Set the cancel callback
    pub fn on_cancel(mut self, callback: impl Fn() + 'static) -> Self {
        self.on_cancel = Some(Box::new(callback));
        self
    }

    /// Set the conflict checker
    pub fn with_conflict_checker(
        mut self,
        checker: impl Fn(&RecordedShortcut) -> Option<ShortcutConflict> + 'static,
    ) -> Self {
        self.conflict_checker = Some(Box::new(checker));
        self
    }

    /// Set command name (mutable version)
    pub fn set_command_name(&mut self, name: Option<String>) {
        self.command_name = name;
    }

    /// Set command description (mutable version)
    pub fn set_command_description(&mut self, description: Option<String>) {
        self.command_description = description;
    }

    /// Clear the recorded shortcut
    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.shortcut = RecordedShortcut::new();
        self.conflict = None;
        self.is_recording = true;
        logging::log("SHORTCUT", "Shortcut cleared");
        cx.notify();
    }

    /// Handle save button press
    pub fn save(&mut self) {
        if self.shortcut.is_complete() && self.conflict.is_none() {
            logging::log(
                "SHORTCUT",
                &format!("Saving shortcut: {}", self.shortcut.to_config_string()),
            );
            // Set pending action for parent to poll
            self.pending_action = Some(RecorderAction::Save(self.shortcut.clone()));
            // Also call legacy callback if set
            if let Some(ref callback) = self.on_save {
                callback(self.shortcut.clone());
            }
        }
    }

    /// Handle cancel button press
    pub fn cancel(&mut self) {
        logging::log("SHORTCUT", "Shortcut recording cancelled");
        // Set pending action for parent to poll
        self.pending_action = Some(RecorderAction::Cancel);
        // Also call legacy callback if set
        if let Some(ref callback) = self.on_cancel {
            callback();
        }
    }

    /// Take the pending action (returns it and clears the field)
    pub fn take_pending_action(&mut self) -> Option<RecorderAction> {
        self.pending_action.take()
    }

    /// Handle a key down event
    pub fn handle_key_down(
        &mut self,
        key: &str,
        modifiers: gpui::Modifiers,
        cx: &mut Context<Self>,
    ) {
        if !self.is_recording {
            return;
        }

        // ALWAYS update current_modifiers for live display
        // This provides feedback even if on_modifiers_changed doesn't fire
        self.current_modifiers = modifiers;

        // Update shortcut modifiers
        self.shortcut.cmd = modifiers.platform;
        self.shortcut.ctrl = modifiers.control;
        self.shortcut.alt = modifiers.alt;
        self.shortcut.shift = modifiers.shift;

        // Check if this is a modifier-only key press
        let is_modifier_key = matches!(
            key.to_lowercase().as_str(),
            "shift"
                | "control"
                | "alt"
                | "meta"
                | "command"
                | "cmd"
                | "super"
                | "win"
                | "ctrl"
                | "opt"
                | "option"
        );

        if !is_modifier_key && !key.is_empty() {
            // Got a real key, record it
            self.shortcut.key = Some(key.to_uppercase());
            self.is_recording = false;

            logging::log(
                "SHORTCUT",
                &format!(
                    "Recorded shortcut: {} (config: {})",
                    self.shortcut.to_display_string(),
                    self.shortcut.to_config_string()
                ),
            );

            // Check for conflicts
            self.check_conflict();
        } else if is_modifier_key {
            // For modifier-only keypresses, log that we're showing live feedback
            logging::log(
                "SHORTCUT",
                &format!(
                    "Modifier key pressed (live feedback): key='{}' cmd={} ctrl={} alt={} shift={}",
                    key, modifiers.platform, modifiers.control, modifiers.alt, modifiers.shift
                ),
            );
        }

        cx.notify();
    }

    /// Handle escape key
    pub fn handle_escape(&mut self, cx: &mut Context<Self>) {
        if self.shortcut.is_empty() {
            // If nothing recorded, cancel
            self.cancel();
        } else {
            // Otherwise, clear the recording
            self.clear(cx);
        }
    }

    /// Check for shortcut conflicts
    fn check_conflict(&mut self) {
        if let Some(ref checker) = self.conflict_checker {
            self.conflict = checker(&self.shortcut);
            if let Some(ref conflict) = self.conflict {
                logging::log(
                    "SHORTCUT",
                    &format!(
                        "Conflict detected with '{}' (shortcut: {})",
                        conflict.command_name, conflict.shortcut
                    ),
                );
            }
        }
    }

    /// Update theme
    pub fn update_theme(&mut self, theme: Arc<Theme>) {
        self.colors = ShortcutRecorderColors::from_theme(&theme);
        self.theme = theme;
    }

    /// Render a single keycap
    fn render_keycap(&self, key: &str) -> impl IntoElement {
        let colors = self.colors;
        div()
            .w(px(KEYCAP_SIZE))
            .h(px(KEYCAP_SIZE))
            .flex()
            .items_center()
            .justify_center()
            .bg(rgba((colors.keycap_bg << 8) | 0xFF))
            .border_1()
            .border_color(rgba((colors.keycap_border << 8) | 0x80))
            .rounded(px(8.))
            .text_xl()
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(rgb(colors.text_primary))
            .child(key.to_string())
    }

    /// Get keycaps for live display - shows current modifiers while recording,
    /// or the final recorded shortcut when complete
    fn get_display_keycaps(&self) -> Vec<String> {
        if self.shortcut.is_complete() {
            // Show the final recorded shortcut
            self.shortcut.to_keycaps()
        } else if self.is_recording {
            // Show currently held modifiers (live feedback)
            let mut keycaps = Vec::new();
            if self.current_modifiers.control {
                keycaps.push("⌃".to_string());
            }
            if self.current_modifiers.alt {
                keycaps.push("⌥".to_string());
            }
            if self.current_modifiers.shift {
                keycaps.push("⇧".to_string());
            }
            if self.current_modifiers.platform {
                keycaps.push("⌘".to_string());
            }
            keycaps
        } else {
            // Recording complete but no final key - show what we have
            self.shortcut.to_keycaps()
        }
    }

    /// Render the key display area
    fn render_key_display(&self) -> impl IntoElement {
        let colors = self.colors;
        let keycaps = self.get_display_keycaps();

        let mut key_row = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .gap(px(KEYCAP_GAP));

        if keycaps.is_empty() {
            // Show placeholder when nothing is pressed
            key_row = key_row.child(
                div()
                    .text_base()
                    .text_color(rgb(colors.text_muted))
                    .child("Press any key combination..."),
            );
        } else {
            // Show keycaps (either live modifiers or recorded shortcut)
            for keycap in keycaps {
                key_row = key_row.child(self.render_keycap(&keycap));
            }
        }

        div()
            .w_full()
            .h(px(KEY_DISPLAY_HEIGHT))
            .px(px(KEY_DISPLAY_PADDING))
            .flex()
            .items_center()
            .justify_center()
            .bg(rgba((colors.key_display_bg << 8) | 0x60))
            .rounded(px(8.))
            .border_1()
            .border_color(rgba((colors.border << 8) | 0x40))
            .child(key_row)
    }

    /// Render conflict warning if present
    fn render_conflict_warning(&self) -> impl IntoElement {
        let colors = self.colors;

        if let Some(ref conflict) = self.conflict {
            div()
                .w_full()
                .mt(px(12.))
                .px(px(12.))
                .py(px(8.))
                .bg(rgba((colors.warning << 8) | 0x20))
                .border_1()
                .border_color(rgba((colors.warning << 8) | 0x40))
                .rounded(px(6.))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                .child(div().text_sm().text_color(rgb(colors.warning)).child("⚠"))
                .child(
                    div()
                        .flex_1()
                        .text_sm()
                        .text_color(rgb(colors.text_secondary))
                        .child(format!("Already used by \"{}\"", conflict.command_name)),
                )
                .into_any_element()
        } else {
            div().into_any_element()
        }
    }
}

impl Focusable for ShortcutRecorder {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ShortcutRecorder {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.colors;
        let button_colors = ButtonColors::from_theme(&self.theme);

        // Determine button states
        let can_save = self.shortcut.is_complete() && self.conflict.is_none();
        let can_clear = !self.shortcut.is_empty();

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
                    .child("Record Keyboard Shortcut"),
            )
            .when_some(self.command_name.clone(), |d, name| {
                d.child(
                    div()
                        .text_base()
                        .text_color(rgb(colors.text_secondary))
                        .child(format!("For: {}", name)),
                )
            })
            .when_some(self.command_description.clone(), |d, desc| {
                d.child(
                    div()
                        .text_sm()
                        .text_color(rgb(colors.text_muted))
                        .child(desc),
                )
            });

        // Build button row
        let clear_handler = cx.listener(|this, _: &gpui::ClickEvent, _window, cx| {
            this.clear(cx);
        });

        let cancel_handler = cx.listener(|this, _: &gpui::ClickEvent, _window, _cx| {
            this.cancel();
        });

        let save_handler = cx.listener(|this, _: &gpui::ClickEvent, _window, _cx| {
            this.save();
        });

        let buttons = div()
            .w_full()
            .mt(px(16.))
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .child(
                // Left side: Clear button
                Button::new("Clear", button_colors)
                    .variant(ButtonVariant::Ghost)
                    .disabled(!can_clear)
                    .on_click(Box::new(move |event, window, cx| {
                        clear_handler(event, window, cx);
                    })),
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

        // Instructions
        let instructions = div()
            .w_full()
            .mt(px(12.))
            .text_xs()
            .text_color(rgb(colors.text_muted))
            .text_center()
            .child("Press a modifier (⌘⌃⌥⇧) + a key");

        // Key down event handler - captures modifiers and keys
        let handle_key_down = cx.listener(move |this, event: &gpui::KeyDownEvent, _window, cx| {
            let key = event.keystroke.key.as_str();
            let mods = event.keystroke.modifiers;

            logging::log(
                "SHORTCUT",
                &format!(
                    "KeyDown: key='{}' cmd={} ctrl={} alt={} shift={}",
                    key, mods.platform, mods.control, mods.alt, mods.shift
                ),
            );

            // Handle special keys
            match key.to_lowercase().as_str() {
                "escape" | "esc" => {
                    this.handle_escape(cx);
                }
                "enter" | "return" if this.shortcut.is_complete() && this.conflict.is_none() => {
                    this.save();
                    cx.notify();
                }
                _ => {
                    this.handle_key_down(key, mods, cx);
                }
            }
        });

        // Modifiers changed handler - CRITICAL for live modifier feedback
        // This fires whenever ANY modifier key is pressed or released (e.g., pressing Cmd alone)
        let handle_modifiers_changed = cx.listener(
            move |this, event: &gpui::ModifiersChangedEvent, _window, cx| {
                // Only update if we're still recording (haven't captured a complete shortcut yet)
                if this.is_recording {
                    logging::log(
                        "SHORTCUT",
                        &format!(
                            "ModifiersChanged: cmd={} ctrl={} alt={} shift={}",
                            event.modifiers.platform,
                            event.modifiers.control,
                            event.modifiers.alt,
                            event.modifiers.shift
                        ),
                    );
                    // Update current modifiers for live display
                    this.current_modifiers = event.modifiers;
                    cx.notify(); // Trigger re-render to show keycaps
                }
            },
        );

        // Cancel handler for backdrop clicks
        let backdrop_cancel = cx.listener(|this, _: &gpui::ClickEvent, _window, _cx| {
            logging::log("SHORTCUT", "Backdrop clicked - cancelling");
            this.cancel();
        });

        // Modal content - with stop propagation to prevent backdrop dismiss
        let modal = div()
            .id("shortcut-modal-content")
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
            .child(self.render_key_display())
            .child(self.render_conflict_warning())
            .child(instructions)
            .child(buttons);

        // Full-screen overlay with backdrop and centered modal
        // The overlay captures ALL keyboard and modifier events while open
        div()
            .id("shortcut-recorder-overlay")
            .absolute()
            .inset_0()
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key_down)
            .on_modifiers_changed(handle_modifiers_changed) // CRITICAL: Live modifier feedback
            // Backdrop layer - semi-transparent, captures clicks to dismiss
            .child(
                div()
                    .id("shortcut-backdrop")
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
    fn test_recorded_shortcut_to_display_string() {
        let mut shortcut = RecordedShortcut::new();
        shortcut.cmd = true;
        shortcut.shift = true;
        shortcut.key = Some("K".to_string());

        assert_eq!(shortcut.to_display_string(), "⇧⌘K");
    }

    #[test]
    fn test_recorded_shortcut_to_config_string() {
        let mut shortcut = RecordedShortcut::new();
        shortcut.cmd = true;
        shortcut.shift = true;
        shortcut.key = Some("K".to_string());

        assert_eq!(shortcut.to_config_string(), "shift+cmd+k");
    }

    #[test]
    fn test_recorded_shortcut_is_empty() {
        let shortcut = RecordedShortcut::new();
        assert!(shortcut.is_empty());

        let mut shortcut_with_mod = RecordedShortcut::new();
        shortcut_with_mod.cmd = true;
        assert!(!shortcut_with_mod.is_empty());
    }

    #[test]
    fn test_recorded_shortcut_is_complete() {
        let mut shortcut = RecordedShortcut::new();
        shortcut.cmd = true;
        assert!(!shortcut.is_complete()); // No key yet

        shortcut.key = Some("K".to_string());
        assert!(shortcut.is_complete()); // Has modifier + key
    }

    #[test]
    fn test_recorded_shortcut_to_keycaps() {
        let mut shortcut = RecordedShortcut::new();
        shortcut.ctrl = true;
        shortcut.alt = true;
        shortcut.shift = true;
        shortcut.cmd = true;
        shortcut.key = Some("K".to_string());

        let keycaps = shortcut.to_keycaps();
        assert_eq!(keycaps, vec!["⌃", "⌥", "⇧", "⌘", "K"]);
    }

    #[test]
    fn test_format_key_display_special_keys() {
        assert_eq!(RecordedShortcut::format_key_display("enter"), "↵");
        assert_eq!(RecordedShortcut::format_key_display("escape"), "⎋");
        assert_eq!(RecordedShortcut::format_key_display("tab"), "⇥");
        assert_eq!(RecordedShortcut::format_key_display("backspace"), "⌫");
        assert_eq!(RecordedShortcut::format_key_display("space"), "␣");
        assert_eq!(RecordedShortcut::format_key_display("up"), "↑");
        assert_eq!(RecordedShortcut::format_key_display("arrowdown"), "↓");
    }

    #[test]
    fn test_shortcut_recorder_colors_default() {
        let colors = ShortcutRecorderColors::default();
        assert_eq!(colors.accent, 0xfbbf24);
        assert_eq!(colors.warning, 0xf59e0b);
    }
}
