        // Root is required for gpui_component's InputState focus tracking
        let window: WindowHandle<Root> = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: None,
                is_movable: true,
                window_background,
                show: false, // Start hidden - only show on hotkey press
                focus: false, // Don't focus on creation
                // CRITICAL: Use PopUp for Raycast-like behavior
                // Creates NSPanel with NonactivatingPanel style, allowing keyboard
                // input without activating the application (preserves previous app focus)
                kind: WindowKind::PopUp,
                ..Default::default()
            },
            |window, cx| {
                logging::log("APP", "Window opened, creating ScriptListApp wrapped in Root");
                let view = cx.new(|cx| ScriptListApp::new(config_for_app, bun_available, window, cx));
                // Store the entity for external access
                *app_entity_for_closure.lock().unwrap_or_else(|e| e.into_inner()) = Some(view.clone());
                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .unwrap();

        // Extract the app entity for use in callbacks
        let app_entity = app_entity_holder.lock().unwrap_or_else(|e| e.into_inner()).clone().expect("App entity should be set");

        // Set initial focus via the Root window
        // We access the app entity within the window context to properly focus it
        let app_entity_for_focus = app_entity.clone();
        window
            .update(cx, |_root, win, root_cx| {
                app_entity_for_focus.update(root_cx, |view, ctx| {
                    let focus_handle = view.focus_handle(ctx);
                    win.focus(&focus_handle, ctx);
                    logging::log("APP", "Focus set on ScriptListApp via Root");

                    // Subscribe to window bounds changes to save position when user drags the window.
                    // This persists the position per-display so it can be restored on next show.
                    // Store subscription in view to keep it alive.
                    view.bounds_subscription = Some(ctx.observe_window_bounds(win, |_view, win, _ctx| {
                        // Only save if window is visible (avoid saving during initial positioning)
                        if is_main_window_visible() {
                            if let Some((x, y, width, height)) = platform::get_main_window_bounds() {
                                let displays = platform::get_macos_displays();
                                let bounds = window_state::PersistedWindowBounds::new(x, y, width, height);
                                if let Some(display) = window_state::find_display_for_bounds(&bounds, &displays) {
                                    logging::log(
                                        "WINDOW_BOUNDS",
                                        &format!(
                                            "Bounds changed - saving position for display {}: ({:.0}, {:.0})",
                                            window_state::display_key(display),
                                            x,
                                            y
                                        ),
                                    );
                                    window_state::save_main_position_for_display(display, bounds);
                                }
                            }
                        }
                        // Suppress unused variable warning - we need win to access window bounds
                        let _ = win;
                    }));

                    // Observe window appearance changes (GPUI fires this when macOS changes light/dark mode)
                    view.appearance_subscription = Some(ctx.observe_window_appearance(win, |view, _win, ctx| {
                        logging::log("APP", "System appearance changed, reloading theme");

                        // Invalidate the cached appearance detection so
                        // detect_system_appearance() gets the fresh value
                        theme::invalidate_appearance_cache();

                        // Reload the theme cache so get_cached_theme() returns
                        // fresh colors (used by vibrancy backgrounds, etc.)
                        let theme = theme::reload_theme_cache();
                        let is_dark = theme.should_use_dark_vibrancy();

                        // Reconfigure vibrancy materials on NSVisualEffectViews
                        platform::configure_window_vibrancy_material_for_appearance(is_dark);

                        // Update all secondary windows (Notes, AI, Actions)
                        platform::update_all_secondary_windows_appearance(is_dark);

                        // Sync gpui-component theme with new system appearance
                        theme::sync_gpui_component_theme(ctx);

                        // Update the app entity theme
                        view.update_theme(ctx);

                        // Notify all registered windows to re-render with new colors
                        windows::notify_all_windows(ctx);
                    }));
                });
            })
            .unwrap();

        // Register the main window with WindowManager before tray init
        // This must happen after GPUI creates the window but before tray creates its windows
        // so we can reliably find our main window by its expected size (~750x500)
        window_manager::find_and_register_main_window();

        // HACK: Swizzle GPUI's BlurredView IMMEDIATELY after window creation
        // GPUI hides the native macOS CAChameleonLayer (vibrancy tint) on every frame.
        // By swizzling now (before any rendering), we preserve the native tint effect.
        // This gives us Raycast/Spotlight-like vibrancy appearance.
        platform::swizzle_gpui_blurred_view();

        // Window starts hidden - no activation, no panel configuration yet
        // Panel will be configured on first show via hotkey
        // WINDOW_VISIBLE is already false by default (static initializer)
        logging::log("HOTKEY", "Window created but not shown (use hotkey to show)");
