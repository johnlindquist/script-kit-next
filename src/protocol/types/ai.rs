use serde::{Deserialize, Serialize};

// ============================================================
// AI CHAT SDK API
// ============================================================

/// Chat information returned by AI SDK API responses
///
/// Used for `aiListChats()`, `aiGetActiveChat()`, and other chat-related operations.
/// Contains metadata about a chat session without the full message history.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AiChatInfo {
    /// Unique chat identifier (UUID)
    pub id: String,
    /// Chat title (auto-generated or user-set)
    pub title: String,
    /// Model ID used for this chat (e.g., "claude-3-5-sonnet-20241022")
    pub model_id: String,
    /// AI provider (e.g., "anthropic", "openai")
    pub provider: String,
    /// When the chat was created (ISO 8601)
    pub created_at: String,
    /// When the chat was last updated (ISO 8601)
    pub updated_at: String,
    /// Whether this chat is soft-deleted
    pub is_deleted: bool,
    /// Preview of the last message (~60 chars)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
    /// Number of messages in the chat
    pub message_count: usize,
}

impl AiChatInfo {
    /// Create a new AiChatInfo with required fields
    pub fn new(id: String, title: String, model_id: String, provider: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        AiChatInfo {
            id,
            title,
            model_id,
            provider,
            created_at: now.clone(),
            updated_at: now,
            is_deleted: false,
            preview: None,
            message_count: 0,
        }
    }

    /// Set the preview text
    pub fn with_preview(mut self, preview: String) -> Self {
        self.preview = Some(preview);
        self
    }

    /// Set the message count
    pub fn with_message_count(mut self, count: usize) -> Self {
        self.message_count = count;
        self
    }

    /// Set timestamps
    pub fn with_timestamps(mut self, created_at: String, updated_at: String) -> Self {
        self.created_at = created_at;
        self.updated_at = updated_at;
        self
    }

    /// Mark as deleted
    pub fn deleted(mut self) -> Self {
        self.is_deleted = true;
        self
    }
}

/// Message information returned by AI SDK API responses
///
/// Used for `aiGetConversation()` and message-related operations.
/// Contains the full message content and metadata.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AiMessageInfo {
    /// Unique message identifier (UUID)
    pub id: String,
    /// Message role: "user", "assistant", or "system"
    pub role: String,
    /// Full message content
    pub content: String,
    /// When the message was created (ISO 8601)
    pub created_at: String,
    /// Number of tokens used (for assistant messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_used: Option<u32>,
}

impl AiMessageInfo {
    /// Create a new AiMessageInfo
    pub fn new(id: String, role: String, content: String) -> Self {
        AiMessageInfo {
            id,
            role,
            content,
            created_at: chrono::Utc::now().to_rfc3339(),
            tokens_used: None,
        }
    }

    /// Create a user message
    pub fn user(id: String, content: String) -> Self {
        Self::new(id, "user".to_string(), content)
    }

    /// Create an assistant message
    pub fn assistant(id: String, content: String) -> Self {
        Self::new(id, "assistant".to_string(), content)
    }

    /// Create a system message
    pub fn system(id: String, content: String) -> Self {
        Self::new(id, "system".to_string(), content)
    }

    /// Set the timestamp
    pub fn with_timestamp(mut self, timestamp: String) -> Self {
        self.created_at = timestamp;
        self
    }

    /// Set tokens used
    pub fn with_tokens(mut self, tokens: u32) -> Self {
        self.tokens_used = Some(tokens);
        self
    }
}
