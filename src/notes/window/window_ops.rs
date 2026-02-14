use super::*;

#[cfg(target_os = "macos")]
const NS_FLOATING_WINDOW_LEVEL: i64 = 3;
#[cfg(target_os = "macos")]
const NS_WINDOW_COLLECTION_BEHAVIOR_CAN_JOIN_ALL_SPACES: u64 = 1 << 0;
#[cfg(target_os = "macos")]
const NS_WINDOW_COLLECTION_BEHAVIOR_MOVE_TO_ACTIVE_SPACE: u64 = 1 << 1;
#[cfg(target_os = "macos")]
const NS_WINDOW_COLLECTION_BEHAVIOR_IGNORES_CYCLE: u64 = 1 << 6;
#[cfg(target_os = "macos")]
const NS_WINDOW_COLLECTION_BEHAVIOR_FULL_SCREEN_AUXILIARY: u64 = 1 << 8;

/// Sync Script Kit theme with gpui-component theme
/// NOTE: Do NOT call gpui_component::init here - it's already called in main.rs
/// and calling it again resets the theme to system defaults (opaque backgrounds),
/// which breaks vibrancy.
fn ensure_theme_initialized(cx: &mut App) {
    // Just sync our theme colors - gpui_component is already initialized in main.rs
    crate::theme::sync_gpui_component_theme(cx);

    info!("Notes window theme synchronized with Script Kit");
}

/// Calculate window bounds positioned in the top-right corner of the display containing the mouse.
fn calculate_top_right_bounds(width: f32, height: f32, padding: f32) -> gpui::Bounds<gpui::Pixels> {
    use crate::platform::{
        clamp_to_visible, display_for_point, get_global_mouse_position, get_macos_visible_displays,
    };

    let displays = get_macos_visible_displays();

    // Find display containing mouse
    let target_display =
        get_global_mouse_position().and_then(|mouse_pt| display_for_point(mouse_pt, &displays));

    // Use found display or fall back to primary
    let display = target_display.or_else(|| displays.first().cloned());

    if let Some(display) = display {
        let visible = &display.visible_area;

        // Position in top-right corner with padding
        let x = visible.origin_x + visible.width - width as f64 - padding as f64;
        let y = visible.origin_y + padding as f64;

        let desired_bounds = gpui::Bounds::new(
            gpui::Point::new(px(x as f32), px(y as f32)),
            gpui::Size::new(px(width), px(height)),
        );

        clamp_to_visible(desired_bounds, visible)
    } else {
        // Fallback to centered on primary
        gpui::Bounds::new(
            gpui::Point::new(px(100.0), px(100.0)),
            gpui::Size::new(px(width), px(height)),
        )
    }
}

/// Toggle the notes window (open if closed, close if open)
pub fn open_notes_window(cx: &mut App) -> Result<()> {
    use crate::logging;

    logging::log("PANEL", "open_notes_window called - checking toggle state");

    // Ensure gpui-component theme is initialized before opening window
    ensure_theme_initialized(cx);

    // SAFETY: Release lock BEFORE calling handle.update() to prevent deadlock.
    // We clone the handle (it's just an ID) and release the lock immediately.
    let existing_handle = {
        let slot = NOTES_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
        slot.lock().ok().and_then(|g| *g)
    };

    // Check if window already exists and is valid
    if let Some(handle) = existing_handle {
        // Window exists - check if it's valid and close it (toggle OFF)
        // Lock is released, safe to call handle.update()
        if handle
            .update(cx, |_, window, _cx| {
                // Save bounds before closing (fixes bounds persistence on toggle close)
                let wb = window.window_bounds();
                crate::window_state::save_window_from_gpui(
                    crate::window_state::WindowRole::Notes,
                    wb,
                );
                window.remove_window();
            })
            .is_ok()
        {
            // Close any open CommandBar windows (command_bar and note_switcher)
            // They use a global singleton, so we close it via the actions module
            crate::actions::close_actions_window(cx);
            logging::log("PANEL", "Notes window was open - closing (toggle OFF)");
            // Clear the stored handle
            let slot = NOTES_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
            if let Ok(mut g) = slot.lock() {
                *g = None;
            }

            // NOTE: We intentionally do NOT call cx.hide() here.
            // Closing Notes should not affect the main window's ability to be shown.
            // The main window hotkey handles its own visibility state.
            // If the user wants to hide everything, they can press the main hotkey
            // when the main window is visible.

            return Ok(());
        }
        // Window handle was invalid, fall through to create new window
        logging::log("PANEL", "Notes window handle was invalid - creating new");
    }

    // If main window is visible, hide it (Notes takes focus)
    // Use platform::hide_main_window() to only hide the main window, not the whole app
    // IMPORTANT: Set visibility to false so the main hotkey knows to SHOW (not hide) next time
    if crate::is_main_window_visible() {
        logging::log(
            "PANEL",
            "Main window was visible - hiding it since Notes is opening",
        );
        crate::set_main_window_visible(false);
        crate::platform::hide_main_window();
    }

    // Create new window (toggle ON)
    logging::log("PANEL", "Notes window not open - creating new (toggle ON)");
    info!("Opening new notes window");

    // Calculate position: try saved position first, then top-right default
    let window_width = 350.0_f32;
    let window_height = 280.0_f32;
    let padding = 20.0_f32; // Padding from screen edges

    let default_bounds = calculate_top_right_bounds(window_width, window_height, padding);
    let displays = crate::platform::get_macos_displays();
    let bounds = crate::window_state::get_initial_bounds(
        crate::window_state::WindowRole::Notes,
        default_bounds,
        &displays,
    );

    // Load theme to determine window background appearance (vibrancy)
    let theme = crate::theme::load_theme();
    let window_background = if theme.is_vibrancy_enabled() {
        gpui::WindowBackgroundAppearance::Blurred
    } else {
        gpui::WindowBackgroundAppearance::Opaque
    };

    let window_options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(gpui::TitlebarOptions {
            title: Some("Notes".into()),
            appears_transparent: true,
            traffic_light_position: Some(gpui::Point {
                x: px(8.),
                y: px(7.), // Centered vertically in 26px header
            }),
        }),
        window_background,
        focus: true,
        show: true,
        // Use PopUp for floating panel behavior - allows keyboard input without
        // activating the app (Raycast-like). Creates NSPanel with NonactivatingPanel mask.
        kind: gpui::WindowKind::PopUp,
        ..Default::default()
    };

    // Store the NotesApp entity so we can focus it after window creation
    let notes_app_holder: std::sync::Arc<std::sync::Mutex<Option<Entity<NotesApp>>>> =
        std::sync::Arc::new(std::sync::Mutex::new(None));
    let notes_app_for_closure = notes_app_holder.clone();

    let handle = cx.open_window(window_options, |window, cx| {
        let view = cx.new(|cx| NotesApp::new(window, cx));
        *notes_app_for_closure
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some(view.clone());
        cx.new(|cx| Root::new(view, window, cx))
    })?;

    // NOTE: We do NOT call cx.activate(true) here!
    // Notes is a PopUp window (NSPanel with NonactivatingPanel style), which means
    // it can receive keyboard input without activating the application.
    // Calling activate(true) would bring ALL windows forward (including main window),
    // causing a flash before we could hide it.
    //
    // Instead, we just ensure the main window is hidden (in case it was visible)
    // and let the PopUp window handle focus naturally.
    crate::platform::hide_main_window();

    // Focus the editor input in the Notes window
    // Release lock before calling update
    let notes_app_entity = notes_app_holder.lock().ok().and_then(|mut g| g.take());
    if let Some(notes_app) = notes_app_entity {
        // Store the entity globally for quick_capture access
        {
            let slot = NOTES_APP_ENTITY.get_or_init(|| std::sync::Mutex::new(None));
            if let Ok(mut g) = slot.lock() {
                *g = Some(notes_app.clone());
            }
        }

        let _ = handle.update(cx, |_root, window, cx| {
            window.activate_window();

            // Focus the NotesApp's editor input and move cursor to end
            notes_app.update(cx, |app, cx| {
                // Get content length for cursor positioning
                let content_len = app.editor_state.read(cx).value().len();

                // Call the InputState's focus method and move cursor to end
                app.editor_state.update(cx, |state, inner_cx| {
                    state.focus(window, inner_cx);
                    // Move cursor to end of text (same as select_note behavior)
                    state.set_selection(content_len, content_len, window, inner_cx);
                });

                if std::env::var("SCRIPT_KIT_TEST_NOTES_HOVERED")
                    .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
                    .unwrap_or(false)
                {
                    app.force_hovered = true;
                    app.window_hovered = true;
                    app.titlebar_hovered = true;
                }

                if std::env::var("SCRIPT_KIT_TEST_NOTES_ACTIONS_PANEL")
                    .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
                    .unwrap_or(false)
                {
                    app.open_actions_panel(window, cx);
                }

                cx.notify();
            });
        });
    }

    // Store the window handle (release lock immediately)
    {
        let slot = NOTES_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
        if let Ok(mut g) = slot.lock() {
            *g = Some(handle);
        }
    }

    // Configure as floating panel (always on top) after window is created
    configure_notes_as_floating_panel();

    // NOTE: Theme hot-reload is now handled by the centralized ThemeService
    // (crate::theme::service::ensure_theme_service) which is started once at app init.
    // This eliminates per-window theme watcher tasks and their potential for leaks.

    Ok(())
}

/// Quick capture - open notes with a new note ready for input
///
/// Creates a new empty note and focuses the editor immediately,
/// providing a frictionless capture experience like Apple Quick Note (Fn+Q)
/// or Raycast's Option-click menu bar.
pub fn quick_capture(cx: &mut App) -> Result<()> {
    use crate::logging;

    // Get existing window and app entity
    let existing_handle = {
        let slot = NOTES_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
        slot.lock().ok().and_then(|g| *g)
    };

    let existing_app = {
        let slot = NOTES_APP_ENTITY.get_or_init(|| std::sync::Mutex::new(None));
        slot.lock().ok().and_then(|g| g.clone())
    };

    // If window exists with valid app entity, create new note in existing window
    if let (Some(handle), Some(notes_app)) = (existing_handle, existing_app) {
        let result = handle.update(cx, |_root, window, cx| {
            notes_app.update(cx, |app, cx| {
                app.create_note(window, cx);
            });
        });

        if result.is_ok() {
            logging::log(
                "PANEL",
                "Quick capture: created new note in existing window",
            );
            return Ok(());
        }
        // Handle was invalid, fall through to create new window
    }

    // Window doesn't exist - create new window with a new note
    open_notes_window(cx)?;

    // After window is created, create a new note using the stored entity
    let handle = {
        let slot = NOTES_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
        slot.lock().ok().and_then(|g| *g)
    };

    let notes_app = {
        let slot = NOTES_APP_ENTITY.get_or_init(|| std::sync::Mutex::new(None));
        slot.lock().ok().and_then(|g| g.clone())
    };

    if let (Some(handle), Some(notes_app)) = (handle, notes_app) {
        let _ = handle.update(cx, |_root, window, cx| {
            notes_app.update(cx, |app, cx| {
                app.create_note(window, cx);
            });
        });
        logging::log("PANEL", "Quick capture: created new window with new note");
    }

    Ok(())
}

/// Close the notes window
pub fn close_notes_window(cx: &mut App) {
    // SAFETY: Release lock BEFORE calling handle.update() to prevent deadlock
    // If handle.update() causes Drop to fire synchronously and tries to acquire
    // the same lock, we would deadlock. Taking the handle out first avoids this.
    let handle = {
        let slot = NOTES_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
        slot.lock().ok().and_then(|mut g| g.take())
    };

    if let Some(handle) = handle {
        let _ = handle.update(cx, |_, window, _| {
            // Save window bounds before closing
            let wb = window.window_bounds();
            crate::window_state::save_window_from_gpui(crate::window_state::WindowRole::Notes, wb);
            window.remove_window();
        });
    }
}

/// Check if the notes window is currently open
///
/// Returns true if the Notes window exists and is valid.
/// This is used by other parts of the app to check if Notes is open
/// without affecting it.
pub fn is_notes_window_open() -> bool {
    let window_handle = NOTES_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
    let guard = window_handle.lock().unwrap_or_else(|e| e.into_inner());
    guard.is_some()
}

/// Check if the given window handle matches the Notes window
///
/// Returns true if the window is the Notes window.
/// Used by keystroke interceptors to avoid handling keys meant for Notes.
pub fn is_notes_window(window: &gpui::Window) -> bool {
    let window_handle = NOTES_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
    if let Ok(guard) = window_handle.lock() {
        if let Some(notes_handle) = guard.as_ref() {
            // Convert WindowHandle<Root> to AnyWindowHandle via Into trait
            let notes_any: gpui::AnyWindowHandle = (*notes_handle).into();
            return window.window_handle() == notes_any;
        }
    }
    false
}

/// Configure the Notes window as a floating panel (always on top).
///
/// This sets:
/// - NSFloatingWindowLevel (3) - floats above normal windows
/// - NSWindowCollectionBehaviorMoveToActiveSpace - moves to current space when shown
/// - Disabled window restoration - prevents macOS position caching
#[cfg(target_os = "macos")]
fn configure_notes_as_floating_panel() {
    use crate::logging;
    use std::ffi::CStr;

    unsafe {
        let app: id = NSApp();
        let windows: id = msg_send![app, windows];
        let count: usize = msg_send![windows, count];

        for i in 0..count {
            let window: id = msg_send![windows, objectAtIndex: i];
            let title: id = msg_send![window, title];

            if title != nil {
                let title_cstr: *const i8 = msg_send![title, UTF8String];
                if !title_cstr.is_null() {
                    let title_str = CStr::from_ptr(title_cstr).to_string_lossy();

                    if title_str == "Notes" {
                        // Found the Notes window - configure it

                        // Use i64 (NSInteger) for proper ABI compatibility on 64-bit macOS
                        let floating_level: i64 = NS_FLOATING_WINDOW_LEVEL;
                        let _: () = msg_send![window, setLevel:floating_level];

                        // Get current collection behavior to preserve existing flags
                        let current: u64 = msg_send![window, collectionBehavior];

                        // Check if window has CanJoinAllSpaces (set by GPUI for PopUp windows)
                        // If so, we can't add MoveToActiveSpace (they're mutually exclusive)
                        let has_can_join_all_spaces =
                            (current & NS_WINDOW_COLLECTION_BEHAVIOR_CAN_JOIN_ALL_SPACES) != 0;

                        // OR in FullScreenAuxiliary (256) + IgnoresCycle (64)
                        // IgnoresCycle excludes Notes from Cmd+Tab - it's a utility window
                        // MoveToActiveSpace (2) only if not already CanJoinAllSpaces
                        let desired: u64 = if has_can_join_all_spaces {
                            current
                                | NS_WINDOW_COLLECTION_BEHAVIOR_FULL_SCREEN_AUXILIARY
                                | NS_WINDOW_COLLECTION_BEHAVIOR_IGNORES_CYCLE
                        } else {
                            current
                                | NS_WINDOW_COLLECTION_BEHAVIOR_MOVE_TO_ACTIVE_SPACE
                                | NS_WINDOW_COLLECTION_BEHAVIOR_FULL_SCREEN_AUXILIARY
                                | NS_WINDOW_COLLECTION_BEHAVIOR_IGNORES_CYCLE
                        };
                        let _: () = msg_send![window, setCollectionBehavior:desired];

                        // Ensure window content is shareable for captureScreenshot()
                        let sharing_type: i64 = 1; // NSWindowSharingReadOnly
                        let _: () = msg_send![window, setSharingType:sharing_type];

                        // Disable window restoration
                        let _: () = msg_send![window, setRestorable:false];

                        // Disable close/hide animation for instant dismiss (NSWindowAnimationBehaviorNone = 2)
                        let _: () = msg_send![window, setAnimationBehavior: 2i64];

                        // ═══════════════════════════════════════════════════════════════════════════
                        // VIBRANCY CONFIGURATION - Match main window for consistent blur
                        // ═══════════════════════════════════════════════════════════════════════════
                        let theme = crate::theme::load_theme();
                        let is_dark = theme.should_use_dark_vibrancy();
                        crate::platform::configure_secondary_window_vibrancy(
                            window, "Notes", is_dark,
                        );

                        // Log detailed breakdown of collection behavior bits
                        let has_can_join =
                            (desired & NS_WINDOW_COLLECTION_BEHAVIOR_CAN_JOIN_ALL_SPACES) != 0;
                        let has_ignores =
                            (desired & NS_WINDOW_COLLECTION_BEHAVIOR_IGNORES_CYCLE) != 0;
                        let has_move_to_active =
                            (desired & NS_WINDOW_COLLECTION_BEHAVIOR_MOVE_TO_ACTIVE_SPACE) != 0;

                        logging::log(
                            "PANEL",
                            &format!(
                                "Notes window: behavior={}->{} [CanJoinAllSpaces={}, IgnoresCycle={}, MoveToActiveSpace={}]",
                                current, desired, has_can_join, has_ignores, has_move_to_active
                            ),
                        );
                        logging::log(
                            "PANEL",
                            "Notes window: Will NOT appear in Cmd+Tab app switcher (floating utility panel)",
                        );
                        return;
                    }
                }
            }
        }

        logging::log(
            "PANEL",
            "Warning: Notes window not found by title for floating panel config",
        );
    }
}

#[cfg(not(target_os = "macos"))]
fn configure_notes_as_floating_panel() {
    // No-op on non-macOS platforms
}
