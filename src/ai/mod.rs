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

// Re-exports intentionally cover the module's API surface.
#![allow(unused_imports)]
#![allow(dead_code)]

pub(crate) mod config;
pub(crate) mod model;
pub(crate) mod providers;
pub(crate) mod script_generation;
pub(crate) mod sdk_handlers;
pub(crate) mod session;
pub(crate) mod storage;
pub(crate) mod window;

// Re-export commonly used types
pub use self::config::{DetectedKeys, ModelInfo, ProviderConfig};
pub use self::model::{Chat, ChatId, ChatSource, Message, MessageRole};
pub use self::providers::{AiProvider, ProviderMessage, ProviderRegistry};
pub use self::script_generation::{generate_script_from_prompt, GeneratedScriptOutput};
pub use self::sdk_handlers::try_handle_ai_message;
pub use self::storage::{
    clear_all_chats, create_chat, delete_chat, get_all_chats, get_chat, get_chat_messages,
    get_deleted_chats, init_ai_db, insert_mock_data, save_message, search_chats, update_chat_title,
};
pub use self::window::{
    add_ai_attachment, close_ai_window, is_ai_window, is_ai_window_open, open_ai_window,
    open_ai_window_with_chat, set_ai_input, set_ai_input_with_image, set_ai_search,
    show_ai_command_bar, simulate_ai_key,
};
