#![allow(unexpected_cfgs)]

//! Binary entrypoint and GPUI application composition for Script Kit GPUI.
//! It defines the `ScriptListApp` runtime, wires stdin protocol handling in `main`,
//! and includes prompt/builtin render implementations for the primary window.
//! This module depends on app subsystems like `protocol`, `hotkeys`, `watcher`, and `window_manager`.

use gpui::{
    div, hsla, list, point, prelude::*, px, rgb, rgba, size, svg, uniform_list, AnyElement, App,
    Application, BoxShadow, Context, CursorStyle, ElementId, Entity, FocusHandle, Focusable,
    FontWeight, KeyDownEvent, ListAlignment, ListOffset, ListSizingBehavior, ListState,
    MouseMoveEvent, Render, ScrollStrategy, SharedString, Subscription, Timer,
    UniformListScrollHandle, Window, WindowBackgroundAppearance, WindowBounds, WindowHandle,
    WindowKind, WindowOptions,
};

// gpui-component Root wrapper for theme and context provision
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::notification::{Notification, NotificationType};
use gpui_component::Root;
use gpui_component::{Sizable, Size};
use std::sync::atomic::{AtomicBool, Ordering};

mod process_manager;
use cocoa::base::id;
use cocoa::foundation::NSRect;
use process_manager::PROCESS_MANAGER;

// Platform utilities - mouse position, display info, window movement, screenshots
use platform::{
    calculate_eye_line_bounds_on_mouse_display, capture_app_screenshot, capture_window_by_title,
};
#[macro_use]
extern crate objc;

mod actions;
#[cfg(test)]
mod actions_button_visibility_tests;
mod agents;
mod ai;
mod aliases;
pub mod calculator;
mod camera;
#[cfg(test)]
mod clipboard_actions_tests;
mod components;
mod config;
mod confirm;
mod designs;
mod editor;
#[allow(dead_code)] // Public API in lib.rs; binary only uses ErrorSeverity currently
mod error;
mod executor;
mod filter_coalescer;
mod focus_coordinator;
mod form_prompt;
#[allow(dead_code)] // TODO: Re-enable once hotkey_pollers is updated for Root wrapper
mod hotkey_pollers;
mod hotkeys;
#[cfg(test)]
mod keyboard_routing_tests;
mod list_item;
#[cfg(test)]
mod list_item_tests;
#[cfg(test)]
mod list_state_init_tests;
mod logging;
mod login_item;
mod navigation;
mod panel;
mod perf;
mod platform;
mod prompts;
mod protocol;
mod scripts;
#[cfg(target_os = "macos")]
mod selected_text;
mod setup;
mod shortcuts;
mod stdin_commands;
mod syntax;
mod term_prompt;
mod terminal;
mod theme;
mod transitions;
mod tray;
mod ui_foundation;
mod utils;
mod warning_banner;
mod watcher;
mod window_manager;
mod window_ops;
mod window_resize;
mod window_state;
#[cfg(test)]
mod window_state_persistence_tests;
mod windows;

// Phase 1 system API modules
mod clipboard_history;
mod file_search;
mod toast_manager;
mod window_control;

// Secrets - age-encrypted secrets storage (replacement for keyring)
mod secrets;

// System actions - macOS AppleScript-based system commands
#[cfg(target_os = "macos")]
mod system_actions;

// Script creation - Create new scripts and scriptlets
mod script_creation;

// Permissions wizard - Check and request macOS permissions
mod permissions_wizard;

// Built-in features registry
mod app_launcher;
mod builtins;
mod fallbacks;
mod favorites;
mod menu_bar;

// Frontmost app tracker - Background observer for tracking active application
#[cfg(target_os = "macos")]
mod frontmost_app_tracker;

// Frecency tracking for script usage
mod frecency;

// Scriptlet parsing and variable substitution
mod scriptlets;

// Typed metadata parser for new `metadata = {}` global syntax
mod metadata_parser;

// Schema parser for `schema = { input: {}, output: {} }` definitions
mod schema_parser;

// Scriptlet codefence metadata parser for ```metadata and ```schema blocks
mod scriptlet_metadata;

// Input history for shell-like up/down navigation through previous inputs
mod input_history;

// VSCode snippet syntax parser for template() SDK function
mod snippet;

// HTML form parsing for form() prompt
mod form_parser;

// Centralized template variable substitution system
mod template_variables;

// Text expansion system components (macOS only)
#[cfg(target_os = "macos")]
mod keyboard_monitor;
mod keystroke_logger;
mod keyword_matcher;
mod text_injector;

// Keyword manager - text expansion system integration
#[cfg(target_os = "macos")]
mod keyword_manager;

// Script scheduling with cron expressions and natural language
mod scheduler;

// HUD manager - system-level overlay notifications (separate floating windows)
mod hud_manager;

// Debug grid overlay for visual testing
mod debug_grid;

// MCP Server modules for AI agent integration
mod mcp_kit_tools;
mod mcp_protocol;
mod mcp_resources;
mod mcp_script_tools;
mod mcp_server;
mod mcp_streaming;

// Notes - Raycast Notes feature parity (separate floating window)
mod notes;

use crate::components::text_input::TextInputState;
use crate::components::toast::{Toast, ToastAction};
use crate::error::ErrorSeverity;
use crate::filter_coalescer::FilterCoalescer;
use crate::form_prompt::FormPromptState;
// TODO: Re-enable when hotkey_pollers.rs is updated for Root wrapper
// use crate::hotkey_pollers::start_hotkey_event_handler;
use crate::navigation::{NavCoalescer, NavDirection};
use crate::toast_manager::{PendingToast, ToastManager};
use components::ToastVariant;
use editor::EditorPrompt;
use prompts::{
    ContainerOptions, ContainerPadding, DivPrompt, DropPrompt, EnvPrompt, PathInfo, PathPrompt,
    PathPromptEvent, SelectPrompt, TemplatePrompt,
};
use tray::{TrayManager, TrayMenuAction};
use ui_foundation::get_vibrancy_background;
use warning_banner::{WarningBanner, WarningBannerColors};
use window_resize::{
    defer_resize_to_view, height_for_view, initial_window_height, reset_resize_debounce,
    resize_first_window_to_height, resize_to_view_sync, ViewType,
};

use components::{
    FormFieldColors, PromptFooter, PromptFooterColors, PromptFooterConfig, Scrollbar,
    ScrollbarColors,
};
use designs::{get_tokens, render_design_item, DesignVariant};
use frecency::FrecencyStore;
use list_item::{
    render_section_header, GroupedListItem, ListItem, ListItemColors, ALPHA_DIVIDER,
    ALPHA_EMPTY_HINT, ALPHA_EMPTY_ICON, ALPHA_EMPTY_MESSAGE, ALPHA_EMPTY_TIPS, ALPHA_HOVER_ACCENT,
    ALPHA_TAB_BADGE_BG, ASK_AI_BUTTON_GAP, ASK_AI_BUTTON_PADDING_X, ASK_AI_BUTTON_PADDING_Y,
    ASK_AI_BUTTON_RADIUS, AVERAGE_ITEM_HEIGHT_FOR_SCROLL, DIVIDER_BORDER_WIDTH_DEFAULT,
    DIVIDER_MARGIN_DEFAULT, EMPTY_STATE_GAP, EMPTY_STATE_ICON_SIZE, EMPTY_STATE_MESSAGE_FONT_SIZE,
    EMPTY_STATE_TIPS_MARGIN_TOP, ESTIMATED_LIST_CONTAINER_HEIGHT, FONT_MONO, LIST_ITEM_HEIGHT,
    LOG_PANEL_MAX_HEIGHT, SECTION_HEADER_HEIGHT, TAB_BADGE_PADDING_X, TAB_BADGE_PADDING_Y,
    TAB_BADGE_RADIUS,
};
use scripts::get_grouped_results;
// strip_html_tags removed - DivPrompt now renders HTML properly

use actions::{
    close_actions_window, is_actions_window_open, notify_actions_window, open_actions_window,
    resize_actions_window, ActionsDialog, ScriptInfo,
};
use confirm::{open_confirm_window, ConfirmCallback};
use panel::{
    CURSOR_GAP_X, CURSOR_HEIGHT_LG, CURSOR_MARGIN_Y, CURSOR_WIDTH, DEFAULT_PLACEHOLDER, HEADER_GAP,
    HEADER_PADDING_X, HEADER_PADDING_Y,
};
use parking_lot::Mutex as ParkingMutex;
use protocol::{Choice, Message, ProtocolAction};
use std::sync::{mpsc, Arc, Mutex};
use syntax::highlight_code_lines;

/// Channel for sending prompt messages from script thread to UI
#[allow(dead_code)]
type PromptChannel = (mpsc::Sender<PromptMessage>, mpsc::Receiver<PromptMessage>);

// Import utilities from modules
use stdin_commands::{
    start_stdin_listener, validate_capture_window_output_path, ExternalCommand,
    ExternalCommandEnvelope, KeyModifier,
};
use utils::render_path_with_highlights;

// Global state for hotkey signaling between threads
static NEEDS_RESET: AtomicBool = AtomicBool::new(false); // Track if window needs reset to script list on next show

pub use script_kit_gpui::{emoji, is_main_window_visible, set_main_window_visible};
static PANEL_CONFIGURED: AtomicBool = AtomicBool::new(false); // Track if floating panel has been configured (one-time setup on first show)
static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false); // Track if shutdown signal received (prevents new script spawns)

include!("main_sections/deeplink.rs");
include!("main_sections/window_visibility.rs");
include!("main_sections/fallbacks.rs");
include!("main_sections/fonts.rs");
include!("main_sections/app_view_state.rs");
include!("main_sections/prompt_messages.rs");
include!("main_sections/app_state.rs");
// Core ScriptListApp implementation extracted to app_impl/mod.rs
include!("app_impl/mod.rs");

// Script execution logic (execute_interactive) extracted
include!("execute_script/mod.rs");

// Prompt message handling (handle_prompt_message) extracted
include!("prompt_handler/mod.rs");

// App navigation methods (selection movement, scrolling)
include!("app_navigation.rs");

// App execution methods (execute_builtin, execute_app, execute_window_focus)
include!("app_execute.rs");

// App actions handling (handle_action, trigger_action_by_name)
include!("app_actions.rs");

// Layout calculation methods (build_component_bounds, build_layout_info)
include!("app_layout.rs");

include!("main_sections/render_impl.rs");
// Render methods extracted to app_render.rs for maintainability
include!("app_render.rs");

// Builtin view render methods (clipboard, app launcher, window switcher)
include!("render_builtins.rs");

// Prompt render methods - split into separate files for maintainability
// Each file adds render_*_prompt methods to ScriptListApp via impl blocks
include!("render_prompts/key_handler.rs");
include!("render_prompts/arg.rs");
include!("render_prompts/div.rs");
include!("render_prompts/form.rs");
include!("render_prompts/term.rs");
include!("render_prompts/editor.rs");
include!("render_prompts/path.rs");
include!("render_prompts/other.rs");

// Script list render method
include!("render_script_list/mod.rs");

fn main() {
    include!("main_entry/app_run_setup.rs");
}
#[cfg(test)]
mod tests {
    use super::{is_main_window_visible, set_main_window_visible};

    #[test]
    fn main_window_visibility_is_shared_with_library() {
        set_main_window_visible(false);
        script_kit_gpui::set_main_window_visible(false);

        set_main_window_visible(true);
        assert!(
            script_kit_gpui::is_main_window_visible(),
            "library visibility should mirror main visibility"
        );

        script_kit_gpui::set_main_window_visible(false);
        assert!(
            !is_main_window_visible(),
            "main visibility should mirror library visibility"
        );
    }
}
