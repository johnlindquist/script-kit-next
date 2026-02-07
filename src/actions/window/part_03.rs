
/// Resize the actions window directly using the window reference
/// Use this from defer callbacks where we already have access to the window
pub fn resize_actions_window_direct(
    window: &mut Window,
    cx: &mut App,
    dialog_entity: &Entity<ActionsDialog>,
) {
    // Read dialog state to calculate new height
    let dialog = dialog_entity.read(cx);
    let num_actions = dialog.filtered_actions.len();
    let hide_search = dialog.hide_search;
    let has_header = dialog.context_title.is_some();

    crate::logging::log(
        "ACTIONS",
        &format!(
            "resize_actions_window_direct: num_actions={}, hide_search={}, has_header={}",
            num_actions, hide_search, has_header
        ),
    );

    // Count section headers when using Headers style
    let section_header_count = if dialog.config.section_style == SectionStyle::Headers {
        count_section_headers(&dialog.actions, &dialog.filtered_actions)
    } else {
        0
    };

    // Calculate new height
    let search_box_height = if hide_search {
        0.0
    } else {
        SEARCH_INPUT_HEIGHT
    };
    let header_height = if has_header { HEADER_HEIGHT } else { 0.0 };
    let section_headers_height = section_header_count as f32 * SECTION_HEADER_HEIGHT;
    // When no actions, still need space for "No actions match" message (use 1 row height)
    let min_items_height = if num_actions == 0 {
        ACTION_ITEM_HEIGHT
    } else {
        0.0
    };
    let items_height = (num_actions as f32 * ACTION_ITEM_HEIGHT + section_headers_height)
        .max(min_items_height)
        .min(POPUP_MAX_HEIGHT - search_box_height - header_height);
    let border_height = 2.0;
    let new_height_f32 = items_height + search_box_height + header_height + border_height;

    let current_bounds = window.bounds();
    let current_height_f32: f32 = current_bounds.size.height.into();
    let current_width_f32: f32 = current_bounds.size.width.into();

    crate::logging::log(
        "ACTIONS",
        &format!(
            "resize_actions_window_direct: current={:.0}x{:.0}, target_height={:.0}",
            current_width_f32, current_height_f32, new_height_f32
        ),
    );

    // Skip if height hasn't changed
    if (current_height_f32 - new_height_f32).abs() < 1.0 {
        crate::logging::log(
            "ACTIONS",
            "resize_actions_window_direct: skipping - height unchanged",
        );
        return;
    }

    // Resize via NSWindow to keep bottom pinned
    #[cfg(target_os = "macos")]
    {
        use cocoa::appkit::NSScreen;
        use cocoa::base::nil;
        use cocoa::foundation::{NSPoint, NSRect, NSSize};
        use objc::{msg_send, sel, sel_impl};

        unsafe {
            let ns_app: cocoa::base::id = cocoa::appkit::NSApp();
            let windows: cocoa::base::id = msg_send![ns_app, windows];
            let count: usize = msg_send![windows, count];

            crate::logging::log(
                "ACTIONS",
                &format!(
                    "NSWindow search: looking for {:.0}x{:.0} among {} windows",
                    current_width_f32, current_height_f32, count
                ),
            );

            for i in 0..count {
                let ns_window: cocoa::base::id = msg_send![windows, objectAtIndex: i];
                if ns_window == nil {
                    continue;
                }

                let frame: NSRect = msg_send![ns_window, frame];

                // Match by width (actions window has unique width of 320)
                if (frame.size.width - current_width_f32 as f64).abs() < 2.0
                    && (frame.size.height - current_height_f32 as f64).abs() < 2.0
                {
                    let window_screen: cocoa::base::id = msg_send![ns_window, screen];
                    if window_screen == nil {
                        let screens: cocoa::base::id = NSScreen::screens(nil);
                        let _primary: cocoa::base::id = msg_send![screens, objectAtIndex: 0u64];
                    }

                    // Keep bottom fixed by keeping origin.y the same
                    let new_frame = NSRect::new(
                        NSPoint::new(frame.origin.x, frame.origin.y),
                        NSSize::new(frame.size.width, new_height_f32 as f64),
                    );

                    let animate = true;
                    let _: () =
                        msg_send![ns_window, setFrame:new_frame display:true animate:animate];

                    crate::logging::log(
                        "ACTIONS",
                        &format!(
                            "Resized actions window (bottom pinned): height {:.0} -> {:.0}, animate={}",
                            current_height_f32, new_height_f32, animate
                        ),
                    );
                    break;
                }
            }
        }
    }

    // Also tell GPUI about the new size
    window.resize(gpui::Size {
        width: current_bounds.size.width,
        height: px(new_height_f32),
    });

    crate::logging::log(
        "ACTIONS",
        &format!(
            "resize_actions_window_direct complete: {} items, height={:.0}",
            num_actions, new_height_f32
        ),
    );
}

/// Resize the actions window to fit the current number of filtered actions
/// Call this after filtering changes the action count
///
/// The window is "pinned to bottom" - the search input stays in place and
/// the window shrinks/grows from the top.
pub fn resize_actions_window(cx: &mut App, dialog_entity: &Entity<ActionsDialog>) {
    crate::logging::log("ACTIONS", "resize_actions_window called");
    if let Some(handle) = get_actions_window_handle() {
        // Read dialog state to calculate new height
        let dialog = dialog_entity.read(cx);
        let num_actions = dialog.filtered_actions.len();
        let hide_search = dialog.hide_search;
        let has_header = dialog.context_title.is_some();
        crate::logging::log(
            "ACTIONS",
            &format!(
                "resize_actions_window: num_actions={}, hide_search={}, has_header={}",
                num_actions, hide_search, has_header
            ),
        );

        // Count section headers when using Headers style
        let section_header_count = if dialog.config.section_style == SectionStyle::Headers {
            count_section_headers(&dialog.actions, &dialog.filtered_actions)
        } else {
            0
        };

        // Calculate new height (same logic as open_actions_window)
        let search_box_height = if hide_search {
            0.0
        } else {
            SEARCH_INPUT_HEIGHT
        };
        let header_height = if has_header { HEADER_HEIGHT } else { 0.0 };
        let section_headers_height = section_header_count as f32 * SECTION_HEADER_HEIGHT;
        // When no actions, still need space for "No actions match" message
        let min_items_height = if num_actions == 0 {
            ACTION_ITEM_HEIGHT
        } else {
            0.0
        };
        let items_height = (num_actions as f32 * ACTION_ITEM_HEIGHT + section_headers_height)
            .max(min_items_height)
            .min(POPUP_MAX_HEIGHT - search_box_height - header_height);
        let border_height = 2.0; // top + bottom border
        let new_height_f32 = items_height + search_box_height + header_height + border_height;

        let update_result = handle.update(cx, |_root, window, cx| {
            let current_bounds = window.bounds();
            let current_height_f32: f32 = current_bounds.size.height.into();
            let current_width_f32: f32 = current_bounds.size.width.into();

            crate::logging::log(
                "ACTIONS",
                &format!(
                    "resize_actions_window inside update: current={:.0}x{:.0}, target_height={:.0}",
                    current_width_f32, current_height_f32, new_height_f32
                ),
            );

            // Skip if height hasn't changed
            if (current_height_f32 - new_height_f32).abs() < 1.0 {
                crate::logging::log(
                    "ACTIONS",
                    "resize_actions_window: skipping - height unchanged",
                );
                return;
            }

            // "Pin to bottom": keep the bottom edge fixed
            // In macOS screen coords (bottom-left origin), the bottom of the window
            // is at frame.origin.y. When we change height, we keep origin.y the same
            // and only change height - this naturally keeps the bottom fixed.
            #[cfg(target_os = "macos")]
            {
                use cocoa::appkit::NSScreen;
                use cocoa::base::nil;
                use cocoa::foundation::{NSPoint, NSRect, NSSize};
                use objc::{msg_send, sel, sel_impl};

                unsafe {
                    let ns_app: cocoa::base::id = cocoa::appkit::NSApp();
                    let windows: cocoa::base::id = msg_send![ns_app, windows];
                    let count: usize = msg_send![windows, count];

                    crate::logging::log(
                        "ACTIONS",
                        &format!(
                            "NSWindow search: looking for {:.0}x{:.0} among {} windows",
                            current_width_f32, current_height_f32, count
                        ),
                    );

                    // Find our actions window by matching current dimensions
                    let mut found = false;
                    for i in 0..count {
                        let ns_window: cocoa::base::id = msg_send![windows, objectAtIndex: i];
                        if ns_window == nil {
                            continue;
                        }

                        let frame: NSRect = msg_send![ns_window, frame];

                        // Match by width (actions window has unique width)
                        if (frame.size.width - current_width_f32 as f64).abs() < 2.0
                            && (frame.size.height - current_height_f32 as f64).abs() < 2.0
                        {
                            // Get the screen this window is on (NOT primary screen!)
                            let window_screen: cocoa::base::id = msg_send![ns_window, screen];
                            if window_screen == nil {
                                // Fallback to primary if no screen
                                let screens: cocoa::base::id = NSScreen::screens(nil);
                                let _primary: cocoa::base::id =
                                    msg_send![screens, objectAtIndex: 0u64];
                            }

                            // In macOS coords (bottom-left origin):
                            // - frame.origin.y is the BOTTOM of the window
                            // - To keep bottom fixed, we keep origin.y the same
                            // - Only change the height
                            let new_frame = NSRect::new(
                                NSPoint::new(frame.origin.x, frame.origin.y),
                                NSSize::new(frame.size.width, new_height_f32 as f64),
                            );

                            let animate = true;
                            let _: () =
                                msg_send![ns_window, setFrame:new_frame display:true animate:animate];

                            crate::logging::log(
                                "ACTIONS",
                                &format!(
                                    "Resized actions window (bottom pinned): height {:.0} -> {:.0}, animate={}",
                                    current_height_f32, new_height_f32, animate
                                ),
                            );
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        crate::logging::log(
                            "ACTIONS",
                            &format!(
                                "NSWindow NOT FOUND - no window matches {:.0}x{:.0}",
                                current_width_f32, current_height_f32
                            ),
                        );
                    }
                }
            }

            // Also tell GPUI about the new size
            window.resize(Size {
                width: current_bounds.size.width,
                height: px(new_height_f32),
            });
            cx.notify();
        });

        if let Err(e) = update_result {
            crate::logging::log("ACTIONS", &format!("handle.update FAILED: {:?}", e));
        }

        crate::logging::log(
            "ACTIONS",
            &format!(
                "Resized actions window: {} items, height={:.0}",
                num_actions, new_height_f32
            ),
        );
    }
}
