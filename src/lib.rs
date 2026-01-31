#![allow(unexpected_cfgs)]

//! Script Kit GPUI - A GPUI-based launcher for Script Kit
//!
//! This library provides the core functionality for executing scripts
//! with bidirectional JSONL communication.

// Actions - Reusable action dialog component
// Provides ActionsDialog with configurable layout for script actions, AI command bar, etc.
pub mod actions;

// App Shell - Unified frame and chrome for all prompts
// Provides ShellSpec, HeaderSpec, FooterSpec, ChromeSpec for consistent prompt layout
pub mod app_shell;

pub mod components;
pub mod config;

// Unified icon system - single API for all icon sources
// Supports gpui_component IconName, embedded SVGs, SF Symbols, app bundles
pub mod debug_grid;
pub mod designs;
pub mod editor;
pub mod error;
pub mod executor;
pub mod focus_coordinator;
pub mod form_prompt;
pub mod hotkeys;
pub mod icons;
pub mod list_item;
pub mod logging;
pub mod navigation;
pub mod panel;
pub mod perf;
pub mod platform;
pub mod prompts;
pub mod protocol;
pub mod scripts;
#[cfg(target_os = "macos")]
pub mod selected_text;
pub mod shortcuts;
pub mod syntax;
pub mod term_prompt;
pub mod terminal;
pub mod theme;
pub mod toast_manager;

// Unified notification system - centralized notification handling
// Supports toast, HUD, banner, system notifications with deduplication and history
pub mod notification;
#[cfg(not(test))]
pub mod tray;
pub mod utils;
pub mod warning_banner;
pub mod window_manager;
pub mod window_ops;
pub mod window_resize;
pub mod window_state;
pub mod windows;

// Phase 1 system API modules
pub mod clipboard_history;
pub mod file_search;
#[cfg(target_os = "macos")]
pub mod window_control;
#[cfg(not(target_os = "macos"))]
#[path = "window_control_stub.rs"]
pub mod window_control;

// Enhanced window control - backends + capabilities architecture
// Provides WindowBounds (AX coords), capability detection, DisplayInfo, SpaceManager
#[cfg(target_os = "macos")]
pub mod window_control_enhanced;

// System actions - macOS AppleScript-based system commands
#[cfg(target_os = "macos")]
pub mod system_actions;

// Script creation - Create new scripts and scriptlets
pub mod script_creation;

// Permissions wizard - Check and request macOS permissions
#[cfg(target_os = "macos")]
pub mod permissions_wizard;

// Menu bar reader - macOS Accessibility API for reading app menus
// Provides get_frontmost_menu_bar() with recursive parsing up to 3 levels
#[cfg(target_os = "macos")]
pub mod menu_bar;

// Menu executor - Execute menu actions via Accessibility API
// Navigates AX hierarchy and performs AXPress on menu items
#[cfg(target_os = "macos")]
pub mod menu_executor;

// Menu cache - SQLite-backed menu bar data caching
// Caches application menu hierarchies by bundle_id to avoid expensive rescans
#[cfg(target_os = "macos")]
pub mod menu_cache;

// Frontmost app tracker - Background observer for tracking active application
// Pre-fetches menu bar items when apps activate (before Script Kit opens)
#[cfg(target_os = "macos")]
pub mod frontmost_app_tracker;

// Action helpers - centralized path extraction, SDK action routing, pbcopy
pub mod action_helpers;

// Built-in features registry
pub mod app_launcher;
pub mod builtins;

// Fallback commands - Raycast-style fallback actions when no scripts match
pub mod fallbacks;

// Frecency tracking for script usage
pub mod frecency;

// Input history for up/down arrow navigation through previous inputs
pub mod input_history;

// Process management for tracking bun script processes
pub mod process_manager;

// Scriptlet parsing and variable substitution
pub mod scriptlets;

// Scriptlet cache for tracking per-file state with change detection
// Used by file watchers to diff scriptlet changes and update registrations incrementally
pub mod scriptlet_cache;

// Typed metadata parser for new `metadata = {}` global syntax
pub mod metadata_parser;

// Schema parser for `schema = { input: {}, output: {} }` definitions
pub mod schema_parser;

// Scriptlet codefence metadata parser for ```metadata and ```schema blocks
pub mod scriptlet_metadata;

// VSCode snippet syntax parser for template() SDK function
pub mod snippet;

// HTML form parsing for form() prompt
pub mod form_parser;

// Centralized template variable substitution system
// Used by expand_manager, template prompts, and future template features
pub mod template_variables;

// Text injection for text expansion/snippet systems
#[cfg(target_os = "macos")]
pub mod text_injector;

// Keyword trigger matching for text expansion
pub mod keyword_matcher;

// Debounced keystroke logging for text expansion system
// Consolidates per-keystroke logs into periodic summaries
pub mod keystroke_logger;

// Global keyboard monitoring for system-wide keystroke capture
// Required for text expansion triggers typed in any application
#[cfg(target_os = "macos")]
pub mod keyboard_monitor;

// Keyword manager - ties together keyboard monitoring, trigger matching,
// and text injection for the complete text expansion system
#[cfg(target_os = "macos")]
pub mod keyword_manager;

// OCR module - macOS Vision framework integration
#[cfg(feature = "ocr")]
pub mod ocr;

// Script scheduling with cron expressions and natural language
pub mod scheduler;

// Kenv environment setup and initialization
// Ensures ~/.scriptkit exists with required directories and starter files
pub mod setup;

// Storybook - Component preview system for development
pub mod storybook;

// Stories - Component story definitions for the storybook
pub mod stories;

// MCP Server - HTTP server for Model Context Protocol integration
// Provides localhost:43210 endpoint with Bearer token auth
pub mod mcp_server;

// MCP Streaming - Server-Sent Events (SSE) and audit logging
// Provides real-time event streaming and tool call audit logs
pub mod mcp_streaming;

// MCP Protocol - JSON-RPC 2.0 protocol handler for MCP
// Handles request parsing, method routing, and response generation
pub mod mcp_protocol;

// MCP Kit Tools - kit/* namespace tools for app control
// Provides kit/show, kit/hide, kit/state tools
pub mod mcp_kit_tools;

// MCP Script Tools - scripts/* namespace auto-generated tools
// Scripts with schema.input become MCP tools automatically
pub mod mcp_script_tools;

// MCP Resources - read-only data resources for MCP clients
// Provides kit://state, scripts://, and scriptlets:// resources
pub mod mcp_resources;

// Stdin commands - external command handling via stdin
// Provides JSON command protocol for testing and automation
pub mod stdin_commands;

// Notes - Raycast Notes feature parity
// Separate floating window for note-taking with gpui-component
pub mod notes;

// AI Chat - Separate floating window for AI conversations
// BYOK (Bring Your Own Key) with SQLite storage at ~/.scriptkit/ai-chats.db
pub mod ai;

// Agents - mdflow agent integration
// Executable markdown prompts that run against Claude, Gemini, Codex, or Copilot
// Located in ~/.scriptkit/*/agents/*.md
pub mod agents;

// Secrets - age-encrypted secrets storage
// Portable alternative to keyring, stores secrets at ~/.scriptkit/secrets.age
// Uses scrypt passphrase-based encryption with machine-derived passphrase
pub mod secrets;

// macOS launch-at-login via SMAppService
// Uses SMAppService on macOS 13+ for modern login item management
#[cfg(target_os = "macos")]
pub mod login_item;

// UI transitions/animations (self-contained module, no external crate dependency)
// Provides TransitionColor, Opacity, SlideOffset, AppearTransition, HoverState
// and easing functions (ease_out_quad, ease_in_quad, etc.)
// Used for smooth hover effects, toast animations, and other UI transitions
pub mod transitions;

// UI Foundation - Shared UI patterns for consistent vibrancy and layout
// Extracts common patterns from main menu (render_script_list.rs) into reusable helpers
// Used by term.rs, editor.rs, div.rs, form.rs and other prompts
pub mod ui_foundation;

// File watchers for theme, config, scripts, and system appearance
pub mod watcher;

// Window state management tests - code audit to prevent regressions
// Verifies that app_execute.rs uses close_and_reset_window() correctly
#[cfg(test)]
mod window_state_tests;

// Shared window visibility state
// Used to track main window visibility across the app
// Notes/AI windows use this to decide whether to hide the app after closing
use std::sync::atomic::{AtomicBool, Ordering};

/// Global state tracking whether the main window is visible
/// - Used by hotkey toggle to show/hide main window
/// - Used by Notes/AI to prevent main window from appearing when they close
pub static MAIN_WINDOW_VISIBLE: AtomicBool = AtomicBool::new(false);

/// Tracks whether a script requested hiding the window (via Hide message)
/// When ScriptExit is received, if this is true, we show the window again
/// This ensures main menu comes back after scripts that temporarily hide (e.g., getSelectedText)
pub static SCRIPT_REQUESTED_HIDE: AtomicBool = AtomicBool::new(false);

/// Check if the main window is currently visible
pub fn is_main_window_visible() -> bool {
    MAIN_WINDOW_VISIBLE.load(Ordering::SeqCst)
}

/// Set the main window visibility state
pub fn set_main_window_visible(visible: bool) {
    MAIN_WINDOW_VISIBLE.store(visible, Ordering::SeqCst);
}

/// Check if a script requested hiding the window
pub fn script_requested_hide() -> bool {
    SCRIPT_REQUESTED_HIDE.load(Ordering::SeqCst)
}

/// Set the script-requested-hide flag
pub fn set_script_requested_hide(value: bool) {
    SCRIPT_REQUESTED_HIDE.store(value, Ordering::SeqCst);
}

/// Channel for requesting the main window to be shown
/// Used by prompt_handler to signal that the window should come back after script exit
static SHOW_WINDOW_CHANNEL: std::sync::OnceLock<(
    async_channel::Sender<()>,
    async_channel::Receiver<()>,
)> = std::sync::OnceLock::new();

/// Get the show window channel (sender, receiver)
pub fn show_window_channel() -> &'static (async_channel::Sender<()>, async_channel::Receiver<()>) {
    SHOW_WINDOW_CHANNEL.get_or_init(|| async_channel::bounded(10))
}

/// Request showing the main window (called from prompt_handler on ScriptExit)
pub fn request_show_main_window() {
    let (tx, _) = show_window_channel();
    let _ = tx.try_send(());
}

/// Timestamp of when the window was last shown (for focus loss grace period)
/// This prevents focus racing from immediately closing the window after it opens
static WINDOW_SHOWN_AT: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);

/// Grace period in milliseconds after showing window during which focus loss is ignored
const FOCUS_LOSS_GRACE_PERIOD_MS: u64 = 200;

/// Mark the window as just shown (call from show_main_window_helper)
pub fn mark_window_shown() {
    if let Ok(mut guard) = WINDOW_SHOWN_AT.lock() {
        *guard = Some(std::time::Instant::now());
    }
}

/// Check if we're within the grace period after showing the window
/// Returns true if focus loss should be ignored (within grace period)
pub fn is_within_focus_grace_period() -> bool {
    if let Ok(guard) = WINDOW_SHOWN_AT.lock() {
        if let Some(shown_at) = *guard {
            let elapsed = shown_at.elapsed().as_millis() as u64;
            return elapsed < FOCUS_LOSS_GRACE_PERIOD_MS;
        }
    }
    false
}
