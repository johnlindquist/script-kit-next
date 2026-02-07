use super::*;

fn ensure_theme_initialized(cx: &mut App) {
    // Use the shared theme sync function from src/theme/gpui_integration.rs
    crate::theme::sync_gpui_component_theme(cx);
    info!("AI window theme synchronized with Script Kit");
}

/// Toggle the AI window (open if closed, bring to front if open)
///
/// The AI window behaves as a NORMAL window (not a floating panel):
/// - Can go behind other windows when it loses focus
/// - Hotkey brings it to front and focuses it
/// - Does NOT affect other windows (main window, notes window)
/// - Does NOT hide the app when closed
pub fn open_ai_window(cx: &mut App) -> Result<()> {
    use crate::logging;

    logging::log("AI", "open_ai_window called - checking state");

    // Ensure gpui-component theme is initialized before opening window
    ensure_theme_initialized(cx);

    // SAFETY: Release lock BEFORE calling handle.update() to prevent deadlock.
    // WindowHandle is Copy, so we just dereference to get it out.
    let existing_handle = {
        let slot = AI_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
        slot.lock().ok().and_then(|g| *g)
    };

    // Check if window already exists and is valid
    if let Some(handle) = existing_handle {
        // Window exists - check if it's valid (lock is released)
        let window_valid = handle
            .update(cx, |_root, window, _cx| {
                // Window is valid - bring it to front and focus it
                window.activate_window();
            })
            .is_ok();

        if window_valid {
            logging::log("AI", "AI window exists - bringing to front and focusing");

            // Ensure regular app mode (in case it was switched back to accessory)
            crate::platform::set_regular_app_mode();

            // Move the window to the display containing the mouse cursor
            // This ensures the AI window appears on the same screen as where the user is working
            let new_bounds = crate::platform::calculate_centered_bounds_on_mouse_display(size(
                px(900.),
                px(700.),
            ));
            let _ = handle.update(cx, |_root, window, cx| {
                crate::window_ops::queue_move(new_bounds, window, cx);
            });

            // Activate the app to ensure the window can receive focus
            cx.activate(true);

            // Request focus on the input field via the global flag.
            // AiApp checks this flag in render() and focuses if set.
            // This avoids the need for a global Entity<AiApp> reference which caused memory leaks.
            AI_FOCUS_REQUESTED.store(true, std::sync::atomic::Ordering::SeqCst);

            // Notify to trigger re-render which will process the focus request
            let _ = handle.update(cx, |_root, _window, cx| {
                cx.notify();
            });

            return Ok(());
        }

        // Window handle was invalid, clear it
        logging::log("AI", "AI window handle was invalid - creating new");
        let slot = AI_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
        if let Ok(mut g) = slot.lock() {
            *g = None;
        }
    }

    // Create new window
    logging::log("AI", "Creating new AI window");
    info!("Opening new AI window");

    // Load theme to determine window background appearance (vibrancy)
    let theme = crate::theme::load_theme();
    let window_background = if theme.is_vibrancy_enabled() {
        gpui::WindowBackgroundAppearance::Blurred
    } else {
        gpui::WindowBackgroundAppearance::Opaque
    };

    // Calculate position: try per-display saved position first, then centered on mouse display
    // Use mouse display positioning so AI window appears on the same screen as the cursor
    let displays = crate::platform::get_macos_displays();
    let bounds = if let Some((mouse_x, mouse_y)) = crate::platform::get_global_mouse_position() {
        if let Some((saved, _display)) =
            crate::window_state::get_ai_position_for_mouse_display(mouse_x, mouse_y, &displays)
        {
            // Use saved per-display position
            saved.to_gpui().get_bounds()
        } else {
            // Fall back to centered on mouse display
            crate::platform::calculate_centered_bounds_on_mouse_display(size(px(900.), px(700.)))
        }
    } else {
        // Mouse position unavailable, fall back to centered
        crate::platform::calculate_centered_bounds_on_mouse_display(size(px(900.), px(700.)))
    };

    let window_options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(gpui::TitlebarOptions {
            title: Some("Script Kit AI".into()),
            appears_transparent: true,
            ..Default::default()
        }),
        window_background,
        focus: true,
        show: true,
        // IMPORTANT: Use Normal window kind (not PopUp) so it behaves like a regular window
        // This allows it to go behind other windows and participate in normal window ordering
        kind: gpui::WindowKind::Normal,
        ..Default::default()
    };

    // Create a holder for the AiApp entity so we can focus it after window creation.
    // NOTE: This is a LOCAL holder, not stored globally, to avoid memory leaks.
    let ai_app_holder: std::sync::Arc<std::sync::Mutex<Option<Entity<AiApp>>>> =
        std::sync::Arc::new(std::sync::Mutex::new(None));
    let ai_app_holder_clone = ai_app_holder.clone();

    let handle = cx.open_window(window_options, |window, cx| {
        let view = cx.new(|cx| AiApp::new(window, cx));
        // Store the AiApp entity temporarily for immediate focus after window creation
        *ai_app_holder_clone
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some(view.clone());
        cx.new(|cx| Root::new(view, window, cx))
    })?;

    // Activate the app and window so user can immediately start typing
    cx.activate(true);
    let _ = handle.update(cx, |_root, window, _cx| {
        window.activate_window();
    });

    // Focus the input field immediately after window creation
    // Use the local entity reference (not stored globally to avoid leaks)
    if let Some(ai_app) = ai_app_holder.lock().ok().and_then(|mut h| h.take()) {
        let _ = handle.update(cx, |_root, window, cx| {
            ai_app.update(cx, |app, cx| {
                app.focus_input(window, cx);
            });
        });
    }

    // Store the window handle (release lock immediately after)
    {
        let slot = AI_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
        if let Ok(mut g) = slot.lock() {
            *g = Some(handle);
        }
    }

    // Switch to regular app mode so AI window appears in Cmd+Tab
    // This is unique to the AI window - other windows (main, notes) stay in accessory mode
    // The mode is restored to accessory when the AI window closes (see AiApp::drop)
    crate::platform::set_regular_app_mode();

    // NOTE: We do NOT configure as floating panel - this is a normal window
    // that can go behind other windows
    // However, we DO want vibrancy configuration for proper blur effect
    configure_ai_window_vibrancy();

    // NOTE: Theme hot-reload is now handled by the centralized ThemeService
    // (crate::theme::service::ensure_theme_service) which is started once at app init.
    // This eliminates per-window theme watcher tasks and their potential for leaks.

    Ok(())
}

/// Pending chat to initialize after window opens.
/// This is used by open_ai_window_with_chat to pass messages to the newly created window.
#[allow(clippy::type_complexity)]
static AI_PENDING_CHAT: std::sync::OnceLock<std::sync::Mutex<Option<Vec<(MessageRole, String)>>>> =
    std::sync::OnceLock::new();

pub(super) fn get_pending_chat() -> &'static std::sync::Mutex<Option<Vec<(MessageRole, String)>>> {
    AI_PENDING_CHAT.get_or_init(|| std::sync::Mutex::new(None))
}

/// Open the AI window with an existing conversation.
///
/// This function:
/// 1. Opens the AI window (or brings it to front if already open)
/// 2. Creates a new chat with the provided messages
/// 3. Displays the chat immediately
///
/// Use this for "Continue in Chat" functionality to transfer a conversation
/// from the chat prompt to the AI window.
pub fn open_ai_window_with_chat(cx: &mut App, messages: Vec<(MessageRole, String)>) -> Result<()> {
    use crate::logging;

    logging::log(
        "AI",
        &format!(
            "open_ai_window_with_chat called with {} messages",
            messages.len()
        ),
    );

    // Store the pending chat messages
    if let Ok(mut pending) = get_pending_chat().lock() {
        *pending = Some(messages);
    }

    // Open or bring the window to front
    open_ai_window(cx)?;

    // Queue a command to initialize the chat with pending messages
    push_ai_command(AiCommand::InitializeWithPendingChat);

    // Notify the window to process the command
    let handle = {
        let slot = AI_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
        slot.lock().ok().and_then(|g| *g)
    };

    if let Some(handle) = handle {
        let _ = handle.update(cx, |_root, _window, cx| {
            cx.notify();
        });
    }

    Ok(())
}

/// Close the AI window
pub fn close_ai_window(cx: &mut App) {
    // SAFETY: Release lock BEFORE calling handle.update() to prevent deadlock
    // If handle.update() causes Drop to fire synchronously and tries to acquire
    // the same lock, we would deadlock. Taking the handle out first avoids this.
    let handle = {
        let slot = AI_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
        slot.lock().ok().and_then(|mut g| g.take())
    };

    if let Some(handle) = handle {
        let _ = handle.update(cx, |_, window, _| {
            // Save window bounds per-display before closing
            let wb = window.window_bounds();
            let persisted = crate::window_state::PersistedWindowBounds::from_gpui(wb);
            let displays = crate::platform::get_macos_displays();
            // Find which display the window center is on
            if let Some(display) =
                crate::window_state::find_display_for_bounds(&persisted, &displays)
            {
                crate::window_state::save_ai_position_for_display(display, persisted);
            } else {
                // Fallback to legacy save if display not found
                crate::window_state::save_window_from_gpui(crate::window_state::WindowRole::Ai, wb);
            }
            window.remove_window();
        });
    }

    // Clear the focus request flag (no longer needed after window closes)
    AI_FOCUS_REQUESTED.store(false, std::sync::atomic::Ordering::SeqCst);
}

/// Check if the AI window is currently open
///
/// Returns true if the AI window exists and is valid.
/// This is used by other parts of the app to check if AI is open
/// without affecting it.
pub fn is_ai_window_open() -> bool {
    let window_handle = AI_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
    let guard = window_handle.lock().unwrap_or_else(|e| e.into_inner());
    guard.is_some()
}

/// Check if the given window handle matches the AI window
///
/// Returns true if the window is the AI window.
/// Used by keystroke interceptors to avoid handling keys meant for AI.
pub fn is_ai_window(window: &gpui::Window) -> bool {
    let window_handle = AI_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
    if let Ok(guard) = window_handle.lock() {
        if let Some(ai_handle) = guard.as_ref() {
            // Convert WindowHandle<Root> to AnyWindowHandle via Into trait
            let ai_any: gpui::AnyWindowHandle = (*ai_handle).into();
            return window.window_handle() == ai_any;
        }
    }
    false
}

/// Set the search filter text in the AI window.
/// Used for testing the search functionality via stdin commands.
pub fn set_ai_search(cx: &mut App, query: &str) {
    use crate::logging;

    let handle = {
        let slot = AI_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
        slot.lock().ok().and_then(|g| *g)
    };
    let window_is_open = handle.is_some();
    let command_queued = get_pending_commands()
        .lock()
        .ok()
        .map(|mut commands| {
            ai_window_queue_command_if_open(
                &mut commands,
                window_is_open,
                AiCommand::SetSearch(query.to_string()),
            )
        })
        .unwrap_or(false);

    if !command_queued {
        logging::log("AI", "Cannot set search - AI window not found");
        return;
    }

    if let Some(handle) = handle {
        let _ = handle.update(cx, |_root, _window, cx| {
            cx.notify();
        });
        logging::log("AI", &format!("Set AI search filter: {}", query));
    }
}

/// Set the main input text in the AI window and optionally submit.
/// Used for testing the streaming functionality via stdin commands.
pub fn set_ai_input(cx: &mut App, text: &str, submit: bool) {
    use crate::logging;

    let handle = {
        let slot = AI_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
        slot.lock().ok().and_then(|g| *g)
    };
    let window_is_open = handle.is_some();
    let command_queued = get_pending_commands()
        .lock()
        .ok()
        .map(|mut commands| {
            ai_window_queue_command_if_open(
                &mut commands,
                window_is_open,
                AiCommand::SetInput {
                    text: text.to_string(),
                    submit,
                },
            )
        })
        .unwrap_or(false);

    if !command_queued {
        logging::log("AI", "Cannot set input - AI window not open");
        return;
    }

    if let Some(handle) = handle {
        let _ = handle.update(cx, |_root, _window, cx| {
            cx.notify();
        });
    }
}

/// Set the main input text with an attached image in the AI window and optionally submit.
/// The image should be base64 encoded PNG data.
/// Used by AI commands like "Send Screen to AI Chat".
pub fn set_ai_input_with_image(cx: &mut App, text: &str, image_base64: &str, submit: bool) {
    use crate::logging;

    let handle = {
        let slot = AI_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
        slot.lock().ok().and_then(|g| *g)
    };
    let window_is_open = handle.is_some();
    let command_queued = get_pending_commands()
        .lock()
        .ok()
        .map(|mut commands| {
            ai_window_queue_command_if_open(
                &mut commands,
                window_is_open,
                AiCommand::SetInputWithImage {
                    text: text.to_string(),
                    image_base64: image_base64.to_string(),
                    submit,
                },
            )
        })
        .unwrap_or(false);

    if !command_queued {
        logging::log("AI", "Cannot set input with image - AI window not open");
        return;
    }

    if let Some(handle) = handle {
        let _ = handle.update(cx, |_root, _window, cx| {
            cx.notify();
        });
    }
}

/// Show the AI command bar (Cmd+K menu) in the AI window.
///
/// This is triggered by the stdin command `{"type":"showAiCommandBar"}`.
/// Opens the AI window if not already open, then shows the command bar overlay.
pub fn show_ai_command_bar(cx: &mut App) {
    use crate::logging;

    // First ensure the AI window is open
    if !is_ai_window_open() {
        if let Err(e) = open_ai_window(cx) {
            logging::log("AI", &format!("Failed to open AI window: {}", e));
            return;
        }
    }

    // Queue the command and notify the window to process it in render()
    // This avoids the need for direct entity access which caused memory leaks.
    push_ai_command(AiCommand::ShowCommandBar);

    // Notify the window to process the command
    let handle = {
        let slot = AI_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
        slot.lock().ok().and_then(|g| *g)
    };

    if let Some(handle) = handle {
        let _ = handle.update(cx, |_root, _window, cx| {
            cx.notify();
        });
        logging::log("AI", "Showing AI command bar");
    } else {
        logging::log("AI", "Cannot show command bar - AI window handle not found");
    }
}

/// Simulate a key press in the AI window.
///
/// This is triggered by the stdin command `{"type":"simulateAiKey","key":"up","modifiers":["cmd"]}`.
/// Used for testing keyboard navigation in the AI window, especially the command bar.
pub fn simulate_ai_key(key: &str, modifiers: Vec<KeyModifier>) {
    use crate::logging;

    // Check if AI window is open
    if !is_ai_window_open() {
        logging::log("AI", "Cannot simulate key - AI window not open");
        return;
    }

    // Queue the command
    push_ai_command(AiCommand::SimulateKey {
        key: key.to_string(),
        modifiers,
    });

    // The command is queued and will be processed in the next render cycle
    // We don't have `cx` here so can't notify, but that's okay since rendering happens continuously
    logging::log(
        "AI",
        &format!(
            "Queued SimulateKey: key='{}' - will process on next render",
            key
        ),
    );
}
