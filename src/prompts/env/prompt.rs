use super::*;

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
    pub(super) input: TextInputState,
    /// Focus handle for keyboard input
    pub focus_handle: FocusHandle,
    /// Callback when user submits a value
    pub on_submit: SubmitCallback,
    /// Theme for styling
    pub theme: Arc<theme::Theme>,
    /// Design variant for styling
    pub design_variant: DesignVariant,
    /// Whether we checked the keyring already
    pub(super) checked_keyring: bool,
    /// Whether a value already exists in keyring (for UX messaging)
    pub exists_in_keyring: bool,
    /// When the secret was last modified (if exists)
    pub modified_at: Option<DateTime<Utc>>,
    /// Inline validation/persistence error shown to the user
    pub(super) validation_error: Option<String>,
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

    pub(super) fn correlation_id(&self) -> String {
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
    pub(super) fn submit(&mut self, cx: &mut Context<Self>) {
        let text = self.input.text();
        if let Some(validation_error) = env_submit_validation_error(text) {
            self.validation_error = Some(validation_error.to_string());
            cx.notify();
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
                cx.notify();
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
    pub(super) fn submit_cancel(&mut self) {
        self.validation_error = None;
        (self.on_submit)(self.id.clone(), None);
    }

    /// Delete the secret and close the prompt
    pub(super) fn submit_delete(&mut self, cx: &mut Context<Self>) {
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
            cx.notify();
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
    pub(super) fn display_text(&self) -> String {
        self.input.display_text(self.secret)
    }

    pub(super) fn render_text_with_cursor_and_selection(
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
    pub(super) fn render_input_text(&self, text_primary: u32, accent_color: u32) -> Div {
        if self.secret {
            let masked = masked_secret_value_for_display(self.input.text());
            self.render_text_with_cursor_and_selection(&masked, text_primary, accent_color)
        } else {
            let text = self.display_text();
            self.render_text_with_cursor_and_selection(&text, text_primary, accent_color)
        }
    }
}
