use super::*;
pub struct ChatPrompt {
    pub id: String,
    pub messages: Vec<ChatPromptMessage>,
    pub placeholder: Option<String>,
    pub hint: Option<String>,
    pub footer: Option<String>,
    pub model: Option<String>,
    pub models: Vec<ChatModel>,
    pub title: Option<String>,
    pub focus_handle: FocusHandle,
    pub input: TextInputState,
    pub on_submit: ChatSubmitCallback,
    pub on_escape: Option<ChatEscapeCallback>,
    pub on_continue: Option<ChatContinueCallback>,
    pub on_retry: Option<ChatRetryCallback>,
    pub theme: Arc<theme::Theme>,
    pub turns_list_state: ListState,
    pub(super) prompt_colors: theme::PromptColors,
    pub(super) conversation_turns_cache: Arc<Vec<ConversationTurn>>,
    pub(super) conversation_turns_dirty: bool,
    pub(super) streaming_message_id: Option<String>,
    pub(super) last_copied_response: Option<String>,
    // Database persistence
    pub(super) save_history: bool,
    // Built-in AI provider support (for inline chat without SDK)
    pub(super) provider_registry: Option<ProviderRegistry>,
    pub(super) available_models: Vec<ModelInfo>,
    pub(super) selected_model: Option<ModelInfo>,
    pub(super) builtin_system_prompt: Option<String>,
    pub(super) builtin_streaming_content: String,
    pub(super) builtin_is_streaming: bool,
    // Word-buffered reveal: full accumulated content from provider and reveal watermark
    pub(super) builtin_accumulated_content: String,
    pub(super) builtin_reveal_offset: usize,
    // When true, streaming updates stop forcing the list to the bottom.
    // Reset on explicit "jump to latest" and new submissions.
    pub(super) user_has_scrolled_up: bool,
    // Auto-submit flag: when true, submit the input on first render (for Tab from main menu)
    pub(super) pending_submit: bool,
    // Auto-respond flag: when true, respond to initial messages on first render (for scriptlets)
    pub(super) needs_initial_response: bool,
    // One-shot focus state so chat input auto-focuses when opened without stealing focus later.
    pub(super) pending_auto_focus: bool,
    // Cursor blink state for input field
    pub(super) cursor_visible: bool,
    pub(super) cursor_blink_started: bool,
    // Loading providers: when true, shows "Connecting to AI..." placeholder while providers load
    pub(super) loading_providers: bool,
    // Setup mode: when true, shows API key configuration card instead of chat
    pub(super) needs_setup: bool,
    // Script generation mode: enables post-response Save/Run actions
    pub(super) script_generation_mode: bool,
    pub(super) script_generation_status: Option<String>,
    pub(super) script_generation_status_is_error: bool,
    // Setup card keyboard focus (0 = Configure Vercel, 1 = Claude Code)
    pub(super) setup_focus_index: usize,
    pub(super) on_configure: Option<ChatConfigureCallback>,
    // Callback for "Connect to Claude Code" (enables Claude Code in config)
    pub(super) on_claude_code: Option<ChatClaudeCodeCallback>,
    // Callback for showing actions dialog (handled by parent)
    pub(super) on_show_actions: Option<ChatShowActionsCallback>,
    // Callback for running a saved generated script via parent app pipeline
    pub(super) on_run_script: Option<RunScriptCallback>,
    // Stable UUID for Claude Code CLI session continuity within this prompt's lifetime.
    // Generated once at construction so all messages share the same session.
    pub(super) cli_session_id: String,
    // Image attachment support
    pub(super) pending_image: Option<String>,
    pub(super) pending_image_render: Option<Arc<RenderImage>>,
    pub(super) image_render_cache: HashMap<String, Arc<RenderImage>>,
}

impl ChatPrompt {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        placeholder: Option<String>,
        messages: Vec<ChatPromptMessage>,
        hint: Option<String>,
        footer: Option<String>,
        focus_handle: FocusHandle,
        on_submit: ChatSubmitCallback,
        theme: Arc<theme::Theme>,
    ) -> Self {
        let prompt_colors = theme.colors.prompt_colors();
        logging::log("PROMPTS", &format!("ChatPrompt::new id={}", id));

        let models = default_models();
        let default_model = models.first().map(|m| m.name.clone());

        Self {
            id,
            messages,
            placeholder,
            hint,
            footer,
            model: default_model,
            models,
            title: Some("Chat".to_string()),
            focus_handle,
            input: TextInputState::new(),
            on_submit,
            on_escape: None,
            on_continue: None,
            on_retry: None,
            theme,
            turns_list_state: ListState::new(0, ListAlignment::Bottom, px(1024.0)),
            prompt_colors,
            conversation_turns_cache: Arc::new(Vec::new()),
            conversation_turns_dirty: true,
            streaming_message_id: None,
            last_copied_response: None,
            save_history: true, // Default to saving
            // Built-in AI fields (disabled by default)
            provider_registry: None,
            available_models: Vec::new(),
            selected_model: None,
            builtin_system_prompt: None,
            builtin_streaming_content: String::new(),
            builtin_is_streaming: false,
            builtin_accumulated_content: String::new(),
            builtin_reveal_offset: 0,
            user_has_scrolled_up: false,
            pending_submit: false,
            needs_initial_response: false,
            pending_auto_focus: true,
            cursor_visible: true,
            cursor_blink_started: false,
            loading_providers: false,
            needs_setup: false,
            script_generation_mode: false,
            script_generation_status: None,
            script_generation_status_is_error: false,
            setup_focus_index: 0,
            on_configure: None,
            on_claude_code: None,
            on_show_actions: None,
            on_run_script: None,
            cli_session_id: uuid::Uuid::new_v4().to_string(),
            pending_image: None,
            pending_image_render: None,
            image_render_cache: HashMap::new(),
        }
    }

    /// Set the callback for showing actions dialog
    pub fn set_on_show_actions(&mut self, callback: ChatShowActionsCallback) {
        self.on_show_actions = Some(callback);
    }

    /// Set the callback for running a generated script path in the parent app.
    pub fn with_run_script_callback(
        mut self,
        callback: impl Fn(std::path::PathBuf, &mut Context<Self>) + Send + Sync + 'static,
    ) -> Self {
        self.on_run_script = Some(Arc::new(callback));
        self
    }

    /// Start the cursor blink timer
    pub fn start_cursor_blink(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            loop {
                Timer::after(Duration::from_millis(530)).await;

                // Skip cx.update() entirely when main window is hidden
                // to avoid unnecessary GPUI context access at idle
                if !crate::is_main_window_visible() {
                    continue;
                }

                let result = cx.update(|cx| {
                    this.update(cx, |chat, cx| {
                        // Skip redundant re-renders while streaming —
                        // the streaming reveal loop already drives repaints.
                        if chat.is_streaming() {
                            return;
                        }
                        chat.cursor_visible = !chat.cursor_visible;
                        cx.notify();
                    })
                });
                // Stop blinking if the entity was dropped
                if result.is_err() {
                    break;
                }
            }
        })
        .detach();
    }

    /// Reset cursor to visible (called on user input to keep cursor visible while typing)
    pub(super) fn reset_cursor_blink(&mut self) {
        self.cursor_visible = true;
    }

    /// Normalize pasted text to Unix newlines so multi-line chat input is preserved.
    pub(super) fn normalize_pasted_text(text: &str) -> String {
        text.replace("\r\n", "\n").replace('\r', "\n")
    }

    /// Render helper for input text: show newline intent in a single-line visual field.
    pub(super) fn input_display_text(text: &str) -> String {
        let mut rendered = String::with_capacity(text.len());
        for ch in text.chars() {
            if ch == '\n' {
                rendered.push('↵');
                rendered.push(' ');
            } else {
                rendered.push(ch);
            }
        }
        rendered
    }

    /// Paste text from clipboard while preserving line breaks.
    pub(super) fn paste_text_from_clipboard(&mut self, cx: &mut Context<Self>) -> bool {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            if let Ok(text) = clipboard.get_text() {
                let normalized = Self::normalize_pasted_text(&text);
                if !normalized.is_empty() {
                    self.input.insert_str(&normalized);
                    self.reset_cursor_blink();
                    cx.notify();
                    return true;
                }
            }
        }
        false
    }

    /// Set custom models for the chat
    pub fn with_models(mut self, models: Vec<ChatModel>) -> Self {
        self.models = models;
        if self.model.is_none() {
            self.model = self.models.first().map(|m| m.name.clone());
        }
        self
    }

    /// Set models from string names (creates ChatModel entries with name=id)
    pub fn with_model_names(mut self, model_names: Vec<String>) -> Self {
        if !model_names.is_empty() {
            self.models = model_names
                .into_iter()
                .map(|name| ChatModel::new(name.clone(), name.clone(), "Custom"))
                .collect();
            if self.model.is_none() {
                self.model = self.models.first().map(|m| m.name.clone());
            }
        }
        self
    }

    /// Set the default model
    pub fn with_default_model(mut self, model: String) -> Self {
        self.model = Some(model);
        self
    }

    /// Set the escape callback
    pub fn with_escape_callback(mut self, callback: ChatEscapeCallback) -> Self {
        self.on_escape = Some(callback);
        self
    }

    /// Set the continue callback
    pub fn with_continue_callback(mut self, callback: ChatContinueCallback) -> Self {
        self.on_continue = Some(callback);
        self
    }

    /// Set the retry callback
    pub fn with_retry_callback(mut self, callback: ChatRetryCallback) -> Self {
        self.on_retry = Some(callback);
        self
    }

    /// Set the title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set whether to save chat history to the database
    pub fn with_save_history(mut self, save: bool) -> Self {
        self.save_history = save;
        self
    }

    /// Enable built-in AI mode with the given provider registry.
    /// When enabled, the ChatPrompt will handle AI calls directly instead of using the SDK callback.
    /// If prefer_vercel is true and Vercel is available, it will be used as the default provider.
    pub fn with_builtin_ai(mut self, registry: ProviderRegistry, prefer_vercel: bool) -> Self {
        let available_models = registry.get_all_models();

        // Select default model: prefer Vercel models if available and preferred, otherwise first available
        let selected_model = if prefer_vercel {
            available_models
                .iter()
                .find(|m| m.provider.to_lowercase() == "vercel")
                .or_else(|| available_models.first())
                .cloned()
        } else {
            available_models.first().cloned()
        };

        // Update display models list from provider registry
        self.models = available_models
            .iter()
            .map(|m| ChatModel::new(m.id.clone(), m.display_name.clone(), m.provider.clone()))
            .collect();
        self.model = selected_model.as_ref().map(|m| m.display_name.clone());

        logging::log(
            "CHAT",
            &format!(
                "ChatPrompt with built-in AI: {} models, selected={:?}",
                available_models.len(),
                selected_model.as_ref().map(|m| &m.display_name)
            ),
        );

        self.provider_registry = Some(registry);
        self.available_models = available_models;
        self.selected_model = selected_model;
        self
    }

    /// Set a fixed system prompt used for built-in AI submissions.
    pub fn with_builtin_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.builtin_system_prompt = Some(prompt.into());
        self
    }

    /// Enable script generation mode, which shows Save/Run actions after responses complete.
    pub fn with_script_generation_mode(mut self, enabled: bool) -> Self {
        self.script_generation_mode = enabled;
        self
    }

    /// Set pending_submit flag - when true, auto-submit input on first render
    /// Used for Tab from main menu to immediately send the query to AI
    pub fn with_pending_submit(mut self, submit: bool) -> Self {
        self.pending_submit = submit;
        self
    }

    /// Set needs_initial_response flag - when true, auto-respond to initial messages on first render
    /// Used for scriptlets that call chat() with pre-populated messages
    pub fn with_needs_initial_response(mut self, needs: bool) -> Self {
        self.needs_initial_response = needs;
        self
    }

    /// Set needs_setup flag - when true, shows API configuration card instead of chat
    /// Used when no AI providers are configured
    pub fn with_needs_setup(mut self, needs_setup: bool) -> Self {
        self.needs_setup = needs_setup;
        if needs_setup {
            self.setup_focus_index = 0;
        }
        self
    }

    /// Set loading_providers flag - when true, shows "Connecting to AI..." placeholder
    /// Used while provider registry is being loaded in the background
    pub fn with_loading_providers(mut self, loading: bool) -> Self {
        self.loading_providers = loading;
        self
    }

    /// Whether providers are currently loading
    pub fn loading_providers(&self) -> bool {
        self.loading_providers
    }

    /// Mutably set the provider registry after construction (e.g., when background loading completes).
    /// Clears loading_providers and updates available models.
    pub fn set_provider_registry(
        &mut self,
        registry: ProviderRegistry,
        prefer_vercel: bool,
        cx: &mut Context<Self>,
    ) {
        let available_models = registry.get_all_models();

        let selected_model = if prefer_vercel {
            available_models
                .iter()
                .find(|m| m.provider.to_lowercase() == "vercel")
                .or_else(|| available_models.first())
                .cloned()
        } else {
            available_models.first().cloned()
        };

        self.models = available_models
            .iter()
            .map(|m| ChatModel::new(m.id.clone(), m.display_name.clone(), m.provider.clone()))
            .collect();
        self.model = selected_model.as_ref().map(|m| m.display_name.clone());

        logging::log(
            "CHAT",
            &format!(
                "set_provider_registry: {} models, selected={:?}",
                available_models.len(),
                selected_model.as_ref().map(|m| &m.display_name)
            ),
        );

        self.provider_registry = Some(registry);
        self.available_models = available_models;
        self.selected_model = selected_model;
        self.loading_providers = false;
        cx.notify();
    }

    /// Set the configure callback - called when user clicks "Configure API Key"
    pub fn with_configure_callback(mut self, callback: ChatConfigureCallback) -> Self {
        self.on_configure = Some(callback);
        self
    }

    /// Set the Claude Code callback - called when user clicks "Connect to Claude Code"
    pub fn with_claude_code_callback(mut self, callback: ChatClaudeCodeCallback) -> Self {
        self.on_claude_code = Some(callback);
        self
    }

    /// Whether the setup card is showing (no providers configured)
    pub fn needs_setup(&self) -> bool {
        self.needs_setup
    }

    /// Handle a key event in setup mode from an external interceptor.
    /// Returns true if the key was handled (caller should stop propagation).
    pub fn handle_setup_key(&mut self, key: &str, shift: bool, cx: &mut Context<Self>) -> bool {
        if !self.needs_setup {
            return false;
        }
        let (next_index, action, changed) =
            resolve_setup_card_key(key, shift, self.setup_focus_index);
        let handled = changed || !matches!(action, SetupCardAction::None);

        if changed {
            self.setup_focus_index = next_index;
            cx.notify();
        }

        match action {
            SetupCardAction::ActivateConfigure => {
                if let Some(ref callback) = self.on_configure {
                    callback();
                }
            }
            SetupCardAction::ActivateClaudeCode => {
                if let Some(ref callback) = self.on_claude_code {
                    callback();
                }
            }
            SetupCardAction::Escape => self.handle_escape(cx),
            SetupCardAction::None => {}
        }

        handled
    }

    /// Check if built-in AI mode is enabled
    pub fn has_builtin_ai(&self) -> bool {
        self.provider_registry.is_some()
    }

    pub(super) fn clear_script_generation_status(&mut self) {
        self.script_generation_status = None;
        self.script_generation_status_is_error = false;
    }

    pub(super) fn set_script_generation_status(
        &mut self,
        is_error: bool,
        message: impl Into<String>,
        cx: &mut Context<Self>,
    ) {
        self.script_generation_status = Some(message.into());
        self.script_generation_status_is_error = is_error;
        cx.notify();
    }

    pub(super) fn latest_script_generation_draft(&self) -> Option<(String, String)> {
        if !self.script_generation_mode {
            return None;
        }

        for (index, message) in self.messages.iter().enumerate().rev() {
            if message.is_user() || message.streaming || message.error.is_some() {
                continue;
            }

            let script_source = message.get_content().trim();
            if script_source.is_empty() {
                continue;
            }

            if let Some(user_message) = self.messages[..index].iter().rev().find(|m| m.is_user()) {
                let prompt_description = user_message.get_content().trim();
                if !prompt_description.is_empty() {
                    return Some((prompt_description.to_string(), script_source.to_string()));
                }
            }
        }

        None
    }

    pub(super) fn should_show_script_generation_actions(&self) -> bool {
        should_show_script_generation_actions(
            self.script_generation_mode,
            self.is_streaming(),
            self.latest_script_generation_draft().is_some(),
        )
    }
}
