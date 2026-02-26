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
    /// Whether secret text is currently visible
    pub(super) reveal_secret: bool,
    /// Monotonic counter used to cancel stale auto-hide timers
    pub(super) reveal_generation: u64,
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
            reveal_secret: false,
            reveal_generation: 0,
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

    pub(super) fn toggle_secret_reveal(&mut self, cx: &mut Context<Self>) {
        if !self.secret {
            return;
        }

        self.reveal_secret = !self.reveal_secret;
        self.reveal_generation = self.reveal_generation.wrapping_add(1);
        let reveal_generation = self.reveal_generation;
        let should_auto_hide = self.reveal_secret;
        cx.notify();

        if !should_auto_hide {
            return;
        }

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_secs(5))
                .await;

            cx.update(|cx| {
                let _ = this.update(cx, |prompt, cx| {
                    if prompt.reveal_secret && prompt.reveal_generation == reveal_generation {
                        prompt.reveal_secret = false;
                        cx.notify();
                    }
                });
            });
        })
        .detach();
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
        if self.secret && !self.reveal_secret {
            masked_secret_value_for_display(self.input.text())
        } else {
            self.input.text().to_string()
        }
    }

    pub(super) fn render_text_with_cursor_and_selection(
        &self,
        text: &str,
        text_primary: u32,
        accent_color: u32,
    ) -> Div {
        crate::components::text_input::render_text_input_cursor_selection(
            crate::components::text_input::TextInputRenderConfig {
                cursor: self.input.cursor(),
                selection: Some(self.input.selection()),
                cursor_visible: true,
                cursor_color: text_primary,
                text_color: text_primary,
                selection_color: accent_color,
                selection_text_color: text_primary,
                overflow_x_hidden: true,
                ..crate::components::text_input::TextInputRenderConfig::default_for_prompt(text)
            },
        )
    }

    /// Render the text input with cursor and selection
    pub(super) fn render_input_text(&self, text_primary: u32, accent_color: u32) -> Div {
        let text = self.display_text();
        self.render_text_with_cursor_and_selection(&text, text_primary, accent_color)
    }
}
