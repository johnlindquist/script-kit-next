//! EnvPrompt - Environment variable prompt with encrypted storage
//!
//! Features:
//! - Prompt for environment variable values
//! - Secure storage via age-encrypted secrets (see crate::secrets)
//! - Mask input for secret values
//! - Remember values for future sessions
//! - Full text selection and clipboard support (cmd+c/v/x, shift+arrows)
//!
//! Design: Full-window centered input with clear visual hierarchy

use chrono::{DateTime, Utc};
use gpui::{
    div, prelude::*, px, rgb, rgba, svg, Context, Div, FocusHandle, Focusable, Render,
    SharedString, Window,
};
use std::sync::Arc;

use crate::components::prompt_footer::{PromptFooter, PromptFooterColors, PromptFooterConfig};
use crate::components::TextInputState;
use crate::designs::icon_variations::IconName;
use crate::designs::{get_tokens, DesignVariant};
use crate::logging;
use crate::panel::{CURSOR_HEIGHT_LG, CURSOR_WIDTH, PROMPT_INPUT_FIELD_HEIGHT};
use crate::secrets;
use crate::theme;
use crate::ui_foundation::{is_key_enter, is_key_escape};

use super::SubmitCallback;

/// Format a DateTime as relative time (e.g., "2 hours ago", "3 days ago")
fn format_relative_time(dt: DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now.signed_duration_since(dt);

    let seconds = diff.num_seconds();
    if seconds < 0 {
        return "just now".to_string();
    }
    let seconds = seconds as u64;

    const MINUTE: u64 = 60;
    const HOUR: u64 = MINUTE * 60;
    const DAY: u64 = HOUR * 24;
    const WEEK: u64 = DAY * 7;
    const MONTH: u64 = DAY * 30;
    const YEAR: u64 = DAY * 365;

    if seconds < MINUTE {
        "just now".to_string()
    } else if seconds < HOUR {
        let mins = seconds / MINUTE;
        format!("{} min{} ago", mins, if mins == 1 { "" } else { "s" })
    } else if seconds < DAY {
        let hours = seconds / HOUR;
        format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
    } else if seconds < WEEK {
        let days = seconds / DAY;
        format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
    } else if seconds < MONTH {
        let weeks = seconds / WEEK;
        format!("{} week{} ago", weeks, if weeks == 1 { "" } else { "s" })
    } else if seconds < YEAR {
        let months = seconds / MONTH;
        format!("{} month{} ago", months, if months == 1 { "" } else { "s" })
    } else {
        let years = seconds / YEAR;
        format!("{} year{} ago", years, if years == 1 { "" } else { "s" })
    }
}

fn env_input_placeholder(key: &str, exists_in_keyring: bool) -> String {
    if exists_in_keyring {
        format!("Paste a replacement value for {key}")
    } else {
        format!("Paste value for {key}")
    }
}

fn env_default_description(key: &str, exists_in_keyring: bool) -> String {
    if exists_in_keyring {
        format!("Update the saved value for {key}")
    } else {
        format!("Enter the value for {key}")
    }
}

fn env_running_status(key: &str) -> String {
    format!("Script is running and waiting for {key}")
}

fn env_input_label(secret: bool) -> &'static str {
    if secret {
        "Secret value"
    } else {
        "Value"
    }
}

fn masked_secret_value_for_display(value: &str) -> String {
    "•".repeat(value.chars().count())
}

fn env_storage_hint_text(secret: bool) -> &'static str {
    if secret {
        "Stored securely in ~/.scriptkit/secrets.age"
    } else {
        "Value is provided to the script for this run only"
    }
}

fn env_submit_validation_error(value: &str) -> Option<&'static str> {
    if value.trim().is_empty() {
        Some("Value cannot be empty")
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EnvKeyAction {
    Submit,
    Cancel,
}

#[inline]
fn env_key_action(key: &str) -> Option<EnvKeyAction> {
    if is_key_enter(key) {
        return Some(EnvKeyAction::Submit);
    }
    if is_key_escape(key) {
        return Some(EnvKeyAction::Cancel);
    }
    None
}

fn env_prompt_correlation_id(id: &str, key: &str) -> String {
    format!("env_prompt:{id}:{key}")
}

/// EnvPrompt - Environment variable prompt with secure storage
///
/// Prompts for environment variable values and stores them securely
/// in the local age-encrypted secrets file. Useful for API keys, tokens, and secrets.
pub struct EnvPrompt {
    /// Unique ID for this prompt instance
    pub id: String,
    /// Environment variable key name
    pub key: String,
    /// Custom prompt text (defaults to "Enter value for {key}")
    pub prompt: Option<String>,
    /// Optional title (e.g., provider name like "Vercel AI Gateway")
    pub title: Option<String>,
    /// Whether to mask input (for secrets)
    pub secret: bool,
    /// Text input state with selection and clipboard support
    input: TextInputState,
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
    /// Whether a value already exists in keyring (for UX messaging)
    pub exists_in_keyring: bool,
    /// When the secret was last modified (if exists)
    pub modified_at: Option<DateTime<Utc>>,
    /// Inline validation/persistence error shown to the user
    validation_error: Option<String>,
}

impl EnvPrompt {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        key: String,
        prompt: Option<String>,
        title: Option<String>,
        secret: bool,
        focus_handle: FocusHandle,
        on_submit: SubmitCallback,
        theme: Arc<theme::Theme>,
        exists_in_keyring: bool,
        modified_at: Option<DateTime<Utc>>,
    ) -> Self {
        let correlation_id = env_prompt_correlation_id(&id, &key);
        logging::log(
            "PROMPTS",
            &format!(
                "correlation_id={correlation_id} EnvPrompt::new key={key} secret={secret} exists={exists_in_keyring} title={title:?} modified={modified_at:?}",
            ),
        );

        EnvPrompt {
            id,
            key,
            prompt,
            title,
            secret,
            input: TextInputState::new(),
            focus_handle,
            on_submit,
            theme,
            design_variant: DesignVariant::Default,
            checked_keyring: false,
            exists_in_keyring,
            modified_at,
            validation_error: None,
        }
    }

    fn correlation_id(&self) -> String {
        env_prompt_correlation_id(&self.id, &self.key)
    }

    /// Check keyring and auto-submit if value exists
    /// Returns true if value was found and submitted
    pub fn check_keyring_and_auto_submit(&mut self) -> bool {
        if self.checked_keyring {
            return false;
        }
        self.checked_keyring = true;

        if let Some(value) = secrets::get_secret(&self.key) {
            let correlation_id = self.correlation_id();
            logging::log(
                "PROMPTS",
                &format!(
                    "correlation_id={correlation_id} EnvPrompt auto-submit existing secret key={}",
                    self.key
                ),
            );
            // Auto-submit the stored value
            (self.on_submit)(self.id.clone(), Some(value));
            return true;
        }
        false
    }

    /// Submit the entered value
    fn submit(&mut self) {
        let text = self.input.text();
        if let Some(validation_error) = env_submit_validation_error(text) {
            self.validation_error = Some(validation_error.to_string());
            logging::log(
                "PROMPTS",
                &format!(
                    "correlation_id={} EnvPrompt submit blocked key={} reason={}",
                    self.correlation_id(),
                    self.key,
                    validation_error
                ),
            );
            return;
        }

        // Persist in encrypted storage only when this prompt is secret-mode.
        if self.secret {
            if let Err(e) = secrets::set_secret(&self.key, text) {
                self.validation_error =
                    Some("Failed to store secret. Check logs and try again.".to_string());
                logging::log(
                    "ERROR",
                    &format!(
                        "correlation_id={} EnvPrompt failed to store secret key={} error={}",
                        self.correlation_id(),
                        self.key,
                        e
                    ),
                );
                return;
            }
        }

        self.validation_error = None;
        (self.on_submit)(self.id.clone(), Some(text.to_string()));
    }

    /// Set the input text programmatically
    pub fn set_input(&mut self, text: String, cx: &mut Context<Self>) {
        if self.input.text() == text {
            return;
        }

        self.input.set_text(text);
        self.validation_error = None;
        cx.notify();
    }

    /// Cancel - submit None
    fn submit_cancel(&mut self) {
        self.validation_error = None;
        (self.on_submit)(self.id.clone(), None);
    }

    /// Delete the secret and close the prompt
    fn submit_delete(&mut self) {
        let correlation_id = self.correlation_id();
        logging::log(
            "PROMPTS",
            &format!(
                "correlation_id={correlation_id} EnvPrompt deleting secret key={}",
                self.key
            ),
        );

        // Delete from keyring
        if let Err(e) = secrets::delete_secret(&self.key) {
            self.validation_error =
                Some("Failed to delete stored value. Check logs and try again.".to_string());
            logging::log(
                "ERROR",
                &format!(
                    "correlation_id={correlation_id} EnvPrompt failed to delete secret key={} error={}",
                    self.key, e
                ),
            );
            return;
        }

        self.validation_error = None;
        // Call callback with None (same as cancel, but secret is now deleted)
        (self.on_submit)(self.id.clone(), None);
    }

    /// Get display text (masked if secret)
    fn display_text(&self) -> String {
        self.input.display_text(self.secret)
    }

    fn render_text_with_cursor_and_selection(
        &self,
        text: &str,
        text_primary: u32,
        accent_color: u32,
    ) -> Div {
        let chars: Vec<char> = text.chars().collect();
        let text_len = chars.len();
        let cursor_pos = self.input.cursor().min(text_len);
        let has_selection = self.input.has_selection();

        if text.is_empty() {
            return div().flex().flex_row().items_center().child(
                div()
                    .w(px(CURSOR_WIDTH))
                    .h(px(CURSOR_HEIGHT_LG))
                    .bg(rgb(text_primary)),
            );
        }

        if has_selection {
            let selection = self.input.selection();
            let (start, end) = selection.range();
            let start = start.min(text_len);
            let end = end.min(text_len);
            let (start, end) = if start <= end {
                (start, end)
            } else {
                (end, start)
            };

            let before: String = chars[..start].iter().collect();
            let selected: String = chars[start..end].iter().collect();
            let after: String = chars[end..].iter().collect();

            return div()
                .flex()
                .flex_row()
                .items_center()
                .overflow_x_hidden()
                .when(!before.is_empty(), |d: Div| d.child(div().child(before)))
                .child(
                    div()
                        .bg(rgba((accent_color << 8) | 0x60))
                        .text_color(rgb(text_primary))
                        .child(selected),
                )
                .when(!after.is_empty(), |d: Div| d.child(div().child(after)));
        }

        let before: String = chars[..cursor_pos].iter().collect();
        let after: String = chars[cursor_pos..].iter().collect();

        div()
            .flex()
            .flex_row()
            .items_center()
            .overflow_x_hidden()
            .when(!before.is_empty(), |d: Div| d.child(div().child(before)))
            .child(
                div()
                    .w(px(CURSOR_WIDTH))
                    .h(px(CURSOR_HEIGHT_LG))
                    .bg(rgb(text_primary)),
            )
            .when(!after.is_empty(), |d: Div| d.child(div().child(after)))
    }

    /// Render the text input with cursor and selection
    fn render_input_text(&self, text_primary: u32, accent_color: u32) -> Div {
        if self.secret {
            let masked = masked_secret_value_for_display(self.input.text());
            self.render_text_with_cursor_and_selection(&masked, text_primary, accent_color)
        } else {
            let text = self.display_text();
            self.render_text_with_cursor_and_selection(&text, text_primary, accent_color)
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
        let design_typography = tokens.typography();

        let handle_key = cx.listener(
            |this: &mut Self,
             event: &gpui::KeyDownEvent,
             _window: &mut Window,
             cx: &mut Context<Self>| {
                let key = event.keystroke.key.as_str();
                let modifiers = &event.keystroke.modifiers;

                // Handle submit/cancel first
                match env_key_action(key) {
                    Some(EnvKeyAction::Submit) => {
                        this.submit();
                        return;
                    }
                    Some(EnvKeyAction::Cancel) => {
                        this.submit_cancel();
                        return;
                    }
                    None => {}
                }

                // Delegate all other keys to TextInputState
                let key_char = event.keystroke.key_char.as_deref();
                let previous_text = this.input.text().to_string();
                let handled = this.input.handle_key(
                    key,
                    key_char,
                    modifiers.platform, // On macOS, platform = Cmd key
                    modifiers.alt,
                    modifiers.shift,
                    cx,
                );

                if handled {
                    if this.validation_error.is_some() && previous_text != this.input.text() {
                        this.validation_error = None;
                    }
                    cx.notify();
                }
            },
        );

        // Use design tokens for consistent styling
        let text_primary = design_colors.text_primary;
        let text_muted = design_colors.text_muted;
        let accent_color = design_colors.accent;
        let bg_surface = design_colors.background_secondary;
        let success_color = design_colors.success;
        let error_color = design_colors.error;

        // Build placeholder text for input
        let input_placeholder: SharedString =
            env_input_placeholder(&self.key, self.exists_in_keyring).into();

        // Build description text
        let description: SharedString = self
            .prompt
            .clone()
            .unwrap_or_else(|| env_default_description(&self.key, self.exists_in_keyring))
            .into();

        let input_is_empty = self.input.is_empty();
        let running_status = env_running_status(&self.key);

        // Full-window centered layout for API key input
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
            // Main content area - centered vertically
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .px(px(32.))
                    .gap(px(24.))
                    // Large key icon at top
                    .child(
                        div()
                            .size(px(64.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(px(16.))
                            .bg(rgba(accent_color << 8 | 0x20)) // Accent with low alpha
                            .child(
                                svg()
                                    .external_path(if self.secret {
                                        IconName::EyeOff.external_path()
                                    } else {
                                        IconName::Settings.external_path()
                                    })
                                    .size(px(32.))
                                    .text_color(rgb(accent_color)),
                            ),
                    )
                    // Title - provider name or key name
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap(px(8.))
                            .child(
                                div()
                                    .text_2xl()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(rgb(text_primary))
                                    .child(self.title.clone().unwrap_or_else(|| self.key.clone())),
                            )
                            // Description
                            .child(
                                div()
                                    .text_base()
                                    .text_color(rgb(text_muted))
                                    .text_center()
                                    .child(description),
                            ),
                    )
                    // Input field with clearer label and focus treatment
                    .child(
                        div()
                            .w_full()
                            .max_w(px(400.))
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(rgb(text_muted))
                                    .child(env_input_label(self.secret)),
                            )
                            .child(
                                div()
                                    .min_h(px(PROMPT_INPUT_FIELD_HEIGHT))
                                    .px(px(16.))
                                    .py(px(12.))
                                    .rounded(px(12.))
                                    .bg(rgb(bg_surface))
                                    .border_1()
                                    .border_color(rgba(accent_color << 8 | 0x80))
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(12.))
                                    // Lock icon inside input
                                    .child(
                                        svg()
                                            .external_path(if self.secret {
                                                IconName::EyeOff.external_path()
                                            } else {
                                                IconName::Settings.external_path()
                                            })
                                            .size(px(18.))
                                            .text_color(rgb(text_muted))
                                            .flex_shrink_0(),
                                    )
                                    // Input text area
                                    .child({
                                        div()
                                            .flex_1()
                                            .overflow_hidden()
                                            .text_lg()
                                            .text_color(if input_is_empty {
                                                rgb(text_muted)
                                            } else {
                                                rgb(text_primary)
                                            })
                                            // When empty: show cursor + placeholder
                                            .when(input_is_empty, |d: Div| {
                                                d.child(
                                                    div()
                                                        .flex()
                                                        .flex_row()
                                                        .items_center()
                                                        .child(
                                                            div()
                                                                .w(px(CURSOR_WIDTH))
                                                                .h(px(CURSOR_HEIGHT_LG))
                                                                .bg(rgb(accent_color)),
                                                        )
                                                        .child(
                                                            div()
                                                                .ml(px(4.))
                                                                .text_color(rgb(text_muted))
                                                                .child(input_placeholder.clone()),
                                                        ),
                                                )
                                            })
                                            // When has text: show masked dots or text with cursor
                                            .when(!input_is_empty, |d: Div| {
                                                d.child(
                                                    self.render_input_text(
                                                        text_primary,
                                                        accent_color,
                                                    ),
                                                )
                                            })
                                    }),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(text_muted))
                                    .child(env_storage_hint_text(self.secret)),
                            ),
                    )
                    .when_some(self.validation_error.clone(), |d: Div, error| {
                        d.child(
                            div()
                                .max_w(px(400.))
                                .w_full()
                                .text_xs()
                                .text_color(rgb(error_color))
                                .child(error),
                        )
                    })
                    // Running state indicator to clarify why this prompt is visible
                    .child(
                        div()
                            .max_w(px(400.))
                            .w_full()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(8.))
                            .child(div().size(px(8.)).rounded_full().bg(rgb(accent_color)))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(text_muted))
                                    .child(running_status),
                            ),
                    )
                    // Status hint - show when key exists with modification date and delete option
                    .when(self.exists_in_keyring, |d: Div| {
                        let modified_text = self
                            .modified_at
                            .map(format_relative_time)
                            .unwrap_or_else(|| "previously".to_string());

                        // Create delete click handler
                        let handle_delete = cx.entity().downgrade();

                        d.child(
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap(px(8.))
                                // Status line with checkmark and modification time
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap(px(6.))
                                        .child(
                                            svg()
                                                .external_path(IconName::Check.external_path())
                                                .size(px(14.))
                                                .text_color(rgb(success_color)),
                                        )
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(rgb(text_muted))
                                                .child(format!("Configured {}", modified_text)),
                                        )
                                        // Separator dot
                                        .child(
                                            div().text_sm().text_color(rgb(text_muted)).child("·"),
                                        )
                                        // Delete link
                                        .child(
                                            div()
                                                .id("delete-secret")
                                                .text_sm()
                                                .text_color(rgb(error_color))
                                                .cursor_pointer()
                                                .hover(|s| s.opacity(0.8))
                                                .on_click(move |_event, _window, cx| {
                                                    if let Some(entity) = handle_delete.upgrade() {
                                                        entity.update(cx, |this, _cx| {
                                                            this.submit_delete();
                                                        });
                                                    }
                                                })
                                                .child("Delete"),
                                        ),
                                ),
                        )
                    }),
            )
            // Footer with submit action
            .child({
                let footer_colors = PromptFooterColors::from_theme(&self.theme);
                let primary_label = if self.exists_in_keyring {
                    "Update & Continue"
                } else {
                    "Save & Continue"
                };
                let footer_config = PromptFooterConfig::new()
                    .primary_label(primary_label)
                    .primary_shortcut("↵")
                    .helper_text("Script running")
                    .show_secondary(true)
                    .secondary_label("Cancel")
                    .secondary_shortcut("Esc");

                // Add click handlers
                let handle = cx.entity().downgrade();
                let handle_cancel = cx.entity().downgrade();
                PromptFooter::new(footer_config, footer_colors)
                    .on_primary_click(Box::new(move |_, _window, cx| {
                        if let Some(entity) = handle.upgrade() {
                            entity.update(cx, |this, _cx| {
                                this.submit();
                            });
                        }
                    }))
                    .on_secondary_click(Box::new(move |_, _window, cx| {
                        if let Some(entity) = handle_cancel.upgrade() {
                            entity.update(cx, |this, _cx| {
                                this.submit_cancel();
                            });
                        }
                    }))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_placeholder_copy_is_contextual() {
        assert_eq!(
            env_input_placeholder("OPENAI_API_KEY", false),
            "Paste value for OPENAI_API_KEY"
        );
        assert_eq!(
            env_input_placeholder("OPENAI_API_KEY", true),
            "Paste a replacement value for OPENAI_API_KEY"
        );
    }

    #[test]
    fn env_run_status_message_mentions_waiting_state() {
        assert_eq!(
            env_running_status("OPENAI_API_KEY"),
            "Script is running and waiting for OPENAI_API_KEY"
        );
    }

    #[test]
    fn env_description_mentions_existing_secret_when_present() {
        assert_eq!(
            env_default_description("OPENAI_API_KEY", true),
            "Update the saved value for OPENAI_API_KEY"
        );
        assert_eq!(
            env_default_description("OPENAI_API_KEY", false),
            "Enter the value for OPENAI_API_KEY"
        );
    }

    #[test]
    fn test_env_secret_mask_uses_char_count_when_input_contains_unicode() {
        assert_eq!(masked_secret_value_for_display("abc"), "•••");
        assert_eq!(masked_secret_value_for_display("🔐é"), "••");
    }

    #[test]
    fn test_env_storage_hint_describes_encrypted_store_when_secret() {
        assert_eq!(
            env_storage_hint_text(true),
            "Stored securely in ~/.scriptkit/secrets.age"
        );
    }

    #[test]
    fn test_env_storage_hint_describes_ephemeral_mode_when_not_secret() {
        assert_eq!(
            env_storage_hint_text(false),
            "Value is provided to the script for this run only"
        );
    }

    #[test]
    fn test_env_validation_returns_error_when_submit_value_is_empty() {
        assert_eq!(
            env_submit_validation_error(""),
            Some("Value cannot be empty"),
        );
        assert_eq!(
            env_submit_validation_error("   "),
            Some("Value cannot be empty"),
        );
        assert_eq!(env_submit_validation_error("abc"), None);
    }

    #[test]
    fn test_env_key_action_handles_return_and_esc_aliases() {
        assert_eq!(env_key_action("return"), Some(EnvKeyAction::Submit));
        assert_eq!(env_key_action("esc"), Some(EnvKeyAction::Cancel));
        assert_eq!(env_key_action("tab"), None);
    }
}
