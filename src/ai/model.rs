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
