/// Show a HUD notification
///
/// This creates a new floating window positioned at the bottom-center of the
/// screen containing the mouse cursor. The HUD auto-dismisses after the
/// specified duration.
///
/// # Arguments
/// * `text` - The message to display
/// * `duration_ms` - Optional duration in milliseconds (default: 2000ms)
/// * `cx` - GPUI App context
pub fn show_hud(text: String, duration_ms: Option<u64>, cx: &mut App) {
    let duration = duration_ms.unwrap_or(DEFAULT_HUD_DURATION_MS);

    logging::log(
        "HUD",
        &format!("Showing HUD: '{}' for {}ms", text, duration),
    );

    // Allocate slot and check if we can show immediately
    let allocated_slot = {
        let manager = get_hud_manager();
        let state = manager.lock();
        state.first_free_slot()
    };

    let slot = match allocated_slot {
        Some(s) => s,
        None => {
            // No free slots, queue the HUD
            logging::log("HUD", "Max HUDs reached, queueing");
            let manager = get_hud_manager();
            let mut state = manager.lock();
            state.pending_queue.push_back(HudNotification {
                text,
                duration_ms: duration,
                created_at: Instant::now(),
                action_label: None,
                action: None,
            });
            return;
        }
    };

    // Calculate position - bottom center of screen with mouse
    let (hud_x, hud_y) = calculate_hud_position(cx);

    // Calculate vertical offset using SLOT index (not len) - this prevents overlap
    let stack_offset = slot as f32 * HUD_STACK_GAP;

    let hud_width: Pixels = px(HUD_WIDTH);
    let hud_height: Pixels = px(HUD_HEIGHT);

    let bounds = gpui::Bounds {
        origin: point(px(hud_x), px(hud_y - stack_offset)),
        size: size(hud_width, hud_height),
    };

    let text_for_log = text.clone();

    // Create the HUD window with specific options for overlay behavior
    let window_result = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: None,
            is_movable: false,
            window_background: WindowBackgroundAppearance::Transparent,
            focus: false, // Don't steal focus
            show: true,   // Show immediately
            ..Default::default()
        },
        |_, cx| cx.new(|_| HudView::new(text)),
    );

    match window_result {
        Ok(window_handle) => {
            // Configure the window as a floating overlay using size-based matching
            // Regular HUDs without actions are click-through (true)
            configure_hud_window_by_size(HUD_WIDTH, HUD_HEIGHT, true);

            // Generate unique ID for this HUD
            let hud_id = next_hud_id();

            // Track the active HUD and register slot
            {
                let manager = get_hud_manager();
                let mut state = manager.lock();
                // Register slot ownership
                state.hud_slots[slot] = Some(HudSlotEntry { id: hud_id });
                state.active_huds.push(ActiveHud {
                    id: hud_id,
                    window: window_handle,
                    created_at: Instant::now(),
                    duration_ms: duration,
                    slot,
                });
            }

            // Schedule cleanup after duration - use ID for dismissal
            let duration_duration = Duration::from_millis(duration);
            cx.spawn(async move |cx: &mut gpui::AsyncApp| {
                Timer::after(duration_duration).await;

                // IMPORTANT: All AppKit calls must happen on the main thread.
                // cx.update() ensures we're on the main thread.
                let _ = cx.update(|cx| {
                    // Dismiss by ID using GPUI's WindowHandle API
                    dismiss_hud_by_id(hud_id, cx);
                });
            })
            .detach();

            logging::log(
                "HUD",
                &format!("HUD window created for: '{}' (slot {})", text_for_log, slot),
            );
        }
        Err(e) => {
            logging::log("HUD", &format!("Failed to create HUD window: {:?}", e));
        }
    }
}
/// Show a HUD notification with a clickable action button
///
/// This creates a HUD with a button that executes an action when clicked.
/// The HUD is wider to accommodate the button.
///
/// # Arguments
/// * `text` - The message to display
/// * `duration_ms` - Optional duration in milliseconds (default: 3000ms for action HUDs)
/// * `action_label` - Label for the action button (e.g., "Open Logs")
/// * `action` - The action to execute when the button is clicked
/// * `cx` - GPUI App context
#[allow(dead_code)]
pub fn show_hud_with_action(
    text: String,
    duration_ms: Option<u64>,
    action_label: String,
    action: HudAction,
    cx: &mut App,
) {
    // Action HUDs have longer default duration (3s) since user might click
    let duration = duration_ms.unwrap_or(3000);

    logging::log(
        "HUD",
        &format!(
            "Showing HUD with action: '{}' [{}] for {}ms",
            text, action_label, duration
        ),
    );

    // Allocate slot and check if we can show immediately
    let allocated_slot = {
        let manager = get_hud_manager();
        let state = manager.lock();
        state.first_free_slot()
    };

    let slot = match allocated_slot {
        Some(s) => s,
        None => {
            // No free slots, queue the HUD
            logging::log("HUD", "Max HUDs reached, queueing action HUD");
            let manager = get_hud_manager();
            let mut state = manager.lock();
            state.pending_queue.push_back(HudNotification {
                text,
                duration_ms: duration,
                created_at: Instant::now(),
                action_label: Some(action_label),
                action: Some(action),
            });
            return;
        }
    };

    // Calculate position - bottom center of screen with mouse
    let (hud_x, hud_y) = calculate_hud_position(cx);

    // Calculate vertical offset using SLOT index (not len) - this prevents overlap
    let stack_offset = slot as f32 * HUD_STACK_GAP;

    // Use wider dimensions for action HUDs
    let hud_width: Pixels = px(HUD_ACTION_WIDTH);
    let hud_height: Pixels = px(HUD_ACTION_HEIGHT);

    // Adjust x position for wider HUD
    let adjusted_x = hud_x - (HUD_ACTION_WIDTH - HUD_WIDTH) / 2.0;

    let bounds = gpui::Bounds {
        origin: point(px(adjusted_x), px(hud_y - stack_offset)),
        size: size(hud_width, hud_height),
    };

    let text_for_log = text.clone();

    // Create the HUD window with action button
    let window_result = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: None,
            is_movable: false,
            window_background: WindowBackgroundAppearance::Transparent,
            focus: false, // Don't steal focus
            show: true,   // Show immediately
            ..Default::default()
        },
        |_, cx| cx.new(|_| HudView::with_action(text, action_label, action)),
    );

    match window_result {
        Ok(window_handle) => {
            // Configure the window as a floating overlay using size-based matching
            // Action HUDs need to receive mouse events for button clicks (click_through = false)
            configure_hud_window_by_size(HUD_ACTION_WIDTH, HUD_ACTION_HEIGHT, false);

            // Generate unique ID for this HUD
            let hud_id = next_hud_id();

            // Track the active HUD and register slot
            {
                let manager = get_hud_manager();
                let mut state = manager.lock();
                // Register slot ownership
                state.hud_slots[slot] = Some(HudSlotEntry { id: hud_id });
                state.active_huds.push(ActiveHud {
                    id: hud_id,
                    window: window_handle,
                    created_at: Instant::now(),
                    duration_ms: duration,
                    slot,
                });
            }

            // Schedule cleanup after duration - use ID for dismissal
            let duration_duration = Duration::from_millis(duration);
            cx.spawn(async move |cx: &mut gpui::AsyncApp| {
                Timer::after(duration_duration).await;

                // IMPORTANT: All AppKit calls must happen on the main thread.
                // cx.update() ensures we're on the main thread.
                let _ = cx.update(|cx| {
                    // Dismiss by ID using GPUI's WindowHandle API
                    dismiss_hud_by_id(hud_id, cx);
                });
            })
            .detach();

            logging::log(
                "HUD",
                &format!(
                    "Action HUD window created for: '{}' (slot {})",
                    text_for_log, slot
                ),
            );
        }
        Err(e) => {
            logging::log(
                "HUD",
                &format!("Failed to create action HUD window: {:?}", e),
            );
        }
    }
}
/// Calculate HUD position - bottom center of screen containing mouse
fn calculate_hud_position(cx: &App) -> (f32, f32) {
    let displays = cx.displays();

    // Try to get mouse position
    let mouse_pos = crate::platform::get_global_mouse_position();

    // Find display containing mouse
    let target_display = if let Some((mouse_x, mouse_y)) = mouse_pos {
        displays.iter().find(|display| {
            let bounds = display.bounds();
            let x: f64 = bounds.origin.x.into();
            let y: f64 = bounds.origin.y.into();
            let w: f64 = bounds.size.width.into();
            let h: f64 = bounds.size.height.into();

            mouse_x >= x && mouse_x < x + w && mouse_y >= y && mouse_y < y + h
        })
    } else {
        None
    };

    // Use found display or primary
    let display = target_display.or_else(|| displays.first());

    if let Some(display) = display {
        let bounds = display.bounds();
        let screen_x: f32 = bounds.origin.x.into();
        let screen_y: f32 = bounds.origin.y.into();
        let screen_width: f32 = bounds.size.width.into();
        let screen_height: f32 = bounds.size.height.into();

        // Center horizontally, position at 85% down the screen
        let hud_x = screen_x + (screen_width - HUD_WIDTH) / 2.0;
        let hud_y = screen_y + screen_height * 0.85;

        (hud_x, hud_y)
    } else {
        // Fallback position
        (500.0, 800.0)
    }
}
/// Configure a HUD window by finding it based on its expected size
///
/// Since HUD windows have unique sizes (200x36 for regular, 300x40 for action),
/// we can reliably find the most recently created window with matching dimensions.
/// This is more reliable than bounds matching since coordinate systems vary.
///
/// # Arguments
/// * `expected_width` - The expected width of the HUD window
/// * `expected_height` - The expected height of the HUD window
/// * `click_through` - If true, window ignores mouse events (for plain HUDs).
///   If false, window receives mouse events (for action HUDs with buttons).
#[cfg(target_os = "macos")]
fn configure_hud_window_by_size(expected_width: f32, expected_height: f32, click_through: bool) {
    use cocoa::appkit::NSApp;
    use cocoa::base::{id, nil};
    use cocoa::foundation::NSRect;

    unsafe {
        let app: id = NSApp();
        let windows: id = msg_send![app, windows];
        let count: usize = msg_send![windows, count];

        // Find a window with matching dimensions (search from most recent)
        for i in 0..count {
            let window: id = msg_send![windows, objectAtIndex: i];
            let frame: NSRect = msg_send![window, frame];

            // Check if this looks like our HUD window by size
            let width_matches = (frame.size.width - expected_width as f64).abs() < 5.0;
            let height_matches = (frame.size.height - expected_height as f64).abs() < 5.0;

            if width_matches && height_matches {
                // Found it! Configure as HUD overlay

                // Set window level very high (NSPopUpMenuWindowLevel = 101)
                let hud_level: i64 = 101;
                let _: () = msg_send![window, setLevel: hud_level];

                // Collection behaviors for HUD:
                // - CanJoinAllSpaces (1): appear on all spaces
                // - Stationary (16): don't move with spaces
                // - IgnoresCycle (64): cmd-tab ignores this window
                let collection_behavior: u64 = 1 | 16 | 64;
                let _: () = msg_send![window, setCollectionBehavior: collection_behavior];

                // Set mouse event handling based on whether HUD has clickable actions
                let ignores_mouse: cocoa::base::BOOL = if click_through {
                    cocoa::base::YES
                } else {
                    cocoa::base::NO
                };
                let _: () = msg_send![window, setIgnoresMouseEvents: ignores_mouse];

                // Don't show in window menu
                let _: () = msg_send![window, setExcludedFromWindowsMenu: true];

                // Order to front without activating the app
                let _: () = msg_send![window, orderFront: nil];

                let click_status = if click_through {
                    "click-through"
                } else {
                    "clickable"
                };
                logging::log(
                    "HUD",
                    &format!(
                        "Configured HUD NSWindow ({}x{}): level={}, {}, orderFront",
                        expected_width, expected_height, hud_level, click_status
                    ),
                );
                return;
            }
        }

        logging::log(
            "HUD",
            &format!(
                "Could not find HUD window with size {}x{}",
                expected_width, expected_height
            ),
        );
    }
}
#[cfg(not(target_os = "macos"))]
fn configure_hud_window_by_size(_expected_width: f32, _expected_height: f32, _click_through: bool) {
    logging::log(
        "HUD",
        "Non-macOS platform, skipping HUD window configuration",
    );
}
/// Dismiss a specific HUD by its ID
///
/// Uses WindowHandle.update() + window.remove_window() for reliable window closing.
/// Uses slot-based clearing instead of swap_remove to prevent position overlap.
fn dismiss_hud_by_id(hud_id: u64, cx: &mut App) {
    let manager = get_hud_manager();

    // Find and remove the HUD with matching ID, getting its window handle for closing
    let window_to_close: Option<WindowHandle<HudView>> = {
        let mut state = manager.lock();

        // First, release the slot (this is the key fix - clears by ID, not swap_remove)
        state.release_slot_by_id(hud_id);

        // Then find and remove from active_huds Vec (retain order, don't swap_remove)
        if let Some(idx) = state.active_huds.iter().position(|h| h.id == hud_id) {
            let hud = state.active_huds.remove(idx); // Use remove() to preserve order
            Some(hud.window)
        } else {
            None
        }
    };

    // Close the window using GPUI's proper API
    if let Some(window_handle) = window_to_close {
        // Use WindowHandle.update() to access window.remove_window()
        let result = window_handle.update(cx, |_view, window, _cx| {
            window.remove_window();
        });

        match result {
            Ok(()) => {
                logging::log("HUD", &format!("Dismissed HUD id={}", hud_id));
            }
            Err(e) => {
                // Window may have already been closed (e.g., by user)
                logging::log(
                    "HUD",
                    &format!("HUD id={} window already closed: {}", hud_id, e),
                );
            }
        }

        // Show any pending HUDs
        cleanup_expired_huds(cx);
    } else {
        // HUD was already dismissed (possibly manually) - this is OK
        logging::log(
            "HUD",
            &format!("HUD id={} already dismissed, skipping", hud_id),
        );
    }
}
