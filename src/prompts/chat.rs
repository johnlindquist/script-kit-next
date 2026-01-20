//! ChatPrompt - Raycast-style chat interface
//!
//! Features:
//! - Input at TOP (not bottom)
//! - Messages bundled as conversation turns (user prompt + AI response in same container)
//! - Full-width containers (not bubbles)
//! - Footer with model selector and "Continue in Chat"
//! - Actions menu (⌘+K) with model picker

use crate::components::prompt_footer::{PromptFooter, PromptFooterColors, PromptFooterConfig};
use crate::components::TextInputState;
use crate::designs::icon_variations::IconName;
use gpui::{
    div, prelude::*, px, rgb, rgba, svg, Context, FocusHandle, Focusable, Hsla, KeyDownEvent,
    Render, ScrollHandle, Timer, Window,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::ai::providers::{ProviderMessage, ProviderRegistry};
use crate::ai::{self, Chat, ChatSource, Message, MessageRole, ModelInfo};
use crate::logging;
use crate::prompts::markdown::render_markdown;
use crate::protocol::{ChatMessagePosition, ChatMessageRole, ChatPromptMessage};
use crate::theme;
use crate::ui_foundation::get_vibrancy_background;

/// Available AI models for the chat
#[derive(Clone, Debug, PartialEq)]
pub struct ChatModel {
    pub id: String,
    pub name: String,
    pub provider: String,
}

impl ChatModel {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        provider: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            provider: provider.into(),
        }
    }
}

/// Default models available in the chat
/// NOTE: First model in list is the default
pub fn default_models() -> Vec<ChatModel> {
    vec![
        // Default: Claude Haiku 4.5 (fast, good quality)
        ChatModel::new("claude-haiku-4-5-20250514", "Claude Haiku 4.5", "Anthropic"),
        ChatModel::new("claude-3-5-haiku-20241022", "Claude 3.5 Haiku", "Anthropic"),
        ChatModel::new(
            "claude-3-5-sonnet-20241022",
            "Claude 3.5 Sonnet",
            "Anthropic",
        ),
        ChatModel::new("gpt-4o-mini", "GPT-4o mini", "OpenAI"),
        ChatModel::new("gpt-4o", "GPT-4o", "OpenAI"),
    ]
}

/// Action item in the actions menu
#[derive(Clone, Debug)]
pub struct ChatAction {
    pub id: String,
    pub label: String,
    pub shortcut: Option<String>,
    pub is_separator: bool,
}

impl ChatAction {
    pub fn new(id: impl Into<String>, label: impl Into<String>, shortcut: Option<&str>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            shortcut: shortcut.map(|s| s.to_string()),
            is_separator: false,
        }
    }

    pub fn separator() -> Self {
        Self {
            id: String::new(),
            label: String::new(),
            shortcut: None,
            is_separator: true,
        }
    }
}

/// Callback type for when user submits a message: (prompt_id, message_text)
pub type ChatSubmitCallback = Arc<dyn Fn(String, String) + Send + Sync>;

/// Callback type for when user presses Escape: (prompt_id)
pub type ChatEscapeCallback = Arc<dyn Fn(String) + Send + Sync>;

/// Callback type for "Continue in Chat": (prompt_id)
pub type ChatContinueCallback = Arc<dyn Fn(String) + Send + Sync>;

/// Callback type for retry: (prompt_id, message_id)
pub type ChatRetryCallback = Arc<dyn Fn(String, String) + Send + Sync>;

/// Callback type for "Configure API" action: () -> triggers API key setup
pub type ChatConfigureCallback = Arc<dyn Fn() + Send + Sync>;

/// Callback type for showing actions menu: (prompt_id) -> triggers ActionsDialog
pub type ChatShowActionsCallback = Arc<dyn Fn(String) + Send + Sync>;

/// A conversation turn: user prompt + optional AI response
#[derive(Clone, Debug)]
pub struct ConversationTurn {
    pub user_prompt: String,
    pub assistant_response: Option<String>,
    pub model: Option<String>,
    pub streaming: bool,
    pub error: Option<String>,
    pub message_id: Option<String>,
}

/// Error types for chat operations
#[derive(Clone, Debug, PartialEq)]
pub enum ChatErrorType {
    NoApiKey,
    NetworkError,
    StreamInterrupted,
    RateLimited,
    InvalidModel,
    TokenLimit,
    Unknown,
}

impl ChatErrorType {
    pub fn from_error_string(s: &str) -> Self {
        let s_lower = s.to_lowercase();
        if s_lower.contains("api key")
            || s_lower.contains("unauthorized")
            || s_lower.contains("401")
        {
            ChatErrorType::NoApiKey
        } else if s_lower.contains("network")
            || s_lower.contains("connection")
            || s_lower.contains("timeout")
        {
            ChatErrorType::NetworkError
        } else if s_lower.contains("interrupt") || s_lower.contains("abort") {
            ChatErrorType::StreamInterrupted
        } else if s_lower.contains("rate limit") || s_lower.contains("429") {
            ChatErrorType::RateLimited
        } else if s_lower.contains("model") || s_lower.contains("invalid") {
            ChatErrorType::InvalidModel
        } else if s_lower.contains("token")
            || s_lower.contains("too long")
            || s_lower.contains("length")
        {
            ChatErrorType::TokenLimit
        } else {
            ChatErrorType::Unknown
        }
    }

    pub fn display_message(&self) -> &'static str {
        match self {
            ChatErrorType::NoApiKey => "⚠ API key not configured. Set up your API key to continue.",
            ChatErrorType::NetworkError => "⚠ Network error. Check your connection and try again.",
            ChatErrorType::StreamInterrupted => "⚠ Response interrupted. Click retry to continue.",
            ChatErrorType::RateLimited => "⚠ Rate limited. Please wait a moment and try again.",
            ChatErrorType::InvalidModel => "⚠ Model unavailable. Using default model.",
            ChatErrorType::TokenLimit => "⚠ Message too long. Try a shorter prompt.",
            ChatErrorType::Unknown => "⚠ Something went wrong. Please try again.",
        }
    }

    pub fn can_retry(&self) -> bool {
        matches!(
            self,
            ChatErrorType::NetworkError
                | ChatErrorType::StreamInterrupted
                | ChatErrorType::RateLimited
                | ChatErrorType::Unknown
        )
    }
}

/// ChatPrompt - Raycast-style chat interface
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
    pub scroll_handle: ScrollHandle,
    prompt_colors: theme::PromptColors,
    streaming_message_id: Option<String>,
    last_copied_response: Option<String>,
    // Actions menu state
    actions_menu_open: bool,
    actions_menu_selected: usize,
    // Database persistence
    save_history: bool,
    // Built-in AI provider support (for inline chat without SDK)
    provider_registry: Option<ProviderRegistry>,
    available_models: Vec<ModelInfo>,
    selected_model: Option<ModelInfo>,
    builtin_streaming_content: String,
    builtin_is_streaming: bool,
    // Auto-submit flag: when true, submit the input on first render (for Tab from main menu)
    pending_submit: bool,
    // Auto-respond flag: when true, respond to initial messages on first render (for scriptlets)
    needs_initial_response: bool,
    // Cursor blink state for input field
    cursor_visible: bool,
    cursor_blink_started: bool,
    // Setup mode: when true, shows API key configuration card instead of chat
    needs_setup: bool,
    on_configure: Option<ChatConfigureCallback>,
    // Callback for showing actions dialog (handled by parent)
    on_show_actions: Option<ChatShowActionsCallback>,
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
            scroll_handle: ScrollHandle::new(),
            prompt_colors,
            streaming_message_id: None,
            last_copied_response: None,
            actions_menu_open: false,
            actions_menu_selected: 0,
            save_history: true, // Default to saving
            // Built-in AI fields (disabled by default)
            provider_registry: None,
            available_models: Vec::new(),
            selected_model: None,
            builtin_streaming_content: String::new(),
            builtin_is_streaming: false,
            pending_submit: false,
            needs_initial_response: false,
            cursor_visible: true,
            cursor_blink_started: false,
            needs_setup: false,
            on_configure: None,
            on_show_actions: None,
        }
    }

    /// Set the callback for showing actions dialog
    pub fn set_on_show_actions(&mut self, callback: ChatShowActionsCallback) {
        self.on_show_actions = Some(callback);
    }

    /// Start the cursor blink timer
    pub fn start_cursor_blink(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            loop {
                Timer::after(Duration::from_millis(530)).await;
                let result = cx.update(|cx| {
                    this.update(cx, |chat, cx| {
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
    fn reset_cursor_blink(&mut self) {
        self.cursor_visible = true;
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
        self
    }

    /// Set the configure callback - called when user clicks "Configure API Key"
    pub fn with_configure_callback(mut self, callback: ChatConfigureCallback) -> Self {
        self.on_configure = Some(callback);
        self
    }

    /// Check if built-in AI mode is enabled
    pub fn has_builtin_ai(&self) -> bool {
        self.provider_registry.is_some()
    }

    pub fn add_message(&mut self, message: ChatPromptMessage, cx: &mut Context<Self>) {
        logging::log(
            "CHAT",
            &format!("Adding message: {:?}", message.get_position()),
        );
        self.messages.push(message);
        self.scroll_handle.scroll_to_bottom();
        cx.notify();
    }

    pub fn start_streaming(
        &mut self,
        message_id: String,
        position: ChatMessagePosition,
        cx: &mut Context<Self>,
    ) {
        let role = match position {
            ChatMessagePosition::Right => Some(ChatMessageRole::User),
            ChatMessagePosition::Left => Some(ChatMessageRole::Assistant),
        };

        let message = ChatPromptMessage {
            id: Some(message_id.clone()),
            role,
            content: Some(String::new()),
            text: String::new(),
            position,
            name: None,
            model: self.model.clone(),
            streaming: true,
            error: None,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
        };
        self.messages.push(message);
        self.streaming_message_id = Some(message_id);
        self.scroll_handle.scroll_to_bottom();
        cx.notify();
    }

    pub fn append_chunk(&mut self, message_id: &str, chunk: &str, cx: &mut Context<Self>) {
        if self.streaming_message_id.as_deref() == Some(message_id) {
            if let Some(msg) = self
                .messages
                .iter_mut()
                .rev()
                .find(|m| m.id.as_deref() == Some(message_id))
            {
                msg.append_content(chunk);
                self.scroll_handle.scroll_to_bottom();
                cx.notify();
            }
        }
    }

    pub fn complete_streaming(&mut self, message_id: &str, cx: &mut Context<Self>) {
        if let Some(msg) = self
            .messages
            .iter_mut()
            .rev()
            .find(|m| m.id.as_deref() == Some(message_id))
        {
            msg.streaming = false;
        }
        if self.streaming_message_id.as_deref() == Some(message_id) {
            self.streaming_message_id = None;
        }
        cx.notify();
    }

    pub fn clear_messages(&mut self, cx: &mut Context<Self>) {
        self.messages.clear();
        self.streaming_message_id = None;
        cx.notify();
    }

    /// Set an error on a message (typically on streaming failure)
    pub fn set_message_error(&mut self, message_id: &str, error: String, cx: &mut Context<Self>) {
        if let Some(msg) = self
            .messages
            .iter_mut()
            .rev()
            .find(|m| m.id.as_deref() == Some(message_id))
        {
            msg.error = Some(error);
            msg.streaming = false; // Stop streaming indicator
        }
        if self.streaming_message_id.as_deref() == Some(message_id) {
            self.streaming_message_id = None;
        }
        cx.notify();
    }

    /// Clear error from a message (before retry)
    pub fn clear_message_error(&mut self, message_id: &str, cx: &mut Context<Self>) {
        if let Some(msg) = self
            .messages
            .iter_mut()
            .rev()
            .find(|m| m.id.as_deref() == Some(message_id))
        {
            msg.error = None;
        }
        cx.notify();
    }

    fn handle_submit(&mut self, cx: &mut Context<Self>) {
        let text = self.input.text().to_string();
        if text.trim().is_empty() {
            return;
        }
        logging::log("CHAT", &format!("User submitted: {}", text));
        self.input.clear();

        // If built-in AI mode is enabled, handle the AI call directly
        if self.has_builtin_ai() {
            self.handle_builtin_ai_submit(text, cx);
        } else {
            // Use SDK callback for script-driven chat
            (self.on_submit)(self.id.clone(), text);
        }
    }

    /// Handle submission in built-in AI mode - calls AI provider directly
    fn handle_builtin_ai_submit(&mut self, text: String, cx: &mut Context<Self>) {
        // Don't allow new messages while streaming
        if self.builtin_is_streaming {
            return;
        }

        // Add user message to UI (ChatPromptMessage::user auto-generates UUID)
        let user_message = ChatPromptMessage::user(text.clone());
        self.messages.push(user_message);
        self.scroll_handle.scroll_to_bottom();
        cx.notify();

        // Get the selected model and provider
        let (model_id, provider) = match &self.selected_model {
            Some(m) => (m.id.clone(), m.provider.clone()),
            None => {
                logging::log("CHAT", "No model selected for built-in AI");
                let error_msg = ChatPromptMessage::assistant(
                    "No AI model configured. Please set up an API key.",
                );
                self.messages.push(error_msg);
                cx.notify();
                return;
            }
        };

        let registry = match &self.provider_registry {
            Some(r) => r.clone(),
            None => return,
        };

        let ai_provider = match registry.find_provider_for_model(&model_id) {
            Some(p) => p.clone(),
            None => {
                logging::log(
                    "CHAT",
                    &format!("No provider found for model: {}", model_id),
                );
                let error_msg = ChatPromptMessage::assistant(format!(
                    "Provider not found for model: {}",
                    model_id
                ));
                self.messages.push(error_msg);
                cx.notify();
                return;
            }
        };

        // Build messages for the API call (convert our messages to provider format)
        let api_messages: Vec<ProviderMessage> = self
            .messages
            .iter()
            .map(|m| {
                if m.is_user() {
                    ProviderMessage::user(m.get_content())
                } else {
                    ProviderMessage::assistant(m.get_content())
                }
            })
            .collect();

        // Set streaming state
        self.builtin_is_streaming = true;
        self.builtin_streaming_content.clear();

        // Add placeholder for assistant response (assistant() auto-generates UUID)
        let assistant_message = ChatPromptMessage::assistant("").with_streaming(true);
        let assistant_msg_id = assistant_message.id.clone().unwrap_or_default();
        self.messages.push(assistant_message);
        self.streaming_message_id = Some(assistant_msg_id.clone());
        cx.notify();

        logging::log(
            "CHAT",
            &format!(
                "Starting built-in AI call: model={}, provider={}, messages={}",
                model_id,
                provider,
                api_messages.len()
            ),
        );

        // Use shared buffer for streaming content
        let shared_content = Arc::new(std::sync::Mutex::new(String::new()));
        let shared_done = Arc::new(AtomicBool::new(false));
        let shared_error = Arc::new(std::sync::Mutex::new(None::<String>));

        let content_clone = shared_content.clone();
        let done_clone = shared_done.clone();
        let error_clone = shared_error.clone();
        let model_id_clone = model_id.clone();

        // Spawn background thread for streaming
        std::thread::spawn(move || {
            let result = ai_provider.stream_message(
                &api_messages,
                &model_id_clone,
                Box::new(move |chunk| {
                    if let Ok(mut content) = content_clone.lock() {
                        content.push_str(&chunk);
                    }
                }),
                None, // SDK chat prompts don't need session persistence
            );

            match result {
                Ok(()) => {
                    done_clone.store(true, Ordering::SeqCst);
                }
                Err(e) => {
                    if let Ok(mut err) = error_clone.lock() {
                        *err = Some(e.to_string());
                    }
                    done_clone.store(true, Ordering::SeqCst);
                }
            }
        });

        // Poll for streaming updates
        let content_for_poll = shared_content.clone();
        let done_for_poll = shared_done.clone();
        let error_for_poll = shared_error.clone();
        let msg_id = assistant_msg_id.clone();

        cx.spawn(async move |this, cx| {
            let mut last_content_len = 0;

            loop {
                Timer::after(std::time::Duration::from_millis(50)).await;

                // Check for new content
                if let Ok(content) = content_for_poll.lock() {
                    if content.len() > last_content_len {
                        let new_content = content.clone();
                        last_content_len = content.len();

                        let msg_id_clone = msg_id.clone();
                        let _ = cx.update(|cx| {
                            this.update(cx, |chat, cx| {
                                // Update the streaming message content
                                if let Some(msg) = chat
                                    .messages
                                    .iter_mut()
                                    .find(|m| m.id.as_deref() == Some(&msg_id_clone))
                                {
                                    msg.set_content(&new_content);
                                }
                                chat.builtin_streaming_content = new_content;
                                chat.scroll_handle.scroll_to_bottom();
                                cx.notify();
                            })
                            .ok();
                        });
                    }
                }

                // Check if done
                if done_for_poll.load(Ordering::SeqCst) {
                    let final_content = content_for_poll.lock().ok().map(|c| c.clone());
                    let error = error_for_poll.lock().ok().and_then(|e| e.clone());

                    let msg_id_clone = msg_id.clone();
                    let _ = cx.update(|cx| {
                        this.update(cx, |chat, cx| {
                            // Complete streaming
                            chat.builtin_is_streaming = false;
                            chat.streaming_message_id = None;

                            if let Some(err) = error {
                                logging::log("CHAT", &format!("Built-in AI error: {}", err));
                                // Set error on the message
                                if let Some(msg) = chat
                                    .messages
                                    .iter_mut()
                                    .find(|m| m.id.as_deref() == Some(&msg_id_clone))
                                {
                                    msg.error = Some(err);
                                    msg.streaming = false;
                                }
                            } else if let Some(content) = final_content {
                                logging::log(
                                    "CHAT",
                                    &format!("Built-in AI complete: {} chars", content.len()),
                                );
                                // Set final content
                                if let Some(msg) = chat
                                    .messages
                                    .iter_mut()
                                    .find(|m| m.id.as_deref() == Some(&msg_id_clone))
                                {
                                    msg.set_content(&content);
                                    msg.streaming = false;
                                }
                            }
                            cx.notify();
                        })
                        .ok();
                    });
                    break;
                }
            }
        })
        .detach();
    }

    /// Handle initial response for pre-populated messages (scriptlets using chat())
    /// Unlike handle_builtin_ai_submit, this doesn't add a new user message - messages are already in self.messages
    fn handle_initial_response(&mut self, cx: &mut Context<Self>) {
        // Don't allow if already streaming
        if self.builtin_is_streaming {
            return;
        }

        // Check if we have messages and the last one is from user
        let has_user_message = self.messages.last().map(|m| m.is_user()).unwrap_or(false);

        if !has_user_message {
            logging::log(
                "CHAT",
                "handle_initial_response: No user message to respond to",
            );
            return;
        }

        logging::log(
            "CHAT",
            &format!(
                "handle_initial_response: Auto-responding to {} initial messages",
                self.messages.len()
            ),
        );

        // Get the selected model and provider
        let (model_id, provider) = match &self.selected_model {
            Some(m) => (m.id.clone(), m.provider.clone()),
            None => {
                logging::log("CHAT", "No model selected for built-in AI initial response");
                let error_msg = ChatPromptMessage::assistant(
                    "No AI model configured. Please set up an API key.",
                );
                self.messages.push(error_msg);
                cx.notify();
                return;
            }
        };

        let registry = match &self.provider_registry {
            Some(r) => r.clone(),
            None => return,
        };

        let ai_provider = match registry.find_provider_for_model(&model_id) {
            Some(p) => p.clone(),
            None => {
                logging::log(
                    "CHAT",
                    &format!("No provider found for model: {}", model_id),
                );
                let error_msg = ChatPromptMessage::assistant(format!(
                    "Provider not found for model: {}",
                    model_id
                ));
                self.messages.push(error_msg);
                cx.notify();
                return;
            }
        };

        // Build messages for the API call (convert our messages to provider format)
        let api_messages: Vec<ProviderMessage> = self
            .messages
            .iter()
            .map(|m| {
                if m.is_user() {
                    ProviderMessage::user(m.get_content())
                } else if matches!(m.role, Some(crate::protocol::ChatMessageRole::System)) {
                    ProviderMessage::system(m.get_content())
                } else {
                    ProviderMessage::assistant(m.get_content())
                }
            })
            .collect();

        // Set streaming state
        self.builtin_is_streaming = true;
        self.builtin_streaming_content.clear();

        // Add placeholder for assistant response
        let assistant_message = ChatPromptMessage::assistant("").with_streaming(true);
        let assistant_msg_id = assistant_message.id.clone().unwrap_or_default();
        self.messages.push(assistant_message);
        self.streaming_message_id = Some(assistant_msg_id.clone());
        cx.notify();

        logging::log(
            "CHAT",
            &format!(
                "Starting built-in AI initial response: model={}, provider={}, messages={}",
                model_id,
                provider,
                api_messages.len()
            ),
        );

        // Use shared buffer for streaming content
        let shared_content = Arc::new(std::sync::Mutex::new(String::new()));
        let shared_done = Arc::new(AtomicBool::new(false));
        let shared_error = Arc::new(std::sync::Mutex::new(None::<String>));

        let content_clone = shared_content.clone();
        let done_clone = shared_done.clone();
        let error_clone = shared_error.clone();
        let model_id_clone = model_id.clone();

        // Spawn background thread for streaming
        std::thread::spawn(move || {
            let result = ai_provider.stream_message(
                &api_messages,
                &model_id_clone,
                Box::new(move |chunk| {
                    if let Ok(mut content) = content_clone.lock() {
                        content.push_str(&chunk);
                    }
                }),
                None, // SDK chat prompts don't need session persistence
            );

            match result {
                Ok(()) => {
                    done_clone.store(true, Ordering::SeqCst);
                }
                Err(e) => {
                    if let Ok(mut err) = error_clone.lock() {
                        *err = Some(e.to_string());
                    }
                    done_clone.store(true, Ordering::SeqCst);
                }
            }
        });

        // Poll for streaming updates
        let content_for_poll = shared_content.clone();
        let done_for_poll = shared_done.clone();
        let error_for_poll = shared_error.clone();
        let msg_id = assistant_msg_id.clone();

        cx.spawn(async move |this, cx| {
            let mut last_content_len = 0;

            loop {
                Timer::after(std::time::Duration::from_millis(50)).await;

                // Check for new content
                if let Ok(content) = content_for_poll.lock() {
                    if content.len() > last_content_len {
                        let new_content = content.clone();
                        last_content_len = content.len();

                        let msg_id_clone = msg_id.clone();
                        let _ = cx.update(|cx| {
                            this.update(cx, |chat, cx| {
                                // Update the streaming message content
                                if let Some(msg) = chat
                                    .messages
                                    .iter_mut()
                                    .find(|m| m.id.as_deref() == Some(&msg_id_clone))
                                {
                                    msg.set_content(&new_content);
                                }
                                chat.builtin_streaming_content = new_content;
                                chat.scroll_handle.scroll_to_bottom();
                                cx.notify();
                            })
                            .ok();
                        });
                    }
                }

                // Check if done
                if done_for_poll.load(Ordering::SeqCst) {
                    let final_content = content_for_poll.lock().ok().map(|c| c.clone());
                    let error = error_for_poll.lock().ok().and_then(|e| e.clone());

                    let msg_id_clone = msg_id.clone();
                    let _ = cx.update(|cx| {
                        this.update(cx, |chat, cx| {
                            // Complete streaming
                            chat.builtin_is_streaming = false;
                            chat.streaming_message_id = None;

                            if let Some(err) = error {
                                logging::log(
                                    "CHAT",
                                    &format!("Built-in AI initial response error: {}", err),
                                );
                                // Set error on the message
                                if let Some(msg) = chat
                                    .messages
                                    .iter_mut()
                                    .find(|m| m.id.as_deref() == Some(&msg_id_clone))
                                {
                                    msg.error = Some(err);
                                    msg.streaming = false;
                                }
                            } else if let Some(content) = final_content {
                                logging::log(
                                    "CHAT",
                                    &format!(
                                        "Built-in AI initial response complete: {} chars",
                                        content.len()
                                    ),
                                );
                                // Set final content
                                if let Some(msg) = chat
                                    .messages
                                    .iter_mut()
                                    .find(|m| m.id.as_deref() == Some(&msg_id_clone))
                                {
                                    msg.set_content(&content);
                                    msg.streaming = false;
                                }
                            }
                            cx.notify();
                        })
                        .ok();
                    });
                    break;
                }
            }
        })
        .detach();
    }

    fn handle_escape(&mut self, _cx: &mut Context<Self>) {
        logging::log("CHAT", "Escape pressed - closing chat");

        // Save conversation to database if save_history is enabled
        if self.save_history {
            self.save_to_database();
        }

        if let Some(ref callback) = self.on_escape {
            callback(self.id.clone());
        }
    }

    /// Save the current conversation to the AI chats database
    fn save_to_database(&self) {
        // Only save if we have messages
        if self.messages.is_empty() {
            logging::log("CHAT", "No messages to save");
            return;
        }

        // Initialize the AI database if needed
        if let Err(e) = ai::init_ai_db() {
            logging::log("CHAT", &format!("Failed to init AI db: {}", e));
            return;
        }

        // Generate title from first user message
        let title = self
            .messages
            .iter()
            .find(|m| m.is_user())
            .map(|m| Chat::generate_title_from_content(m.get_content()))
            .unwrap_or_else(|| "Chat Prompt Conversation".to_string());

        // Determine the model and provider
        let model_id = self.model.clone().unwrap_or_else(|| "unknown".to_string());
        let provider = self
            .models
            .iter()
            .find(|m| m.name == model_id || m.id == model_id)
            .map(|m| m.provider.clone())
            .unwrap_or_else(|| "unknown".to_string());

        // Create the chat record with ChatPrompt source
        let chat = Chat::new(&model_id, &provider).with_source(ChatSource::ChatPrompt);
        let mut chat = chat;
        chat.set_title(&title);

        // Save the chat
        if let Err(e) = ai::create_chat(&chat) {
            logging::log("CHAT", &format!("Failed to save chat: {}", e));
            return;
        }

        // Save all messages
        for (i, msg) in self.messages.iter().enumerate() {
            let role = if msg.is_user() {
                MessageRole::User
            } else {
                MessageRole::Assistant
            };

            let message = Message::new(chat.id, role, msg.get_content());
            if let Err(e) = ai::save_message(&message) {
                logging::log("CHAT", &format!("Failed to save message {}: {}", i, e));
            }
        }

        logging::log(
            "CHAT",
            &format!(
                "Saved conversation with {} messages (id: {})",
                self.messages.len(),
                chat.id
            ),
        );
    }

    pub fn handle_continue_in_chat(&mut self, cx: &mut Context<Self>) {
        logging::log("CHAT", "Continue in Chat - opening AI window");

        // Collect conversation history from messages
        let messages: Vec<(MessageRole, String)> = self
            .messages
            .iter()
            .map(|m| {
                let role = if m.is_user() {
                    MessageRole::User
                } else {
                    MessageRole::Assistant
                };
                (role, m.get_content().to_string())
            })
            .collect();

        logging::log(
            "CHAT",
            &format!("Transferring {} messages to AI window", messages.len()),
        );

        // Open AI window with the chat history
        if let Err(e) = ai::open_ai_window_with_chat(cx, messages) {
            logging::log("CHAT", &format!("Failed to open AI window: {}", e));
        }

        // Close this prompt by calling the escape callback
        if let Some(ref callback) = self.on_escape {
            callback(self.id.clone());
        }
    }

    pub fn handle_copy_last_response(&mut self, cx: &mut Context<Self>) {
        // Find the last assistant message
        if let Some(last_assistant) = self.messages.iter().rev().find(|m| !m.is_user()) {
            let content = last_assistant.get_content().to_string();
            self.last_copied_response = Some(content.clone());
            logging::log("CHAT", &format!("Copied response: {} chars", content.len()));
            // Copy to clipboard via cx
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(content));
        }
    }

    fn handle_clear(&mut self, cx: &mut Context<Self>) {
        logging::log("CHAT", "Clearing conversation (⌘+⌫)");
        self.clear_messages(cx);
    }

    // ============================================
    // Actions Menu Methods
    // ============================================

    fn toggle_actions_menu(&mut self, _cx: &mut Context<Self>) {
        // Delegate to parent via callback to open standard ActionsDialog
        if let Some(ref callback) = self.on_show_actions {
            logging::log("CHAT", "Requesting actions dialog via callback");
            callback(self.id.clone());
        } else {
            logging::log("CHAT", "No on_show_actions callback set");
        }
    }

    fn close_actions_menu(&mut self, _cx: &mut Context<Self>) {
        // Actions menu is now handled by parent - nothing to do here
    }

    /// Get the list of action items for the menu
    fn get_actions(&self) -> Vec<ChatAction> {
        vec![
            ChatAction::new("continue", "Continue in Chat", Some("⌘ ↵")),
            ChatAction::new("copy", "Copy Last Response", Some("⌘ C")),
            ChatAction::new("clear", "Clear Conversation", Some("⌘ ⌫")),
        ]
    }

    /// Get total selectable items (models + actions)
    fn get_menu_item_count(&self) -> usize {
        self.models.len() + self.get_actions().len()
    }

    fn actions_menu_up(&mut self, cx: &mut Context<Self>) {
        if self.actions_menu_selected > 0 {
            self.actions_menu_selected -= 1;
            cx.notify();
        }
    }

    fn actions_menu_down(&mut self, cx: &mut Context<Self>) {
        let max = self.get_menu_item_count().saturating_sub(1);
        if self.actions_menu_selected < max {
            self.actions_menu_selected += 1;
            cx.notify();
        }
    }

    fn actions_menu_select(&mut self, cx: &mut Context<Self>) {
        let selected = self.actions_menu_selected;
        let model_count = self.models.len();

        if selected < model_count {
            // Selected a model
            let model = &self.models[selected];
            self.model = Some(model.name.clone());
            logging::log("CHAT", &format!("Selected model: {}", model.name));
            self.close_actions_menu(cx);
        } else {
            // Selected an action
            let action_idx = selected - model_count;
            let actions = self.get_actions();
            if action_idx < actions.len() {
                let action = &actions[action_idx];
                logging::log("CHAT", &format!("Selected action: {}", action.id));
                match action.id.as_str() {
                    "continue" => {
                        self.close_actions_menu(cx);
                        self.handle_continue_in_chat(cx);
                    }
                    "copy" => {
                        self.handle_copy_last_response(cx);
                        self.close_actions_menu(cx);
                    }
                    "clear" => {
                        self.handle_clear(cx);
                        self.close_actions_menu(cx);
                    }
                    _ => {}
                }
            }
        }
    }

    /// Handle clicking on a specific model in the menu
    fn select_model_by_index(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.models.len() {
            let model = &self.models[index];
            self.model = Some(model.name.clone());
            logging::log("CHAT", &format!("Selected model: {}", model.name));
            self.close_actions_menu(cx);
        }
    }

    /// Handle clicking on a specific action in the menu
    fn select_action_by_id(&mut self, action_id: &str, cx: &mut Context<Self>) {
        match action_id {
            "continue" => {
                self.close_actions_menu(cx);
                self.handle_continue_in_chat(cx);
            }
            "copy" => {
                self.handle_copy_last_response(cx);
                self.close_actions_menu(cx);
            }
            "clear" => {
                self.handle_clear(cx);
                self.close_actions_menu(cx);
            }
            _ => {}
        }
    }

    /// Render the actions menu overlay
    fn render_actions_menu(&self, cx: &Context<Self>) -> impl IntoElement {
        let colors = &self.prompt_colors;
        let model_count = self.models.len();
        let current_model = self.model.clone().unwrap_or_default();

        let menu_bg = rgba((colors.code_bg << 8) | 0xF0);
        let hover_bg = rgba((colors.accent_color << 8) | 0x20);
        let selected_bg = rgba((colors.accent_color << 8) | 0x40);
        let border_color = rgba((colors.quote_border << 8) | 0x60);

        let mut menu = div()
            .absolute()
            .bottom(px(50.0)) // Position above footer
            .left(px(12.0))
            .right(px(12.0))
            .bg(menu_bg)
            .border_1()
            .border_color(border_color)
            .rounded(px(8.0))
            .shadow_lg()
            .flex()
            .flex_col()
            .overflow_hidden();

        // Header
        menu = menu.child(
            div()
                .w_full()
                .px(px(12.0))
                .py(px(8.0))
                .border_b_1()
                .border_color(border_color)
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_xs()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(rgb(colors.text_secondary))
                        .child("Actions"),
                )
                .child(
                    div()
                        .px(px(6.0))
                        .py(px(2.0))
                        .bg(rgba((colors.code_bg << 8) | 0x80))
                        .rounded(px(4.0))
                        .text_xs()
                        .text_color(rgb(colors.text_tertiary))
                        .child("⌘ K"),
                ),
        );

        // Models section
        for (i, model) in self.models.iter().enumerate() {
            let is_selected = i == self.actions_menu_selected;
            let is_current = model.name == current_model;

            let row_bg = if is_selected { Some(selected_bg) } else { None };

            let model_name = model.name.clone();
            let index = i;

            menu = menu.child(
                div()
                    .id(format!("model-{}", i))
                    .w_full()
                    .px(px(12.0))
                    .py(px(8.0))
                    .when_some(row_bg, |d, bg| d.bg(bg))
                    .hover(|s| s.bg(hover_bg))
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _window, cx| {
                        this.select_model_by_index(index, cx);
                    }))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                // Radio button
                                div()
                                    .w(px(14.0))
                                    .h(px(14.0))
                                    .rounded_full()
                                    .border_1()
                                    .border_color(if is_current {
                                        rgb(colors.accent_color)
                                    } else {
                                        rgb(colors.text_tertiary)
                                    })
                                    .when(is_current, |d| {
                                        d.child(
                                            div()
                                                .w(px(8.0))
                                                .h(px(8.0))
                                                .m(px(2.0))
                                                .rounded_full()
                                                .bg(rgb(colors.accent_color)),
                                        )
                                    }),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(colors.text_primary))
                                    .child(model_name),
                            ),
                    )
                    .when(is_current, |d| {
                        d.child(
                            div()
                                .text_xs()
                                .text_color(rgb(colors.text_tertiary))
                                .child("✓"),
                        )
                    }),
            );
        }

        // Separator
        menu = menu.child(div().w_full().h(px(1.0)).bg(border_color));

        // Actions section
        let actions = self.get_actions();
        for (i, action) in actions.iter().enumerate() {
            if action.is_separator {
                menu = menu.child(div().w_full().h(px(1.0)).bg(border_color));
                continue;
            }

            let menu_index = model_count + i;
            let is_selected = menu_index == self.actions_menu_selected;

            let row_bg = if is_selected { Some(selected_bg) } else { None };

            let action_id = action.id.clone();
            let action_label = action.label.clone();
            let shortcut = action.shortcut.clone();

            menu = menu.child(
                div()
                    .id(format!("action-{}", i))
                    .w_full()
                    .px(px(12.0))
                    .py(px(8.0))
                    .when_some(row_bg, |d, bg| d.bg(bg))
                    .hover(|s| s.bg(hover_bg))
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _window, cx| {
                        this.select_action_by_id(&action_id, cx);
                    }))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(colors.text_primary))
                            .child(action_label),
                    )
                    .when_some(shortcut, |d, s| {
                        d.child(
                            div()
                                .text_xs()
                                .text_color(rgb(colors.text_tertiary))
                                .child(s),
                        )
                    }),
            );
        }

        menu
    }

    /// Group messages into conversation turns (user + assistant pairs)
    fn get_conversation_turns(&self) -> Vec<ConversationTurn> {
        let mut turns = Vec::new();
        let mut i = 0;

        while i < self.messages.len() {
            let msg = &self.messages[i];

            if msg.is_user() {
                // Start a new turn with this user message
                let user_prompt = msg.get_content().to_string();
                let mut turn = ConversationTurn {
                    user_prompt,
                    assistant_response: None,
                    model: None,
                    streaming: false,
                    error: None,
                    message_id: msg.id.clone(),
                };

                // Look for the next assistant response
                if i + 1 < self.messages.len() {
                    let next_msg = &self.messages[i + 1];
                    if !next_msg.is_user() {
                        turn.assistant_response = Some(next_msg.get_content().to_string());
                        turn.model = next_msg.model.clone();
                        turn.streaming = next_msg.streaming;
                        turn.error = next_msg.error.clone();
                        turn.message_id = next_msg.id.clone().or(turn.message_id);
                        i += 1;
                    }
                }

                turns.push(turn);
            } else {
                // Standalone assistant message (no user prompt before it)
                // This happens for system-initiated messages
                let turn = ConversationTurn {
                    user_prompt: String::new(),
                    assistant_response: Some(msg.get_content().to_string()),
                    model: msg.model.clone(),
                    streaming: msg.streaming,
                    error: msg.error.clone(),
                    message_id: msg.id.clone(),
                };
                turns.push(turn);
            }

            i += 1;
        }

        turns
    }

    /// Render a conversation turn (user prompt + AI response bundled)
    fn render_turn(
        &self,
        turn: &ConversationTurn,
        turn_index: usize,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let colors = &self.prompt_colors;

        // VIBRANCY: Use white at low opacity for subtle brightening that lets blur show through
        let container_bg = rgba((0xFFFFFF << 8) | 0x15); // White at ~8% opacity
        let copy_hover_bg = rgba((0xFFFFFF << 8) | 0x28); // White at ~16% for hover
        let error_color = self.theme.colors.ui.error;
        let error_bg = rgba((error_color << 8) | 0x40); // Theme error with transparency
        let retry_hover_bg = rgba((colors.accent_color << 8) | 0x40);
        let has_retry_callback = self.on_retry.is_some();

        let mut content = div().flex().flex_col().gap(px(4.0)).w_full().min_w_0();
        // Note: removed overflow_hidden() to allow text to wrap naturally

        // User prompt (small, bold) - only if not empty
        if !turn.user_prompt.is_empty() {
            content = content.child(
                div()
                    .w_full()
                    .min_w_0()
                    .text_sm()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(colors.text_secondary))
                    .child(turn.user_prompt.clone()),
            );
        }

        // Error state - show error message with optional retry button
        if let Some(ref error_str) = turn.error {
            let error_type = ChatErrorType::from_error_string(error_str);
            let error_message = error_type.display_message();
            let can_retry = error_type.can_retry() && has_retry_callback;

            let mut error_row = div().flex().flex_row().items_center().gap(px(8.0)).child(
                div()
                    .text_sm()
                    .text_color(rgb(error_color))
                    .child(error_message.to_string()),
            );

            // Add retry button if applicable
            if can_retry {
                let message_id = turn.message_id.clone();
                error_row = error_row.child(
                    div()
                        .id(format!("retry-turn-{}", turn_index))
                        .px(px(8.0))
                        .py(px(4.0))
                        .bg(error_bg)
                        .rounded(px(4.0))
                        .cursor_pointer()
                        .hover(|s| s.bg(retry_hover_bg))
                        .text_xs()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(rgb(colors.text_primary))
                        .child("Retry")
                        .on_click(cx.listener(move |this, _, _window, _cx| {
                            if let Some(msg_id) = &message_id {
                                this.handle_retry(msg_id.clone());
                            }
                        })),
                );
            }

            content = content.child(error_row);
        }
        // AI response (only show if no error, or show partial if stream interrupted)
        else if let Some(ref response) = turn.assistant_response {
            // Use markdown rendering for assistant responses
            if turn.streaming && response.is_empty() {
                // Empty streaming state
                content = content.child(div().text_xs().opacity(0.6).child("Thinking..."));
            } else if turn.streaming {
                // Streaming with content - render markdown + cursor
                content = content.child(
                    div()
                        .flex()
                        .flex_col()
                        .w_full()
                        .min_w_0()
                        .overflow_x_hidden() // Only clip horizontal overflow for long unbreakable content
                        .child(render_markdown(response, colors))
                        .child(div().text_color(rgb(colors.accent_color)).child("▌")),
                );
            } else {
                // Complete response - full markdown rendering (with container for proper wrapping)
                content = content.child(
                    div()
                        .w_full()
                        .min_w_0()
                        .overflow_x_hidden()
                        .child(render_markdown(response, colors)),
                );
            }
        }

        // Copy button (appears on right side) - copies assistant response
        let copy_button = div()
            .id(format!("copy-turn-{}", turn_index))
            .flex()
            .items_center()
            .justify_center()
            .w(px(24.0))
            .h(px(24.0))
            .rounded(px(4.0))
            .cursor_pointer()
            .opacity(0.7)
            .hover(|s| s.opacity(1.0).bg(copy_hover_bg))
            .child(
                svg()
                    .external_path(IconName::Copy.external_path())
                    .size(px(16.))
                    .text_color(rgb(colors.text_secondary)),
            )
            .on_click(cx.listener(move |this, _, _window, cx| {
                this.copy_turn_response(turn_index, cx);
            }));

        // The full-width container with copy button
        div()
            .w_full()
            .px(px(12.0))
            .py(px(10.0))
            .bg(container_bg)
            .rounded(px(8.0))
            .flex()
            .flex_row()
            .gap(px(8.0))
            .child(content.flex_1().min_w_0())
            .child(copy_button)
    }

    /// Handle retry for a failed message
    fn handle_retry(&self, message_id: String) {
        logging::log(
            "CHAT",
            &format!("Retry requested for message: {}", message_id),
        );
        if let Some(ref callback) = self.on_retry {
            callback(self.id.clone(), message_id);
        }
    }

    /// Copy the assistant response from a specific turn
    fn copy_turn_response(&mut self, turn_index: usize, cx: &mut Context<Self>) {
        let turns = self.get_conversation_turns();
        if let Some(turn) = turns.get(turn_index) {
            if let Some(ref response) = turn.assistant_response {
                let content = response.clone();
                logging::log(
                    "CHAT",
                    &format!(
                        "Copied turn {} response: {} chars",
                        turn_index,
                        content.len()
                    ),
                );
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(content));
            } else if !turn.user_prompt.is_empty() {
                // If no assistant response, copy the user prompt
                let content = turn.user_prompt.clone();
                logging::log(
                    "CHAT",
                    &format!(
                        "Copied turn {} user prompt: {} chars",
                        turn_index,
                        content.len()
                    ),
                );
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(content));
            }
        }
    }

    /// Render the setup card when no API keys are configured
    fn render_setup_card(&self, cx: &Context<Self>) -> impl IntoElement {
        let colors = &self.prompt_colors;

        // Card styling values
        let _card_border = (colors.quote_border << 8) | 0x60; // 40% opacity (unused but kept for future)
        let card_bg = (colors.code_bg << 8) | 0x30; // ~20% opacity
        let accent_bg = (colors.accent_color << 8) | 0x26; // 15% opacity
        let accent_border = (colors.accent_color << 8) | 0x40; // 25% opacity

        // Get the configure callback for the button click
        let on_configure = self.on_configure.clone();

        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .flex_1()
            .gap(px(20.))
            .px(px(24.))
            // Icon - settings/key icon
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(56.))
                    .rounded(px(14.))
                    .bg(rgba(card_bg))
                    .child(
                        svg()
                            .path(IconName::Settings.external_path())
                            .size(px(28.))
                            .text_color(rgb(colors.text_secondary)),
                    ),
            )
            // Title
            .child(
                div()
                    .text_lg()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(colors.text_primary))
                    .child("API Key Required"),
            )
            // Description
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(colors.text_secondary))
                    .text_center()
                    .max_w(px(320.))
                    .child("Set up an API key to use the Ask AI feature. The easiest option is Vercel AI Gateway."),
            )
            // Configure button
            .child(
                div()
                    .id("configure-button")
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap(px(8.))
                    .px(px(16.))
                    .py(px(10.))
                    .rounded(px(8.))
                    .bg(rgba(accent_bg))
                    .border_1()
                    .border_color(rgba(accent_border))
                    .cursor_pointer()
                    .hover(|s| s.bg(rgba((colors.accent_color << 8) | 0x40)))
                    .when_some(on_configure.clone(), |d, callback| {
                        d.on_click(cx.listener(move |_this, _event, _window, _cx| {
                            logging::log("CHAT", "Configure button clicked - triggering API key setup");
                            callback();
                        }))
                    })
                    .child(
                        svg()
                            .path(IconName::Settings.external_path())
                            .size(px(16.))
                            .text_color(rgb(colors.accent_color)),
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(rgb(colors.accent_color))
                            .child("Configure Vercel AI Gateway"),
                    ),
            )
            // Hint about no restart needed
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(4.))
                    .mt(px(8.))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(colors.text_tertiary))
                            .child("No restart required"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(colors.text_tertiary))
                            .child("After configuring, press Tab again to try"),
                    ),
            )
            // Keyboard hint
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .mt(px(12.))
                    .child(
                        div()
                            .px(px(6.))
                            .py(px(2.))
                            .rounded(px(4.))
                            .bg(rgba(card_bg))
                            .text_xs()
                            .text_color(rgb(colors.text_tertiary))
                            .child("Enter"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(colors.text_tertiary))
                            .child("to configure"),
                    )
                    .child(
                        div()
                            .px(px(6.))
                            .py(px(2.))
                            .rounded(px(4.))
                            .bg(rgba(card_bg))
                            .text_xs()
                            .text_color(rgb(colors.text_tertiary))
                            .child("Esc"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(colors.text_tertiary))
                            .child("to go back"),
                    ),
            )
    }

    /// Render the input field at the top
    fn render_input(&self, _cx: &Context<Self>) -> impl IntoElement {
        let colors = &self.prompt_colors;
        let text = self.input.text();
        let cursor_pos = self.input.cursor();
        let chars: Vec<char> = text.chars().collect();
        let cursor_visible = self.cursor_visible;

        let mut input_content = div().flex().flex_row().items_center();

        // Text before cursor
        if cursor_pos > 0 {
            let before: String = chars[..cursor_pos].iter().collect();
            input_content =
                input_content.child(div().text_color(rgb(colors.text_primary)).child(before));
        }

        // Cursor (blinking)
        let cursor = div()
            .w(px(2.0))
            .h(px(16.0))
            .when(cursor_visible, |d| d.bg(rgb(colors.accent_color)));
        input_content = input_content.child(cursor);

        // Text after cursor
        if cursor_pos < chars.len() {
            let after: String = chars[cursor_pos..].iter().collect();
            input_content =
                input_content.child(div().text_color(rgb(colors.text_primary)).child(after));
        }

        // Placeholder if empty - cursor appears BEFORE placeholder text
        if text.is_empty() {
            let placeholder = self
                .placeholder
                .clone()
                .unwrap_or_else(|| "Ask follow-up...".into());
            let cursor = div()
                .w(px(2.0))
                .h(px(16.0))
                .when(cursor_visible, |d| d.bg(rgb(colors.accent_color)));
            input_content = div().flex().flex_row().items_center().child(cursor).child(
                div()
                    .text_color(rgb(colors.text_tertiary))
                    .child(placeholder),
            );
        }

        input_content
    }

    /// Render the header with back button and title
    fn render_header(&self) -> impl IntoElement {
        let colors = &self.prompt_colors;
        let title = self.title.clone().unwrap_or_else(|| "Chat".into());

        div()
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.0))
            .px(px(12.0))
            .py(px(8.0))
            .border_b_1()
            .border_color(rgba((colors.quote_border << 8) | 0x40))
            .child(
                // Back arrow
                div()
                    .text_sm()
                    .text_color(rgb(colors.text_secondary))
                    .child("←"),
            )
            .child(
                // Title
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_primary))
                    .child(title),
            )
    }

    /// Render the footer using the standard PromptFooter component
    /// Shows: Logo + Model name | Continue in Chat ⌘↵ | Actions ⌘K
    fn render_footer(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        // Use standard PromptFooter colors from theme
        let footer_colors = PromptFooterColors::from_theme(&self.theme);

        // Build model display text (show model name if available)
        let model_text = self.model.clone().unwrap_or_else(|| "Select Model".into());

        // Configure footer with chat-specific labels
        let footer_config = PromptFooterConfig::new()
            .primary_label("Continue in Chat")
            .primary_shortcut("⌘↵")
            .secondary_label("Actions")
            .secondary_shortcut("⌘K")
            .show_logo(true)
            .show_secondary(true)
            .helper_text(model_text); // Show model name next to logo

        // Note: Click handlers are not wired up here because PromptFooter uses
        // RenderOnce with static callbacks. The keyboard shortcuts (⌘↵ and ⌘K)
        // handle the actual functionality via the parent's key handler.
        PromptFooter::new(footer_config, footer_colors)
    }
}

impl Focusable for ChatPrompt {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ChatPrompt {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Start cursor blink timer on first render (only needed when not in setup mode)
        if !self.needs_setup && !self.cursor_blink_started {
            self.cursor_blink_started = true;
            self.start_cursor_blink(cx);
        }

        // Process pending_submit on first render (used when Tab opens chat with query)
        // Skip if in setup mode
        if !self.needs_setup && self.pending_submit && !self.input.is_empty() {
            self.pending_submit = false;
            logging::log(
                "CHAT",
                "Processing pending_submit - auto-submitting query from Tab",
            );
            self.handle_submit(cx);
        }

        // Process needs_initial_response on first render (used for scriptlets with pre-populated messages)
        // Skip if in setup mode, requires built-in AI to be enabled
        if !self.needs_setup && self.needs_initial_response && self.has_builtin_ai() {
            self.needs_initial_response = false;
            logging::log(
                "CHAT",
                "Processing needs_initial_response - auto-responding to initial messages",
            );
            self.handle_initial_response(cx);
        }

        let colors = &self.prompt_colors;

        let needs_setup = self.needs_setup;
        let on_configure = self.on_configure.clone();

        let handle_key = cx.listener(move |this, event: &KeyDownEvent, _window, cx| {
            let key = event.keystroke.key.to_lowercase();
            let key_char = event.keystroke.key_char.as_deref();
            let has_cmd = event.keystroke.modifiers.platform; // ⌘ on macOS

            // In setup mode, only handle Escape and Enter
            if needs_setup {
                match key.as_str() {
                    "escape" => this.handle_escape(cx),
                    "enter" => {
                        // Trigger configure callback on Enter
                        if let Some(ref callback) = on_configure {
                            logging::log(
                                "CHAT",
                                "Enter pressed in setup mode - triggering configure",
                            );
                            callback();
                        }
                    }
                    _ => {}
                }
                return;
            }

            // Note: Actions menu keyboard navigation is handled by ActionsDialog window
            // We just need to handle ⌘K to open it via callback

            match key.as_str() {
                // Escape - close chat
                "escape" => this.handle_escape(cx),
                // ⌘+K - Toggle actions menu
                "k" if has_cmd => this.toggle_actions_menu(cx),
                // ⌘+Enter - Continue in Chat
                "enter" if has_cmd => this.handle_continue_in_chat(cx),
                // Enter - Submit message
                "enter" if !event.keystroke.modifiers.shift => this.handle_submit(cx),
                // ⌘+C - Copy last response
                "c" if has_cmd => this.handle_copy_last_response(cx),
                // ⌘+Backspace - Clear conversation
                "backspace" if has_cmd => this.handle_clear(cx),
                // Regular backspace
                "backspace" => {
                    this.input.backspace();
                    this.reset_cursor_blink();
                    cx.notify();
                }
                _ => {
                    // Ignore command shortcuts (don't insert characters)
                    if has_cmd {
                        return;
                    }
                    if let Some(ch_str) = key_char {
                        for ch in ch_str.chars() {
                            if ch.is_ascii_graphic() || ch == ' ' {
                                this.input.insert_char(ch);
                            }
                        }
                        this.reset_cursor_blink();
                        cx.notify();
                    }
                }
            }
        });

        let container_bg: Option<Hsla> = get_vibrancy_background(&self.theme).map(Hsla::from);

        // If needs_setup, render setup card instead of normal chat
        if self.needs_setup {
            return div()
                .id("chat-prompt-setup")
                .flex()
                .flex_col()
                .w_full()
                .h_full()
                .when_some(container_bg, |d, bg| d.bg(bg))
                .key_context("chat_prompt_setup")
                .track_focus(&self.focus_handle)
                .on_key_down(handle_key)
                // Header with back button and title
                .child(self.render_header())
                // Setup card content
                .child(self.render_setup_card(cx))
                .into_any_element();
        }

        // Input area at TOP
        let input_area = div()
            .w_full()
            .px(px(12.0))
            .py(px(10.0))
            .border_b_1()
            .border_color(rgba((colors.quote_border << 8) | 0x40))
            .child(self.render_input(cx));

        // Message list (conversation turns)
        let turns = self.get_conversation_turns();
        let mut message_list = div()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .w_full()
            .px(px(12.0))
            .py(px(12.0));

        for (i, turn) in turns.iter().enumerate() {
            message_list = message_list.child(self.render_turn(turn, i, cx));
        }

        // Empty state
        if turns.is_empty() {
            message_list = message_list.child(
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(rgb(colors.text_tertiary))
                    .text_sm()
                    .child("Type a question to start..."),
            );
        }

        div()
            .id("chat-prompt")
            .relative() // For absolute positioning of actions menu
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .when_some(container_bg, |d, bg| d.bg(bg))
            .key_context("chat_prompt")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            // Header with back button and title
            .child(self.render_header())
            // Input area
            .child(input_area)
            // Scrollable message area
            .child(
                div()
                    .id("chat-messages")
                    .flex_1()
                    .min_h(px(0.))
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll_handle)
                    .child(message_list),
            )
            // Footer with model selector, Continue in Chat, and Actions
            .child(self.render_footer(cx))
            // Note: Actions menu is now handled by parent via on_show_actions callback
            // The parent opens the standard ActionsDialog window
            .into_any_element()
    }
}
