//! EnvPrompt - Environment variable prompt with keyring storage
//!
//! Features:
//! - Prompt for environment variable values
//! - Secure storage via system keyring (keychain on macOS)
//! - Mask input for secret values
//! - Remember values for future sessions
//!
//! Design: Matches ArgPrompt-no-choices (single input line, minimal height)

use gpui::{
    div, prelude::*, px, rgb, rgba, svg, Context, FocusHandle, Focusable, Render, SharedString,
    Window,
};
use std::sync::Arc;

use crate::designs::{get_tokens, DesignVariant};
use crate::logging;
use crate::panel::{CURSOR_GAP_X, CURSOR_HEIGHT_LG, CURSOR_MARGIN_Y, CURSOR_WIDTH};
use crate::theme;

use super::SubmitCallback;

/// Service name for keyring storage
const KEYRING_SERVICE: &str = "com.scriptkit.env";

/// Get a secret from the system keyring
pub fn get_secret(key: &str) -> Option<String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, key);
    match entry {
        Ok(entry) => match entry.get_password() {
            Ok(value) => {
                logging::log("KEYRING", &format!("Retrieved secret for key: {}", key));
                Some(value)
            }
            Err(keyring::Error::NoEntry) => {
                logging::log("KEYRING", &format!("No entry found for key: {}", key));
                None
            }
            Err(e) => {
                logging::log(
                    "KEYRING",
                    &format!("Error retrieving secret for key {}: {}", key, e),
                );
                None
            }
        },
        Err(e) => {
            logging::log(
                "KEYRING",
                &format!("Error creating keyring entry for key {}: {}", key, e),
            );
            None
        }
    }
}

/// Set a secret in the system keyring
pub fn set_secret(key: &str, value: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, key)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    entry
        .set_password(value)
        .map_err(|e| format!("Failed to store secret: {}", e))?;

    logging::log("KEYRING", &format!("Stored secret for key: {}", key));
    Ok(())
}

/// Delete a secret from the system keyring
#[allow(dead_code)]
pub fn delete_secret(key: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, key)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    entry
        .delete_credential()
        .map_err(|e| format!("Failed to delete secret: {}", e))?;

    logging::log("KEYRING", &format!("Deleted secret for key: {}", key));
    Ok(())
}

/// EnvPrompt - Environment variable prompt with secure storage
///
/// Prompts for environment variable values and stores them securely
/// using the system keyring. Useful for API keys, tokens, and secrets.
pub struct EnvPrompt {
    /// Unique ID for this prompt instance
    pub id: String,
    /// Environment variable key name
    pub key: String,
    /// Custom prompt text (defaults to "Enter value for {key}")
    pub prompt: Option<String>,
    /// Whether to mask input (for secrets)
    pub secret: bool,
    /// Current input value
    pub input_text: String,
    /// Focus handle for keyboard input
    pub focus_handle: FocusHandle,
    /// Callback when user submits a value
    pub on_submit: SubmitCallback,
    /// Theme for styling
    pub theme: Arc<theme::Theme>,
    /// Design variant for styling
    pub design_variant: DesignVariant,
    /// Whether we checked the keyring already
    checked_keyring: bool,
}

impl EnvPrompt {
    pub fn new(
        id: String,
        key: String,
        prompt: Option<String>,
        secret: bool,
        focus_handle: FocusHandle,
        on_submit: SubmitCallback,
        theme: Arc<theme::Theme>,
    ) -> Self {
        logging::log(
            "PROMPTS",
            &format!("EnvPrompt::new for key: {} (secret: {})", key, secret),
        );

        EnvPrompt {
            id,
            key,
            prompt,
            secret,
            input_text: String::new(),
            focus_handle,
            on_submit,
            theme,
            design_variant: DesignVariant::Default,
            checked_keyring: false,
        }
    }

    /// Check keyring and auto-submit if value exists
    /// Returns true if value was found and submitted
    pub fn check_keyring_and_auto_submit(&mut self) -> bool {
        if self.checked_keyring {
            return false;
        }
        self.checked_keyring = true;

        if let Some(value) = get_secret(&self.key) {
            logging::log(
                "PROMPTS",
                &format!("Found existing value in keyring for key: {}", self.key),
            );
            // Auto-submit the stored value
            (self.on_submit)(self.id.clone(), Some(value));
            return true;
        }
        false
    }

    /// Submit the entered value
    fn submit(&mut self) {
        if !self.input_text.is_empty() {
            // Store in keyring if this is a secret
            if self.secret {
                if let Err(e) = set_secret(&self.key, &self.input_text) {
                    logging::log("ERROR", &format!("Failed to store secret: {}", e));
                }
            }
            (self.on_submit)(self.id.clone(), Some(self.input_text.clone()));
        }
    }

    /// Set the input text programmatically
    pub fn set_input(&mut self, text: String, cx: &mut Context<Self>) {
        if self.input_text == text {
            return;
        }

        self.input_text = text;
        cx.notify();
    }

    /// Cancel - submit None
    fn submit_cancel(&mut self) {
        (self.on_submit)(self.id.clone(), None);
    }

    /// Handle character input
    fn handle_char(&mut self, ch: char, cx: &mut Context<Self>) {
        self.input_text.push(ch);
        cx.notify();
    }

    /// Handle backspace
    fn handle_backspace(&mut self, cx: &mut Context<Self>) {
        if !self.input_text.is_empty() {
            self.input_text.pop();
            cx.notify();
        }
    }

    /// Get display text (masked if secret)
    fn display_text(&self) -> String {
        if self.secret && !self.input_text.is_empty() {
            "•".repeat(self.input_text.len())
        } else {
            self.input_text.clone()
        }
    }
}

impl Focusable for EnvPrompt {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for EnvPrompt {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tokens = get_tokens(self.design_variant);
        let design_colors = tokens.colors();
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();

        let handle_key = cx.listener(
            |this: &mut Self,
             event: &gpui::KeyDownEvent,
             _window: &mut Window,
             cx: &mut Context<Self>| {
                let key_str = event.keystroke.key.to_lowercase();

                match key_str.as_str() {
                    "enter" => this.submit(),
                    "escape" => this.submit_cancel(),
                    "backspace" => this.handle_backspace(cx),
                    _ => {
                        if let Some(ref key_char) = event.keystroke.key_char {
                            if let Some(ch) = key_char.chars().next() {
                                if !ch.is_control() {
                                    this.handle_char(ch, cx);
                                }
                            }
                        }
                    }
                }
            },
        );

        // Use design tokens for consistent styling (matches ArgPrompt)
        let text_primary = design_colors.text_primary;
        let text_muted = design_colors.text_muted;
        let text_dimmed = design_colors.text_dimmed;
        let accent_color = design_colors.accent;

        // Build placeholder text: "Enter {KEY}" or custom prompt, with lock icon for secrets
        let placeholder: SharedString = self
            .prompt
            .clone()
            .map(|p| {
                if self.secret {
                    format!("🔒 {}", p)
                } else {
                    p
                }
            })
            .unwrap_or_else(|| {
                if self.secret {
                    format!("🔒 Enter {}", self.key)
                } else {
                    format!("Enter {}", self.key)
                }
            })
            .into();

        let display_text = self.display_text();
        let input_is_empty = display_text.is_empty();

        let input_display: SharedString = if input_is_empty {
            placeholder
        } else {
            display_text.into()
        };

        // Cursor visibility (always visible for now, can add blink timer later)
        let cursor_visible = true;

        // Main container - matches ArgPrompt-no-choices layout exactly
        // Single row with: input area + Submit button + logo
        div()
            .id(gpui::ElementId::Name("window:env".into()))
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .text_color(rgb(text_primary))
            .font_family(design_typography.font_family)
            .key_context("env_prompt")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            // Single header row (matches ArgPrompt exactly)
            .child(
                div()
                    .w_full()
                    .px(px(design_spacing.padding_lg))
                    .py(px(design_spacing.padding_md))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    // Input area with cursor (same pattern as ArgPrompt)
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_row()
                            .items_center()
                            .text_lg()
                            .text_color(if input_is_empty {
                                rgb(text_muted)
                            } else {
                                rgb(text_primary)
                            })
                            // When empty: cursor LEFT (before placeholder)
                            .when(input_is_empty, |d| {
                                d.child(
                                    div()
                                        .w(px(CURSOR_WIDTH))
                                        .h(px(CURSOR_HEIGHT_LG))
                                        .my(px(CURSOR_MARGIN_Y))
                                        .mr(px(CURSOR_GAP_X))
                                        .when(cursor_visible, |d| d.bg(rgb(text_primary))),
                                )
                            })
                            // Display text - with negative margin for placeholder alignment
                            .when(input_is_empty, |d| {
                                d.child(
                                    div()
                                        .ml(px(-(CURSOR_WIDTH + CURSOR_GAP_X)))
                                        .child(input_display.clone()),
                                )
                            })
                            .when(!input_is_empty, |d| d.child(input_display.clone()))
                            // When typing: cursor RIGHT (after text)
                            .when(!input_is_empty, |d| {
                                d.child(
                                    div()
                                        .w(px(CURSOR_WIDTH))
                                        .h(px(CURSOR_HEIGHT_LG))
                                        .my(px(CURSOR_MARGIN_Y))
                                        .ml(px(CURSOR_GAP_X))
                                        .when(cursor_visible, |d| d.bg(rgb(text_primary))),
                                )
                            }),
                    )
                    // Submit button area (matches ArgPrompt style)
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .child(
                                div()
                                    .text_color(rgb(accent_color))
                                    .text_sm()
                                    .child("Submit"),
                            )
                            .child(
                                div()
                                    .ml(px(4.))
                                    .px(px(4.))
                                    .py(px(2.))
                                    .rounded(px(4.))
                                    .bg(rgba((text_dimmed << 8) | 0x30))
                                    .text_color(rgb(text_muted))
                                    .text_xs()
                                    .child("↵"),
                            )
                            .child(
                                div()
                                    .mx(px(4.))
                                    .text_color(rgba((text_dimmed << 8) | 0x60))
                                    .text_sm()
                                    .child("|"),
                            ),
                    )
                    // Script Kit logo
                    .child(
                        svg()
                            .path(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logo.svg"))
                            .size(px(16.))
                            .text_color(rgb(accent_color)),
                    ),
            )
    }
}
