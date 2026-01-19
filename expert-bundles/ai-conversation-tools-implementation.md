  # AI Conversation Logic & Tools Implementation Expert Bundle
  
  ## Original Goal
  
  > Create a bundle for all the logic around our AI conversation logic. We need to ask an expert how we can implement tools like web search and other common/required tools we need for a solid AI implementation.
  
  This bundle contains the complete AI conversation system for Script Kit GPUI, ready for an expert to review and advise on implementing AI tools (web search, code execution, etc.).
  
  ## Executive Summary
  
  Script Kit GPUI has a well-structured AI conversation system built in Rust using GPUI. The system currently supports:
  - Multiple AI providers (OpenAI, Anthropic, Google, Groq, Vercel AI Gateway)
  - Streaming responses via SSE (Server-Sent Events)
  - Multi-modal messages (text + images)
  - Chat persistence with SQLite + FTS5 search
  - Both standalone AI window and inline ChatPrompt
  
  **The key gap**: The system has **NO tool/function calling implementation**. The `AiProvider` trait only supports `send_message` and `stream_message` - there's no mechanism for tools, tool_use, or function calling.
  
  ### Key Problems:
  1. **No tool calling infrastructure** - The provider trait and message types don't support tools
  2. **No tool definitions** - No way to define or register tools (web search, code execution, etc.)
  3. **No tool result handling** - No mechanism to receive tool calls, execute them, and return results
  4. **Streaming doesn't support tool_use events** - SSE parsing only extracts text content
  
  ### Required Fixes:
  1. Extend `ProviderMessage` to support tool definitions and tool results
  2. Add `tool_use` parsing to SSE stream handlers for each provider
  3. Create a tool registry system for registering available tools
  4. Implement common tools: web search, code execution, file operations
  5. Add tool execution loop (AI calls tool → execute → return result → continue)
  
  ### Files Included:
  - `src/ai/mod.rs` - Module exports and documentation
  - `src/ai/providers.rs` - **KEY**: Provider trait and implementations (OpenAI, Anthropic, Vercel)
  - `src/ai/model.rs` - Data models (Chat, Message, MessageRole)
  - `src/ai/config.rs` - API key detection and model configuration
  - `src/ai/sdk_handlers.rs` - Protocol handlers for SDK messages
  - `src/prompts/chat.rs` - ChatPrompt UI component with streaming
  
  ---
  
  ## Architecture Overview
  
  ```
  ┌─────────────────────────────────────────────────────────────────┐
  │                         AI Module                                │
  ├─────────────────────────────────────────────────────────────────┤
  │                                                                  │
  │   ┌─────────────┐     ┌──────────────┐     ┌────────────────┐   │
  │   │   config    │────▶│  providers   │────▶│  ProviderMsg   │   │
  │   │  (API keys) │     │  (trait impl)│     │  (text+images) │   │
  │   └─────────────┘     └──────────────┘     └────────────────┘   │
  │                              │                                   │
  │                              │ AiProvider trait                  │
  │                              │  - send_message()                 │
  │                              │  - stream_message()               │
  │                              ▼                                   │
  │   ┌─────────────┐     ┌──────────────┐     ┌────────────────┐   │
  │   │   storage   │◀────│   model      │────▶│  ChatPrompt    │   │
  │   │  (SQLite)   │     │ (Chat, Msg)  │     │  (UI + stream) │   │
  │   └─────────────┘     └──────────────┘     └────────────────┘   │
  │                                                                  │
  └─────────────────────────────────────────────────────────────────┘
  ```
  
  ## Current Provider Trait (No Tool Support)
  
  ```rust
  // From src/ai/providers.rs - This is what needs to be extended
  
  pub trait AiProvider: Send + Sync {
      fn provider_id(&self) -> &str;
      fn display_name(&self) -> &str;
      fn available_models(&self) -> Vec<ModelInfo>;
      
      // These only support text/images, NOT tools
      fn send_message(&self, messages: &[ProviderMessage], model_id: &str) -> Result<String>;
      fn stream_message(
          &self,
          messages: &[ProviderMessage],
          model_id: &str,
          on_chunk: StreamCallback,
      ) -> Result<()>;
  }
  ```
  
  ## What's Missing for Tools
  
  ### 1. Tool Definition Type
  ```rust
  // NEEDED: Tool schema definition
  pub struct ToolDefinition {
      pub name: String,
      pub description: String,
      pub parameters: serde_json::Value, // JSON Schema
  }
  ```
  
  ### 2. Tool Use/Result Types
  ```rust
  // NEEDED: Tool call from AI
  pub struct ToolUse {
      pub id: String,
      pub name: String,
      pub input: serde_json::Value,
  }
  
  // NEEDED: Tool result to send back
  pub struct ToolResult {
      pub tool_use_id: String,
      pub content: String, // or structured output
      pub is_error: bool,
  }
  ```
  
  ### 3. Extended Provider Trait
  ```rust
  // NEEDED: Provider trait with tool support
  pub trait AiProvider: Send + Sync {
      // ... existing methods ...
      
      fn send_message_with_tools(
          &self,
          messages: &[ProviderMessage],
          tools: &[ToolDefinition],
          model_id: &str,
      ) -> Result<MessageWithToolUse>;
      
      fn stream_message_with_tools(
          &self,
          messages: &[ProviderMessage],
          tools: &[ToolDefinition],
          model_id: &str,
          on_chunk: StreamCallback,
          on_tool_use: ToolUseCallback,
      ) -> Result<()>;
  }
  ```
  
  ---
  
  ## Code Bundle (from packx)
  
  This file is a merged representation of the filtered codebase, combined into a single document by packx.
  
  <file_summary>
  This section contains a summary of this file.
  
  <purpose>
  This file contains a packed representation of filtered repository contents.
  It is designed to be easily consumable by AI systems for analysis, code review,
  or other automated processes.
  </purpose>
  
  <usage_guidelines>
  - Treat this file as a snapshot of the repository's state
  - Be aware that this file may contain sensitive information
  </usage_guidelines>
  
  <notes>
  - Files were filtered by packx based on content and extension matching
  - Total files included: 6
  </notes>
  </file_summary>
  
  <directory_structure>
  src/prompts/chat.rs
  src/ai/model.rs
  src/ai/providers.rs
  src/ai/mod.rs
  src/ai/config.rs
  src/ai/sdk_handlers.rs
  </directory_structure>
  
  <files>
  This section contains the contents of the repository's files.
  
  <file path="src/prompts/chat.rs">
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
  
  </file>
  
  <file path="src/ai/model.rs">
  //! AI Chat Data Models
  //!
  //! Core data structures for the AI chat window feature.
  //! Follows the same patterns as src/notes/model.rs for consistency.
  
  use chrono::{DateTime, Utc};
  use serde::{Deserialize, Serialize};
  use uuid::Uuid;
  
  /// Unique identifier for a chat conversation
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
  pub struct ChatId(pub Uuid);
  
  impl ChatId {
      /// Create a new random ChatId
      pub fn new() -> Self {
          Self(Uuid::new_v4())
      }
  
      /// Create a ChatId from a UUID string
      pub fn parse(s: &str) -> Option<Self> {
          Uuid::parse_str(s).ok().map(Self)
      }
  
      /// Get the UUID as a string
      pub fn as_str(&self) -> String {
          self.0.to_string()
      }
  }
  
  impl Default for ChatId {
      fn default() -> Self {
          Self::new()
      }
  }
  
  impl std::fmt::Display for ChatId {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
          write!(f, "{}", self.0)
      }
  }
  
  /// Role of a message in a chat conversation
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
  #[serde(rename_all = "lowercase")]
  pub enum MessageRole {
      /// Message from the user
      User,
      /// Message from the AI assistant
      Assistant,
      /// System prompt/instruction
      System,
  }
  
  impl MessageRole {
      /// Convert to string representation
      pub fn as_str(&self) -> &'static str {
          match self {
              MessageRole::User => "user",
              MessageRole::Assistant => "assistant",
              MessageRole::System => "system",
          }
      }
  
      /// Parse from string (fallible, returns Option)
      pub fn parse(s: &str) -> Option<Self> {
          match s.to_lowercase().as_str() {
              "user" => Some(MessageRole::User),
              "assistant" => Some(MessageRole::Assistant),
              "system" => Some(MessageRole::System),
              _ => None,
          }
      }
  }
  
  impl std::str::FromStr for MessageRole {
      type Err = String;
  
      fn from_str(s: &str) -> Result<Self, Self::Err> {
          MessageRole::parse(s).ok_or_else(|| format!("Invalid message role: {}", s))
      }
  }
  
  impl std::fmt::Display for MessageRole {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
          write!(f, "{}", self.as_str())
      }
  }
  
  /// Source of a chat (where it originated from)
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
  #[serde(rename_all = "snake_case")]
  pub enum ChatSource {
      /// Chat from the AI window
      #[default]
      AiWindow,
      /// Chat from the chat() SDK prompt
      ChatPrompt,
      /// Chat from a script (programmatic)
      Script,
  }
  
  impl ChatSource {
      /// Convert to string for storage
      pub fn as_str(&self) -> &'static str {
          match self {
              ChatSource::AiWindow => "ai_window",
              ChatSource::ChatPrompt => "chat_prompt",
              ChatSource::Script => "script",
          }
      }
  
      /// Parse from string
      pub fn parse(s: &str) -> Self {
          match s {
              "chat_prompt" => ChatSource::ChatPrompt,
              "script" => ChatSource::Script,
              _ => ChatSource::AiWindow,
          }
      }
  }
  
  /// A chat conversation
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct Chat {
      /// Unique identifier
      pub id: ChatId,
  
      /// Chat title (auto-generated from first message or user-set)
      pub title: String,
  
      /// When the chat was created
      pub created_at: DateTime<Utc>,
  
      /// When the chat was last modified
      pub updated_at: DateTime<Utc>,
  
      /// When the chat was soft-deleted (None = not deleted)
      pub deleted_at: Option<DateTime<Utc>>,
  
      /// Model identifier (e.g., "claude-3-opus", "gpt-4")
      pub model_id: String,
  
      /// Provider identifier (e.g., "anthropic", "openai")
      pub provider: String,
  
      /// Source of the chat (ai_window, chat_prompt, script)
      #[serde(default)]
      pub source: ChatSource,
  }
  
  impl Chat {
      /// Create a new empty chat with the specified model and provider
      pub fn new(model_id: impl Into<String>, provider: impl Into<String>) -> Self {
          let now = Utc::now();
          Self {
              id: ChatId::new(),
              title: "New Chat".to_string(),
              created_at: now,
              updated_at: now,
              deleted_at: None,
              model_id: model_id.into(),
              provider: provider.into(),
              source: ChatSource::default(),
          }
      }
  
      /// Create a new chat with a specific source
      pub fn with_source(mut self, source: ChatSource) -> Self {
          self.source = source;
          self
      }
  
      /// Update the title
      pub fn set_title(&mut self, title: impl Into<String>) {
          self.title = title.into();
          self.updated_at = Utc::now();
      }
  
      /// Update the timestamp to now
      pub fn touch(&mut self) {
          self.updated_at = Utc::now();
      }
  
      /// Check if this chat is in the trash
      pub fn is_deleted(&self) -> bool {
          self.deleted_at.is_some()
      }
  
      /// Soft delete the chat
      pub fn soft_delete(&mut self) {
          self.deleted_at = Some(Utc::now());
      }
  
      /// Restore the chat from trash
      pub fn restore(&mut self) {
          self.deleted_at = None;
      }
  
      /// Generate a title from the first user message content
      pub fn generate_title_from_content(content: &str) -> String {
          let trimmed = content.trim();
          if trimmed.is_empty() {
              return "New Chat".to_string();
          }
  
          // Take first line or first ~50 chars
          let first_line = trimmed.lines().next().unwrap_or(trimmed);
          let truncated: String = first_line.chars().take(50).collect();
  
          if truncated.len() < first_line.len() {
              format!("{}...", truncated.trim())
          } else {
              truncated
          }
      }
  }
  
  impl Default for Chat {
      fn default() -> Self {
          Self::new("claude-3-5-sonnet", "anthropic")
      }
  }
  
  /// Image attachment for multimodal messages
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct ImageAttachment {
      /// Base64 encoded image data
      pub data: String,
  
      /// MIME type of the image (e.g., "image/png", "image/jpeg")
      pub media_type: String,
  }
  
  impl ImageAttachment {
      /// Create a new image attachment from base64 data
      pub fn new(data: String, media_type: String) -> Self {
          Self { data, media_type }
      }
  
      /// Create a PNG image attachment
      pub fn png(data: String) -> Self {
          Self::new(data, "image/png".to_string())
      }
  
      /// Create a JPEG image attachment
      pub fn jpeg(data: String) -> Self {
          Self::new(data, "image/jpeg".to_string())
      }
  
      /// Estimate size in bytes (base64 is ~4/3 of original)
      pub fn estimated_size(&self) -> usize {
          self.data.len() * 3 / 4
      }
  }
  
  /// A message in a chat conversation
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct Message {
      /// Unique identifier
      pub id: String,
  
      /// The chat this message belongs to
      pub chat_id: ChatId,
  
      /// Role of the message sender
      pub role: MessageRole,
  
      /// Message content
      pub content: String,
  
      /// When the message was created
      pub created_at: DateTime<Utc>,
  
      /// Token count for this message (if available)
      pub tokens_used: Option<u32>,
  
      /// Image attachments for multimodal messages
      #[serde(default, skip_serializing_if = "Vec::is_empty")]
      pub images: Vec<ImageAttachment>,
  }
  
  impl Message {
      /// Create a new message
      pub fn new(chat_id: ChatId, role: MessageRole, content: impl Into<String>) -> Self {
          Self {
              id: Uuid::new_v4().to_string(),
              chat_id,
              role,
              content: content.into(),
              created_at: Utc::now(),
              tokens_used: None,
              images: Vec::new(),
          }
      }
  
      /// Create a user message
      pub fn user(chat_id: ChatId, content: impl Into<String>) -> Self {
          Self::new(chat_id, MessageRole::User, content)
      }
  
      /// Create an assistant message
      pub fn assistant(chat_id: ChatId, content: impl Into<String>) -> Self {
          Self::new(chat_id, MessageRole::Assistant, content)
      }
  
      /// Create a system message
      pub fn system(chat_id: ChatId, content: impl Into<String>) -> Self {
          Self::new(chat_id, MessageRole::System, content)
      }
  
      /// Set the token count
      pub fn with_tokens(mut self, tokens: u32) -> Self {
          self.tokens_used = Some(tokens);
          self
      }
  
      /// Get a preview of the content (first ~100 chars)
      pub fn preview(&self) -> String {
          let chars: String = self.content.chars().take(100).collect();
          if chars.len() < self.content.len() {
              format!("{}...", chars.trim())
          } else {
              chars
          }
      }
  }
  
  #[cfg(test)]
  mod tests {
      use super::*;
  
      #[test]
      fn test_chat_id_creation() {
          let id = ChatId::new();
          assert!(!id.0.is_nil());
  
          let id2 = ChatId::new();
          assert_ne!(id, id2);
      }
  
      #[test]
      fn test_chat_id_parse() {
          let id = ChatId::new();
          let parsed = ChatId::parse(&id.as_str());
          assert!(parsed.is_some());
          assert_eq!(parsed.unwrap(), id);
  
          assert!(ChatId::parse("invalid").is_none());
      }
  
      #[test]
      fn test_chat_creation() {
          let chat = Chat::new("claude-3-opus", "anthropic");
          assert!(!chat.id.0.is_nil());
          assert_eq!(chat.title, "New Chat");
          assert_eq!(chat.model_id, "claude-3-opus");
          assert_eq!(chat.provider, "anthropic");
          assert!(!chat.is_deleted());
      }
  
      #[test]
      fn test_chat_soft_delete() {
          let mut chat = Chat::default();
          assert!(!chat.is_deleted());
  
          chat.soft_delete();
          assert!(chat.is_deleted());
  
          chat.restore();
          assert!(!chat.is_deleted());
      }
  
      #[test]
      fn test_generate_title() {
          assert_eq!(
              Chat::generate_title_from_content("Hello, how are you?"),
              "Hello, how are you?"
          );
  
          assert_eq!(Chat::generate_title_from_content(""), "New Chat");
  
          assert_eq!(Chat::generate_title_from_content("   "), "New Chat");
  
          let long_text = "This is a very long message that should be truncated to approximately fifty characters or so.";
          let title = Chat::generate_title_from_content(long_text);
          assert!(title.ends_with("..."));
          assert!(title.len() <= 56); // 50 chars + "..."
      }
  
      #[test]
      fn test_message_creation() {
          let chat_id = ChatId::new();
          let msg = Message::user(chat_id, "Hello!");
  
          assert_eq!(msg.chat_id, chat_id);
          assert_eq!(msg.role, MessageRole::User);
          assert_eq!(msg.content, "Hello!");
          assert!(msg.tokens_used.is_none());
      }
  
      #[test]
      fn test_message_with_tokens() {
          let chat_id = ChatId::new();
          let msg = Message::assistant(chat_id, "Response").with_tokens(150);
  
          assert_eq!(msg.role, MessageRole::Assistant);
          assert_eq!(msg.tokens_used, Some(150));
      }
  
      #[test]
      fn test_message_role_conversion() {
          assert_eq!(MessageRole::User.as_str(), "user");
          assert_eq!(MessageRole::Assistant.as_str(), "assistant");
          assert_eq!(MessageRole::System.as_str(), "system");
  
          assert_eq!(MessageRole::parse("user"), Some(MessageRole::User));
          assert_eq!(MessageRole::parse("USER"), Some(MessageRole::User));
          assert_eq!(MessageRole::parse("invalid"), None);
  
          // Test FromStr trait
          assert_eq!("user".parse::<MessageRole>(), Ok(MessageRole::User));
          assert!("invalid".parse::<MessageRole>().is_err());
      }
  }
  
  </file>
  
  <file path="src/ai/providers.rs">
  //! AI provider abstraction layer.
  //!
  //! This module provides a trait-based abstraction for AI providers, allowing
  //! Script Kit to work with multiple AI services (OpenAI, Anthropic, etc.) through
  //! a unified interface.
  //!
  //! # Architecture
  //!
  //! - `AiProvider` trait defines the interface all providers must implement
  //! - `ProviderRegistry` manages available providers based on detected API keys
  //! - Individual provider implementations (OpenAI, Anthropic, etc.) implement the trait
  //!
  
  use anyhow::{anyhow, Context, Result};
  use std::collections::HashMap;
  use std::io::{BufRead, BufReader};
  use std::sync::Arc;
  use std::time::Duration;
  
  use super::config::{default_models, DetectedKeys, ModelInfo, ProviderConfig};
  
  /// Extract a user-friendly error message from an API error response body.
  ///
  /// Tries to parse JSON error responses from various AI providers and extract
  /// the most useful error message for display to users.
  fn extract_api_error_message(body: &str) -> Option<String> {
      let parsed: serde_json::Value = serde_json::from_str(body).ok()?;
  
      // OpenAI/Vercel format: {"error": {"message": "...", "type": "..."}}
      if let Some(error) = parsed.get("error") {
          let message = error.get("message").and_then(|m| m.as_str());
          let error_type = error.get("type").and_then(|t| t.as_str());
  
          return match (message, error_type) {
              (Some(msg), Some(typ)) => Some(format!("{}: {}", typ, msg)),
              (Some(msg), None) => Some(msg.to_string()),
              _ => None,
          };
      }
  
      // Anthropic format: {"type": "error", "error": {"type": "...", "message": "..."}}
      if parsed.get("type").and_then(|t| t.as_str()) == Some("error") {
          if let Some(error) = parsed.get("error") {
              let message = error.get("message").and_then(|m| m.as_str());
              let error_type = error.get("type").and_then(|t| t.as_str());
  
              return match (message, error_type) {
                  (Some(msg), Some(typ)) => Some(format!("{}: {}", typ, msg)),
                  (Some(msg), None) => Some(msg.to_string()),
                  _ => None,
              };
          }
      }
  
      None
  }
  
  /// Handle HTTP response and return an error if status is not 2xx.
  ///
  /// Reads the error body and extracts a user-friendly message.
  fn handle_http_response(
      response: ureq::http::Response<ureq::Body>,
      provider_name: &str,
  ) -> Result<ureq::http::Response<ureq::Body>> {
      let status = response.status().as_u16();
  
      if (200..300).contains(&status) {
          return Ok(response);
      }
  
      // Read the error body
      let mut body = response.into_body();
      let body_str = body.read_to_string().unwrap_or_default();
  
      // Try to extract a meaningful error message
      let error_detail = extract_api_error_message(&body_str);
  
      // Build user-friendly error message based on status code
      let user_message = match status {
          401 => {
              let detail = error_detail.unwrap_or_else(|| "Invalid or missing API key".to_string());
              format!(
                  "{} authentication failed: {}",
                  provider_name,
                  simplify_auth_error(&detail)
              )
          }
          403 => {
              let detail = error_detail.unwrap_or_else(|| "Access denied".to_string());
              format!("{} access denied: {}", provider_name, detail)
          }
          404 => {
              let detail = error_detail.unwrap_or_else(|| "Model or endpoint not found".to_string());
              format!("{}: {}", provider_name, detail)
          }
          429 => {
              let detail = error_detail.unwrap_or_else(|| "Too many requests".to_string());
              format!("{} rate limited: {}", provider_name, detail)
          }
          500..=599 => {
              let detail = error_detail.unwrap_or_else(|| "Server error".to_string());
              format!("{} server error ({}): {}", provider_name, status, detail)
          }
          _ => {
              let detail = error_detail.unwrap_or_else(|| body_str.clone());
              format!("{} error (HTTP {}): {}", provider_name, status, detail)
          }
      };
  
      tracing::warn!(
          status = status,
          provider = provider_name,
          raw_error = %body_str,
          "API request failed"
      );
  
      Err(anyhow!(user_message))
  }
  
  /// Simplify verbose authentication error messages for display.
  fn simplify_auth_error(detail: &str) -> String {
      // Vercel OIDC errors are very verbose - simplify them
      if detail.contains("OIDC") || detail.contains("VERCEL_OIDC_TOKEN") {
          return "Vercel AI Gateway requires OIDC authentication. This is only available when running on Vercel. For local development, use direct API keys (SCRIPT_KIT_ANTHROPIC_API_KEY, SCRIPT_KIT_OPENAI_API_KEY).".to_string();
      }
      detail.to_string()
  }
  
  /// Default timeouts for API requests
  const CONNECT_TIMEOUT_SECS: u64 = 10;
  const READ_TIMEOUT_SECS: u64 = 60;
  
  /// Create a ureq::Agent with standard timeouts for API requests.
  fn create_agent() -> ureq::Agent {
      ureq::Agent::config_builder()
          .timeout_connect(Some(Duration::from_secs(CONNECT_TIMEOUT_SECS)))
          .timeout_recv_body(Some(Duration::from_secs(READ_TIMEOUT_SECS)))
          .build()
          .new_agent()
  }
  
  /// Parse SSE (Server-Sent Events) stream and process data lines.
  ///
  /// This helper handles:
  /// - CRLF line endings (trims trailing \r)
  /// - Multi-line data accumulation
  /// - [DONE] termination marker
  ///
  /// # Arguments
  ///
  /// * `reader` - A BufRead implementation (typically from response body)
  /// * `on_data` - Callback invoked for each complete data payload; returns true to continue, false to stop
  fn stream_sse_lines<R: BufRead>(
      reader: R,
      mut on_data: impl FnMut(&str) -> Result<bool>,
  ) -> Result<()> {
      let mut data_buf = String::new();
  
      for line in reader.lines() {
          let mut line = line.context("Failed to read SSE line")?;
          // Handle CRLF endings
          if line.ends_with('\r') {
              line.pop();
          }
  
          // Blank line: end of event
          if line.is_empty() {
              if data_buf.is_empty() {
                  continue;
              }
              if data_buf == "[DONE]" {
                  break;
              }
  
              // on_data returns true to continue, false to stop
              if !on_data(&data_buf)? {
                  break;
              }
              data_buf.clear();
              continue;
          }
  
          // Collect data lines
          if let Some(d) = line.strip_prefix("data: ") {
              if !data_buf.is_empty() {
                  data_buf.push('\n');
              }
              data_buf.push_str(d);
          }
      }
      Ok(())
  }
  
  /// Image data for multimodal API calls
  #[derive(Debug, Clone)]
  pub struct ProviderImage {
      /// Base64 encoded image data
      pub data: String,
      /// MIME type of the image (e.g., "image/png", "image/jpeg")
      pub media_type: String,
  }
  
  impl ProviderImage {
      /// Create a new image from base64 data
      pub fn new(data: String, media_type: String) -> Self {
          Self { data, media_type }
      }
  
      /// Create a PNG image
      pub fn png(data: String) -> Self {
          Self::new(data, "image/png".to_string())
      }
  
      /// Create a JPEG image
      pub fn jpeg(data: String) -> Self {
          Self::new(data, "image/jpeg".to_string())
      }
  }
  
  /// Message for AI provider API calls.
  #[derive(Debug, Clone)]
  pub struct ProviderMessage {
      /// Role of the message sender: "user", "assistant", or "system"
      pub role: String,
      /// Text content of the message
      pub content: String,
      /// Image attachments for multimodal messages
      pub images: Vec<ProviderImage>,
  }
  
  impl ProviderMessage {
      /// Create a new user message.
      pub fn user(content: impl Into<String>) -> Self {
          Self {
              role: "user".to_string(),
              content: content.into(),
              images: Vec::new(),
          }
      }
  
      /// Create a new user message with images.
      pub fn user_with_images(content: impl Into<String>, images: Vec<ProviderImage>) -> Self {
          Self {
              role: "user".to_string(),
              content: content.into(),
              images,
          }
      }
  
      /// Create a new assistant message.
      pub fn assistant(content: impl Into<String>) -> Self {
          Self {
              role: "assistant".to_string(),
              content: content.into(),
              images: Vec::new(),
          }
      }
  
      /// Create a new system message.
      pub fn system(content: impl Into<String>) -> Self {
          Self {
              role: "system".to_string(),
              content: content.into(),
              images: Vec::new(),
          }
      }
  
      /// Check if this message has images attached
      pub fn has_images(&self) -> bool {
          !self.images.is_empty()
      }
  }
  
  /// Callback type for streaming responses.
  pub type StreamCallback = Box<dyn Fn(String) + Send + Sync>;
  
  /// Trait defining the interface for AI providers.
  ///
  /// All AI providers (OpenAI, Anthropic, etc.) implement this trait to provide
  /// a consistent interface for the AI window.
  ///
  /// # Note on Async
  ///
  /// Currently methods are synchronous for simplicity. When real HTTP integration
  /// is added, these will become async using the `async_trait` crate.
  pub trait AiProvider: Send + Sync {
      /// Unique identifier for this provider (e.g., "openai", "anthropic").
      fn provider_id(&self) -> &str;
  
      /// Human-readable display name (e.g., "OpenAI", "Anthropic").
      fn display_name(&self) -> &str;
  
      /// Get the list of available models for this provider.
      fn available_models(&self) -> Vec<ModelInfo>;
  
      /// Send a message and get a response (non-streaming).
      ///
      /// # Arguments
      ///
      /// * `messages` - The conversation history
      /// * `model_id` - The model to use for generation
      ///
      /// # Returns
      ///
      /// The generated response text, or an error.
      fn send_message(&self, messages: &[ProviderMessage], model_id: &str) -> Result<String>;
  
      /// Send a message with streaming response.
      ///
      /// # Arguments
      ///
      /// * `messages` - The conversation history
      /// * `model_id` - The model to use for generation
      /// * `on_chunk` - Callback invoked for each chunk of the response
      ///
      /// # Returns
      ///
      /// Ok(()) on success, or an error.
      fn stream_message(
          &self,
          messages: &[ProviderMessage],
          model_id: &str,
          on_chunk: StreamCallback,
      ) -> Result<()>;
  }
  
  /// OpenAI provider implementation with real API calls.
  pub struct OpenAiProvider {
      config: ProviderConfig,
      agent: ureq::Agent,
  }
  
  /// OpenAI API constants
  const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";
  
  impl OpenAiProvider {
      /// Create a new OpenAI provider with the given API key.
      pub fn new(api_key: impl Into<String>) -> Self {
          Self {
              config: ProviderConfig::new("openai", "OpenAI", api_key),
              agent: create_agent(),
          }
      }
  
      /// Create with a custom base URL (for Azure OpenAI or proxies).
      pub fn with_base_url(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
          Self {
              config: ProviderConfig::new("openai", "OpenAI", api_key).with_base_url(base_url),
              agent: create_agent(),
          }
      }
  
      /// Get the API URL (uses custom base_url if set)
      fn api_url(&self) -> &str {
          self.config.base_url.as_deref().unwrap_or(OPENAI_API_URL)
      }
  
      /// Build the request body for OpenAI API
      ///
      /// Supports multimodal messages with images using OpenAI's content array format:
      /// ```json
      /// {
      ///   "role": "user",
      ///   "content": [
      ///     {"type": "image_url", "image_url": {"url": "data:image/png;base64,..."}},
      ///     {"type": "text", "text": "What's in this image?"}
      ///   ]
      /// }
      /// ```
      fn build_request_body(
          &self,
          messages: &[ProviderMessage],
          model_id: &str,
          stream: bool,
      ) -> serde_json::Value {
          let api_messages: Vec<serde_json::Value> = messages
              .iter()
              .map(|m| {
                  // If message has images, use content array format
                  if m.has_images() {
                      let mut content_blocks: Vec<serde_json::Value> = Vec::new();
  
                      // Add images (OpenAI uses data URL format)
                      for img in &m.images {
                          let data_url = format!("data:{};base64,{}", img.media_type, img.data);
                          content_blocks.push(serde_json::json!({
                              "type": "image_url",
                              "image_url": {
                                  "url": data_url
                              }
                          }));
                      }
  
                      // Add text content if not empty
                      if !m.content.is_empty() {
                          content_blocks.push(serde_json::json!({
                              "type": "text",
                              "text": m.content
                          }));
                      }
  
                      serde_json::json!({
                          "role": m.role,
                          "content": content_blocks
                      })
                  } else {
                      // Text-only message (simpler format)
                      serde_json::json!({
                          "role": m.role,
                          "content": m.content
                      })
                  }
              })
              .collect();
  
          serde_json::json!({
              "model": model_id,
              "stream": stream,
              "messages": api_messages
          })
      }
  
      /// Parse an SSE line and extract content delta (OpenAI format)
      fn parse_sse_line(line: &str) -> Option<String> {
          // SSE format: "data: {json}"
          if !line.starts_with("data: ") {
              return None;
          }
  
          let json_str = &line[6..]; // Skip "data: "
  
          // Check for stream end
          if json_str == "[DONE]" {
              return None;
          }
  
          // Parse the JSON
          let parsed: serde_json::Value = serde_json::from_str(json_str).ok()?;
  
          // OpenAI streaming format:
          // {"choices": [{"delta": {"content": "..."}}]}
          parsed
              .get("choices")?
              .as_array()?
              .first()?
              .get("delta")?
              .get("content")?
              .as_str()
              .map(|s| s.to_string())
      }
  }
  
  impl AiProvider for OpenAiProvider {
      fn provider_id(&self) -> &str {
          &self.config.provider_id
      }
  
      fn display_name(&self) -> &str {
          &self.config.display_name
      }
  
      fn available_models(&self) -> Vec<ModelInfo> {
          default_models::openai()
      }
  
      fn send_message(&self, messages: &[ProviderMessage], model_id: &str) -> Result<String> {
          let body = self.build_request_body(messages, model_id, false);
  
          tracing::debug!(
              model = model_id,
              message_count = messages.len(),
              "Sending non-streaming request to OpenAI"
          );
  
          let response = self
              .agent
              .post(self.api_url())
              .header("Content-Type", "application/json")
              .header(
                  "Authorization",
                  &format!("Bearer {}", self.config.api_key()),
              )
              .send_json(&body)
              .context("Network error connecting to OpenAI")?;
  
          // Check HTTP status and extract meaningful error if not 2xx
          let response = handle_http_response(response, "OpenAI")?;
  
          let response_json: serde_json::Value = response
              .into_body()
              .read_json()
              .context("Failed to parse OpenAI response")?;
  
          // Extract content from response
          // Response format: {"choices": [{"message": {"content": "..."}}]}
          let content = response_json
              .get("choices")
              .and_then(|c| c.as_array())
              .and_then(|arr| arr.first())
              .and_then(|choice| choice.get("message"))
              .and_then(|msg| msg.get("content"))
              .and_then(|c| c.as_str())
              .unwrap_or("")
              .to_string();
  
          tracing::debug!(
              content_len = content.len(),
              "Received non-streaming response from OpenAI"
          );
  
          Ok(content)
      }
  
      fn stream_message(
          &self,
          messages: &[ProviderMessage],
          model_id: &str,
          on_chunk: StreamCallback,
      ) -> Result<()> {
          let body = self.build_request_body(messages, model_id, true);
  
          tracing::debug!(
              model = model_id,
              message_count = messages.len(),
              "Starting streaming request to OpenAI"
          );
  
          let response = self
              .agent
              .post(self.api_url())
              .header("Content-Type", "application/json")
              .header(
                  "Authorization",
                  &format!("Bearer {}", self.config.api_key()),
              )
              .header("Accept", "text/event-stream")
              .send_json(&body)
              .context("Network error connecting to OpenAI")?;
  
          // Check HTTP status and extract meaningful error if not 2xx
          let response = handle_http_response(response, "OpenAI")?;
  
          // Read the SSE stream using the helper
          let reader = BufReader::new(response.into_body().into_reader());
  
          stream_sse_lines(reader, |data| {
              // Parse OpenAI streaming format
              if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data) {
                  if let Some(content) = parsed
                      .get("choices")
                      .and_then(|c| c.as_array())
                      .and_then(|arr| arr.first())
                      .and_then(|choice| choice.get("delta"))
                      .and_then(|delta| delta.get("content"))
                      .and_then(|c| c.as_str())
                  {
                      on_chunk(content.to_string());
                  }
              }
              Ok(true) // continue processing
          })?;
  
          tracing::debug!("Completed streaming response from OpenAI");
  
          Ok(())
      }
  }
  
  /// Anthropic provider implementation with real API calls.
  pub struct AnthropicProvider {
      config: ProviderConfig,
      agent: ureq::Agent,
  }
  
  /// Anthropic API constants
  const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
  const ANTHROPIC_VERSION: &str = "2023-06-01";
  const DEFAULT_MAX_TOKENS: u32 = 4096;
  
  impl AnthropicProvider {
      /// Create a new Anthropic provider with the given API key.
      pub fn new(api_key: impl Into<String>) -> Self {
          Self {
              config: ProviderConfig::new("anthropic", "Anthropic", api_key),
              agent: create_agent(),
          }
      }
  
      /// Create with a custom base URL (for proxies).
      pub fn with_base_url(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
          Self {
              config: ProviderConfig::new("anthropic", "Anthropic", api_key).with_base_url(base_url),
              agent: create_agent(),
          }
      }
  
      /// Get the API URL (uses custom base_url if set)
      fn api_url(&self) -> &str {
          self.config.base_url.as_deref().unwrap_or(ANTHROPIC_API_URL)
      }
  
      /// Build the request body for Anthropic API
      ///
      /// Supports multimodal messages with images using Anthropic's content array format:
      /// ```json
      /// {
      ///   "role": "user",
      ///   "content": [
      ///     {"type": "image", "source": {"type": "base64", "media_type": "image/png", "data": "..."}},
      ///     {"type": "text", "text": "What's in this image?"}
      ///   ]
      /// }
      /// ```
      fn build_request_body(
          &self,
          messages: &[ProviderMessage],
          model_id: &str,
          stream: bool,
      ) -> serde_json::Value {
          // Separate system message from conversation messages
          let system_msg = messages
              .iter()
              .find(|m| m.role == "system")
              .map(|m| m.content.clone());
  
          // Filter out system messages and build multimodal content for the messages array
          let api_messages: Vec<serde_json::Value> = messages
              .iter()
              .filter(|m| m.role != "system")
              .map(|m| {
                  // If message has images, use content array format
                  if m.has_images() {
                      let mut content_blocks: Vec<serde_json::Value> = Vec::new();
  
                      // Add images first (Anthropic recommends images before text)
                      for img in &m.images {
                          content_blocks.push(serde_json::json!({
                              "type": "image",
                              "source": {
                                  "type": "base64",
                                  "media_type": img.media_type,
                                  "data": img.data
                              }
                          }));
                      }
  
                      // Add text content if not empty
                      if !m.content.is_empty() {
                          content_blocks.push(serde_json::json!({
                              "type": "text",
                              "text": m.content
                          }));
                      }
  
                      serde_json::json!({
                          "role": m.role,
                          "content": content_blocks
                      })
                  } else {
                      // Text-only message (simpler format)
                      serde_json::json!({
                          "role": m.role,
                          "content": m.content
                      })
                  }
              })
              .collect();
  
          let mut body = serde_json::json!({
              "model": model_id,
              "max_tokens": DEFAULT_MAX_TOKENS,
              "stream": stream,
              "messages": api_messages
          });
  
          // Add system message if present
          if let Some(system) = system_msg {
              body["system"] = serde_json::Value::String(system);
          }
  
          body
      }
  
      /// Parse an SSE line and extract content delta
      fn parse_sse_line(line: &str) -> Option<String> {
          // SSE format: "data: {json}"
          if !line.starts_with("data: ") {
              return None;
          }
  
          let json_str = &line[6..]; // Skip "data: "
  
          // Check for stream end
          if json_str == "[DONE]" {
              return None;
          }
  
          // Parse the JSON
          let parsed: serde_json::Value = serde_json::from_str(json_str).ok()?;
  
          // Anthropic streaming format:
          // - content_block_delta events contain: {"type": "content_block_delta", "delta": {"type": "text_delta", "text": "..."}}
          if parsed.get("type")?.as_str()? == "content_block_delta" {
              if let Some(delta) = parsed.get("delta") {
                  if delta.get("type")?.as_str()? == "text_delta" {
                      return delta.get("text")?.as_str().map(|s| s.to_string());
                  }
              }
          }
  
          None
      }
  }
  
  impl AiProvider for AnthropicProvider {
      fn provider_id(&self) -> &str {
          &self.config.provider_id
      }
  
      fn display_name(&self) -> &str {
          &self.config.display_name
      }
  
      fn available_models(&self) -> Vec<ModelInfo> {
          default_models::anthropic()
      }
  
      fn send_message(&self, messages: &[ProviderMessage], model_id: &str) -> Result<String> {
          let body = self.build_request_body(messages, model_id, false);
  
          tracing::debug!(
              model = model_id,
              message_count = messages.len(),
              "Sending non-streaming request to Anthropic"
          );
  
          let response = self
              .agent
              .post(self.api_url())
              .header("Content-Type", "application/json")
              .header("x-api-key", self.config.api_key())
              .header("anthropic-version", ANTHROPIC_VERSION)
              .send_json(&body)
              .context("Network error connecting to Anthropic")?;
  
          // Check HTTP status and extract meaningful error if not 2xx
          let response = handle_http_response(response, "Anthropic")?;
  
          let response_json: serde_json::Value = response
              .into_body()
              .read_json()
              .context("Failed to parse Anthropic response")?;
  
          // Extract content from response - join ALL content blocks, not just first
          // Response format: {"content": [{"type": "text", "text": "..."}, ...], ...}
          let content = response_json
              .get("content")
              .and_then(|c| c.as_array())
              .map(|arr| {
                  arr.iter()
                      .filter_map(|b| b.get("text").and_then(|t| t.as_str()))
                      .collect::<Vec<_>>()
                      .join("")
              })
              .unwrap_or_default();
  
          tracing::debug!(
              content_len = content.len(),
              "Received non-streaming response from Anthropic"
          );
  
          Ok(content)
      }
  
      fn stream_message(
          &self,
          messages: &[ProviderMessage],
          model_id: &str,
          on_chunk: StreamCallback,
      ) -> Result<()> {
          let body = self.build_request_body(messages, model_id, true);
  
          tracing::debug!(
              model = model_id,
              message_count = messages.len(),
              "Starting streaming request to Anthropic"
          );
  
          let response = self
              .agent
              .post(self.api_url())
              .header("Content-Type", "application/json")
              .header("x-api-key", self.config.api_key())
              .header("anthropic-version", ANTHROPIC_VERSION)
              .header("Accept", "text/event-stream")
              .send_json(&body)
              .context("Network error connecting to Anthropic")?;
  
          // Check HTTP status and extract meaningful error if not 2xx
          let response = handle_http_response(response, "Anthropic")?;
  
          // Read the SSE stream using the helper
          let reader = BufReader::new(response.into_body().into_reader());
  
          stream_sse_lines(reader, |data| {
              // Parse Anthropic streaming format
              if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data) {
                  // Anthropic streaming format:
                  // content_block_delta events: {"type": "content_block_delta", "delta": {"type": "text_delta", "text": "..."}}
                  if parsed.get("type").and_then(|t| t.as_str()) == Some("content_block_delta") {
                      if let Some(delta) = parsed.get("delta") {
                          if delta.get("type").and_then(|t| t.as_str()) == Some("text_delta") {
                              if let Some(text) = delta.get("text").and_then(|t| t.as_str()) {
                                  on_chunk(text.to_string());
                              }
                          }
                      }
                  }
              }
              Ok(true) // continue processing
          })?;
  
          tracing::debug!("Completed streaming response from Anthropic");
  
          Ok(())
      }
  }
  
  /// Google (Gemini) provider implementation.
  pub struct GoogleProvider {
      config: ProviderConfig,
  }
  
  impl GoogleProvider {
      /// Create a new Google provider with the given API key.
      pub fn new(api_key: impl Into<String>) -> Self {
          Self {
              config: ProviderConfig::new("google", "Google", api_key),
          }
      }
  }
  
  impl AiProvider for GoogleProvider {
      fn provider_id(&self) -> &str {
          &self.config.provider_id
      }
  
      fn display_name(&self) -> &str {
          &self.config.display_name
      }
  
      fn available_models(&self) -> Vec<ModelInfo> {
          default_models::google()
      }
  
      fn send_message(&self, messages: &[ProviderMessage], model_id: &str) -> Result<String> {
          let last_user_msg = messages
              .iter()
              .rev()
              .find(|m| m.role == "user")
              .map(|m| m.content.as_str())
              .unwrap_or("(no message)");
  
          Ok(format!(
              "[Mock Google Response]\nModel: {}\nProvider: {}\n\nI received your message: \"{}\"",
              model_id,
              self.display_name(),
              last_user_msg
          ))
      }
  
      fn stream_message(
          &self,
          messages: &[ProviderMessage],
          model_id: &str,
          on_chunk: StreamCallback,
      ) -> Result<()> {
          let response = self.send_message(messages, model_id)?;
  
          for word in response.split_whitespace() {
              on_chunk(format!("{} ", word));
          }
  
          Ok(())
      }
  }
  
  /// Groq provider implementation.
  pub struct GroqProvider {
      config: ProviderConfig,
  }
  
  impl GroqProvider {
      /// Create a new Groq provider with the given API key.
      pub fn new(api_key: impl Into<String>) -> Self {
          Self {
              config: ProviderConfig::new("groq", "Groq", api_key),
          }
      }
  }
  
  impl AiProvider for GroqProvider {
      fn provider_id(&self) -> &str {
          &self.config.provider_id
      }
  
      fn display_name(&self) -> &str {
          &self.config.display_name
      }
  
      fn available_models(&self) -> Vec<ModelInfo> {
          default_models::groq()
      }
  
      fn send_message(&self, messages: &[ProviderMessage], model_id: &str) -> Result<String> {
          let last_user_msg = messages
              .iter()
              .rev()
              .find(|m| m.role == "user")
              .map(|m| m.content.as_str())
              .unwrap_or("(no message)");
  
          Ok(format!(
              "[Mock Groq Response]\nModel: {}\nProvider: {}\n\nI received your message: \"{}\"",
              model_id,
              self.display_name(),
              last_user_msg
          ))
      }
  
      fn stream_message(
          &self,
          messages: &[ProviderMessage],
          model_id: &str,
          on_chunk: StreamCallback,
      ) -> Result<()> {
          let response = self.send_message(messages, model_id)?;
  
          for word in response.split_whitespace() {
              on_chunk(format!("{} ", word));
          }
  
          Ok(())
      }
  }
  
  /// Vercel AI Gateway URL
  const VERCEL_GATEWAY_URL: &str = "https://ai-gateway.vercel.sh/v1";
  
  /// Vercel AI Gateway provider implementation.
  ///
  /// Routes requests through Vercel's AI Gateway, which supports multiple providers
  /// through namespaced model IDs (e.g., "openai/gpt-4o", "anthropic/claude-sonnet-4.5").
  pub struct VercelGatewayProvider {
      config: ProviderConfig,
      agent: ureq::Agent,
  }
  
  impl VercelGatewayProvider {
      /// Create a new Vercel Gateway provider with the given API key.
      pub fn new(api_key: impl Into<String>) -> Self {
          Self {
              config: ProviderConfig::new("vercel", "Vercel AI Gateway", api_key),
              agent: create_agent(),
          }
      }
  
      /// Get the chat completions API URL
      fn api_url(&self) -> String {
          format!("{}/chat/completions", VERCEL_GATEWAY_URL)
      }
  
      /// Normalize a model ID to include provider prefix if missing.
      ///
      /// Vercel Gateway expects namespaced model IDs like "openai/gpt-4o".
      /// If no prefix is provided, defaults to "openai/".
      fn normalize_model_id(model_id: &str) -> String {
          if model_id.contains('/') {
              model_id.to_string()
          } else {
              format!("openai/{}", model_id)
          }
      }
  
      /// Build the request body for Vercel Gateway (OpenAI-compatible format)
      ///
      /// Supports multimodal messages with images using OpenAI's content array format:
      /// ```json
      /// {
      ///   "role": "user",
      ///   "content": [
      ///     {"type": "image_url", "image_url": {"url": "data:image/png;base64,..."}},
      ///     {"type": "text", "text": "What's in this image?"}
      ///   ]
      /// }
      /// ```
      fn build_request_body(
          &self,
          messages: &[ProviderMessage],
          model_id: &str,
          stream: bool,
      ) -> serde_json::Value {
          let api_messages: Vec<serde_json::Value> = messages
              .iter()
              .map(|m| {
                  // If message has images, use content array format
                  if m.has_images() {
                      let mut content_blocks: Vec<serde_json::Value> = Vec::new();
  
                      // Add images (OpenAI-compatible data URL format)
                      for img in &m.images {
                          let data_url = format!("data:{};base64,{}", img.media_type, img.data);
                          content_blocks.push(serde_json::json!({
                              "type": "image_url",
                              "image_url": {
                                  "url": data_url
                              }
                          }));
                      }
  
                      // Add text content if not empty
                      if !m.content.is_empty() {
                          content_blocks.push(serde_json::json!({
                              "type": "text",
                              "text": m.content
                          }));
                      }
  
                      serde_json::json!({
                          "role": m.role,
                          "content": content_blocks
                      })
                  } else {
                      // Text-only message (simpler format)
                      serde_json::json!({
                          "role": m.role,
                          "content": m.content
                      })
                  }
              })
              .collect();
  
          serde_json::json!({
              "model": Self::normalize_model_id(model_id),
              "stream": stream,
              "messages": api_messages
          })
      }
  }
  
  impl AiProvider for VercelGatewayProvider {
      fn provider_id(&self) -> &str {
          &self.config.provider_id
      }
  
      fn display_name(&self) -> &str {
          &self.config.display_name
      }
  
      fn available_models(&self) -> Vec<ModelInfo> {
          // Vercel Gateway supports various models from different providers.
          // These are curated defaults; the full list is available via GET https://ai-gateway.vercel.sh/v1/models
          // Model IDs are namespaced: provider/model (e.g., "openai/gpt-4o", "anthropic/claude-haiku-4.5")
          // These MUST match the exact IDs from https://ai-gateway.vercel.sh/v1/models
          // NOTE: The FIRST model in this list is the default model for new chats
          vec![
              // Default model: Claude Haiku 4.5 (fast, cheap, good quality)
              ModelInfo::new(
                  "anthropic/claude-haiku-4.5",
                  "Claude Haiku 4.5 (via Vercel)",
                  "vercel",
                  true,
                  200000,
              ),
              ModelInfo::new(
                  "anthropic/claude-3.5-haiku",
                  "Claude 3.5 Haiku (via Vercel)",
                  "vercel",
                  true,
                  200000,
              ),
              // Other Anthropic models
              ModelInfo::new(
                  "anthropic/claude-sonnet-4.5",
                  "Claude Sonnet 4.5 (via Vercel)",
                  "vercel",
                  true,
                  200000,
              ),
              ModelInfo::new(
                  "anthropic/claude-opus-4.5",
                  "Claude Opus 4.5 (via Vercel)",
                  "vercel",
                  true,
                  200000,
              ),
              ModelInfo::new(
                  "anthropic/claude-sonnet-4",
                  "Claude Sonnet 4 (via Vercel)",
                  "vercel",
                  true,
                  200000,
              ),
              // OpenAI models
              ModelInfo::new("openai/gpt-5", "GPT-5 (via Vercel)", "vercel", true, 400000),
              ModelInfo::new(
                  "openai/gpt-5-mini",
                  "GPT-5 mini (via Vercel)",
                  "vercel",
                  true,
                  400000,
              ),
              ModelInfo::new(
                  "openai/gpt-4o",
                  "GPT-4o (via Vercel)",
                  "vercel",
                  true,
                  128000,
              ),
              ModelInfo::new("openai/o3", "o3 (via Vercel)", "vercel", true, 200000),
              ModelInfo::new(
                  "openai/gpt-4o-mini",
                  "GPT-4o mini (via Vercel)",
                  "vercel",
                  true,
                  128000,
              ),
              ModelInfo::new("openai/o3", "o3 (via Vercel)", "vercel", true, 200000),
              ModelInfo::new(
                  "openai/o3-mini",
                  "o3 mini (via Vercel)",
                  "vercel",
                  true,
                  200000,
              ),
              // Google models
              ModelInfo::new(
                  "google/gemini-2.5-pro",
                  "Gemini 2.5 Pro (via Vercel)",
                  "vercel",
                  true,
                  1048576,
              ),
              ModelInfo::new(
                  "google/gemini-2.5-flash",
                  "Gemini 2.5 Flash (via Vercel)",
                  "vercel",
                  true,
                  1048576,
              ),
              // xAI models
              ModelInfo::new("xai/grok-3", "Grok 3 (via Vercel)", "vercel", true, 131072),
              // DeepSeek models
              ModelInfo::new(
                  "deepseek/deepseek-r1",
                  "DeepSeek R1 (via Vercel)",
                  "vercel",
                  true,
                  160000,
              ),
          ]
      }
  
      fn send_message(&self, messages: &[ProviderMessage], model_id: &str) -> Result<String> {
          let body = self.build_request_body(messages, model_id, false);
  
          tracing::debug!(
              model = model_id,
              normalized_model = Self::normalize_model_id(model_id),
              message_count = messages.len(),
              "Sending non-streaming request to Vercel Gateway"
          );
  
          let response = self
              .agent
              .post(&self.api_url())
              .header("Content-Type", "application/json")
              .header(
                  "Authorization",
                  &format!("Bearer {}", self.config.api_key()),
              )
              .send_json(&body)
              .context("Network error connecting to Vercel AI Gateway")?;
  
          // Check HTTP status and extract meaningful error if not 2xx
          let response = handle_http_response(response, "Vercel AI Gateway")?;
  
          let response_json: serde_json::Value = response
              .into_body()
              .read_json()
              .context("Failed to parse Vercel Gateway response")?;
  
          // OpenAI-compatible response format
          let content = response_json
              .get("choices")
              .and_then(|c| c.as_array())
              .and_then(|arr| arr.first())
              .and_then(|choice| choice.get("message"))
              .and_then(|msg| msg.get("content"))
              .and_then(|c| c.as_str())
              .unwrap_or("")
              .to_string();
  
          tracing::debug!(
              content_len = content.len(),
              "Received non-streaming response from Vercel Gateway"
          );
  
          Ok(content)
      }
  
      fn stream_message(
          &self,
          messages: &[ProviderMessage],
          model_id: &str,
          on_chunk: StreamCallback,
      ) -> Result<()> {
          let body = self.build_request_body(messages, model_id, true);
  
          tracing::debug!(
              model = model_id,
              normalized_model = Self::normalize_model_id(model_id),
              message_count = messages.len(),
              "Starting streaming request to Vercel Gateway"
          );
  
          let response = self
              .agent
              .post(&self.api_url())
              .header("Content-Type", "application/json")
              .header(
                  "Authorization",
                  &format!("Bearer {}", self.config.api_key()),
              )
              .header("Accept", "text/event-stream")
              .send_json(&body)
              .context("Network error connecting to Vercel AI Gateway")?;
  
          // Check HTTP status and extract meaningful error if not 2xx
          let response = handle_http_response(response, "Vercel AI Gateway")?;
  
          // Read the SSE stream using the helper (OpenAI-compatible format)
          let reader = BufReader::new(response.into_body().into_reader());
  
          stream_sse_lines(reader, |data| {
              // Parse OpenAI-compatible streaming format
              if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data) {
                  if let Some(content) = parsed
                      .get("choices")
                      .and_then(|c| c.as_array())
                      .and_then(|arr| arr.first())
                      .and_then(|choice| choice.get("delta"))
                      .and_then(|delta| delta.get("content"))
                      .and_then(|c| c.as_str())
                  {
                      on_chunk(content.to_string());
                  }
              }
              Ok(true) // continue processing
          })?;
  
          tracing::debug!("Completed streaming response from Vercel Gateway");
  
          Ok(())
      }
  }
  
  /// Registry of available AI providers.
  ///
  /// The registry automatically discovers available providers based on
  /// environment variables and provides a unified interface to access them.
  #[derive(Clone)]
  pub struct ProviderRegistry {
      providers: HashMap<String, Arc<dyn AiProvider>>,
  }
  
  impl ProviderRegistry {
      /// Create an empty registry.
      pub fn new() -> Self {
          Self {
              providers: HashMap::new(),
          }
      }
  
      /// Create a registry populated from environment variables.
      ///
      /// Scans for `SCRIPT_KIT_*_API_KEY` environment variables and
      /// creates providers for each detected key.
      pub fn from_environment() -> Self {
          let keys = DetectedKeys::from_environment();
          let mut registry = Self::new();
  
          if let Some(key) = keys.openai {
              registry.register(Arc::new(OpenAiProvider::new(key)));
          }
  
          if let Some(key) = keys.anthropic {
              registry.register(Arc::new(AnthropicProvider::new(key)));
          }
  
          if let Some(key) = keys.google {
              registry.register(Arc::new(GoogleProvider::new(key)));
          }
  
          if let Some(key) = keys.groq {
              registry.register(Arc::new(GroqProvider::new(key)));
          }
  
          if let Some(key) = keys.vercel {
              registry.register(Arc::new(VercelGatewayProvider::new(key)));
          }
  
          // Log which providers are available (without exposing keys)
          let available: Vec<_> = registry.providers.keys().collect();
          if !available.is_empty() {
              tracing::info!(
                  providers = ?available,
                  "AI providers initialized from environment"
              );
          } else {
              tracing::debug!("No AI provider API keys found in environment");
          }
  
          registry
      }
  
      /// Register a provider with the registry.
      pub fn register(&mut self, provider: Arc<dyn AiProvider>) {
          self.providers
              .insert(provider.provider_id().to_string(), provider);
      }
  
      /// Check if any providers are available.
      pub fn has_any_provider(&self) -> bool {
          !self.providers.is_empty()
      }
  
      /// Get a provider by ID.
      pub fn get_provider(&self, id: &str) -> Option<&Arc<dyn AiProvider>> {
          self.providers.get(id)
      }
  
      /// Get all registered provider IDs.
      pub fn provider_ids(&self) -> Vec<&str> {
          self.providers.keys().map(|s| s.as_str()).collect()
      }
  
      /// Get all available models from all providers.
      pub fn get_all_models(&self) -> Vec<ModelInfo> {
          let mut models = Vec::new();
          for provider in self.providers.values() {
              models.extend(provider.available_models());
          }
          models
      }
  
      /// Get models for a specific provider.
      pub fn get_models_for_provider(&self, provider_id: &str) -> Vec<ModelInfo> {
          self.providers
              .get(provider_id)
              .map(|p| p.available_models())
              .unwrap_or_default()
      }
  
      /// Find the provider that owns a specific model.
      pub fn find_provider_for_model(&self, model_id: &str) -> Option<&Arc<dyn AiProvider>> {
          self.providers
              .values()
              .find(|provider| provider.available_models().iter().any(|m| m.id == model_id))
      }
  }
  
  impl Default for ProviderRegistry {
      fn default() -> Self {
          Self::new()
      }
  }
  
  #[cfg(test)]
  mod tests {
      use super::*;
  
      #[test]
      fn test_provider_message_constructors() {
          let user = ProviderMessage::user("Hello");
          assert_eq!(user.role, "user");
          assert_eq!(user.content, "Hello");
  
          let assistant = ProviderMessage::assistant("Hi there");
          assert_eq!(assistant.role, "assistant");
          assert_eq!(assistant.content, "Hi there");
  
          let system = ProviderMessage::system("You are helpful");
          assert_eq!(system.role, "system");
          assert_eq!(system.content, "You are helpful");
      }
  
      #[test]
      fn test_openai_provider() {
          let provider = OpenAiProvider::new("test-key");
          assert_eq!(provider.provider_id(), "openai");
          assert_eq!(provider.display_name(), "OpenAI");
  
          let models = provider.available_models();
          assert!(!models.is_empty());
          assert!(models.iter().any(|m| m.id == "gpt-4o"));
      }
  
      #[test]
      fn test_anthropic_provider() {
          let provider = AnthropicProvider::new("test-key");
          assert_eq!(provider.provider_id(), "anthropic");
          assert_eq!(provider.display_name(), "Anthropic");
  
          let models = provider.available_models();
          assert!(!models.is_empty());
      }
  
      /// Test send_message with real API calls (requires API key)
      /// Run with: cargo test --features system-tests test_send_message_real -- --ignored
      #[test]
      #[ignore = "Requires real API key - run with SCRIPT_KIT_OPENAI_API_KEY set"]
      fn test_send_message_real() {
          let api_key = std::env::var("SCRIPT_KIT_OPENAI_API_KEY")
              .expect("SCRIPT_KIT_OPENAI_API_KEY must be set for this test");
          let provider = OpenAiProvider::new(api_key);
          let messages = vec![
              ProviderMessage::system("You are helpful"),
              ProviderMessage::user("Say hello"),
          ];
  
          let response = provider.send_message(&messages, "gpt-4o-mini").unwrap();
          assert!(!response.is_empty());
      }
  
      /// Test stream_message with real API calls (requires API key)
      /// Run with: cargo test --features system-tests test_stream_message_real -- --ignored
      #[test]
      #[ignore = "Requires real API key - run with SCRIPT_KIT_OPENAI_API_KEY set"]
      fn test_stream_message_real() {
          let api_key = std::env::var("SCRIPT_KIT_OPENAI_API_KEY")
              .expect("SCRIPT_KIT_OPENAI_API_KEY must be set for this test");
          let provider = OpenAiProvider::new(api_key);
          let messages = vec![ProviderMessage::user("Say hello")];
  
          let chunks = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
          let chunks_clone = chunks.clone();
  
          provider
              .stream_message(
                  &messages,
                  "gpt-4o-mini",
                  Box::new(move |chunk| {
                      chunks_clone.lock().unwrap().push(chunk);
                  }),
              )
              .unwrap();
  
          let collected = chunks.lock().unwrap();
          assert!(!collected.is_empty());
      }
  
      #[test]
      fn test_request_body_construction() {
          let provider = OpenAiProvider::new("test-key");
          let messages = vec![
              ProviderMessage::system("You are helpful"),
              ProviderMessage::user("Hello"),
          ];
  
          let body = provider.build_request_body(&messages, "gpt-4o", false);
  
          assert_eq!(body["model"], "gpt-4o");
          assert_eq!(body["stream"], false);
          assert!(body["messages"].is_array());
          assert_eq!(body["messages"].as_array().unwrap().len(), 2);
      }
  
      #[test]
      fn test_anthropic_request_body_construction() {
          let provider = AnthropicProvider::new("test-key");
          let messages = vec![
              ProviderMessage::system("You are helpful"),
              ProviderMessage::user("Hello"),
          ];
  
          let body = provider.build_request_body(&messages, "claude-3-5-sonnet-20241022", true);
  
          assert_eq!(body["model"], "claude-3-5-sonnet-20241022");
          assert_eq!(body["stream"], true);
          assert_eq!(body["system"], "You are helpful");
          // Messages array should NOT contain the system message
          assert_eq!(body["messages"].as_array().unwrap().len(), 1);
      }
  
      #[test]
      fn test_sse_parsing_openai() {
          // Test OpenAI SSE format
          let line = r#"data: {"choices": [{"delta": {"content": "Hello"}}]}"#;
          let result = OpenAiProvider::parse_sse_line(line);
          assert_eq!(result, Some("Hello".to_string()));
  
          // Empty delta
          let line = r#"data: {"choices": [{"delta": {}}]}"#;
          let result = OpenAiProvider::parse_sse_line(line);
          assert_eq!(result, None);
  
          // [DONE] marker
          let line = "data: [DONE]";
          let result = OpenAiProvider::parse_sse_line(line);
          assert_eq!(result, None);
  
          // Non-data line
          let line = "event: message";
          let result = OpenAiProvider::parse_sse_line(line);
          assert_eq!(result, None);
      }
  
      #[test]
      fn test_sse_parsing_anthropic() {
          // Test Anthropic SSE format
          let line = r#"data: {"type": "content_block_delta", "delta": {"type": "text_delta", "text": "World"}}"#;
          let result = AnthropicProvider::parse_sse_line(line);
          assert_eq!(result, Some("World".to_string()));
  
          // Other event types should be ignored
          let line = r#"data: {"type": "message_start", "message": {}}"#;
          let result = AnthropicProvider::parse_sse_line(line);
          assert_eq!(result, None);
  
          // [DONE] marker
          let line = "data: [DONE]";
          let result = AnthropicProvider::parse_sse_line(line);
          assert_eq!(result, None);
      }
  
      #[test]
      fn test_registry_empty() {
          let registry = ProviderRegistry::new();
          assert!(!registry.has_any_provider());
          assert!(registry.get_all_models().is_empty());
      }
  
      #[test]
      fn test_registry_register() {
          let mut registry = ProviderRegistry::new();
          registry.register(Arc::new(OpenAiProvider::new("test-key")));
  
          assert!(registry.has_any_provider());
          assert!(registry.get_provider("openai").is_some());
          assert!(registry.get_provider("anthropic").is_none());
      }
  
      #[test]
      fn test_registry_get_all_models() {
          let mut registry = ProviderRegistry::new();
          registry.register(Arc::new(OpenAiProvider::new("test")));
          registry.register(Arc::new(AnthropicProvider::new("test")));
  
          let models = registry.get_all_models();
          assert!(models.iter().any(|m| m.provider == "openai"));
          assert!(models.iter().any(|m| m.provider == "anthropic"));
      }
  
      #[test]
      fn test_registry_find_provider_for_model() {
          let mut registry = ProviderRegistry::new();
          registry.register(Arc::new(OpenAiProvider::new("test")));
          registry.register(Arc::new(AnthropicProvider::new("test")));
  
          let provider = registry.find_provider_for_model("gpt-4o");
          assert!(provider.is_some());
          assert_eq!(provider.unwrap().provider_id(), "openai");
  
          let provider = registry.find_provider_for_model("claude-3-5-sonnet-20241022");
          assert!(provider.is_some());
          assert_eq!(provider.unwrap().provider_id(), "anthropic");
  
          let provider = registry.find_provider_for_model("nonexistent");
          assert!(provider.is_none());
      }
  
      #[test]
      fn test_stream_sse_lines_basic() {
          use std::io::Cursor;
  
          // Simulate SSE stream with basic data
          let sse_data = "data: hello\n\ndata: world\n\n";
          let reader = Cursor::new(sse_data);
  
          let mut collected = Vec::new();
          stream_sse_lines(reader, |data| {
              collected.push(data.to_string());
              Ok(true)
          })
          .unwrap();
  
          assert_eq!(collected, vec!["hello", "world"]);
      }
  
      #[test]
      fn test_stream_sse_lines_done_marker() {
          use std::io::Cursor;
  
          // [DONE] should stop processing
          let sse_data = "data: first\n\ndata: [DONE]\n\ndata: should_not_see\n\n";
          let reader = Cursor::new(sse_data);
  
          let mut collected = Vec::new();
          stream_sse_lines(reader, |data| {
              collected.push(data.to_string());
              Ok(true)
          })
          .unwrap();
  
          assert_eq!(collected, vec!["first"]);
      }
  
      #[test]
      fn test_stream_sse_lines_crlf() {
          use std::io::Cursor;
  
          // Should handle CRLF line endings
          let sse_data = "data: with_cr\r\n\r\n";
          let reader = Cursor::new(sse_data);
  
          let mut collected = Vec::new();
          stream_sse_lines(reader, |data| {
              collected.push(data.to_string());
              Ok(true)
          })
          .unwrap();
  
          assert_eq!(collected, vec!["with_cr"]);
      }
  
      #[test]
      fn test_stream_sse_lines_callback_stop() {
          use std::io::Cursor;
  
          // Callback returning false should stop processing
          let sse_data = "data: first\n\ndata: second\n\ndata: third\n\n";
          let reader = Cursor::new(sse_data);
  
          let mut collected = Vec::new();
          stream_sse_lines(reader, |data| {
              collected.push(data.to_string());
              Ok(collected.len() < 2) // Stop after 2 items
          })
          .unwrap();
  
          assert_eq!(collected, vec!["first", "second"]);
      }
  
      #[test]
      fn test_vercel_provider() {
          let provider = VercelGatewayProvider::new("test-key");
          assert_eq!(provider.provider_id(), "vercel");
          assert_eq!(provider.display_name(), "Vercel AI Gateway");
  
          let models = provider.available_models();
          assert!(!models.is_empty());
          assert!(models.iter().any(|m| m.id.contains("openai/")));
          assert!(models.iter().any(|m| m.id.contains("anthropic/")));
      }
  
      #[test]
      fn test_vercel_normalize_model_id() {
          // Already prefixed - should not change
          assert_eq!(
              VercelGatewayProvider::normalize_model_id("openai/gpt-4o"),
              "openai/gpt-4o"
          );
          assert_eq!(
              VercelGatewayProvider::normalize_model_id("anthropic/claude-haiku-4.5"),
              "anthropic/claude-haiku-4.5"
          );
  
          // Not prefixed - should add openai/
          assert_eq!(
              VercelGatewayProvider::normalize_model_id("gpt-4o"),
              "openai/gpt-4o"
          );
          assert_eq!(
              VercelGatewayProvider::normalize_model_id("gpt-4o-mini"),
              "openai/gpt-4o-mini"
          );
      }
  
      #[test]
      fn test_vercel_request_body_normalizes_model() {
          let provider = VercelGatewayProvider::new("test-key");
          let messages = vec![ProviderMessage::user("Hello")];
  
          // Test with unprefixed model
          let body = provider.build_request_body(&messages, "gpt-4o", false);
          assert_eq!(body["model"], "openai/gpt-4o");
  
          // Test with prefixed model
          let body = provider.build_request_body(&messages, "anthropic/claude-haiku-4.5", true);
          assert_eq!(body["model"], "anthropic/claude-haiku-4.5");
      }
  
      #[test]
      fn test_anthropic_api_url_respects_base_url() {
          // Default URL
          let provider = AnthropicProvider::new("test-key");
          assert_eq!(provider.api_url(), ANTHROPIC_API_URL);
  
          // Custom base URL
          let provider = AnthropicProvider::with_base_url("test-key", "https://custom.proxy.com/v1");
          assert_eq!(provider.api_url(), "https://custom.proxy.com/v1");
      }
  
      #[test]
      fn test_registry_with_vercel() {
          let mut registry = ProviderRegistry::new();
          registry.register(Arc::new(VercelGatewayProvider::new("test")));
  
          assert!(registry.has_any_provider());
          assert!(registry.get_provider("vercel").is_some());
  
          let models = registry.get_all_models();
          assert!(models.iter().any(|m| m.provider == "vercel"));
      }
  
      #[test]
      fn test_extract_api_error_message_openai_format() {
          // OpenAI/Vercel format
          let body = r#"{"error": {"message": "Invalid API key", "type": "authentication_error"}}"#;
          let result = extract_api_error_message(body);
          assert_eq!(
              result,
              Some("authentication_error: Invalid API key".to_string())
          );
  
          // Missing type
          let body = r#"{"error": {"message": "Something went wrong"}}"#;
          let result = extract_api_error_message(body);
          assert_eq!(result, Some("Something went wrong".to_string()));
      }
  
      #[test]
      fn test_extract_api_error_message_anthropic_format() {
          // Anthropic format
          let body = r#"{"type": "error", "error": {"type": "invalid_request_error", "message": "Invalid model"}}"#;
          let result = extract_api_error_message(body);
          assert_eq!(
              result,
              Some("invalid_request_error: Invalid model".to_string())
          );
      }
  
      #[test]
      fn test_extract_api_error_message_invalid_json() {
          let result = extract_api_error_message("not json");
          assert_eq!(result, None);
  
          let result = extract_api_error_message(r#"{"foo": "bar"}"#);
          assert_eq!(result, None);
      }
  
      #[test]
      fn test_simplify_auth_error_vercel_oidc() {
          let detail = "Error verifying OIDC token\nThe AI Gateway OIDC authentication token...";
          let result = simplify_auth_error(detail);
          assert!(result.contains("Vercel AI Gateway requires OIDC authentication"));
          assert!(result.contains("local development"));
      }
  
      #[test]
      fn test_simplify_auth_error_passthrough() {
          let detail = "Invalid API key provided";
          let result = simplify_auth_error(detail);
          assert_eq!(result, detail);
      }
  }
  
  </file>
  
  <file path="src/ai/mod.rs">
  //! AI Chat Module
  //!
  //! This module provides the data layer for the AI chat window feature.
  //! It includes data models, SQLite storage with FTS5 search support,
  //! and provider abstraction for BYOK (Bring Your Own Key) AI integration.
  //!
  //! # Architecture
  //!
  //! ```text
  //! src/ai/
  //! ├── mod.rs       - Module exports and documentation
  //! ├── model.rs     - Data models (Chat, Message, ChatId, MessageRole)
  //! ├── storage.rs   - SQLite persistence layer
  //! ├── config.rs    - Environment variable detection and model configuration
  //! └── providers.rs - Provider trait and implementations (OpenAI, Anthropic, etc.)
  //! ```
  //!
  //! # Database Location
  //!
  //! The AI chats database is stored at `~/.scriptkit/ai-chats.db`.
  //!
  //!
  //! # Features
  //!
  //! - **BYOK (Bring Your Own Key)**: Stores model and provider info per chat
  //! - **FTS5 Search**: Full-text search across chat titles and message content
  //! - **Soft Delete**: Chats can be moved to trash and restored
  //! - **Token Tracking**: Optional token usage tracking per message
  //! - **Auto-Pruning**: Old deleted chats can be automatically pruned
  
  // Allow unused for now - these are for future use by other modules
  #![allow(unused_imports)]
  #![allow(dead_code)]
  
  pub mod config;
  pub mod model;
  pub mod providers;
  pub mod sdk_handlers;
  pub mod storage;
  pub mod window;
  
  // Re-export commonly used types
  pub use model::{Chat, ChatId, ChatSource, Message, MessageRole};
  pub use storage::{
      clear_all_chats, create_chat, delete_chat, get_all_chats, get_chat, get_chat_messages,
      get_deleted_chats, init_ai_db, insert_mock_data, restore_chat, save_message, search_chats,
      update_chat_title,
  };
  
  // Re-export provider types
  pub use config::{DetectedKeys, ModelInfo, ProviderConfig};
  pub use providers::{AiProvider, ProviderMessage, ProviderRegistry};
  
  // Re-export window functions
  pub use window::{
      close_ai_window, is_ai_window, is_ai_window_open, open_ai_window, open_ai_window_with_chat,
      set_ai_input, set_ai_input_with_image, set_ai_search, show_ai_command_bar, simulate_ai_key,
  };
  
  // Re-export SDK handler
  pub use sdk_handlers::try_handle_ai_message;
  
  </file>
  
  <file path="src/ai/config.rs">
  //! AI provider configuration and environment variable detection.
  //!
  //! This module handles automatic discovery of AI provider API keys from environment
  //! variables and the system keyring using the `SCRIPT_KIT_*_API_KEY` pattern for security.
  //!
  //! # Key Detection Order
  //!
  //! API keys are detected in the following order (first found wins):
  //! 1. Environment variable (`SCRIPT_KIT_*_API_KEY`)
  //! 2. System keyring (macOS Keychain via `com.scriptkit.env` service)
  //!
  //! This allows users to either set environment variables in their shell profile
  //! or use the built-in "Configure API Key" commands which store in the keyring.
  //!
  //! # Environment Variable Pattern
  //!
  //! API keys are detected with the `SCRIPT_KIT_` prefix:
  //! - `SCRIPT_KIT_OPENAI_API_KEY` -> OpenAI provider
  //! - `SCRIPT_KIT_ANTHROPIC_API_KEY` -> Anthropic provider
  //! - `SCRIPT_KIT_VERCEL_API_KEY` -> Vercel AI Gateway
  //!
  //! This prefix ensures users explicitly configure keys for Script Kit,
  //! rather than accidentally exposing keys from other applications.
  
  use std::env;
  
  use crate::secrets::get_secret;
  
  /// Represents a detected AI provider configuration.
  #[derive(Clone)]
  pub struct ProviderConfig {
      /// Unique identifier for the provider (e.g., "openai", "anthropic")
      pub provider_id: String,
      /// Human-readable name (e.g., "OpenAI", "Anthropic")
      pub display_name: String,
      /// The API key (should never be logged or displayed)
      api_key: String,
      /// Base URL for the API (for custom endpoints)
      pub base_url: Option<String>,
  }
  
  impl ProviderConfig {
      /// Create a new provider configuration.
      pub fn new(
          provider_id: impl Into<String>,
          display_name: impl Into<String>,
          api_key: impl Into<String>,
      ) -> Self {
          Self {
              provider_id: provider_id.into(),
              display_name: display_name.into(),
              api_key: api_key.into(),
              base_url: None,
          }
      }
  
      /// Create a provider configuration with a custom base URL.
      pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
          self.base_url = Some(base_url.into());
          self
      }
  
      /// Get the API key for making requests.
      ///
      /// # Security Note
      /// This method intentionally returns a reference to prevent accidental
      /// copies of the API key. Never log or display the returned value.
      pub fn api_key(&self) -> &str {
          &self.api_key
      }
  
      /// Check if this provider has a valid (non-empty) API key.
      pub fn has_valid_key(&self) -> bool {
          !self.api_key.is_empty()
      }
  }
  
  impl std::fmt::Debug for ProviderConfig {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
          f.debug_struct("ProviderConfig")
              .field("provider_id", &self.provider_id)
              .field("display_name", &self.display_name)
              .field("api_key", &"<redacted>")
              .field("base_url", &self.base_url)
              .finish()
      }
  }
  
  /// Information about an AI model.
  #[derive(Debug, Clone)]
  pub struct ModelInfo {
      /// Unique identifier for the model (e.g., "gpt-4o", "claude-3-5-sonnet")
      pub id: String,
      /// Human-readable display name
      pub display_name: String,
      /// Provider this model belongs to
      pub provider: String,
      /// Whether this model supports streaming responses
      pub supports_streaming: bool,
      /// Context window size in tokens
      pub context_window: u32,
  }
  
  impl ModelInfo {
      /// Create a new model info.
      pub fn new(
          id: impl Into<String>,
          display_name: impl Into<String>,
          provider: impl Into<String>,
          supports_streaming: bool,
          context_window: u32,
      ) -> Self {
          Self {
              id: id.into(),
              display_name: display_name.into(),
              provider: provider.into(),
              supports_streaming,
              context_window,
          }
      }
  }
  
  /// Environment variable names for API keys.
  pub mod env_vars {
      /// OpenAI API key environment variable
      pub const OPENAI_API_KEY: &str = "SCRIPT_KIT_OPENAI_API_KEY";
      /// Anthropic API key environment variable
      pub const ANTHROPIC_API_KEY: &str = "SCRIPT_KIT_ANTHROPIC_API_KEY";
      /// Google AI (Gemini) API key environment variable
      pub const GOOGLE_API_KEY: &str = "SCRIPT_KIT_GOOGLE_API_KEY";
      /// Groq API key environment variable
      pub const GROQ_API_KEY: &str = "SCRIPT_KIT_GROQ_API_KEY";
      /// OpenRouter API key environment variable
      pub const OPENROUTER_API_KEY: &str = "SCRIPT_KIT_OPENROUTER_API_KEY";
      /// Vercel API key environment variable
      pub const VERCEL_API_KEY: &str = "SCRIPT_KIT_VERCEL_API_KEY";
  }
  
  /// Read an environment variable, trimming whitespace and filtering empty values.
  ///
  /// Returns `None` if the variable is not set, is empty after trimming, or contains only whitespace.
  fn read_env_nonempty(name: &str) -> Option<String> {
      env::var(name)
          .ok()
          .map(|s| s.trim().to_string())
          .filter(|s| !s.is_empty())
  }
  
  /// Read an API key from environment variable or system keyring.
  ///
  /// Checks in order:
  /// 1. Environment variable (for users who set keys in shell profile)
  /// 2. System keyring (for users who use built-in "Configure API Key" commands)
  ///
  /// Returns the first non-empty value found, or `None` if not configured.
  fn read_key_env_or_keyring(name: &str) -> Option<String> {
      // First check environment variable
      if let Some(value) = read_env_nonempty(name) {
          crate::logging::log(
              "CONFIG",
              &format!("Found API key in environment variable: {}", name),
          );
          return Some(value);
      }
  
      // Fall back to keyring
      let keyring_result = get_secret(name);
      if keyring_result.is_some() {
          crate::logging::log("CONFIG", &format!("Found API key in keyring for: {}", name));
      }
      keyring_result
  }
  
  /// Detected API keys from environment.
  #[derive(Default)]
  pub struct DetectedKeys {
      pub openai: Option<String>,
      pub anthropic: Option<String>,
      pub google: Option<String>,
      pub groq: Option<String>,
      pub openrouter: Option<String>,
      pub vercel: Option<String>,
  }
  
  impl std::fmt::Debug for DetectedKeys {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
          f.debug_struct("DetectedKeys")
              .field("openai", &self.openai.is_some())
              .field("anthropic", &self.anthropic.is_some())
              .field("google", &self.google.is_some())
              .field("groq", &self.groq.is_some())
              .field("openrouter", &self.openrouter.is_some())
              .field("vercel", &self.vercel.is_some())
              .finish()
      }
  }
  
  impl DetectedKeys {
      /// Scan environment variables and system keyring for API keys.
      ///
      /// Looks for the `SCRIPT_KIT_*_API_KEY` pattern in:
      /// 1. Environment variables (for users who set keys in shell profile)
      /// 2. System keyring (for users who use built-in "Configure API Key" commands)
      ///
      /// This allows the SDK's `await env("SCRIPT_KIT_VERCEL_API_KEY")` to store keys
      /// in the keyring, and have the AI Chat window automatically pick them up.
      pub fn from_environment() -> Self {
          Self {
              openai: read_key_env_or_keyring(env_vars::OPENAI_API_KEY),
              anthropic: read_key_env_or_keyring(env_vars::ANTHROPIC_API_KEY),
              google: read_key_env_or_keyring(env_vars::GOOGLE_API_KEY),
              groq: read_key_env_or_keyring(env_vars::GROQ_API_KEY),
              openrouter: read_key_env_or_keyring(env_vars::OPENROUTER_API_KEY),
              vercel: read_key_env_or_keyring(env_vars::VERCEL_API_KEY),
          }
      }
  
      /// Check if any API keys were detected.
      pub fn has_any(&self) -> bool {
          self.openai.is_some()
              || self.anthropic.is_some()
              || self.google.is_some()
              || self.groq.is_some()
              || self.openrouter.is_some()
              || self.vercel.is_some()
      }
  
      /// Get a summary of which providers are available (for logging).
      ///
      /// Returns a list of provider names that have API keys configured.
      /// Does NOT include the actual keys.
      pub fn available_providers(&self) -> Vec<&'static str> {
          let mut providers = Vec::new();
          if self.openai.is_some() {
              providers.push("OpenAI");
          }
          if self.anthropic.is_some() {
              providers.push("Anthropic");
          }
          if self.google.is_some() {
              providers.push("Google");
          }
          if self.groq.is_some() {
              providers.push("Groq");
          }
          if self.openrouter.is_some() {
              providers.push("OpenRouter");
          }
          if self.vercel.is_some() {
              providers.push("Vercel");
          }
          providers
      }
  }
  
  /// Default models for each provider.
  pub mod default_models {
      use super::ModelInfo;
  
      /// Get default OpenAI models.
      pub fn openai() -> Vec<ModelInfo> {
          vec![
              ModelInfo::new("gpt-4o", "GPT-4o", "openai", true, 128_000),
              ModelInfo::new("gpt-4o-mini", "GPT-4o Mini", "openai", true, 128_000),
              ModelInfo::new("gpt-4-turbo", "GPT-4 Turbo", "openai", true, 128_000),
              ModelInfo::new("gpt-3.5-turbo", "GPT-3.5 Turbo", "openai", true, 16_385),
          ]
      }
  
      /// Get default Anthropic models.
      /// NOTE: The FIRST model in this list is the default model for new chats
      pub fn anthropic() -> Vec<ModelInfo> {
          vec![
              // Default: Claude Haiku 4.5 (fast, cheap, good quality)
              ModelInfo::new(
                  "claude-haiku-4-5-20250514",
                  "Claude Haiku 4.5",
                  "anthropic",
                  true,
                  200_000,
              ),
              ModelInfo::new(
                  "claude-3-5-sonnet-20241022",
                  "Claude 3.5 Sonnet",
                  "anthropic",
                  true,
                  200_000,
              ),
              ModelInfo::new(
                  "claude-3-5-sonnet-20241022",
                  "Claude 3.5 Sonnet",
                  "anthropic",
                  true,
                  200_000,
              ),
              ModelInfo::new(
                  "claude-3-opus-20240229",
                  "Claude 3 Opus",
                  "anthropic",
                  true,
                  200_000,
              ),
          ]
      }
  
      /// Get default Google (Gemini) models.
      pub fn google() -> Vec<ModelInfo> {
          vec![
              ModelInfo::new(
                  "gemini-2.0-flash-exp",
                  "Gemini 2.0 Flash",
                  "google",
                  true,
                  1_000_000,
              ),
              ModelInfo::new(
                  "gemini-1.5-pro",
                  "Gemini 1.5 Pro",
                  "google",
                  true,
                  2_000_000,
              ),
              ModelInfo::new(
                  "gemini-1.5-flash",
                  "Gemini 1.5 Flash",
                  "google",
                  true,
                  1_000_000,
              ),
          ]
      }
  
      /// Get default Groq models.
      pub fn groq() -> Vec<ModelInfo> {
          vec![
              ModelInfo::new(
                  "llama-3.3-70b-versatile",
                  "Llama 3.3 70B",
                  "groq",
                  true,
                  128_000,
              ),
              ModelInfo::new(
                  "llama-3.1-8b-instant",
                  "Llama 3.1 8B Instant",
                  "groq",
                  true,
                  128_000,
              ),
              ModelInfo::new("mixtral-8x7b-32768", "Mixtral 8x7B", "groq", true, 32_768),
          ]
      }
  }
  
  #[cfg(test)]
  mod tests {
      use super::*;
  
      #[test]
      fn test_provider_config_creation() {
          let config = ProviderConfig::new("openai", "OpenAI", "sk-test-key");
          assert_eq!(config.provider_id, "openai");
          assert_eq!(config.display_name, "OpenAI");
          assert_eq!(config.api_key(), "sk-test-key");
          assert!(config.has_valid_key());
      }
  
      #[test]
      fn test_provider_config_empty_key() {
          let config = ProviderConfig::new("openai", "OpenAI", "");
          assert!(!config.has_valid_key());
      }
  
      #[test]
      fn test_provider_config_with_base_url() {
          let config = ProviderConfig::new("openai", "OpenAI", "sk-test")
              .with_base_url("https://api.custom.com");
          assert_eq!(config.base_url, Some("https://api.custom.com".to_string()));
      }
  
      #[test]
      fn test_model_info_creation() {
          let model = ModelInfo::new("gpt-4o", "GPT-4o", "openai", true, 128_000);
          assert_eq!(model.id, "gpt-4o");
          assert_eq!(model.display_name, "GPT-4o");
          assert_eq!(model.provider, "openai");
          assert!(model.supports_streaming);
          assert_eq!(model.context_window, 128_000);
      }
  
      #[test]
      fn test_detected_keys_empty() {
          // Clear any existing env vars for this test
          let keys = DetectedKeys::default();
          assert!(!keys.has_any());
          assert!(keys.available_providers().is_empty());
      }
  
      #[test]
      fn test_detected_keys_with_provider() {
          // Manually construct to avoid env dependency in test
          let keys = DetectedKeys {
              openai: Some("sk-test".to_string()),
              anthropic: None,
              google: None,
              groq: None,
              openrouter: None,
              vercel: None,
          };
          assert!(keys.has_any());
          assert_eq!(keys.available_providers(), vec!["OpenAI"]);
      }
  
      #[test]
      fn test_default_models() {
          let openai_models = default_models::openai();
          assert!(!openai_models.is_empty());
          assert!(openai_models.iter().any(|m| m.id == "gpt-4o"));
  
          let anthropic_models = default_models::anthropic();
          assert!(!anthropic_models.is_empty());
          assert!(anthropic_models.iter().any(|m| m.id.contains("claude")));
      }
  
      #[test]
      fn test_provider_config_debug_redacts_api_key() {
          let config = ProviderConfig::new("openai", "OpenAI", "sk-super-secret-key-12345");
          let debug_output = format!("{:?}", config);
  
          // The API key should NOT appear in debug output
          assert!(!debug_output.contains("sk-super-secret-key-12345"));
          // Instead, it should show <redacted>
          assert!(debug_output.contains("<redacted>"));
          // Other fields should still be visible
          assert!(debug_output.contains("openai"));
          assert!(debug_output.contains("OpenAI"));
      }
  
      #[test]
      fn test_detected_keys_debug_shows_only_presence() {
          let keys = DetectedKeys {
              openai: Some("sk-secret-openai-key".to_string()),
              anthropic: Some("sk-ant-secret-key".to_string()),
              google: None,
              groq: None,
              openrouter: None,
              vercel: Some("vk-vercel-key".to_string()),
          };
          let debug_output = format!("{:?}", keys);
  
          // Actual key values should NOT appear
          assert!(!debug_output.contains("sk-secret-openai-key"));
          assert!(!debug_output.contains("sk-ant-secret-key"));
          assert!(!debug_output.contains("vk-vercel-key"));
          // Should show boolean presence indicators
          assert!(debug_output.contains("openai: true"));
          assert!(debug_output.contains("anthropic: true"));
          assert!(debug_output.contains("google: false"));
          assert!(debug_output.contains("vercel: true"));
      }
  
      #[test]
      fn test_read_env_nonempty_trims_whitespace() {
          // Set up test env var with whitespace
          std::env::set_var("TEST_READ_ENV_WHITESPACE", "  test-value  ");
          let result = read_env_nonempty("TEST_READ_ENV_WHITESPACE");
          assert_eq!(result, Some("test-value".to_string()));
          std::env::remove_var("TEST_READ_ENV_WHITESPACE");
      }
  
      #[test]
      fn test_read_env_nonempty_filters_empty() {
          // Set up test env var that's empty
          std::env::set_var("TEST_READ_ENV_EMPTY", "");
          let result = read_env_nonempty("TEST_READ_ENV_EMPTY");
          assert_eq!(result, None);
          std::env::remove_var("TEST_READ_ENV_EMPTY");
      }
  
      #[test]
      fn test_read_env_nonempty_filters_whitespace_only() {
          // Set up test env var with only whitespace
          std::env::set_var("TEST_READ_ENV_WS_ONLY", "   ");
          let result = read_env_nonempty("TEST_READ_ENV_WS_ONLY");
          assert_eq!(result, None);
          std::env::remove_var("TEST_READ_ENV_WS_ONLY");
      }
  
      #[test]
      fn test_detected_keys_with_vercel() {
          let keys = DetectedKeys {
              openai: None,
              anthropic: None,
              google: None,
              groq: None,
              openrouter: None,
              vercel: Some("vk-test".to_string()),
          };
          assert!(keys.has_any());
          assert_eq!(keys.available_providers(), vec!["Vercel"]);
      }
  }
  
  </file>
  
  <file path="src/ai/sdk_handlers.rs">
  //! AI SDK Protocol Handlers
  //!
  //! This module handles AI SDK protocol messages from scripts.
  //! It converts between protocol types and storage/window operations.
  
  use anyhow::Result;
  use tracing::{debug, error, info};
  
  use crate::protocol::{AiChatInfo, AiMessageInfo, Message};
  
  use super::model::{Chat, ChatId, Message as AiMessage, MessageRole};
  use super::storage;
  
  /// Convert a Chat from storage to AiChatInfo for protocol
  fn chat_to_info(chat: &Chat, message_count: usize) -> AiChatInfo {
      AiChatInfo {
          id: chat.id.as_str(),
          title: chat.title.clone(),
          model_id: chat.model_id.clone(),
          provider: chat.provider.clone(),
          created_at: chat.created_at.to_rfc3339(),
          updated_at: chat.updated_at.to_rfc3339(),
          is_deleted: chat.deleted_at.is_some(),
          preview: None, // Could add first message preview later
          message_count,
      }
  }
  
  /// Convert a Message from storage to AiMessageInfo for protocol
  fn message_to_info(msg: &AiMessage) -> AiMessageInfo {
      AiMessageInfo {
          id: msg.id.to_string(),
          role: msg.role.as_str().to_string(),
          content: msg.content.clone(),
          created_at: msg.created_at.to_rfc3339(),
          tokens_used: msg.tokens_used,
      }
  }
  
  /// Handle AiIsOpen request - check if AI window is open
  pub fn handle_ai_is_open(request_id: String) -> Message {
      let is_open = super::is_ai_window_open();
      // TODO: Get active chat ID from window state
      let active_chat_id = None;
  
      debug!(request_id = %request_id, is_open = is_open, "AiIsOpen handled");
  
      Message::AiIsOpenResult {
          request_id,
          is_open,
          active_chat_id,
      }
  }
  
  /// Handle AiGetActiveChat request - get info about active chat
  pub fn handle_ai_get_active_chat(request_id: String) -> Message {
      // TODO: Get active chat ID from window state
      // For now, return the most recently updated chat
      let chat = match storage::get_all_chats() {
          Ok(chats) => chats.into_iter().next(),
          Err(e) => {
              error!(error = %e, "Failed to get chats for AiGetActiveChat");
              None
          }
      };
  
      let chat_info = chat.map(|c| {
          let msg_count = storage::get_chat_messages(&c.id)
              .map(|msgs| msgs.len())
              .unwrap_or(0);
          chat_to_info(&c, msg_count)
      });
  
      debug!(request_id = %request_id, has_chat = chat_info.is_some(), "AiGetActiveChat handled");
  
      Message::AiActiveChatResult {
          request_id,
          chat: chat_info,
      }
  }
  
  /// Handle AiListChats request - list all chats
  pub fn handle_ai_list_chats(
      request_id: String,
      limit: Option<usize>,
      include_deleted: bool,
  ) -> Message {
      let mut chats = match storage::get_all_chats() {
          Ok(c) => c,
          Err(e) => {
              error!(error = %e, "Failed to list chats");
              return Message::AiChatListResult {
                  request_id,
                  chats: vec![],
                  total_count: 0,
              };
          }
      };
  
      if include_deleted {
          if let Ok(deleted) = storage::get_deleted_chats() {
              chats.extend(deleted);
          }
      }
  
      let total_count = chats.len();
  
      // Apply limit
      if let Some(limit) = limit {
          chats.truncate(limit);
      }
  
      let chat_infos: Vec<AiChatInfo> = chats
          .iter()
          .map(|c| {
              let msg_count = storage::get_chat_messages(&c.id)
                  .map(|msgs| msgs.len())
                  .unwrap_or(0);
              chat_to_info(c, msg_count)
          })
          .collect();
  
      debug!(
          request_id = %request_id,
          count = chat_infos.len(),
          total = total_count,
          "AiListChats handled"
      );
  
      Message::AiChatListResult {
          request_id,
          chats: chat_infos,
          total_count,
      }
  }
  
  /// Handle AiGetConversation request - get messages from a chat
  pub fn handle_ai_get_conversation(
      request_id: String,
      chat_id: Option<String>,
      limit: Option<usize>,
  ) -> Message {
      // Get chat ID - use provided or try to get active
      let target_chat_id = match chat_id {
          Some(id) => match ChatId::parse(&id) {
              Some(cid) => cid,
              None => {
                  error!(chat_id = %id, "Invalid chat ID format");
                  return Message::AiConversationResult {
                      request_id,
                      chat_id: id,
                      messages: vec![],
                      has_more: false,
                  };
              }
          },
          None => {
              // Try to get most recent chat
              match storage::get_all_chats() {
                  Ok(chats) => match chats.into_iter().next() {
                      Some(c) => c.id,
                      None => {
                          return Message::AiConversationResult {
                              request_id,
                              chat_id: String::new(),
                              messages: vec![],
                              has_more: false,
                          };
                      }
                  },
                  Err(e) => {
                      error!(error = %e, "Failed to get chats for conversation");
                      return Message::AiConversationResult {
                          request_id,
                          chat_id: String::new(),
                          messages: vec![],
                          has_more: false,
                      };
                  }
              }
          }
      };
  
      let messages = match limit {
          Some(lim) => storage::get_recent_messages(&target_chat_id, lim),
          None => storage::get_chat_messages(&target_chat_id),
      };
  
      let (message_infos, has_more) = match messages {
          Ok(msgs) => {
              let total = msgs.len();
              let infos: Vec<AiMessageInfo> = msgs.iter().map(message_to_info).collect();
              let has_more = limit.map(|l| total >= l).unwrap_or(false);
              (infos, has_more)
          }
          Err(e) => {
              error!(error = %e, chat_id = %target_chat_id, "Failed to get messages");
              (vec![], false)
          }
      };
  
      debug!(
          request_id = %request_id,
          chat_id = %target_chat_id,
          message_count = message_infos.len(),
          "AiGetConversation handled"
      );
  
      Message::AiConversationResult {
          request_id,
          chat_id: target_chat_id.as_str(),
          messages: message_infos,
          has_more,
      }
  }
  
  /// Handle AiDeleteChat request - delete a chat
  pub fn handle_ai_delete_chat(request_id: String, chat_id: String, permanent: bool) -> Message {
      let parsed_id = match ChatId::parse(&chat_id) {
          Some(id) => id,
          None => {
              return Message::AiChatDeleted {
                  request_id,
                  success: false,
                  error: Some(format!("Invalid chat ID: {}", chat_id)),
              };
          }
      };
  
      let result = if permanent {
          storage::delete_chat_permanently(&parsed_id)
      } else {
          storage::delete_chat(&parsed_id)
      };
  
      match result {
          Ok(()) => {
              info!(chat_id = %chat_id, permanent = permanent, "Chat deleted via SDK");
              Message::AiChatDeleted {
                  request_id,
                  success: true,
                  error: None,
              }
          }
          Err(e) => {
              error!(error = %e, chat_id = %chat_id, "Failed to delete chat");
              Message::AiChatDeleted {
                  request_id,
                  success: false,
                  error: Some(e.to_string()),
              }
          }
      }
  }
  
  /// Handle AiFocus request - focus the AI window
  pub fn handle_ai_focus(request_id: String) -> Option<Message> {
      // This needs to be handled by the UI thread, return None to signal forwarding needed
      debug!(request_id = %request_id, "AiFocus needs UI thread handling");
      None
  }
  
  /// Handle AiGetStreamingStatus request
  pub fn handle_ai_get_streaming_status(request_id: String, _chat_id: Option<String>) -> Message {
      // TODO: Get streaming status from window state
      // For now, return not streaming
      Message::AiStreamingStatusResult {
          request_id,
          is_streaming: false,
          chat_id: None,
          partial_content: None,
      }
  }
  
  /// Check if a message is an AI SDK message that can be handled directly
  /// Returns Some(response) if handled, None if needs UI thread
  pub fn try_handle_ai_message(msg: &Message) -> Option<Message> {
      match msg {
          Message::AiIsOpen { request_id } => Some(handle_ai_is_open(request_id.clone())),
  
          Message::AiGetActiveChat { request_id } => {
              Some(handle_ai_get_active_chat(request_id.clone()))
          }
  
          Message::AiListChats {
              request_id,
              limit,
              include_deleted,
          } => Some(handle_ai_list_chats(
              request_id.clone(),
              *limit,
              *include_deleted,
          )),
  
          Message::AiGetConversation {
              request_id,
              chat_id,
              limit,
          } => Some(handle_ai_get_conversation(
              request_id.clone(),
              chat_id.clone(),
              *limit,
          )),
  
          Message::AiDeleteChat {
              request_id,
              chat_id,
              permanent,
          } => Some(handle_ai_delete_chat(
              request_id.clone(),
              chat_id.clone(),
              *permanent,
          )),
  
          Message::AiGetStreamingStatus {
              request_id,
              chat_id,
          } => Some(handle_ai_get_streaming_status(
              request_id.clone(),
              chat_id.clone(),
          )),
  
          // These need UI thread - return None
          Message::AiFocus { .. }
          | Message::AiStartChat { .. }
          | Message::AiAppendMessage { .. }
          | Message::AiSendMessage { .. }
          | Message::AiSetSystemPrompt { .. }
          | Message::AiSubscribe { .. }
          | Message::AiUnsubscribe { .. } => None,
  
          // Not an AI message
          _ => None,
      }
  }
  
  #[cfg(test)]
  mod tests {
      use super::*;
  
      #[test]
      fn test_ai_is_open_when_closed() {
          let response = handle_ai_is_open("test-123".to_string());
          match response {
              Message::AiIsOpenResult {
                  request_id,
                  is_open,
                  ..
              } => {
                  assert_eq!(request_id, "test-123");
                  assert!(!is_open); // Window not open in tests
              }
              _ => panic!("Expected AiIsOpenResult"),
          }
      }
  }
  
  </file>
  
  </files>
  ---
  
  ## Implementation Guide
  
  ### Step 1: Add Tool Definition Types (`src/ai/tools.rs` - NEW FILE)
  
  ```rust
  //! AI Tool definitions and registry
  //!
  //! Implements function calling / tool use for AI providers.
  
  use serde::{Deserialize, Serialize};
  use std::collections::HashMap;
  use std::sync::Arc;
  
  /// JSON Schema for tool parameters
  pub type ParameterSchema = serde_json::Value;
  
  /// Definition of a tool that can be called by the AI
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct ToolDefinition {
      /// Unique name of the tool (e.g., "web_search", "run_code")
      pub name: String,
      /// Description shown to the AI (be detailed!)
      pub description: String,
      /// JSON Schema for the input parameters
      pub input_schema: ParameterSchema,
  }
  
  impl ToolDefinition {
      pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
          Self {
              name: name.into(),
              description: description.into(),
              input_schema: serde_json::json!({
                  "type": "object",
                  "properties": {},
                  "required": []
              }),
          }
      }
  
      pub fn with_schema(mut self, schema: ParameterSchema) -> Self {
          self.input_schema = schema;
          self
      }
  }
  
  /// A tool use request from the AI
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct ToolUse {
      /// Unique ID for this tool use (for matching results)
      pub id: String,
      /// Name of the tool being called
      pub name: String,
      /// Input arguments as JSON
      pub input: serde_json::Value,
  }
  
  /// Result of executing a tool
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct ToolResult {
      /// ID of the tool use this is responding to
      pub tool_use_id: String,
      /// The result content (text or structured data)
      pub content: String,
      /// Whether the tool execution failed
      pub is_error: bool,
  }
  
  impl ToolResult {
      pub fn success(tool_use_id: String, content: impl Into<String>) -> Self {
          Self {
              tool_use_id,
              content: content.into(),
              is_error: false,
          }
      }
  
      pub fn error(tool_use_id: String, error: impl Into<String>) -> Self {
          Self {
              tool_use_id,
              content: error.into(),
              is_error: true,
          }
      }
  }
  
  /// Callback for executing a tool
  pub type ToolExecutor = Arc<dyn Fn(&ToolUse) -> ToolResult + Send + Sync>;
  
  /// Registry of available tools
  pub struct ToolRegistry {
      definitions: HashMap<String, ToolDefinition>,
      executors: HashMap<String, ToolExecutor>,
  }
  
  impl ToolRegistry {
      pub fn new() -> Self {
          Self {
              definitions: HashMap::new(),
              executors: HashMap::new(),
          }
      }
  
      pub fn register(&mut self, definition: ToolDefinition, executor: ToolExecutor) {
          self.executors.insert(definition.name.clone(), executor);
          self.definitions.insert(definition.name.clone(), definition);
      }
  
      pub fn get_definitions(&self) -> Vec<&ToolDefinition> {
          self.definitions.values().collect()
      }
  
      pub fn execute(&self, tool_use: &ToolUse) -> ToolResult {
          match self.executors.get(&tool_use.name) {
              Some(executor) => executor(tool_use),
              None => ToolResult::error(
                  tool_use.id.clone(),
                  format!("Unknown tool: {}", tool_use.name),
              ),
          }
      }
  }
  ```
  
  ### Step 2: Implement Web Search Tool
  
  ```rust
  //! Web search tool implementation
  
  use super::tools::{ToolDefinition, ToolUse, ToolResult, ToolExecutor};
  use std::sync::Arc;
  
  /// Create the web search tool definition
  pub fn web_search_definition() -> ToolDefinition {
      ToolDefinition::new(
          "web_search",
          "Search the web for information. Use this when you need current information, \
          facts, or to look up something you don't know. Returns relevant search results."
      ).with_schema(serde_json::json!({
          "type": "object",
          "properties": {
              "query": {
                  "type": "string",
                  "description": "The search query"
              },
              "num_results": {
                  "type": "integer",
                  "description": "Number of results to return (default 5, max 10)",
                  "default": 5
              }
          },
          "required": ["query"]
      }))
  }
  
  /// Create the web search executor
  pub fn web_search_executor() -> ToolExecutor {
      Arc::new(|tool_use: &ToolUse| {
          let query = tool_use.input.get("query")
              .and_then(|v| v.as_str())
              .unwrap_or("");
          
          let num_results = tool_use.input.get("num_results")
              .and_then(|v| v.as_u64())
              .unwrap_or(5) as usize;
          
          // TODO: Implement actual web search using:
          // - SearXNG (self-hosted, no API key)
          // - Brave Search API
          // - Tavily API (AI-optimized)
          // - Perplexity API (includes AI summary)
          
          match execute_search(query, num_results) {
              Ok(results) => ToolResult::success(tool_use.id.clone(), results),
              Err(e) => ToolResult::error(tool_use.id.clone(), e.to_string()),
          }
      })
  }
  
  fn execute_search(query: &str, num_results: usize) -> anyhow::Result<String> {
      // Example using Brave Search API
      let api_key = std::env::var("SCRIPT_KIT_BRAVE_API_KEY")?;
      
      let response = ureq::get("https://api.search.brave.com/res/v1/web/search")
          .query("q", query)
          .query("count", &num_results.to_string())
          .set("X-Subscription-Token", &api_key)
          .call()?
          .into_json::<serde_json::Value>()?;
      
      // Format results for AI consumption
      let mut output = format!("Search results for: {}\n\n", query);
      
      if let Some(results) = response.get("web").and_then(|w| w.get("results")).and_then(|r| r.as_array()) {
          for (i, result) in results.iter().take(num_results).enumerate() {
              let title = result.get("title").and_then(|t| t.as_str()).unwrap_or("No title");
              let url = result.get("url").and_then(|u| u.as_str()).unwrap_or("");
              let description = result.get("description").and_then(|d| d.as_str()).unwrap_or("");
              
              output.push_str(&format!("{}. {}\n   URL: {}\n   {}\n\n", i + 1, title, url, description));
          }
      }
      
      Ok(output)
  }
  ```
  
  ### Step 3: Update Anthropic Provider for Tool Use
  
  ```rust
  // In src/ai/providers.rs - Add tool support to AnthropicProvider
  
  impl AnthropicProvider {
      /// Build request body WITH tools
      fn build_request_body_with_tools(
          &self,
          messages: &[ProviderMessage],
          tools: &[ToolDefinition],
          model_id: &str,
          stream: bool,
      ) -> serde_json::Value {
          let api_messages = /* ... existing message conversion ... */;
          
          // Convert tools to Anthropic format
          let api_tools: Vec<serde_json::Value> = tools.iter().map(|t| {
              serde_json::json!({
                  "name": t.name,
                  "description": t.description,
                  "input_schema": t.input_schema
              })
          }).collect();
  
          let mut body = serde_json::json!({
              "model": model_id,
              "max_tokens": DEFAULT_MAX_TOKENS,
              "stream": stream,
              "messages": api_messages,
              "tools": api_tools
          });
  
          // Add system message if present
          if let Some(system) = system_msg {
              body["system"] = serde_json::Value::String(system);
          }
  
          body
      }
  
      /// Parse tool_use events from Anthropic SSE stream
      fn parse_tool_use_from_sse(data: &str) -> Option<ToolUse> {
          let parsed: serde_json::Value = serde_json::from_str(data).ok()?;
          
          // Anthropic tool_use format:
          // {"type": "content_block_start", "content_block": {"type": "tool_use", "id": "...", "name": "...", "input": {}}}
          // {"type": "content_block_delta", "delta": {"type": "input_json_delta", "partial_json": "..."}}
          // {"type": "content_block_stop"}
          
          if parsed.get("type")?.as_str()? == "content_block_start" {
              let block = parsed.get("content_block")?;
              if block.get("type")?.as_str()? == "tool_use" {
                  return Some(ToolUse {
                      id: block.get("id")?.as_str()?.to_string(),
                      name: block.get("name")?.as_str()?.to_string(),
                      input: block.get("input").cloned().unwrap_or(serde_json::json!({})),
                  });
              }
          }
          
          None
      }
  }
  ```
  
  ### Step 4: Create Tool Execution Loop
  
  ```rust
  //! Tool execution loop for handling AI tool calls
  
  use crate::ai::providers::{AiProvider, ProviderMessage};
  use crate::ai::tools::{ToolRegistry, ToolUse, ToolResult};
  
  /// Execute a conversation with tool support
  /// 
  /// This implements the "agentic loop":
  /// 1. Send messages to AI
  /// 2. If AI returns tool_use, execute tools
  /// 3. Add tool results to conversation
  /// 4. Repeat until AI returns text-only response
  pub async fn execute_with_tools(
      provider: &dyn AiProvider,
      messages: Vec<ProviderMessage>,
      tools: &ToolRegistry,
      model_id: &str,
      max_iterations: usize,
  ) -> anyhow::Result<String> {
      let mut conversation = messages;
      let tool_defs: Vec<_> = tools.get_definitions().into_iter().cloned().collect();
      
      for iteration in 0..max_iterations {
          // Send to AI with tools
          let response = provider.send_message_with_tools(&conversation, &tool_defs, model_id)?;
          
          // Check if AI wants to use tools
          if response.tool_uses.is_empty() {
              // No tool calls - return the text response
              return Ok(response.text_content);
          }
          
          // Execute all requested tools
          let mut results = Vec::new();
          for tool_use in &response.tool_uses {
              tracing::info!(tool = %tool_use.name, id = %tool_use.id, "Executing tool");
              let result = tools.execute(tool_use);
              results.push(result);
          }
          
          // Add assistant message with tool_use to conversation
          conversation.push(ProviderMessage::assistant_with_tool_use(response.tool_uses));
          
          // Add tool results to conversation
          for result in results {
              conversation.push(ProviderMessage::tool_result(result));
          }
          
          tracing::debug!(iteration, "Tool loop iteration complete");
      }
      
      anyhow::bail!("Max tool iterations ({}) exceeded", max_iterations)
  }
  ```
  
  ### Step 5: Common Tools to Implement
  
  | Tool | Description | API/Method |
  |------|-------------|------------|
  | `web_search` | Search the web | Brave, Tavily, SearXNG |
  | `read_url` | Fetch and parse a URL | jina.ai reader, scraping |
  | `run_code` | Execute code snippet | Sandboxed Bun/Node |
  | `read_file` | Read a local file | Direct FS access |
  | `write_file` | Write to a file | Direct FS with confirmation |
  | `list_files` | List directory contents | Direct FS |
  | `shell_command` | Run shell command | Sandboxed execution |
  | `get_clipboard` | Get clipboard content | Platform API |
  | `set_clipboard` | Set clipboard content | Platform API |
  | `get_selected_text` | Get selected text | Accessibility API |
  
  ---
  
  ## Instructions for the Next AI Agent
  
  ### Context
  You are reviewing the AI conversation system for Script Kit GPUI. The codebase uses:
  - **Rust** with GPUI (GPU-accelerated UI framework)
  - **ureq** for HTTP requests (blocking, thread-spawned)
  - **SSE streaming** for AI responses
  - **SQLite** for chat persistence
  
  ### Your Task
  Design and implement AI tool calling (function calling) support for this codebase. Specifically:
  
  1. **Review the `AiProvider` trait** in `src/ai/providers.rs` and extend it to support tools
  2. **Implement tool_use parsing** for Anthropic's SSE format (the primary provider)
  3. **Create the tool registry system** that can register/execute tools
  4. **Implement web search** as the first tool (suggest the best API for this use case)
  
  ### Key Constraints
  - Must work with streaming responses
  - Tools execute in Rust (not forwarded to TypeScript SDK)
  - Support both Anthropic and OpenAI tool formats
  - Must handle the agentic loop (AI calls tools → execute → return → continue)
  
  ### Questions to Answer
  1. Which web search API should we use? (Brave, Tavily, SearXNG, Perplexity)
  2. Should tool execution be async or blocking (given ureq is blocking)?
  3. How should we handle long-running tools (timeout, cancellation)?
  4. Should tools be sandboxed for security?
  5. What's the best UX for showing tool execution in the chat UI?
  
  ### Expected Deliverables
  1. `src/ai/tools.rs` - Tool types and registry
  2. `src/ai/tools/mod.rs` - Built-in tools (web_search, etc.)
  3. Updated `src/ai/providers.rs` with tool support
  4. Updated `src/prompts/chat.rs` to show tool execution
  
  ### Code Style
  - Use `anyhow::Result` for errors with `.context()`
  - Use `tracing` for structured logging
  - Follow existing patterns in the codebase
  - Add tests in a `#[cfg(test)]` module
  
  ---
  
  ## Additional Context Files (Not Included - Query Separately If Needed)
  
  - `src/ai/storage.rs` - SQLite persistence (~750 lines)
  - `src/ai/window.rs` - AI chat window UI (~2000 lines)
  - `src/protocol.rs` - SDK message protocol types
  - `Cargo.toml` - Project dependencies
