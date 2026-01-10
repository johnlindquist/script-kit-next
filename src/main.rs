#![allow(unexpected_cfgs)]

use gpui::{
    div, hsla, list, point, prelude::*, px, rgb, rgba, size, svg, uniform_list, AnyElement, App,
    Application, BoxShadow, Context, ElementId, Entity, FocusHandle, Focusable, ListAlignment,
    ListSizingBehavior, ListState, Render, ScrollStrategy, SharedString, Subscription, Timer,
    UniformListScrollHandle, Window, WindowBackgroundAppearance, WindowBounds, WindowHandle,
    WindowOptions,
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
mod agents;
mod confirm;
mod ai;
mod aliases;
mod components;
mod config;
mod designs;
mod editor;
mod error;
mod executor;
mod filter_coalescer;
mod form_prompt;
#[allow(dead_code)] // TODO: Re-enable once hotkey_pollers is updated for Root wrapper
mod hotkey_pollers;
mod hotkeys;
#[cfg(test)]
mod keyboard_routing_tests;
mod list_item;
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
    render_section_header, GroupedListItem, ListItem, ListItemColors, LIST_ITEM_HEIGHT,
    SECTION_HEADER_HEIGHT,
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
use stdin_commands::{start_stdin_listener, ExternalCommand};
use utils::render_path_with_highlights;

// Global state for hotkey signaling between threads
static NEEDS_RESET: AtomicBool = AtomicBool::new(false); // Track if window needs reset to script list on next show

pub use script_kit_gpui::{is_main_window_visible, set_main_window_visible};
static PANEL_CONFIGURED: AtomicBool = AtomicBool::new(false); // Track if floating panel has been configured (one-time setup on first show)
static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false); // Track if shutdown signal received (prevents new script spawns)

// DEEPLINK_CHANNEL: Channel for handling scriptkit:// URL scheme events
// URLs are sent from on_open_urls callback and processed inside the app
static DEEPLINK_CHANNEL: std::sync::OnceLock<(
    async_channel::Sender<String>,
    async_channel::Receiver<String>,
)> = std::sync::OnceLock::new();

/// Get the deeplink channel, initializing it on first access.
fn deeplink_channel() -> &'static (
    async_channel::Sender<String>,
    async_channel::Receiver<String>,
) {
    DEEPLINK_CHANNEL.get_or_init(|| async_channel::bounded(10))
}

/// Parse a scriptkit:// URL and extract the command ID
/// Supported formats:
/// - scriptkit://commands/{command_id} - Execute any command (app/builtin/script/scriptlet)
/// - scriptkit://run/{script_name} - Execute a script by name (legacy)
/// - scriptkit://notes/{note_id} - Open a specific note
fn parse_deeplink_url(url: &str) -> Option<String> {
    // Remove the scheme
    let path = url.strip_prefix("scriptkit://")?;

    if let Some(command_id) = path.strip_prefix("commands/") {
        // Format: scriptkit://commands/app/com.apple.Safari
        // or: scriptkit://commands/builtin/clipboard-history
        Some(command_id.to_string())
    } else if let Some(script_name) = path.strip_prefix("run/") {
        // Legacy format: scriptkit://run/my-script -> script/{name}
        Some(format!("script/{}", script_name))
    } else if let Some(note_id) = path.strip_prefix("notes/") {
        // Notes deeplink - handled specially
        Some(format!("notes/{}", note_id))
    } else {
        logging::log("DEEPLINK", &format!("Unknown deeplink format: {}", url));
        None
    }
}

/// Convert our ToastVariant to gpui-component's NotificationType
fn toast_variant_to_notification_type(variant: ToastVariant) -> NotificationType {
    match variant {
        ToastVariant::Success => NotificationType::Success,
        ToastVariant::Warning => NotificationType::Warning,
        ToastVariant::Error => NotificationType::Error,
        ToastVariant::Info => NotificationType::Info,
    }
}

/// Convert a PendingToast to a gpui-component Notification
fn pending_toast_to_notification(toast: &PendingToast) -> Notification {
    let notification_type = toast_variant_to_notification_type(toast.variant);

    let mut notification = Notification::new()
        .message(&toast.message)
        .with_type(notification_type);

    // Add title for errors/warnings (makes them stand out more)
    match toast.variant {
        ToastVariant::Error => {
            notification = notification.title("Error");
        }
        ToastVariant::Warning => {
            notification = notification.title("Warning");
        }
        _ => {}
    }

    // Note: gpui-component Notification has fixed 5s autohide
    // For persistent toasts, set autohide(false)
    if toast.duration_ms.is_none() {
        notification = notification.autohide(false);
    }

    notification
}

/// Check if shutdown has been requested (prevents new script spawns during shutdown)
#[allow(dead_code)]
pub fn is_shutting_down() -> bool {
    SHUTDOWN_REQUESTED.load(Ordering::SeqCst)
}

// ============================================================================
// WINDOW SHOW/HIDE HELPERS
// ============================================================================
// These helpers consolidate duplicated window show/hide logic that was
// scattered across hotkey handler, tray menu, stdin commands, and fallback.
// All show/hide paths should use these helpers for consistency.

/// Show the main window with proper positioning, panel configuration, and focus.
///
/// This is the canonical way to show the main window. It:
/// 1. Sets MAIN_WINDOW_VISIBLE state
/// 2. Moves window to active space
/// 3. Positions at eye-line on the display containing the mouse
/// 4. Configures as floating panel (first time only)
/// 5. Activates the window and focuses the input
/// 6. Resets resize debounce and handles NEEDS_RESET if set
///
/// # Arguments
/// * `window` - The main window handle (WindowHandle<Root>)
/// * `app_entity` - The ScriptListApp entity
/// * `cx` - The application context
fn show_main_window_helper(
    window: WindowHandle<Root>,
    app_entity: Entity<ScriptListApp>,
    cx: &mut App,
) {
    logging::log("VISIBILITY", "show_main_window_helper called");

    // 1. Set visibility state
    set_main_window_visible(true);

    // 2. Move to active space (macOS)
    platform::ensure_move_to_active_space();

    // 3. Position at eye-line on mouse display
    let window_size = gpui::size(px(750.), initial_window_height());
    let bounds = platform::calculate_eye_line_bounds_on_mouse_display(window_size);
    platform::move_first_window_to_bounds(&bounds);

    // 4. Configure as floating panel (first time only)
    if !PANEL_CONFIGURED.load(Ordering::SeqCst) {
        platform::configure_as_floating_panel();
        // HACK: Swizzle GPUI's BlurredView to preserve native CAChameleonLayer tint
        // GPUI hides this layer which removes the native macOS vibrancy tinting.
        // By swizzling, we get proper native blur appearance like Raycast/Spotlight.
        platform::swizzle_gpui_blurred_view();
        // Configure vibrancy material to HUD_WINDOW for proper dark appearance
        // This prevents background colors from bleeding through the blur
        platform::configure_window_vibrancy_material();
        PANEL_CONFIGURED.store(true, Ordering::SeqCst);
    }

    // 5. Activate window
    cx.activate(true);
    let _ = window.update(cx, |_root, win, _cx| {
        win.activate_window();
    });

    // 6. Focus input, reset resize debounce, and handle NEEDS_RESET
    app_entity.update(cx, |view, ctx| {
        let focus_handle = view.focus_handle(ctx);
        let _ = window.update(ctx, |_root, win, _cx| {
            win.focus(&focus_handle, _cx);
        });

        // Reset resize debounce to ensure proper window sizing
        reset_resize_debounce();

        // Handle NEEDS_RESET: if set (e.g., script completed while hidden),
        // reset to script list.
        if NEEDS_RESET
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            logging::log(
                "VISIBILITY",
                "NEEDS_RESET was true - resetting to script list",
            );
            view.reset_to_script_list(ctx);
        }

        // Always ensure window size matches current view using deferred resize.
        // This uses Window::defer to avoid RefCell borrow conflicts.
        let _ = window.update(ctx, |_root, win, win_cx| {
            defer_resize_to_view(ViewType::ScriptList, 0, win, win_cx);
        });
    });

    logging::log("VISIBILITY", "Main window shown and focused");
}

/// Hide the main window with proper state management.
///
/// This is the canonical way to hide the main window. It:
/// 1. Sets MAIN_WINDOW_VISIBLE state to false
/// 2. Cancels any active prompt (if in prompt mode)
/// 3. Resets to script list
/// 4. Uses hide_main_window() if Notes/AI windows are open (to avoid hiding them)
/// 5. Uses cx.hide() if no secondary windows are open
///
/// # Arguments
/// * `app_entity` - The ScriptListApp entity
/// * `cx` - The application context
fn hide_main_window_helper(app_entity: Entity<ScriptListApp>, cx: &mut App) {
    logging::log("VISIBILITY", "hide_main_window_helper called");

    // 1. Set visibility state
    set_main_window_visible(false);

    // 2. Check secondary windows BEFORE the update closure
    let notes_open = notes::is_notes_window_open();
    let ai_open = ai::is_ai_window_open();
    logging::log(
        "VISIBILITY",
        &format!(
            "Secondary windows: notes_open={}, ai_open={}",
            notes_open, ai_open
        ),
    );

    // 3. Cancel prompt and reset UI
    app_entity.update(cx, |view, ctx| {
        if view.is_in_prompt() {
            logging::log("VISIBILITY", "Canceling prompt before hiding");
            view.cancel_script_execution(ctx);
        }
        view.reset_to_script_list(ctx);
    });

    // 4. Hide appropriately based on secondary windows
    if notes_open || ai_open {
        logging::log(
            "VISIBILITY",
            "Using hide_main_window() - secondary windows are open",
        );
        platform::hide_main_window();
    } else {
        logging::log("VISIBILITY", "Using cx.hide() - no secondary windows");
        cx.hide();
    }

    logging::log("VISIBILITY", "Main window hidden");
}

/// Execute a fallback action based on the fallback ID and input text.
///
/// This handles the various fallback action types:
/// - run-in-terminal: Open terminal with command
/// - add-to-notes: Open Notes window with quick capture
/// - copy-to-clipboard: Copy text to clipboard
/// - search-google/search-duckduckgo: Open browser with search URL
/// - open-url: Open the input as a URL
/// - calculate: Evaluate math expression (basic)
/// - open-file: Open file/folder with default app
fn execute_fallback_action(
    app: &mut ScriptListApp,
    fallback_id: &str,
    input: &str,
    _window: &mut Window,
    cx: &mut Context<ScriptListApp>,
) {
    use fallbacks::builtins::{get_builtin_fallbacks, FallbackResult};

    logging::log(
        "FALLBACK",
        &format!("Executing fallback '{}' with input: {}", fallback_id, input),
    );

    // Find the fallback by ID
    let fallbacks = get_builtin_fallbacks();
    let fallback = fallbacks.iter().find(|f| f.id == fallback_id);

    let Some(fallback) = fallback else {
        logging::log("FALLBACK", &format!("Unknown fallback ID: {}", fallback_id));
        return;
    };

    // Execute the fallback and get the result
    match fallback.execute(input) {
        Ok(result) => {
            match result {
                FallbackResult::RunTerminal { command } => {
                    logging::log("FALLBACK", &format!("RunTerminal: {}", command));
                    // Open Terminal.app with the command
                    #[cfg(target_os = "macos")]
                    {
                        // Use AppleScript to open Terminal and run the command
                        let script = format!(
                            r#"tell application "Terminal"
                                activate
                                do script "{}"
                            end tell"#,
                            command.replace("\"", "\\\"").replace("\\", "\\\\")
                        );
                        match std::process::Command::new("osascript")
                            .arg("-e")
                            .arg(&script)
                            .spawn()
                        {
                            Ok(_) => logging::log("FALLBACK", "Opened Terminal with command"),
                            Err(e) => {
                                logging::log("FALLBACK", &format!("Failed to open Terminal: {}", e))
                            }
                        }
                    }
                    #[cfg(not(target_os = "macos"))]
                    {
                        logging::log("FALLBACK", "RunTerminal not implemented for this platform");
                    }
                }

                FallbackResult::AddNote { content } => {
                    logging::log("FALLBACK", &format!("AddNote: {}", content));
                    // First copy content to clipboard so user can paste it
                    let item = gpui::ClipboardItem::new_string(content.clone());
                    cx.write_to_clipboard(item);
                    // Then open Notes window - user can paste with Cmd+V
                    if let Err(e) = notes::open_notes_window(cx) {
                        logging::log("FALLBACK", &format!("Failed to open Notes: {}", e));
                    } else {
                        hud_manager::show_hud(
                            "Text copied - paste into Notes".to_string(),
                            Some(2000),
                            cx,
                        );
                    }
                }

                FallbackResult::Copy { text } => {
                    logging::log("FALLBACK", &format!("Copy: {} chars", text.len()));
                    // Copy to clipboard using GPUI
                    let item = gpui::ClipboardItem::new_string(text);
                    cx.write_to_clipboard(item);
                    logging::log("FALLBACK", "Text copied to clipboard");
                }

                FallbackResult::OpenUrl { url } => {
                    logging::log("FALLBACK", &format!("OpenUrl: {}", url));
                    // Open URL in default browser
                    if let Err(e) = open::that(&url) {
                        logging::log("FALLBACK", &format!("Failed to open URL: {}", e));
                    } else {
                        logging::log("FALLBACK", "URL opened in browser");
                    }
                }

                FallbackResult::Calculate { expression } => {
                    logging::log("FALLBACK", &format!("Calculate: {}", expression));
                    // Basic math evaluation using meval crate
                    match meval::eval_str(&expression) {
                        Ok(result) => {
                            let result_str = result.to_string();
                            logging::log("FALLBACK", &format!("Result: {}", result_str));
                            // Copy result to clipboard
                            let item = gpui::ClipboardItem::new_string(result_str.clone());
                            cx.write_to_clipboard(item);
                            // Show HUD with result
                            hud_manager::show_hud(format!("= {}", result_str), Some(2000), cx);
                        }
                        Err(e) => {
                            logging::log("FALLBACK", &format!("Calculation error: {}", e));
                            hud_manager::show_hud(format!("Error: {}", e), Some(3000), cx);
                        }
                    }
                }

                FallbackResult::OpenFile { path } => {
                    logging::log("FALLBACK", &format!("OpenFile: {}", path));
                    // Expand ~ to home directory
                    let expanded = shellexpand::tilde(&path).to_string();
                    // Open with default application
                    if let Err(e) = open::that(&expanded) {
                        logging::log("FALLBACK", &format!("Failed to open file: {}", e));
                    } else {
                        logging::log("FALLBACK", "File opened with default application");
                    }
                }

                FallbackResult::SearchFiles { query } => {
                    logging::log("FALLBACK", &format!("SearchFiles: {}", query));
                    app.open_file_search(query, cx);
                }
            }
        }
        Err(e) => {
            logging::log("FALLBACK", &format!("Fallback execution failed: {}", e));
        }
    }
}

/// Register bundled JetBrains Mono font with GPUI's text system
///
/// This embeds the font files directly in the binary and registers them
/// at application startup, making "JetBrains Mono" available as a font family.
fn register_bundled_fonts(cx: &mut App) {
    use std::borrow::Cow;

    // Embed font files at compile time
    static JETBRAINS_MONO_REGULAR: &[u8] =
        include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf");
    static JETBRAINS_MONO_BOLD: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-Bold.ttf");
    static JETBRAINS_MONO_ITALIC: &[u8] =
        include_bytes!("../assets/fonts/JetBrainsMono-Italic.ttf");
    static JETBRAINS_MONO_BOLD_ITALIC: &[u8] =
        include_bytes!("../assets/fonts/JetBrainsMono-BoldItalic.ttf");
    static JETBRAINS_MONO_MEDIUM: &[u8] =
        include_bytes!("../assets/fonts/JetBrainsMono-Medium.ttf");
    static JETBRAINS_MONO_SEMIBOLD: &[u8] =
        include_bytes!("../assets/fonts/JetBrainsMono-SemiBold.ttf");

    let fonts: Vec<Cow<'static, [u8]>> = vec![
        Cow::Borrowed(JETBRAINS_MONO_REGULAR),
        Cow::Borrowed(JETBRAINS_MONO_BOLD),
        Cow::Borrowed(JETBRAINS_MONO_ITALIC),
        Cow::Borrowed(JETBRAINS_MONO_BOLD_ITALIC),
        Cow::Borrowed(JETBRAINS_MONO_MEDIUM),
        Cow::Borrowed(JETBRAINS_MONO_SEMIBOLD),
    ];

    match cx.text_system().add_fonts(fonts) {
        Ok(()) => {
            logging::log("FONT", "Registered JetBrains Mono font family (6 styles)");
        }
        Err(e) => {
            logging::log(
                "FONT",
                &format!(
                    "Failed to register JetBrains Mono: {}. Falling back to system font.",
                    e
                ),
            );
        }
    }
}

/// Application state - what view are we currently showing
#[derive(Debug, Clone)]
enum AppView {
    /// Showing the script list
    ScriptList,
    /// Showing the actions dialog (mini searchable popup)
    #[allow(dead_code)]
    ActionsDialog,
    /// Showing an arg prompt from a script
    ArgPrompt {
        id: String,
        placeholder: String,
        choices: Vec<Choice>,
        actions: Option<Vec<ProtocolAction>>,
    },
    /// Showing a div prompt from a script
    DivPrompt {
        #[allow(dead_code)]
        id: String,
        entity: Entity<DivPrompt>,
    },
    /// Showing a form prompt from a script (HTML form with submit button)
    FormPrompt {
        #[allow(dead_code)]
        id: String,
        entity: Entity<FormPromptState>,
    },
    /// Showing a terminal prompt from a script
    TermPrompt {
        #[allow(dead_code)]
        id: String,
        entity: Entity<term_prompt::TermPrompt>,
    },
    /// Showing an editor prompt from a script (gpui-component based with Find/Replace)
    EditorPrompt {
        #[allow(dead_code)]
        id: String,
        entity: Entity<EditorPrompt>,
        /// Separate focus handle for the editor (not shared with parent)
        /// Note: This is kept for API compatibility but focus is managed via entity.focus()
        #[allow(dead_code)]
        focus_handle: FocusHandle,
    },
    /// Showing a select prompt from a script (multi-select)
    SelectPrompt {
        #[allow(dead_code)]
        id: String,
        entity: Entity<SelectPrompt>,
    },
    /// Showing a path prompt from a script (file/folder picker)
    PathPrompt {
        #[allow(dead_code)]
        id: String,
        entity: Entity<PathPrompt>,
        focus_handle: FocusHandle,
    },
    /// Showing env prompt for environment variable input with keyring storage
    EnvPrompt {
        #[allow(dead_code)]
        id: String,
        entity: Entity<EnvPrompt>,
    },
    /// Showing drop prompt for drag and drop file handling
    DropPrompt {
        #[allow(dead_code)]
        id: String,
        entity: Entity<DropPrompt>,
    },
    /// Showing template prompt for string template editing
    TemplatePrompt {
        #[allow(dead_code)]
        id: String,
        entity: Entity<TemplatePrompt>,
    },
    /// Showing clipboard history
    /// P0 FIX: View state only - data comes from clipboard_history module cache
    ClipboardHistoryView {
        filter: String,
        selected_index: usize,
    },
    /// Showing app launcher
    /// P0 FIX: View state only - data comes from ScriptListApp.apps or app_launcher module
    AppLauncherView {
        filter: String,
        selected_index: usize,
    },
    /// Showing window switcher
    /// P0 FIX: View state only - windows stored in ScriptListApp.cached_windows
    WindowSwitcherView {
        filter: String,
        selected_index: usize,
    },
    /// Showing design gallery (separator and icon variations)
    DesignGalleryView {
        filter: String,
        selected_index: usize,
    },
    /// Showing scratch pad editor (auto-saves to disk)
    ScratchPadView {
        entity: Entity<EditorPrompt>,
        #[allow(dead_code)]
        focus_handle: FocusHandle,
    },
    /// Showing quick terminal
    QuickTerminalView {
        entity: Entity<term_prompt::TermPrompt>,
    },
    /// Showing file search results
    FileSearchView {
        query: String,
        selected_index: usize,
    },
}

/// Wrapper to hold a script session that can be shared across async boundaries
/// Uses parking_lot::Mutex which doesn't poison on panic, avoiding .unwrap() calls
type SharedSession = Arc<ParkingMutex<Option<executor::ScriptSession>>>;

/// Tracks which input field currently has focus for cursor display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusedInput {
    /// Main script list filter input
    MainFilter,
    /// Actions dialog search input
    ActionsSearch,
    /// Arg prompt input (when running a script)
    ArgPrompt,
    /// No input focused (e.g., terminal prompt)
    None,
}

/// Pending focus target - identifies which element should receive focus
/// when window access becomes available. This prevents the "perpetual focus
/// enforcement in render()" anti-pattern that causes focus thrash.
///
/// Focus is applied once when pending_focus is set, then cleared.
/// This mechanism allows non-render code paths (like handle_prompt_message)
/// to request focus changes that are applied on the next render.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusTarget {
    /// Focus the main filter input (gpui_input_state)
    MainFilter,
    /// Focus the app root (self.focus_handle)
    AppRoot,
    /// Focus the actions dialog (if open)
    ActionsDialog,
    /// Focus the path prompt's focus handle
    PathPrompt,
    /// Focus the form prompt (delegates to active field)
    FormPrompt,
    /// Focus the editor prompt
    EditorPrompt,
    /// Focus the select prompt
    SelectPrompt,
    /// Focus the env prompt
    EnvPrompt,
    /// Focus the drop prompt
    DropPrompt,
    /// Focus the template prompt
    TemplatePrompt,
    /// Focus the term prompt
    TermPrompt,
}

/// Identifies which prompt type is hosting the actions dialog.
///
/// This determines focus restoration behavior when the dialog closes,
/// since different prompt types have different focus targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // MainList variant reserved for render_script_list.rs refactoring
enum ActionsDialogHost {
    /// Actions in arg prompt (restore focus to ArgPrompt input)
    ArgPrompt,
    /// Actions in div prompt (restore focus to None - div has no input)
    DivPrompt,
    /// Actions in editor prompt (restore focus to None - editor handles its own focus)
    EditorPrompt,
    /// Actions in term prompt (restore focus to None - terminal handles its own focus)
    TermPrompt,
    /// Actions in form prompt (restore focus to None - form handles field focus)
    FormPrompt,
    /// Actions in main script list (restore focus to MainFilter)
    MainList,
    /// Actions in file search (restore focus to file search input)
    FileSearch,
}

/// Result of routing a key event to the actions dialog.
///
/// Returned by `route_key_to_actions_dialog` to indicate how the caller
/// should proceed after routing.
#[derive(Debug, Clone)]
enum ActionsRoute {
    /// Actions popup is not open - key was not handled, caller should process normally
    NotHandled,
    /// Key was handled by the actions dialog - caller should return/stop propagation
    Handled,
    /// User selected an action - caller should execute it via trigger_action_by_name
    Execute { action_id: String },
}

/// State for the inline shortcut recorder overlay.
///
/// When this is Some, the ShortcutRecorder modal is displayed.
/// Used for configuring keyboard shortcuts without opening an external editor.
#[derive(Debug, Clone)]
struct ShortcutRecorderState {
    /// The unique command identifier (e.g., "scriptlet/my-script", "builtin/clipboard-history")
    command_id: String,
    /// Human-readable name of the command being configured
    command_name: String,
}

/// State for the inline alias input overlay.
///
/// When this is Some, the alias input modal is displayed.
/// Used for configuring command aliases.
#[derive(Debug, Clone)]
struct AliasInputState {
    /// The unique command identifier (e.g., "builtin/clipboard-history", "app/com.apple.Safari")
    command_id: String,
    /// Human-readable name of the command being configured
    command_name: String,
    /// Current alias text being edited
    alias_text: String,
}

/// Messages sent from the prompt poller back to the main app
#[derive(Debug, Clone)]
enum PromptMessage {
    ShowArg {
        id: String,
        placeholder: String,
        choices: Vec<Choice>,
        actions: Option<Vec<ProtocolAction>>,
    },
    ShowDiv {
        id: String,
        html: String,
        /// Tailwind classes for the content container
        container_classes: Option<String>,
        actions: Option<Vec<ProtocolAction>>,
        /// Placeholder text (header)
        placeholder: Option<String>,
        /// Hint text
        hint: Option<String>,
        /// Footer text
        footer: Option<String>,
        /// Container background color
        container_bg: Option<String>,
        /// Container padding (number or "none")
        container_padding: Option<serde_json::Value>,
        /// Container opacity (0-100)
        opacity: Option<u8>,
    },
    ShowForm {
        id: String,
        html: String,
        actions: Option<Vec<ProtocolAction>>,
    },
    ShowTerm {
        id: String,
        command: Option<String>,
        actions: Option<Vec<ProtocolAction>>,
    },
    ShowEditor {
        id: String,
        content: Option<String>,
        language: Option<String>,
        template: Option<String>,
        actions: Option<Vec<ProtocolAction>>,
    },
    /// Path picker prompt for file/folder selection
    ShowPath {
        id: String,
        start_path: Option<String>,
        hint: Option<String>,
    },
    /// Environment variable prompt with optional secret handling
    ShowEnv {
        id: String,
        key: String,
        prompt: Option<String>,
        secret: bool,
    },
    /// Drag and drop prompt for file uploads
    ShowDrop {
        id: String,
        placeholder: Option<String>,
        hint: Option<String>,
    },
    /// Template prompt for tab-through string templates
    ShowTemplate {
        id: String,
        template: String,
    },
    /// Multi-select prompt from choices
    ShowSelect {
        id: String,
        placeholder: Option<String>,
        choices: Vec<Choice>,
        multiple: bool,
    },
    /// Confirmation dialog with yes/no choice
    ShowConfirm {
        id: String,
        message: String,
        confirm_text: Option<String>,
        cancel_text: Option<String>,
    },
    HideWindow,
    OpenBrowser {
        url: String,
    },
    ScriptExit,
    /// External command to run a script by path
    RunScript {
        path: String,
    },
    /// Script error with detailed information for toast display
    ScriptError {
        error_message: String,
        stderr_output: Option<String>,
        exit_code: Option<i32>,
        stack_trace: Option<String>,
        script_path: String,
        suggestions: Vec<String>,
    },
    /// Protocol parsing error reported from script stdout
    ProtocolError {
        correlation_id: String,
        summary: String,
        details: Option<String>,
        severity: ErrorSeverity,
        script_path: String,
    },
    /// Unhandled message type from script - shows warning toast
    UnhandledMessage {
        message_type: String,
    },
    /// Request to get current UI state - triggers StateResult response
    GetState {
        request_id: String,
    },
    /// Request to get layout info with component tree and computed styles
    GetLayoutInfo {
        request_id: String,
    },
    /// Force submit the current prompt with a value (from SDK's submit() function)
    ForceSubmit {
        value: serde_json::Value,
    },
    /// Set the current prompt input text
    SetInput {
        text: String,
    },
    /// Show HUD overlay message
    ShowHud {
        text: String,
        duration_ms: Option<u64>,
    },
    /// Set SDK actions for the ActionsDialog
    SetActions {
        actions: Vec<protocol::ProtocolAction>,
    },
    /// Show the debug grid overlay
    ShowGrid {
        options: protocol::GridOptions,
    },
    /// Hide the debug grid overlay
    HideGrid,
}

struct ScriptListApp {
    /// H1 Optimization: Arc-wrapped scripts for cheap cloning during filter operations
    scripts: Vec<std::sync::Arc<scripts::Script>>,
    /// H1 Optimization: Arc-wrapped scriptlets for cheap cloning during filter operations
    scriptlets: Vec<std::sync::Arc<scripts::Scriptlet>>,
    builtin_entries: Vec<builtins::BuiltInEntry>,
    /// Cached list of installed applications for main search and AppLauncherView
    apps: Vec<app_launcher::AppInfo>,
    /// P0 FIX: Cached clipboard entries for ClipboardHistoryView (avoids cloning per frame)
    cached_clipboard_entries: Vec<clipboard_history::ClipboardEntryMeta>,
    /// P0 FIX: Cached windows for WindowSwitcherView (avoids cloning per frame)
    cached_windows: Vec<window_control::WindowInfo>,
    /// Cached file results for FileSearchView (avoids cloning per frame)
    cached_file_results: Vec<file_search::FileResult>,
    selected_index: usize,
    /// Main menu filter text (mirrors gpui-component input state)
    filter_text: String,
    /// gpui-component input state for the main filter
    gpui_input_state: Entity<InputState>,
    gpui_input_focused: bool,
    #[allow(dead_code)]
    gpui_input_subscriptions: Vec<Subscription>,
    /// Suppress handling of programmatic InputEvent::Change updates.
    suppress_filter_events: bool,
    /// Sync gpui input text on next render when window access is available.
    pending_filter_sync: bool,
    /// Pending placeholder text to set on next render (needs Window access).
    pending_placeholder: Option<String>,
    last_output: Option<SharedString>,
    focus_handle: FocusHandle,
    show_logs: bool,
    /// Theme wrapped in Arc for cheap cloning when passing to prompts/dialogs
    theme: std::sync::Arc<theme::Theme>,
    #[allow(dead_code)]
    config: config::Config,
    // Scroll activity tracking for scrollbar fade
    /// Whether scroll activity is happening (scrollbar should be visible)
    is_scrolling: bool,
    /// Timestamp of last scroll activity (for fade-out timer)
    last_scroll_time: Option<std::time::Instant>,
    // Interactive script state
    current_view: AppView,
    script_session: SharedSession,
    // Prompt-specific state (used when view is ArgPrompt or DivPrompt)
    // Uses TextInputState for selection and clipboard support
    arg_input: TextInputState,
    arg_selected_index: usize,
    // Channel for receiving prompt messages from script thread (async_channel for event-driven)
    prompt_receiver: Option<async_channel::Receiver<PromptMessage>>,
    // Channel for sending responses back to script
    // FIX: Use SyncSender (bounded channel) to prevent OOM from slow scripts
    response_sender: Option<mpsc::SyncSender<Message>>,
    // List state for variable-height list (supports section headers at 24px + items at 48px)
    main_list_state: ListState,
    // Scroll handle for uniform_list (still used for backward compat in some views)
    list_scroll_handle: UniformListScrollHandle,
    // P0: Scroll handle for virtualized arg prompt choices
    arg_list_scroll_handle: UniformListScrollHandle,
    // Scroll handle for clipboard history list
    clipboard_list_scroll_handle: UniformListScrollHandle,
    // Scroll handle for window switcher list
    window_list_scroll_handle: UniformListScrollHandle,
    // Scroll handle for design gallery list
    design_gallery_scroll_handle: UniformListScrollHandle,
    // Scroll handle for file search list
    file_search_scroll_handle: UniformListScrollHandle,
    // File search loading state (true while mdfind is running)
    file_search_loading: bool,
    // Debounce task for file search (cancelled when new input arrives)
    file_search_debounce_task: Option<gpui::Task<()>>,
    // Current directory being listed (for instant filter mode)
    file_search_current_dir: Option<String>,
    // Frozen filter during directory transitions (prevents wrong results flash)
    // When Some, use this filter instead of deriving from query
    // Outer Option: None = use query filter, Some = use frozen filter
    // Inner Option: None = no filter, Some(s) = filter by s
    file_search_frozen_filter: Option<Option<String>>,
    // Path of the file selected for actions (for file search actions handling)
    file_search_actions_path: Option<String>,
    // Actions popup overlay
    show_actions_popup: bool,
    // ActionsDialog entity for focus management
    actions_dialog: Option<Entity<ActionsDialog>>,
    // Cursor blink state and focus tracking
    cursor_visible: bool,
    /// Which input currently has focus (for cursor display)
    focused_input: FocusedInput,
    // Current script process PID for explicit cleanup (belt-and-suspenders)
    current_script_pid: Option<u32>,
    // P1: Cache for filtered_results() - invalidate on filter_text change only
    cached_filtered_results: Vec<scripts::SearchResult>,
    filter_cache_key: String,
    // P1: Cache for get_grouped_results() - invalidate on filter_text change only
    // This avoids recomputing grouped results 9+ times per keystroke
    // P1-Arc: Use Arc<[T]> for cheap clone in render closures
    cached_grouped_items: Arc<[GroupedListItem]>,
    cached_grouped_flat_results: Arc<[scripts::SearchResult]>,
    grouped_cache_key: String,
    // P3: Two-stage filter - display vs search separation with coalescing
    /// What the search cache is built from (may lag behind filter_text during rapid typing)
    computed_filter_text: String,
    /// Coalesces filter updates and keeps only the latest value per tick
    filter_coalescer: FilterCoalescer,
    // Scroll stabilization: track last scrolled-to index to avoid redundant scroll_to_item calls
    last_scrolled_index: Option<usize>,
    // Preview cache: avoid re-reading file and re-highlighting on every render
    preview_cache_path: Option<String>,
    preview_cache_lines: Vec<syntax::HighlightedLine>,
    // Current design variant for hot-swappable UI designs
    current_design: DesignVariant,
    // Toast manager for notification queue
    toast_manager: ToastManager,
    // Cache for decoded clipboard images (entry_id -> RenderImage)
    clipboard_image_cache: std::collections::HashMap<String, Arc<gpui::RenderImage>>,
    // Frecency store for tracking script usage
    frecency_store: FrecencyStore,
    // Mouse hover tracking - independent from selected_index (keyboard focus)
    // hovered_index shows subtle visual feedback, selected_index shows full focus styling
    hovered_index: Option<usize>,
    // Fallback mode: when true, we're showing fallback commands instead of scripts
    // This happens when filter_text doesn't match any scripts
    fallback_mode: bool,
    // Selected index within the fallback list (0-based)
    fallback_selected_index: usize,
    // Cached fallback items for the current filter_text
    cached_fallbacks: Vec<crate::fallbacks::FallbackItem>,
    // P0-2: Debounce hover notify calls (16ms window to reduce 50% unnecessary re-renders)
    last_hover_notify: std::time::Instant,
    // Pending path action - when set, show ActionsDialog for this path
    // Uses Arc<Mutex<>> so callbacks can write to it
    pending_path_action: Arc<Mutex<Option<PathInfo>>>,
    // Signal to close path actions dialog (set by callback on Escape/__cancel__)
    close_path_actions: Arc<Mutex<bool>>,
    // Shared state: whether path actions dialog is currently showing
    // Used by PathPrompt to implement toggle behavior for Cmd+K
    path_actions_showing: Arc<Mutex<bool>>,
    // Shared state: current search text in path actions dialog
    // Used by PathPrompt to display search in header (like main menu does)
    path_actions_search_text: Arc<Mutex<String>>,
    // DEPRECATED: These mutexes were used for polling in render before event-based refactor.
    // Kept for reset_to_script_list cleanup. Will be removed in future cleanup pass.
    #[allow(dead_code)]
    pending_path_action_result: Arc<Mutex<Option<(String, PathInfo)>>>,
    /// Alias registry: lowercase_alias -> script_path (for O(1) lookup)
    /// Conflict rule: first-registered wins
    alias_registry: std::collections::HashMap<String, String>,
    /// Shortcut registry: shortcut -> script_path (for O(1) lookup)
    /// Conflict rule: first-registered wins
    shortcut_registry: std::collections::HashMap<String, String>,
    /// SDK actions set via setActions() - stored for trigger_action_by_name lookup
    sdk_actions: Option<Vec<protocol::ProtocolAction>>,
    /// SDK action shortcuts: normalized_shortcut -> action_name (for O(1) lookup)
    action_shortcuts: std::collections::HashMap<String, String>,
    /// Debug grid overlay configuration (None = hidden)
    grid_config: Option<debug_grid::GridConfig>,
    // Navigation coalescing for rapid arrow key events (20ms window)
    // NOTE: Currently unused - arrow keys handled in interceptor without coalescing
    #[allow(dead_code)]
    nav_coalescer: NavCoalescer,
    // Wheel scroll accumulator for smooth trackpad scrolling
    // Accumulates fractional deltas until they cross 1.0, then converts to item steps
    wheel_accum: f32,
    // Window focus tracking - for detecting focus lost and auto-dismissing prompts
    // When window loses focus while in a dismissable prompt, close and reset
    was_window_focused: bool,
    /// Pin state - when true, window stays open on blur (only closes via ESC/Cmd+W)
    /// Toggle with Cmd+Shift+P
    is_pinned: bool,
    /// Pending focus target - when set, focus will be applied once on next render
    /// then cleared. This avoids the "perpetually enforce focus in render()" anti-pattern.
    pending_focus: Option<FocusTarget>,
    // Show warning banner when bun is not available
    show_bun_warning: bool,
    // Pending confirmation: when set, the entry with this ID is awaiting confirmation
    // Used for dangerous actions like Shut Down, Restart, Log Out, Empty Trash
    pending_confirmation: Option<String>,
    // Scroll stabilization: track last scrolled-to index for each scroll handle
    #[allow(dead_code)]
    last_scrolled_main: Option<usize>,
    #[allow(dead_code)]
    last_scrolled_arg: Option<usize>,
    #[allow(dead_code)]
    last_scrolled_clipboard: Option<usize>,
    #[allow(dead_code)]
    last_scrolled_window: Option<usize>,
    #[allow(dead_code)]
    last_scrolled_design_gallery: Option<usize>,
    // Menu bar integration: Now handled by frontmost_app_tracker module
    // which pre-fetches menu items in background when apps activate
    /// Shortcut recorder state - when Some, shows the inline recorder overlay
    shortcut_recorder_state: Option<ShortcutRecorderState>,
    /// The shortcut recorder entity (persisted to maintain focus)
    shortcut_recorder_entity:
        Option<Entity<crate::components::shortcut_recorder::ShortcutRecorder>>,
    /// Alias input state - when Some, shows the alias input modal
    alias_input_state: Option<AliasInputState>,
    /// Input history for shell-like up/down navigation through previous inputs
    input_history: input_history::InputHistory,
    /// Pending API key configuration - tracks which provider is being configured
    /// Used to show success toast after EnvPrompt completes
    pending_api_key_config: Option<String>,
    /// Sender for API key configuration completion signals
    /// The EnvPrompt callback uses this to signal when done
    api_key_completion_sender: mpsc::SyncSender<(String, bool)>,
    /// Receiver for API key configuration completion signals
    /// Checked by timer to trigger toast and view reset
    api_key_completion_receiver: mpsc::Receiver<(String, bool)>,
    /// Whether the current built-in view was opened from the main menu.
    /// When true, ESC returns to main menu. When false (opened via hotkey/protocol), ESC closes window.
    opened_from_main_menu: bool,
}

/// Result of alias matching - either a Script or Scriptlet
#[derive(Clone, Debug)]
enum AliasMatch {
    Script(Arc<scripts::Script>),
    Scriptlet(Arc<scripts::Scriptlet>),
    BuiltIn(Arc<builtins::BuiltInEntry>),
    App(Arc<app_launcher::AppInfo>),
}

// Core ScriptListApp implementation extracted to app_impl.rs
include!("app_impl.rs");

// Script execution logic (execute_interactive) extracted
include!("execute_script.rs");

// Prompt message handling (handle_prompt_message) extracted
include!("prompt_handler.rs");

// App navigation methods (selection movement, scrolling)
include!("app_navigation.rs");

// App execution methods (execute_builtin, execute_app, execute_window_focus)
include!("app_execute.rs");

// App actions handling (handle_action, trigger_action_by_name)
include!("app_actions.rs");

// Layout calculation methods (build_component_bounds, build_layout_info)
include!("app_layout.rs");

impl Focusable for ScriptListApp {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ScriptListApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Flush any pending toasts to gpui-component's NotificationList
        // This is needed because toast push sites don't have window access
        self.flush_pending_toasts(window, cx);

        // Check for API key configuration completion (from built-in commands)
        // The EnvPrompt callback signals completion via channel
        if let Ok((provider, success)) = self.api_key_completion_receiver.try_recv() {
            self.handle_api_key_completion(provider, success, cx);
        }

        // Focus-lost auto-dismiss: Close dismissable prompts when the main window loses focus
        // This includes focus loss to other app windows like Notes/AI.
        // When is_pinned is true, the window stays open on blur (only closes via ESC/Cmd+W)
        let is_window_focused = platform::is_main_window_focused();
        if self.was_window_focused && !is_window_focused {
            // Window just lost focus (user clicked another window)
            // Only auto-dismiss if we're in a dismissable view AND window is visible AND not pinned
            if self.is_dismissable_view()
                && script_kit_gpui::is_main_window_visible()
                && !self.is_pinned
            {
                logging::log(
                    "FOCUS",
                    "Main window lost focus while in dismissable view - closing",
                );
                self.close_and_reset_window(cx);
            } else if self.is_pinned {
                logging::log(
                    "FOCUS",
                    "Main window lost focus but is pinned - staying open",
                );
            }
        }
        self.was_window_focused = is_window_focused;

        // Apply pending focus request (if any). This is the new "apply once" mechanism
        // that replaces the old "perpetually enforce focus in render()" pattern.
        // Focus is applied exactly once when pending_focus is set, then cleared.
        self.apply_pending_focus(window, cx);

        // Sync filter input if needed (views that use shared input)
        if matches!(
            self.current_view,
            AppView::ScriptList
                | AppView::ClipboardHistoryView { .. }
                | AppView::AppLauncherView { .. }
                | AppView::WindowSwitcherView { .. }
                | AppView::FileSearchView { .. }
        ) {
            self.sync_filter_input_if_needed(window, cx);
        }

        // NOTE: Prompt messages are now handled via event-driven async_channel listener
        // spawned in execute_interactive() - no polling needed in render()

        // P0-4: Clone current_view only for dispatch (needed to call &mut self methods)
        // The clone is unavoidable due to borrow checker: we need &mut self for render methods
        // but also need to match on self.current_view. Future optimization: refactor render
        // methods to take &str/&[T] references instead of owned values.
        //
        // HUD is now handled by hud_manager as a separate floating window
        // No need to render it as part of this view
        let current_view = self.current_view.clone();
        let main_content: AnyElement = match current_view {
            AppView::ScriptList => self.render_script_list(cx).into_any_element(),
            AppView::ActionsDialog => self.render_actions_dialog(cx),
            AppView::ArgPrompt {
                id,
                placeholder,
                choices,
                actions,
            } => self
                .render_arg_prompt(id, placeholder, choices, actions, cx)
                .into_any_element(),
            AppView::DivPrompt { id, entity } => {
                self.render_div_prompt(id, entity, cx).into_any_element()
            }
            AppView::FormPrompt { entity, .. } => {
                self.render_form_prompt(entity, cx).into_any_element()
            }
            AppView::TermPrompt { entity, .. } => {
                self.render_term_prompt(entity, cx).into_any_element()
            }
            AppView::EditorPrompt { entity, .. } => {
                self.render_editor_prompt(entity, cx).into_any_element()
            }
            AppView::SelectPrompt { entity, .. } => {
                self.render_select_prompt(entity, cx).into_any_element()
            }
            AppView::PathPrompt { entity, .. } => {
                self.render_path_prompt(entity, cx).into_any_element()
            }
            AppView::EnvPrompt { entity, .. } => {
                self.render_env_prompt(entity, cx).into_any_element()
            }
            AppView::DropPrompt { entity, .. } => {
                self.render_drop_prompt(entity, cx).into_any_element()
            }
            AppView::TemplatePrompt { entity, .. } => {
                self.render_template_prompt(entity, cx).into_any_element()
            }
            // P0 FIX: View state only - data comes from self.cached_clipboard_entries
            AppView::ClipboardHistoryView {
                filter,
                selected_index,
            } => self
                .render_clipboard_history(filter, selected_index, cx)
                .into_any_element(),
            // P0 FIX: View state only - data comes from self.apps
            AppView::AppLauncherView {
                filter,
                selected_index,
            } => self
                .render_app_launcher(filter, selected_index, cx)
                .into_any_element(),
            // P0 FIX: View state only - data comes from self.cached_windows
            AppView::WindowSwitcherView {
                filter,
                selected_index,
            } => self
                .render_window_switcher(filter, selected_index, cx)
                .into_any_element(),
            AppView::DesignGalleryView {
                filter,
                selected_index,
            } => self
                .render_design_gallery(filter, selected_index, cx)
                .into_any_element(),
            AppView::ScratchPadView { entity, .. } => {
                self.render_editor_prompt(entity, cx).into_any_element()
            }
            AppView::QuickTerminalView { entity, .. } => {
                self.render_term_prompt(entity, cx).into_any_element()
            }
            AppView::FileSearchView {
                ref query,
                selected_index,
            } => self
                .render_file_search(query, selected_index, cx)
                .into_any_element(),
        };

        // Wrap content in a container that can have the debug grid overlay
        let window_bounds = window.bounds();
        let window_size = gpui::size(window_bounds.size.width, window_bounds.size.height);

        // Clone grid_config for use in the closure
        let grid_config = self.grid_config.clone();

        // Build component bounds for the current view (for debug overlay)
        // P0 FIX: Only compute bounds when grid overlay is actually enabled
        // Previously this was computed unconditionally on every frame
        let component_bounds = if grid_config.is_some() {
            self.build_component_bounds(window_size)
        } else {
            Vec::new()
        };

        // Build warning banner if needed (bun not available)
        let warning_banner = if self.show_bun_warning {
            let banner_colors = WarningBannerColors::from_theme(&self.theme);
            let entity = cx.entity().downgrade();
            let entity_for_dismiss = entity.clone();

            Some(
                div().w_full().px(px(12.)).pt(px(8.)).child(
                    WarningBanner::new(
                        "bun is not installed. Click to download from bun.sh",
                        banner_colors,
                    )
                    .on_click(Box::new(move |_event, _window, cx| {
                        if let Some(app) = entity.upgrade() {
                            app.update(cx, |this, _cx| {
                                this.open_bun_website();
                            });
                        }
                    }))
                    .on_dismiss(Box::new(move |_event, _window, cx| {
                        if let Some(app) = entity_for_dismiss.upgrade() {
                            app.update(cx, |this, cx| {
                                this.dismiss_bun_warning(cx);
                            });
                        }
                    })),
                ),
            )
        } else {
            None
        };

        // Build shortcut recorder overlay if state is set
        let shortcut_recorder_overlay = self.render_shortcut_recorder_overlay(window, cx);

        // Build alias input overlay if state is set
        let alias_input_overlay = self.render_alias_input_overlay(window, cx);

        div()
            .w_full()
            .h_full()
            .relative()
            .flex()
            .flex_col()
            // Warning banner appears at the top when bun is not available
            .when_some(warning_banner, |container, banner| container.child(banner))
            // Main content takes remaining space
            .child(div().flex_1().w_full().min_h(px(0.)).child(main_content))
            // Shortcut recorder overlay (on top of main content when recording)
            .when_some(shortcut_recorder_overlay, |container, overlay| {
                container.child(overlay)
            })
            // Alias input overlay (on top of main content when entering alias)
            .when_some(alias_input_overlay, |container, overlay| {
                container.child(overlay)
            })
            .when_some(grid_config, |container, config| {
                let overlay_bounds = gpui::Bounds {
                    origin: gpui::point(px(0.), px(0.)),
                    size: window_size,
                };
                container.child(debug_grid::render_grid_overlay(
                    &config,
                    overlay_bounds,
                    &component_bounds,
                ))
            })
    }
}

// Render methods extracted to app_render.rs for maintainability
include!("app_render.rs");

// Builtin view render methods (clipboard, app launcher, window switcher)
include!("render_builtins.rs");

// Prompt render methods - split into separate files for maintainability
// Each file adds render_*_prompt methods to ScriptListApp via impl blocks
include!("render_prompts/arg.rs");
include!("render_prompts/div.rs");
include!("render_prompts/form.rs");
include!("render_prompts/term.rs");
include!("render_prompts/editor.rs");
include!("render_prompts/path.rs");
include!("render_prompts/other.rs");

// Script list render method
include!("render_script_list.rs");

fn main() {
    logging::init();

    // Migrate from legacy ~/.kenv to new ~/.scriptkit structure (one-time migration)
    // This must happen BEFORE ensure_kit_setup() so the new path is used
    if setup::migrate_from_kenv() {
        logging::log("APP", "Migrated from ~/.kenv to ~/.scriptkit");
    }

    // Ensure ~/.scriptkit environment is properly set up (directories, SDK, config, etc.)
    // This is idempotent - it creates missing directories and files without overwriting user configs
    let setup_result = setup::ensure_kit_setup();
    if setup_result.is_fresh_install {
        logging::log(
            "APP",
            &format!(
                "Fresh install detected - created ~/.scriptkit at {}",
                setup_result.kit_path.display()
            ),
        );
    }
    for warning in &setup_result.warnings {
        logging::log("APP", &format!("Setup warning: {}", warning));
    }
    if !setup_result.bun_available {
        logging::log(
            "APP",
            "Warning: bun not found in PATH or common locations. Scripts may not run.",
        );
    }

    // Write main PID file for orphan detection on crash
    if let Err(e) = PROCESS_MANAGER.write_main_pid() {
        logging::log("APP", &format!("Failed to write main PID file: {}", e));
    } else {
        logging::log("APP", "Main PID file written");
    }

    // Clean up any orphaned processes from a previous crash
    let orphans_killed = PROCESS_MANAGER.cleanup_orphans();
    if orphans_killed > 0 {
        logging::log(
            "APP",
            &format!(
                "Cleaned up {} orphaned process(es) from previous session",
                orphans_killed
            ),
        );
    }

    // Register signal handlers for graceful shutdown
    // SAFETY: Signal handlers can only safely call async-signal-safe functions.
    // We ONLY set an atomic flag here. All cleanup (logging, killing processes,
    // removing PID files) happens in a GPUI task that monitors this flag.
    #[cfg(unix)]
    {
        extern "C" fn handle_signal(_sig: libc::c_int) {
            // ASYNC-SIGNAL-SAFE: Only set atomic flag
            // Do NOT call: logging, mutexes, heap allocation, or any Rust code
            // that might allocate or lock. The GPUI shutdown monitor task will
            // handle all cleanup on the main thread.
            SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
        }

        unsafe {
            // Register handlers for common termination signals
            libc::signal(libc::SIGINT, handle_signal as libc::sighandler_t);
            libc::signal(libc::SIGTERM, handle_signal as libc::sighandler_t);
            libc::signal(libc::SIGHUP, handle_signal as libc::sighandler_t);
            logging::log(
                "APP",
                "Signal handlers registered (SIGINT, SIGTERM, SIGHUP) - cleanup via GPUI task",
            );
        }
    }

    // Load config early so we can use it for hotkey registration AND clipboard history settings
    // This avoids duplicate config::load_config() calls (~100-300ms startup savings)
    let loaded_config = config::load_config();
    logging::log(
        "APP",
        &format!(
            "Loaded config: hotkey={:?}+{}, bun_path={:?}",
            loaded_config.hotkey.modifiers, loaded_config.hotkey.key, loaded_config.bun_path
        ),
    );
    clipboard_history::set_max_text_content_len(
        loaded_config.get_clipboard_history_max_text_length(),
    );

    // Initialize clipboard history monitoring (background thread)
    if let Err(e) = clipboard_history::init_clipboard_history() {
        logging::log(
            "APP",
            &format!("Failed to initialize clipboard history: {}", e),
        );
    } else {
        logging::log("APP", "Clipboard history monitoring initialized");
    }

    // Initialize text expansion system (background thread with keyboard monitoring)
    // This must be done early, before the GPUI run loop starts
    // Uses a global singleton so the manager can be updated when scriptlet files change
    #[cfg(target_os = "macos")]
    {
        // Spawn initialization in a thread to not block startup
        std::thread::spawn(move || {
            logging::log("KEYWORD", "Initializing text expansion system");

            match keyword_manager::init_keyword_manager() {
                Ok(Some(count)) => {
                    logging::log(
                        "KEYWORD",
                        &format!("Text expansion system enabled with {} triggers", count),
                    );
                }
                Ok(None) => {
                    logging::log(
                        "KEYWORD",
                        "Accessibility permissions not granted - text expansion disabled",
                    );
                    logging::log(
                        "KEYWORD",
                        "Enable in System Preferences > Privacy & Security > Accessibility",
                    );
                }
                Err(e) => {
                    logging::log(
                        "KEYWORD",
                        &format!("Failed to initialize text expansion: {}", e),
                    );
                }
            }
        });
    }

    // Clone before start_hotkey_listener consumes original
    let config_for_app = loaded_config.clone();

    // Start MCP server for AI agent integration
    // Server runs on localhost:43210 with Bearer token authentication
    // Discovery file written to ~/.scriptkit/server.json
    let _mcp_handle = match mcp_server::McpServer::with_defaults() {
        Ok(server) => match server.start() {
            Ok(handle) => {
                logging::log(
                    "MCP",
                    &format!(
                        "MCP server started on {} (token in ~/.scriptkit/agent-token)",
                        server.url()
                    ),
                );
                Some(handle)
            }
            Err(e) => {
                logging::log("MCP", &format!("Failed to start MCP server: {}", e));
                None
            }
        },
        Err(e) => {
            logging::log("MCP", &format!("Failed to create MCP server: {}", e));
            None
        }
    };

    hotkeys::start_hotkey_listener(loaded_config);

    // Start watchers and track which ones succeeded
    // We only spawn poll loops for watchers that successfully started
    let (mut appearance_watcher, appearance_rx) = watcher::AppearanceWatcher::new();
    let appearance_watcher_ok = match appearance_watcher.start() {
        Ok(()) => {
            logging::log("APP", "Appearance watcher started");
            true
        }
        Err(e) => {
            logging::log("APP", &format!("Failed to start appearance watcher: {}", e));
            false
        }
    };

    let (mut config_watcher, config_rx) = watcher::ConfigWatcher::new();
    let config_watcher_ok = match config_watcher.start() {
        Ok(()) => {
            logging::log("APP", "Config watcher started");
            true
        }
        Err(e) => {
            logging::log("APP", &format!("Failed to start config watcher: {}", e));
            false
        }
    };

    let (mut script_watcher, script_rx) = watcher::ScriptWatcher::new();
    let script_watcher_ok = match script_watcher.start() {
        Ok(()) => {
            logging::log("APP", "Script watcher started");
            true
        }
        Err(e) => {
            logging::log("APP", &format!("Failed to start script watcher: {}", e));
            false
        }
    };

    // Create AppWatcher for live application cache updates
    let (mut app_watcher, app_rx) = watcher::AppWatcher::new();
    let app_watcher_ok = match app_watcher.start() {
        Ok(()) => {
            logging::log("APP", "App watcher started");
            true
        }
        Err(e) => {
            logging::log("APP", &format!("Failed to start app watcher: {}", e));
            false
        }
    };

    // Initialize script scheduler
    // Creates the scheduler and scans for scripts with // Cron: or // Schedule: metadata
    let (mut scheduler, scheduler_rx) = scheduler::Scheduler::new();
    let scheduled_count = scripts::register_scheduled_scripts(&scheduler);
    logging::log(
        "APP",
        &format!("Registered {} scheduled scripts", scheduled_count),
    );

    // Start the scheduler background thread (checks every 30 seconds for due scripts)
    if scheduled_count > 0 {
        if let Err(e) = scheduler.start() {
            logging::log("APP", &format!("Failed to start scheduler: {}", e));
        } else {
            logging::log("APP", "Scheduler started successfully");
        }
    } else {
        logging::log("APP", "No scheduled scripts found, scheduler not started");
    }

    // Wrap scheduler in Arc<Mutex<>> for thread-safe access (needed for re-scanning on file changes)
    let scheduler = Arc::new(Mutex::new(scheduler));

    // Register URL scheme handler for scriptkit:// deeplinks
    // This must be done before .run() as it's called on Application
    let app = Application::new();
    app.on_open_urls(|urls| {
        logging::log("DEEPLINK", &format!("Received {} URL(s)", urls.len()));
        for url in urls {
            logging::log("DEEPLINK", &format!("Processing URL: {}", url));
            if let Some(command_id) = parse_deeplink_url(&url) {
                logging::log("DEEPLINK", &format!("Parsed command_id: {}", command_id));
                // Send to channel for processing inside the app
                if deeplink_channel().0.try_send(command_id).is_err() {
                    logging::log(
                        "DEEPLINK",
                        "Failed to send command to channel (full or closed)",
                    );
                }
            }
        }
    });
    app.run(move |cx: &mut App| {
        logging::log("APP", "GPUI Application starting");

        // Configure as accessory app FIRST, before any windows are created
        // This is equivalent to LSUIElement=true in Info.plist:
        // - No Dock icon
        // - No menu bar ownership (critical for window actions to work)
        platform::configure_as_accessory_app();

        // Start frontmost app tracker - watches for app activations and pre-fetches menu bar items
        // Must be started after configure_as_accessory_app() so we're correctly classified
        #[cfg(target_os = "macos")]
        frontmost_app_tracker::start_tracking();

        // Register bundled JetBrains Mono font
        // This makes "JetBrains Mono" available as a font family for the editor
        register_bundled_fonts(cx);

        // Initialize gpui-component (theme, context providers)
        // Must be called before opening windows that use Root wrapper
        gpui_component::init(cx);

        // Sync Script Kit theme with gpui-component's ThemeColor system
        // This ensures all gpui-component widgets use our colors
        theme::sync_gpui_component_theme(cx);

        // Start the centralized theme service for hot-reload
        // This replaces per-window theme watchers and ensures all windows
        // stay in sync with theme.json changes
        theme::service::ensure_theme_service(cx);

        // Initialize tray icon and menu
        // MUST be done after Application::new() creates the NSApplication
        let tray_manager = match TrayManager::new() {
            Ok(tm) => {
                logging::log("TRAY", "Tray icon initialized successfully");
                Some(tm)
            }
            Err(e) => {
                logging::log("TRAY", &format!("Failed to initialize tray icon: {}", e));
                None
            }
        };

        // Calculate window bounds: try saved position first, then eye-line
        let window_size = size(px(750.), initial_window_height());
        let default_bounds = calculate_eye_line_bounds_on_mouse_display(window_size);
        let displays = platform::get_macos_displays();
        let bounds = window_state::get_initial_bounds(
            window_state::WindowRole::Main,
            default_bounds,
            &displays,
        );

        // Load theme to determine window background appearance (vibrancy)
        let initial_theme = theme::load_theme();
        let window_background = if initial_theme.is_vibrancy_enabled() {
            WindowBackgroundAppearance::Blurred
        } else {
            WindowBackgroundAppearance::Opaque
        };
        logging::log(
            "THEME",
            &format!(
                "Window background appearance: {:?} (vibrancy_enabled={})",
                window_background,
                initial_theme.is_vibrancy_enabled()
            ),
        );

        // Store the ScriptListApp entity for direct access (needed since Root wraps the view)
        let app_entity_holder: Arc<Mutex<Option<Entity<ScriptListApp>>>> = Arc::new(Mutex::new(None));
        let app_entity_for_closure = app_entity_holder.clone();

        // Capture bun_available for use in window creation
        let bun_available = setup_result.bun_available;

        let window: WindowHandle<Root> = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: None,
                is_movable: true,
                window_background,
                show: false, // Start hidden - only show on hotkey press
                focus: false, // Don't focus on creation
                ..Default::default()
            },
            |window, cx| {
                logging::log("APP", "Window opened, creating ScriptListApp wrapped in Root");
                let view = cx.new(|cx| ScriptListApp::new(config_for_app, bun_available, window, cx));
                // Store the entity for external access
                *app_entity_for_closure.lock().unwrap() = Some(view.clone());
                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .unwrap();

        // Extract the app entity for use in callbacks
        let app_entity = app_entity_holder.lock().unwrap().clone().expect("App entity should be set");

        // Set initial focus via the Root window
        // We access the app entity within the window context to properly focus it
        let app_entity_for_focus = app_entity.clone();
        window
            .update(cx, |_root, win, root_cx| {
                app_entity_for_focus.update(root_cx, |view, ctx| {
                    let focus_handle = view.focus_handle(ctx);
                    win.focus(&focus_handle, ctx);
                    logging::log("APP", "Focus set on ScriptListApp via Root");
                });
            })
            .unwrap();

        // Register the main window with WindowManager before tray init
        // This must happen after GPUI creates the window but before tray creates its windows
        // so we can reliably find our main window by its expected size (~750x500)
        window_manager::find_and_register_main_window();

        // HACK: Swizzle GPUI's BlurredView IMMEDIATELY after window creation
        // GPUI hides the native macOS CAChameleonLayer (vibrancy tint) on every frame.
        // By swizzling now (before any rendering), we preserve the native tint effect.
        // This gives us Raycast/Spotlight-like vibrancy appearance.
        platform::swizzle_gpui_blurred_view();

        // Window starts hidden - no activation, no panel configuration yet
        // Panel will be configured on first show via hotkey
        // WINDOW_VISIBLE is already false by default (static initializer)
        logging::log("HOTKEY", "Window created but not shown (use hotkey to show)");

        // Fallback: If both hotkey AND tray fail, the user has no way to access the app!
        // Wait a short time for hotkey registration, then check if we need to show the window.
        let tray_ok = tray_manager.is_some();
        let window_for_fallback = window;
        let app_entity_for_fallback = app_entity.clone();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            // Wait 500ms for hotkey registration to complete (it runs in a separate thread)
            Timer::after(std::time::Duration::from_millis(500)).await;

            let hotkey_ok = hotkeys::is_main_hotkey_registered();

            if !hotkey_ok && !tray_ok {
                logging::log("APP", "");
                logging::log("APP", "╔════════════════════════════════════════════════════════════════════════════╗");
                logging::log("APP", "║  WARNING: Both hotkey and tray initialization failed!                     ║");
                logging::log("APP", "║  Showing window at startup as fallback entry point.                       ║");
                logging::log("APP", "║  Check logs for specific errors.                                          ║");
                logging::log("APP", "╚════════════════════════════════════════════════════════════════════════════╝");
                logging::log("APP", "");

                // Show window using the centralized helper
                let _ = cx.update(|cx| {
                    show_main_window_helper(window_for_fallback, app_entity_for_fallback, cx);
                });
            } else {
                logging::log("APP", &format!("Entry points available: hotkey={}, tray={}", hotkey_ok, tray_ok));
            }
        }).detach();

        // Main window hotkey listener - uses Entity<ScriptListApp> instead of WindowHandle
        let app_entity_for_hotkey = app_entity.clone();
        let window_for_hotkey = window;
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            logging::log("HOTKEY", "Main hotkey listener started");
            while let Ok(()) = hotkeys::hotkey_channel().1.recv().await {
                logging::log("VISIBILITY", "");
                logging::log("VISIBILITY", "╔════════════════════════════════════════════════════════════╗");
                logging::log("VISIBILITY", "║  HOTKEY TRIGGERED - TOGGLE WINDOW                          ║");
                logging::log("VISIBILITY", "╚════════════════════════════════════════════════════════════╝");

                let is_visible = script_kit_gpui::is_main_window_visible();
                logging::log("VISIBILITY", &format!("State: WINDOW_VISIBLE={}", is_visible));

                let app_entity_inner = app_entity_for_hotkey.clone();
                let window_inner = window_for_hotkey;

                if is_visible {
                    logging::log("VISIBILITY", "Decision: HIDE");
                    let _ = cx.update(move |cx: &mut gpui::App| {
                        hide_main_window_helper(app_entity_inner, cx);
                    });
                } else {
                    logging::log("VISIBILITY", "Decision: SHOW");
                    let _ = cx.update(move |cx: &mut gpui::App| {
                        show_main_window_helper(window_inner, app_entity_inner, cx);
                    });
                }
            }
            logging::log("HOTKEY", "Main hotkey listener exiting");
        }).detach();

        // Notes hotkey listener - event-driven via async_channel
        // The hotkey thread dispatches via GPUI's ForegroundExecutor, which wakes this task
        // This works even before main window activates because the executor is initialized first
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            logging::log("HOTKEY", "Notes hotkey listener started (event-driven)");
            // Event-driven: .recv().await blocks until a message arrives
            // This is more efficient than polling and responds immediately
            while let Ok(()) = hotkeys::notes_hotkey_channel().1.recv().await {
                logging::log("HOTKEY", "Notes hotkey triggered - opening notes window");
                let _ = cx.update(|cx: &mut gpui::App| {
                    if let Err(e) = notes::open_notes_window(cx) {
                        logging::log("HOTKEY", &format!("Failed to open notes window: {}", e));
                    }
                });
            }
            logging::log("HOTKEY", "Notes hotkey listener exiting (channel closed)");
        }).detach();

        // AI hotkey listener - event-driven via async_channel
        // Same pattern as Notes hotkey - works immediately on app launch
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            logging::log("HOTKEY", "AI hotkey listener started (event-driven)");
            // Event-driven: .recv().await blocks until a message arrives
            while let Ok(()) = hotkeys::ai_hotkey_channel().1.recv().await {
                logging::log("HOTKEY", "AI hotkey triggered - opening AI window");
                let _ = cx.update(|cx: &mut gpui::App| {
                    if let Err(e) = ai::open_ai_window(cx) {
                        logging::log("HOTKEY", &format!("Failed to open AI window: {}", e));
                    }
                });
            }
            logging::log("HOTKEY", "AI hotkey listener exiting (channel closed)");
        }).detach();

        // Script/Scriptlet/App hotkey listener - event-driven via async_channel
        // Handles shortcuts from shortcuts.json for scriptlets, builtins, and apps
        let app_entity_for_scripts = app_entity.clone();
        let window_for_scripts = window;
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            logging::log("HOTKEY", "Script shortcut listener started (event-driven)");
            while let Ok(command_id) = hotkeys::script_hotkey_channel().1.recv().await {
                logging::log(
                    "HOTKEY",
                    &format!("Script shortcut received in main.rs: {}", command_id),
                );

                let id_clone = command_id.clone();
                let app_entity_inner = app_entity_for_scripts.clone();
                let window_inner = window_for_scripts;

                let _ = cx.update(move |cx: &mut gpui::App| {
                    logging::log(
                        "HOTKEY",
                        &format!("Executing command_id: {}", id_clone),
                    );

                    // Use app_entity.update to access ScriptListApp directly
                    // Returns whether main window should be shown (apps/certain builtins don't need it)
                    let should_show_window = app_entity_inner.update(cx, |view, ctx| {
                        logging::log(
                            "HOTKEY",
                            "Inside app_entity update, calling execute_by_command_id_or_path",
                        );
                        view.execute_by_command_id_or_path(&id_clone, ctx)
                    });

                    // Only show window if command needs it AND it's currently hidden
                    if should_show_window && !script_kit_gpui::is_main_window_visible() {
                        logging::log("HOTKEY", "Command needs main window, showing it");
                        show_main_window_helper(window_inner, app_entity_inner.clone(), cx);
                    } else if !should_show_window {
                        logging::log("HOTKEY", "Command doesn't need main window (app/ai/notes)");
                    }
                });
            }
            logging::log("HOTKEY", "Script shortcut listener exiting (channel closed)");
        }).detach();

        // Deeplink listener - handles scriptkit:// URLs (same logic as hotkeys)
        let app_entity_for_deeplinks = app_entity.clone();
        let window_for_deeplinks = window;
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            logging::log("DEEPLINK", "Deeplink listener started (event-driven)");
            while let Ok(command_id) = deeplink_channel().1.recv().await {
                logging::log(
                    "DEEPLINK",
                    &format!("Processing deeplink command: {}", command_id),
                );

                // Handle special deeplink types
                if command_id.starts_with("notes/") {
                    // Notes deeplink - open notes window
                    // Note: Currently opens notes window; specific note navigation can be added later
                    let note_id = command_id.strip_prefix("notes/").unwrap_or("");
                    logging::log("DEEPLINK", &format!("Opening notes (note_id: {})", note_id));
                    let _ = cx.update(|cx| {
                        if let Err(e) = notes::open_notes_window(cx) {
                            logging::log("DEEPLINK", &format!("Failed to open notes: {}", e));
                        }
                    });
                    continue;
                }

                let id_clone = command_id.clone();
                let app_entity_inner = app_entity_for_deeplinks.clone();
                let window_inner = window_for_deeplinks;

                let _ = cx.update(move |cx: &mut gpui::App| {
                    logging::log(
                        "DEEPLINK",
                        &format!("Executing command_id: {}", id_clone),
                    );

                    // Use app_entity.update to access ScriptListApp directly
                    // Returns whether main window should be shown (apps/certain builtins don't need it)
                    let should_show_window = app_entity_inner.update(cx, |view, ctx| {
                        logging::log(
                            "DEEPLINK",
                            "Inside app_entity update, calling execute_by_command_id_or_path",
                        );
                        view.execute_by_command_id_or_path(&id_clone, ctx)
                    });

                    // Only show window if command needs it AND it's currently hidden
                    if should_show_window && !script_kit_gpui::is_main_window_visible() {
                        logging::log("DEEPLINK", "Command needs main window, showing it");
                        show_main_window_helper(window_inner, app_entity_inner.clone(), cx);
                    } else if !should_show_window {
                        logging::log("DEEPLINK", "Command doesn't need main window (app/ai/notes)");
                    }
                });
            }
            logging::log("DEEPLINK", "Deeplink listener exiting (channel closed)");
        }).detach();

        // Appearance change watcher - event-driven with async_channel
        // Only spawn if watcher started successfully
        if appearance_watcher_ok {
            let app_entity_for_appearance = app_entity.clone();
            cx.spawn(async move |cx: &mut gpui::AsyncApp| {
                // Event-driven: blocks until appearance change event received
                while let Ok(_event) = appearance_rx.recv().await {
                    logging::log("APP", "System appearance changed, updating theme");
                    let _ = cx.update(|cx| {
                        // Sync gpui-component theme with new system appearance
                        theme::sync_gpui_component_theme(cx);

                        app_entity_for_appearance.update(cx, |view, ctx| {
                            view.update_theme(ctx);
                        });
                    });
                }
                logging::log("APP", "Appearance watcher channel closed");
            }).detach();
        }

        // Config reload watcher - watches ~/.scriptkit/kit/config.ts for changes
        // Only spawn if watcher started successfully
        if config_watcher_ok {
            let app_entity_for_config = app_entity.clone();
            cx.spawn(async move |cx: &mut gpui::AsyncApp| {
                loop {
                    Timer::after(std::time::Duration::from_millis(200)).await;

                    if config_rx.try_recv().is_ok() {
                        logging::log("APP", "Config file changed, reloading");
                        let _ = cx.update(|cx| {
                            app_entity_for_config.update(cx, |view, ctx| {
                                view.update_config(ctx);
                            });
                        });
                    }
                }
            }).detach();
        }

        // Script/scriptlets reload watcher - watches ~/.scriptkit/*/scripts/ and ~/.scriptkit/*/scriptlets/
        // Uses incremental updates for scriptlet files, full reload for scripts
        // Also re-scans for scheduled scripts to pick up new/modified schedules
        // Only spawn if watcher started successfully
        if script_watcher_ok {
            let app_entity_for_scripts = app_entity.clone();
            let scheduler_for_scripts = scheduler.clone();
            cx.spawn(async move |cx: &mut gpui::AsyncApp| {
                use watcher::ScriptReloadEvent;

                loop {
                    Timer::after(std::time::Duration::from_millis(200)).await;

                    // Drain all pending events
                    while let Ok(event) = script_rx.try_recv() {
                        match event {
                            ScriptReloadEvent::FileChanged(path) | ScriptReloadEvent::FileCreated(path) => {
                                // Check if it's a scriptlet file (markdown in scriptlets directory)
                                let is_scriptlet = path.extension().map(|e| e == "md").unwrap_or(false);

                                if is_scriptlet {
                                    logging::log("APP", &format!("Scriptlet file changed: {}", path.display()));
                                    let path_clone = path.clone();
                                    let _ = cx.update(|cx| {
                                        app_entity_for_scripts.update(cx, |view, ctx| {
                                            view.handle_scriptlet_file_change(&path_clone, false, ctx);
                                        });
                                    });
                                } else {
                                    logging::log("APP", &format!("Script file changed: {}", path.display()));
                                    // Re-scan for scheduled scripts when script files change
                                    if let Ok(scheduler_guard) = scheduler_for_scripts.lock() {
                                        let new_count = scripts::register_scheduled_scripts(&scheduler_guard);
                                        if new_count > 0 {
                                            logging::log("APP", &format!("Re-registered {} scheduled scripts after file change", new_count));
                                        }
                                    }
                                    let _ = cx.update(|cx| {
                                        app_entity_for_scripts.update(cx, |view, ctx| {
                                            view.refresh_scripts(ctx);
                                        });
                                    });
                                }
                            }
                            ScriptReloadEvent::FileDeleted(path) => {
                                let is_scriptlet = path.extension().map(|e| e == "md").unwrap_or(false);

                                if is_scriptlet {
                                    logging::log("APP", &format!("Scriptlet file deleted: {}", path.display()));
                                    let path_clone = path.clone();
                                    let _ = cx.update(|cx| {
                                        app_entity_for_scripts.update(cx, |view, ctx| {
                                            view.handle_scriptlet_file_change(&path_clone, true, ctx);
                                        });
                                    });
                                } else {
                                    logging::log("APP", &format!("Script file deleted: {}", path.display()));
                                    let _ = cx.update(|cx| {
                                        app_entity_for_scripts.update(cx, |view, ctx| {
                                            view.refresh_scripts(ctx);
                                        });
                                    });
                                }
                            }
                            ScriptReloadEvent::FullReload => {
                                logging::log("APP", "Full script/scriptlet reload requested");
                                // Re-scan for scheduled scripts
                                if let Ok(scheduler_guard) = scheduler_for_scripts.lock() {
                                    let new_count = scripts::register_scheduled_scripts(&scheduler_guard);
                                    if new_count > 0 {
                                        logging::log("APP", &format!("Re-registered {} scheduled scripts after full reload", new_count));
                                    }
                                }
                                let _ = cx.update(|cx| {
                                    app_entity_for_scripts.update(cx, |view, ctx| {
                                        view.refresh_scripts(ctx);
                                    });
                                });
                            }
                        }
                    }
                }
            }).detach();
        }

        // App watcher poll loop - watches /Applications and ~/Applications for changes
        // When apps are added/removed/updated, refresh the app launcher cache
        // Only spawn if watcher started successfully
        if app_watcher_ok {
            let app_entity_for_apps = app_entity.clone();
            cx.spawn(async move |cx: &mut gpui::AsyncApp| {
                use watcher::AppReloadEvent;

                logging::log("APP", "App watcher poll loop started (event-driven)");

                // Event-driven: blocks until app change event received
                while let Ok(event) = app_rx.recv().await {
                    logging::log("APP", &format!("App watcher event: {:?}", event));

                    match event {
                        AppReloadEvent::AppAdded(path) => {
                            logging::log("APP", &format!("App added: {}", path.display()));
                        }
                        AppReloadEvent::AppRemoved(path) => {
                            logging::log("APP", &format!("App removed: {}", path.display()));
                        }
                        AppReloadEvent::AppUpdated(path) => {
                            logging::log("APP", &format!("App updated: {}", path.display()));
                        }
                        AppReloadEvent::FullReload => {
                            logging::log("APP", "Full app reload requested");
                        }
                    }

                    // Trigger cache refresh (scan_applications updates the in-memory cache)
                    let _ = app_launcher::scan_applications();

                    // Notify UI to re-fetch cached apps and invalidate search caches
                    // This ensures new apps appear in search results immediately
                    let _ = cx.update(|cx| {
                        app_entity_for_apps.update(cx, |view, ctx| {
                            view.refresh_apps(ctx);
                        });
                    });
                }

                logging::log("APP", "App watcher poll loop exiting (channel closed)");
            }).detach();
        }

        // NOTE: Prompt message listener is now spawned per-script in execute_interactive()
        // using event-driven async_channel instead of 50ms polling

        // Scheduler event handler - runs scripts when their cron schedule triggers
        // Uses std::sync::mpsc::Receiver which requires a polling approach
        let _window_for_scheduler = window;
        std::thread::spawn(move || {
            logging::log("APP", "Scheduler event handler started");

            loop {
                // Check shutdown flag - exit loop if shutting down
                if SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
                    logging::log("SCHEDULER", "Shutdown requested, exiting scheduler event handler");
                    break;
                }

                // Use recv_timeout to periodically check for events without blocking forever
                match scheduler_rx.recv_timeout(std::time::Duration::from_secs(1)) {
                    Ok(event) => {
                        match event {
                            scheduler::SchedulerEvent::RunScript(path) => {
                                // Check shutdown flag before spawning new scripts
                                if SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
                                    logging::log("SCHEDULER", &format!("Skipping scheduled script (shutdown in progress): {}", path.display()));
                                    continue;
                                }

                                logging::log("SCHEDULER", &format!("Executing scheduled script: {}", path.display()));

                                // Execute the script using the existing executor infrastructure
                                // This spawns it in the background without blocking the scheduler
                                let path_str = path.to_string_lossy().to_string();

                                // Use bun to run the script directly (non-interactive for scheduled scripts)
                                // Find bun path (same logic as executor)
                                let bun_path = std::env::var("BUN_PATH")
                                    .ok()
                                    .or_else(|| {
                                        // Check common locations
                                        for candidate in &[
                                            "/opt/homebrew/bin/bun",
                                            "/usr/local/bin/bun",
                                            std::env::var("HOME").ok().map(|h| format!("{}/.bun/bin/bun", h)).unwrap_or_default().as_str(),
                                        ] {
                                            if std::path::Path::new(candidate).exists() {
                                                return Some(candidate.to_string());
                                            }
                                        }
                                        None
                                    })
                                    .unwrap_or_else(|| "bun".to_string());

                                // Spawn bun process to run the script
                                match std::process::Command::new(&bun_path)
                                    .arg("run")
                                    .arg("--preload")
                                    .arg(format!("{}/.scriptkit/sdk/kit-sdk.ts", std::env::var("HOME").unwrap_or_default()))
                                    .arg(&path_str)
                                    .stdout(std::process::Stdio::piped())
                                    .stderr(std::process::Stdio::piped())
                                    .spawn()
                                {
                                    Ok(child) => {
                                        let pid = child.id();
                                        // Track the process
                                        PROCESS_MANAGER.register_process(pid, &path_str);
                                        logging::log("SCHEDULER", &format!("Spawned scheduled script PID {}: {}", pid, path_str));

                                        // Wait for completion in a separate thread to not block scheduler
                                        let path_for_log = path_str.clone();
                                        std::thread::spawn(move || {
                                            match child.wait_with_output() {
                                                Ok(output) => {
                                                    // Unregister the process now that it's done
                                                    PROCESS_MANAGER.unregister_process(pid);

                                                    if output.status.success() {
                                                        logging::log("SCHEDULER", &format!("Scheduled script completed: {}", path_for_log));
                                                    } else {
                                                        let stderr = String::from_utf8_lossy(&output.stderr);
                                                        logging::log("SCHEDULER", &format!("Scheduled script failed: {} - {}", path_for_log, stderr));
                                                    }
                                                }
                                                Err(e) => {
                                                    // Unregister on error too
                                                    PROCESS_MANAGER.unregister_process(pid);
                                                    logging::log("SCHEDULER", &format!("Scheduled script error: {} - {}", path_for_log, e));
                                                }
                                            }
                                        });
                                    }
                                    Err(e) => {
                                        logging::log("SCHEDULER", &format!("Failed to spawn scheduled script: {} - {}", path_str, e));
                                    }
                                }
                            }
                            scheduler::SchedulerEvent::Error(msg) => {
                                logging::log("SCHEDULER", &format!("Scheduler error: {}", msg));
                            }
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        // Normal timeout, continue loop
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        logging::log("APP", "Scheduler event channel disconnected, exiting handler");
                        break;
                    }
                }
            }
        });

        // Test command file watcher - allows smoke tests to trigger script execution
        // SECURITY: This feature is ONLY enabled in debug builds to prevent local privilege escalation.
        // In release builds, any process that can write to /tmp could trigger script execution.
        #[cfg(debug_assertions)]
        {
            let app_entity_for_test = app_entity.clone();
            cx.spawn(async move |cx: &mut gpui::AsyncApp| {
                logging::log("TEST", "Debug command file watcher enabled (debug build only)");
                let cmd_file = std::path::PathBuf::from("/tmp/script-kit-gpui-cmd.txt");
                loop {
                    Timer::after(std::time::Duration::from_millis(500)).await;

                    if cmd_file.exists() {
                        if let Ok(content) = std::fs::read_to_string(&cmd_file) {
                            let _ = std::fs::remove_file(&cmd_file); // Remove immediately to prevent re-processing

                            for line in content.lines() {
                                if line.starts_with("run:") {
                                    let script_name = line.strip_prefix("run:").unwrap_or("").trim();
                                    logging::log("TEST", &format!("Test command: run script '{}'", script_name));

                                    let script_name_owned = script_name.to_string();
                                    let app_entity_inner = app_entity_for_test.clone();
                                    let _ = cx.update(|cx| {
                                        app_entity_inner.update(cx, |view, ctx| {
                                            // Find and run the script interactively
                                            if let Some(script) = view.scripts.iter().find(|s| s.name == script_name_owned || s.path.to_string_lossy().contains(&script_name_owned)).cloned() {
                                                logging::log("TEST", &format!("Found script: {}", script.name));
                                                view.execute_interactive(&script, ctx);
                                            } else {
                                                logging::log("TEST", &format!("Script not found: {}", script_name_owned));
                                            }
                                        });
                                    });
                                }
                            }
                        }
                    }
                }
            }).detach();
        }

        // External command listener - receives commands via stdin (event-driven, no polling)
        let stdin_rx = start_stdin_listener();
        let window_for_stdin = window;
        let app_entity_for_stdin = app_entity.clone();

        // Track if we've received any stdin commands (for timeout warning)
        static STDIN_RECEIVED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

        // Spawn a timeout warning task - helps AI agents detect when they forgot to use stdin protocol
        cx.spawn(async move |_cx: &mut gpui::AsyncApp| {
            Timer::after(std::time::Duration::from_secs(2)).await;
            if !STDIN_RECEIVED.load(std::sync::atomic::Ordering::SeqCst) {
                logging::log("STDIN", "");
                logging::log("STDIN", "╔════════════════════════════════════════════════════════════════════════════╗");
                logging::log("STDIN", "║  WARNING: No stdin JSON received after 2 seconds                          ║");
                logging::log("STDIN", "║                                                                            ║");
                logging::log("STDIN", "║  If you're testing, use the stdin JSON protocol:                          ║");
                logging::log("STDIN", "║  echo '{\"type\":\"run\",\"path\":\"...\"}' | ./target/debug/script-kit-gpui     ║");
                logging::log("STDIN", "║                                                                            ║");
                logging::log("STDIN", "║  Command line args do NOT work:                                           ║");
                logging::log("STDIN", "║  ./target/debug/script-kit-gpui test.ts  # WRONG - does nothing!          ║");
                logging::log("STDIN", "╚════════════════════════════════════════════════════════════════════════════╝");
                logging::log("STDIN", "");
            }
        }).detach();

        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            logging::log("STDIN", "Async stdin command handler started");

            // Event-driven: recv().await yields until a command arrives
            while let Ok(cmd) = stdin_rx.recv().await {
                // Mark that we've received stdin (clears the timeout warning)
                STDIN_RECEIVED.store(true, std::sync::atomic::Ordering::SeqCst);
                logging::log("STDIN", &format!("Processing external command: {:?}", cmd));

                let app_entity_inner = app_entity_for_stdin.clone();
                let _ = cx.update(|cx| {
                    // Use the Root window to get Window reference, then update the app entity
                    let _ = window_for_stdin.update(cx, |_root, window, root_cx| {
                        app_entity_inner.update(root_cx, |view, ctx| {
                            // Note: We have both `window` from Root and `view` from entity here
                            // ctx is Context<ScriptListApp>, window is &mut Window
                        match cmd {
                            ExternalCommand::Run { ref path, ref request_id } => {
                                let rid = request_id.as_deref().unwrap_or("-");
                                logging::log("STDIN", &format!("[{}] Executing script: {}", rid, path));

                                // NOTE: This is a simplified show path for script execution.
                                // We show the window, then immediately run the script.
                                // The core logic matches show_main_window_helper().

                                script_kit_gpui::set_main_window_visible(true);
                                platform::ensure_move_to_active_space();

                                // Use Window::defer via window_ops to coalesce and defer window move.
                                // This avoids RefCell borrow conflicts from synchronous macOS window operations.
                                let window_size = gpui::size(px(750.), initial_window_height());
                                let bounds = platform::calculate_eye_line_bounds_on_mouse_display(window_size);
                                window_ops::queue_move(bounds, window, ctx);

                                if !PANEL_CONFIGURED.load(std::sync::atomic::Ordering::SeqCst) {
                                    platform::configure_as_floating_panel();
                                    platform::swizzle_gpui_blurred_view();
                                    platform::configure_window_vibrancy_material();
                                    PANEL_CONFIGURED.store(true, std::sync::atomic::Ordering::SeqCst);
                                }

                                ctx.activate(true);
                                window.activate_window();
                                let focus_handle = view.focus_handle(ctx);
                                window.focus(&focus_handle, ctx);

                                // Send RunScript message to be handled
                                view.handle_prompt_message(PromptMessage::RunScript { path: path.clone() }, ctx);
                            }
                            ExternalCommand::Show { ref request_id } => {
                                let rid = request_id.as_deref().unwrap_or("-");
                                logging::log("STDIN", &format!("[{}] Showing window", rid));

                                // NOTE: This is a simplified show path for explicit stdin commands.
                                // Unlike the hotkey handler, we don't need NEEDS_RESET handling
                                // because this is an explicit show (not a toggle).
                                // The core logic matches show_main_window_helper().

                                script_kit_gpui::set_main_window_visible(true);
                                platform::ensure_move_to_active_space();

                                // Use Window::defer via window_ops to coalesce and defer window move.
                                // This avoids RefCell borrow conflicts from synchronous macOS window operations.
                                let window_size = gpui::size(px(750.), initial_window_height());
                                let bounds = platform::calculate_eye_line_bounds_on_mouse_display(window_size);
                                window_ops::queue_move(bounds, window, ctx);

                                if !PANEL_CONFIGURED.load(std::sync::atomic::Ordering::SeqCst) {
                                    platform::configure_as_floating_panel();
                                    platform::swizzle_gpui_blurred_view();
                                    platform::configure_window_vibrancy_material();
                                    PANEL_CONFIGURED.store(true, std::sync::atomic::Ordering::SeqCst);
                                }

                                ctx.activate(true);
                                window.activate_window();
                                let focus_handle = view.focus_handle(ctx);
                                window.focus(&focus_handle, ctx);
                            }
                            ExternalCommand::Hide { ref request_id } => {
                                let rid = request_id.as_deref().unwrap_or("-");
                                logging::log("STDIN", &format!("[{}] Hiding main window", rid));
                                script_kit_gpui::set_main_window_visible(false);

                                // Check if Notes or AI windows are open
                                let notes_open = notes::is_notes_window_open();
                                let ai_open = ai::is_ai_window_open();

                                // CRITICAL: Only hide main window if Notes/AI are open
                                // ctx.hide() hides the ENTIRE app (all windows)
                                if notes_open || ai_open {
                                    logging::log("STDIN", "Using hide_main_window() - secondary windows are open");
                                    platform::hide_main_window();
                                } else {
                                    ctx.hide();
                                }
                            }
                            ExternalCommand::SetFilter { ref text, ref request_id } => {
                                let rid = request_id.as_deref().unwrap_or("-");
                                logging::log("STDIN", &format!("[{}] Setting filter to: '{}'", rid, text));
                                view.set_filter_text_immediate(text.clone(), window, ctx);
                                let _ = view.get_filtered_results_cached(); // Update cache
                            }
                            ExternalCommand::TriggerBuiltin { ref name } => {
                                logging::log("STDIN", &format!("Triggering built-in: '{}'", name));
                                // Opened via protocol command - ESC should close window (not return to main menu)
                                view.opened_from_main_menu = false;
                                // Match built-in name and trigger the corresponding feature
                                match name.to_lowercase().as_str() {
                                    "design-gallery" | "designgallery" | "design gallery" => {
                                        view.current_view = AppView::DesignGalleryView {
                                            filter: String::new(),
                                            selected_index: 0,
                                        };
                                        view.update_window_size_deferred(window, ctx);
                                    }
                                    // P0 FIX: Store data in self, view holds only state
                                    "clipboard" | "clipboard-history" | "clipboardhistory" => {
                                        view.cached_clipboard_entries =
                                            clipboard_history::get_cached_entries(100);
                                        view.current_view = AppView::ClipboardHistoryView {
                                            filter: String::new(),
                                            selected_index: 0,
                                        };
                                        view.update_window_size_deferred(window, ctx);
                                    }
                                    // P0 FIX: Use existing self.apps, view holds only state
                                    "apps" | "app-launcher" | "applauncher" => {
                                        view.current_view = AppView::AppLauncherView {
                                            filter: String::new(),
                                            selected_index: 0,
                                        };
                                        view.update_window_size_deferred(window, ctx);
                                    }
                                    "file-search" | "filesearch" | "files" | "searchfiles" => {
                                        view.open_file_search(String::new(), ctx);
                                    }
                                    _ => {
                                        logging::log("ERROR", &format!("Unknown built-in: '{}'", name));
                                    }
                                }
                            }
                            ExternalCommand::SimulateKey { ref key, ref modifiers } => {
                                logging::log("STDIN", &format!("Simulating key: '{}' with modifiers: {:?}", key, modifiers));

                                // Parse modifiers
                                let has_cmd = modifiers.iter().any(|m| m == "cmd" || m == "meta" || m == "command");
                                let has_shift = modifiers.iter().any(|m| m == "shift");
                                let _has_alt = modifiers.iter().any(|m| m == "alt" || m == "option");
                                let _has_ctrl = modifiers.iter().any(|m| m == "ctrl" || m == "control");

                                // Handle key based on current view
                                let key_lower = key.to_lowercase();

                                match &view.current_view {
                                    AppView::ScriptList => {
                                        // Main script list key handling
                                        if has_cmd && key_lower == "k" {
                                            logging::log("STDIN", "SimulateKey: Cmd+K - toggle actions");
                                            view.toggle_actions(ctx, window);
                                        } else if view.fallback_mode && !view.cached_fallbacks.is_empty() {
                                            // Handle keys in fallback mode
                                            match key_lower.as_str() {
                                                "up" | "arrowup" => {
                                                    if view.fallback_selected_index > 0 {
                                                        view.fallback_selected_index -= 1;
                                                        ctx.notify();
                                                    }
                                                }
                                                "down" | "arrowdown" => {
                                                    if view.fallback_selected_index < view.cached_fallbacks.len().saturating_sub(1) {
                                                        view.fallback_selected_index += 1;
                                                        ctx.notify();
                                                    }
                                                }
                                                "enter" => {
                                                    logging::log("STDIN", "SimulateKey: Enter - execute fallback");
                                                    view.execute_selected_fallback(ctx);
                                                }
                                                "escape" => {
                                                    logging::log("STDIN", "SimulateKey: Escape - clear filter (exit fallback mode)");
                                                    view.clear_filter(window, ctx);
                                                }
                                                _ => {
                                                    logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in fallback mode", key_lower));
                                                }
                                            }
                                        } else {
                                            match key_lower.as_str() {
                                                "up" | "arrowup" => {
                                                    // Use move_selection_up to properly skip section headers
                                                    view.move_selection_up(ctx);
                                                }
                                                "down" | "arrowdown" => {
                                                    // Use move_selection_down to properly skip section headers
                                                    view.move_selection_down(ctx);
                                                }
                                                "enter" => {
                                                    logging::log("STDIN", "SimulateKey: Enter - execute selected");
                                                    view.execute_selected(ctx);
                                                }
                                                "escape" => {
                                                    logging::log("STDIN", "SimulateKey: Escape - clear filter or hide");
                                                    if !view.filter_text.is_empty() {
                                                        view.clear_filter(window, ctx);
                                                    } else {
                                                        script_kit_gpui::set_main_window_visible(false);
                                                        ctx.hide();
                                                    }
                                                }
                                                _ => {
                                                    logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in ScriptList", key_lower));
                                                }
                                            }
                                        }
                                    }
                                    AppView::PathPrompt { entity, .. } => {
                                        // Path prompt key handling
                                        logging::log("STDIN", &format!("SimulateKey: Dispatching '{}' to PathPrompt", key_lower));
                                        let entity_clone = entity.clone();
                                        entity_clone.update(ctx, |path_prompt: &mut PathPrompt, path_cx| {
                                            if has_cmd && key_lower == "k" {
                                                path_prompt.toggle_actions(path_cx);
                                            } else {
                                                match key_lower.as_str() {
                                                    "up" | "arrowup" => path_prompt.move_up(path_cx),
                                                    "down" | "arrowdown" => path_prompt.move_down(path_cx),
                                                    "enter" => path_prompt.handle_enter(path_cx),
                                                    "escape" => path_prompt.submit_cancel(),
                                                    "left" | "arrowleft" => path_prompt.navigate_to_parent(path_cx),
                                                    "right" | "arrowright" => path_prompt.navigate_into_selected(path_cx),
                                                    _ => {
                                                        logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in PathPrompt", key_lower));
                                                    }
                                                }
                                            }
                                        });
                                    }
                                    AppView::ArgPrompt { id, .. } => {
                                        // Arg prompt key handling via SimulateKey
                                        logging::log("STDIN", &format!("SimulateKey: Dispatching '{}' to ArgPrompt (actions_popup={})", key_lower, view.show_actions_popup));

                                        // Check for Cmd+K to toggle actions popup
                                        if has_cmd && key_lower == "k" {
                                            logging::log("STDIN", "SimulateKey: Cmd+K - toggle arg actions");
                                            view.toggle_arg_actions(ctx, window);
                                        } else if view.show_actions_popup {
                                            // If actions popup is open, route to it
                                            if let Some(ref dialog) = view.actions_dialog {
                                                match key_lower.as_str() {
                                                    "up" | "arrowup" => {
                                                        logging::log("STDIN", "SimulateKey: Up in actions dialog");
                                                        dialog.update(ctx, |d, cx| d.move_up(cx));
                                                    }
                                                    "down" | "arrowdown" => {
                                                        logging::log("STDIN", "SimulateKey: Down in actions dialog");
                                                        dialog.update(ctx, |d, cx| d.move_down(cx));
                                                    }
                                                    "enter" => {
                                                        logging::log("STDIN", "SimulateKey: Enter in actions dialog");
                                                        let action_id = dialog.read(ctx).get_selected_action_id();
                                                        let should_close = dialog.read(ctx).selected_action_should_close();
                                                        if let Some(action_id) = action_id {
                                                            logging::log("ACTIONS", &format!("SimulateKey: Executing action: {} (close={})", action_id, should_close));
                                                            if should_close {
                                                                view.show_actions_popup = false;
                                                                view.actions_dialog = None;
                                                                view.focused_input = FocusedInput::ArgPrompt;
                                                                window.focus(&view.focus_handle, ctx);
                                                            }
                                                            view.trigger_action_by_name(&action_id, ctx);
                                                        }
                                                    }
                                                    "escape" => {
                                                        logging::log("STDIN", "SimulateKey: Escape - close actions dialog");
                                                        view.show_actions_popup = false;
                                                        view.actions_dialog = None;
                                                        view.focused_input = FocusedInput::ArgPrompt;
                                                        window.focus(&view.focus_handle, ctx);
                                                    }
                                                    _ => {
                                                        logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in ArgPrompt actions dialog", key_lower));
                                                    }
                                                }
                                            }
                                        } else {
                                            // Normal arg prompt key handling
                                            let prompt_id = id.clone();
                                            match key_lower.as_str() {
                                                "up" | "arrowup" => {
                                                    if view.arg_selected_index > 0 {
                                                        view.arg_selected_index -= 1;
                                                        view.arg_list_scroll_handle.scroll_to_item(view.arg_selected_index, ScrollStrategy::Nearest);
                                                        logging::log("STDIN", &format!("SimulateKey: Arg up, index={}", view.arg_selected_index));
                                                    }
                                                }
                                                "down" | "arrowdown" => {
                                                    let filtered = view.filtered_arg_choices();
                                                    if view.arg_selected_index < filtered.len().saturating_sub(1) {
                                                        view.arg_selected_index += 1;
                                                        view.arg_list_scroll_handle.scroll_to_item(view.arg_selected_index, ScrollStrategy::Nearest);
                                                        logging::log("STDIN", &format!("SimulateKey: Arg down, index={}", view.arg_selected_index));
                                                    }
                                                }
                                                "enter" => {
                                                    logging::log("STDIN", "SimulateKey: Enter - submit selection");
                                                    let filtered = view.filtered_arg_choices();
                                                    if let Some((_, choice)) = filtered.get(view.arg_selected_index) {
                                                        let value = choice.value.clone();
                                                        view.submit_prompt_response(prompt_id, Some(value), ctx);
                                                    } else if !view.arg_input.is_empty() {
                                                        let value = view.arg_input.text().to_string();
                                                        view.submit_prompt_response(prompt_id, Some(value), ctx);
                                                    }
                                                }
                                                "escape" => {
                                                    logging::log("STDIN", "SimulateKey: Escape - cancel script");
                                                    view.submit_prompt_response(prompt_id, None, ctx);
                                                    view.cancel_script_execution(ctx);
                                                }
                                                _ => {
                                                    logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in ArgPrompt", key_lower));
                                                }
                                            }
                                        }
                                    }
                                    AppView::EditorPrompt { entity, id, .. } => {
                                        // Editor prompt key handling for template/snippet navigation and choice popup
                                        logging::log("STDIN", &format!("SimulateKey: Dispatching '{}' to EditorPrompt", key_lower));
                                        let entity_clone = entity.clone();
                                        let prompt_id_clone = id.clone();

                                        // Check if choice popup is visible
                                        let has_choice_popup = entity_clone.update(ctx, |editor: &mut EditorPrompt, _| {
                                            editor.is_choice_popup_visible()
                                        });

                                        if has_choice_popup {
                                            // Handle choice popup navigation
                                            match key_lower.as_str() {
                                                "up" | "arrowup" => {
                                                    logging::log("STDIN", "SimulateKey: Up in choice popup");
                                                    entity_clone.update(ctx, |editor, cx| {
                                                        editor.choice_popup_up_public(cx);
                                                    });
                                                }
                                                "down" | "arrowdown" => {
                                                    logging::log("STDIN", "SimulateKey: Down in choice popup");
                                                    entity_clone.update(ctx, |editor, cx| {
                                                        editor.choice_popup_down_public(cx);
                                                    });
                                                }
                                                "enter" if !has_cmd => {
                                                    logging::log("STDIN", "SimulateKey: Enter in choice popup - confirming");
                                                    entity_clone.update(ctx, |editor, cx| {
                                                        editor.choice_popup_confirm_public(window, cx);
                                                    });
                                                }
                                                "escape" => {
                                                    logging::log("STDIN", "SimulateKey: Escape in choice popup - cancelling");
                                                    entity_clone.update(ctx, |editor, cx| {
                                                        editor.choice_popup_cancel_public(cx);
                                                    });
                                                }
                                                "tab" if !has_shift => {
                                                    logging::log("STDIN", "SimulateKey: Tab in choice popup - confirm and next");
                                                    entity_clone.update(ctx, |editor, cx| {
                                                        editor.choice_popup_confirm_public(window, cx);
                                                        editor.next_tabstop_public(window, cx);
                                                    });
                                                }
                                                _ => {
                                                    logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in choice popup", key_lower));
                                                }
                                            }
                                        } else if key_lower == "tab" && !has_cmd {
                                            // Handle Tab key for snippet navigation
                                            entity_clone.update(ctx, |editor: &mut EditorPrompt, editor_cx| {
                                                logging::log("STDIN", "SimulateKey: Tab in EditorPrompt - calling next_tabstop");
                                                if editor.in_snippet_mode() {
                                                    editor.next_tabstop_public(window, editor_cx);
                                                } else {
                                                    logging::log("STDIN", "SimulateKey: Tab - not in snippet mode");
                                                }
                                            });
                                        } else if key_lower == "enter" && has_cmd {
                                            // Cmd+Enter submits - get content from editor
                                            logging::log("STDIN", "SimulateKey: Cmd+Enter in EditorPrompt - submitting");
                                            let content = entity_clone.update(ctx, |editor, editor_cx| {
                                                editor.content(editor_cx)
                                            });
                                            view.submit_prompt_response(prompt_id_clone.clone(), Some(content), ctx);
                                        } else if key_lower == "escape" && !has_cmd {
                                            logging::log("STDIN", "SimulateKey: Escape in EditorPrompt - cancelling");
                                            view.submit_prompt_response(prompt_id_clone.clone(), None, ctx);
                                            view.cancel_script_execution(ctx);
                                        } else {
                                            logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in EditorPrompt", key_lower));
                                        }
                                    }
                                    _ => {
                                        logging::log("STDIN", &format!("SimulateKey: View {:?} not supported for key simulation", std::mem::discriminant(&view.current_view)));
                                    }
                                }
                            }
                            ExternalCommand::OpenNotes => {
                                logging::log("STDIN", "Opening notes window via stdin command");
                                if let Err(e) = notes::open_notes_window(ctx) {
                                    logging::log("STDIN", &format!("Failed to open notes window: {}", e));
                                }
                            }
                            ExternalCommand::OpenAi => {
                                logging::log("STDIN", "Opening AI window via stdin command");
                                if let Err(e) = ai::open_ai_window(ctx) {
                                    logging::log("STDIN", &format!("Failed to open AI window: {}", e));
                                }
                            }
                            ExternalCommand::OpenAiWithMockData => {
                                logging::log("STDIN", "Opening AI window with mock data via stdin command");
                                // First insert mock data
                                if let Err(e) = ai::insert_mock_data() {
                                    logging::log("STDIN", &format!("Failed to insert mock data: {}", e));
                                } else {
                                    logging::log("STDIN", "Mock data inserted successfully");
                                }
                                // Then open the window
                                if let Err(e) = ai::open_ai_window(ctx) {
                                    logging::log("STDIN", &format!("Failed to open AI window: {}", e));
                                }
                            }
                            ExternalCommand::CaptureWindow { title, path } => {
                                logging::log("STDIN", &format!("Capturing window with title '{}' to '{}'", title, path));
                                match capture_window_by_title(&title, false) {
                                    Ok((png_data, width, height)) => {
                                        // Save to file
                                        if let Err(e) = std::fs::write(&path, &png_data) {
                                            logging::log("STDIN", &format!("Failed to write screenshot: {}", e));
                                        } else {
                                            logging::log("STDIN", &format!("Screenshot saved: {} ({}x{})", path, width, height));
                                        }
                                    }
                                    Err(e) => {
                                        logging::log("STDIN", &format!("Failed to capture window: {}", e));
                                    }
                                }
                            }
                            ExternalCommand::SetAiSearch { text } => {
                                logging::log("STDIN", &format!("Setting AI search filter to: {}", text));
                                ai::set_ai_search(ctx, &text);
                            }
                            ExternalCommand::SetAiInput { text, submit } => {
                                logging::log("STDIN", &format!("Setting AI input to: {} (submit={})", text, submit));
                                ai::set_ai_input(ctx, &text, submit);
                            }
                            ExternalCommand::ShowGrid { grid_size, show_bounds, show_box_model, show_alignment_guides, show_dimensions, ref depth } => {
                                logging::log("STDIN", &format!(
                                    "ShowGrid: size={}, bounds={}, box_model={}, guides={}, dimensions={}, depth={:?}",
                                    grid_size, show_bounds, show_box_model, show_alignment_guides, show_dimensions, depth
                                ));
                                let options = protocol::GridOptions {
                                    grid_size,
                                    show_bounds,
                                    show_box_model,
                                    show_alignment_guides,
                                    show_dimensions,
                                    depth: depth.clone(),
                                    color_scheme: None,
                                };
                                view.show_grid(options, ctx);
                            }
                            ExternalCommand::HideGrid => {
                                logging::log("STDIN", "HideGrid: hiding debug grid overlay");
                                view.hide_grid(ctx);
                            }
                            ExternalCommand::ExecuteFallback { ref fallback_id, ref input } => {
                                logging::log("STDIN", &format!("ExecuteFallback: id='{}', input='{}'", fallback_id, input));
                                execute_fallback_action(view, fallback_id, input, window, ctx);
                            }
                            ExternalCommand::ShowShortcutRecorder { ref command_id, ref command_name } => {
                                logging::log("STDIN", &format!("ShowShortcutRecorder: command_id='{}', command_name='{}'", command_id, command_name));
                                view.show_shortcut_recorder(command_id.clone(), command_name.clone(), ctx);
                            }
                        }
                        ctx.notify();
                        }); // close app_entity_inner.update
                    }); // close window_for_stdin.update
                }); // close cx.update
            }

            logging::log("STDIN", "Async stdin command handler exiting");
        }).detach();

        // Tray menu event handler - polls for menu events
        // Clone config for use in tray handler
        let config_for_tray = config::load_config();
        if let Some(tray_mgr) = tray_manager {
            let window_for_tray = window;
            let app_entity_for_tray = app_entity.clone();
            cx.spawn(async move |cx: &mut gpui::AsyncApp| {
                logging::log("TRAY", "Tray menu event handler started");

                loop {
                    // Poll for tray menu events every 100ms
                    Timer::after(std::time::Duration::from_millis(100)).await;

                    // Check for menu events
                    if let Ok(event) = tray_mgr.menu_event_receiver().try_recv() {
                        // Convert event to action using type-safe IDs (pure function)
                        let action = TrayManager::action_from_event(&event);

                        // Handle side effects for LaunchAtLogin before the match
                        if let Some(TrayMenuAction::LaunchAtLogin) = action {
                            if let Err(e) = tray_mgr.handle_action(TrayMenuAction::LaunchAtLogin) {
                                logging::log("TRAY", &format!("Failed to toggle login item: {}", e));
                            }
                        }

                        match action {
                            Some(TrayMenuAction::OpenScriptKit) => {
                                logging::log("TRAY", "Open Script Kit menu item clicked");
                                let window_inner = window_for_tray;
                                let app_entity_inner = app_entity_for_tray.clone();
                                let _ = cx.update(|cx| {
                                    show_main_window_helper(window_inner, app_entity_inner, cx);
                                });
                            }
                            Some(TrayMenuAction::OpenNotes) => {
                                logging::log("TRAY", "Notes menu item clicked");
                                let _ = cx.update(|cx| {
                                    if let Err(e) = notes::open_notes_window(cx) {
                                        logging::log(
                                            "TRAY",
                                            &format!("Failed to open notes window: {}", e),
                                        );
                                    }
                                });
                            }
                            Some(TrayMenuAction::OpenAiChat) => {
                                logging::log("TRAY", "AI Chat menu item clicked");
                                let _ = cx.update(|cx| {
                                    if let Err(e) = ai::open_ai_window(cx) {
                                        logging::log(
                                            "TRAY",
                                            &format!("Failed to open AI window: {}", e),
                                        );
                                    }
                                });
                            }
                            Some(TrayMenuAction::LaunchAtLogin) => {
                                // Side effects (toggle + checkbox update) handled above
                                logging::log("TRAY", "Launch at Login toggled");
                            }
                            Some(TrayMenuAction::Settings) => {
                                logging::log("TRAY", "Settings menu item clicked");
                                // Open config file in editor
                                let editor = config_for_tray.get_editor();
                                let config_path = shellexpand::tilde("~/.scriptkit/kit/config.ts").to_string();

                                logging::log("TRAY", &format!("Opening {} in editor '{}'", config_path, editor));
                                match std::process::Command::new(&editor)
                                    .arg(&config_path)
                                    .spawn()
                                {
                                    Ok(_) => logging::log("TRAY", &format!("Spawned editor: {}", editor)),
                                    Err(e) => logging::log("TRAY", &format!("Failed to spawn editor '{}': {}", editor, e)),
                                }
                            }
                            Some(TrayMenuAction::OpenOnGitHub) => {
                                logging::log("TRAY", "Open on GitHub menu item clicked");
                                let url = "https://github.com/script-kit/app";
                                if let Err(e) = open::that(url) {
                                    logging::log("TRAY", &format!("Failed to open GitHub URL: {}", e));
                                }
                            }
                            Some(TrayMenuAction::OpenManual) => {
                                logging::log("TRAY", "Manual menu item clicked");
                                let url = "https://scriptkit.com";
                                if let Err(e) = open::that(url) {
                                    logging::log("TRAY", &format!("Failed to open manual URL: {}", e));
                                }
                            }
                            Some(TrayMenuAction::JoinCommunity) => {
                                logging::log("TRAY", "Join Community menu item clicked");
                                let url = "https://discord.gg/qnUX4XqJQd";
                                if let Err(e) = open::that(url) {
                                    logging::log("TRAY", &format!("Failed to open Discord URL: {}", e));
                                }
                            }
                            Some(TrayMenuAction::FollowUs) => {
                                logging::log("TRAY", "Follow Us menu item clicked");
                                let url = "https://twitter.com/scriptkitapp";
                                if let Err(e) = open::that(url) {
                                    logging::log("TRAY", &format!("Failed to open Twitter URL: {}", e));
                                }
                            }
                            Some(TrayMenuAction::Quit) => {
                                logging::log("TRAY", "Quit menu item clicked");
                                // Set shutdown flag FIRST - prevents new script spawns
                                // and triggers the shutdown monitor task for unified cleanup
                                SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);

                                // Clean up processes and PID file before quitting
                                PROCESS_MANAGER.kill_all_processes();
                                PROCESS_MANAGER.remove_main_pid();
                                let _ = cx.update(|cx| {
                                    cx.quit();
                                });
                                break; // Exit the polling loop
                            }
                            None => {
                                logging::log("TRAY", "Unknown menu event received");
                            }
                        }
                    }
                }

                logging::log("TRAY", "Tray menu event handler exiting");
            }).detach();
        }

        // Shutdown monitor task - checks SHUTDOWN_REQUESTED flag set by signal handler
        // Performs all cleanup on the main thread where it's safe to call logging,
        // mutexes, and other non-async-signal-safe functions.
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            loop {
                // Check every 100ms for shutdown signal
                Timer::after(std::time::Duration::from_millis(100)).await;

                if SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
                    logging::log("SHUTDOWN", "Shutdown signal detected, performing graceful cleanup");

                    // Kill all tracked child processes
                    logging::log("SHUTDOWN", "Killing all child processes");
                    PROCESS_MANAGER.kill_all_processes();

                    // Remove main PID file
                    PROCESS_MANAGER.remove_main_pid();

                    logging::log("SHUTDOWN", "Cleanup complete, quitting application");

                    // Quit the GPUI application
                    let _ = cx.update(|cx| {
                        cx.quit();
                    });

                    break;
                }
            }
        }).detach();

        logging::log("APP", "Application ready - Cmd+; to show, Esc to hide, Cmd+K for actions");
    });
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
