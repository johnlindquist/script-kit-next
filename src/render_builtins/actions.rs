impl ScriptListApp {
    /// Toggle the actions dialog for file search results.
    ///
    /// When a row is selected, shows both row-scoped file actions and
    /// current-directory actions.  When no row is selected but a browsed
    /// directory exists, shows directory-only actions.
    fn toggle_file_search_actions(
        &mut self,
        selected_file: Option<&file_search::FileResult>,
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
                });
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
            cx.notify();
            return;
        }

        // Build current-directory context if browsing a concrete directory
        let dir_path = self.current_file_search_directory_abs();
        let dir_info = dir_path.as_ref().map(|path| {
            let dir_name = std::path::Path::new(path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| path.clone());
            crate::actions::FileSearchDirectoryInfo::new(
                path.clone(),
                dir_name,
                self.file_search_sort_mode,
            )
        });

        // Need at least one context source to open the dialog
        if selected_file.is_none() && dir_info.is_none() {
            return;
        }

        // Open actions popup
        self.show_actions_popup = true;

        // Use coordinator to push overlay - saves current focus state for restore
        self.push_focus_overlay(focus_coordinator::FocusRequest::actions_dialog(), cx);

        // CRITICAL: Transfer focus from Input to main focus_handle
        self.focus_handle.focus(window, cx);
        self.gpui_input_focused = false;
        self.focused_input = FocusedInput::ActionsSearch;

        // Store the file path for action handling
        self.file_search_actions_path = selected_file.map(|file| file.path.clone());

        // Create file info from the result
        let file_info = selected_file.map(file_search::FileInfo::from_result);

        // Determine placeholder text — show both scopes when available
        let placeholder_text = match (file_info.as_ref(), dir_info.as_ref()) {
            (Some(file), Some(dir)) => format!("{} · {}", file.name, dir.name),
            (Some(file), None) => file.name.clone(),
            (None, Some(dir)) => dir.name.clone(),
            (None, None) => "Actions".to_string(),
        };

        // Create the dialog entity
        let theme_arc = std::sync::Arc::clone(&self.theme);
        let dialog = cx.new(|cx| {
            let focus_handle = cx.focus_handle();
            let mut dialog = ActionsDialog::with_file_search_context(
                focus_handle,
                std::sync::Arc::new(|_action_id| {}), // Callback handled via main app
                file_info.as_ref(),
                dir_info.as_ref(),
                theme_arc,
            );

            // Match the mini main menu's actions dialog config:
            // search at top, anchor top, centered, icons visible
            dialog.set_config(crate::actions::ActionsDialogConfig {
                search_position: crate::actions::SearchPosition::Top,
                section_style: crate::actions::SectionStyle::Headers,
                anchor: crate::actions::AnchorPosition::Top,
                show_icons: true,
                search_placeholder: Some(placeholder_text),
                show_context_header: false,
                ..crate::actions::ActionsDialogConfig::default()
            });

            dialog
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
        let parent_window_handle = window.window_handle();

        logging::log(
            "ACTIONS",
            &format!(
                "Opening file search actions: file={}, dir={}",
                selected_file
                    .map(|f| f.name.as_str())
                    .unwrap_or("none"),
                dir_info
                    .as_ref()
                    .map(|d| d.name.as_str())
                    .unwrap_or("none"),
            ),
        );

        // Open the actions window — centered like the mini main menu
        cx.spawn(async move |_this, cx| {
            cx.update(|cx| {
                match open_actions_window(
                    cx,
                    parent_window_handle,
                    main_bounds,
                    display_id,
                    dialog,
                    crate::actions::WindowPosition::TopCenter,
                ) {
                    Ok(_handle) => {
                        logging::log("ACTIONS", "File search actions popup window opened");
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to open actions window: {}", e));
                    }
                }
            });
        })
        .detach();

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
                });
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
            let entry_placeholder = entry_info.preview.clone();
            let entry_info_for_dialog = entry_info.clone();
            let dialog = cx.new(move |cx| {
                let focus_handle = cx.focus_handle();
                let mut dialog = ActionsDialog::with_clipboard_entry(
                    focus_handle,
                    std::sync::Arc::new(|_action_id| {}), // Callback handled via main app
                    &entry_info_for_dialog,
                    theme_arc,
                );

                // Match the mini main menu's actions dialog config:
                // search at top, anchor top, centered, icons visible
                dialog.set_config(crate::actions::ActionsDialogConfig {
                    search_position: crate::actions::SearchPosition::Top,
                    section_style: crate::actions::SectionStyle::Headers,
                    anchor: crate::actions::AnchorPosition::Top,
                    show_icons: true,
                    search_placeholder: Some(entry_placeholder),
                    show_context_header: false,
                    ..crate::actions::ActionsDialogConfig::default()
                });

                dialog
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
            let parent_window_handle = window.window_handle();

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
                        parent_window_handle,
                        main_bounds,
                        display_id,
                        dialog,
                        crate::actions::WindowPosition::TopCenter,
                    ) {
                        Ok(_handle) => {
                            logging::log("ACTIONS", "Clipboard actions popup window opened");
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to open actions window: {}", e));
                        }
                    }
                });
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

    #[test]
    fn test_render_builtins_actions_clipboard_popup_uses_mini_menu_contract() {
        let source = fs::read_to_string("src/render_builtins/actions.rs")
            .expect("Failed to read src/render_builtins/actions.rs");

        let clipboard_fn = source
            .split("fn toggle_clipboard_actions(")
            .nth(1)
            .expect("missing toggle_clipboard_actions");

        assert!(
            clipboard_fn.contains("dialog.set_config(crate::actions::ActionsDialogConfig {"),
            "clipboard actions should set an explicit ActionsDialogConfig"
        );
        assert!(
            clipboard_fn.contains("search_position: crate::actions::SearchPosition::Top"),
            "clipboard actions should place search at the top"
        );
        assert!(
            clipboard_fn.contains("section_style: crate::actions::SectionStyle::Headers"),
            "clipboard actions should use section headers"
        );
        assert!(
            clipboard_fn.contains("anchor: crate::actions::AnchorPosition::Top"),
            "clipboard actions should anchor to the top"
        );
        assert!(
            clipboard_fn.contains("show_icons: true"),
            "clipboard actions should show icons"
        );
        assert!(
            clipboard_fn.contains("show_context_header: false"),
            "clipboard actions should hide the context header"
        );
        assert!(
            clipboard_fn.contains("crate::actions::WindowPosition::TopCenter"),
            "clipboard actions should open in the top-center mini-menu position"
        );
        assert!(
            !clipboard_fn.contains("crate::actions::WindowPosition::BottomRight"),
            "clipboard actions should not open in the bottom-right position"
        );
    }

}
