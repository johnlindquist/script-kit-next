//! GPUI Prompt UI Components
//!
//! This module provides modular prompt components for Script Kit.
//! Each prompt type is implemented in its own submodule for parallel development.
//!
//! # Module Structure
//! - `base`: PromptBase - Shared base infrastructure (fields, DesignContext, macros)
//! - `chat`: ChatPrompt - Raycast-style chat interface with streaming support
//! - `div`: DivPrompt - HTML content display
//! - `path`: PathPrompt - File/folder picker (skeleton)
//! - `env`: EnvPrompt - Environment variable/secrets (skeleton)
//! - `drop`: DropPrompt - Drag and drop (skeleton)
//! - `template`: TemplatePrompt - String templates with placeholders (skeleton)
//! - `select`: SelectPrompt - Multi-select with checkboxes (skeleton)

#![allow(dead_code)]

pub mod base;
pub mod chat;
pub mod div;
mod drop;
pub mod env;
pub mod markdown;
mod path;
mod select;
mod template;

// Re-export prompt types for use when they're integrated into main.rs
// When integrating:
// 1. Create Entity<PromptType> in main.rs
// 2. Switch from inline rendering to entity-based rendering
// Note: ArgPrompt is implemented inline in render_prompts/arg.rs, not as a standalone component

// Base infrastructure for prompts - will be used as prompts adopt PromptBase
#[allow(unused_imports)]
pub use base::{DesignContext, PromptBase, ResolvedColors};
pub use chat::{
    ChatClaudeCodeCallback, ChatConfigureCallback, ChatEscapeCallback, ChatPrompt,
    ChatSubmitCallback,
};
pub use div::{ContainerOptions, ContainerPadding, DivPrompt};

// These exports are ready for use in main.rs when AppView variants are added
// The #[allow(unused_imports)] is temporary until main.rs integrations are complete
#[allow(unused_imports)]
pub use drop::DropPrompt;
#[allow(unused_imports)]
pub use env::EnvPrompt;
#[allow(unused_imports)]
pub use path::PathInfo;
#[allow(unused_imports)]
pub use path::PathPrompt;
#[allow(unused_imports)]
pub use path::PathPromptEvent;
#[allow(unused_imports)]
pub use path::ShowActionsCallback;
#[allow(unused_imports)]
pub use select::SelectPrompt;
#[allow(unused_imports)]
pub use template::TemplatePrompt;

// Re-export common types used by prompts
use std::sync::Arc;

/// Callback for prompt submission
/// Signature: (id: String, value: Option<String>)
pub type SubmitCallback = Arc<dyn Fn(String, Option<String>) + Send + Sync>;
