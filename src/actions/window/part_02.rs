#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actions_window_key_intent_supports_aliases_and_jump_keys() {
        let no_mods = gpui::Modifiers::default();

        assert_eq!(
            actions_window_key_intent("return", &no_mods),
            Some(ActionsWindowKeyIntent::ExecuteSelected)
        );
        assert_eq!(
            actions_window_key_intent("esc", &no_mods),
            Some(ActionsWindowKeyIntent::Close)
        );
        assert_eq!(
            actions_window_key_intent("home", &no_mods),
            Some(ActionsWindowKeyIntent::MoveHome)
        );
        assert_eq!(
            actions_window_key_intent("end", &no_mods),
            Some(ActionsWindowKeyIntent::MoveEnd)
        );
        assert_eq!(
            actions_window_key_intent("pageup", &no_mods),
            Some(ActionsWindowKeyIntent::MovePageUp)
        );
        assert_eq!(
            actions_window_key_intent("pagedown", &no_mods),
            Some(ActionsWindowKeyIntent::MovePageDown)
        );
    }

    #[test]
    fn test_actions_window_selectable_index_helpers_skip_section_headers() {
        let rows = vec![
            GroupedActionItem::SectionHeader("One".to_string()),
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("Two".to_string()),
            GroupedActionItem::Item(1),
        ];

        assert_eq!(first_selectable_index(&rows), Some(1));
        assert_eq!(last_selectable_index(&rows), Some(3));
        assert_eq!(selectable_index_at_or_before(&rows, 2), Some(1));
        assert_eq!(selectable_index_at_or_after(&rows, 2), Some(3));
    }

    #[test]
    fn test_actions_window_dynamic_height_matches_single_row_when_empty() {
        let empty_height = actions_window_dynamic_height(0, 0, false, false);
        let single_row_height = actions_window_dynamic_height(1, 0, false, false);

        assert!(
            (empty_height - single_row_height).abs() < 0.001,
            "empty_height={empty_height}, single_row_height={single_row_height}"
        );
    }
}

#[inline]
fn actions_window_dynamic_height(
    num_actions: usize,
    section_header_count: usize,
    hide_search: bool,
    has_header: bool,
) -> f32 {
    const POPUP_BORDER_HEIGHT: f32 = 2.0;
    let search_box_height = if hide_search {
        0.0
    } else {
        SEARCH_INPUT_HEIGHT
    };
    let header_height = if has_header { HEADER_HEIGHT } else { 0.0 };
    let section_headers_height = section_header_count as f32 * SECTION_HEADER_HEIGHT;
    let min_items_height = if num_actions == 0 {
        ACTION_ITEM_HEIGHT
    } else {
        0.0
    };
    let items_height = (num_actions as f32 * ACTION_ITEM_HEIGHT + section_headers_height)
        .max(min_items_height)
        .min(POPUP_MAX_HEIGHT - search_box_height - header_height);
    let border_height = POPUP_BORDER_HEIGHT;
    items_height + search_box_height + header_height + border_height
}

#[inline]
fn compute_popup_height(dialog: &ActionsDialog) -> f32 {
    let num_actions = dialog.filtered_actions.len();
    let hide_search = dialog.hide_search;
    let has_header = dialog.context_title.is_some();

    let section_header_count = if dialog.config.section_style == SectionStyle::Headers {
        count_section_headers(&dialog.actions, &dialog.filtered_actions)
    } else {
        0
    };

    actions_window_dynamic_height(num_actions, section_header_count, hide_search, has_header)
}

/// Open the actions window as a separate floating window with vibrancy
///
/// The window is positioned at the top-right of the main window, below the header.
/// It does NOT take keyboard focus - the main window keeps focus and routes
/// keyboard events to the shared ActionsDialog entity.
///
/// # Arguments
/// * `cx` - The application context
/// * `main_window_bounds` - The bounds of the main window in SCREEN-RELATIVE coordinates
///   (as returned by GPUI's window.bounds() - top-left origin relative to the window's screen)
/// * `display_id` - The display where the main window is located (actions window will be on same display)
/// * `dialog_entity` - The shared ActionsDialog entity (created by main app)
/// * `position` - Where to position the window relative to the main window
///
/// # Returns
/// The window handle on success
pub fn open_actions_window(
    cx: &mut App,
    main_window_bounds: Bounds<Pixels>,
    display_id: Option<DisplayId>,
    dialog_entity: Entity<ActionsDialog>,
    position: WindowPosition,
) -> anyhow::Result<WindowHandle<Root>> {
    // Close any existing actions window first
    close_actions_window(cx);

    // Load theme for vibrancy settings
    let theme = theme::load_theme();
    let window_background = if theme.is_vibrancy_enabled() {
        gpui::WindowBackgroundAppearance::Blurred
    } else {
        gpui::WindowBackgroundAppearance::Opaque
    };

    // Calculate dynamic window height based on number of actions.
    // Open and resize paths intentionally share compute_popup_height().
    let dialog = dialog_entity.read(cx);
    let dynamic_height = compute_popup_height(dialog);

    // Calculate window position:
    // - X: Right edge of main window, minus actions width, minus margin
    // - Y: Depends on position parameter:
    //   - BottomRight: Above footer, aligned to bottom
    //   - TopRight: Below titlebar, aligned to top
    //
    // CRITICAL: main_window_bounds must be in SCREEN-RELATIVE coordinates from GPUI's
    // window.bounds(). These are top-left origin, relative to the window's current screen.
    // When we pass display_id to WindowOptions, GPUI will position this window on the
    // same screen as the main window, using these screen-relative coordinates.
    let window_width = px(ACTIONS_WINDOW_WIDTH);
    let window_height = px(dynamic_height);

    let window_x = main_window_bounds.origin.x + main_window_bounds.size.width
        - window_width
        - px(ACTIONS_MARGIN_X);

    // Calculate Y position based on anchor position
    let window_y = match position {
        WindowPosition::BottomRight => {
            // Position popup above the footer (footer is 40px)
            main_window_bounds.origin.y + main_window_bounds.size.height
                - window_height
                - px(FOOTER_HEIGHT)
                - px(ACTIONS_MARGIN_Y)
        }
        WindowPosition::TopRight => {
            // Position popup below the titlebar
            main_window_bounds.origin.y + px(TITLEBAR_HEIGHT) + px(ACTIONS_MARGIN_Y)
        }
        WindowPosition::TopCenter => {
            // Position popup below the titlebar (same Y as TopRight)
            main_window_bounds.origin.y + px(TITLEBAR_HEIGHT) + px(ACTIONS_MARGIN_Y)
        }
    };

    // Override X position for TopCenter - center horizontally in the parent window
    let window_x = match position {
        WindowPosition::TopCenter => {
            // Center horizontally within the parent window
            main_window_bounds.origin.x + (main_window_bounds.size.width - window_width) / 2.0
        }
        _ => window_x, // Keep the right-aligned X for other positions
    };

    let bounds = Bounds {
        origin: Point {
            x: window_x,
            y: window_y,
        },
        size: Size {
            width: window_width,
            height: window_height,
        },
    };

    crate::logging::log(
        "ACTIONS",
        &format!(
            "Opening actions window at ({:?}, {:?}), size {:?}x{:?}, display_id={:?}",
            window_x, window_y, window_width, window_height, display_id
        ),
    );

    let window_options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: None, // No titlebar = no drag affordance
        window_background,
        // DON'T take focus - let the parent AI window keep focus and route keys to us
        // macOS popup windows often don't receive keyboard events properly
        focus: false,
        show: true,
        kind: WindowKind::PopUp, // Floating popup window
        display_id,              // CRITICAL: Position on same display as main window
        ..Default::default()
    };

    // Create the window with the shared dialog entity
    // NOTE: We DON'T focus the ActionsWindow's focus_handle here.
    // The parent window (AI window, Notes window, etc.) keeps focus and routes
    // keyboard events to us via its own capture_key_down handler.
    // This avoids focus conflicts where both windows try to handle keys.
    let handle = cx.open_window(window_options, |window, cx| {
        let actions_window = cx.new(|cx| ActionsWindow::new(dialog_entity, cx));
        // Wrap in Root for gpui-component theming and vibrancy
        cx.new(|cx| Root::new(actions_window, window, cx))
    })?;

    // Configure the window as non-movable on macOS
    // Use window.defer() to avoid RefCell borrow conflicts - GPUI may still have
    // internal state borrowed immediately after open_window returns.
    #[cfg(target_os = "macos")]
    {
        let configure_result = handle.update(cx, |_root, window, cx| {
            window.defer(cx, |_window, _cx| {
                use cocoa::appkit::NSApp;
                use cocoa::base::nil;
                use objc::{msg_send, sel, sel_impl};

                // Get the NSWindow from the app's windows array
                // The popup window should be the most recently created one
                unsafe {
                    let app: cocoa::base::id = NSApp();
                    let windows: cocoa::base::id = msg_send![app, windows];
                    let count: usize = msg_send![windows, count];
                    if count > 0 {
                        // Get the last window (most recently created)
                        let ns_window: cocoa::base::id = msg_send![windows, lastObject];
                        if ns_window != nil {
                            let theme = crate::theme::load_theme();
                            let is_dark = theme.should_use_dark_vibrancy();
                            platform::configure_actions_popup_window(ns_window, is_dark);
                        }
                    }
                }
            });
        });

        if let Err(error) = configure_result {
            crate::logging::log(
                "WARN",
                &format!(
                    "ACTIONS_WINDOW_OP_FAIL configure_popup_window update failed: operation=position_focus error={error:?}"
                ),
            );
            crate::logging::log_debug(
                "ACTIONS",
                &format!(
                    "ACTIONS_WINDOW_OP_FAIL configure_popup_window context: display_id={display_id:?}, position={position:?}"
                ),
            );
        }
    }

    // Store the handle globally
    let window_storage = ACTIONS_WINDOW.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = window_storage.lock() {
        *guard = Some(handle);
    }

    crate::logging::log("ACTIONS", "Actions popup window opened with vibrancy");

    Ok(handle)
}

/// Close the actions window if it's open
pub fn close_actions_window(cx: &mut App) {
    if let Some(window_storage) = ACTIONS_WINDOW.get() {
        if let Ok(mut guard) = window_storage.lock() {
            if let Some(handle) = guard.take() {
                crate::logging::log("ACTIONS", "Closing actions popup window");
                // Close the window
                let close_result = handle.update(cx, |_root, window, _cx| {
                    window.remove_window();
                });
                if let Err(error) = close_result {
                    crate::logging::log(
                        "WARN",
                        &format!(
                            "ACTIONS_WINDOW_OP_FAIL close_actions_window update failed: operation=focus_cleanup error={error:?}"
                        ),
                    );
                    crate::logging::log_debug(
                        "ACTIONS",
                        "ACTIONS_WINDOW_OP_FAIL close_actions_window context: remove_window requested",
                    );
                }
            }
        }
    }
}

/// Check if the actions window is currently open
pub fn is_actions_window_open() -> bool {
    if let Some(window_storage) = ACTIONS_WINDOW.get() {
        if let Ok(guard) = window_storage.lock() {
            return guard.is_some();
        }
    }
    false
}

/// Get the actions window handle if it exists
pub fn get_actions_window_handle() -> Option<WindowHandle<Root>> {
    if let Some(window_storage) = ACTIONS_WINDOW.get() {
        if let Ok(guard) = window_storage.lock() {
            return *guard;
        }
    }
    None
}

/// Notify the actions window to re-render (call after updating dialog entity)
pub fn notify_actions_window(cx: &mut App) {
    if let Some(handle) = get_actions_window_handle() {
        let notify_result = handle.update(cx, |_root, _window, cx| {
            cx.notify();
        });
        if let Err(error) = notify_result {
            crate::logging::log(
                "WARN",
                &format!(
                    "ACTIONS_WINDOW_OP_FAIL notify_actions_window update failed: operation=focus_refresh error={error:?}"
                ),
            );
            crate::logging::log_debug(
                "ACTIONS",
                "ACTIONS_WINDOW_OP_FAIL notify_actions_window context: cx.notify() skipped",
            );
        }
    }
}
