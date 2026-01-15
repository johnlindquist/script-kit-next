use gpui::{px, size, App, AppContext as _, AsyncApp, Context, Focusable, Window, WindowHandle};

use crate::ai;
use crate::hotkeys;
use crate::notes;
use crate::platform::calculate_eye_line_bounds_on_mouse_display;
use crate::window_ops;
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
                    // Calculate bounds and do GPUI operations synchronously.
                    // Window move is deferred via window_ops::queue_move() which uses Window::defer
                    // to avoid RefCell borrow conflicts during GPUI's update cycle.
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

                        // Step 2: Show window WITHOUT activating the app
                        // This is critical for floating panels - we want to show and focus
                        // the window without stealing focus from the previous app.
                        // This allows tools like "copy selected text" to still work.
                        platform::show_main_window_without_activation();
                        logging::log("HOTKEY", "Window shown without app activation");

                        // Step 2.5: Configure as floating panel on first show only
                        if !PANEL_CONFIGURED.swap(true, std::sync::atomic::Ordering::SeqCst) {
                            platform::configure_as_floating_panel();
                            logging::log("HOTKEY", "Configured window as floating panel (first show)");
                        }

                        // Step 2.6: Send AI window to back so it doesn't come forward with main menu
                        // The AI window should only come forward via Cmd+Tab or explicit user action
                        platform::send_ai_window_to_back();

                        // Step 3: Activate the specific window, focus it, and queue deferred move
                        let _ = window_clone.update(
                            cx,
                            |view: &mut ScriptListApp, win: &mut Window, ctx: &mut Context<ScriptListApp>| {
                                win.activate_window();
                                let focus_handle = view.focus_handle(ctx);
                                win.focus(&focus_handle, ctx);
                                logging::log("HOTKEY", "Window activated and focused");

                                // Menu bar items are now tracked by frontmost_app_tracker
                                // No state reset needed here

                                // Step 4: Check if we need to reset to script list (after script completion)
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
                                    view.reset_to_script_list(ctx);
                                }

                                // Step 5: Queue window move via Window::defer (replaces Timer::after pattern)
                                // This defers the native macOS setFrame call to end of effect cycle,
                                // avoiding RefCell borrow conflicts during GPUI's update.
                                window_ops::queue_move(new_bounds, win, ctx);
                                logging::log("HOTKEY", "Window move queued via Window::defer");
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

            while let Ok(command_id) = hotkeys::script_hotkey_channel().1.recv().await {
                logging::log(
                    "HOTKEY",
                    &format!("Script shortcut received: {}", command_id),
                );

                let id_clone = command_id.clone();
                let _ = cx.update(move |cx: &mut App| {
                    // Execute command and check if main window should be shown
                    let should_show = window
                        .update(
                            cx,
                            |view: &mut ScriptListApp,
                             _win: &mut Window,
                             ctx: &mut Context<ScriptListApp>| {
                                // Handle both file paths (legacy) and command IDs (new format)
                                // Returns whether main window should be shown
                                view.execute_by_command_id_or_path(&id_clone, ctx)
                            },
                        )
                        .unwrap_or(true); // Default to showing window on error

                    // Only show window if command needs it (apps/ai/notes don't)
                    if should_show && !script_kit_gpui::is_main_window_visible() {
                        logging::log("HOTKEY", "Command needs main window, showing it");
                        // Show WITHOUT activating (floating panel behavior)
                        // This preserves focus on previous app for features like "copy selected text"
                        platform::show_main_window_without_activation();
                        // Send AI window to back so it doesn't come forward
                        platform::send_ai_window_to_back();
                    } else if !should_show {
                        logging::log("HOTKEY", "Command doesn't need main window (app/ai/notes)");
                    }
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
