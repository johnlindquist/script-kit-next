//! ChatPrompt - Raycast-style chat interface
//!
//! Features:
//! - Input at TOP (not bottom)
//! - Messages bundled as conversation turns (user prompt + AI response in same container)
//! - Full-width containers (not bubbles)
//! - Footer with model selector and "Continue in Chat"
//! - Actions menu (⌘+K) with model picker

use crate::components::prompt_footer::PromptFooterColors;
use crate::components::TextInputState;
use crate::designs::icon_variations::IconName;
use gpui::{
    div, img, list, prelude::*, px, rgb, rgba, svg, AnyElement, App, Context, ExternalPaths,
    FocusHandle, Focusable, Hsla, KeyDownEvent, ListAlignment, ListSizingBehavior, ListState,
    Render, RenderImage, ScrollWheelEvent, Timer, Window,
};
use gpui_component::{scroll::ScrollableElement, theme::ActiveTheme};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::ai::providers::{ProviderMessage, ProviderRegistry};
use crate::ai::{self, Chat, ChatSource, Message, MessageRole, ModelInfo};
use crate::logging;
use crate::prompts::commands::transform_with_command;
use crate::prompts::context::expand_context;
use crate::prompts::markdown::render_markdown;
use crate::protocol::{ChatMessagePosition, ChatMessageRole, ChatPromptMessage};
use crate::theme;
use crate::ui_foundation::get_vibrancy_background;

mod actions;
mod prompt;
mod render_core;
mod render_input;
mod render_setup;
mod render_turns;
mod state;
mod streaming;
#[cfg(test)]
mod tests;
mod types;

#[cfg(test)]
pub(crate) use tests::chat_tests;

use self::types::{
    build_conversation_turns, default_conversation_starters, next_chat_scroll_follow_state,
    next_reveal_boundary, resolve_chat_input_key_action, resolve_setup_card_key,
    should_ignore_stream_reveal_update, should_show_script_generation_actions, ChatInputKeyAction,
    ChatScrollDirection, RunScriptCallback, ScriptGenerationAction, SetupCardAction,
};

pub use prompt::ChatPrompt;
pub use types::{
    default_models, ChatClaudeCodeCallback, ChatConfigureCallback, ChatContinueCallback,
    ChatErrorType, ChatEscapeCallback, ChatModel, ChatRetryCallback, ChatShowActionsCallback,
    ChatSubmitCallback, ConversationStarter, ConversationTurn,
};
