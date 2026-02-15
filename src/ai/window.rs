//! AI Chat Window
//!
//! A separate floating window for AI chat, built with gpui-component.
//! This is completely independent from the main Script Kit launcher window.

use anyhow::Result;
use chrono::{Datelike, NaiveDate, Utc};
use gpui::{
    div, hsla, img, list, point, prelude::*, px, rgba, size, svg, App, BoxShadow, Context,
    CursorStyle, Entity, ExternalPaths, FocusHandle, Focusable, IntoElement, KeyDownEvent,
    ListAlignment, ListSizingBehavior, ListState, MouseMoveEvent, ParentElement, Render,
    RenderImage, ScrollWheelEvent, SharedString, Styled, Subscription, Window, WindowBounds,
    WindowOptions,
};

use crate::designs::icon_variations::IconName as LocalIconName;

#[cfg(target_os = "macos")]
use cocoa::appkit::NSApp;
#[cfg(target_os = "macos")]
use cocoa::base::{id, nil};
use gpui_component::{
    button::{Button, ButtonCustomVariant, ButtonVariants},
    input::{Input, InputEvent, InputState},
    kbd::Kbd,
    scroll::ScrollableElement,
    theme::ActiveTheme,
    tooltip::Tooltip,
    Icon, IconName, Root, Sizable,
};
#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};
use tracing::{debug, info};

use super::config::ModelInfo;
use super::model::{Chat, ChatId, ChatSource, Message, MessageRole};
use super::providers::ProviderRegistry;
use super::storage;
use crate::actions::{get_ai_command_bar_actions, CommandBar, CommandBarConfig};
use crate::prompts::markdown::render_markdown;
use crate::stdin_commands::KeyModifier;
use crate::theme;

mod types;
use types::*;
mod state;
use state::*;

mod chat;
mod command_bar;
mod dropdowns;
mod images;
mod init;
mod interactions;
mod platform;
mod render_input;
mod render_keydown;
mod render_main_panel;
mod render_message;
mod render_message_actions;
mod render_messages;
mod render_overlays_attachments;
mod render_overlays_dropdowns;
mod render_overlays_shortcuts;
mod render_root;
mod render_setup;
mod render_sidebar;
mod render_sidebar_items;
mod render_streaming;
mod render_welcome;
mod search;
mod setup;
mod streaming_control;
mod streaming_submit;
mod theme_helpers;
mod traits;
mod window_api;
use platform::*;

#[cfg(test)]
mod tests;

pub use window_api::{
    add_ai_attachment, close_ai_window, is_ai_window, is_ai_window_open, open_ai_window,
    open_ai_window_with_chat, set_ai_input, set_ai_input_with_image, set_ai_search,
    show_ai_command_bar, simulate_ai_key,
};
