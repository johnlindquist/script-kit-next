impl ScriptListApp {
    fn dictation_history_actions_dialog_config(
        placeholder: String,
    ) -> crate::actions::ActionsDialogConfig {
        crate::actions::ActionsDialogConfig {
            search_position: crate::actions::SearchPosition::Top,
            section_style: crate::actions::SectionStyle::Headers,
            anchor: crate::actions::AnchorPosition::Top,
            show_icons: true,
            search_placeholder: Some(placeholder),
            show_context_header: false,
            ..crate::actions::ActionsDialogConfig::default()
        }
    }

    fn dictation_history_actions_for_dialog() -> Vec<crate::actions::Action> {
        use crate::actions::{Action, ActionCategory};
        use crate::designs::icon_variations::IconName;

        vec![
            Action::new(
                "dictation_history_paste",
                "Paste to Frontmost App",
                Some("Hide Script Kit and paste this transcript into the active app".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("↵")
            .with_section("Reuse")
            .with_icon(IconName::ArrowRight),
            Action::new(
                "dictation_history_attach_to_ai",
                "Attach to Agent Chat",
                Some("Open Agent Chat and stage this transcript in the composer".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌃⌘A")
            .with_section("Reuse")
            .with_icon(IconName::MessageCircle),
            Action::new(
                "dictation_history_save_note",
                "Save as Note",
                Some("Create a new note pre-filled with this transcript".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_section("Reuse")
            .with_icon(IconName::Plus),
            Action::new(
                "dictation_history_copy",
                "Copy Transcript",
                Some("Copy this transcript to the clipboard".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘C")
            .with_section("Reuse")
            .with_icon(IconName::Copy),
            Action::new(
                "dictation_history_delete",
                "Delete from History",
                Some("Remove this saved transcript from dictation history".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘⌫")
            .with_section("Manage")
            .with_icon(IconName::Trash),
        ]
    }

    fn favorites_actions_for_dialog() -> Vec<crate::actions::Action> {
        use crate::actions::{Action, ActionCategory};
        use crate::designs::icon_variations::IconName;

        vec![
            Action::new(
                "favorites_run",
                "Run",
                Some("Run the selected favorite".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("↵")
            .with_section("Actions")
            .with_icon(IconName::PlayFilled),
            Action::new(
                "favorites_edit_script",
                "Edit Script",
                Some("Open the selected favorite in the configured editor".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_section("Actions")
            .with_icon(IconName::Pencil),
            Action::new(
                "favorites_copy_script_url",
                "Copy Script URL",
                Some("Copy the selected favorite's scriptkit://run URL".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_section("Actions")
            .with_icon(IconName::Copy),
            Action::new(
                "favorites_move_up",
                "Move Up",
                Some("Move the selected favorite up".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("U")
            .with_section("Actions")
            .with_icon(IconName::ArrowUp),
            Action::new(
                "favorites_move_down",
                "Move Down",
                Some("Move the selected favorite down".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("J")
            .with_section("Actions")
            .with_icon(IconName::ArrowDown),
            Action::new(
                "favorites_remove",
                "Remove from Favorites",
                Some("Remove the selected favorite".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("D")
            .with_section("Manage")
            .with_icon(IconName::Trash),
        ]
    }

    fn theme_chooser_actions_for_dialog() -> Vec<crate::actions::Action> {
        use crate::actions::{Action, ActionCategory};
        use crate::designs::icon_variations::IconName;

        vec![
            Action::new(
                "theme_chooser_done",
                "Done",
                Some("Persist the current theme and return to the launcher".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("↵")
            .with_section("Theme")
            .with_icon(IconName::Check),
            Action::new(
                "theme_chooser_undo_close",
                "Undo Changes and Close",
                Some("Restore the theme from when Theme Designer opened".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_section("Theme")
            .with_icon(IconName::Close),
            Action::new(
                "theme_chooser_remix",
                "Surprise Me",
                Some("Remix accent, opacity, and material from the current theme".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘J")
            .with_section("Customize")
            .with_icon(IconName::BoltFilled),
            Action::new(
                "theme_chooser_reset",
                "Reset to Defaults",
                Some("Reset customization controls to the selected preset".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘R")
            .with_section("Customize")
            .with_icon(IconName::Refresh),
            Action::new(
                "theme_chooser_accent_previous",
                "Previous Accent Color",
                Some("Move to the previous accent swatch".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘[")
            .with_section("Customize")
            .with_icon(IconName::ChevronRight),
            Action::new(
                "theme_chooser_accent_next",
                "Next Accent Color",
                Some("Move to the next accent swatch".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘]")
            .with_section("Customize")
            .with_icon(IconName::ArrowRight),
            Action::new(
                "theme_chooser_opacity_decrease",
                "Decrease Surface Opacity",
                Some("Use the next lower opacity preset".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘-")
            .with_section("Customize")
            .with_icon(IconName::ArrowDown),
            Action::new(
                "theme_chooser_opacity_increase",
                "Increase Surface Opacity",
                Some("Use the next higher opacity preset".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘=")
            .with_section("Customize")
            .with_icon(IconName::ArrowUp),
            Action::new(
                "theme_chooser_vibrancy_toggle",
                "Toggle Vibrancy Blur",
                Some("Turn vibrancy blur on or off".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘B")
            .with_section("Customize")
            .with_icon(IconName::EyeOff),
            Action::new(
                "theme_chooser_material_cycle",
                "Cycle Vibrancy Material",
                Some("Switch to the next AppKit vibrancy material".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘M")
            .with_section("Customize")
            .with_icon(IconName::Sidebar),
            Action::new(
                "theme_chooser_font_size_decrease",
                "Decrease UI Font Size",
                Some("Use the next smaller UI font preset".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_section("Typography")
            .with_icon(IconName::ArrowDown),
            Action::new(
                "theme_chooser_font_size_increase",
                "Increase UI Font Size",
                Some("Use the next larger UI font preset".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_section("Typography")
            .with_icon(IconName::ArrowUp),
        ]
    }

    fn toggle_theme_chooser_actions(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        logging::log("KEY", "Toggling theme chooser actions popup");

        if self.show_actions_popup || is_actions_window_open() {
            self.close_actions_popup(ActionsDialogHost::ThemeChooser, window, cx);
            return;
        }

        self.mark_actions_popup_opening();
        self.push_focus_overlay(focus_coordinator::FocusRequest::actions_dialog(), cx);
        self.focus_handle.focus(window, cx);
        self.gpui_input_focused = false;
        self.focused_input = FocusedInput::ActionsSearch;

        let theme_arc = std::sync::Arc::clone(&self.theme);
        let actions = Self::theme_chooser_actions_for_dialog();
        let dialog = cx.new(move |cx| {
            let focus_handle = cx.focus_handle();
            let mut dialog = ActionsDialog::with_config(
                focus_handle,
                std::sync::Arc::new(|_action_id| {}),
                actions,
                theme_arc,
                crate::actions::ActionsDialogConfig {
                    search_position: crate::actions::SearchPosition::Top,
                    section_style: crate::actions::SectionStyle::Headers,
                    anchor: crate::actions::AnchorPosition::Top,
                    show_icons: true,
                    search_placeholder: Some("Theme Designer actions".to_string()),
                    show_context_header: false,
                    ..crate::actions::ActionsDialogConfig::default()
                },
            );
            dialog.set_match_main_window_background(true);
            dialog
        });

        self.actions_dialog = Some(dialog.clone());

        let app_entity = cx.entity().clone();
        dialog.update(cx, |d, _cx| {
            d.set_on_activation(Self::make_actions_dialog_activation_callback(
                app_entity.clone(),
                ActionsDialogHost::ThemeChooser,
            ));
            d.set_on_close(Self::make_actions_window_on_close_callback(
                app_entity,
                ActionsDialogHost::ThemeChooser,
                "Theme chooser actions closed via escape, focus restored via coordinator",
            ));
        });

        let parent_window_handle = window.window_handle();
        let main_bounds = window.bounds();
        let display_id = window.display(cx).map(|d| d.id());

        Self::spawn_open_actions_window(
            cx,
            parent_window_handle,
            main_bounds,
            display_id,
            dialog,
            crate::actions::WindowPosition::TopCenter,
            "Theme chooser actions popup window opened",
            "Failed to open theme chooser actions window",
        );

        cx.notify();
    }

    fn toggle_favorites_actions(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        logging::log("KEY", "Toggling favorites actions popup");

        if self.show_actions_popup || is_actions_window_open() {
            self.close_actions_popup(ActionsDialogHost::Favorites, window, cx);
            return;
        }

        let Some(selected_id) = self.selected_favorite_id() else {
            logging::log("ACTIONS", "Favorites actions ignored: no selected favorite");
            return;
        };

        self.mark_actions_popup_opening();
        self.push_focus_overlay(focus_coordinator::FocusRequest::actions_dialog(), cx);
        self.focus_handle.focus(window, cx);
        self.gpui_input_focused = false;
        self.focused_input = FocusedInput::ActionsSearch;

        let theme_arc = std::sync::Arc::clone(&self.theme);
        let actions = Self::favorites_actions_for_dialog();
        let dialog = cx.new(move |cx| {
            let focus_handle = cx.focus_handle();
            let mut dialog = ActionsDialog::with_config(
                focus_handle,
                std::sync::Arc::new(|_action_id| {}),
                actions,
                theme_arc,
                crate::actions::ActionsDialogConfig {
                    search_position: crate::actions::SearchPosition::Top,
                    section_style: crate::actions::SectionStyle::Headers,
                    anchor: crate::actions::AnchorPosition::Top,
                    show_icons: true,
                    search_placeholder: Some(selected_id),
                    show_context_header: false,
                    ..crate::actions::ActionsDialogConfig::default()
                },
            );
            dialog.set_match_main_window_background(true);
            dialog
        });

        self.actions_dialog = Some(dialog.clone());

        let app_entity = cx.entity().clone();
        dialog.update(cx, |d, _cx| {
            d.set_on_activation(Self::make_actions_dialog_activation_callback(
                app_entity.clone(),
                ActionsDialogHost::Favorites,
            ));
            d.set_on_close(Self::make_actions_window_on_close_callback(
                app_entity,
                ActionsDialogHost::Favorites,
                "Favorites actions closed via escape, focus restored via coordinator",
            ));
        });

        let parent_window_handle = window.window_handle();
        let main_bounds = window.bounds();
        let display_id = window.display(cx).map(|d| d.id());

        Self::spawn_open_actions_window(
            cx,
            parent_window_handle,
            main_bounds,
            display_id,
            dialog,
            crate::actions::WindowPosition::TopCenter,
            "Favorites actions popup window opened",
            "Failed to open favorites actions window",
        );

        cx.notify();
    }

    fn toggle_dictation_history_actions(
        &mut self,
        entry: crate::dictation::DictationHistoryEntry,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        logging::log("KEY", "Toggling dictation history actions popup");

        if self.show_actions_popup || is_actions_window_open() {
            self.close_actions_popup(ActionsDialogHost::DictationHistory, window, cx);
            return;
        }

        self.mark_actions_popup_opening();
        self.push_focus_overlay(focus_coordinator::FocusRequest::actions_dialog(), cx);
        self.focus_handle.focus(window, cx);
        self.gpui_input_focused = false;
        self.focused_input = FocusedInput::ActionsSearch;

        let theme_arc = std::sync::Arc::clone(&self.theme);
        let placeholder = entry.preview.clone();
        let actions = Self::dictation_history_actions_for_dialog();
        let dialog = cx.new(move |cx| {
            let focus_handle = cx.focus_handle();
            let mut dialog = ActionsDialog::with_config(
                focus_handle,
                std::sync::Arc::new(|_action_id| {}),
                actions,
                theme_arc,
                Self::dictation_history_actions_dialog_config(placeholder),
            );
            dialog.set_match_main_window_background(true);
            dialog
        });

        self.actions_dialog = Some(dialog.clone());

        let app_entity = cx.entity().clone();
        dialog.update(cx, |d, _cx| {
            d.set_on_activation(Self::make_actions_dialog_activation_callback(
                app_entity.clone(),
                ActionsDialogHost::DictationHistory,
            ));
            d.set_on_close(Self::make_actions_window_on_close_callback(
                app_entity,
                ActionsDialogHost::DictationHistory,
                "Dictation history actions closed via escape, focus restored via coordinator",
            ));
        });

        let parent_window_handle = window.window_handle();
        let main_bounds = window.bounds();
        let display_id = window.display(cx).map(|d| d.id());

        Self::spawn_open_actions_window(
            cx,
            parent_window_handle,
            main_bounds,
            display_id,
            dialog,
            crate::actions::WindowPosition::TopCenter,
            "Dictation history actions popup window opened",
            "Failed to open dictation history actions window",
        );

        cx.notify();
    }

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
            self.mark_actions_popup_closed();
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

        // Run 14 Pass 1 — story `actions-debounce-builtins-cross-host-live`:
        // when neither a file nor a directory context is available the
        // dialog used to silently close. Now we always open the dialog —
        // `with_file_search_context` will fall through to the global
        // actions block (Pass 3 of Run 13) so the user sees that Cmd+K
        // landed even when the file-search input is empty.

        // Open actions popup
        self.mark_actions_popup_opening();

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

            dialog.set_match_main_window_background(true);
            dialog
        });

        // Store the dialog entity for keyboard routing
        self.actions_dialog = Some(dialog.clone());

        // Set up the on_close callback to restore focus when escape is pressed in ActionsWindow
        // Match what close_actions_popup does for FileSearch host
        let app_entity = cx.entity().clone();
        dialog.update(cx, |d, _cx| {
            d.set_on_activation(Self::make_actions_dialog_activation_callback(
                app_entity.clone(),
                ActionsDialogHost::FileSearch,
            ));
            d.set_on_close(std::sync::Arc::new(move |cx| {
                let app_entity = app_entity.clone();
                cx.defer(move |cx| {
                    app_entity.update(cx, |app, cx| {
                        if !app.show_actions_popup && app.actions_dialog.is_none() {
                            app.file_search_actions_path = None;
                            return;
                        }

                        app.mark_actions_popup_closed();
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
        let parent_window_handle = window.window_handle();
        let main_bounds = window.bounds();
        let display_id = window.display(cx).map(|d| d.id());
        logging::log(
            "ACTIONS",
            &format!(
                "Opening file search actions: file={}, dir={}",
                selected_file.map(|f| f.name.as_str()).unwrap_or("none"),
                dir_info.as_ref().map(|d| d.name.as_str()).unwrap_or("none"),
            ),
        );

        // Open the actions window — centered like the mini main menu
        let parent_automation_id = crate::windows::focused_automation_window_id();
        cx.spawn(async move |_this, cx| {
            cx.update(|cx| {
                match open_actions_window(
                    cx,
                    parent_window_handle,
                    main_bounds,
                    display_id,
                    dialog,
                    crate::actions::WindowPosition::TopCenter,
                    parent_automation_id.as_deref(),
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
            self.mark_actions_popup_closed();

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
            self.mark_actions_popup_opening();
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
                d.set_on_activation(Self::make_actions_dialog_activation_callback(
                    app_entity.clone(),
                    ActionsDialogHost::ClipboardHistory,
                ));
                d.set_on_close(std::sync::Arc::new(move |cx| {
                    let app_entity = app_entity.clone();
                    cx.defer(move |cx| {
                        app_entity.update(cx, |app, cx| {
                            if !app.show_actions_popup && app.actions_dialog.is_none() {
                                return;
                            }

                            app.mark_actions_popup_closed();
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
            let parent_window_handle = window.window_handle();
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
            let parent_automation_id = crate::windows::focused_automation_window_id();
            cx.spawn(async move |_this, cx| {
                cx.update(|cx| {
                    match open_actions_window(
                        cx,
                        parent_window_handle,
                        main_bounds,
                        display_id,
                        dialog,
                        crate::actions::WindowPosition::TopCenter,
                        parent_automation_id.as_deref(),
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

        let set_on_close_count = source
            .matches("d.set_on_close(std::sync::Arc::new(move |cx| {")
            .count();
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
