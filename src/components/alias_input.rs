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
use crate::panel::PROMPT_INPUT_FIELD_HEIGHT;
use crate::theme::Theme;
use crate::transitions;
use gpui::{
    div, prelude::*, px, rgb, rgba, App, Context, FocusHandle, Focusable, IntoElement, Render,
    Window,
};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Constants for alias input styling
const MODAL_WIDTH: f32 = 420.0;
const MODAL_PADDING: f32 = 24.0;
const INPUT_PADDING: f32 = 12.0;
const BUTTON_GAP: f32 = 12.0;
const ALIAS_MAX_LENGTH: usize = 32;
const ALIAS_INPUT_PLACEHOLDER: &str = "Type a short alias, e.g. ch for Clipboard History";
const ALIAS_VALID_HELP_TEXT: &str = "Alias runs with <alias> + space in the main menu";
const OVERLAY_ANIMATION_DURATION_MS: u64 = 140;
const OVERLAY_MODAL_ENTRY_OFFSET_PX: f32 = 12.0;
const OVERLAY_MODAL_START_OPACITY: f32 = 0.82;

#[derive(Clone, Copy, Debug)]
struct OverlayAppearStyle {
    backdrop_opacity: f32,
    modal_opacity: f32,
    modal_offset_y: f32,
    complete: bool,
}

fn compute_overlay_appear_style(elapsed: Duration) -> OverlayAppearStyle {
    let progress =
        (elapsed.as_secs_f32() / (OVERLAY_ANIMATION_DURATION_MS as f32 / 1000.0)).clamp(0.0, 1.0);
    let eased = transitions::ease_out_quad(progress);
    let modal_opacity = OVERLAY_MODAL_START_OPACITY + ((1.0 - OVERLAY_MODAL_START_OPACITY) * eased);

    OverlayAppearStyle {
        backdrop_opacity: eased,
        modal_opacity,
        modal_offset_y: OVERLAY_MODAL_ENTRY_OFFSET_PX * (1.0 - eased),
        complete: progress >= 1.0,
    }
}

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
    /// Error text color for validation feedback
    pub text_error: u32,
}

impl AliasInputColors {
    /// Create colors from theme reference
    pub fn from_theme(theme: &Theme) -> Self {
        Self {
            overlay_bg: theme.colors.background.main,
            modal_bg: theme.colors.background.main,
            border: theme.colors.ui.border,
            text_primary: theme.colors.text.primary,
            text_secondary: theme.colors.text.secondary,
            text_muted: theme.colors.text.muted,
            accent: theme.colors.accent.selected,
            input_bg: theme.colors.background.search_box,
            input_border: theme.colors.ui.border,
            selection_bg: theme.colors.accent.selected,
            text_error: theme.colors.ui.error,
        }
    }
}

impl Default for AliasInputColors {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

/// Validation error for alias input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AliasValidationError {
    Empty,
    ContainsWhitespace,
    InvalidCharacters,
    TooLong { max_length: usize },
}

/// Validate and normalize alias input from the modal.
fn validate_alias_input(input: &str) -> Result<String, AliasValidationError> {
    let alias = input.trim();
    if alias.is_empty() {
        return Err(AliasValidationError::Empty);
    }

    if alias.chars().any(char::is_whitespace) {
        return Err(AliasValidationError::ContainsWhitespace);
    }

    if !alias
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(AliasValidationError::InvalidCharacters);
    }

    if alias.chars().count() > ALIAS_MAX_LENGTH {
        return Err(AliasValidationError::TooLong {
            max_length: ALIAS_MAX_LENGTH,
        });
    }

    Ok(alias.to_string())
}

fn is_command_modifier(platform: bool, control: bool) -> bool {
    platform || control
}

fn is_clear_alias_shortcut(key: &str, command_modifier: bool, has_current_alias: bool) -> bool {
    has_current_alias
        && command_modifier
        && (key.eq_ignore_ascii_case("backspace") || key.eq_ignore_ascii_case("delete"))
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
    /// Cursor visibility for blinking (controlled by parent's blink timer)
    pub cursor_visible: bool,
    /// Timestamp for enter animation start (fade/slide-in)
    overlay_animation_started_at: Instant,
    /// Ensures we schedule at most one animation tick task at a time
    overlay_animation_tick_scheduled: bool,
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
            cursor_visible: true,
            overlay_animation_started_at: Instant::now(),
            overlay_animation_tick_scheduled: false,
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
        if let Ok(alias) = validate_alias_input(self.input.text()) {
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

    /// Update cursor visibility (called from parent's blink timer)
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
    }

    fn overlay_appear_style(&self) -> OverlayAppearStyle {
        compute_overlay_appear_style(self.overlay_animation_started_at.elapsed())
    }

    fn schedule_overlay_animation_tick_if_needed(
        &mut self,
        animation_complete: bool,
        cx: &mut Context<Self>,
    ) {
        if animation_complete || self.overlay_animation_tick_scheduled {
            return;
        }

        self.overlay_animation_tick_scheduled = true;
        cx.spawn(async move |this, cx| {
            gpui::Timer::after(Duration::from_millis(16)).await;
            let _ = cx.update(|cx| {
                let _ = this.update(cx, |app, cx| {
                    app.overlay_animation_tick_scheduled = false;
                    cx.notify();
                });
            });
        })
        .detach();
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
                .child(ALIAS_INPUT_PLACEHOLDER)
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
                        // Cursor - conditionally visible for blinking
                        div()
                            .w(px(2.))
                            .h(px(18.))
                            .rounded(px(1.))
                            .when(self.cursor_visible, |d| d.bg(rgb(colors.accent))),
                    )
                    .child(after)
            }
        };

        div()
            .id("alias-input-field")
            .w_full()
            .h(px(PROMPT_INPUT_FIELD_HEIGHT))
            .px(px(INPUT_PADDING))
            .flex()
            .items_center()
            .bg(rgba((colors.input_bg << 8) | 0xFF))
            .border_1()
            .border_color(rgba((colors.input_border << 8) | 0x80))
            .rounded(px(8.))
            .child(display_text)
    }

    fn validation_feedback(&self) -> Result<&'static str, String> {
        match validate_alias_input(self.input.text()) {
            Ok(_) => Ok(ALIAS_VALID_HELP_TEXT),
            Err(AliasValidationError::Empty) => Err("Alias is required".to_string()),
            Err(AliasValidationError::ContainsWhitespace) => {
                Err("Alias cannot contain spaces".to_string())
            }
            Err(AliasValidationError::InvalidCharacters) => {
                Err("Use only letters, numbers, hyphens, or underscores".to_string())
            }
            Err(AliasValidationError::TooLong { max_length }) => {
                Err(format!("Alias must be {max_length} characters or fewer"))
            }
        }
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
        let validation_feedback = self.validation_feedback();
        let overlay_appear = self.overlay_appear_style();
        self.schedule_overlay_animation_tick_if_needed(overlay_appear.complete, cx);

        // Determine button states
        let can_save = validation_feedback.is_ok();
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

        let validation = match &validation_feedback {
            Ok(message) => div()
                .w_full()
                .mt(px(8.))
                .text_sm()
                .text_color(rgb(colors.text_muted))
                .child((*message).to_string()),
            Err(message) => div()
                .w_full()
                .mt(px(8.))
                .text_sm()
                .text_color(rgb(colors.text_error))
                .child(message.clone()),
        };

        // Key down event handler - captures all key events for text input
        let handle_key_down = cx.listener(move |this, event: &gpui::KeyDownEvent, _window, cx| {
            let key = event.keystroke.key.as_str();
            let mods = event.keystroke.modifiers;
            let key_char = event.keystroke.key_char.as_deref();
            let command_modifier = is_command_modifier(mods.platform, mods.control);

            logging::log_debug(
                "ALIAS_INPUT",
                &format!(
                    "KeyDown: key='{}' cmd={} ctrl={} alt={} shift={}",
                    key, mods.platform, mods.control, mods.alt, mods.shift
                ),
            );

            // Handle special keys first
            match key.to_lowercase().as_str() {
                "escape" | "esc" => {
                    this.cancel();
                    cx.notify();
                    return;
                }
                "enter" | "return" if !this.input.text().trim().is_empty() => {
                    this.save();
                    cx.notify();
                    return;
                }
                _ => {}
            }

            if is_clear_alias_shortcut(key, command_modifier, this.current_alias.is_some()) {
                this.clear_alias();
                cx.notify();
                return;
            }

            // Pass to text input handler for all other keys
            let handled =
                this.input
                    .handle_key(key, key_char, command_modifier, mods.alt, mods.shift, cx);

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
            .child(validation)
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
                    .opacity(overlay_appear.backdrop_opacity)
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
                    .mt(px(overlay_appear.modal_offset_y))
                    .opacity(overlay_appear.modal_opacity)
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
    fn test_alias_input_colors_from_theme_uses_theme_overlay_token() {
        let mut theme = Theme::default();
        theme.colors.background.main = 0x1a2b3c;

        let colors = AliasInputColors::from_theme(&theme);
        assert_eq!(colors.overlay_bg, 0x1a2b3c);
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

    #[test]
    fn test_alias_placeholder_copy_is_clear() {
        assert_eq!(
            ALIAS_INPUT_PLACEHOLDER,
            "Type a short alias, e.g. ch for Clipboard History"
        );
    }

    #[test]
    fn test_validate_alias_rejects_empty_and_whitespace() {
        assert!(matches!(
            validate_alias_input("   "),
            Err(AliasValidationError::Empty)
        ));
        assert!(matches!(
            validate_alias_input("two words"),
            Err(AliasValidationError::ContainsWhitespace)
        ));
    }

    #[test]
    fn test_validate_alias_accepts_trimmed_valid_input() {
        assert_eq!(
            validate_alias_input("  clip  ").expect("alias should be valid"),
            "clip"
        );
    }

    #[test]
    fn test_validate_alias_rejects_invalid_characters() {
        assert!(matches!(
            validate_alias_input("clip!"),
            Err(AliasValidationError::InvalidCharacters)
        ));
        assert!(matches!(
            validate_alias_input("clip.history"),
            Err(AliasValidationError::InvalidCharacters)
        ));
    }

    #[test]
    fn test_alias_command_modifier_uses_platform_or_control() {
        assert!(is_command_modifier(true, false));
        assert!(is_command_modifier(false, true));
        assert!(!is_command_modifier(false, false));
    }

    #[test]
    fn test_alias_clear_shortcut_requires_modifier_and_existing_alias() {
        assert!(is_clear_alias_shortcut("backspace", true, true));
        assert!(is_clear_alias_shortcut("delete", true, true));
        assert!(!is_clear_alias_shortcut("backspace", false, true));
        assert!(!is_clear_alias_shortcut("backspace", true, false));
    }

    #[test]
    fn test_compute_overlay_appear_style_starts_hidden_offset_and_transparent() {
        let style = compute_overlay_appear_style(Duration::from_millis(0));
        assert_eq!(style.backdrop_opacity, 0.0);
        assert!(style.modal_offset_y > 0.0);
        assert!(style.modal_opacity < 1.0);
        assert!(!style.complete);
    }

    #[test]
    fn test_compute_overlay_appear_style_reaches_full_visibility_after_duration() {
        let style =
            compute_overlay_appear_style(Duration::from_millis(OVERLAY_ANIMATION_DURATION_MS));
        assert!((style.backdrop_opacity - 1.0).abs() < 0.001);
        assert!((style.modal_offset_y - 0.0).abs() < 0.001);
        assert!((style.modal_opacity - 1.0).abs() < 0.001);
        assert!(style.complete);
    }
}
