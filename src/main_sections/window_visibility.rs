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
/// 5. If NEEDS_RESET is set, resets to script list before showing
/// 6. Shows the window as a non-activating panel and focuses the input
/// 7. Resets resize debounce
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
    logging::bench_log("show_main_window_helper_start");
    logging::log("VISIBILITY", "show_main_window_helper called");

    // 1. Set visibility state
    set_main_window_visible(true);

    // 2. Re-enable position saving (may have been suppressed by reset)
    window_state::allow_save();

    // 3. Mark window shown timestamp for focus grace period
    // This prevents the window from being closed by focus loss immediately after opening
    script_kit_gpui::mark_window_shown();

    // 4. Move to active space (macOS)
    platform::ensure_move_to_active_space();

    // 4. Position window - try per-display saved position first, then fall back to eye-line
    let window_size = gpui::size(px(750.), initial_window_height());
    let displays = platform::get_macos_displays();
    let bounds = if let Some((mouse_x, mouse_y)) = platform::get_global_mouse_position() {
        // Try to restore saved position for the mouse display
        if let Some((saved, display)) =
            window_state::get_main_position_for_mouse_display(mouse_x, mouse_y, &displays)
        {
            // Validate the saved position is still visible
            if window_state::is_bounds_visible(&saved, &displays) {
                logging::log(
                    "VISIBILITY",
                    &format!(
                        "Restoring saved position for display {}: ({:.0}, {:.0})",
                        window_state::display_key(&display),
                        saved.x,
                        saved.y
                    ),
                );
                // Use saved position but with current window height (may have changed)
                gpui::Bounds {
                    origin: gpui::point(px(saved.x as f32), px(saved.y as f32)),
                    size: window_size,
                }
            } else {
                logging::log(
                    "VISIBILITY",
                    "Saved position no longer visible, using eye-line",
                );
                platform::calculate_eye_line_bounds_on_mouse_display(window_size)
            }
        } else {
            logging::log(
                "VISIBILITY",
                "No saved position for this display, using eye-line",
            );
            platform::calculate_eye_line_bounds_on_mouse_display(window_size)
        }
    } else {
        logging::log("VISIBILITY", "Could not get mouse position, using eye-line");
        platform::calculate_eye_line_bounds_on_mouse_display(window_size)
    };
    platform::move_first_window_to_bounds(&bounds);

    // 5. Configure as floating panel (first time only)
    if !PANEL_CONFIGURED.load(Ordering::SeqCst) {
        platform::configure_as_floating_panel();
        // HACK: Swizzle GPUI's BlurredView to preserve native CAChameleonLayer tint
        // GPUI hides this layer which removes the native macOS vibrancy tinting.
        // By swizzling, we get proper native blur appearance like Raycast/Spotlight.
        platform::swizzle_gpui_blurred_view();
        // Configure vibrancy material based on theme's actual colors
        // Uses VibrantDark for dark-colored themes, VibrantLight for light-colored themes
        let theme = theme::get_cached_theme();
        let is_dark = theme.should_use_dark_vibrancy();
        platform::configure_window_vibrancy_material_for_appearance(is_dark);
        PANEL_CONFIGURED.store(true, Ordering::SeqCst);
    }

    // 6. Handle NEEDS_RESET before showing.
    // This avoids flashing stale UI (e.g. clipboard history) for one frame.
    let needs_reset_before_show = NEEDS_RESET
        .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok();
    if needs_reset_before_show {
        logging::log(
            "VISIBILITY",
            "NEEDS_RESET was true - resetting to script list before showing window",
        );
        app_entity.update(cx, |view, ctx| {
            view.reset_to_script_list(ctx);
        });
    }

    // 7. Show without app activation, then focus (DEFERRED via cx.spawn).
    //
    // macOS makeKeyWindow / makeKeyAndOrderFront: synchronously fires
    // windowDidBecomeKey: → GPUI request_frame_callback → AsyncApp::update().
    // If we're already inside an AsyncApp::update() (the caller's cx.update()),
    // that second borrow_mut() on the AppCell panics with "RefCell already borrowed".
    //
    // Spawning defers to the next event-loop tick where no AppCell borrow is held.
    // Platform calls that trigger delegate callbacks run OUTSIDE cx.update();
    // GPUI-only state changes (focus, selection, resize) run INSIDE cx.update().
    cx.spawn({
        let app_entity = app_entity.clone();
        async move |cx: &mut gpui::AsyncApp| {
            logging::bench_log("window_show_native_start");

            // Platform calls — trigger macOS delegate callbacks.
            // Safe here: no AppCell borrow is active.
            platform::show_main_window_without_activation();
            platform::send_ai_window_to_back();

            logging::bench_log("window_activated");

            // GPUI state changes — no macOS callbacks, safe inside borrow.
            cx.update(move |cx: &mut gpui::App| {
                app_entity.update(cx, |view, ctx| {
                    let focus_handle = view.focus_handle(ctx);
                    let _ = window.update(ctx, |_root, win, _cx| {
                        win.activate_window();
                        win.focus(&focus_handle, _cx);
                    });

                    // Reset resize debounce to ensure proper window sizing
                    reset_resize_debounce();

                    if !needs_reset_before_show {
                        view.ensure_selection_at_first_item(ctx);
                        view.focused_input = FocusedInput::MainFilter;
                        view.pending_focus = Some(FocusTarget::MainFilter);
                    }

                    // Always ensure window size matches current view using deferred resize.
                    let _ = window.update(ctx, |_root, win, win_cx| {
                        defer_resize_to_view(ViewType::ScriptList, 0, win, win_cx);
                    });
                });

                logging::log("VISIBILITY", "Main window shown and focused");
            });
        }
    })
    .detach();
}

/// Hide the main window with proper state management.
///
/// This is the canonical way to hide the main window. It:
/// 1. Saves window position for the current display (per-display persistence)
/// 2. Sets MAIN_WINDOW_VISIBLE state to false
/// 3. Cancels any active prompt (if in prompt mode)
/// 4. Resets to script list
/// 5. Uses hide_main_window() if Notes/AI windows are open (to avoid hiding them)
/// 6. Uses cx.hide() if no secondary windows are open
///
/// # Arguments
/// * `app_entity` - The ScriptListApp entity
/// * `cx` - The application context
fn hide_main_window_helper(app_entity: Entity<ScriptListApp>, cx: &mut App) {
    logging::log("VISIBILITY", "hide_main_window_helper called");

    // 1. Save window position for the current display BEFORE hiding
    if let Some((x, y, width, height)) = platform::get_main_window_bounds() {
        let displays = platform::get_macos_displays();
        let bounds = window_state::PersistedWindowBounds::new(x, y, width, height);

        // Find which display the window center is on
        if let Some(display) = window_state::find_display_for_bounds(&bounds, &displays) {
            logging::log(
                "VISIBILITY",
                &format!(
                    "Saving position for display {}: ({:.0}, {:.0})",
                    window_state::display_key(display),
                    x,
                    y
                ),
            );
            window_state::save_main_position_for_display(display, bounds);
        } else {
            logging::log(
                "VISIBILITY",
                "Could not determine display for window position",
            );
        }
    } else {
        logging::log("VISIBILITY", "Could not get window bounds for saving");
    }

    // 2. Set visibility state
    set_main_window_visible(false);

    // 3. Check secondary windows BEFORE the update closure
    let notes_open = notes::is_notes_window_open();
    let ai_open = ai::is_ai_window_open();
    logging::log(
        "VISIBILITY",
        &format!(
            "Secondary windows: notes_open={}, ai_open={}",
            notes_open, ai_open
        ),
    );

    // 4. Cancel prompt and reset UI
    app_entity.update(cx, |view, ctx| {
        if view.is_in_prompt() {
            logging::log("VISIBILITY", "Canceling prompt before hiding");
            view.cancel_script_execution(ctx);
        }
        view.reset_to_script_list(ctx);
    });

    // 5. Hide appropriately based on secondary windows
    if notes_open || ai_open {
        logging::log(
            "VISIBILITY",
            "Using defer_hide_main_window() - secondary windows are open",
        );
        platform::defer_hide_main_window(cx);
    } else {
        logging::log("VISIBILITY", "Using cx.hide() - no secondary windows");
        cx.hide();
    }

    logging::log("VISIBILITY", "Main window hidden");
}
