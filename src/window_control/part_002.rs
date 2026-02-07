/// Tile a window to a predefined position on the screen.
///
/// # Arguments
/// * `window_id` - The unique window identifier from `list_windows()`
/// * `position` - The tiling position (half, quadrant, or fullscreen)
///
/// # Errors
/// Returns error if window not found or operation fails.
///
#[instrument]
pub fn tile_window(window_id: u32, position: TilePosition) -> Result<()> {
    let window = get_cached_window(window_id)
        .or_else(|| {
            let _ = list_windows();
            get_cached_window(window_id)
        })
        .context("Window not found")?;

    // Get current position to determine which display the window is on
    let (current_x, current_y) = get_window_position(window.as_ptr()).unwrap_or((0, 0));

    // Get the visible display bounds (accounting for menu bar and dock)
    let display = get_visible_display_bounds(current_x, current_y);

    let bounds = calculate_tile_bounds(&display, position);

    set_window_position(window.as_ptr(), bounds.x, bounds.y)?;
    set_window_size(window.as_ptr(), bounds.width, bounds.height)?;

    info!(window_id, ?position, "Tiled window");
    Ok(())
}
/// Close a window.
///
/// Note: This performs the close action on the window, which may prompt
/// the user to save unsaved changes depending on the application.
///
/// # Arguments
/// * `window_id` - The unique window identifier from `list_windows()`
///
/// # Errors
/// Returns error if window not found or operation fails.
#[instrument]
pub fn close_window(window_id: u32) -> Result<()> {
    let window = get_cached_window(window_id)
        .or_else(|| {
            let _ = list_windows();
            get_cached_window(window_id)
        })
        .context("Window not found")?;

    // Get the close button and press it
    if let Ok(close_button) = get_ax_attribute(window.as_ptr(), "AXCloseButton") {
        perform_ax_action(close_button as AXUIElementRef, "AXPress")?;
        cf_release(close_button);
    } else {
        bail!("Window does not have a close button");
    }

    info!(window_id, "Closed window");
    Ok(())
}
/// Focus (bring to front) a window.
///
/// # Arguments
/// * `window_id` - The unique window identifier from `list_windows()`
///
/// # Errors
/// Returns error if window not found or operation fails.
#[instrument]
pub fn focus_window(window_id: u32) -> Result<()> {
    let window = get_cached_window(window_id)
        .or_else(|| {
            let _ = list_windows();
            get_cached_window(window_id)
        })
        .context("Window not found")?;

    // Raise the window
    perform_ax_action(window.as_ptr(), "AXRaise")?;

    // Also activate the owning application
    let pid = (window_id >> 16) as i32;

    unsafe {
        use objc::runtime::{Class, Object};
        use objc::{msg_send, sel, sel_impl};

        let workspace_class = Class::get("NSWorkspace").context("Failed to get NSWorkspace")?;
        let workspace: *mut Object = msg_send![workspace_class, sharedWorkspace];
        let running_apps: *mut Object = msg_send![workspace, runningApplications];
        let app_count: usize = msg_send![running_apps, count];

        for i in 0..app_count {
            let app: *mut Object = msg_send![running_apps, objectAtIndex: i];
            let app_pid: i32 = msg_send![app, processIdentifier];

            if app_pid == pid {
                let _: bool = msg_send![app, activateWithOptions: 1u64]; // NSApplicationActivateIgnoringOtherApps
                break;
            }
        }
    }

    info!(window_id, "Focused window");
    Ok(())
}
/// Move a window to the next display (cycles through available displays).
#[instrument]
pub fn move_to_next_display(window_id: u32) -> Result<()> {
    move_to_adjacent_display(window_id, true)
}
/// Move a window to the previous display (cycles through available displays).
#[instrument]
pub fn move_to_previous_display(window_id: u32) -> Result<()> {
    move_to_adjacent_display(window_id, false)
}
/// Internal helper to move window to adjacent display
fn move_to_adjacent_display(window_id: u32, next: bool) -> Result<()> {
    let window = get_cached_window(window_id)
        .or_else(|| {
            let _ = list_windows();
            get_cached_window(window_id)
        })
        .context("Window not found")?;

    let (current_x, current_y) = get_window_position(window.as_ptr()).unwrap_or((0, 0));
    let (current_width, current_height) = get_window_size(window.as_ptr()).unwrap_or((800, 600));

    let displays = get_all_display_bounds()?;
    if displays.len() <= 1 {
        info!(window_id, "Only one display, cannot move to adjacent");
        return Ok(());
    }

    let current_display_idx = displays
        .iter()
        .position(|d| {
            current_x >= d.x
                && current_x < d.x + d.width as i32
                && current_y >= d.y
                && current_y < d.y + d.height as i32
        })
        .unwrap_or(0);

    let target_idx = if next {
        (current_display_idx + 1) % displays.len()
    } else if current_display_idx == 0 {
        displays.len() - 1
    } else {
        current_display_idx - 1
    };

    let current_display = &displays[current_display_idx];
    let target_display = &displays[target_idx];

    let rel_x = (current_x - current_display.x) as f64 / current_display.width as f64;
    let rel_y = (current_y - current_display.y) as f64 / current_display.height as f64;

    let new_x = target_display.x + (rel_x * target_display.width as f64) as i32;
    let new_y = target_display.y + (rel_y * target_display.height as f64) as i32;

    let scale_x = target_display.width as f64 / current_display.width as f64;
    let scale_y = target_display.height as f64 / current_display.height as f64;
    let new_width = (current_width as f64 * scale_x).min(target_display.width as f64) as u32;
    let new_height = (current_height as f64 * scale_y).min(target_display.height as f64) as u32;

    set_window_position(window.as_ptr(), new_x, new_y)?;
    set_window_size(window.as_ptr(), new_width, new_height)?;

    info!(
        window_id,
        from_display = current_display_idx,
        to_display = target_idx,
        "Moved window to {} display",
        if next { "next" } else { "previous" }
    );
    Ok(())
}
/// Get bounds for all available displays
fn get_all_display_bounds() -> Result<Vec<Bounds>> {
    let mut displays = Vec::new();

    unsafe {
        use objc::runtime::{Class, Object};
        use objc::{msg_send, sel, sel_impl};

        let nsscreen_class = Class::get("NSScreen").context("Failed to get NSScreen class")?;
        let screens: *mut Object = msg_send![nsscreen_class, screens];
        if screens.is_null() {
            bail!("Failed to get screens");
        }

        let screen_count: usize = msg_send![screens, count];
        if screen_count == 0 {
            bail!("No screens found");
        }

        let primary_screen: *mut Object = msg_send![screens, objectAtIndex: 0usize];
        let primary_frame: CGRect = msg_send![primary_screen, frame];
        let primary_height = primary_frame.size.height;

        for i in 0..screen_count {
            let screen: *mut Object = msg_send![screens, objectAtIndex: i];
            if screen.is_null() {
                continue;
            }

            let visible_frame: CGRect = msg_send![screen, visibleFrame];
            let cg_y = primary_height - (visible_frame.origin.y + visible_frame.size.height);

            displays.push(Bounds {
                x: visible_frame.origin.x as i32,
                y: cg_y as i32,
                width: visible_frame.size.width as u32,
                height: visible_frame.size.height as u32,
            });
        }
    }

    Ok(displays)
}
// ============================================================================
// Helper Functions for Display Bounds
// ============================================================================

/// Get the visible display bounds (excluding menu bar and dock) for the display
/// containing the given point.
///
/// Uses NSScreen.visibleFrame to get accurate bounds that account for:
/// - Menu bar (on main display)
/// - Dock (on any edge, any display)
/// - Notch area (on newer MacBooks)
fn get_visible_display_bounds(x: i32, y: i32) -> Bounds {
    // Use NSScreen to get accurate visible frame
    unsafe {
        use objc::runtime::{Class, Object};
        use objc::{msg_send, sel, sel_impl};

        let nsscreen_class = match Class::get("NSScreen") {
            Some(c) => c,
            None => return get_visible_display_bounds_fallback(x, y),
        };

        // Get all screens
        let screens: *mut Object = msg_send![nsscreen_class, screens];
        if screens.is_null() {
            return get_visible_display_bounds_fallback(x, y);
        }

        let screen_count: usize = msg_send![screens, count];
        if screen_count == 0 {
            return get_visible_display_bounds_fallback(x, y);
        }

        // Get the PRIMARY screen height for coordinate conversion (do this once outside the loop)
        // IMPORTANT: We MUST use screens[0] (the primary screen with the menu bar),
        // NOT mainScreen (which is the screen with the key window).
        // Cocoa coordinates have their origin at bottom-left of the PRIMARY screen,
        // and CoreGraphics coordinates have origin at top-left of PRIMARY screen.
        // Using mainScreen causes incorrect coordinate conversion on multi-monitor setups
        // when the focused window is on a secondary display.
        let primary_screen: *mut Object = msg_send![screens, objectAtIndex: 0usize];
        let primary_frame: CGRect = msg_send![primary_screen, frame];
        let primary_height = primary_frame.size.height;

        // Convert CG y to Cocoa y (once, outside the loop)
        let cocoa_y = primary_height - y as f64;

        // Find the screen containing the point
        for i in 0..screen_count {
            let screen: *mut Object = msg_send![screens, objectAtIndex: i];
            if screen.is_null() {
                continue;
            }

            // Get the full frame (in Cocoa coordinates - origin at bottom-left)
            let frame: CGRect = msg_send![screen, frame];

            // Check if point is within this screen's frame
            if (x as f64) >= frame.origin.x
                && (x as f64) < frame.origin.x + frame.size.width
                && cocoa_y >= frame.origin.y
                && cocoa_y < frame.origin.y + frame.size.height
            {
                // Get the visible frame (excludes menu bar and dock)
                let visible_frame: CGRect = msg_send![screen, visibleFrame];

                // Convert Cocoa coordinates back to CoreGraphics coordinates
                // CG origin is at top-left of primary screen
                // Cocoa origin.y is distance from bottom of primary screen
                // CG y = primary_height - (cocoa_y + height)
                let cg_y = primary_height - (visible_frame.origin.y + visible_frame.size.height);

                debug!(
                    screen_index = i,
                    frame_x = frame.origin.x,
                    frame_y = frame.origin.y,
                    frame_w = frame.size.width,
                    frame_h = frame.size.height,
                    visible_x = visible_frame.origin.x,
                    visible_y = visible_frame.origin.y,
                    visible_w = visible_frame.size.width,
                    visible_h = visible_frame.size.height,
                    cg_y = cg_y,
                    "Found screen for point ({}, {})",
                    x,
                    y
                );

                return Bounds {
                    x: visible_frame.origin.x as i32,
                    y: cg_y as i32,
                    width: visible_frame.size.width as u32,
                    height: visible_frame.size.height as u32,
                };
            }
        }
    }

    // Fallback if no screen found
    get_visible_display_bounds_fallback(x, y)
}
/// Fallback method using CGDisplay when NSScreen is unavailable
fn get_visible_display_bounds_fallback(x: i32, y: i32) -> Bounds {
    // Get all displays
    if let Ok(display_ids) = CGDisplay::active_displays() {
        for display_id in display_ids {
            let display = CGDisplay::new(display_id);
            let frame = display.bounds();

            // Check if point is within this display
            if x >= frame.origin.x as i32
                && x < (frame.origin.x + frame.size.width) as i32
                && y >= frame.origin.y as i32
                && y < (frame.origin.y + frame.size.height) as i32
            {
                let is_main = display_id == CGDisplay::main().id;

                // Conservative estimates for menu bar and dock
                let menu_bar_height = if is_main { 25 } else { 0 };
                let dock_height = if is_main { 70 } else { 0 };

                return Bounds {
                    x: frame.origin.x as i32,
                    y: frame.origin.y as i32 + menu_bar_height,
                    width: frame.size.width as u32,
                    height: (frame.size.height as i32 - menu_bar_height - dock_height) as u32,
                };
            }
        }
    }

    // Final fallback to main display
    let main = CGDisplay::main();
    let frame = main.bounds();
    Bounds {
        x: frame.origin.x as i32,
        y: frame.origin.y as i32 + 25,
        width: frame.size.width as u32,
        height: (frame.size.height - 95.0) as u32,
    }
}
