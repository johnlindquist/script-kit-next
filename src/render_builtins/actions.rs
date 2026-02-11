impl ScriptListApp {
    /// Toggle the actions dialog for file search results
    /// Opens a popup with file-specific actions: Open, Show in Finder, Quick Look, etc.
    fn toggle_file_search_actions(
        &mut self,
        file: &file_search::FileResult,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        logging::log("KEY", "Toggling file search actions popup");

        if self.show_actions_popup || is_actions_window_open() {
            // Close the actions popup
            self.show_actions_popup = false;
            self.actions_dialog = None;
            self.file_search_actions_path = None;

            // Close the actions window via spawn
            cx.spawn(async move |_this, cx| {
                cx.update(|cx| {
                    close_actions_window(cx);
                })
                .ok();
            })
            .detach();

            // Use coordinator to restore focus (will pop the overlay and set pending_focus)
            self.pop_focus_overlay(cx);

            // Also directly focus main filter for immediate feedback
            self.focus_main_filter(window, cx);
            logging::log(
                "FOCUS",
                "File search actions closed, focus restored via coordinator",
            );
        } else {
            // Open actions popup for the selected file
            self.show_actions_popup = true;

            // Use coordinator to push overlay - saves current focus state for restore
            self.push_focus_overlay(focus_coordinator::FocusRequest::actions_dialog(), cx);

            // CRITICAL: Transfer focus from Input to main focus_handle
            // This prevents the Input from receiving text (which would go to file search filter)
            // while keeping keyboard focus in main window for routing to actions dialog
            self.focus_handle.focus(window, cx);
            self.gpui_input_focused = false;
            self.focused_input = FocusedInput::ActionsSearch;

            // Store the file path for action handling
            self.file_search_actions_path = Some(file.path.clone());

            // Create file info from the result
            let file_info = file_search::FileInfo::from_result(file);

            // Create the dialog entity
            let theme_arc = std::sync::Arc::clone(&self.theme);
            let dialog = cx.new(|cx| {
                let focus_handle = cx.focus_handle();
                ActionsDialog::with_file(
                    focus_handle,
                    std::sync::Arc::new(|_action_id| {}), // Callback handled via main app
                    &file_info,
                    theme_arc,
                )
            });

            // Store the dialog entity for keyboard routing
            self.actions_dialog = Some(dialog.clone());

            // Set up the on_close callback to restore focus when escape is pressed in ActionsWindow
            // Match what close_actions_popup does for FileSearch host
            let app_entity = cx.entity().clone();
            dialog.update(cx, |d, _cx| {
                d.set_on_close(std::sync::Arc::new(move |cx| {
                    let app_entity = app_entity.clone();
                    cx.defer(move |cx| {
                        app_entity.update(cx, |app, cx| {
                            if !app.show_actions_popup && app.actions_dialog.is_none() {
                                app.file_search_actions_path = None;
                                return;
                            }

                            app.show_actions_popup = false;
                            app.actions_dialog = None;
                            app.file_search_actions_path = None;
                            // Use coordinator to pop overlay and restore previous focus
                            app.pop_focus_overlay(cx);
                            logging::log(
                                "FOCUS",
                                "File search actions closed via escape, focus restored via coordinator",
                            );
                        });
                    });
                }));
            });

            // Get main window bounds and display_id for positioning
            let main_bounds = window.bounds();
            let display_id = window.display(cx).map(|d| d.id());

            logging::log(
                "ACTIONS",
                &format!(
                    "Opening file search actions for: {} (is_dir={})",
                    file_info.name, file_info.is_dir
                ),
            );

            // Open the actions window
            cx.spawn(async move |_this, cx| {
                cx.update(|cx| {
                    match open_actions_window(
                        cx,
                        main_bounds,
                        display_id,
                        dialog,
                        crate::actions::WindowPosition::BottomRight,
                    ) {
                        Ok(_handle) => {
                            logging::log("ACTIONS", "File search actions popup window opened");
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to open actions window: {}", e));
                        }
                    }
                })
                .ok();
            })
            .detach();
        }
        cx.notify();
    }

    /// Toggle the actions dialog for a clipboard history entry
    fn toggle_clipboard_actions(
        &mut self,
        entry: clipboard_history::ClipboardEntryMeta,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        logging::log("KEY", "Toggling clipboard actions popup");

        if self.show_actions_popup || is_actions_window_open() {
            // Close the actions popup
            self.show_actions_popup = false;
            self.actions_dialog = None;

            // Close the actions window via spawn
            cx.spawn(async move |_this, cx| {
                cx.update(|cx| {
                    close_actions_window(cx);
                })
                .ok();
            })
            .detach();

            // Use coordinator to restore focus (will pop the overlay and set pending_focus)
            self.pop_focus_overlay(cx);

            // Also directly focus main filter for immediate feedback
            self.focus_main_filter(window, cx);
            logging::log(
                "FOCUS",
                "Clipboard actions closed, focus restored via coordinator",
            );
        } else {
            // Open actions popup for the selected clipboard entry
            self.show_actions_popup = true;
            self.focused_clipboard_entry_id = Some(entry.id.clone());

            // Use coordinator to push overlay - saves current focus state for restore
            self.push_focus_overlay(focus_coordinator::FocusRequest::actions_dialog(), cx);

            // Transfer focus from Input to main focus_handle for actions routing
            self.focus_handle.focus(window, cx);
            self.gpui_input_focused = false;
            self.focused_input = FocusedInput::ActionsSearch;

            let entry_content_type = entry.content_type;
            let entry_info = crate::actions::ClipboardEntryInfo {
                id: entry.id.clone(),
                content_type: entry.content_type,
                pinned: entry.pinned,
                preview: entry.display_preview(),
                image_dimensions: entry.image_width.zip(entry.image_height),
                frontmost_app_name: None,
            };

            // Create the dialog entity
            let theme_arc = std::sync::Arc::clone(&self.theme);
            let dialog = cx.new(|cx| {
                let focus_handle = cx.focus_handle();
                ActionsDialog::with_clipboard_entry(
                    focus_handle,
                    std::sync::Arc::new(|_action_id| {}), // Callback handled via main app
                    &entry_info,
                    theme_arc,
                )
            });

            // Store the dialog entity for keyboard routing
            self.actions_dialog = Some(dialog.clone());

            // Set up the on_close callback to restore focus when escape is pressed
            let app_entity = cx.entity().clone();
            dialog.update(cx, |d, _cx| {
                d.set_on_close(std::sync::Arc::new(move |cx| {
                    let app_entity = app_entity.clone();
                    cx.defer(move |cx| {
                        app_entity.update(cx, |app, cx| {
                            if !app.show_actions_popup && app.actions_dialog.is_none() {
                                return;
                            }

                            app.show_actions_popup = false;
                            app.actions_dialog = None;
                            // Use coordinator to pop overlay and restore previous focus
                            app.pop_focus_overlay(cx);
                            logging::log(
                                "FOCUS",
                                "Clipboard actions closed via escape, focus restored via coordinator",
                            );
                        });
                    });
                }));
            });

            // Get main window bounds and display_id for positioning
            let main_bounds = window.bounds();
            let display_id = window.display(cx).map(|d| d.id());

            logging::log(
                "ACTIONS",
                &format!(
                    "Opening clipboard actions for entry: {} (type={:?}, pinned={})",
                    entry.id, entry_content_type, entry.pinned
                ),
            );

            // Open the actions window
            cx.spawn(async move |_this, cx| {
                cx.update(|cx| {
                    match open_actions_window(
                        cx,
                        main_bounds,
                        display_id,
                        dialog,
                        crate::actions::WindowPosition::BottomRight,
                    ) {
                        Ok(_handle) => {
                            logging::log("ACTIONS", "Clipboard actions popup window opened");
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to open actions window: {}", e));
                        }
                    }
                })
                .ok();
            })
            .detach();
        }
        cx.notify();
    }
}

#[cfg(test)]
mod on_close_reentrancy_tests {
    use std::fs;

    #[test]
    fn test_render_builtins_actions_on_close_defers_script_list_app_updates() {
        let source = fs::read_to_string("src/render_builtins/actions.rs")
            .expect("Failed to read src/render_builtins/actions.rs");

        let set_on_close_count = source.matches("d.set_on_close(std::sync::Arc::new(move |cx| {").count();
        let defer_count = source.matches("cx.defer(move |cx| {").count();

        assert_eq!(
            set_on_close_count, 2,
            "render_builtins/actions should define two on_close callbacks"
        );
        assert!(
            defer_count >= 2,
            "render_builtins/actions on_close callbacks should defer ScriptListApp updates"
        );
        assert!(
            source.contains("if !app.show_actions_popup && app.actions_dialog.is_none()"),
            "render_builtins/actions on_close callbacks should guard already-closed popup state"
        );
        assert!(
            source.contains("app.file_search_actions_path = None;"),
            "file-search on_close path should clear file_search_actions_path"
        );
    }
}
