🧩 Packing 6 file(s)...
📝 Files selected:
  • src/ai/model.rs
  • src/ai/providers.rs
  • src/ai/storage.rs
  • src/ai/mod.rs
  • src/ai/window.rs
  • src/ai/config.rs
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
src/ai/model.rs
src/ai/providers.rs
src/ai/storage.rs
src/ai/mod.rs
src/ai/window.rs
src/ai/config.rs
</directory_structure>

<files>
This section contains the contents of the repository's files.

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
        }
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

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::sync::Arc;

use super::config::{default_models, DetectedKeys, ModelInfo, ProviderConfig};

/// Message for AI provider API calls.
#[derive(Debug, Clone)]
pub struct ProviderMessage {
    /// Role of the message sender: "user", "assistant", or "system"
    pub role: String,
    /// Content of the message
    pub content: String,
}

impl ProviderMessage {
    /// Create a new user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    /// Create a new assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }

    /// Create a new system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
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
}

/// OpenAI API constants
const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";

impl OpenAiProvider {
    /// Create a new OpenAI provider with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            config: ProviderConfig::new("openai", "OpenAI", api_key),
        }
    }

    /// Create with a custom base URL (for Azure OpenAI or proxies).
    pub fn with_base_url(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            config: ProviderConfig::new("openai", "OpenAI", api_key).with_base_url(base_url),
        }
    }

    /// Get the API URL (uses custom base_url if set)
    fn api_url(&self) -> &str {
        self.config.base_url.as_deref().unwrap_or(OPENAI_API_URL)
    }

    /// Build the request body for OpenAI API
    fn build_request_body(
        &self,
        messages: &[ProviderMessage],
        model_id: &str,
        stream: bool,
    ) -> serde_json::Value {
        let api_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": m.role,
                    "content": m.content
                })
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

        let response = ureq::post(self.api_url())
            .header("Content-Type", "application/json")
            .header(
                "Authorization",
                &format!("Bearer {}", self.config.api_key()),
            )
            .send_json(&body)
            .context("Failed to send request to OpenAI API")?;

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

        let response = ureq::post(self.api_url())
            .header("Content-Type", "application/json")
            .header(
                "Authorization",
                &format!("Bearer {}", self.config.api_key()),
            )
            .header("Accept", "text/event-stream")
            .send_json(&body)
            .context("Failed to send streaming request to OpenAI API")?;

        // Read the SSE stream
        let reader = BufReader::new(response.into_body().into_reader());

        for line in reader.lines() {
            let line = line.context("Failed to read SSE line")?;

            // Skip empty lines
            if line.is_empty() {
                continue;
            }

            // Parse and extract content delta
            if let Some(text) = Self::parse_sse_line(&line) {
                on_chunk(text);
            }
        }

        tracing::debug!("Completed streaming response from OpenAI");

        Ok(())
    }
}

/// Anthropic provider implementation with real API calls.
pub struct AnthropicProvider {
    config: ProviderConfig,
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
        }
    }

    /// Build the request body for Anthropic API
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

        // Filter out system messages for the messages array
        let api_messages: Vec<serde_json::Value> = messages
            .iter()
            .filter(|m| m.role != "system")
            .map(|m| {
                serde_json::json!({
                    "role": m.role,
                    "content": m.content
                })
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

        let response = ureq::post(ANTHROPIC_API_URL)
            .header("Content-Type", "application/json")
            .header("x-api-key", self.config.api_key())
            .header("anthropic-version", ANTHROPIC_VERSION)
            .send_json(&body)
            .context("Failed to send request to Anthropic API")?;

        let response_json: serde_json::Value = response
            .into_body()
            .read_json()
            .context("Failed to parse Anthropic response")?;

        // Extract content from response
        // Response format: {"content": [{"type": "text", "text": "..."}], ...}
        let content = response_json
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|block| block.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();

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

        let response = ureq::post(ANTHROPIC_API_URL)
            .header("Content-Type", "application/json")
            .header("x-api-key", self.config.api_key())
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("Accept", "text/event-stream")
            .send_json(&body)
            .context("Failed to send streaming request to Anthropic API")?;

        // Read the SSE stream
        let reader = BufReader::new(response.into_body().into_reader());

        for line in reader.lines() {
            let line = line.context("Failed to read SSE line")?;

            // Skip empty lines (SSE uses blank lines as event separators)
            if line.is_empty() {
                continue;
            }

            // Parse and extract content delta
            if let Some(text) = Self::parse_sse_line(&line) {
                on_chunk(text);
            }
        }

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

/// Registry of available AI providers.
///
/// The registry automatically discovers available providers based on
/// environment variables and provides a unified interface to access them.
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
}

</file>

<file path="src/ai/storage.rs">
//! AI Chat Storage Layer
//!
//! SQLite-backed persistence for AI chats with CRUD operations and FTS5 search.
//! Follows the same patterns as src/notes/storage.rs for consistency.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use tracing::{debug, info};

use super::model::{Chat, ChatId, Message, MessageRole};

/// Global database connection for AI chats
static AI_DB: OnceLock<Arc<Mutex<Connection>>> = OnceLock::new();

/// Get the path to the AI chats database (~/.sk/kit/db/ai-chats.sqlite)
fn get_ai_db_path() -> PathBuf {
    let kit_dir = dirs::home_dir()
        .map(|h| h.join(".sk/kit"))
        .unwrap_or_else(|| PathBuf::from(".sk/kit"));

    kit_dir.join("db").join("ai-chats.sqlite")
}

/// Initialize the AI chats database
pub fn init_ai_db() -> Result<()> {
    let db_path = get_ai_db_path();

    // Ensure directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create AI db directory")?;
    }

    let conn = Connection::open(&db_path).context("Failed to open AI chats database")?;

    // Create tables
    conn.execute_batch(
        r#"
        -- Chats table
        CREATE TABLE IF NOT EXISTS chats (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL DEFAULT 'New Chat',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT,
            model_id TEXT NOT NULL,
            provider TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_chats_updated_at ON chats(updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_chats_deleted_at ON chats(deleted_at);
        CREATE INDEX IF NOT EXISTS idx_chats_provider ON chats(provider);

        -- Messages table
        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            chat_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL,
            tokens_used INTEGER,
            FOREIGN KEY (chat_id) REFERENCES chats(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_messages_chat_id ON messages(chat_id);
        CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at);

        -- Full-text search support for chats (searches titles and message content)
        CREATE VIRTUAL TABLE IF NOT EXISTS chats_fts USING fts5(
            title,
            content='chats',
            content_rowid='rowid'
        );

        -- Full-text search for messages
        CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
            content,
            content='messages',
            content_rowid='rowid'
        );

        -- Triggers to keep chat FTS in sync
        CREATE TRIGGER IF NOT EXISTS chats_ai AFTER INSERT ON chats BEGIN
            INSERT INTO chats_fts(rowid, title) 
            VALUES (NEW.rowid, NEW.title);
        END;

        CREATE TRIGGER IF NOT EXISTS chats_ad AFTER DELETE ON chats BEGIN
            INSERT INTO chats_fts(chats_fts, rowid, title) 
            VALUES('delete', OLD.rowid, OLD.title);
        END;

        CREATE TRIGGER IF NOT EXISTS chats_au AFTER UPDATE ON chats BEGIN
            INSERT INTO chats_fts(chats_fts, rowid, title) 
            VALUES('delete', OLD.rowid, OLD.title);
            INSERT INTO chats_fts(rowid, title) 
            VALUES (NEW.rowid, NEW.title);
        END;

        -- Triggers to keep message FTS in sync
        CREATE TRIGGER IF NOT EXISTS messages_ai AFTER INSERT ON messages BEGIN
            INSERT INTO messages_fts(rowid, content) 
            VALUES (NEW.rowid, NEW.content);
        END;

        CREATE TRIGGER IF NOT EXISTS messages_ad AFTER DELETE ON messages BEGIN
            INSERT INTO messages_fts(messages_fts, rowid, content) 
            VALUES('delete', OLD.rowid, OLD.content);
        END;

        CREATE TRIGGER IF NOT EXISTS messages_au AFTER UPDATE ON messages BEGIN
            INSERT INTO messages_fts(messages_fts, rowid, content) 
            VALUES('delete', OLD.rowid, OLD.content);
            INSERT INTO messages_fts(rowid, content) 
            VALUES (NEW.rowid, NEW.content);
        END;
        "#,
    )
    .context("Failed to create AI tables")?;

    info!(db_path = %db_path.display(), "AI chats database initialized");

    AI_DB
        .set(Arc::new(Mutex::new(conn)))
        .map_err(|_| anyhow::anyhow!("AI database already initialized"))?;

    Ok(())
}

/// Get a reference to the AI database connection
fn get_db() -> Result<Arc<Mutex<Connection>>> {
    AI_DB
        .get()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("AI database not initialized"))
}

// ============================================================================
// Chat Operations
// ============================================================================

/// Create a new chat
pub fn create_chat(chat: &Chat) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    conn.execute(
        r#"
        INSERT INTO chats (id, title, created_at, updated_at, deleted_at, model_id, provider)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
        params![
            chat.id.as_str(),
            chat.title,
            chat.created_at.to_rfc3339(),
            chat.updated_at.to_rfc3339(),
            chat.deleted_at.map(|dt| dt.to_rfc3339()),
            chat.model_id,
            chat.provider,
        ],
    )
    .context("Failed to create chat")?;

    debug!(chat_id = %chat.id, title = %chat.title, "Chat created");
    Ok(())
}

/// Update an existing chat
pub fn update_chat(chat: &Chat) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    conn.execute(
        r#"
        UPDATE chats 
        SET title = ?2, updated_at = ?3, deleted_at = ?4, model_id = ?5, provider = ?6
        WHERE id = ?1
        "#,
        params![
            chat.id.as_str(),
            chat.title,
            chat.updated_at.to_rfc3339(),
            chat.deleted_at.map(|dt| dt.to_rfc3339()),
            chat.model_id,
            chat.provider,
        ],
    )
    .context("Failed to update chat")?;

    debug!(chat_id = %chat.id, "Chat updated");
    Ok(())
}

/// Update chat title
pub fn update_chat_title(chat_id: &ChatId, title: &str) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE chats SET title = ?2, updated_at = ?3 WHERE id = ?1",
        params![chat_id.as_str(), title, now],
    )
    .context("Failed to update chat title")?;

    debug!(chat_id = %chat_id, title = %title, "Chat title updated");
    Ok(())
}

/// Get a chat by ID
pub fn get_chat(id: &ChatId) -> Result<Option<Chat>> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, title, created_at, updated_at, deleted_at, model_id, provider
            FROM chats
            WHERE id = ?1
            "#,
        )
        .context("Failed to prepare get_chat query")?;

    let result = stmt
        .query_row(params![id.as_str()], row_to_chat)
        .optional()
        .context("Failed to get chat")?;

    Ok(result)
}

/// Get all active chats (not deleted), sorted by updated_at desc
pub fn get_all_chats() -> Result<Vec<Chat>> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, title, created_at, updated_at, deleted_at, model_id, provider
            FROM chats
            WHERE deleted_at IS NULL
            ORDER BY updated_at DESC
            "#,
        )
        .context("Failed to prepare get_all_chats query")?;

    let chats = stmt
        .query_map([], row_to_chat)
        .context("Failed to query chats")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect chats")?;

    debug!(count = chats.len(), "Retrieved all chats");
    Ok(chats)
}

/// Get chats in trash (soft-deleted)
pub fn get_deleted_chats() -> Result<Vec<Chat>> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, title, created_at, updated_at, deleted_at, model_id, provider
            FROM chats
            WHERE deleted_at IS NOT NULL
            ORDER BY deleted_at DESC
            "#,
        )
        .context("Failed to prepare get_deleted_chats query")?;

    let chats = stmt
        .query_map([], row_to_chat)
        .context("Failed to query deleted chats")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect deleted chats")?;

    debug!(count = chats.len(), "Retrieved deleted chats");
    Ok(chats)
}

/// Soft delete a chat
pub fn delete_chat(chat_id: &ChatId) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE chats SET deleted_at = ?2, updated_at = ?2 WHERE id = ?1",
        params![chat_id.as_str(), now],
    )
    .context("Failed to soft delete chat")?;

    info!(chat_id = %chat_id, "Chat soft deleted");
    Ok(())
}

/// Restore a chat from trash
pub fn restore_chat(chat_id: &ChatId) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE chats SET deleted_at = NULL, updated_at = ?2 WHERE id = ?1",
        params![chat_id.as_str(), now],
    )
    .context("Failed to restore chat")?;

    info!(chat_id = %chat_id, "Chat restored from trash");
    Ok(())
}

/// Permanently delete a chat and all its messages
pub fn delete_chat_permanently(chat_id: &ChatId) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    // Delete messages first (foreign key constraint)
    conn.execute(
        "DELETE FROM messages WHERE chat_id = ?1",
        params![chat_id.as_str()],
    )
    .context("Failed to delete chat messages")?;

    conn.execute("DELETE FROM chats WHERE id = ?1", params![chat_id.as_str()])
        .context("Failed to delete chat")?;

    info!(chat_id = %chat_id, "Chat permanently deleted");
    Ok(())
}

/// Search chats by title or message content
pub fn search_chats(query: &str) -> Result<Vec<Chat>> {
    if query.trim().is_empty() {
        return get_all_chats();
    }

    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    // Search in chat titles
    let mut stmt = conn
        .prepare(
            r#"
            SELECT DISTINCT c.id, c.title, c.created_at, c.updated_at, 
                   c.deleted_at, c.model_id, c.provider
            FROM chats c
            LEFT JOIN chats_fts fts ON c.rowid = fts.rowid
            LEFT JOIN messages m ON c.id = m.chat_id
            LEFT JOIN messages_fts mfts ON m.rowid = mfts.rowid
            WHERE c.deleted_at IS NULL 
              AND (chats_fts MATCH ?1 OR messages_fts MATCH ?1)
            ORDER BY c.updated_at DESC
            "#,
        )
        .context("Failed to prepare search query")?;

    let chats = stmt
        .query_map(params![query], row_to_chat)
        .context("Failed to search chats")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect search results")?;

    debug!(query = %query, count = chats.len(), "Chat search completed");
    Ok(chats)
}

// ============================================================================
// Message Operations
// ============================================================================

/// Save a message
pub fn save_message(message: &Message) -> Result<()> {
    save_message_internal(message, true)
}

/// Save a message without updating the chat's updated_at timestamp.
/// Used for mock data insertion where we want to preserve historical dates.
fn save_message_without_update(message: &Message) -> Result<()> {
    save_message_internal(message, false)
}

/// Internal message save with optional chat timestamp update
fn save_message_internal(message: &Message, update_chat_timestamp: bool) -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    conn.execute(
        r#"
        INSERT INTO messages (id, chat_id, role, content, created_at, tokens_used)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        ON CONFLICT(id) DO UPDATE SET
            content = excluded.content,
            tokens_used = excluded.tokens_used
        "#,
        params![
            message.id,
            message.chat_id.as_str(),
            message.role.as_str(),
            message.content,
            message.created_at.to_rfc3339(),
            message.tokens_used,
        ],
    )
    .context("Failed to save message")?;

    // Update the chat's updated_at timestamp (unless explicitly skipped for mock data)
    if update_chat_timestamp {
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE chats SET updated_at = ?2 WHERE id = ?1",
            params![message.chat_id.as_str(), now],
        )
        .context("Failed to update chat timestamp")?;
    }

    debug!(
        message_id = %message.id,
        chat_id = %message.chat_id,
        role = %message.role,
        "Message saved"
    );
    Ok(())
}

/// Get all messages for a chat, ordered by creation time
pub fn get_chat_messages(chat_id: &ChatId) -> Result<Vec<Message>> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, chat_id, role, content, created_at, tokens_used
            FROM messages
            WHERE chat_id = ?1
            ORDER BY created_at ASC
            "#,
        )
        .context("Failed to prepare get_chat_messages query")?;

    let messages = stmt
        .query_map(params![chat_id.as_str()], row_to_message)
        .context("Failed to query messages")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect messages")?;

    debug!(chat_id = %chat_id, count = messages.len(), "Retrieved chat messages");
    Ok(messages)
}

/// Get the last N messages for a chat
pub fn get_recent_messages(chat_id: &ChatId, limit: usize) -> Result<Vec<Message>> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, chat_id, role, content, created_at, tokens_used
            FROM messages
            WHERE chat_id = ?1
            ORDER BY created_at DESC
            LIMIT ?2
            "#,
        )
        .context("Failed to prepare get_recent_messages query")?;

    let mut messages: Vec<Message> = stmt
        .query_map(params![chat_id.as_str(), limit as i64], row_to_message)
        .context("Failed to query recent messages")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect recent messages")?;

    // Reverse to get chronological order
    messages.reverse();

    Ok(messages)
}

/// Get total token usage for a chat
pub fn get_chat_token_usage(chat_id: &ChatId) -> Result<u64> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let total: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(tokens_used), 0) FROM messages WHERE chat_id = ?1",
            params![chat_id.as_str()],
            |row| row.get(0),
        )
        .context("Failed to get token usage")?;

    Ok(total as u64)
}

/// Get chat count (active only)
pub fn get_chat_count() -> Result<usize> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM chats WHERE deleted_at IS NULL",
            [],
            |row| row.get(0),
        )
        .context("Failed to count chats")?;

    Ok(count as usize)
}

/// Prune chats deleted more than `days` ago
pub fn prune_old_deleted_chats(days: u32) -> Result<usize> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    let cutoff = Utc::now() - chrono::Duration::days(days as i64);

    // Get IDs of chats to delete
    let chat_ids: Vec<String> = {
        let mut stmt = conn
            .prepare("SELECT id FROM chats WHERE deleted_at IS NOT NULL AND deleted_at < ?1")
            .context("Failed to prepare prune query")?;

        let results = stmt
            .query_map(params![cutoff.to_rfc3339()], |row| row.get(0))
            .context("Failed to query chats to prune")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect chat IDs")?;
        results
    };

    // Delete messages for these chats
    for chat_id in &chat_ids {
        conn.execute("DELETE FROM messages WHERE chat_id = ?1", params![chat_id])
            .context("Failed to delete messages for pruned chat")?;
    }

    // Delete the chats
    let count = conn
        .execute(
            "DELETE FROM chats WHERE deleted_at IS NOT NULL AND deleted_at < ?1",
            params![cutoff.to_rfc3339()],
        )
        .context("Failed to prune old deleted chats")?;

    if count > 0 {
        info!(count, days, "Pruned old deleted chats");
    }

    Ok(count)
}

// ============================================================================
// Row Converters
// ============================================================================

/// Convert a database row to a Chat
fn row_to_chat(row: &rusqlite::Row) -> rusqlite::Result<Chat> {
    let id_str: String = row.get(0)?;
    let title: String = row.get(1)?;
    let created_at_str: String = row.get(2)?;
    let updated_at_str: String = row.get(3)?;
    let deleted_at_str: Option<String> = row.get(4)?;
    let model_id: String = row.get(5)?;
    let provider: String = row.get(6)?;

    let id = ChatId::parse(&id_str).unwrap_or_default();

    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let deleted_at = deleted_at_str.and_then(|s| {
        DateTime::parse_from_rfc3339(&s)
            .map(|dt| dt.with_timezone(&Utc))
            .ok()
    });

    Ok(Chat {
        id,
        title,
        created_at,
        updated_at,
        deleted_at,
        model_id,
        provider,
    })
}

/// Convert a database row to a Message
fn row_to_message(row: &rusqlite::Row) -> rusqlite::Result<Message> {
    let id: String = row.get(0)?;
    let chat_id_str: String = row.get(1)?;
    let role_str: String = row.get(2)?;
    let content: String = row.get(3)?;
    let created_at_str: String = row.get(4)?;
    let tokens_used: Option<u32> = row.get(5)?;

    let chat_id = ChatId::parse(&chat_id_str).unwrap_or_default();

    let role = MessageRole::parse(&role_str).unwrap_or(MessageRole::User);

    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    Ok(Message {
        id,
        chat_id,
        role,
        content,
        created_at,
        tokens_used,
    })
}

// ============================================================================
// Mock Data for Testing
// ============================================================================

/// Insert comprehensive mock chat data for visual testing.
/// Creates many chats across different time periods with long conversations
/// to verify search functionality and scrolling behavior.
#[allow(clippy::useless_vec)] // vec! is more readable for mock data definitions
pub fn insert_mock_data() -> Result<()> {
    use chrono::Duration;

    // First, ensure the database is initialized
    if AI_DB.get().is_none() {
        init_ai_db()?;
    }

    let now = Utc::now();

    // Helper to create a chat with messages
    let mut total_chats = 0;
    let mut total_messages = 0;

    // ========================================================================
    // TODAY - 5 chats
    // ========================================================================

    // Chat 1: Long conversation about Python (TODAY - scrollable)
    let chat1_id = ChatId::new();
    create_chat(&Chat {
        id: chat1_id,
        title: "Python Best Practices Discussion".to_string(),
        created_at: now - Duration::hours(2),
        updated_at: now - Duration::minutes(10),
        deleted_at: None,
        model_id: "claude-3-5-sonnet-20241022".to_string(),
        provider: "anthropic".to_string(),
    })?;
    total_chats += 1;

    let python_messages = vec![
        (MessageRole::User, "What are some Python best practices I should follow?"),
        (MessageRole::Assistant, "Great question! Here are the key Python best practices:\n\n## 1. Follow PEP 8 Style Guide\n\n```python\n# Good\ndef calculate_total(items: list[Item]) -> float:\n    return sum(item.price for item in items)\n\n# Bad\ndef calculateTotal(Items):\n    total=0\n    for i in Items:total+=i.price\n    return total\n```\n\n## 2. Use Type Hints\n\nType hints improve code readability and enable better IDE support:\n\n```python\nfrom typing import Optional, List\n\ndef find_user(user_id: int) -> Optional[User]:\n    return users.get(user_id)\n```\n\n## 3. Use Context Managers\n\n```python\nwith open('file.txt', 'r') as f:\n    content = f.read()\n```\n\nWould you like me to elaborate on any of these?"),
        (MessageRole::User, "Yes, tell me more about type hints"),
        (MessageRole::Assistant, "## Deep Dive into Type Hints\n\nType hints were introduced in Python 3.5 and have become increasingly important.\n\n### Basic Types\n\n```python\nname: str = \"Alice\"\nage: int = 30\nheight: float = 5.9\nis_active: bool = True\n```\n\n### Collection Types\n\n```python\nfrom typing import List, Dict, Set, Tuple\n\nnames: List[str] = [\"Alice\", \"Bob\"]\nages: Dict[str, int] = {\"Alice\": 30}\nunique_ids: Set[int] = {1, 2, 3}\ncoordinates: Tuple[float, float] = (10.5, 20.3)\n```\n\n### Modern Syntax (Python 3.9+)\n\n```python\n# You can use built-in types directly\nnames: list[str] = [\"Alice\", \"Bob\"]\nages: dict[str, int] = {\"Alice\": 30}\n```\n\n### Optional and Union\n\n```python\nfrom typing import Optional, Union\n\ndef find_user(id: int) -> Optional[User]:\n    return None  # or User instance\n\n# Union for multiple types\ndef process(data: Union[str, bytes]) -> str:\n    if isinstance(data, bytes):\n        return data.decode()\n    return data\n```\n\n### Generic Types\n\n```python\nfrom typing import TypeVar, Generic\n\nT = TypeVar('T')\n\nclass Stack(Generic[T]):\n    def __init__(self) -> None:\n        self._items: list[T] = []\n    \n    def push(self, item: T) -> None:\n        self._items.append(item)\n    \n    def pop(self) -> T:\n        return self._items.pop()\n```"),
        (MessageRole::User, "What about dataclasses?"),
        (MessageRole::Assistant, "## Python Dataclasses\n\nDataclasses reduce boilerplate for classes that primarily store data.\n\n### Basic Usage\n\n```python\nfrom dataclasses import dataclass\n\n@dataclass\nclass User:\n    name: str\n    age: int\n    email: str = \"\"\n```\n\nThis automatically generates:\n- `__init__`\n- `__repr__`\n- `__eq__`\n\n### Frozen (Immutable) Dataclasses\n\n```python\n@dataclass(frozen=True)\nclass Point:\n    x: float\n    y: float\n```\n\n### With Default Factory\n\n```python\nfrom dataclasses import dataclass, field\nfrom typing import List\n\n@dataclass\nclass Team:\n    name: str\n    members: List[str] = field(default_factory=list)\n```\n\n### Post-Init Processing\n\n```python\n@dataclass\nclass Rectangle:\n    width: float\n    height: float\n    area: float = field(init=False)\n    \n    def __post_init__(self):\n        self.area = self.width * self.height\n```"),
        (MessageRole::User, "Thanks! One more question - what about async/await?"),
        (MessageRole::Assistant, "## Async/Await in Python\n\nAsynchronous programming allows concurrent execution without threads.\n\n### Basic Async Function\n\n```python\nimport asyncio\n\nasync def fetch_data(url: str) -> dict:\n    # Simulate network delay\n    await asyncio.sleep(1)\n    return {\"url\": url, \"data\": \"...\"}\n```\n\n### Running Async Code\n\n```python\nasync def main():\n    result = await fetch_data(\"https://api.example.com\")\n    print(result)\n\nasyncio.run(main())\n```\n\n### Concurrent Execution\n\n```python\nasync def fetch_all(urls: list[str]) -> list[dict]:\n    tasks = [fetch_data(url) for url in urls]\n    results = await asyncio.gather(*tasks)\n    return results\n```\n\n### Async Context Managers\n\n```python\nclass AsyncDatabase:\n    async def __aenter__(self):\n        await self.connect()\n        return self\n    \n    async def __aexit__(self, *args):\n        await self.disconnect()\n\nasync def main():\n    async with AsyncDatabase() as db:\n        await db.query(\"SELECT * FROM users\")\n```\n\n### Async Iterators\n\n```python\nasync def read_lines(filename: str):\n    async with aiofiles.open(filename) as f:\n        async for line in f:\n            yield line.strip()\n\nasync def main():\n    async for line in read_lines(\"data.txt\"):\n        print(line)\n```"),
    ];

    for (i, (role, content)) in python_messages.iter().enumerate() {
        save_message_without_update(&Message {
            id: uuid::Uuid::new_v4().to_string(),
            chat_id: chat1_id,
            role: *role,
            content: content.to_string(),
            created_at: now - Duration::minutes(120 - i as i64 * 10),
            tokens_used: Some(content.len() as u32 / 4),
        })?;
        total_messages += 1;
    }

    // Chat 2: Quick question (TODAY)
    let chat2_id = ChatId::new();
    create_chat(&Chat {
        id: chat2_id,
        title: "Git Rebase vs Merge".to_string(),
        created_at: now - Duration::hours(1),
        updated_at: now - Duration::minutes(30),
        deleted_at: None,
        model_id: "gpt-4o".to_string(),
        provider: "openai".to_string(),
    })?;
    total_chats += 1;

    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat2_id,
        role: MessageRole::User,
        content: "What's the difference between git rebase and merge?".to_string(),
        created_at: now - Duration::minutes(35),
        tokens_used: Some(12),
    })?;
    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat2_id,
        role: MessageRole::Assistant,
        content: "## Git Merge vs Rebase\n\n**Merge** creates a new commit that combines two branches:\n```\n  A---B---C feature\n /         \\\nD---E---F---G main (merge commit)\n```\n\n**Rebase** replays your commits on top of another branch:\n```\n          A'--B'--C' feature\n         /\nD---E---F main\n```\n\n### When to use each:\n- **Merge**: Preserves history, good for shared branches\n- **Rebase**: Cleaner history, good for local/feature branches".to_string(),
        created_at: now - Duration::minutes(34),
        tokens_used: Some(85),
    })?;
    total_messages += 2;

    // Chat 3: Another today chat (searchable keyword: "typescript")
    let chat3_id = ChatId::new();
    create_chat(&Chat {
        id: chat3_id,
        title: "TypeScript Generics Tutorial".to_string(),
        created_at: now - Duration::hours(3),
        updated_at: now - Duration::hours(2),
        deleted_at: None,
        model_id: "claude-3-5-sonnet-20241022".to_string(),
        provider: "anthropic".to_string(),
    })?;
    total_chats += 1;

    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat3_id,
        role: MessageRole::User,
        content: "Explain TypeScript generics with examples".to_string(),
        created_at: now - Duration::hours(3),
        tokens_used: Some(8),
    })?;
    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat3_id,
        role: MessageRole::Assistant,
        content: "## TypeScript Generics\n\nGenerics allow you to write reusable, type-safe code.\n\n```typescript\nfunction identity<T>(arg: T): T {\n    return arg;\n}\n\nconst num = identity<number>(42);\nconst str = identity<string>(\"hello\");\n```\n\n### Generic Interfaces\n\n```typescript\ninterface Container<T> {\n    value: T;\n    getValue(): T;\n}\n```\n\n### Generic Constraints\n\n```typescript\ninterface Lengthwise {\n    length: number;\n}\n\nfunction logLength<T extends Lengthwise>(arg: T): void {\n    console.log(arg.length);\n}\n```".to_string(),
        created_at: now - Duration::hours(3) + Duration::minutes(1),
        tokens_used: Some(95),
    })?;
    total_messages += 2;

    // Chat 4: Docker question (TODAY)
    let chat4_id = ChatId::new();
    create_chat(&Chat {
        id: chat4_id,
        title: "Docker Compose Setup".to_string(),
        created_at: now - Duration::hours(4),
        updated_at: now - Duration::hours(3),
        deleted_at: None,
        model_id: "gpt-4o".to_string(),
        provider: "openai".to_string(),
    })?;
    total_chats += 1;

    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat4_id,
        role: MessageRole::User,
        content: "How do I set up Docker Compose for a Node.js app with PostgreSQL?".to_string(),
        created_at: now - Duration::hours(4),
        tokens_used: Some(15),
    })?;
    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat4_id,
        role: MessageRole::Assistant,
        content: "Here's a complete Docker Compose setup:\n\n```yaml\nversion: '3.8'\n\nservices:\n  app:\n    build: .\n    ports:\n      - \"3000:3000\"\n    environment:\n      - DATABASE_URL=postgresql://user:pass@db:5432/mydb\n    depends_on:\n      - db\n\n  db:\n    image: postgres:15\n    environment:\n      - POSTGRES_USER=user\n      - POSTGRES_PASSWORD=pass\n      - POSTGRES_DB=mydb\n    volumes:\n      - postgres_data:/var/lib/postgresql/data\n\nvolumes:\n  postgres_data:\n```\n\nAnd your Dockerfile:\n\n```dockerfile\nFROM node:20-alpine\nWORKDIR /app\nCOPY package*.json ./\nRUN npm install\nCOPY . .\nEXPOSE 3000\nCMD [\"npm\", \"start\"]\n```".to_string(),
        created_at: now - Duration::hours(4) + Duration::minutes(1),
        tokens_used: Some(120),
    })?;
    total_messages += 2;

    // Chat 5: React hooks (TODAY)
    let chat5_id = ChatId::new();
    create_chat(&Chat {
        id: chat5_id,
        title: "React useEffect Cleanup".to_string(),
        created_at: now - Duration::hours(5),
        updated_at: now - Duration::hours(4),
        deleted_at: None,
        model_id: "claude-3-5-sonnet-20241022".to_string(),
        provider: "anthropic".to_string(),
    })?;
    total_chats += 1;

    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat5_id,
        role: MessageRole::User,
        content: "When do I need cleanup in useEffect?".to_string(),
        created_at: now - Duration::hours(5),
        tokens_used: Some(10),
    })?;
    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat5_id,
        role: MessageRole::Assistant,
        content: "## useEffect Cleanup\n\nYou need cleanup when your effect:\n\n1. **Subscribes to events**\n```jsx\nuseEffect(() => {\n  window.addEventListener('resize', handler);\n  return () => window.removeEventListener('resize', handler);\n}, []);\n```\n\n2. **Sets up timers**\n```jsx\nuseEffect(() => {\n  const id = setInterval(tick, 1000);\n  return () => clearInterval(id);\n}, []);\n```\n\n3. **Opens connections**\n```jsx\nuseEffect(() => {\n  const ws = new WebSocket(url);\n  return () => ws.close();\n}, [url]);\n```".to_string(),
        created_at: now - Duration::hours(5) + Duration::minutes(1),
        tokens_used: Some(80),
    })?;
    total_messages += 2;

    // ========================================================================
    // YESTERDAY - 4 chats
    // ========================================================================
    let yesterday = now - Duration::days(1);

    // Chat 6: API Design (YESTERDAY) - Long conversation
    let chat6_id = ChatId::new();
    create_chat(&Chat {
        id: chat6_id,
        title: "RESTful API Design Principles".to_string(),
        created_at: yesterday - Duration::hours(5),
        updated_at: yesterday,
        deleted_at: None,
        model_id: "gpt-4o".to_string(),
        provider: "openai".to_string(),
    })?;
    total_chats += 1;

    let api_messages = vec![
        (MessageRole::User, "What are the best practices for designing a REST API?"),
        (MessageRole::Assistant, "## REST API Design Best Practices\n\n### 1. Use Nouns for Resources\n```\nGET /users          # List users\nGET /users/123      # Get user 123\nPOST /users         # Create user\nPUT /users/123      # Update user 123\nDELETE /users/123   # Delete user 123\n```\n\n### 2. Use HTTP Methods Correctly\n- **GET**: Read (idempotent)\n- **POST**: Create\n- **PUT**: Full update (idempotent)\n- **PATCH**: Partial update\n- **DELETE**: Remove (idempotent)\n\n### 3. Use Proper Status Codes\n- 200 OK\n- 201 Created\n- 204 No Content\n- 400 Bad Request\n- 401 Unauthorized\n- 403 Forbidden\n- 404 Not Found\n- 500 Internal Server Error"),
        (MessageRole::User, "What about pagination?"),
        (MessageRole::Assistant, "## API Pagination\n\n### Offset-Based (Simple)\n```\nGET /users?page=2&limit=20\n```\n\nResponse:\n```json\n{\n  \"data\": [...],\n  \"pagination\": {\n    \"page\": 2,\n    \"limit\": 20,\n    \"total\": 150,\n    \"pages\": 8\n  }\n}\n```\n\n### Cursor-Based (Scalable)\n```\nGET /users?cursor=abc123&limit=20\n```\n\nResponse:\n```json\n{\n  \"data\": [...],\n  \"next_cursor\": \"xyz789\",\n  \"has_more\": true\n}\n```\n\n### Link Headers (RFC 5988)\n```\nLink: </users?page=3>; rel=\"next\",\n      </users?page=1>; rel=\"prev\""),
        (MessageRole::User, "How do I handle errors?"),
        (MessageRole::Assistant, "## API Error Handling\n\n### Standard Error Response Format\n\n```json\n{\n  \"error\": {\n    \"code\": \"VALIDATION_ERROR\",\n    \"message\": \"Invalid input data\",\n    \"details\": [\n      {\n        \"field\": \"email\",\n        \"message\": \"Invalid email format\"\n      }\n    ],\n    \"request_id\": \"req_abc123\"\n  }\n}\n```\n\n### Error Codes by Category\n\n**Client Errors (4xx)**\n- 400: Validation failed\n- 401: Missing/invalid auth\n- 403: Insufficient permissions\n- 404: Resource not found\n- 409: Conflict (duplicate)\n- 422: Unprocessable entity\n- 429: Rate limited\n\n**Server Errors (5xx)**\n- 500: Internal error\n- 502: Bad gateway\n- 503: Service unavailable\n- 504: Gateway timeout"),
    ];

    for (i, (role, content)) in api_messages.iter().enumerate() {
        save_message_without_update(&Message {
            id: uuid::Uuid::new_v4().to_string(),
            chat_id: chat6_id,
            role: *role,
            content: content.to_string(),
            created_at: yesterday - Duration::hours(5) + Duration::minutes(i as i64 * 5),
            tokens_used: Some(content.len() as u32 / 4),
        })?;
        total_messages += 1;
    }

    // Chat 7: SQL Query (YESTERDAY)
    let chat7_id = ChatId::new();
    create_chat(&Chat {
        id: chat7_id,
        title: "Complex SQL JOIN Query".to_string(),
        created_at: yesterday - Duration::hours(8),
        updated_at: yesterday - Duration::hours(7),
        deleted_at: None,
        model_id: "claude-3-5-sonnet-20241022".to_string(),
        provider: "anthropic".to_string(),
    })?;
    total_chats += 1;

    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat7_id,
        role: MessageRole::User,
        content: "Help me write a SQL query to get users with their orders".to_string(),
        created_at: yesterday - Duration::hours(8),
        tokens_used: Some(15),
    })?;
    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat7_id,
        role: MessageRole::Assistant,
        content: "```sql\nSELECT \n    u.id,\n    u.name,\n    u.email,\n    COUNT(o.id) as order_count,\n    COALESCE(SUM(o.total), 0) as total_spent\nFROM users u\nLEFT JOIN orders o ON u.id = o.user_id\nWHERE u.active = true\nGROUP BY u.id, u.name, u.email\nHAVING COUNT(o.id) > 0\nORDER BY total_spent DESC\nLIMIT 100;\n```".to_string(),
        created_at: yesterday - Duration::hours(8) + Duration::minutes(1),
        tokens_used: Some(65),
    })?;
    total_messages += 2;

    // Chat 8: Kubernetes (YESTERDAY)
    let chat8_id = ChatId::new();
    create_chat(&Chat {
        id: chat8_id,
        title: "Kubernetes Deployment YAML".to_string(),
        created_at: yesterday - Duration::hours(10),
        updated_at: yesterday - Duration::hours(9),
        deleted_at: None,
        model_id: "gpt-4o".to_string(),
        provider: "openai".to_string(),
    })?;
    total_chats += 1;

    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat8_id,
        role: MessageRole::User,
        content: "Create a Kubernetes deployment for a web app".to_string(),
        created_at: yesterday - Duration::hours(10),
        tokens_used: Some(12),
    })?;
    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat8_id,
        role: MessageRole::Assistant,
        content: "```yaml\napiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: web-app\n  labels:\n    app: web-app\nspec:\n  replicas: 3\n  selector:\n    matchLabels:\n      app: web-app\n  template:\n    metadata:\n      labels:\n        app: web-app\n    spec:\n      containers:\n      - name: web-app\n        image: myapp:latest\n        ports:\n        - containerPort: 8080\n        resources:\n          limits:\n            memory: \"256Mi\"\n            cpu: \"500m\"\n        livenessProbe:\n          httpGet:\n            path: /health\n            port: 8080\n          initialDelaySeconds: 30\n---\napiVersion: v1\nkind: Service\nmetadata:\n  name: web-app-service\nspec:\n  selector:\n    app: web-app\n  ports:\n  - port: 80\n    targetPort: 8080\n  type: LoadBalancer\n```".to_string(),
        created_at: yesterday - Duration::hours(10) + Duration::minutes(1),
        tokens_used: Some(120),
    })?;
    total_messages += 2;

    // Chat 9: CSS Grid (YESTERDAY)
    let chat9_id = ChatId::new();
    create_chat(&Chat {
        id: chat9_id,
        title: "CSS Grid Layout Examples".to_string(),
        created_at: yesterday - Duration::hours(12),
        updated_at: yesterday - Duration::hours(11),
        deleted_at: None,
        model_id: "claude-3-5-sonnet-20241022".to_string(),
        provider: "anthropic".to_string(),
    })?;
    total_chats += 1;

    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat9_id,
        role: MessageRole::User,
        content: "Show me CSS Grid examples".to_string(),
        created_at: yesterday - Duration::hours(12),
        tokens_used: Some(8),
    })?;
    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat9_id,
        role: MessageRole::Assistant,
        content: "## CSS Grid Examples\n\n### Basic Grid\n```css\n.container {\n  display: grid;\n  grid-template-columns: repeat(3, 1fr);\n  gap: 20px;\n}\n```\n\n### Named Areas\n```css\n.layout {\n  display: grid;\n  grid-template-areas:\n    \"header header header\"\n    \"sidebar main main\"\n    \"footer footer footer\";\n  grid-template-columns: 200px 1fr 1fr;\n}\n\n.header { grid-area: header; }\n.sidebar { grid-area: sidebar; }\n.main { grid-area: main; }\n```".to_string(),
        created_at: yesterday - Duration::hours(12) + Duration::minutes(1),
        tokens_used: Some(75),
    })?;
    total_messages += 2;

    // ========================================================================
    // THIS WEEK - 5 chats (2-6 days ago)
    // ========================================================================

    // Chat 10: Rust Ownership (3 days ago) - Very long conversation for scrolling
    let three_days_ago = now - Duration::days(3);
    let chat10_id = ChatId::new();
    create_chat(&Chat {
        id: chat10_id,
        title: "Understanding Rust Ownership".to_string(),
        created_at: three_days_ago - Duration::hours(2),
        updated_at: three_days_ago,
        deleted_at: None,
        model_id: "claude-3-5-sonnet-20241022".to_string(),
        provider: "anthropic".to_string(),
    })?;
    total_chats += 1;

    let rust_messages = vec![
        (MessageRole::User, "Explain Rust ownership to me"),
        (MessageRole::Assistant, "## Rust Ownership\n\nOwnership is Rust's most unique feature and enables memory safety without garbage collection.\n\n### The Three Rules\n\n1. Each value has an owner\n2. Only one owner at a time\n3. Value is dropped when owner goes out of scope\n\n```rust\nfn main() {\n    let s1 = String::from(\"hello\"); // s1 owns the string\n    let s2 = s1;                     // ownership moves to s2\n    // println!(\"{}\", s1);          // ERROR! s1 no longer valid\n    println!(\"{}\", s2);              // OK\n}\n```"),
        (MessageRole::User, "What about borrowing?"),
        (MessageRole::Assistant, "## Borrowing in Rust\n\nBorrowing lets you reference data without taking ownership.\n\n### Immutable Borrows\n\n```rust\nfn calculate_length(s: &String) -> usize {\n    s.len()\n}\n\nfn main() {\n    let s = String::from(\"hello\");\n    let len = calculate_length(&s); // borrow s\n    println!(\"Length of '{}' is {}\", s, len); // s still valid\n}\n```\n\n### Mutable Borrows\n\n```rust\nfn change(s: &mut String) {\n    s.push_str(\", world\");\n}\n\nfn main() {\n    let mut s = String::from(\"hello\");\n    change(&mut s);\n    println!(\"{}\", s); // \"hello, world\"\n}\n```\n\n### Borrowing Rules\n\n1. You can have either ONE mutable reference OR any number of immutable references\n2. References must always be valid"),
        (MessageRole::User, "Can you explain lifetimes?"),
        (MessageRole::Assistant, "## Rust Lifetimes\n\nLifetimes ensure references are valid for as long as they're used.\n\n### Why Lifetimes?\n\n```rust\n// This won't compile - dangling reference\nfn dangling() -> &String {\n    let s = String::from(\"hello\");\n    &s  // s goes out of scope, reference would be invalid\n}\n```\n\n### Lifetime Annotations\n\n```rust\n// 'a is a lifetime parameter\nfn longest<'a>(x: &'a str, y: &'a str) -> &'a str {\n    if x.len() > y.len() { x } else { y }\n}\n```\n\nThis says: \"The returned reference will live at least as long as the shorter of x and y.\"\n\n### Lifetime Elision\n\nRust can often infer lifetimes:\n\n```rust\n// These are equivalent\nfn first_word(s: &str) -> &str { ... }\nfn first_word<'a>(s: &'a str) -> &'a str { ... }\n```\n\n### Static Lifetime\n\n```rust\nlet s: &'static str = \"I live forever!\";\n```"),
        (MessageRole::User, "What are smart pointers?"),
        (MessageRole::Assistant, "## Smart Pointers in Rust\n\n### Box<T> - Heap Allocation\n\n```rust\nlet b = Box::new(5);\nprintln!(\"b = {}\", b);\n\n// Useful for recursive types\nenum List {\n    Cons(i32, Box<List>),\n    Nil,\n}\n```\n\n### Rc<T> - Reference Counting\n\n```rust\nuse std::rc::Rc;\n\nlet a = Rc::new(5);\nlet b = Rc::clone(&a); // Increases ref count\nlet c = Rc::clone(&a);\n\nprintln!(\"count = {}\", Rc::strong_count(&a)); // 3\n```\n\n### Arc<T> - Thread-Safe Rc\n\n```rust\nuse std::sync::Arc;\nuse std::thread;\n\nlet data = Arc::new(vec![1, 2, 3]);\n\nfor _ in 0..3 {\n    let data = Arc::clone(&data);\n    thread::spawn(move || {\n        println!(\"{:?}\", data);\n    });\n}\n```\n\n### RefCell<T> - Interior Mutability\n\n```rust\nuse std::cell::RefCell;\n\nlet data = RefCell::new(5);\n\n*data.borrow_mut() += 1;\nprintln!(\"{}\", data.borrow()); // 6\n```"),
        (MessageRole::User, "How do I handle errors in Rust?"),
        (MessageRole::Assistant, "## Error Handling in Rust\n\n### Result<T, E>\n\n```rust\nuse std::fs::File;\nuse std::io::{self, Read};\n\nfn read_file(path: &str) -> Result<String, io::Error> {\n    let mut file = File::open(path)?;\n    let mut contents = String::new();\n    file.read_to_string(&mut contents)?;\n    Ok(contents)\n}\n```\n\n### The ? Operator\n\nPropagates errors automatically:\n\n```rust\nfn get_user(id: u32) -> Result<User, Error> {\n    let conn = database::connect()?;\n    let user = conn.query_user(id)?;\n    Ok(user)\n}\n```\n\n### Custom Error Types\n\n```rust\n#[derive(Debug)]\nenum AppError {\n    IoError(io::Error),\n    ParseError(String),\n    NotFound,\n}\n\nimpl From<io::Error> for AppError {\n    fn from(err: io::Error) -> Self {\n        AppError::IoError(err)\n    }\n}\n```\n\n### anyhow for Applications\n\n```rust\nuse anyhow::{Context, Result};\n\nfn main() -> Result<()> {\n    let config = read_config()\n        .context(\"Failed to read config\")?;\n    Ok(())\n}\n```\n\n### thiserror for Libraries\n\n```rust\nuse thiserror::Error;\n\n#[derive(Error, Debug)]\npub enum DataError {\n    #[error(\"database error: {0}\")]\n    Database(#[from] sqlx::Error),\n    #[error(\"not found: {0}\")]\n    NotFound(String),\n}\n```"),
    ];

    for (i, (role, content)) in rust_messages.iter().enumerate() {
        save_message_without_update(&Message {
            id: uuid::Uuid::new_v4().to_string(),
            chat_id: chat10_id,
            role: *role,
            content: content.to_string(),
            created_at: three_days_ago - Duration::hours(2) + Duration::minutes(i as i64 * 8),
            tokens_used: Some(content.len() as u32 / 4),
        })?;
        total_messages += 1;
    }

    // Chat 11: Authentication (4 days ago)
    let four_days_ago = now - Duration::days(4);
    let chat11_id = ChatId::new();
    create_chat(&Chat {
        id: chat11_id,
        title: "JWT Authentication Setup".to_string(),
        created_at: four_days_ago,
        updated_at: four_days_ago,
        deleted_at: None,
        model_id: "gpt-4o".to_string(),
        provider: "openai".to_string(),
    })?;
    total_chats += 1;

    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat11_id,
        role: MessageRole::User,
        content: "How do I implement JWT authentication?".to_string(),
        created_at: four_days_ago,
        tokens_used: Some(10),
    })?;
    save_message_without_update(&Message {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id: chat11_id,
        role: MessageRole::Assistant,
        content: "## JWT Authentication\n\n```javascript\nconst jwt = require('jsonwebtoken');\n\n// Generate token\nconst token = jwt.sign(\n  { userId: user.id },\n  process.env.JWT_SECRET,\n  { expiresIn: '24h' }\n);\n\n// Verify token middleware\nfunction authenticate(req, res, next) {\n  const token = req.headers.authorization?.split(' ')[1];\n  try {\n    const decoded = jwt.verify(token, process.env.JWT_SECRET);\n    req.user = decoded;\n    next();\n  } catch (err) {\n    res.status(401).json({ error: 'Invalid token' });\n  }\n}\n```".to_string(),
        created_at: four_days_ago + Duration::minutes(1),
        tokens_used: Some(85),
    })?;
    total_messages += 2;

    // Chat 12-14: More chats for variety
    for (i, (title, topic)) in [
        ("GraphQL Schema Design", "graphql"),
        ("WebSocket Implementation", "websocket"),
        ("CI/CD Pipeline Setup", "pipeline"),
    ]
    .iter()
    .enumerate()
    {
        let days_ago = now - Duration::days(5 + i as i64);
        let chat_id = ChatId::new();
        create_chat(&Chat {
            id: chat_id,
            title: title.to_string(),
            created_at: days_ago,
            updated_at: days_ago,
            deleted_at: None,
            model_id: if i % 2 == 0 {
                "claude-3-5-sonnet-20241022"
            } else {
                "gpt-4o"
            }
            .to_string(),
            provider: if i % 2 == 0 { "anthropic" } else { "openai" }.to_string(),
        })?;
        total_chats += 1;

        save_message_without_update(&Message {
            id: uuid::Uuid::new_v4().to_string(),
            chat_id,
            role: MessageRole::User,
            content: format!("Tell me about {}", topic),
            created_at: days_ago,
            tokens_used: Some(6),
        })?;
        save_message_without_update(&Message {
            id: uuid::Uuid::new_v4().to_string(),
            chat_id,
            role: MessageRole::Assistant,
            content: format!(
                "Here's an overview of {}...\n\n(This is mock content for testing)",
                topic
            ),
            created_at: days_ago + Duration::minutes(1),
            tokens_used: Some(20),
        })?;
        total_messages += 2;
    }

    // ========================================================================
    // OLDER - 5+ chats (8+ days ago)
    // ========================================================================

    for (i, title) in [
        "Machine Learning Basics",
        "Database Optimization",
        "Security Best Practices",
        "Microservices Architecture",
        "Testing Strategies",
        "Performance Tuning",
        "Code Review Guidelines",
    ]
    .iter()
    .enumerate()
    {
        let days_ago = now - Duration::days(10 + i as i64 * 3);
        let chat_id = ChatId::new();
        create_chat(&Chat {
            id: chat_id,
            title: title.to_string(),
            created_at: days_ago,
            updated_at: days_ago,
            deleted_at: None,
            model_id: "claude-3-5-sonnet-20241022".to_string(),
            provider: "anthropic".to_string(),
        })?;
        total_chats += 1;

        save_message_without_update(&Message {
            id: uuid::Uuid::new_v4().to_string(),
            chat_id,
            role: MessageRole::User,
            content: format!("Explain {} in detail", title.to_lowercase()),
            created_at: days_ago,
            tokens_used: Some(8),
        })?;
        save_message_without_update(&Message {
            id: uuid::Uuid::new_v4().to_string(),
            chat_id,
            role: MessageRole::Assistant,
            content: format!("## {}\n\nThis is a comprehensive topic...\n\n(Mock content for testing the older section)", title),
            created_at: days_ago + Duration::minutes(1),
            tokens_used: Some(25),
        })?;
        total_messages += 2;
    }

    info!(
        chat_count = total_chats,
        message_count = total_messages,
        "Comprehensive mock data inserted for AI visual testing"
    );

    Ok(())
}

/// Clear all mock data (for test cleanup)
pub fn clear_all_chats() -> Result<()> {
    let db = get_db()?;
    let conn = db
        .lock()
        .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;

    conn.execute("DELETE FROM messages", [])
        .context("Failed to delete all messages")?;
    conn.execute("DELETE FROM chats", [])
        .context("Failed to delete all chats")?;

    info!("All chats and messages cleared");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_path() {
        let path = get_ai_db_path();
        assert!(path.to_string_lossy().contains("ai-chats.sqlite"));
        assert!(path.to_string_lossy().contains(".sk/kit/db"));
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
//! The AI chats database is stored at `~/.sk/kit/ai-chats.db`.
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
pub mod storage;
pub mod window;

// Re-export commonly used types
pub use model::{Chat, ChatId, Message, MessageRole};
pub use storage::{
    clear_all_chats, create_chat, delete_chat, get_all_chats, get_chat, get_chat_messages,
    get_deleted_chats, init_ai_db, insert_mock_data, restore_chat, save_message, search_chats,
    update_chat_title,
};

// Re-export provider types
pub use config::{DetectedKeys, ModelInfo, ProviderConfig};
pub use providers::{AiProvider, ProviderMessage, ProviderRegistry};

// Re-export window functions
pub use window::{close_ai_window, is_ai_window_open, open_ai_window, set_ai_input, set_ai_search};

</file>

<file path="src/ai/window.rs">
//! AI Chat Window
//!
//! A separate floating window for AI chat, built with gpui-component.
//! This is completely independent from the main Script Kit launcher window.
//!
//! # Architecture
//!
//! The window follows a Raycast-style layout:
//! - Left sidebar: Chat history list with search, grouped by date (Today, Yesterday, This Week, Older)
//! - Right main panel: Welcome state ("Ask Anything") or chat messages
//! - Bottom: Input area + model picker + submit button

use anyhow::Result;
use chrono::{Datelike, NaiveDate, Utc};
use gpui::{
    div, hsla, point, prelude::*, px, rgb, size, svg, App, BoxShadow, Context, Entity, FocusHandle,
    Focusable, Hsla, IntoElement, KeyDownEvent, ParentElement, Render, ScrollHandle, SharedString,
    Styled, Subscription, Window, WindowBounds, WindowOptions,
};

// Import local IconName for SVG icons (has external_path() method)
use crate::designs::icon_variations::IconName as LocalIconName;

#[cfg(target_os = "macos")]
use cocoa::appkit::NSApp;
#[cfg(target_os = "macos")]
use cocoa::base::{id, nil};
use gpui_component::{
    button::{Button, ButtonCustomVariant, ButtonVariants},
    input::{Input, InputEvent, InputState},
    scroll::ScrollableElement,
    theme::{ActiveTheme, Theme as GpuiTheme, ThemeColor, ThemeMode},
    Icon, IconName, Root, Sizable,
};
#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};
use tracing::{debug, info};

use super::config::ModelInfo;
use super::model::{Chat, ChatId, Message, MessageRole};
use super::providers::ProviderRegistry;
use super::storage;
use crate::watcher::ThemeWatcher;

/// Events from the streaming thread
enum StreamingEvent {
    /// A chunk of text received
    Chunk(String),
    /// Streaming completed successfully
    Done,
    /// An error occurred
    Error(String),
}

/// Date group categories for sidebar organization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DateGroup {
    Today,
    Yesterday,
    ThisWeek,
    Older,
}

impl DateGroup {
    /// Get the display label for this group
    fn label(&self) -> &'static str {
        match self {
            DateGroup::Today => "Today",
            DateGroup::Yesterday => "Yesterday",
            DateGroup::ThisWeek => "This Week",
            DateGroup::Older => "Older",
        }
    }
}

/// Determine which date group a date belongs to
fn get_date_group(date: NaiveDate, today: NaiveDate) -> DateGroup {
    let days_ago = today.signed_duration_since(date).num_days();

    if days_ago == 0 {
        DateGroup::Today
    } else if days_ago == 1 {
        DateGroup::Yesterday
    } else if days_ago < 7
        && date.weekday().num_days_from_monday() < today.weekday().num_days_from_monday()
    {
        // Same week (and not earlier in a previous week)
        DateGroup::ThisWeek
    } else if days_ago < 7 {
        DateGroup::ThisWeek
    } else {
        DateGroup::Older
    }
}

/// Group chats by date categories
fn group_chats_by_date(chats: &[Chat]) -> Vec<(DateGroup, Vec<&Chat>)> {
    let today = Utc::now().date_naive();

    let mut today_chats: Vec<&Chat> = Vec::new();
    let mut yesterday_chats: Vec<&Chat> = Vec::new();
    let mut this_week_chats: Vec<&Chat> = Vec::new();
    let mut older_chats: Vec<&Chat> = Vec::new();

    for chat in chats {
        let chat_date = chat.updated_at.date_naive();
        match get_date_group(chat_date, today) {
            DateGroup::Today => today_chats.push(chat),
            DateGroup::Yesterday => yesterday_chats.push(chat),
            DateGroup::ThisWeek => this_week_chats.push(chat),
            DateGroup::Older => older_chats.push(chat),
        }
    }

    let mut groups = Vec::new();
    if !today_chats.is_empty() {
        groups.push((DateGroup::Today, today_chats));
    }
    if !yesterday_chats.is_empty() {
        groups.push((DateGroup::Yesterday, yesterday_chats));
    }
    if !this_week_chats.is_empty() {
        groups.push((DateGroup::ThisWeek, this_week_chats));
    }
    if !older_chats.is_empty() {
        groups.push((DateGroup::Older, older_chats));
    }

    groups
}

/// Generate a contextual mock AI response based on the user's message
/// Used for demo/testing when no AI providers are configured
fn generate_mock_response(user_message: &str) -> String {
    let msg_lower = user_message.to_lowercase();

    // Contextual responses based on common patterns
    if msg_lower.contains("hello") || msg_lower.contains("hi") || msg_lower.starts_with("hey") {
        return "Hello! I'm Script Kit's AI assistant running in demo mode. Since no API key is configured, I'm providing mock responses. To enable real AI, set `SCRIPT_KIT_ANTHROPIC_API_KEY` or `SCRIPT_KIT_OPENAI_API_KEY` in your environment.".to_string();
    }

    if msg_lower.contains("script") || msg_lower.contains("automation") {
        return "Script Kit is a powerful automation tool! Here are some things you can do:\n\n1. **Create scripts** - Write TypeScript/JavaScript to automate tasks\n2. **Use prompts** - `arg()`, `editor()`, `div()` for interactive UIs\n3. **Hotkeys** - Bind scripts to global keyboard shortcuts\n4. **Snippets** - Text expansion with dynamic content\n\nTry running a script with `Cmd+;` to see it in action!".to_string();
    }

    if msg_lower.contains("help") || msg_lower.contains("how") {
        return "I'm here to help! In demo mode, I can explain Script Kit concepts:\n\n• **Scripts** live in `~/.sk/kit/scripts/`\n• **SDK** provides `arg()`, `div()`, `editor()`, and more\n• **Hotkeys** are configured in script metadata\n• **This AI chat** works with Claude or GPT when you add an API key\n\nWhat would you like to know more about?".to_string();
    }

    if msg_lower.contains("code") || msg_lower.contains("example") {
        return "Here's a simple Script Kit example:\n\n```typescript\n// Name: Hello World\n// Shortcut: cmd+shift+h\n\nconst name = await arg(\"What's your name?\");\nawait div(`<h1>Hello, ${name}!</h1>`);\n```\n\nThis creates a script that:\n1. Asks for your name via a prompt\n2. Displays a greeting in an HTML view\n\nSave this to `~/.sk/kit/scripts/hello.ts` and run it!".to_string();
    }

    if msg_lower.contains("api") || msg_lower.contains("key") || msg_lower.contains("configure") {
        return "To enable real AI responses, configure an API key:\n\n**For Claude (Anthropic):**\n```bash\nexport SCRIPT_KIT_ANTHROPIC_API_KEY=\"sk-ant-...\"\n```\n\n**For GPT (OpenAI):**\n```bash\nexport SCRIPT_KIT_OPENAI_API_KEY=\"sk-...\"\n```\n\nAdd these to your `~/.zshrc` or `~/.sk/kit/.env` file, then restart Script Kit.".to_string();
    }

    // Default response for unrecognized queries
    format!(
        "I received your message: \"{}\"\n\n\
        I'm running in **demo mode** because no AI API key is configured. \
        My responses are pre-written examples.\n\n\
        To get real AI responses:\n\
        1. Get an API key from Anthropic or OpenAI\n\
        2. Set `SCRIPT_KIT_ANTHROPIC_API_KEY` or `SCRIPT_KIT_OPENAI_API_KEY`\n\
        3. Restart Script Kit\n\n\
        Try asking about \"scripts\", \"help\", or \"code examples\" to see more demo responses!",
        user_message.chars().take(50).collect::<String>()
    )
}

/// Global handle to the AI window
static AI_WINDOW: std::sync::OnceLock<std::sync::Mutex<Option<gpui::WindowHandle<Root>>>> =
    std::sync::OnceLock::new();

/// Global handle to the AiApp entity (for updating state from outside)
static AI_APP_ENTITY: std::sync::OnceLock<std::sync::Mutex<Option<Entity<AiApp>>>> =
    std::sync::OnceLock::new();

/// The main AI chat application view
pub struct AiApp {
    /// All chats (cached from storage)
    chats: Vec<Chat>,

    /// Currently selected chat ID
    selected_chat_id: Option<ChatId>,

    /// Cache of last message preview per chat (ChatId -> preview text)
    message_previews: std::collections::HashMap<ChatId, String>,

    /// Chat input state (using gpui-component's Input)
    input_state: Entity<InputState>,

    /// Search input state for sidebar
    search_state: Entity<InputState>,

    /// Current search query
    search_query: String,

    /// Whether the sidebar is collapsed
    sidebar_collapsed: bool,

    /// Provider registry with available AI providers
    provider_registry: ProviderRegistry,

    /// Available models from all providers
    available_models: Vec<ModelInfo>,

    /// Currently selected model for new chats
    selected_model: Option<ModelInfo>,

    /// Focus handle for keyboard navigation
    focus_handle: FocusHandle,

    /// Subscriptions to keep alive
    _subscriptions: Vec<Subscription>,

    // === Streaming State ===
    /// Whether we're currently streaming a response
    is_streaming: bool,

    /// Content accumulated during streaming
    streaming_content: String,

    /// Messages for the currently selected chat (cached for display)
    current_messages: Vec<Message>,

    /// Scroll handle for the messages area (for auto-scrolling during streaming)
    messages_scroll_handle: ScrollHandle,

    /// Cached box shadows from theme (avoid reloading theme on every render)
    cached_box_shadows: Vec<BoxShadow>,
}

impl AiApp {
    /// Create a new AiApp
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Initialize storage
        if let Err(e) = storage::init_ai_db() {
            tracing::error!(error = %e, "Failed to initialize AI database");
        }

        // Load chats from storage
        let chats = storage::get_all_chats().unwrap_or_default();
        let selected_chat_id = chats.first().map(|c| c.id);

        // Load message previews for each chat
        let mut message_previews = std::collections::HashMap::new();
        for chat in &chats {
            if let Ok(messages) = storage::get_recent_messages(&chat.id, 1) {
                if let Some(last_msg) = messages.first() {
                    // Truncate preview to ~60 chars
                    let preview: String = last_msg.content.chars().take(60).collect();
                    let preview = if preview.len() < last_msg.content.len() {
                        format!("{}...", preview.trim())
                    } else {
                        preview
                    };
                    message_previews.insert(chat.id, preview);
                }
            }
        }

        // Initialize provider registry from environment
        let provider_registry = ProviderRegistry::from_environment();
        let available_models = provider_registry.get_all_models();

        // Select default model (prefer Claude, then GPT-4o)
        let selected_model = available_models
            .iter()
            .find(|m| m.id.contains("claude-3-5-sonnet"))
            .or_else(|| available_models.iter().find(|m| m.id == "gpt-4o"))
            .or_else(|| available_models.first())
            .cloned();

        info!(
            providers = provider_registry.provider_ids().len(),
            models = available_models.len(),
            selected = selected_model
                .as_ref()
                .map(|m| m.display_name.as_str())
                .unwrap_or("none"),
            "AI providers initialized"
        );

        // Create input states
        let input_state = cx.new(|cx| InputState::new(window, cx).placeholder("Ask anything..."));

        let search_state = cx.new(|cx| InputState::new(window, cx).placeholder("Search chats..."));

        let focus_handle = cx.focus_handle();

        // Subscribe to input changes and Enter key
        let input_sub = cx.subscribe_in(&input_state, window, {
            move |this, _, ev: &InputEvent, window, cx| match ev {
                InputEvent::Change => this.on_input_change(cx),
                InputEvent::PressEnter { .. } => this.submit_message(window, cx),
                _ => {}
            }
        });

        // Subscribe to search changes
        let search_sub = cx.subscribe_in(&search_state, window, {
            move |this, _, ev: &InputEvent, _window, cx| {
                if matches!(ev, InputEvent::Change) {
                    this.on_search_change(cx);
                }
            }
        });

        // Load messages for the selected chat
        let current_messages = selected_chat_id
            .and_then(|id| storage::get_chat_messages(&id).ok())
            .unwrap_or_default();

        info!(chat_count = chats.len(), "AI app initialized");

        // Pre-compute box shadows from theme (avoid reloading on every render)
        let cached_box_shadows = Self::compute_box_shadows();

        Self {
            chats,
            selected_chat_id,
            message_previews,
            input_state,
            search_state,
            search_query: String::new(),
            sidebar_collapsed: false,
            provider_registry,
            available_models,
            selected_model,
            focus_handle,
            _subscriptions: vec![input_sub, search_sub],
            // Streaming state
            is_streaming: false,
            streaming_content: String::new(),
            current_messages,
            messages_scroll_handle: ScrollHandle::new(),
            cached_box_shadows,
        }
    }

    /// Handle input changes
    fn on_input_change(&mut self, _cx: &mut Context<Self>) {
        // TODO: Handle input changes (e.g., streaming, auto-complete)
    }

    /// Handle model selection change
    fn on_model_change(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(model) = self.available_models.get(index) {
            info!(
                model_id = model.id,
                model_name = model.display_name,
                provider = model.provider,
                "Model selected"
            );
            self.selected_model = Some(model.clone());
            cx.notify();
        }
    }

    /// Handle search query changes - filters chats in real-time as user types
    fn on_search_change(&mut self, cx: &mut Context<Self>) {
        let query = self.search_state.read(cx).value().to_string();
        self.search_query = query.clone();

        debug!(query = %query, "Search query changed");

        // If search is not empty, filter chats
        if !query.trim().is_empty() {
            // Use simple case-insensitive title matching for responsiveness
            // FTS search is available but can fail on special characters
            let query_lower = query.to_lowercase();
            let all_chats = storage::get_all_chats().unwrap_or_default();
            self.chats = all_chats
                .into_iter()
                .filter(|chat| chat.title.to_lowercase().contains(&query_lower))
                .collect();

            debug!(results = self.chats.len(), "Search filtered chats");

            // Always select first result when filtering
            if !self.chats.is_empty() {
                let first_id = self.chats[0].id;
                if self.selected_chat_id != Some(first_id) {
                    self.selected_chat_id = Some(first_id);
                    // Load messages for the selected chat
                    self.current_messages =
                        storage::get_chat_messages(&first_id).unwrap_or_default();
                }
            } else {
                self.selected_chat_id = None;
                self.current_messages = Vec::new();
            }
        } else {
            // Reload all chats when search is cleared
            self.chats = storage::get_all_chats().unwrap_or_default();
            // Keep current selection if it still exists, otherwise select first
            if let Some(id) = self.selected_chat_id {
                if !self.chats.iter().any(|c| c.id == id) {
                    self.selected_chat_id = self.chats.first().map(|c| c.id);
                    if let Some(new_id) = self.selected_chat_id {
                        self.current_messages =
                            storage::get_chat_messages(&new_id).unwrap_or_default();
                    }
                }
            }
        }

        cx.notify();
    }

    /// Create a new chat
    fn create_chat(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Option<ChatId> {
        // Get model and provider from selected model, or use defaults
        let (model_id, provider) = self
            .selected_model
            .as_ref()
            .map(|m| (m.id.clone(), m.provider.clone()))
            .unwrap_or_else(|| {
                (
                    "claude-3-5-sonnet-20241022".to_string(),
                    "anthropic".to_string(),
                )
            });

        // Create a new chat with selected model
        let chat = Chat::new(&model_id, &provider);
        let id = chat.id;

        // Save to storage
        if let Err(e) = storage::create_chat(&chat) {
            tracing::error!(error = %e, "Failed to create chat");
            return None;
        }

        // Add to cache and select it
        self.chats.insert(0, chat);
        self.select_chat(id, window, cx);

        info!(chat_id = %id, model = model_id, "New chat created");
        Some(id)
    }

    /// Select a chat
    fn select_chat(&mut self, id: ChatId, _window: &mut Window, cx: &mut Context<Self>) {
        self.selected_chat_id = Some(id);

        // Load messages for this chat
        self.current_messages = storage::get_chat_messages(&id).unwrap_or_default();

        // Scroll to bottom to show latest messages
        self.messages_scroll_handle.scroll_to_bottom();

        // Clear any streaming state
        self.is_streaming = false;
        self.streaming_content.clear();

        cx.notify();
    }

    /// Delete the currently selected chat (soft delete)
    fn delete_selected_chat(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_chat_id {
            if let Err(e) = storage::delete_chat(&id) {
                tracing::error!(error = %e, "Failed to delete chat");
                return;
            }

            // Remove from visible list and select next
            self.chats.retain(|c| c.id != id);
            self.selected_chat_id = self.chats.first().map(|c| c.id);

            cx.notify();
        }
    }

    /// Submit the current input as a message
    fn submit_message(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let content = self.input_state.read(cx).value().to_string();

        if content.trim().is_empty() {
            return;
        }

        // Don't allow new messages while streaming
        if self.is_streaming {
            return;
        }

        // If no chat selected, create a new one
        let chat_id = if let Some(id) = self.selected_chat_id {
            id
        } else {
            match self.create_chat(window, cx) {
                Some(id) => id,
                None => {
                    tracing::error!("Failed to create chat for message submission");
                    return;
                }
            }
        };

        // Update chat title if this is the first message
        if let Some(chat) = self.chats.iter_mut().find(|c| c.id == chat_id) {
            if chat.title == "New Chat" {
                let new_title = Chat::generate_title_from_content(&content);
                chat.set_title(&new_title);

                // Persist title update
                if let Err(e) = storage::update_chat_title(&chat_id, &new_title) {
                    tracing::error!(error = %e, "Failed to update chat title");
                }
            }
        }

        // Create and save user message
        let user_message = Message::user(chat_id, &content);
        if let Err(e) = storage::save_message(&user_message) {
            tracing::error!(error = %e, "Failed to save user message");
            return;
        }

        // Add to current messages for display
        self.current_messages.push(user_message);

        // Scroll to bottom to show the new message
        self.messages_scroll_handle.scroll_to_bottom();

        // Update message preview cache
        let preview: String = content.chars().take(60).collect();
        let preview = if preview.len() < content.len() {
            format!("{}...", preview.trim())
        } else {
            preview
        };
        self.message_previews.insert(chat_id, preview);

        // Clear the input
        self.input_state.update(cx, |state, cx| {
            state.set_value("", window, cx);
        });

        info!(
            chat_id = %chat_id,
            content_len = content.len(),
            "User message submitted"
        );

        // Start streaming response
        self.start_streaming_response(chat_id, cx);

        cx.notify();
    }

    /// Start streaming an AI response (or mock response if no providers configured)
    fn start_streaming_response(&mut self, chat_id: ChatId, cx: &mut Context<Self>) {
        // Check if we have a model selected - if not, use mock mode
        let use_mock_mode = self.selected_model.is_none() || self.available_models.is_empty();

        if use_mock_mode {
            info!(chat_id = %chat_id, "No AI providers configured - using mock mode");
            self.start_mock_streaming_response(chat_id, cx);
            return;
        }

        // Get the selected model
        let model = match &self.selected_model {
            Some(m) => m.clone(),
            None => {
                tracing::error!("No model selected for streaming");
                return;
            }
        };

        // Find the provider for this model
        let provider = match self.provider_registry.find_provider_for_model(&model.id) {
            Some(p) => p.clone(),
            None => {
                tracing::error!(model_id = model.id, "No provider found for model");
                return;
            }
        };

        // Build messages for the API call
        let api_messages: Vec<super::providers::ProviderMessage> = self
            .current_messages
            .iter()
            .map(|m| super::providers::ProviderMessage {
                role: m.role.to_string(),
                content: m.content.clone(),
            })
            .collect();

        // Set streaming state
        self.is_streaming = true;
        self.streaming_content.clear();

        info!(
            chat_id = %chat_id,
            model = model.id,
            provider = model.provider,
            message_count = api_messages.len(),
            "Starting AI streaming response"
        );

        // Use a shared buffer for streaming content
        let shared_content = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
        let shared_done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let shared_error = std::sync::Arc::new(std::sync::Mutex::new(None::<String>));

        let model_id = model.id.clone();
        let content_clone = shared_content.clone();
        let done_clone = shared_done.clone();
        let error_clone = shared_error.clone();

        // Spawn background thread for streaming
        std::thread::spawn(move || {
            let result = provider.stream_message(
                &api_messages,
                &model_id,
                Box::new(move |chunk| {
                    if let Ok(mut content) = content_clone.lock() {
                        content.push_str(&chunk);
                    }
                }),
            );

            match result {
                Ok(()) => {
                    done_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                }
                Err(e) => {
                    if let Ok(mut err) = error_clone.lock() {
                        *err = Some(e.to_string());
                    }
                    done_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                }
            }
        });

        // Poll for streaming updates using background executor
        let content_for_poll = shared_content.clone();
        let done_for_poll = shared_done.clone();
        let error_for_poll = shared_error.clone();

        cx.spawn(async move |this, cx| {
            use gpui::Timer;
            loop {
                Timer::after(std::time::Duration::from_millis(50)).await;

                // Check if done or errored
                if done_for_poll.load(std::sync::atomic::Ordering::SeqCst) {
                    // Get final content
                    let final_content = content_for_poll.lock().ok().map(|c| c.clone());
                    let error = error_for_poll.lock().ok().and_then(|e| e.clone());

                    let _ = cx.update(|cx| {
                        this.update(cx, |app, cx| {
                            if let Some(err) = error {
                                tracing::error!(error = err, "Streaming error");
                                app.is_streaming = false;
                                app.streaming_content.clear();
                            } else if let Some(content) = final_content {
                                app.streaming_content = content;
                                app.finish_streaming(chat_id, cx);
                            }
                            cx.notify();
                        })
                    });
                    break;
                }

                // Update with current content
                if let Ok(content) = content_for_poll.lock() {
                    if !content.is_empty() {
                        let current = content.clone();
                        let _ = cx.update(|cx| {
                            this.update(cx, |app, cx| {
                                app.streaming_content = current;
                                // Auto-scroll to bottom as new content arrives
                                app.messages_scroll_handle.scroll_to_bottom();
                                cx.notify();
                            })
                        });
                    }
                }
            }
        })
        .detach();
    }

    /// Start a mock streaming response for testing/demo when no AI providers are configured
    fn start_mock_streaming_response(&mut self, chat_id: ChatId, cx: &mut Context<Self>) {
        // Set streaming state
        self.is_streaming = true;
        self.streaming_content.clear();

        // Get the last user message to generate a contextual mock response
        let user_message = self
            .current_messages
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_default();

        // Generate a mock response based on the user's message
        let mock_response = generate_mock_response(&user_message);

        info!(
            chat_id = %chat_id,
            user_message_len = user_message.len(),
            mock_response_len = mock_response.len(),
            "Starting mock streaming response"
        );

        // Simulate streaming by revealing the response word by word
        let words: Vec<String> = mock_response
            .split_inclusive(char::is_whitespace)
            .map(|s| s.to_string())
            .collect();

        cx.spawn(async move |this, cx| {
            use gpui::Timer;

            let mut accumulated = String::new();
            let mut delay_counter = 0u64;

            for word in words {
                // Vary delay slightly based on word position (30-60ms range)
                delay_counter = delay_counter.wrapping_add(17); // Simple pseudo-variation
                let delay = 30 + (delay_counter % 30);
                Timer::after(std::time::Duration::from_millis(delay)).await;

                accumulated.push_str(&word);

                let current_content = accumulated.clone();
                let _ = cx.update(|cx| {
                    this.update(cx, |app, cx| {
                        app.streaming_content = current_content;
                        // Auto-scroll to bottom as new content arrives
                        app.messages_scroll_handle.scroll_to_bottom();
                        cx.notify();
                    })
                });
            }

            // Small delay before finishing
            Timer::after(std::time::Duration::from_millis(100)).await;

            // Finish streaming
            let _ = cx.update(|cx| {
                this.update(cx, |app, cx| {
                    app.finish_streaming(chat_id, cx);
                })
            });
        })
        .detach();
    }

    /// Finish streaming and save the assistant message
    fn finish_streaming(&mut self, chat_id: ChatId, cx: &mut Context<Self>) {
        if !self.streaming_content.is_empty() {
            // Create and save assistant message
            let assistant_message = Message::assistant(chat_id, &self.streaming_content);
            if let Err(e) = storage::save_message(&assistant_message) {
                tracing::error!(error = %e, "Failed to save assistant message");
            }

            // Add to current messages
            self.current_messages.push(assistant_message);

            // Update message preview
            let preview: String = self.streaming_content.chars().take(60).collect();
            let preview = if preview.len() < self.streaming_content.len() {
                format!("{}...", preview.trim())
            } else {
                preview
            };
            self.message_previews.insert(chat_id, preview);

            info!(
                chat_id = %chat_id,
                content_len = self.streaming_content.len(),
                "Streaming response complete"
            );
        }

        self.is_streaming = false;
        self.streaming_content.clear();
        cx.notify();
    }

    /// Get the currently selected chat
    fn get_selected_chat(&self) -> Option<&Chat> {
        self.selected_chat_id
            .and_then(|id| self.chats.iter().find(|c| c.id == id))
    }

    /// Render the search input
    fn render_search(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        // Fixed height container to prevent layout shift when typing
        div()
            .w_full()
            .h(px(36.)) // Fixed height to prevent layout shift
            .flex()
            .items_center()
            .child(
                Input::new(&self.search_state)
                    .w_full()
                    .small()
                    .focus_bordered(false), // Disable default focus border (too bright)
            )
    }

    /// Toggle sidebar visibility
    fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        self.sidebar_collapsed = !self.sidebar_collapsed;
        cx.notify();
    }

    /// Render the sidebar toggle button using the Sidebar icon from our icon library
    fn render_sidebar_toggle(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // Use opacity to indicate state - dimmed when collapsed
        let icon_color = if self.sidebar_collapsed {
            cx.theme().muted_foreground.opacity(0.5)
        } else {
            cx.theme().muted_foreground
        };

        div()
            .id("sidebar-toggle")
            .flex()
            .items_center()
            .justify_center()
            .size(px(24.))
            .rounded_md()
            .cursor_pointer()
            .hover(|s| s.bg(cx.theme().muted.opacity(0.3)))
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.toggle_sidebar(cx);
                }),
            )
            .child(
                svg()
                    .external_path(LocalIconName::Sidebar.external_path())
                    .size(px(16.))
                    .text_color(icon_color),
            )
    }

    /// Render the chats sidebar with date groupings
    fn render_sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // If sidebar is collapsed, just show a thin strip with toggle button
        if self.sidebar_collapsed {
            return div()
                .flex()
                .flex_col()
                .w(px(48.))
                .h_full()
                .bg(cx.theme().sidebar)
                .border_r_1()
                .border_color(cx.theme().sidebar_border)
                .items_center()
                // Top row - aligned with traffic lights (h=28px to match window chrome)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_end()
                        .w_full()
                        .h(px(28.))
                        .px_2()
                        .child(self.render_sidebar_toggle(cx)),
                )
                // New chat button below
                .child(
                    div().pt_1().child(
                        Button::new("new-chat-collapsed")
                            .ghost()
                            .xsmall()
                            .icon(IconName::Plus)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.create_chat(window, cx);
                            })),
                    ),
                )
                .into_any_element();
        }

        let selected_id = self.selected_chat_id;
        let date_groups = group_chats_by_date(&self.chats);

        // Build a custom sidebar with date groupings using divs
        // This gives us more control over the layout than SidebarGroup
        div()
            .flex()
            .flex_col()
            .w(px(240.))
            .h_full()
            .bg(cx.theme().sidebar)
            .border_r_1()
            .border_color(cx.theme().sidebar_border)
            // Top row - sidebar toggle aligned with traffic lights (right side of that row)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_end() // Push to right side (traffic lights are on left)
                    .w_full()
                    .h(px(28.)) // Match traffic light row height
                    .px_2()
                    .child(self.render_sidebar_toggle(cx)),
            )
            // Header with new chat button and search
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .px_2()
                    .pb_2()
                    .gap_2()
                    // New chat button row
                    .child(
                        div().flex().items_center().justify_end().w_full().child(
                            Button::new("new-chat")
                                .ghost()
                                .xsmall()
                                .icon(IconName::Plus)
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.create_chat(window, cx);
                                })),
                        ),
                    )
                    .child(self.render_search(cx)),
            )
            // Scrollable chat list with date groups
            // Note: overflow_y_scrollbar() wraps the element in a Scrollable container
            // that uses size_full(), so flex_1() goes on the wrapper, not the inner content
            .child(
                div()
                    .flex()
                    .flex_col()
                    .px_2()
                    .pb_2()
                    .gap_3()
                    .children(date_groups.into_iter().map(|(group, chats)| {
                        self.render_date_group(group, chats, selected_id, cx)
                    }))
                    .overflow_y_scrollbar()
                    .flex_1(),
            )
            .into_any_element()
    }

    /// Render a date group section (Today, Yesterday, This Week, Older)
    fn render_date_group(
        &self,
        group: DateGroup,
        chats: Vec<&Chat>,
        selected_id: Option<ChatId>,
        cx: &mut Context<Self>,
    ) -> gpui::Div {
        div()
            .flex()
            .flex_col()
            .w_full()
            .gap_1()
            // Group header
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(cx.theme().muted_foreground)
                    .px_1()
                    .py_1()
                    .child(group.label()),
            )
            // Chat items
            .children(
                chats
                    .into_iter()
                    .map(|chat| self.render_chat_item(chat, selected_id, cx)),
            )
    }

    /// Render a single chat item with title and preview
    fn render_chat_item(
        &self,
        chat: &Chat,
        selected_id: Option<ChatId>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let chat_id = chat.id;
        let is_selected = selected_id == Some(chat_id);

        let title: SharedString = if chat.title.is_empty() {
            "New Chat".into()
        } else {
            chat.title.clone().into()
        };

        let preview = self.message_previews.get(&chat_id).cloned();

        // Create a custom chat item with title and preview
        div()
            .id(SharedString::from(format!("chat-{}", chat_id)))
            .flex()
            .flex_col()
            .w_full()
            .px_2()
            .py_1()
            .rounded_md()
            .cursor_pointer()
            .when(is_selected, |d| d.bg(cx.theme().sidebar_accent))
            .when(!is_selected, |d| {
                d.hover(|d| d.bg(cx.theme().sidebar_accent.opacity(0.5)))
            })
            .on_click(cx.listener(move |this, _, window, cx| {
                this.select_chat(chat_id, window, cx);
            }))
            .child(
                // Title
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(cx.theme().sidebar_foreground)
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(title),
            )
            .when_some(preview, |d, preview_text| {
                // Clean up preview: skip markdown headings, find actual content
                let clean_preview: String = preview_text
                    .lines()
                    .map(|line| line.trim())
                    .find(|line| {
                        // Skip empty lines
                        !line.is_empty()
                        // Skip markdown headings
                        && !line.starts_with('#')
                        // Skip code fence markers
                        && !line.starts_with("```")
                        // Skip horizontal rules
                        && !line.chars().all(|c| c == '-' || c == '*' || c == '_')
                    })
                    .unwrap_or("")
                    .chars()
                    .take(50)
                    .collect();

                d.child(
                    // Preview (muted, smaller text, single line only)
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_ellipsis()
                        .child(clean_preview),
                )
            })
    }

    /// Render the model picker button
    /// Clicking cycles to the next model; shows current model name
    fn render_model_picker(&self, cx: &mut Context<Self>) -> impl IntoElement {
        if self.available_models.is_empty() {
            // No models available - show message
            return div()
                .flex()
                .items_center()
                .px_2()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("No AI providers configured")
                .into_any_element();
        }

        // Get current model display name
        let model_label: SharedString = self
            .selected_model
            .as_ref()
            .map(|m| m.display_name.clone())
            .unwrap_or_else(|| "Select Model".to_string())
            .into();

        // Model picker button - clicking cycles through models
        Button::new("model-picker")
            .ghost()
            .xsmall()
            .icon(IconName::ChevronDown)
            .child(model_label)
            .on_click(cx.listener(|this, _, _window, cx| {
                this.cycle_model(cx);
            }))
            .into_any_element()
    }

    /// Cycle to the next model in the list
    fn cycle_model(&mut self, cx: &mut Context<Self>) {
        if self.available_models.is_empty() {
            return;
        }

        // Find current index
        let current_idx = self
            .selected_model
            .as_ref()
            .and_then(|sm| self.available_models.iter().position(|m| m.id == sm.id))
            .unwrap_or(0);

        // Cycle to next
        let next_idx = (current_idx + 1) % self.available_models.len();
        self.on_model_change(next_idx, cx);
    }

    /// Render the welcome state (no chat selected or empty chat)
    fn render_welcome(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .flex_1()
            .gap_4()
            .child(
                div()
                    .text_xl()
                    .text_color(cx.theme().foreground)
                    .child("Ask Anything"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Start a conversation with AI"),
            )
    }

    /// Render a single message bubble
    fn render_message(&self, message: &Message, cx: &mut Context<Self>) -> impl IntoElement {
        let is_user = message.role == MessageRole::User;

        div()
            .flex()
            .flex_col()
            .w_full()
            .mb_3()
            .child(
                // Role label
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(cx.theme().muted_foreground)
                    .mb_1()
                    .child(if is_user { "You" } else { "Assistant" }),
            )
            .child(
                // Message content
                div()
                    .w_full()
                    .p_3()
                    .rounded_md()
                    .when(is_user, |d| d.bg(cx.theme().secondary))
                    .when(!is_user, |d| d.bg(cx.theme().muted.opacity(0.3)))
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().foreground)
                            .child(message.content.clone()),
                    ),
            )
    }

    /// Render streaming content (assistant response in progress)
    fn render_streaming_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .w_full()
            .mb_3()
            .child(
                // Role label with streaming indicator
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .mb_1()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(cx.theme().muted_foreground)
                            .child("Assistant"),
                    )
                    .child(
                        // Streaming indicator
                        div().text_xs().text_color(cx.theme().accent).child("●"),
                    ),
            )
            .child(
                // Streaming content
                div()
                    .w_full()
                    .p_3()
                    .rounded_md()
                    .bg(cx.theme().muted.opacity(0.3))
                    .child(div().text_sm().text_color(cx.theme().foreground).child(
                        if self.streaming_content.is_empty() {
                            "...".to_string()
                        } else {
                            self.streaming_content.clone()
                        },
                    )),
            )
    }

    /// Render the messages area
    fn render_messages(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let streaming_element = if self.is_streaming {
            Some(self.render_streaming_content(cx))
        } else {
            None
        };

        // Messages list with vertical scrollbar
        // Note: The container (in render_main_panel) handles flex_1 for sizing
        // We use size_full() here to fill the bounded container
        div()
            .id("messages-scroll-container")
            .flex()
            .flex_col()
            .p_3()
            .gap_3()
            .size_full()
            // Render all messages
            .children(
                self.current_messages
                    .iter()
                    .map(|msg| self.render_message(msg, cx)),
            )
            // Show streaming content if streaming
            .children(streaming_element)
            .overflow_y_scroll()
            .track_scroll(&self.messages_scroll_handle)
    }

    /// Render the main chat panel
    fn render_main_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let has_selection = self.selected_chat_id.is_some();

        // Build titlebar
        let titlebar = div()
            .id("ai-titlebar")
            .flex()
            .items_center()
            .justify_between()
            .h(px(36.))
            .px_3()
            .bg(cx.theme().title_bar)
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                // Chat title (truncated)
                div()
                    .flex_1()
                    .overflow_hidden()
                    .text_ellipsis()
                    .text_sm()
                    .text_color(cx.theme().foreground)
                    .child(
                        self.get_selected_chat()
                            .map(|c| {
                                if c.title.is_empty() {
                                    "New Chat".to_string()
                                } else {
                                    c.title.clone()
                                }
                            })
                            .unwrap_or_else(|| "AI Chat".to_string()),
                    ),
            )
            .when(has_selection, |d| {
                d.child(
                    div().flex().items_center().gap_1().child(
                        Button::new("delete-chat")
                            .ghost()
                            .xsmall()
                            .icon(IconName::Delete)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.delete_selected_chat(cx);
                            })),
                    ),
                )
            });

        // Build input area at bottom - Raycast-style layout:
        // Row 1: [+ icon] [input field with magenta border]
        // Row 2: [Model picker with spinner] ... [Submit ↵] | [Actions ⌘K]

        // Use theme accent color for input border (follows theme)
        let input_border_color = cx.theme().accent;

        let input_area = div()
            .flex()
            .flex_col()
            .w_full()
            .bg(cx.theme().title_bar)
            .px_3()
            .pt_3()
            .pb_2() // Reduced bottom padding
            .gap_2()
            // Input row with + icon and accent border
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .w_full()
                    // Plus button on the left using SVG icon (properly centered)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(28.))
                            .rounded_full()
                            .border_1()
                            .border_color(cx.theme().muted_foreground.opacity(0.4))
                            .cursor_pointer()
                            .hover(|s| s.bg(cx.theme().muted.opacity(0.3)))
                            .child(
                                svg()
                                    .external_path(LocalIconName::Plus.external_path())
                                    .size(px(14.))
                                    .text_color(cx.theme().muted_foreground),
                            ),
                    )
                    // Input field with subtle accent border
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .rounded_lg()
                            .border_1()
                            .border_color(input_border_color.opacity(0.6)) // Subtle border
                            .overflow_hidden()
                            .child(
                                Input::new(&self.input_state).w_full().focus_bordered(false), // Disable default focus ring
                            ),
                    ),
            )
            // Bottom row: Model picker left, actions right (reduced padding)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .overflow_hidden()
                    // Left side: Model picker with potential spinner
                    .child(self.render_model_picker(cx))
                    // Right side: Submit and Actions as text labels
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .flex_shrink_0()
                            // Submit ↵ - clickable text
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .px_2()
                                    .py(px(2.)) // Reduced vertical padding
                                    .rounded_md()
                                    .cursor_pointer()
                                    .hover(|s| s.bg(cx.theme().muted.opacity(0.3)))
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(|this, _, window, cx| {
                                            this.submit_message(window, cx);
                                        }),
                                    )
                                    .child("Submit ↵"),
                            )
                            // Divider
                            .child(div().w(px(1.)).h(px(16.)).bg(cx.theme().border))
                            // Actions ⌘K - placeholder for future actions menu
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .px_2()
                                    .py(px(2.)) // Reduced vertical padding to match Submit
                                    .rounded_md()
                                    .cursor_pointer()
                                    .hover(|s| s.bg(cx.theme().muted.opacity(0.3)))
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Actions ⌘K"),
                            ),
                    ),
            );

        // Determine what to show in the content area
        let has_messages = !self.current_messages.is_empty() || self.is_streaming;

        // Build main layout
        // Structure: titlebar (fixed) -> content area (flex_1, scrollable) -> input area (fixed)
        div()
            .flex_1()
            .flex()
            .flex_col()
            .h_full()
            .overflow_hidden()
            // Titlebar (fixed height)
            .child(titlebar)
            // Content area - this wrapper gets flex_1 to fill remaining space
            // The scrollable content goes inside this bounded container
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .child(if has_messages {
                        self.render_messages(cx).into_any_element()
                    } else {
                        self.render_welcome(cx).into_any_element()
                    }),
            )
            // Input area (fixed height, always visible at bottom)
            .child(input_area)
    }

    /// Get cached box shadows (computed once at construction)
    fn create_box_shadows(&self) -> Vec<BoxShadow> {
        self.cached_box_shadows.clone()
    }

    /// Compute box shadows from theme configuration (called once at construction)
    fn compute_box_shadows() -> Vec<BoxShadow> {
        let theme = crate::theme::load_theme();
        let shadow_config = theme.get_drop_shadow();

        if !shadow_config.enabled {
            return vec![];
        }

        // Convert hex color to HSLA
        let r = ((shadow_config.color >> 16) & 0xFF) as f32 / 255.0;
        let g = ((shadow_config.color >> 8) & 0xFF) as f32 / 255.0;
        let b = (shadow_config.color & 0xFF) as f32 / 255.0;

        // Simple RGB to HSL conversion
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;

        let (h, s) = if max == min {
            (0.0, 0.0)
        } else {
            let d = max - min;
            let s = if l > 0.5 {
                d / (2.0 - max - min)
            } else {
                d / (max + min)
            };
            let h = if max == r {
                (g - b) / d + if g < b { 6.0 } else { 0.0 }
            } else if max == g {
                (b - r) / d + 2.0
            } else {
                (r - g) / d + 4.0
            };
            (h / 6.0, s)
        };

        vec![BoxShadow {
            color: hsla(h, s, l, shadow_config.opacity),
            offset: point(px(shadow_config.offset_x), px(shadow_config.offset_y)),
            blur_radius: px(shadow_config.blur_radius),
            spread_radius: px(shadow_config.spread_radius),
        }]
    }

    /// Update cached box shadows when theme changes
    pub fn update_theme(&mut self, _cx: &mut Context<Self>) {
        self.cached_box_shadows = Self::compute_box_shadows();
    }
}

impl Focusable for AiApp {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for AiApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let box_shadows = self.create_box_shadows();

        div()
            .flex()
            .flex_row()
            .size_full()
            .bg(cx.theme().background)
            .shadow(box_shadows)
            .text_color(cx.theme().foreground)
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                // Handle keyboard shortcuts
                let key = event.keystroke.key.as_str();
                let modifiers = &event.keystroke.modifiers;

                // platform modifier = Cmd on macOS, Ctrl on Windows/Linux
                if modifiers.platform {
                    match key {
                        "n" => {
                            this.create_chat(window, cx);
                        }
                        "enter" | "return" => this.submit_message(window, cx),
                        // Cmd+\ to toggle sidebar (like Raycast)
                        "\\" | "backslash" => this.toggle_sidebar(cx),
                        // Cmd+B also toggles sidebar (common convention)
                        "b" => this.toggle_sidebar(cx),
                        _ => {}
                    }
                }
            }))
            .child(self.render_sidebar(cx))
            .child(self.render_main_panel(cx))
    }
}

/// Convert a u32 hex color to Hsla
#[inline]
fn hex_to_hsla(hex: u32) -> Hsla {
    rgb(hex).into()
}

/// Map Script Kit's ColorScheme to gpui-component's ThemeColor
///
/// NOTE: We intentionally do NOT apply the user's opacity.* values to theme colors here.
/// The opacity values are for window-level transparency (vibrancy effect),
/// not for making UI elements semi-transparent. UI elements should remain solid.
fn map_scriptkit_to_gpui_theme(sk_theme: &crate::theme::Theme) -> ThemeColor {
    let colors = &sk_theme.colors;

    // Get default dark theme as base and override with Script Kit colors
    let mut theme_color = *ThemeColor::dark();

    // Main background and foreground
    theme_color.background = hex_to_hsla(colors.background.main);
    theme_color.foreground = hex_to_hsla(colors.text.primary);

    // Accent colors (Script Kit yellow/gold)
    theme_color.accent = hex_to_hsla(colors.accent.selected);
    theme_color.accent_foreground = hex_to_hsla(colors.text.primary);

    // Border
    theme_color.border = hex_to_hsla(colors.ui.border);
    theme_color.input = hex_to_hsla(colors.ui.border);

    // List/sidebar colors
    theme_color.list = hex_to_hsla(colors.background.main);
    theme_color.list_active = hex_to_hsla(colors.accent.selected_subtle);
    theme_color.list_active_border = hex_to_hsla(colors.accent.selected);
    theme_color.list_hover = hex_to_hsla(colors.accent.selected_subtle);
    theme_color.list_even = hex_to_hsla(colors.background.main);
    theme_color.list_head = hex_to_hsla(colors.background.title_bar);

    // Sidebar (use slightly lighter background)
    theme_color.sidebar = hex_to_hsla(colors.background.title_bar);
    theme_color.sidebar_foreground = hex_to_hsla(colors.text.primary);
    theme_color.sidebar_border = hex_to_hsla(colors.ui.border);
    theme_color.sidebar_accent = hex_to_hsla(colors.accent.selected_subtle);
    theme_color.sidebar_accent_foreground = hex_to_hsla(colors.text.primary);
    theme_color.sidebar_primary = hex_to_hsla(colors.accent.selected);
    theme_color.sidebar_primary_foreground = hex_to_hsla(colors.text.primary);

    // Primary (accent-colored buttons) - yellow/gold background with dark text/icons
    // Use explicit black Hsla for foreground to ensure icon visibility against yellow
    theme_color.primary = hex_to_hsla(colors.accent.selected);
    // Black with full opacity - using Hsla directly since rgb(0x000000).into() may have issues
    theme_color.primary_foreground = Hsla {
        h: 0.0,
        s: 0.0,
        l: 0.0, // Black (0% lightness)
        a: 1.0, // Full opacity
    };
    theme_color.primary_hover = hex_to_hsla(colors.accent.selected).opacity(0.9);
    theme_color.primary_active = hex_to_hsla(colors.accent.selected).opacity(0.8);

    // Secondary (muted buttons)
    theme_color.secondary = hex_to_hsla(colors.background.search_box);
    theme_color.secondary_foreground = hex_to_hsla(colors.text.primary);
    theme_color.secondary_hover = hex_to_hsla(colors.background.title_bar);
    theme_color.secondary_active = hex_to_hsla(colors.background.title_bar);

    // Muted (disabled states, subtle elements)
    theme_color.muted = hex_to_hsla(colors.background.search_box);
    theme_color.muted_foreground = hex_to_hsla(colors.text.muted);

    // Title bar
    theme_color.title_bar = hex_to_hsla(colors.background.title_bar);
    theme_color.title_bar_border = hex_to_hsla(colors.ui.border);

    // Popover
    theme_color.popover = hex_to_hsla(colors.background.main);
    theme_color.popover_foreground = hex_to_hsla(colors.text.primary);

    // Status colors
    theme_color.success = hex_to_hsla(colors.ui.success);
    theme_color.success_foreground = hex_to_hsla(colors.text.primary);
    theme_color.danger = hex_to_hsla(colors.ui.error);
    theme_color.danger_foreground = hex_to_hsla(colors.text.primary);
    theme_color.warning = hex_to_hsla(colors.ui.warning);
    theme_color.warning_foreground = hex_to_hsla(colors.text.primary);
    theme_color.info = hex_to_hsla(colors.ui.info);
    theme_color.info_foreground = hex_to_hsla(colors.text.primary);

    // Scrollbar
    theme_color.scrollbar = hex_to_hsla(colors.background.main);
    theme_color.scrollbar_thumb = hex_to_hsla(colors.text.dimmed);
    theme_color.scrollbar_thumb_hover = hex_to_hsla(colors.text.muted);

    // Caret (cursor) - cyan by default
    theme_color.caret = hex_to_hsla(0x00ffff);

    // Selection
    theme_color.selection = hex_to_hsla(colors.accent.selected_subtle);

    // Ring (focus ring) - use a more subtle version of the accent color
    // The full accent is too bright for focus borders
    let mut ring_color = hex_to_hsla(colors.accent.selected);
    ring_color.a = 0.5; // 50% opacity for a subtler focus ring
    theme_color.ring = ring_color;

    // Tab colors
    theme_color.tab = hex_to_hsla(colors.background.main);
    theme_color.tab_active = hex_to_hsla(colors.background.search_box);
    theme_color.tab_active_foreground = hex_to_hsla(colors.text.primary);
    theme_color.tab_foreground = hex_to_hsla(colors.text.secondary);
    theme_color.tab_bar = hex_to_hsla(colors.background.title_bar);

    debug!(
        background = format!("#{:06x}", colors.background.main),
        accent = format!("#{:06x}", colors.accent.selected),
        "Script Kit theme mapped to gpui-component (AI window)"
    );

    theme_color
}

/// Initialize gpui-component theme and sync with Script Kit theme
fn ensure_theme_initialized(cx: &mut App) {
    // First, initialize gpui-component (this sets up the default theme)
    gpui_component::init(cx);

    // Load Script Kit's theme
    let sk_theme = crate::theme::load_theme();

    // Map Script Kit colors to gpui-component ThemeColor
    let custom_colors = map_scriptkit_to_gpui_theme(&sk_theme);

    // Apply the custom colors to the global theme
    let theme = GpuiTheme::global_mut(cx);
    theme.colors = custom_colors;
    theme.mode = ThemeMode::Dark; // Script Kit uses dark mode by default

    info!("AI window theme synchronized with Script Kit");
}

/// Toggle the AI window (open if closed, bring to front if open)
///
/// The AI window behaves as a NORMAL window (not a floating panel):
/// - Can go behind other windows when it loses focus
/// - Hotkey brings it to front and focuses it
/// - Does NOT affect other windows (main window, notes window)
/// - Does NOT hide the app when closed
pub fn open_ai_window(cx: &mut App) -> Result<()> {
    use crate::logging;

    logging::log("AI", "open_ai_window called - checking state");

    // Ensure gpui-component theme is initialized before opening window
    ensure_theme_initialized(cx);

    let window_handle = AI_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
    let mut guard = window_handle.lock().unwrap();

    // Check if window already exists and is valid
    if let Some(ref handle) = *guard {
        // Window exists - check if it's valid
        let window_valid = handle
            .update(cx, |_root, window, _cx| {
                // Window is valid - bring it to front and focus it
                window.activate_window();
            })
            .is_ok();

        if window_valid {
            logging::log("AI", "AI window exists - bringing to front and focusing");
            // Activate the app to ensure the window can receive focus
            cx.activate(true);
            return Ok(());
        }

        // Window handle was invalid, fall through to create new window
        logging::log("AI", "AI window handle was invalid - creating new");
        *guard = None;
    }

    // Create new window
    logging::log("AI", "Creating new AI window");
    info!("Opening new AI window");

    // Load theme to determine window background appearance (vibrancy)
    let theme = crate::theme::load_theme();
    let window_background = if theme.is_vibrancy_enabled() {
        gpui::WindowBackgroundAppearance::Blurred
    } else {
        gpui::WindowBackgroundAppearance::Opaque
    };

    let window_options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::centered(
            None,
            size(px(900.), px(700.)),
            cx,
        ))),
        titlebar: Some(gpui::TitlebarOptions {
            title: Some("Script Kit AI".into()),
            appears_transparent: true,
            ..Default::default()
        }),
        window_background,
        focus: true,
        show: true,
        // IMPORTANT: Use Normal window kind (not PopUp) so it behaves like a regular window
        // This allows it to go behind other windows and participate in normal window ordering
        kind: gpui::WindowKind::Normal,
        ..Default::default()
    };

    // Create a holder for the AiApp entity so we can store it
    let ai_app_holder: std::sync::Arc<std::sync::Mutex<Option<Entity<AiApp>>>> =
        std::sync::Arc::new(std::sync::Mutex::new(None));
    let ai_app_holder_clone = ai_app_holder.clone();

    let handle = cx.open_window(window_options, |window, cx| {
        let view = cx.new(|cx| AiApp::new(window, cx));
        // Store the AiApp entity for later access
        *ai_app_holder_clone.lock().unwrap() = Some(view.clone());
        cx.new(|cx| Root::new(view, window, cx))
    })?;

    // Store the AiApp entity globally
    let app_entity_holder = AI_APP_ENTITY.get_or_init(|| std::sync::Mutex::new(None));
    *app_entity_holder.lock().unwrap() = ai_app_holder.lock().unwrap().take();

    // Activate the app and window so user can immediately start typing
    cx.activate(true);
    let _ = handle.update(cx, |_root, window, _cx| {
        window.activate_window();
    });

    *guard = Some(handle);

    // NOTE: We do NOT configure as floating panel - this is a normal window
    // that can go behind other windows

    // Theme hot-reload watcher for AI window
    // Spawns a background task that watches ~/.sk/kit/theme.json for changes
    let app_entity_holder_ref = AI_APP_ENTITY.get_or_init(|| std::sync::Mutex::new(None));
    if let Some(ai_app) = app_entity_holder_ref.lock().unwrap().clone() {
        let ai_app_for_theme = ai_app.clone();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            let (mut theme_watcher, theme_rx) = ThemeWatcher::new();
            if theme_watcher.start().is_err() {
                return;
            }
            loop {
                gpui::Timer::after(std::time::Duration::from_millis(200)).await;
                if theme_rx.try_recv().is_ok() {
                    info!("AI window: theme.json changed, reloading");
                    let _ = cx.update(|cx| {
                        // Re-sync gpui-component theme with updated Script Kit theme
                        crate::theme::sync_gpui_component_theme(cx);
                        // Notify the AI window to re-render with new colors
                        ai_app_for_theme.update(cx, |_app, cx| {
                            cx.notify();
                        });
                    });
                }
            }
        })
        .detach();
    }

    Ok(())
}

/// Close the AI window
pub fn close_ai_window(cx: &mut App) {
    let window_handle = AI_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
    let mut guard = window_handle.lock().unwrap();

    if let Some(handle) = guard.take() {
        let _ = handle.update(cx, |_, window, _| {
            window.remove_window();
        });
    }

    // Also clear the AiApp entity reference
    let app_entity_holder = AI_APP_ENTITY.get_or_init(|| std::sync::Mutex::new(None));
    *app_entity_holder.lock().unwrap() = None;
}

/// Check if the AI window is currently open
///
/// Returns true if the AI window exists and is valid.
/// This is used by other parts of the app to check if AI is open
/// without affecting it.
pub fn is_ai_window_open() -> bool {
    let window_handle = AI_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
    let guard = window_handle.lock().unwrap();
    guard.is_some()
}

/// Set the search filter text in the AI window.
/// Used for testing the search functionality via stdin commands.
pub fn set_ai_search(cx: &mut App, query: &str) {
    use crate::logging;

    // Get the AI window handle for the window context
    let window_handle = AI_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
    let window_guard = window_handle.lock().unwrap();

    // Get the AiApp entity
    let app_entity_holder = AI_APP_ENTITY.get_or_init(|| std::sync::Mutex::new(None));
    let app_guard = app_entity_holder.lock().unwrap();

    if let (Some(handle), Some(app_entity)) = (window_guard.as_ref(), app_guard.as_ref()) {
        let query_owned = query.to_string();
        let app_entity_clone = app_entity.clone();

        let _ = handle.update(cx, |_root, window, cx| {
            app_entity_clone.update(cx, |app, cx| {
                // Set the search input value
                app.search_state.update(cx, |state, cx| {
                    state.set_value(query_owned.clone(), window, cx);
                });
                // Trigger the search change handler
                app.on_search_change(cx);
                logging::log("AI", &format!("Search filter set to: {}", query_owned));
            });
        });
    } else {
        logging::log("AI", "Cannot set search - AI window not open");
    }
}

/// Set the main input text in the AI window and optionally submit.
/// Used for testing the streaming functionality via stdin commands.
pub fn set_ai_input(cx: &mut App, text: &str, submit: bool) {
    use crate::logging;

    // Get the AI window handle for the window context
    let window_handle = AI_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
    let window_guard = window_handle.lock().unwrap();

    // Get the AiApp entity
    let app_entity_holder = AI_APP_ENTITY.get_or_init(|| std::sync::Mutex::new(None));
    let app_guard = app_entity_holder.lock().unwrap();

    if let (Some(handle), Some(app_entity)) = (window_guard.as_ref(), app_guard.as_ref()) {
        let text_owned = text.to_string();
        let app_entity_clone = app_entity.clone();

        let _ = handle.update(cx, |_root, window, cx| {
            app_entity_clone.update(cx, |app, cx| {
                // Set the input value
                app.input_state.update(cx, |state, cx| {
                    state.set_value(text_owned.clone(), window, cx);
                });
                logging::log("AI", &format!("Input set to: {}", text_owned));

                // Optionally submit the message (triggers streaming)
                if submit {
                    app.submit_message(window, cx);
                    logging::log("AI", "Message submitted - streaming started");
                }
            });
        });
    } else {
        logging::log("AI", "Cannot set input - AI window not open");
    }
}

/// Configure the AI window as a floating panel (always on top).
///
/// This sets:
/// - NSFloatingWindowLevel (3) - floats above normal windows
/// - NSWindowCollectionBehaviorMoveToActiveSpace - moves to current space when shown
/// - Disabled window restoration - prevents macOS position caching
#[cfg(target_os = "macos")]
fn configure_ai_as_floating_panel() {
    use crate::logging;
    use std::ffi::CStr;

    unsafe {
        let app: id = NSApp();
        let windows: id = msg_send![app, windows];
        let count: usize = msg_send![windows, count];

        for i in 0..count {
            let window: id = msg_send![windows, objectAtIndex: i];
            let title: id = msg_send![window, title];

            if title != nil {
                let title_cstr: *const i8 = msg_send![title, UTF8String];
                if !title_cstr.is_null() {
                    let title_str = CStr::from_ptr(title_cstr).to_string_lossy();

                    if title_str == "Script Kit AI" {
                        // Found the AI window - configure it

                        // NSFloatingWindowLevel = 3
                        let floating_level: i32 = 3;
                        let _: () = msg_send![window, setLevel:floating_level];

                        // NSWindowCollectionBehaviorMoveToActiveSpace = 2
                        let collection_behavior: u64 = 2;
                        let _: () = msg_send![window, setCollectionBehavior:collection_behavior];

                        // Disable window restoration
                        let _: () = msg_send![window, setRestorable:false];

                        logging::log(
                            "PANEL",
                            "AI window configured as floating panel (level=3, MoveToActiveSpace)",
                        );
                        return;
                    }
                }
            }
        }

        logging::log(
            "PANEL",
            "Warning: AI window not found by title for floating panel config",
        );
    }
}

#[cfg(not(target_os = "macos"))]
fn configure_ai_as_floating_panel() {
    // No-op on non-macOS platforms
}

</file>

<file path="src/ai/config.rs">
//! AI provider configuration and environment variable detection.
//!
//! This module handles automatic discovery of AI provider API keys from environment
//! variables using the `SCRIPT_KIT_*_API_KEY` pattern for security.
//!
//! # Environment Variable Pattern
//!
//! API keys are detected with the `SCRIPT_KIT_` prefix:
//! - `SCRIPT_KIT_OPENAI_API_KEY` -> OpenAI provider
//! - `SCRIPT_KIT_ANTHROPIC_API_KEY` -> Anthropic provider
//!
//! This prefix ensures users explicitly configure keys for Script Kit,
//! rather than accidentally exposing keys from other applications.

use std::env;

/// Represents a detected AI provider configuration.
#[derive(Debug, Clone)]
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
}

/// Detected API keys from environment.
#[derive(Debug, Default)]
pub struct DetectedKeys {
    pub openai: Option<String>,
    pub anthropic: Option<String>,
    pub google: Option<String>,
    pub groq: Option<String>,
    pub openrouter: Option<String>,
}

impl DetectedKeys {
    /// Scan environment variables for API keys.
    ///
    /// Looks for the `SCRIPT_KIT_*_API_KEY` pattern and collects all found keys.
    pub fn from_environment() -> Self {
        Self {
            openai: env::var(env_vars::OPENAI_API_KEY)
                .ok()
                .filter(|s| !s.is_empty()),
            anthropic: env::var(env_vars::ANTHROPIC_API_KEY)
                .ok()
                .filter(|s| !s.is_empty()),
            google: env::var(env_vars::GOOGLE_API_KEY)
                .ok()
                .filter(|s| !s.is_empty()),
            groq: env::var(env_vars::GROQ_API_KEY)
                .ok()
                .filter(|s| !s.is_empty()),
            openrouter: env::var(env_vars::OPENROUTER_API_KEY)
                .ok()
                .filter(|s| !s.is_empty()),
        }
    }

    /// Check if any API keys were detected.
    pub fn has_any(&self) -> bool {
        self.openai.is_some()
            || self.anthropic.is_some()
            || self.google.is_some()
            || self.groq.is_some()
            || self.openrouter.is_some()
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
    pub fn anthropic() -> Vec<ModelInfo> {
        vec![
            ModelInfo::new(
                "claude-3-5-sonnet-20241022",
                "Claude 3.5 Sonnet",
                "anthropic",
                true,
                200_000,
            ),
            ModelInfo::new(
                "claude-3-5-haiku-20241022",
                "Claude 3.5 Haiku",
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
}

</file>

</files>
📊 Pack Summary:
────────────────
  Total Files: 6 files
  Search Mode: ripgrep (fast)
  Total Tokens: ~40.9K (40,903 exact)
  Total Chars: 177,369 chars
       Output: -

📁 Extensions Found:
────────────────────
  .rs

📂 Top 10 Files (by tokens):
──────────────────────────
     15.5K - src/ai/window.rs
     13.8K - src/ai/storage.rs
      6.3K - src/ai/providers.rs
      2.6K - src/ai/config.rs
      2.3K - src/ai/model.rs
       462 - src/ai/mod.rs
