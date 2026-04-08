// ============================================================================
// WINDOW SHOW/HIDE HELPERS
// ============================================================================
// These helpers consolidate duplicated window show/hide logic that was
// scattered across hotkey handler, tray menu, stdin commands, and fallback.
// All show/hide paths should use these helpers for consistency.

fn automation_window_bounds_from_gpui(
    bounds: gpui::Bounds<gpui::Pixels>,
) -> crate::protocol::AutomationWindowBounds {
    crate::protocol::AutomationWindowBounds {
        x: f32::from(bounds.origin.x) as f64,
        y: f32::from(bounds.origin.y) as f64,
        width: f32::from(bounds.size.width) as f64,
        height: f32::from(bounds.size.height) as f64,
    }
}

fn current_main_automation_bounds() -> Option<crate::protocol::AutomationWindowBounds> {
    crate::platform::get_main_window_bounds().map(|(x, y, width, height)| {
        crate::protocol::AutomationWindowBounds {
            x,
            y,
            width,
            height,
        }
    })
}

fn sync_main_automation_window(
    bounds: Option<crate::protocol::AutomationWindowBounds>,
    visible: bool,
    focused: bool,
) {
    let Some(handle) = crate::get_main_window_handle() else {
        tracing::debug!(
            target: "script_kit::automation",
            visible,
            focused,
            "automation.main_window_sync_skipped_missing_handle"
        );
        return;
    };

    crate::windows::upsert_runtime_window_handle("main", handle);
    crate::windows::upsert_automation_window(crate::protocol::AutomationWindowInfo {
        id: "main".to_string(),
        kind: crate::protocol::AutomationWindowKind::Main,
        title: Some("Script Kit".to_string()),
        focused,
        visible,
        semantic_surface: Some("scriptList".to_string()),
        bounds: bounds.or_else(current_main_automation_bounds),
        parent_window_id: None,
        parent_kind: None,
    });
}

/// Show the main window with proper positioning, panel configuration, and focus.
///
/// This is the canonical way to show the main window. It:
/// 1. Sets MAIN_WINDOW_VISIBLE state
/// 2. Moves the panel to the active space
/// 3. Consumes NEEDS_RESET and resets hidden state before sizing
/// 4. Computes and applies final hidden geometry
/// 5. Configures as floating panel (first time only)
/// 6. Shows the window as a non-activating panel
/// 7. Restores focus state after the panel becomes key
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

    // 5. Consume NEEDS_RESET BEFORE any geometry is computed so we size the hidden
    // window for the actual post-reset view instead of the stale pre-show surface.
    let needs_reset_before_show = NEEDS_RESET
        .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok();
    if needs_reset_before_show {
        logging::log(
            "VISIBILITY",
            "NEEDS_RESET was true - resetting to script list before computing show bounds",
        );
    }

    let current_bounds = platform::get_main_window_bounds();
    let current_window_width = current_bounds.map(|(_, _, width, _)| width as f32);
    logging::log(
        "POSITION_TRACE",
        &format!(
            "Current window bounds before show: {:?}",
            current_bounds
        ),
    );
    let window_size = app_entity.update(cx, |view, ctx| {
        if needs_reset_before_show {
            view.reset_to_script_list(ctx);
        } else {
            view.ensure_selection_at_first_item(ctx);
        }

        if matches!(view.current_view, AppView::ScriptList)
            && view.main_window_mode == MainWindowMode::Mini
        {
            let (grouped_items, _) = view.get_grouped_results_cached();
            let sizing =
                crate::window_resize::mini_main_window_sizing_from_grouped_items(&grouped_items);
            gpui::size(
                px(
                    crate::window_resize::width_for_view(ViewType::MiniMainWindow)
                        .unwrap_or(750.0),
                ),
                crate::window_resize::height_for_mini_main_window(sizing),
            )
        } else if let Some((view_type, item_count)) = view.calculate_window_size_params() {
            gpui::size(
                px(
                    crate::window_resize::width_for_view(view_type)
                        .or(current_window_width)
                        .unwrap_or(750.0),
                ),
                crate::window_resize::height_for_view(view_type, item_count),
            )
        } else {
            gpui::size(
                px(current_window_width.unwrap_or(750.0)),
                crate::window_resize::height_for_view(ViewType::ScriptList, 0),
            )
        }
    });
    logging::log(
        "POSITION_TRACE",
        &format!(
            "Computed window_size for show: width={:.0}, height={:.0}",
            f32::from(window_size.width),
            f32::from(window_size.height)
        ),
    );

    // Keep legacy resize state clear before re-opening. Interactive resizes still
    // use the deferred paths after the window is already visible.
    reset_resize_debounce();

    // 6. Position the hidden window using the exact target size, then show it.
    let visible_displays = platform::get_macos_visible_displays();
    let displays: Vec<_> = visible_displays
        .iter()
        .map(|display| display.frame.clone())
        .collect();
    let mouse = platform::get_global_mouse_position();
    let bounds = if let Some((mouse_x, mouse_y)) = mouse {
        // Try to restore saved position for the mouse display
        if let Some((saved, display)) =
            window_state::get_main_position_for_mouse_display(mouse_x, mouse_y, &displays)
        {
            // Validate the saved position is still visible
            if window_state::is_bounds_visible(&saved, &displays) {
                let target_width = f32::from(window_size.width) as f64;
                let width_delta = saved.width - target_width;
                logging::log(
                    "POSITION_TRACE",
                    &format!(
                        "Restoring saved position for display {}: saved=({:.0}, {:.0}, {:.0}x{:.0}), target_width={:.0}, width_delta={:.1}",
                        window_state::display_key(&display),
                        saved.x, saved.y, saved.width, saved.height,
                        target_width, width_delta
                    ),
                );
                // Re-center horizontally when the target width differs from the saved width.
                // Without this, a width change shifts the left edge, causing the window
                // to appear offset from its original center.
                let adjusted_x = saved.x + (saved.width - target_width) / 2.0;
                logging::log(
                    "POSITION_TRACE",
                    &format!(
                        "Adjusted x: saved.x={:.0} -> adjusted_x={:.0} (shift={:.1})",
                        saved.x, adjusted_x, adjusted_x - saved.x
                    ),
                );
                // Use saved position but with current window height (may have changed)
                gpui::Bounds {
                    origin: gpui::point(px(adjusted_x as f32), px(saved.y as f32)),
                    size: window_size,
                }
            } else {
                logging::log(
                    "VISIBILITY",
                    "Saved position no longer visible, using eye-line",
                );
                platform::calculate_eye_line_bounds_for_snapshot(
                    window_size,
                    mouse,
                    &visible_displays,
                )
            }
        } else {
            logging::log(
                "VISIBILITY",
                "No saved position for this display, using eye-line",
            );
            platform::calculate_eye_line_bounds_for_snapshot(
                window_size,
                mouse,
                &visible_displays,
            )
        }
    } else {
        logging::log("VISIBILITY", "Could not get mouse position, using eye-line");
        platform::calculate_eye_line_bounds_for_snapshot(window_size, mouse, &visible_displays)
    };
    platform::move_first_window_to_bounds(&bounds);

    // 7. Configure as floating panel (first time only)
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
        let material = theme.get_vibrancy().material;
        platform::configure_window_vibrancy_material_for_appearance(is_dark, material);
        PANEL_CONFIGURED.store(true, Ordering::SeqCst);
    }

    // 8. Show without app activation, then focus (DEFERRED via cx.spawn).
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

                    // Always re-apply focus state after the window becomes visible.
                    // reset_to_script_list sets these too, but that runs BEFORE the
                    // window is shown — the render loop needs pending_focus set AFTER
                    // the window is key to actually move focus to the input element.
                    view.focused_input = FocusedInput::MainFilter;
                    view.pending_focus = Some(FocusTarget::MainFilter);
                    ctx.notify();
                });

                logging::log("VISIBILITY", "Main window shown and focused");
                sync_main_automation_window(None, true, true);
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
        logging::log(
            "POSITION_TRACE",
            &format!(
                "Hide: saving bounds ({:.0}, {:.0}, {:.0}x{:.0})",
                x, y, width, height
            ),
        );

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
    sync_main_automation_window(current_main_automation_bounds(), false, false);
    crate::footer_popup::close_main_footer_popup(cx);

    // 3. Check secondary windows BEFORE the update closure
    let notes_open = notes::is_notes_window_open();
    let ai_open = ai::is_ai_window_open();
    let acp_chat_open = ai::acp::chat_window::is_chat_window_open();
    logging::log(
        "VISIBILITY",
        &format!(
            "Secondary windows: notes_open={}, ai_open={}, acp_chat_open={}",
            notes_open, ai_open, acp_chat_open
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
    if notes_open || ai_open || acp_chat_open {
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
