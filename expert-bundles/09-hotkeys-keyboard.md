🧩 Packing 3 file(s)...
📝 Files selected:
  • src/keyboard_monitor.rs
  • src/hotkey_pollers.rs
  • src/hotkeys.rs
This file is a merged representation of the filtered codebase, combined into a single document by packx.

<file_summary>
This section contains a summary of this file.

<purpose>
This file contains a packed representation of filtered repository contents.
It is designed to be easily consumable by AI systems for analysis, code review,
or other automated processes.
</purpose>

<usage_guidelines>
- Treat this file as a snapshot of the repository's state
- Be aware that this file may contain sensitive information
</usage_guidelines>

<notes>
- Files were filtered by packx based on content and extension matching
- Total files included: 3
</notes>
</file_summary>

<directory_structure>
src/keyboard_monitor.rs
src/hotkey_pollers.rs
src/hotkeys.rs
</directory_structure>

<files>
This section contains the contents of the repository's files.

<file path="src/keyboard_monitor.rs">
//! Global keyboard monitoring using macOS CGEventTap API
//!
//! This module provides system-wide keyboard event capture, regardless of which
//! application has focus. This is essential for text expansion/snippet features
//! that need to detect trigger sequences typed in any application.
//!
//! # Requirements
//! - macOS only (uses Core Graphics CGEventTap)
//! - Requires Accessibility permissions to be enabled in System Preferences
//!
//! # Example
//! ```no_run
//! use script_kit_gpui::keyboard_monitor::{KeyboardMonitor, KeyEvent};
//!
//! let mut monitor = KeyboardMonitor::new(|event: KeyEvent| {
//!     println!("Key pressed: {:?}", event.character);
//! });
//!
//! monitor.start().expect("Failed to start keyboard monitor");
//! // ... monitor runs in background thread ...
//! monitor.stop();
//! ```

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use core_foundation::runloop::{kCFRunLoopCommonModes, kCFRunLoopDefaultMode, CFRunLoop};
use core_graphics::event::{
    CGEvent, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
    EventField,
};
use macos_accessibility_client::accessibility;
use thiserror::Error;
use tracing::{debug, error, info, warn};

/// Errors that can occur when using the keyboard monitor
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum KeyboardMonitorError {
    #[error("Accessibility permissions not granted. Please enable in System Preferences > Privacy & Security > Accessibility")]
    AccessibilityNotGranted,

    #[error("Failed to create event tap - this may indicate accessibility permissions issue")]
    EventTapCreationFailed,

    #[error("Failed to create run loop source from event tap")]
    RunLoopSourceCreationFailed,

    #[error("Monitor is already running")]
    AlreadyRunning,

    #[error("Monitor is not running")]
    NotRunning,

    #[error("Failed to start monitor thread")]
    ThreadSpawnFailed,
}

/// Represents a keyboard event captured by the monitor
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct KeyEvent {
    /// The character that was typed (if available)
    /// This is the actual character produced, taking into account modifiers
    pub character: Option<String>,

    /// The virtual key code (hardware key identifier)
    pub key_code: u16,

    /// Whether the shift modifier was held
    pub shift: bool,

    /// Whether the control modifier was held
    pub control: bool,

    /// Whether the option/alt modifier was held
    pub option: bool,

    /// Whether the command modifier was held
    pub command: bool,

    /// Whether this is an auto-repeat event
    pub is_repeat: bool,
}

/// Callback type for receiving keyboard events
/// Must be Send + Sync since it's shared across threads via Arc
pub type KeyEventCallback = Box<dyn Fn(KeyEvent) + Send + Sync + 'static>;

/// Global keyboard monitor using macOS CGEventTap
///
/// This monitor captures keystrokes system-wide, regardless of which application
/// has focus. It runs on a dedicated background thread with its own CFRunLoop.
pub struct KeyboardMonitor {
    /// Whether the monitor is currently running
    running: Arc<AtomicBool>,

    /// Handle to the background thread running the event loop
    thread_handle: Option<JoinHandle<()>>,

    /// The callback to invoke for each key event
    callback: Arc<KeyEventCallback>,

    /// Run loop reference for stopping (stored after start)
    run_loop: Arc<std::sync::Mutex<Option<CFRunLoop>>>,
}

impl KeyboardMonitor {
    /// Create a new keyboard monitor with the given callback
    ///
    /// The callback will be invoked for each key-down event captured.
    /// The monitor does not start automatically - call `start()` to begin monitoring.
    pub fn new<F>(callback: F) -> Self
    where
        F: Fn(KeyEvent) + Send + Sync + 'static,
    {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
            callback: Arc::new(Box::new(callback)),
            run_loop: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    /// Check if accessibility permissions are granted
    ///
    /// Returns true if the application has been granted accessibility permissions.
    /// If false, the monitor will fail to start.
    pub fn has_accessibility_permission() -> bool {
        accessibility::application_is_trusted()
    }

    /// Check if accessibility permissions are granted, prompting the user if not
    ///
    /// This will show the macOS accessibility permission dialog if permissions
    /// haven't been granted yet. Returns true if permissions are granted.
    #[allow(dead_code)]
    pub fn request_accessibility_permission() -> bool {
        accessibility::application_is_trusted_with_prompt()
    }

    /// Start the keyboard monitor
    ///
    /// This spawns a background thread that captures keyboard events system-wide.
    /// The provided callback will be invoked for each key-down event.
    ///
    /// # Errors
    /// - `AccessibilityNotGranted` - Accessibility permissions not enabled
    /// - `AlreadyRunning` - Monitor is already running
    /// - `EventTapCreationFailed` - Failed to create the event tap
    pub fn start(&mut self) -> Result<(), KeyboardMonitorError> {
        // Check if already running
        if self.running.load(Ordering::SeqCst) {
            return Err(KeyboardMonitorError::AlreadyRunning);
        }

        // Check accessibility permissions
        if !Self::has_accessibility_permission() {
            warn!("Accessibility permissions not granted for keyboard monitor");
            return Err(KeyboardMonitorError::AccessibilityNotGranted);
        }

        info!("Starting global keyboard monitor");

        let running = Arc::clone(&self.running);
        let callback = Arc::clone(&self.callback);
        let run_loop_storage = Arc::clone(&self.run_loop);

        // Set running flag before spawning thread
        self.running.store(true, Ordering::SeqCst);

        let handle = thread::Builder::new()
            .name("keyboard-monitor".to_string())
            .spawn(move || {
                Self::event_loop(running, callback, run_loop_storage);
            })
            .map_err(|e| {
                error!("Failed to spawn keyboard monitor thread: {}", e);
                self.running.store(false, Ordering::SeqCst);
                KeyboardMonitorError::ThreadSpawnFailed
            })?;

        self.thread_handle = Some(handle);
        Ok(())
    }

    /// Stop the keyboard monitor
    ///
    /// This stops the background thread and cleans up resources.
    /// Safe to call even if the monitor is not running.
    pub fn stop(&mut self) {
        if !self.running.load(Ordering::SeqCst) {
            debug!("Keyboard monitor already stopped");
            return;
        }

        info!("Stopping global keyboard monitor");

        // Signal the thread to stop
        self.running.store(false, Ordering::SeqCst);

        // Stop the run loop
        if let Ok(guard) = self.run_loop.lock() {
            if let Some(ref run_loop) = *guard {
                run_loop.stop();
            }
        }

        // Wait for thread to finish
        if let Some(handle) = self.thread_handle.take() {
            if let Err(e) = handle.join() {
                error!("Keyboard monitor thread panicked: {:?}", e);
            }
        }

        debug!("Keyboard monitor stopped");
    }

    /// Check if the monitor is currently running
    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// The main event loop that runs on the background thread
    fn event_loop(
        running: Arc<AtomicBool>,
        callback: Arc<KeyEventCallback>,
        run_loop_storage: Arc<std::sync::Mutex<Option<CFRunLoop>>>,
    ) {
        debug!("Keyboard monitor event loop starting");

        // Get current run loop and store it for stopping
        let current_run_loop = CFRunLoop::get_current();
        if let Ok(mut guard) = run_loop_storage.lock() {
            *guard = Some(current_run_loop.clone());
        }

        // Create event tap for key down events
        debug!("Creating CGEventTap with HID location for KeyDown events");
        let event_tap_result = CGEventTap::new(
            CGEventTapLocation::HID,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::ListenOnly, // We only observe, don't modify
            vec![CGEventType::KeyDown],
            move |_proxy, _event_type, event: &CGEvent| {
                debug!("CGEventTap callback invoked - key event received!");

                // Extract key event information
                let key_event = Self::extract_key_event(event);

                debug!(
                    character = ?key_event.character,
                    key_code = key_event.key_code,
                    "Extracted key event"
                );

                // Invoke callback
                callback(key_event);

                // Return None to not modify the event (we're just observing)
                None
            },
        );

        let event_tap = match event_tap_result {
            Ok(tap) => tap,
            Err(()) => {
                error!(
                    "Failed to create CGEventTap - accessibility permissions may not be granted"
                );
                running.store(false, Ordering::SeqCst);
                return;
            }
        };

        // Create run loop source from the event tap
        let run_loop_source = match event_tap.mach_port.create_runloop_source(0) {
            Ok(source) => source,
            Err(()) => {
                error!("Failed to create run loop source from event tap");
                running.store(false, Ordering::SeqCst);
                return;
            }
        };

        // Add source to run loop and enable the tap
        unsafe {
            current_run_loop.add_source(&run_loop_source, kCFRunLoopCommonModes);
        }
        event_tap.enable();

        info!("Keyboard monitor event tap enabled, entering run loop");

        // Run the loop until stopped
        while running.load(Ordering::SeqCst) {
            // Run for a short interval, then check if we should stop
            // Note: Must use kCFRunLoopDefaultMode (not kCFRunLoopCommonModes) for run_in_mode
            let result = CFRunLoop::run_in_mode(
                unsafe { kCFRunLoopDefaultMode },
                Duration::from_millis(100),
                true,
            );

            // Check if run loop was stopped externally
            if matches!(
                result,
                core_foundation::runloop::CFRunLoopRunResult::Stopped
            ) {
                debug!("Run loop was stopped");
                break;
            }
        }

        // Clean up: remove source from run loop
        // Note: The event tap will be disabled when it goes out of scope

        debug!("Keyboard monitor event loop exiting");
        running.store(false, Ordering::SeqCst);

        // Clear the stored run loop
        if let Ok(mut guard) = run_loop_storage.lock() {
            *guard = None;
        }
    }

    /// Extract key event information from a CGEvent
    fn extract_key_event(event: &CGEvent) -> KeyEvent {
        use core_graphics::event::CGEventFlags;

        // Get key code
        let key_code = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u16;

        // Get modifier flags
        let flags = event.get_flags();
        let shift = flags.contains(CGEventFlags::CGEventFlagShift);
        let control = flags.contains(CGEventFlags::CGEventFlagControl);
        let option = flags.contains(CGEventFlags::CGEventFlagAlternate);
        let command = flags.contains(CGEventFlags::CGEventFlagCommand);

        // Check if auto-repeat
        let is_repeat = event.get_integer_value_field(EventField::KEYBOARD_EVENT_AUTOREPEAT) != 0;

        // Try to get the character from the event
        // This uses the CGEventKeyboardGetUnicodeString function internally
        let character = Self::get_character_from_event(event);

        KeyEvent {
            character,
            key_code,
            shift,
            control,
            option,
            command,
            is_repeat,
        }
    }

    /// Get the character string from a keyboard event
    ///
    /// This attempts to get the actual character that would be typed,
    /// taking into account the keyboard layout and modifier keys.
    fn get_character_from_event(event: &CGEvent) -> Option<String> {
        // We need to use the CGEventKeyboardGetUnicodeString function
        // which isn't directly exposed by core-graphics, so we'll use FFI
        extern "C" {
            fn CGEventKeyboardGetUnicodeString(
                event: core_graphics::sys::CGEventRef,
                max_len: libc::c_ulong,
                actual_len: *mut libc::c_ulong,
                buffer: *mut u16,
            );
        }

        let mut buffer: [u16; 4] = [0; 4];
        let mut actual_len: libc::c_ulong = 0;

        unsafe {
            use foreign_types::ForeignType;
            // Get raw pointer to the CGEvent for FFI call
            let event_ptr = event.as_ptr();
            CGEventKeyboardGetUnicodeString(event_ptr, 4, &mut actual_len, buffer.as_mut_ptr());
        }

        if actual_len > 0 {
            String::from_utf16(&buffer[..actual_len as usize]).ok()
        } else {
            None
        }
    }
}

impl Drop for KeyboardMonitor {
    fn drop(&mut self) {
        self.stop();
    }
}

// KeyboardMonitor is Send because it uses Arc for shared state
// and the callback is required to be Send
unsafe impl Send for KeyboardMonitor {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accessibility_check_does_not_panic() {
        // This test just verifies the accessibility check doesn't panic
        // The actual result depends on system permissions
        let _ = KeyboardMonitor::has_accessibility_permission();
    }

    #[test]
    fn test_key_event_creation() {
        let event = KeyEvent {
            character: Some("a".to_string()),
            key_code: 0,
            shift: false,
            control: false,
            option: false,
            command: false,
            is_repeat: false,
        };

        assert_eq!(event.character, Some("a".to_string()));
        assert!(!event.shift);
        assert!(!event.is_repeat);
    }

    #[test]
    fn test_monitor_not_running_initially() {
        let monitor = KeyboardMonitor::new(|_| {});
        assert!(!monitor.is_running());
    }

    #[test]
    fn test_stop_when_not_running_is_safe() {
        let mut monitor = KeyboardMonitor::new(|_| {});
        // Should not panic
        monitor.stop();
        assert!(!monitor.is_running());
    }

    // Integration tests that require accessibility permissions are marked as ignored
    // Run with: cargo test --features system-tests -- --ignored
    #[test]
    #[ignore = "Requires accessibility permissions"]
    fn test_start_and_stop() {
        let mut monitor = KeyboardMonitor::new(|event| {
            println!("Key event: {:?}", event);
        });

        if !KeyboardMonitor::has_accessibility_permission() {
            eprintln!("Skipping test - accessibility permissions not granted");
            return;
        }

        assert!(monitor.start().is_ok());
        assert!(monitor.is_running());

        // Let it run briefly
        std::thread::sleep(std::time::Duration::from_millis(100));

        monitor.stop();
        assert!(!monitor.is_running());
    }

    #[test]
    #[ignore = "Requires accessibility permissions"]
    fn test_double_start_fails() {
        let mut monitor = KeyboardMonitor::new(|_| {});

        if !KeyboardMonitor::has_accessibility_permission() {
            eprintln!("Skipping test - accessibility permissions not granted");
            return;
        }

        assert!(monitor.start().is_ok());
        assert!(matches!(
            monitor.start(),
            Err(KeyboardMonitorError::AlreadyRunning)
        ));

        monitor.stop();
    }
}

</file>

<file path="src/hotkey_pollers.rs">
use gpui::{px, size, App, AppContext as _, AsyncApp, Context, Focusable, Window, WindowHandle};

use crate::ai;
use crate::hotkeys;
use crate::notes;
use crate::platform::{calculate_eye_line_bounds_on_mouse_display, move_first_window_to_bounds};
use crate::window_resize::{initial_window_height, reset_resize_debounce};
use crate::{logging, platform, ScriptListApp, NEEDS_RESET, PANEL_CONFIGURED};

/// A simple model that listens for hotkey triggers via async_channel (event-driven).
#[allow(dead_code)]
pub struct HotkeyPoller {
    window: WindowHandle<ScriptListApp>,
}

impl HotkeyPoller {
    pub fn new(window: WindowHandle<ScriptListApp>) -> Self {
        Self { window }
    }

    pub fn start_listening(&self, cx: &mut Context<Self>) {
        let window = self.window;
        // Event-driven: recv().await yields immediately when hotkey is pressed
        // No polling - replaces 100ms Timer::after loop
        cx.spawn(async move |_this, cx: &mut AsyncApp| {
            logging::log("HOTKEY", "Hotkey listener started (event-driven via async_channel)");

            while let Ok(()) = hotkeys::hotkey_channel().1.recv().await {
                logging::log("VISIBILITY", "");
                logging::log("VISIBILITY", "╔════════════════════════════════════════════════════════════╗");
                logging::log("VISIBILITY", "║  HOTKEY TRIGGERED - TOGGLE WINDOW                          ║");
                logging::log("VISIBILITY", "╚════════════════════════════════════════════════════════════╝");

                // CRITICAL: If Notes or AI windows are open, the main hotkey should be completely ignored.
                // The hotkeys are independent - main hotkey should have ZERO effect on Notes/AI.
                let notes_open = notes::is_notes_window_open();
                let ai_open = ai::is_ai_window_open();

                if notes_open || ai_open {
                    logging::log(
                        "VISIBILITY",
                        &format!(
                            "Notes/AI window is open (notes={}, ai={}) - main hotkey IGNORED",
                            notes_open, ai_open
                        )
                    );
                    logging::log("VISIBILITY", "═══════════════════════════════════════════════════════════════");
                    continue; // Completely skip - don't toggle main window at all
                }

                // Check current visibility state for toggle behavior
                let is_visible = script_kit_gpui::is_main_window_visible();
                let needs_reset = NEEDS_RESET.load(std::sync::atomic::Ordering::SeqCst);
                logging::log(
                    "VISIBILITY",
                    &format!(
                        "State check: WINDOW_VISIBLE={}, NEEDS_RESET={}",
                        is_visible, needs_reset
                    ),
                );

                if is_visible {
                    logging::log("VISIBILITY", "Decision: HIDE (window is currently visible)");
                    // Update visibility state FIRST to prevent race conditions
                    // Even though the hide is async, we mark it as hidden immediately
                    script_kit_gpui::set_main_window_visible(false);
                    logging::log("VISIBILITY", "WINDOW_VISIBLE set to: false");

                    // Window is visible - check if in prompt mode
                    let window_clone = window;

                    // Check if Notes or AI windows are open - if so, only hide main window, not the whole app
                    let notes_open = notes::is_notes_window_open();
                    let ai_open = ai::is_ai_window_open();

                    // First check if we're in a prompt - if so, cancel and hide
                    let _ = cx.update(move |cx: &mut App| {
                        let _ = window_clone.update(
                            cx,
                            |view: &mut ScriptListApp,
                             _win: &mut Window,
                             ctx: &mut Context<ScriptListApp>| {
                                if view.is_in_prompt() {
                                    logging::log(
                                        "HOTKEY",
                                        "In prompt mode - canceling script before hiding",
                                    );
                                    view.cancel_script_execution(ctx);
                                }
                                // Reset UI state before hiding (clears selection, scroll position, filter)
                                logging::log("HOTKEY", "Resetting to script list before hiding");
                                view.reset_to_script_list(ctx);
                            },
                        );

                        // Hide the main window
                        logging::log("HOTKEY", "Hiding window (toggle: visible -> hidden)");
                        let hide_start = std::time::Instant::now();

                        // CRITICAL: If Notes or AI windows are open, only hide the main window
                        // using platform::hide_main_window(). Don't call cx.hide() which would
                        // hide ALL windows including Notes/AI.
                        if notes_open || ai_open {
                            logging::log("HOTKEY", "Notes/AI window open - using orderOut to hide only main window");
                            platform::hide_main_window();
                        } else {
                            // No other windows open - safe to hide the entire app
                            cx.hide();
                        }

                        let hide_elapsed = hide_start.elapsed();
                        logging::log(
                            "PERF",
                            &format!("Window hide took {:.2}ms", hide_elapsed.as_secs_f64() * 1000.0),
                        );
                        logging::log("HOTKEY", "Main window hidden");
                    });
                } else {
                    logging::log("VISIBILITY", "Decision: SHOW (window is currently hidden)");

                    // Menu bar tracking is now handled by frontmost_app_tracker module
                    // which pre-fetches menu items in background when apps activate

                    // Update visibility state FIRST to prevent race conditions
                    script_kit_gpui::set_main_window_visible(true);
                    logging::log("VISIBILITY", "WINDOW_VISIBLE set to: true");

                    let window_clone = window;
                    let _ = cx.update(move |cx: &mut App| {
                        // Step 0: CRITICAL - Set MoveToActiveSpace BEFORE any activation
                        // This MUST happen before move_first_window_to_bounds, cx.activate(),
                        // or win.activate_window() to prevent macOS from switching spaces
                        platform::ensure_move_to_active_space();

                        // Step 1: Calculate new bounds on display with mouse, at eye-line height
                        let window_size = size(px(750.), initial_window_height());
                        let new_bounds = calculate_eye_line_bounds_on_mouse_display(window_size);

                        logging::log(
                            "HOTKEY",
                            &format!(
                                "Calculated bounds: origin=({:.0}, {:.0}) size={:.0}x{:.0}",
                                f64::from(new_bounds.origin.x),
                                f64::from(new_bounds.origin.y),
                                f64::from(new_bounds.size.width),
                                f64::from(new_bounds.size.height)
                            ),
                        );

                        // Step 2: Move window (position only, no activation)
                        // Note: makeKeyAndOrderFront was removed - ordering happens via GPUI below
                        move_first_window_to_bounds(&new_bounds);
                        logging::log("HOTKEY", "Window repositioned to mouse display");

                        // Step 3: NOW activate the app (makes window visible at new position)
                        cx.activate(true);
                        logging::log("HOTKEY", "App activated (window now visible)");

                        // Step 3.5: Configure as floating panel on first show only
                        if !PANEL_CONFIGURED.swap(true, std::sync::atomic::Ordering::SeqCst) {
                            platform::configure_as_floating_panel();
                            logging::log("HOTKEY", "Configured window as floating panel (first show)");
                        }

                        // Step 4: Activate the specific window and focus it
                        let _ = window_clone.update(
                            cx,
                            |view: &mut ScriptListApp, win: &mut Window, cx: &mut Context<ScriptListApp>| {
                                win.activate_window();
                                let focus_handle = view.focus_handle(cx);
                                win.focus(&focus_handle, cx);
                                logging::log("HOTKEY", "Window activated and focused");

                                // Menu bar items are now tracked by frontmost_app_tracker
                                // No state reset needed here

                                // Step 5: Check if we need to reset to script list (after script completion)
                                // Reset debounce timer to allow immediate resize after window move
                                reset_resize_debounce();

                                if NEEDS_RESET
                                    .compare_exchange(
                                        true,
                                        false,
                                        std::sync::atomic::Ordering::SeqCst,
                                        std::sync::atomic::Ordering::SeqCst,
                                    )
                                    .is_ok()
                                {
                                    logging::log(
                                        "VISIBILITY",
                                        "NEEDS_RESET was true - clearing and resetting to script list",
                                    );
                                    view.reset_to_script_list(cx);
                                } else {
                                    // Even without reset, ensure window is properly sized for current content
                                    view.update_window_size();
                                }
                            },
                        );

                        logging::log("VISIBILITY", "Window show sequence complete");
                    });
                }

                let final_visible = script_kit_gpui::is_main_window_visible();
                let final_reset = NEEDS_RESET.load(std::sync::atomic::Ordering::SeqCst);
                logging::log(
                    "VISIBILITY",
                    &format!("Final state: WINDOW_VISIBLE={}, NEEDS_RESET={}", final_visible, final_reset),
                );
                logging::log("VISIBILITY", "═══════════════════════════════════════════════════════════════");
            }

            logging::log("HOTKEY", "Hotkey listener exiting (channel closed)");
        })
        .detach();
    }
}

/// A model that listens for script hotkey triggers via async_channel.
#[allow(dead_code)]
pub struct ScriptHotkeyPoller {
    window: WindowHandle<ScriptListApp>,
}

impl ScriptHotkeyPoller {
    pub fn new(window: WindowHandle<ScriptListApp>) -> Self {
        Self { window }
    }

    pub fn start_listening(&self, cx: &mut Context<Self>) {
        let window = self.window;
        cx.spawn(async move |_this, cx: &mut AsyncApp| {
            logging::log("HOTKEY", "Script hotkey listener started");

            while let Ok(script_path) = hotkeys::script_hotkey_channel().1.recv().await {
                logging::log(
                    "HOTKEY",
                    &format!("Script shortcut received: {}", script_path),
                );

                let path_clone = script_path.clone();
                let _ = cx.update(move |cx: &mut App| {
                    let _ = window.update(
                        cx,
                        |view: &mut ScriptListApp,
                         _win: &mut Window,
                         ctx: &mut Context<ScriptListApp>| {
                            // Find and execute the script by path
                            view.execute_script_by_path(&path_clone, ctx);
                        },
                    );
                });
            }

            logging::log("HOTKEY", "Script hotkey listener exiting");
        })
        .detach();
    }
}

/// A model that listens for notes hotkey triggers via async_channel.
#[allow(dead_code)]
pub struct NotesHotkeyPoller;

impl NotesHotkeyPoller {
    pub fn new() -> Self {
        Self
    }

    pub fn start_listening(&self, cx: &mut Context<Self>) {
        cx.spawn(async move |_this, cx: &mut AsyncApp| {
            logging::log("HOTKEY", "Notes hotkey listener started");

            while let Ok(()) = hotkeys::notes_hotkey_channel().1.recv().await {
                logging::log("HOTKEY", "Notes hotkey triggered - opening notes window");

                let _ = cx.update(move |cx: &mut App| {
                    if let Err(e) = notes::open_notes_window(cx) {
                        logging::log("HOTKEY", &format!("Failed to open notes window: {}", e));
                    }
                });
            }

            logging::log("HOTKEY", "Notes hotkey listener exiting");
        })
        .detach();
    }
}

/// A model that listens for AI hotkey triggers via async_channel.
#[allow(dead_code)]
pub struct AiHotkeyPoller;

impl AiHotkeyPoller {
    pub fn new() -> Self {
        Self
    }

    pub fn start_listening(&self, cx: &mut Context<Self>) {
        cx.spawn(async move |_this, cx: &mut AsyncApp| {
            logging::log("HOTKEY", "AI hotkey listener started");

            while let Ok(()) = hotkeys::ai_hotkey_channel().1.recv().await {
                logging::log("HOTKEY", "AI hotkey triggered - opening AI window");

                let _ = cx.update(move |cx: &mut App| {
                    if let Err(e) = ai::open_ai_window(cx) {
                        logging::log("HOTKEY", &format!("Failed to open AI window: {}", e));
                    }
                });
            }

            logging::log("HOTKEY", "AI hotkey listener exiting");
        })
        .detach();
    }
}

#[allow(dead_code)]
pub(crate) fn start_hotkey_event_handler(cx: &mut App, window: WindowHandle<ScriptListApp>) {
    // Start main hotkey listener (for app show/hide toggle)
    let handler = cx.new(|_| HotkeyPoller::new(window));
    handler.update(cx, |p, cx| {
        p.start_listening(cx);
    });

    // Start script hotkey listener (for direct script execution via shortcuts)
    let script_handler = cx.new(|_| ScriptHotkeyPoller::new(window));
    script_handler.update(cx, |p, cx| {
        p.start_listening(cx);
    });

    // Start notes hotkey listener (for opening notes window)
    let notes_handler = cx.new(|_| NotesHotkeyPoller::new());
    notes_handler.update(cx, |p, cx| {
        p.start_listening(cx);
    });

    // Start AI hotkey listener (for opening AI window)
    let ai_handler = cx.new(|_| AiHotkeyPoller::new());
    ai_handler.update(cx, |p, cx| {
        p.start_listening(cx);
    });
}

</file>

<file path="src/hotkeys.rs">
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use crate::{config, logging, scripts, shortcuts};

// =============================================================================
// Dynamic Script Hotkey Manager
// =============================================================================

/// Manages dynamic registration/unregistration of script hotkeys.
/// Uses a thread-safe global singleton pattern for access from multiple contexts.
pub struct ScriptHotkeyManager {
    /// The underlying global hotkey manager
    manager: GlobalHotKeyManager,
    /// Maps hotkey ID -> script path
    hotkey_map: HashMap<u32, String>,
    /// Maps script path -> hotkey ID (reverse lookup for unregistration)
    path_to_id: HashMap<String, u32>,
    /// Maps script path -> HotKey object (needed for proper unregistration)
    path_to_hotkey: HashMap<String, HotKey>,
}

impl ScriptHotkeyManager {
    /// Create a new ScriptHotkeyManager.
    /// NOTE: Must be created on the main thread.
    fn new(manager: GlobalHotKeyManager) -> Self {
        Self {
            manager,
            hotkey_map: HashMap::new(),
            path_to_id: HashMap::new(),
            path_to_hotkey: HashMap::new(),
        }
    }

    /// Register a hotkey for a script.
    /// Returns the hotkey ID on success.
    pub fn register(&mut self, path: &str, shortcut: &str) -> anyhow::Result<u32> {
        // Parse the shortcut
        let (mods, code) = shortcuts::parse_shortcut(shortcut)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse shortcut: {}", shortcut))?;

        let hotkey = HotKey::new(Some(mods), code);
        let hotkey_id = hotkey.id();

        // Register with the OS
        self.manager
            .register(hotkey)
            .map_err(|e| anyhow::anyhow!("Failed to register hotkey: {}", e))?;

        // Track the mapping
        self.hotkey_map.insert(hotkey_id, path.to_string());
        self.path_to_id.insert(path.to_string(), hotkey_id);
        self.path_to_hotkey.insert(path.to_string(), hotkey);

        logging::log(
            "HOTKEY",
            &format!(
                "Registered script hotkey '{}' for {} (id: {})",
                shortcut, path, hotkey_id
            ),
        );

        Ok(hotkey_id)
    }

    /// Unregister a hotkey for a script by path.
    /// Returns Ok(()) even if the path wasn't registered (no-op).
    pub fn unregister(&mut self, path: &str) -> anyhow::Result<()> {
        if let Some(hotkey_id) = self.path_to_id.remove(path) {
            // Remove from hotkey_map
            self.hotkey_map.remove(&hotkey_id);

            // Unregister from OS using stored HotKey object
            if let Some(hotkey) = self.path_to_hotkey.remove(path) {
                if let Err(e) = self.manager.unregister(hotkey) {
                    logging::log(
                        "HOTKEY",
                        &format!(
                            "Warning: Failed to unregister hotkey for {} (id: {}): {}",
                            path, hotkey_id, e
                        ),
                    );
                    // Continue anyway - the internal tracking is already updated
                }
            }

            logging::log(
                "HOTKEY",
                &format!(
                    "Unregistered script hotkey for {} (id: {})",
                    path, hotkey_id
                ),
            );
        }
        // If path wasn't registered, this is a no-op (success)
        Ok(())
    }

    /// Update a script's hotkey.
    /// Handles add (old=None, new=Some), remove (old=Some, new=None), and change (both Some).
    pub fn update(
        &mut self,
        path: &str,
        old_shortcut: Option<&str>,
        new_shortcut: Option<&str>,
    ) -> anyhow::Result<()> {
        match (old_shortcut, new_shortcut) {
            (None, None) => {
                // No change needed
                Ok(())
            }
            (None, Some(new)) => {
                // Add new hotkey
                self.register(path, new)?;
                Ok(())
            }
            (Some(_old), None) => {
                // Remove old hotkey
                self.unregister(path)
            }
            (Some(_old), Some(new)) => {
                // Change: unregister old, register new
                self.unregister(path)?;
                self.register(path, new)?;
                Ok(())
            }
        }
    }

    /// Get the script path for a given hotkey ID.
    pub fn get_script_path(&self, hotkey_id: u32) -> Option<&String> {
        self.hotkey_map.get(&hotkey_id)
    }

    /// Get all registered hotkeys as (path, hotkey_id) pairs.
    pub fn get_registered_hotkeys(&self) -> Vec<(String, u32)> {
        self.path_to_id
            .iter()
            .map(|(path, id)| (path.clone(), *id))
            .collect()
    }

    /// Check if a script has a registered hotkey.
    #[allow(dead_code)]
    pub fn is_registered(&self, path: &str) -> bool {
        self.path_to_id.contains_key(path)
    }
}

/// Global singleton for the ScriptHotkeyManager.
/// Initialized when start_hotkey_listener is called.
static SCRIPT_HOTKEY_MANAGER: OnceLock<Mutex<ScriptHotkeyManager>> = OnceLock::new();

/// Initialize the global ScriptHotkeyManager.
/// Must be called from the main thread.
/// Returns an error if already initialized.
#[allow(dead_code)]
pub fn init_script_hotkey_manager(manager: GlobalHotKeyManager) -> anyhow::Result<()> {
    SCRIPT_HOTKEY_MANAGER
        .set(Mutex::new(ScriptHotkeyManager::new(manager)))
        .map_err(|_| anyhow::anyhow!("ScriptHotkeyManager already initialized"))
}

/// Register a script hotkey dynamically.
/// Returns the hotkey ID on success.
pub fn register_script_hotkey(path: &str, shortcut: &str) -> anyhow::Result<u32> {
    let manager = SCRIPT_HOTKEY_MANAGER
        .get()
        .ok_or_else(|| anyhow::anyhow!("ScriptHotkeyManager not initialized"))?;

    let mut guard = manager
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
    guard.register(path, shortcut)
}

/// Unregister a script hotkey by path.
/// Returns Ok(()) even if the path wasn't registered (no-op).
pub fn unregister_script_hotkey(path: &str) -> anyhow::Result<()> {
    let manager = SCRIPT_HOTKEY_MANAGER
        .get()
        .ok_or_else(|| anyhow::anyhow!("ScriptHotkeyManager not initialized"))?;

    let mut guard = manager
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
    guard.unregister(path)
}

/// Update a script's hotkey.
/// Handles add (old=None, new=Some), remove (old=Some, new=None), and change (both Some).
pub fn update_script_hotkey(
    path: &str,
    old_shortcut: Option<&str>,
    new_shortcut: Option<&str>,
) -> anyhow::Result<()> {
    let manager = SCRIPT_HOTKEY_MANAGER
        .get()
        .ok_or_else(|| anyhow::anyhow!("ScriptHotkeyManager not initialized"))?;

    let mut guard = manager
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
    guard.update(path, old_shortcut, new_shortcut)
}

/// Get the script path for a given hotkey ID.
#[allow(dead_code)]
pub fn get_script_for_hotkey(hotkey_id: u32) -> Option<String> {
    let manager = SCRIPT_HOTKEY_MANAGER.get()?;
    let guard = manager.lock().ok()?;
    guard.get_script_path(hotkey_id).cloned()
}

/// Get all registered script hotkeys.
#[allow(dead_code)]
pub fn get_registered_hotkeys() -> Vec<(String, u32)> {
    SCRIPT_HOTKEY_MANAGER
        .get()
        .and_then(|m| m.lock().ok())
        .map(|guard| guard.get_registered_hotkeys())
        .unwrap_or_default()
}

// =============================================================================
// GCD dispatch for immediate main-thread execution (bypasses async runtime)
// =============================================================================

use std::sync::Arc;

/// Callback type for hotkey actions - uses Arc<dyn Fn()> for repeated invocation
pub type HotkeyHandler = Arc<dyn Fn() + Send + Sync>;

/// Static storage for handlers to be invoked on main thread
static NOTES_HANDLER: OnceLock<std::sync::Mutex<Option<HotkeyHandler>>> = OnceLock::new();
static AI_HANDLER: OnceLock<std::sync::Mutex<Option<HotkeyHandler>>> = OnceLock::new();

/// Register a handler to be invoked when the Notes hotkey is pressed.
/// This handler will be executed on the main thread via GCD dispatch_async.
/// The handler can be called multiple times (it's not consumed).
#[allow(dead_code)]
pub fn set_notes_hotkey_handler<F: Fn() + Send + Sync + 'static>(handler: F) {
    let storage = NOTES_HANDLER.get_or_init(|| std::sync::Mutex::new(None));
    *storage.lock().unwrap() = Some(Arc::new(handler));
}

/// Register a handler to be invoked when the AI hotkey is pressed.
/// This handler will be executed on the main thread via GCD dispatch_async.
/// The handler can be called multiple times (it's not consumed).
#[allow(dead_code)]
pub fn set_ai_hotkey_handler<F: Fn() + Send + Sync + 'static>(handler: F) {
    let storage = AI_HANDLER.get_or_init(|| std::sync::Mutex::new(None));
    *storage.lock().unwrap() = Some(Arc::new(handler));
}

#[cfg(target_os = "macos")]
mod gcd {
    use std::ffi::c_void;

    // Link to libSystem for GCD functions
    // Note: dispatch_get_main_queue is actually a macro that returns &_dispatch_main_q
    // We use the raw symbol directly instead
    #[link(name = "System", kind = "framework")]
    extern "C" {
        fn dispatch_async_f(
            queue: *const c_void,
            context: *mut c_void,
            work: extern "C" fn(*mut c_void),
        );
        // The main dispatch queue is a global static symbol, not a function
        #[link_name = "_dispatch_main_q"]
        static DISPATCH_MAIN_QUEUE: c_void;
    }

    /// Dispatch a closure to the main thread via GCD.
    /// This is the key to making hotkeys work before the GPUI event loop is "warmed up".
    pub fn dispatch_to_main<F: FnOnce() + Send + 'static>(f: F) {
        let boxed: Box<dyn FnOnce() + Send> = Box::new(f);
        let raw = Box::into_raw(Box::new(boxed));

        extern "C" fn trampoline(context: *mut c_void) {
            unsafe {
                let boxed: Box<Box<dyn FnOnce() + Send>> = Box::from_raw(context as *mut _);
                boxed();
            }
        }

        unsafe {
            let main_queue = &DISPATCH_MAIN_QUEUE as *const c_void;
            dispatch_async_f(main_queue, raw as *mut c_void, trampoline);
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod gcd {
    /// Fallback for non-macOS: just call the closure directly (in the current thread)
    pub fn dispatch_to_main<F: FnOnce() + Send + 'static>(f: F) {
        f();
    }
}

/// Dispatch the Notes hotkey handler to the main thread.
///
/// Strategy:
/// 1. Send to channel (wakes any async waiters)
/// 2. Dispatch a no-op to main thread via GCD (ensures GPUI event loop processes)
///
/// This works even before the main window is activated because GCD dispatch
/// directly integrates with the NSApplication run loop that GPUI uses.
fn dispatch_notes_hotkey() {
    // Send to channel - this wakes any async task waiting on recv()
    if notes_hotkey_channel().0.try_send(()).is_err() {
        logging::log("HOTKEY", "Notes hotkey channel full/closed");
    }

    // Also try the handler approach for immediate execution
    let handler = NOTES_HANDLER
        .get_or_init(|| std::sync::Mutex::new(None))
        .lock()
        .unwrap()
        .clone();

    if let Some(handler) = handler {
        gcd::dispatch_to_main(move || {
            handler();
        });
    } else {
        // Dispatch an empty closure to wake GPUI's event loop
        // This ensures the channel message gets processed even if GPUI was idle
        gcd::dispatch_to_main(|| {
            // Empty closure - just wakes the run loop
        });
    }
}

/// Dispatch the AI hotkey handler to the main thread.
/// Same strategy as Notes hotkey.
fn dispatch_ai_hotkey() {
    // Send to channel - this wakes any async task waiting on recv()
    if ai_hotkey_channel().0.try_send(()).is_err() {
        logging::log("HOTKEY", "AI hotkey channel full/closed");
    }

    // Also try the handler approach for immediate execution
    let handler = AI_HANDLER
        .get_or_init(|| std::sync::Mutex::new(None))
        .lock()
        .unwrap()
        .clone();

    if let Some(handler) = handler {
        gcd::dispatch_to_main(move || {
            handler();
        });
    } else {
        // Dispatch an empty closure to wake GPUI's event loop
        gcd::dispatch_to_main(|| {
            // Empty closure - just wakes the run loop
        });
    }
}

// HOTKEY_CHANNEL: Event-driven async_channel for hotkey events (replaces AtomicBool polling)
#[allow(dead_code)]
static HOTKEY_CHANNEL: OnceLock<(async_channel::Sender<()>, async_channel::Receiver<()>)> =
    OnceLock::new();

/// Get the hotkey channel, initializing it on first access.
#[allow(dead_code)]
pub(crate) fn hotkey_channel() -> &'static (async_channel::Sender<()>, async_channel::Receiver<()>)
{
    HOTKEY_CHANNEL.get_or_init(|| async_channel::bounded(10))
}

// SCRIPT_HOTKEY_CHANNEL: Channel for script shortcut events (sends script path)
#[allow(dead_code)]
static SCRIPT_HOTKEY_CHANNEL: OnceLock<(
    async_channel::Sender<String>,
    async_channel::Receiver<String>,
)> = OnceLock::new();

/// Get the script hotkey channel, initializing it on first access.
#[allow(dead_code)]
pub(crate) fn script_hotkey_channel() -> &'static (
    async_channel::Sender<String>,
    async_channel::Receiver<String>,
) {
    SCRIPT_HOTKEY_CHANNEL.get_or_init(|| async_channel::bounded(10))
}

// NOTES_HOTKEY_CHANNEL: Channel for notes hotkey events
#[allow(dead_code)]
static NOTES_HOTKEY_CHANNEL: OnceLock<(async_channel::Sender<()>, async_channel::Receiver<()>)> =
    OnceLock::new();

/// Get the notes hotkey channel, initializing it on first access.
#[allow(dead_code)]
pub(crate) fn notes_hotkey_channel(
) -> &'static (async_channel::Sender<()>, async_channel::Receiver<()>) {
    NOTES_HOTKEY_CHANNEL.get_or_init(|| async_channel::bounded(10))
}

// AI_HOTKEY_CHANNEL: Channel for AI hotkey events
#[allow(dead_code)]
static AI_HOTKEY_CHANNEL: OnceLock<(async_channel::Sender<()>, async_channel::Receiver<()>)> =
    OnceLock::new();

/// Get the AI hotkey channel, initializing it on first access.
#[allow(dead_code)]
pub(crate) fn ai_hotkey_channel(
) -> &'static (async_channel::Sender<()>, async_channel::Receiver<()>) {
    AI_HOTKEY_CHANNEL.get_or_init(|| async_channel::bounded(10))
}

#[allow(dead_code)]
static HOTKEY_TRIGGER_COUNT: AtomicU64 = AtomicU64::new(0);

#[allow(dead_code)]
pub(crate) fn start_hotkey_listener(config: config::Config) {
    std::thread::spawn(move || {
        let manager = match GlobalHotKeyManager::new() {
            Ok(m) => m,
            Err(e) => {
                logging::log("HOTKEY", &format!("Failed to create hotkey manager: {}", e));
                return;
            }
        };

        // Convert config hotkey to global_hotkey::Code
        let code = match config.hotkey.key.as_str() {
            "Semicolon" => Code::Semicolon,
            "KeyK" => Code::KeyK,
            "KeyP" => Code::KeyP,
            "Space" => Code::Space,
            "Enter" => Code::Enter,
            "Digit0" => Code::Digit0,
            "Digit1" => Code::Digit1,
            "Digit2" => Code::Digit2,
            "Digit3" => Code::Digit3,
            "Digit4" => Code::Digit4,
            "Digit5" => Code::Digit5,
            "Digit6" => Code::Digit6,
            "Digit7" => Code::Digit7,
            "Digit8" => Code::Digit8,
            "Digit9" => Code::Digit9,
            "KeyA" => Code::KeyA,
            "KeyB" => Code::KeyB,
            "KeyC" => Code::KeyC,
            "KeyD" => Code::KeyD,
            "KeyE" => Code::KeyE,
            "KeyF" => Code::KeyF,
            "KeyG" => Code::KeyG,
            "KeyH" => Code::KeyH,
            "KeyI" => Code::KeyI,
            "KeyJ" => Code::KeyJ,
            "KeyL" => Code::KeyL,
            "KeyM" => Code::KeyM,
            "KeyN" => Code::KeyN,
            "KeyO" => Code::KeyO,
            "KeyQ" => Code::KeyQ,
            "KeyR" => Code::KeyR,
            "KeyS" => Code::KeyS,
            "KeyT" => Code::KeyT,
            "KeyU" => Code::KeyU,
            "KeyV" => Code::KeyV,
            "KeyW" => Code::KeyW,
            "KeyX" => Code::KeyX,
            "KeyY" => Code::KeyY,
            "KeyZ" => Code::KeyZ,
            // Function keys
            "F1" => Code::F1,
            "F2" => Code::F2,
            "F3" => Code::F3,
            "F4" => Code::F4,
            "F5" => Code::F5,
            "F6" => Code::F6,
            "F7" => Code::F7,
            "F8" => Code::F8,
            "F9" => Code::F9,
            "F10" => Code::F10,
            "F11" => Code::F11,
            "F12" => Code::F12,
            other => {
                logging::log(
                    "HOTKEY",
                    &format!(
                        "Unknown key code: '{}'. Valid keys: KeyA-KeyZ, Digit0-Digit9, F1-F12, Space, Enter, Semicolon. Falling back to Semicolon",
                        other
                    ),
                );
                Code::Semicolon
            }
        };

        // Convert modifiers from config strings to Modifiers flags
        let mut modifiers = Modifiers::empty();
        for modifier in &config.hotkey.modifiers {
            match modifier.as_str() {
                "meta" => modifiers |= Modifiers::META,
                "ctrl" => modifiers |= Modifiers::CONTROL,
                "alt" => modifiers |= Modifiers::ALT,
                "shift" => modifiers |= Modifiers::SHIFT,
                other => {
                    logging::log("HOTKEY", &format!("Unknown modifier: {}", other));
                }
            }
        }

        let hotkey = HotKey::new(Some(modifiers), code);
        let main_hotkey_id = hotkey.id();

        let hotkey_display = format!(
            "{}{}",
            config.hotkey.modifiers.join("+"),
            if config.hotkey.modifiers.is_empty() {
                String::new()
            } else {
                "+".to_string()
            }
        ) + &config.hotkey.key;

        if let Err(e) = manager.register(hotkey) {
            logging::log(
                "HOTKEY",
                &format!("Failed to register {}: {}", hotkey_display, e),
            );
            return;
        }

        logging::log(
            "HOTKEY",
            &format!(
                "Registered global hotkey {} (id: {})",
                hotkey_display, main_hotkey_id
            ),
        );

        // Register notes hotkey (Cmd+Shift+N by default)
        let notes_config = config.get_notes_hotkey();
        let notes_code = match notes_config.key.as_str() {
            "KeyN" => Code::KeyN,
            "KeyM" => Code::KeyM,
            "KeyO" => Code::KeyO,
            "KeyP" => Code::KeyP,
            _ => Code::KeyN, // Default to N
        };

        let mut notes_modifiers = Modifiers::empty();
        for modifier in &notes_config.modifiers {
            match modifier.as_str() {
                "meta" => notes_modifiers |= Modifiers::META,
                "ctrl" => notes_modifiers |= Modifiers::CONTROL,
                "alt" => notes_modifiers |= Modifiers::ALT,
                "shift" => notes_modifiers |= Modifiers::SHIFT,
                _ => {}
            }
        }

        let notes_hotkey = HotKey::new(Some(notes_modifiers), notes_code);
        let notes_hotkey_id = notes_hotkey.id();

        let notes_display = format!(
            "{}{}",
            notes_config.modifiers.join("+"),
            if notes_config.modifiers.is_empty() {
                String::new()
            } else {
                "+".to_string()
            }
        ) + &notes_config.key;

        if let Err(e) = manager.register(notes_hotkey) {
            logging::log(
                "HOTKEY",
                &format!("Failed to register notes hotkey {}: {}", notes_display, e),
            );
        } else {
            logging::log(
                "HOTKEY",
                &format!(
                    "Registered notes hotkey {} (id: {})",
                    notes_display, notes_hotkey_id
                ),
            );
        }

        // Register AI hotkey (Cmd+Shift+Space by default)
        let ai_config = config.get_ai_hotkey();
        let ai_code = match ai_config.key.as_str() {
            "Space" => Code::Space,
            "KeyA" => Code::KeyA,
            "KeyI" => Code::KeyI,
            _ => Code::Space, // Default to Space
        };

        let mut ai_modifiers = Modifiers::empty();
        for modifier in &ai_config.modifiers {
            match modifier.as_str() {
                "meta" => ai_modifiers |= Modifiers::META,
                "ctrl" => ai_modifiers |= Modifiers::CONTROL,
                "alt" => ai_modifiers |= Modifiers::ALT,
                "shift" => ai_modifiers |= Modifiers::SHIFT,
                _ => {}
            }
        }

        let ai_hotkey = HotKey::new(Some(ai_modifiers), ai_code);
        let ai_hotkey_id = ai_hotkey.id();

        let ai_display = format!(
            "{}{}",
            ai_config.modifiers.join("+"),
            if ai_config.modifiers.is_empty() {
                String::new()
            } else {
                "+".to_string()
            }
        ) + &ai_config.key;

        if let Err(e) = manager.register(ai_hotkey) {
            logging::log(
                "HOTKEY",
                &format!("Failed to register AI hotkey {}: {}", ai_display, e),
            );
        } else {
            logging::log(
                "HOTKEY",
                &format!("Registered AI hotkey {} (id: {})", ai_display, ai_hotkey_id),
            );
        }

        // Register script shortcuts
        // Map from hotkey ID to script path
        let mut script_hotkey_map: std::collections::HashMap<u32, String> =
            std::collections::HashMap::new();

        // Load scripts with shortcuts
        let all_scripts = scripts::read_scripts();
        for script in &all_scripts {
            if let Some(ref shortcut) = script.shortcut {
                if let Some((mods, key_code)) = shortcuts::parse_shortcut(shortcut) {
                    let script_hotkey = HotKey::new(Some(mods), key_code);
                    let script_hotkey_id = script_hotkey.id();

                    match manager.register(script_hotkey) {
                        Ok(()) => {
                            script_hotkey_map.insert(
                                script_hotkey_id,
                                script.path.to_string_lossy().to_string(),
                            );
                            logging::log(
                                "HOTKEY",
                                &format!(
                                    "Registered script shortcut '{}' for {} (id: {})",
                                    shortcut, script.name, script_hotkey_id
                                ),
                            );
                        }
                        Err(e) => {
                            logging::log(
                                "HOTKEY",
                                &format!(
                                    "Failed to register shortcut '{}' for {}: {}",
                                    shortcut, script.name, e
                                ),
                            );
                        }
                    }
                } else {
                    logging::log(
                        "HOTKEY",
                        &format!(
                            "Failed to parse shortcut '{}' for script {}",
                            shortcut, script.name
                        ),
                    );
                }
            }
        }

        // Load scriptlets with shortcuts
        let all_scriptlets = scripts::load_scriptlets();
        for scriptlet in &all_scriptlets {
            if let Some(ref shortcut) = scriptlet.shortcut {
                if let Some((mods, key_code)) = shortcuts::parse_shortcut(shortcut) {
                    let scriptlet_hotkey = HotKey::new(Some(mods), key_code);
                    let scriptlet_hotkey_id = scriptlet_hotkey.id();

                    // Use file_path as the identifier (already includes #command)
                    let scriptlet_path = scriptlet
                        .file_path
                        .clone()
                        .unwrap_or_else(|| scriptlet.name.clone());

                    match manager.register(scriptlet_hotkey) {
                        Ok(()) => {
                            script_hotkey_map.insert(scriptlet_hotkey_id, scriptlet_path.clone());
                            logging::log(
                                "HOTKEY",
                                &format!(
                                    "Registered scriptlet shortcut '{}' for {} (id: {})",
                                    shortcut, scriptlet.name, scriptlet_hotkey_id
                                ),
                            );
                        }
                        Err(e) => {
                            logging::log(
                                "HOTKEY",
                                &format!(
                                    "Failed to register shortcut '{}' for {}: {}",
                                    shortcut, scriptlet.name, e
                                ),
                            );
                        }
                    }
                }
            }
        }

        logging::log(
            "HOTKEY",
            &format!(
                "Registered {} script/scriptlet shortcuts",
                script_hotkey_map.len()
            ),
        );

        let receiver = GlobalHotKeyEvent::receiver();

        // Log all registered hotkey IDs for debugging
        logging::log(
            "HOTKEY",
            &format!(
                "Hotkey ID map: main={}, notes={}, ai={}",
                main_hotkey_id, notes_hotkey_id, ai_hotkey_id
            ),
        );

        loop {
            if let Ok(event) = receiver.recv() {
                // Only respond to key PRESS, not release
                if event.state != HotKeyState::Pressed {
                    continue;
                }

                // Log EVERY hotkey event with its ID for debugging
                logging::log(
                    "HOTKEY",
                    &format!(
                        "Received event id={} (main={}, notes={}, ai={})",
                        event.id, main_hotkey_id, notes_hotkey_id, ai_hotkey_id
                    ),
                );

                // Check if it's the main app hotkey
                if event.id == main_hotkey_id {
                    let count = HOTKEY_TRIGGER_COUNT.fetch_add(1, Ordering::SeqCst);
                    // Send via async_channel for immediate event-driven handling
                    if hotkey_channel().0.send_blocking(()).is_err() {
                        logging::log("HOTKEY", "Hotkey channel closed, cannot send");
                    }
                    logging::log(
                        "HOTKEY",
                        &format!("{} pressed (trigger #{})", hotkey_display, count + 1),
                    );
                }
                // Check if it's the notes hotkey - dispatch directly to main thread via GCD
                else if event.id == notes_hotkey_id {
                    logging::log(
                        "HOTKEY",
                        &format!(
                            "{} pressed (notes) - dispatching to main thread",
                            notes_display
                        ),
                    );
                    dispatch_notes_hotkey();
                }
                // Check if it's the AI hotkey - dispatch directly to main thread via GCD
                else if event.id == ai_hotkey_id {
                    logging::log(
                        "HOTKEY",
                        &format!("{} pressed (AI) - dispatching to main thread", ai_display),
                    );
                    dispatch_ai_hotkey();
                }
                // Check if it's a script shortcut
                else if let Some(script_path) = script_hotkey_map.get(&event.id) {
                    logging::log(
                        "HOTKEY",
                        &format!("Script shortcut triggered: {}", script_path),
                    );
                    // Send the script path to be executed
                    if script_hotkey_channel()
                        .0
                        .send_blocking(script_path.clone())
                        .is_err()
                    {
                        logging::log("HOTKEY", "Script hotkey channel closed, cannot send");
                    }
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_channel::TryRecvError;

    #[test]
    fn hotkey_channels_are_independent() {
        while hotkey_channel().1.try_recv().is_ok() {}
        while script_hotkey_channel().1.try_recv().is_ok() {}

        hotkey_channel().0.send_blocking(()).expect("send hotkey");
        assert!(matches!(
            script_hotkey_channel().1.try_recv(),
            Err(TryRecvError::Empty)
        ));
        assert!(hotkey_channel().1.try_recv().is_ok());

        script_hotkey_channel()
            .0
            .send_blocking("script".to_string())
            .expect("send script hotkey");
        assert_eq!(
            script_hotkey_channel()
                .1
                .try_recv()
                .expect("recv script hotkey"),
            "script"
        );
    }

    // =============================================================================
    // ScriptHotkeyManager Unit Tests
    // =============================================================================
    // Note: These tests cannot actually register system hotkeys in the test environment
    // because GlobalHotKeyManager requires a running event loop and proper OS permissions.
    // Instead, we test the logic of the manager's internal tracking.

    mod script_hotkey_manager_tests {
        use super::*;

        /// Helper to create a manager for testing.
        /// Note: Registration will fail without an event loop, but we can test tracking logic.
        fn create_test_manager() -> Option<ScriptHotkeyManager> {
            // GlobalHotKeyManager::new() may fail in test environment
            GlobalHotKeyManager::new()
                .ok()
                .map(ScriptHotkeyManager::new)
        }

        #[test]
        fn test_manager_creation() {
            // Just verify we can create the struct (manager creation may fail in CI)
            if let Some(manager) = create_test_manager() {
                assert!(manager.hotkey_map.is_empty());
                assert!(manager.path_to_id.is_empty());
            }
        }

        #[test]
        fn test_get_registered_hotkeys_empty() {
            if let Some(manager) = create_test_manager() {
                assert!(manager.get_registered_hotkeys().is_empty());
            }
        }

        #[test]
        fn test_is_registered_false_for_unknown_path() {
            if let Some(manager) = create_test_manager() {
                assert!(!manager.is_registered("/some/unknown/path.ts"));
            }
        }

        #[test]
        fn test_unregister_nonexistent_is_noop() {
            if let Some(mut manager) = create_test_manager() {
                // Should not error when unregistering a path that was never registered
                let result = manager.unregister("/nonexistent/path.ts");
                assert!(result.is_ok());
            }
        }

        #[test]
        fn test_update_none_to_none_is_noop() {
            if let Some(mut manager) = create_test_manager() {
                // No old, no new -> no-op, should succeed
                let result = manager.update("/some/path.ts", None, None);
                assert!(result.is_ok());
            }
        }

        // Note: The following tests would require a working GlobalHotKeyManager
        // which may not be available in all test environments.
        // In a real CI environment, these would be integration tests.

        #[test]
        fn test_register_tracks_mapping() {
            if let Some(mut manager) = create_test_manager() {
                // Try to register - this may fail in test environment, that's OK
                let result = manager.register("/test/script.ts", "cmd+shift+t");
                if result.is_ok() {
                    // If registration succeeded, verify tracking
                    assert!(manager.is_registered("/test/script.ts"));
                    let hotkeys = manager.get_registered_hotkeys();
                    assert_eq!(hotkeys.len(), 1);
                    assert_eq!(hotkeys[0].0, "/test/script.ts");
                }
                // If it failed (no event loop), that's expected in test env
            }
        }

        #[test]
        fn test_unregister_removes_tracking() {
            if let Some(mut manager) = create_test_manager() {
                // Try to register first
                if manager.register("/test/script.ts", "cmd+shift+u").is_ok() {
                    assert!(manager.is_registered("/test/script.ts"));

                    // Now unregister
                    let result = manager.unregister("/test/script.ts");
                    assert!(result.is_ok());
                    assert!(!manager.is_registered("/test/script.ts"));
                }
            }
        }

        #[test]
        fn test_update_add_hotkey() {
            if let Some(mut manager) = create_test_manager() {
                // None -> Some = add
                let result = manager.update("/test/add.ts", None, Some("cmd+shift+a"));
                if result.is_ok() {
                    assert!(manager.is_registered("/test/add.ts"));
                }
            }
        }

        #[test]
        fn test_update_remove_hotkey() {
            if let Some(mut manager) = create_test_manager() {
                // First register
                if manager.register("/test/remove.ts", "cmd+shift+r").is_ok() {
                    // Some -> None = remove
                    let result = manager.update("/test/remove.ts", Some("cmd+shift+r"), None);
                    assert!(result.is_ok());
                    assert!(!manager.is_registered("/test/remove.ts"));
                }
            }
        }

        #[test]
        fn test_update_change_hotkey() {
            if let Some(mut manager) = create_test_manager() {
                // First register with old shortcut
                if manager.register("/test/change.ts", "cmd+shift+c").is_ok() {
                    // Some -> Some (different) = change
                    let result =
                        manager.update("/test/change.ts", Some("cmd+shift+c"), Some("cmd+alt+c"));
                    if result.is_ok() {
                        // Should still be registered (with new shortcut)
                        assert!(manager.is_registered("/test/change.ts"));
                    }
                }
            }
        }

        #[test]
        fn test_get_script_path() {
            if let Some(mut manager) = create_test_manager() {
                if let Ok(hotkey_id) = manager.register("/test/lookup.ts", "cmd+shift+l") {
                    let path = manager.get_script_path(hotkey_id);
                    assert_eq!(path, Some(&"/test/lookup.ts".to_string()));

                    // Unknown ID returns None
                    assert!(manager.get_script_path(99999).is_none());
                }
            }
        }
    }
}

</file>

</files>
📊 Pack Summary:
────────────────
  Total Files: 3 files
  Search Mode: ripgrep (fast)
  Total Tokens: ~14.3K (14,253 exact)
  Total Chars: 68,396 chars
       Output: -

📁 Extensions Found:
────────────────────
  .rs

📂 Top 10 Files (by tokens):
──────────────────────────
      7.8K - src/hotkeys.rs
      3.4K - src/keyboard_monitor.rs
      3.1K - src/hotkey_pollers.rs
