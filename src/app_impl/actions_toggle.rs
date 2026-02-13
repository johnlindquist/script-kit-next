use super::*;

pub(crate) const TERM_PROMPT_CLEAR_ACTION_ID: &str = "clear";
pub(crate) const TERM_PROMPT_CLEAR_SHORTCUT: &str = "⌘K";
pub(crate) const TERM_PROMPT_ACTIONS_TOGGLE_ACTION_ID: &str = "term_prompt_toggle_actions";
pub(crate) const TERM_PROMPT_ACTIONS_TOGGLE_SHORTCUT: &str = "⌘⇧K";
const TERM_PROMPT_SCROLL_TO_BOTTOM_ACTION_ID: &str = "scroll_to_bottom";

fn terminal_action_sort_key(action_id: &str) -> Option<usize> {
    match action_id {
        "copy" => Some(0),
        "copy_all" => Some(1),
        "copy_last_command" => Some(2),
        "copy_last_output" => Some(3),
        "paste" => Some(4),
        "select_all" => Some(5),
        "find" => Some(6),
        "scroll_to_top" => Some(7),
        TERM_PROMPT_SCROLL_TO_BOTTOM_ACTION_ID => Some(8),
        TERM_PROMPT_CLEAR_ACTION_ID => Some(9),
        "reset" => Some(10),
        _ => None,
    }
}

fn terminal_action_section(action_id: &str) -> Option<&'static str> {
    match action_id {
        "copy" | "copy_all" | "copy_last_command" | "copy_last_output" | "paste" | "select_all" => {
            Some("Clipboard")
        }
        "find" => Some("Search"),
        "scroll_to_top" | TERM_PROMPT_SCROLL_TO_BOTTOM_ACTION_ID => Some("Navigation"),
        TERM_PROMPT_CLEAR_ACTION_ID | "reset" => Some("Session"),
        _ => None,
    }
}

fn terminal_action_icon(action_id: &str) -> Option<crate::designs::icon_variations::IconName> {
    use crate::designs::icon_variations::IconName;

    match action_id {
        "copy" | "copy_all" | "copy_last_command" | "copy_last_output" | "paste" | "select_all" => {
            Some(IconName::Copy)
        }
        "find" => Some(IconName::MagnifyingGlass),
        "scroll_to_top" => Some(IconName::ArrowUp),
        TERM_PROMPT_SCROLL_TO_BOTTOM_ACTION_ID => Some(IconName::ArrowDown),
        TERM_PROMPT_CLEAR_ACTION_ID => Some(IconName::Trash),
        "reset" => Some(IconName::Refresh),
        _ => None,
    }
}

fn terminal_actions_dialog_config() -> crate::actions::ActionsDialogConfig {
    use crate::actions::{ActionsDialogConfig, AnchorPosition, SearchPosition, SectionStyle};

    ActionsDialogConfig {
        search_position: SearchPosition::Top,
        section_style: SectionStyle::Headers,
        anchor: AnchorPosition::Top,
        show_icons: true,
        show_footer: true,
    }
}

fn terminal_actions_for_dialog() -> Vec<crate::actions::Action> {
    use crate::actions::{Action, ActionCategory};
    use crate::designs::icon_variations::IconName;

    let mut actions: Vec<Action> = crate::terminal::get_terminal_commands()
        .into_iter()
        .filter_map(|cmd| {
            let action_id = cmd.action.id();
            let sort_key = terminal_action_sort_key(action_id)?;

            let shortcut = if action_id == TERM_PROMPT_CLEAR_ACTION_ID {
                Some(TERM_PROMPT_CLEAR_SHORTCUT.to_string())
            } else {
                cmd.shortcut.clone()
            };

            let mut action = Action::new(
                action_id,
                cmd.name.clone(),
                Some(cmd.description.clone()),
                ActionCategory::Terminal,
            )
            .with_shortcut_opt(shortcut);

            if let Some(section) = terminal_action_section(action_id) {
                action = action.with_section(section);
            }

            if let Some(icon) = terminal_action_icon(action_id) {
                action = action.with_icon(icon);
            }

            Some((sort_key, action))
        })
        .map(|(_sort_key, action)| action)
        .collect();

    if !actions
        .iter()
        .any(|action| action.id == TERM_PROMPT_SCROLL_TO_BOTTOM_ACTION_ID)
    {
        actions.push(
            Action::new(
                TERM_PROMPT_SCROLL_TO_BOTTOM_ACTION_ID,
                "Scroll to Bottom",
                Some("Jump to the bottom (latest output)".to_string()),
                ActionCategory::Terminal,
            )
            .with_shortcut("⌘↓")
            .with_section("Navigation")
            .with_icon(IconName::ArrowDown),
        );
    }

    actions
        .sort_by_key(|action| terminal_action_sort_key(action.id.as_str()).unwrap_or(usize::MAX));

    actions.push(
        Action::new(
            TERM_PROMPT_ACTIONS_TOGGLE_ACTION_ID,
            "Toggle Actions",
            Some("Open or close the terminal actions palette".to_string()),
            ActionCategory::Terminal,
        )
        .with_shortcut(TERM_PROMPT_ACTIONS_TOGGLE_SHORTCUT)
        .with_icon(IconName::Settings),
    );

    actions
}

impl ScriptListApp {
    pub(crate) fn toggle_actions(&mut self, cx: &mut Context<Self>, window: &mut Window) {
        let popup_state = self.show_actions_popup;
        let window_open = is_actions_window_open();
        logging::log(
            "KEY",
            &format!(
                "Toggling actions popup (show_actions_popup={}, is_actions_window_open={})",
                popup_state, window_open
            ),
        );
        if self.show_actions_popup || is_actions_window_open() {
            self.close_actions_popup(ActionsDialogHost::MainList, window, cx);
        } else {
            if !self.has_actions() {
                return;
            }
            self.resync_filter_input_after_actions_if_needed(window, cx);
            // Open actions as a separate window with vibrancy blur
            self.show_actions_popup = true;

            // Use coordinator to push overlay - saves current focus state for restore
            self.push_focus_overlay(focus_coordinator::FocusRequest::actions_dialog(), cx);

            // CRITICAL: Transfer focus from Input to main focus_handle
            // This prevents the Input from receiving text (which would go to main filter)
            // while keeping keyboard focus in main window for routing to actions dialog
            self.focus_handle.focus(window, cx);
            self.gpui_input_focused = false;

            let script_info = self.get_focused_script_info();

            // Get the full scriptlet with actions if focused item is a scriptlet
            let focused_scriptlet = self.get_focused_scriptlet_with_actions();

            // Create the dialog entity HERE in main app (for keyboard routing)
            let theme_arc = std::sync::Arc::clone(&self.theme);
            // Create the dialog entity (search input shown at bottom, Raycast-style)
            let dialog = cx.new(|cx| {
                let focus_handle = cx.focus_handle();
                let mut dialog = ActionsDialog::with_script(
                    focus_handle,
                    std::sync::Arc::new(|_action_id| {}), // Callback handled via main app
                    script_info.clone(),
                    theme_arc,
                );

                // If we have a scriptlet with actions, pass it to the dialog
                if let Some(ref scriptlet) = focused_scriptlet {
                    dialog.set_focused_scriptlet(script_info.clone(), Some(scriptlet.clone()));
                }

                dialog
            });

            // Store the dialog entity for keyboard routing
            self.actions_dialog = Some(dialog.clone());

            // Set up the on_close callback to restore focus when escape is pressed in ActionsWindow
            // This ensures the same cleanup happens whether closing via Cmd+K toggle or Escape
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
                            app.mark_filter_resync_after_actions_if_needed();
                            // Use coordinator to pop overlay and restore previous focus
                            app.pop_focus_overlay(cx);
                            logging::log(
                                "FOCUS",
                                "Actions closed via escape, focus restored via coordinator",
                            );
                        });
                    });
                }));
            });

            // Get main window bounds and display_id for positioning the actions popup
            //
            // CRITICAL: We use GPUI's window.bounds() which returns SCREEN-RELATIVE coordinates
            // (top-left origin, relative to the window's current screen). We also capture the
            // display_id so the actions window is created on the SAME screen as the main window.
            //
            // This fixes multi-monitor issues where the actions popup would appear on the wrong
            // screen or at wrong coordinates when the main window was on a secondary display.
            let main_bounds = window.bounds();
            let display_id = window.display(cx).map(|d| d.id());

            logging::log(
                "ACTIONS",
                &format!(
                    "Main window bounds (GPUI screen-relative): origin=({:?}, {:?}), size={:?}x{:?}, display_id={:?}",
                    main_bounds.origin.x, main_bounds.origin.y,
                    main_bounds.size.width, main_bounds.size.height,
                    display_id
                ),
            );

            // Open the actions window via spawn, passing the shared dialog entity and display_id
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
                            logging::log("ACTIONS", "Actions popup window opened");
                        }
                        Err(e) => {
                            logging::log(
                                "ACTIONS",
                                &format!("Failed to open actions window: {}", e),
                            );
                        }
                    }
                })
                .ok();
            })
            .detach();

            logging::log("FOCUS", "Actions opened, keyboard routing active");
        }
        cx.notify();
    }

    /// Toggle actions dialog for arg prompts with SDK-defined actions
    pub(crate) fn toggle_arg_actions(&mut self, cx: &mut Context<Self>, window: &mut Window) {
        logging::log(
            "KEY",
            &format!(
                "toggle_arg_actions called: show_actions_popup={}, actions_dialog.is_some={}, sdk_actions.is_some={}",
                self.show_actions_popup,
                self.actions_dialog.is_some(),
                self.sdk_actions.is_some()
            ),
        );
        if self.show_actions_popup || is_actions_window_open() {
            self.close_actions_popup(ActionsDialogHost::ArgPrompt, window, cx);
        } else {
            // Clone SDK actions early to avoid borrow conflicts
            let sdk_actions_opt = self.sdk_actions.clone();

            // Check if we have SDK actions
            if let Some(sdk_actions) = sdk_actions_opt {
                logging::log("KEY", &format!("SDK actions count: {}", sdk_actions.len()));
                if !sdk_actions.is_empty() {
                    self.resync_filter_input_after_actions_if_needed(window, cx);
                    // Open - push overlay to save arg prompt focus state
                    self.show_actions_popup = true;
                    self.push_focus_overlay(focus_coordinator::FocusRequest::actions_dialog(), cx);

                    let theme_arc = std::sync::Arc::clone(&self.theme);
                    let dialog = cx.new(|cx| {
                        let focus_handle = cx.focus_handle();
                        let mut dialog = ActionsDialog::with_script(
                            focus_handle,
                            std::sync::Arc::new(|_action_id| {}), // Callback handled separately
                            None,                                 // No script info for arg prompts
                            theme_arc,
                        );
                        // Set SDK actions to replace built-in actions
                        dialog.set_sdk_actions(sdk_actions);
                        dialog
                    });

                    // Show search input at bottom (Raycast-style)

                    // Focus the dialog's internal focus handle
                    self.actions_dialog = Some(dialog.clone());
                    let dialog_focus_handle = dialog.read(cx).focus_handle.clone();
                    window.focus(&dialog_focus_handle, cx);
                    logging::log(
                        "FOCUS",
                        &format!(
                            "Arg actions OPENED: show_actions_popup={}, actions_dialog.is_some={}",
                            self.show_actions_popup,
                            self.actions_dialog.is_some()
                        ),
                    );
                } else {
                    logging::log("KEY", "No SDK actions available to show (empty list)");
                }
            } else {
                logging::log("KEY", "No SDK actions defined for this arg prompt (None)");
            }
        }
    }
    /// Toggle actions dialog for webcam prompt (built-in command).
    /// Opens as a separate window (same pattern as toggle_chat_actions).
    pub fn toggle_webcam_actions(&mut self, cx: &mut Context<Self>, window: &mut Window) {
        use crate::actions::{ActionsDialog, ActionsDialogConfig};

        logging::log(
            "KEY",
            &format!(
                "toggle_webcam_actions called: show_actions_popup={}, is_actions_window_open={}",
                self.show_actions_popup,
                is_actions_window_open()
            ),
        );

        if self.show_actions_popup || is_actions_window_open() {
            // Close — delegate to central close_actions_popup
            self.close_actions_popup(ActionsDialogHost::WebcamPrompt, window, cx);
        } else {
            self.resync_filter_input_after_actions_if_needed(window, cx);
            // Open actions as a separate window — same pattern as toggle_chat_actions
            self.show_actions_popup = true;
            self.push_focus_overlay(focus_coordinator::FocusRequest::actions_dialog(), cx);

            // Transfer focus to main focus_handle while actions window is open
            self.focus_handle.focus(window, cx);
            self.gpui_input_focused = false;

            let theme_arc = std::sync::Arc::clone(&self.theme);
            let webcam_actions = Self::webcam_actions_for_dialog();

            // Use native Action rows with default actions config so webcam uses the same
            // filtering/navigation behavior as the main actions dialog.
            let dialog = cx.new(move |cx| {
                let focus_handle = cx.focus_handle();
                let mut dialog = ActionsDialog::with_config(
                    focus_handle,
                    std::sync::Arc::new(|_action_id| {}),
                    webcam_actions,
                    theme_arc,
                    ActionsDialogConfig::default(),
                );
                dialog.set_context_title(Some("Webcam".to_string()));
                dialog
            });

            self.actions_dialog = Some(dialog.clone());

            // Set up on_close callback — same pattern as toggle_chat_actions
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
                            app.mark_filter_resync_after_actions_if_needed();
                            app.pop_focus_overlay(cx);
                            logging::log(
                                "FOCUS",
                                "Webcam actions closed via escape, focus restored via coordinator",
                            );
                        });
                    });
                }));
            });

            // Get main window bounds for positioning
            let main_bounds = window.bounds();
            let display_id = window.display(cx).map(|d| d.id());

            // Open the actions window — same as toggle_chat_actions
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
                            logging::log("ACTIONS", "Webcam actions popup window opened");
                        }
                        Err(e) => {
                            logging::log(
                                "ACTIONS",
                                &format!("Failed to open webcam actions window: {}", e),
                            );
                        }
                    }
                })
                .ok();
            })
            .detach();

            logging::log("FOCUS", "Webcam actions opened, keyboard routing active");
        }
        cx.notify();
    }

    /// Toggle terminal command bar for built-in terminal
    /// Shows common terminal actions (Clear, Copy, Paste, Scroll, etc.)
    #[allow(dead_code)]
    pub fn toggle_terminal_commands(&mut self, cx: &mut Context<Self>, window: &mut Window) {
        use crate::actions::ActionsDialog;

        logging::log(
            "KEY",
            &format!(
                "toggle_terminal_commands called: show_actions_popup={}, actions_dialog.is_some={}",
                self.show_actions_popup,
                self.actions_dialog.is_some()
            ),
        );

        if self.show_actions_popup || is_actions_window_open() {
            self.close_actions_popup(ActionsDialogHost::TermPrompt, window, cx);
        } else {
            self.resync_filter_input_after_actions_if_needed(window, cx);
            // Open - create actions from terminal commands
            self.show_actions_popup = true;
            self.push_focus_overlay(focus_coordinator::FocusRequest::actions_dialog(), cx);

            let theme_arc = std::sync::Arc::clone(&self.theme);
            let actions = terminal_actions_for_dialog();
            let config = terminal_actions_dialog_config();

            let dialog = cx.new(|cx| {
                let focus_handle = cx.focus_handle();
                ActionsDialog::with_config(
                    focus_handle,
                    std::sync::Arc::new(|_action_id| {}),
                    actions,
                    theme_arc,
                    config,
                )
            });
            dialog.update(cx, |d, _cx| {
                d.set_context_title(Some("Terminal".to_string()));
            });

            self.actions_dialog = Some(dialog.clone());
            let dialog_focus_handle = dialog.read(cx).focus_handle.clone();
            window.focus(&dialog_focus_handle, cx);
            logging::log("FOCUS", "Terminal commands opened");
        }
    }

    /// Toggle actions dialog for chat prompts
    /// Opens ActionsDialog with model selection and chat-specific actions
    pub fn toggle_chat_actions(&mut self, cx: &mut Context<Self>, window: &mut Window) {
        use crate::actions::{ChatModelInfo, ChatPromptInfo};

        logging::log(
            "KEY",
            &format!(
                "toggle_chat_actions called: show_actions_popup={}, actions_dialog.is_some={}",
                self.show_actions_popup,
                self.actions_dialog.is_some()
            ),
        );

        if self.show_actions_popup || is_actions_window_open() {
            self.close_actions_popup(ActionsDialogHost::ChatPrompt, window, cx);
        } else {
            // Get chat info from current ChatPrompt entity
            let chat_info = if let AppView::ChatPrompt { entity, .. } = &self.current_view {
                let chat = entity.read(cx);
                ChatPromptInfo {
                    current_model: chat.model.clone(),
                    available_models: chat
                        .models
                        .iter()
                        .map(|m| ChatModelInfo {
                            id: m.id.clone(),
                            display_name: m.name.clone(),
                            provider: m.provider.clone(),
                        })
                        .collect(),
                    has_messages: !chat.messages.is_empty(),
                    has_response: chat
                        .messages
                        .iter()
                        .any(|m| m.position == crate::protocol::ChatMessagePosition::Left),
                }
            } else {
                logging::log(
                    "KEY",
                    "toggle_chat_actions called but current view is not ChatPrompt",
                );
                return;
            };

            self.resync_filter_input_after_actions_if_needed(window, cx);
            // Open actions as a separate window with vibrancy blur
            self.show_actions_popup = true;
            // Push overlay to save chat prompt focus state
            self.push_focus_overlay(focus_coordinator::FocusRequest::actions_dialog(), cx);

            // CRITICAL: Transfer focus from ChatPrompt to main focus_handle
            // This prevents the ChatPrompt from receiving text input while
            // the actions dialog is open (same pattern as toggle_actions for ScriptList)
            self.focus_handle.focus(window, cx);
            self.gpui_input_focused = false;

            let theme_arc = std::sync::Arc::clone(&self.theme);
            let dialog = cx.new(|cx| {
                let focus_handle = cx.focus_handle();
                ActionsDialog::with_chat(
                    focus_handle,
                    std::sync::Arc::new(|_action_id| {}), // Callback handled via main app
                    &chat_info,
                    theme_arc,
                )
            });

            // Store the dialog entity for keyboard routing
            self.actions_dialog = Some(dialog.clone());

            // Set up the on_close callback to restore focus when escape is pressed in ActionsWindow
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
                            app.mark_filter_resync_after_actions_if_needed();
                            // Use coordinator to pop overlay and restore previous focus
                            app.pop_focus_overlay(cx);
                            logging::log(
                                "FOCUS",
                                "Chat actions closed via escape, focus restored via coordinator",
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
                    "Chat actions: Main window bounds origin=({:?}, {:?}), size={:?}x{:?}, display_id={:?}",
                    main_bounds.origin.x, main_bounds.origin.y,
                    main_bounds.size.width, main_bounds.size.height,
                    display_id
                ),
            );

            // Open the actions window via spawn
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
                            logging::log("ACTIONS", "Chat actions popup window opened");
                        }
                        Err(e) => {
                            logging::log(
                                "ACTIONS",
                                &format!("Failed to open chat actions window: {}", e),
                            );
                        }
                    }
                })
                .ok();
            })
            .detach();

            logging::log("FOCUS", "Chat actions opened, keyboard routing active");
        }
        cx.notify();
    }
}

#[cfg(test)]
mod on_close_reentrancy_tests {
    use std::fs;

    #[test]
    fn test_actions_toggle_on_close_defers_script_list_app_updates() {
        let source = fs::read_to_string("src/app_impl/actions_toggle.rs")
            .expect("Failed to read src/app_impl/actions_toggle.rs");

        let set_on_close_count = source
            .matches("d.set_on_close(std::sync::Arc::new(move |cx| {")
            .count();
        let defer_count = source.matches("cx.defer(move |cx| {").count();

        assert_eq!(
            set_on_close_count, 3,
            "actions_toggle should define three on_close callbacks"
        );
        assert!(
            defer_count >= 3,
            "each actions_toggle on_close callback should defer ScriptListApp updates"
        );
        assert!(
            source.contains("if !app.show_actions_popup && app.actions_dialog.is_none()"),
            "actions_toggle on_close callbacks should guard already-closed popup state"
        );
    }

    #[test]
    fn test_toggle_actions_paths_resync_filter_input_state() {
        let source = fs::read_to_string("src/app_impl/actions_toggle.rs")
            .expect("Failed to read src/app_impl/actions_toggle.rs");
        let impl_source = source
            .split("\n#[cfg(test)]")
            .next()
            .expect("Expected implementation section before tests");

        let pre_open_resync_count = impl_source
            .matches("self.resync_filter_input_after_actions_if_needed(window, cx);")
            .count();
        assert_eq!(
            pre_open_resync_count, 5,
            "all toggle_*_actions open paths should resync canonical filter input first"
        );

        let on_close_mark_count = impl_source
            .matches("app.mark_filter_resync_after_actions_if_needed();")
            .count();
        assert_eq!(
            on_close_mark_count, 3,
            "actions window on_close callbacks should mark filter resync for next render"
        );
    }
}

#[cfg(test)]
mod terminal_command_shortcut_tests {
    use super::*;
    use crate::actions::{AnchorPosition, SearchPosition, SectionStyle};
    use crate::designs::icon_variations::IconName;
    use std::fs;

    #[test]
    fn test_terminal_actions_for_dialog_shows_cmd_k_for_clear_terminal() {
        let clear_action = terminal_actions_for_dialog()
            .into_iter()
            .find(|action| action.id == TERM_PROMPT_CLEAR_ACTION_ID)
            .expect("clear action should exist in terminal actions");

        assert_eq!(
            clear_action.shortcut.as_deref(),
            Some(TERM_PROMPT_CLEAR_SHORTCUT)
        );
    }

    #[test]
    fn test_terminal_actions_for_dialog_adds_cmd_shift_k_toggle_shortcut() {
        let toggle_actions = terminal_actions_for_dialog()
            .into_iter()
            .find(|action| action.id == TERM_PROMPT_ACTIONS_TOGGLE_ACTION_ID)
            .expect("toggle actions entry should exist in terminal actions");

        assert_eq!(
            toggle_actions.shortcut.as_deref(),
            Some(TERM_PROMPT_ACTIONS_TOGGLE_SHORTCUT)
        );
    }

    #[test]
    fn test_terminal_actions_for_dialog_groups_sections_and_icons() {
        let actions = terminal_actions_for_dialog();

        let copy_action = actions
            .iter()
            .find(|action| action.id == "copy")
            .expect("copy action should exist");
        assert_eq!(copy_action.section.as_deref(), Some("Clipboard"));
        assert_eq!(copy_action.icon, Some(IconName::Copy));

        let find_action = actions
            .iter()
            .find(|action| action.id == "find")
            .expect("find action should exist");
        assert_eq!(find_action.section.as_deref(), Some("Search"));
        assert_eq!(find_action.icon, Some(IconName::MagnifyingGlass));

        let scroll_to_top_action = actions
            .iter()
            .find(|action| action.id == "scroll_to_top")
            .expect("scroll_to_top action should exist");
        assert_eq!(scroll_to_top_action.section.as_deref(), Some("Navigation"));
        assert_eq!(scroll_to_top_action.icon, Some(IconName::ArrowUp));

        let scroll_to_bottom_action = actions
            .iter()
            .find(|action| action.id == TERM_PROMPT_SCROLL_TO_BOTTOM_ACTION_ID)
            .expect("scroll_to_bottom action should exist");
        assert_eq!(
            scroll_to_bottom_action.section.as_deref(),
            Some("Navigation")
        );
        assert_eq!(scroll_to_bottom_action.icon, Some(IconName::ArrowDown));

        let clear_action = actions
            .iter()
            .find(|action| action.id == TERM_PROMPT_CLEAR_ACTION_ID)
            .expect("clear action should exist");
        assert_eq!(clear_action.section.as_deref(), Some("Session"));
        assert_eq!(clear_action.icon, Some(IconName::Trash));

        let reset_action = actions
            .iter()
            .find(|action| action.id == "reset")
            .expect("reset action should exist");
        assert_eq!(reset_action.section.as_deref(), Some("Session"));
        assert_eq!(reset_action.icon, Some(IconName::Refresh));
    }

    #[test]
    fn test_terminal_actions_dialog_config_enables_visual_features() {
        let config = terminal_actions_dialog_config();

        assert_eq!(config.search_position, SearchPosition::Top);
        assert_eq!(config.section_style, SectionStyle::Headers);
        assert_eq!(config.anchor, AnchorPosition::Top);
        assert!(config.show_icons);
        assert!(config.show_footer);
    }

    #[test]
    fn test_toggle_terminal_commands_sets_terminal_context_title() {
        let source = fs::read_to_string("src/app_impl/actions_toggle.rs")
            .expect("Failed to read src/app_impl/actions_toggle.rs");

        assert!(
            source.contains("d.set_context_title(Some(\"Terminal\".to_string()));"),
            "toggle_terminal_commands should set terminal context title"
        );
    }
}
