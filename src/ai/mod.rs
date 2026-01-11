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
    close_ai_window, is_ai_window_open, open_ai_window, open_ai_window_with_chat, set_ai_input,
    set_ai_input_with_image, set_ai_search,
};

// Re-export SDK handler
pub use sdk_handlers::try_handle_ai_message;
