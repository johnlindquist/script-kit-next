use super::*;
use crate::stdin_commands::KeyModifier;
use crate::protocol::AutomationWindowTarget;
use gpui::{Context, Window, ScrollStrategy};

pub(crate) struct SimulatedKeyInput<'a> {
    pub key: &'a str,
    pub modifiers: &'a [KeyModifier],
    pub target: Option<&'a AutomationWindowTarget>,
}

impl ScriptListApp {
    pub(crate) fn dispatch_simulate_key(
        &mut self,
        window: &mut Window,
        ctx: &mut Context<Self>,
        input: SimulatedKeyInput<'_>,
    ) {
        let view = self;
        let key = input.key;
        let modifiers = input.modifiers;
        let target = input.target;

        logging::log("STDIN", &format!("Simulating key: '{}' with modifiers: {:?}", key, modifiers));

        // Parse modifiers
        let has_cmd = modifiers.contains(&KeyModifier::Cmd);
        let has_shift = modifiers.contains(&KeyModifier::Shift);
        let _has_alt = modifiers.contains(&KeyModifier::Alt);
        let _has_ctrl = modifiers.contains(&KeyModifier::Ctrl);

        // Handle key based on current view
        let key_lower = key.to_lowercase();
        if key_lower == "escape" {
            if let Some(target_val) = target {
                if crate::windows::resolve_automation_window(Some(target_val)).is_ok_and(|info| {
                    info.id == crate::inline_agent::INLINE_AGENT_WINDOW_AUTOMATION_ID
                        && matches!(info.kind, crate::protocol::AutomationWindowKind::MiniAi)
                }) {
                    crate::inline_agent::close_inline_agent_overlay_window(ctx);
                    logging::log(
                        "STDIN",
                        "SimulateKey: Escape - close Inline Agent target",
                    );
                    return;
                }
            }
        }

        let simulate_key_target_is_notes = target.map_or_else(
            || {
                crate::windows::focused_automation_window().is_some_and(
                    |info| {
                        matches!(
                            info.kind,
                            crate::protocol::AutomationWindowKind::Notes
                        )
                    },
                )
            },
            |target_val| {
                crate::windows::resolve_automation_window(Some(target_val))
                    .is_ok_and(|info| {
                        matches!(
                            info.kind,
                            crate::protocol::AutomationWindowKind::Notes
                        )
                    })
            },
        );

        if has_cmd
            && has_shift
            && key_lower == "p"
            && simulate_key_target_is_notes
        {
            if let Some((notes_entity, notes_handle)) =
                notes::get_notes_app_entity_and_handle()
            {
                let _ = notes_handle.update(ctx, |_root, notes_window, cx| {
                    notes_entity.update(cx, |app, cx| {
                        app.toggle_preview(notes_window, cx);
                    });
                });
                logging::log(
                    "STDIN",
                    "SimulateKey: Cmd+Shift+P - toggle Notes preview",
                );
                return;
            }
        }
        if !has_cmd
            && !_has_alt
            && !_has_ctrl
            && simulate_key_target_is_notes
            && (key_lower == "escape"
                || key_lower == "esc"
                || key_lower == "tab"
                || key_lower == "`"
                || key_lower == "backtick")
        {
            match notes::handle_notes_ghost_key_for_automation(ctx, key) {
                Ok(result) => {
                    let handled = result
                        .get("handled")
                        .and_then(|value| value.as_bool())
                        .unwrap_or(false);
                    logging::log(
                        "STDIN",
                        &format!("SimulateKey: {key} - Notes ghost autocomplete {result}"),
                    );
                    if handled {
                        return;
                    }
                }
                Err(error) => {
                    logging::log(
                        "STDIN",
                        &format!(
                            "SimulateKey: {key} - Notes ghost autocomplete unavailable: {error}"
                        ),
                    );
                }
            }
        }
        // Mirror live GPUI Keystroke.key_char: Some(&str) only for
        // single-character keys (printable input like "a", "!", "A"),
        // None for named keys ("Escape", "Up", "Enter"). This lets
        // route_key_to_actions_dialog's printable_char branch fire on
        // stdin-driven simulateKey — without it, alphanumeric keystrokes
        // silently fail to reach the ActionsDialog filter.
        let key_char: Option<&str> = if key.chars().count() == 1 {
            Some(key)
        } else {
            None
        };

        // Actions-popup pre-dispatch: when the current view hosts an
        // open actions popup, route the key through the shared actions
        // dispatcher BEFORE falling through to per-view arms. This
        // mirrors the live GPUI handler at app_impl/startup.rs:1685 so
        // that stdin `simulateKey enter` against a popup-open surface
        // fires the highlighted action instead of the parent view's
        // enter arm (e.g., ACP composer submit). Same applies to all
        // actions-popup navigation/activation keys across every host.
        let mut actions_popup_consumed_key = false;
        if view.show_actions_popup {
            if let Some(host) = view.current_actions_host() {
                let gpui_modifiers = gpui::Modifiers {
                    platform: has_cmd,
                    shift: has_shift,
                    control: _has_ctrl,
                    alt: _has_alt,
                    function: false,
                };
                match view.route_key_to_actions_dialog(
                    &key_lower,
                    key_char,
                    &gpui_modifiers,
                    host,
                    window,
                    ctx,
                ) {
                    crate::ActionsRoute::NotHandled => {}
                    crate::ActionsRoute::Handled => {
                        logging::log(
                            "STDIN",
                            &format!(
                                "SimulateKey: actions popup handled '{}' for host {:?}",
                                key_lower, host
                            ),
                        );
                        actions_popup_consumed_key = true;
                    }
                    crate::ActionsRoute::Execute {
                        action_id,
                        should_close,
                    } => {
                        logging::log(
                            "STDIN",
                            &format!(
                                "SimulateKey: actions popup execute action_id='{}' for host {:?}",
                                action_id, host
                            ),
                        );
                        view.execute_actions_route_action(
                            host,
                            action_id,
                            should_close,
                            window,
                            ctx,
                        );
                        actions_popup_consumed_key = true;
                    }
                }
            }
        }

        if !actions_popup_consumed_key {
        match &view.current_view {
            AppView::ScriptList => {
                // Main script list key handling
                if _has_alt
                    && !has_cmd
                    && !_has_ctrl
                    && !has_shift
                    && (crate::ui_foundation::is_key_left(&key_lower)
                        || crate::ui_foundation::is_key_right(&key_lower))
                {
                    // Alt+Left / Alt+Right cycle the main-menu theme exploration
                    // variation (mirrors the live keyboard + interceptor paths).
                    let forward = crate::ui_foundation::is_key_right(&key_lower);
                    logging::log(
                        "STDIN",
                        "SimulateKey: Alt+Arrow - cycle main menu theme",
                    );
                    view.cycle_main_menu_theme(forward, window, ctx);
                } else if view.try_execute_root_file_action_shortcut(
                    &key_lower, has_cmd, has_shift, _has_alt, _has_ctrl,
                    window, ctx,
                ) {
                    logging::log(
                        "STDIN",
                        "SimulateKey: root file direct action shortcut",
                    );
                } else if key_lower == "tab"
                    && has_shift
                    && !has_cmd
                    && !_has_alt
                    && !_has_ctrl
                    && view.spine_enabled
                    && !view.show_actions_popup
                    && !view.menu_syntax_capture_form_owns_input()
                {
                    tracing::info!(
                        target: "script_kit::spine",
                        event = "profile_switcher_open_shift_tab",
                        source = "simulate_key",
                        "simulateKey Shift+Tab -> Profile Search"
                    );
                    logging::log(
                        "STDIN",
                        "SimulateKey: Shift+Tab - open Profile Search",
                    );
                    view.open_profile_search(ctx);
                } else if has_cmd && key_lower == "k" {
                    logging::log(
                        "STDIN",
                        "SimulateKey: Cmd+K - dispatch actions toggle",
                    );
                    view.handle_cmd_k_actions_toggle(window, ctx);
                } else if has_cmd
                    && key_lower == "enter"
                    && !has_shift
                    && !_has_alt
                    && !_has_ctrl
                {
                    // Mirrors the live GPUI handler at
                    // src/render_script_list/mod.rs:881-890: Cmd+Enter
                    // (no shift/alt/ctrl) routes the current ScriptList
                    // selection into ACP as an explicit
                    // FocusedTarget context part rather than the plain
                    // frontmost-app context. Without this arm, automation
                    // callers of SimulateKey fell through to the plain
                    // `enter` case and executed the selected item.
                    logging::log(
                        "STDIN",
                        "SimulateKey: Cmd+Enter - route to ACP context capture",
                    );
                    view.try_route_global_cmd_enter_to_acp_context_capture(ctx);
                } else if view.handle_menu_syntax_form_key_input(
                    &key_lower,
                    key_char,
                    &gpui::Modifiers {
                        platform: has_cmd,
                        shift: has_shift,
                        control: _has_ctrl,
                        alt: _has_alt,
                        function: false,
                    },
                    window,
                    ctx,
                ) {
                    logging::log(
                        "STDIN",
                        "SimulateKey: menu-syntax form text input",
                    );
                } else if view.main_menu_fallback_state.is_active() {
                    // Handle keys in fallback mode
                    match key_lower.as_str() {
                        "tab" => {
                            if view.menu_syntax_capture_form_owns_input() {
                                if has_shift {
                                    view.focus_previous_menu_syntax_form_field(window, ctx);
                                } else {
                                    view.focus_next_menu_syntax_form_field(window, ctx);
                                }
                                logging::log(
                                    "STDIN",
                                    "SimulateKey: Tab - move menu syntax form focus",
                                );
                            } else if view.try_navigate_root_file_directory_with_tab(
                                has_shift, window, ctx,
                            ) {
                                logging::log(
                                    "STDIN",
                                    "SimulateKey: Tab - navigate root directory file row",
                                );
                            }
                        }
                        "up" | "arrowup" => {
                            if view.main_menu_fallback_state.move_up() {
                                ctx.notify();
                            }
                        }
                        "down" | "arrowdown" => {
                            if view.main_menu_fallback_state.move_down() {
                                ctx.notify();
                            }
                        }
                        "enter" => {
                            if view.try_handle_spine_enter(window, ctx) {
                                logging::log("STDIN", "SimulateKey: Enter - spine consumed (fallback)");
                                return;
                            }
                            logging::log("STDIN", "SimulateKey: Enter - execute fallback");
                            view.execute_selected_fallback(ctx);
                        }
                        "escape" => {
                            logging::log("STDIN", "SimulateKey: Escape - clear filter (exit fallback mode)");
                            view.clear_filter(window, ctx);
                        }
                        _ => {
                            logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in fallback mode", key_lower));
                        }
                    }
                } else if view.pending_menu_syntax_ai_proposal.is_some()
                    && !has_cmd
                    && !_has_alt
                    && !_has_ctrl
                    && matches!(
                        key_lower.as_str(),
                        "tab" | "enter" | "escape"
                    )
                {
                    // Run 12 Pass 13 — `ai-proposal-accept-dismiss`.
                    // Tab/Enter accept, Esc dismisses while the inline
                    // AI proposal hint card is up. Mirrors the GPUI
                    // intercept in render_script_list/mod.rs.
                    let action = if key_lower == "escape" {
                        crate::menu_syntax_ai_apply::ProposalApplyAction::Dismiss
                    } else {
                        crate::menu_syntax_ai_apply::ProposalApplyAction::Accept
                    };
                    logging::log(
                        "STDIN",
                        &format!(
                            "SimulateKey: '{}' applies pending menu_syntax_ai_proposal as {:?}",
                            key_lower, action
                        ),
                    );
                    view.try_apply_pending_menu_syntax_ai_proposal(
                        action, window, ctx,
                    );
                } else {
                    match key_lower.as_str() {
                        "tab" => {
                            if view.menu_syntax_capture_form_owns_input() {
                                if has_shift {
                                    view.focus_previous_menu_syntax_form_field(window, ctx);
                                } else {
                                    view.focus_next_menu_syntax_form_field(window, ctx);
                                }
                                logging::log(
                                    "STDIN",
                                    "SimulateKey: Tab - move menu syntax form focus",
                                );
                            } else if view.try_navigate_root_file_directory_with_tab(
                                has_shift, window, ctx,
                            ) {
                                logging::log(
                                    "STDIN",
                                    "SimulateKey: Tab - navigate root directory file row",
                                );
                            }
                        }
                        "up" | "arrowup" => {
                            // Use move_selection_up to properly skip section headers
                            view.move_selection_up(ctx);
                        }
                        "down" | "arrowdown" => {
                            // Use move_selection_down to properly skip section headers
                            view.move_selection_down(ctx);
                        }
                        "enter" => {
                            if crate::menu_syntax_object_selector_popup_window::is_menu_syntax_object_selector_popup_window_open() {
                                if view.apply_menu_syntax_object_selector_intent(
                                    crate::menu_syntax::InlinePickerKeyIntent::Accept,
                                    window,
                                    ctx,
                                ) {
                                    logging::log("STDIN", "SimulateKey: Enter - accept menu-syntax object selector");
                                    return;
                                }
                            }
                            if crate::menu_syntax_trigger_popup_window::is_menu_syntax_trigger_popup_window_open() {
                                if view.apply_menu_syntax_trigger_popup_intent(
                                    crate::menu_syntax::InlinePickerKeyIntent::Accept,
                                    window,
                                    ctx,
                                ) {
                                    logging::log("STDIN", "SimulateKey: Enter - accept menu-syntax popup");
                                    return;
                                }
                            }
                            if view.try_handle_spine_enter(window, ctx) {
                                logging::log("STDIN", "SimulateKey: Enter - spine consumed");
                                return;
                            }
                            logging::log("STDIN", "SimulateKey: Enter - execute selected");
                            view.execute_selected(ctx);
                        }
                        "escape" => {
                            logging::log("STDIN", "SimulateKey: Escape - close menu-syntax popup, clear filter, go back, or hide");
                            if crate::menu_syntax_object_selector_popup_window::is_menu_syntax_object_selector_popup_window_open() {
                                if view.apply_menu_syntax_object_selector_intent(
                                    crate::menu_syntax::InlinePickerKeyIntent::Close,
                                    window,
                                    ctx,
                                ) {
                                    return;
                                }
                            }
                            if crate::menu_syntax_trigger_popup_window::is_menu_syntax_trigger_popup_window_open() {
                                if view.apply_menu_syntax_trigger_popup_intent(
                                    crate::menu_syntax::InlinePickerKeyIntent::Close,
                                    window,
                                    ctx,
                                ) {
                                    return;
                                }
                            }
                            if !view.filter_text.is_empty() {
                                view.clear_filter(window, ctx);
                            } else if view.opened_from_main_menu {
                                // Mini main window or other opened-from-menu view:
                                // delegate to go_back_or_close which restores Full
                                // mode and resizes the window back to full width.
                                view.go_back_or_close(window, ctx);
                            } else {
                                // Save window position for the current display BEFORE hiding
                                if let Some((x, y, width, height)) = platform::get_main_window_bounds() {
                                    let displays = platform::get_macos_displays();
                                    let bounds = window_state::PersistedWindowBounds::new(x, y, width, height);
                                    if let Some(display) = window_state::find_display_for_bounds(&bounds, &displays) {
                                        window_state::save_main_position_for_display(display, bounds);
                                    }
                                }
                                script_kit_gpui::set_main_window_visible(false);
                                sync_main_automation_window(current_main_automation_bounds(), false, false);
                                platform::defer_hide_main_window(ctx);
                            }
                        }
                        _ => {
                            logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in ScriptList", key_lower));
                        }
                    }
                }
            }
            AppView::ScriptIssuesView { report } => {
                let report = report.clone();
                match key_lower.as_str() {
                    "enter" | "return" if !has_shift && !has_cmd => {
                        logging::log(
                            "STDIN",
                            "SimulateKey: Enter - fix script issues in Agent Chat",
                        );
                        view.fix_script_issues_in_agent(&report, ctx);
                    }
                    "escape" => {
                        logging::log(
                            "STDIN",
                            "SimulateKey: Escape - leave script issues view",
                        );
                        view.go_back_or_close(window, ctx);
                    }
                    _ => {
                        logging::log(
                            "STDIN",
                            &format!(
                                "SimulateKey: Unhandled key '{}' in ScriptIssuesView",
                                key_lower
                            ),
                        );
                    }
                }
            }
            AppView::FileSearchView { .. } => {
                // File-search key handling. Mirrors the GPUI live
                // handler arms at src/render_builtins/file_search.rs
                // (Cmd+K / Enter) and the arrow interceptor at
                // src/app_impl/startup_new_arrow.rs (Up/Down).
                logging::log(
                    "STDIN",
                    &format!(
                        "SimulateKey: Dispatching '{}' to FileSearchView",
                        key_lower
                    ),
                );

                if has_cmd && key_lower == "k" {
                    let selected = view
                        .selected_file_search_result_owned()
                        .map(|(_, f)| f);
                    logging::log(
                        "STDIN",
                        "SimulateKey: Cmd+K - toggle file search actions",
                    );
                    view.toggle_file_search_actions(
                        selected.as_ref(),
                        window,
                        ctx,
                    );
                } else {
                    match key_lower.as_str() {
                        "up" | "arrowup" | "down" | "arrowdown" => {
                            let is_up = matches!(
                                key_lower.as_str(),
                                "up" | "arrowup"
                            );
                            let filtered_len =
                                view.file_search_display_len();
                            let mut moved_selection = false;
                            let mut new_index = 0usize;
                            if let AppView::FileSearchView {
                                selected_index, ..
                            } = &mut view.current_view
                            {
                                if filtered_len == 0 {
                                    *selected_index = 0;
                                    new_index = 0;
                                } else {
                                    if *selected_index >= filtered_len {
                                        *selected_index = filtered_len - 1;
                                    }
                                    if is_up && *selected_index > 0 {
                                        *selected_index -= 1;
                                        moved_selection = true;
                                    } else if !is_up
                                        && *selected_index + 1
                                            < filtered_len
                                    {
                                        *selected_index += 1;
                                        moved_selection = true;
                                    }
                                    new_index = *selected_index;
                                }
                            }
                            if moved_selection {
                                view.lock_file_search_selection_to_user_choice();
                            }
                            if filtered_len > 0 {
                                view.file_search_scroll_handle
                                    .scroll_to_item(
                                        new_index,
                                        ScrollStrategy::Nearest,
                                    );
                            }
                            ctx.notify();
                        }
                        "enter" => {
                            if let Some((_, file)) =
                                view.selected_file_search_result_owned()
                            {
                                if view.cwd_pick_mode {
                                    if file.file_type
                                        == crate::file_search::FileType::Directory
                                    {
                                        let label = crate::file_search::shorten_path(
                                            &file.path,
                                        )
                                        .trim_end_matches('/')
                                        .to_string();
                                        logging::log(
                                            "STDIN",
                                            &format!(
                                                "SimulateKey: Enter (cwd-pick) - set cwd to {}",
                                                &file.path
                                            ),
                                        );
                                        view.spine_cwd = Some(
                                            std::path::PathBuf::from(&file.path),
                                        );
                                        view.spine_cwd_label = Some(label);
                                        view.spine_cwd_revision =
                                            view.spine_cwd_revision.wrapping_add(1);
                                        view.cwd_pick_mode = false;
                                        view.invalidate_grouped_cache();
                                        view.prewarm_acp_for_spine_cwd(ctx);
                                        view.persist_spine_cwd();
                                        view.reset_to_script_list(ctx);
                                        view.clear_filter(window, ctx);
                                        view.record_return_to_script_list_submit(
                                            "cwd_pick",
                                            "simulate_key_enter",
                                            Some(&file.path),
                                        );
                                    } else {
                                        logging::log(
                                            "STDIN",
                                            "SimulateKey: Enter (cwd-pick) - selection is a file, ignoring",
                                        );
                                    }
                                    return;
                                }
                                logging::log(
                                    "STDIN",
                                    &format!(
                                        "SimulateKey: Enter - open file {}",
                                        &file.path
                                    ),
                                );
                                let _ = crate::file_search::open_file(
                                    &file.path,
                                );
                                view.close_and_reset_window(ctx);
                            } else {
                                logging::log(
                                    "STDIN",
                                    "SimulateKey: Enter in FileSearchView - no selection",
                                );
                            }
                        }
                        "tab" if !has_shift => {
                            if view.navigate_file_search_into_selected_directory(ctx) {
                                logging::log(
                                    "STDIN",
                                    "SimulateKey: Tab - navigate into selected directory",
                                );
                            } else {
                                logging::log(
                                    "STDIN",
                                    "SimulateKey: Tab in FileSearchView - no selected directory",
                                );
                            }
                        }
                        "escape" => {
                            logging::log(
                                "STDIN",
                                "SimulateKey: Escape - close FileSearchView",
                            );
                            view.close_and_reset_window(ctx);
                        }
                        _ => {
                            logging::log(
                                "STDIN",
                                &format!(
                                    "SimulateKey: Unhandled key '{}' in FileSearchView",
                                    key_lower
                                ),
                            );
                        }
                    }
                }
            }
            AppView::ProfileSearchView { .. } => {
                match key_lower.as_str() {
                    "up" | "arrowup" => {
                        logging::log("STDIN", "SimulateKey: Up - Profile Search selection");
                        view.move_profile_search_selection(true, ctx);
                    }
                    "down" | "arrowdown" => {
                        logging::log("STDIN", "SimulateKey: Down - Profile Search selection");
                        view.move_profile_search_selection(false, ctx);
                    }
                    "enter" => {
                        logging::log("STDIN", "SimulateKey: Enter - select Profile Search row");
                        view.select_profile_search_result(ctx);
                    }
                    "escape" => {
                        logging::log("STDIN", "SimulateKey: Escape - close Profile Search");
                        view.go_back_or_close(window, ctx);
                    }
                    _ => {
                        logging::log(
                            "STDIN",
                            &format!("SimulateKey: Unhandled key '{}' in ProfileSearchView", key_lower),
                        );
                    }
                }
            }
            AppView::PathPrompt { entity, .. } => {
                // Path prompt key handling
                logging::log("STDIN", &format!("SimulateKey: Dispatching '{}' to PathPrompt", key_lower));
                let entity_clone = entity.clone();
                entity_clone.update(ctx, |path_prompt: &mut PathPrompt, path_cx| {
                    if has_cmd && key_lower == "k" {
                        path_prompt.toggle_actions(path_cx);
                    } else {
                        match key_lower.as_str() {
                            "up" | "arrowup" => path_prompt.move_up(path_cx),
                            "down" | "arrowdown" => path_prompt.move_down(path_cx),
                            "enter" => path_prompt.handle_enter(path_cx),
                            "escape" => path_prompt.submit_cancel(),
                            "left" | "arrowleft" => path_prompt.navigate_to_parent(path_cx),
                            "right" | "arrowright" => path_prompt.navigate_into_selected(path_cx),
                            _ => {
                                logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in PathPrompt", key_lower));
                            }
                        }
                    }
                });
            }
            AppView::ArgPrompt { id, .. }
            | AppView::MiniPrompt { id, .. } => {
                // Arg prompt key handling via SimulateKey
                logging::log("STDIN", &format!("SimulateKey: Dispatching '{}' to ArgPrompt (actions_popup={})", key_lower, view.show_actions_popup));

                // Check for Cmd+K to toggle actions popup
                if has_cmd && key_lower == "k" {
                    logging::log("STDIN", "SimulateKey: Cmd+K - toggle arg actions");
                    view.toggle_arg_actions(ctx, window);
                } else if view.show_actions_popup {
                    // If actions popup is open, route to it
                    if let Some(ref dialog) = view.actions_dialog {
                        match key_lower.as_str() {
                            "up" | "arrowup" => {
                                logging::log("STDIN", "SimulateKey: Up in actions dialog");
                                dialog.update(ctx, |d, cx| d.move_up(cx));
                            }
                            "down" | "arrowdown" => {
                                logging::log("STDIN", "SimulateKey: Down in actions dialog");
                                dialog.update(ctx, |d, cx| d.move_down(cx));
                            }
                            "enter" => {
                                logging::log("STDIN", "SimulateKey: Enter in actions dialog");
                                let action_id = dialog.read(ctx).get_selected_action_id();
                                let should_close = dialog.read(ctx).selected_action_should_close();
                                if let Some(action_id) = action_id {
                                    logging::log("ACTIONS", &format!("SimulateKey: Executing action: {} (close={})", action_id, should_close));
                                    if should_close {
                                        view.mark_actions_popup_closed();
                                        view.focused_input = FocusedInput::ArgPrompt;
                                        window.focus(&view.focus_handle, ctx);
                                    }
                                    view.trigger_action_by_name(&action_id, ctx);
                                }
                            }
                            "escape" => {
                                logging::log("STDIN", "SimulateKey: Escape - close actions dialog");
                                view.mark_actions_popup_closed();
                                view.focused_input = FocusedInput::ArgPrompt;
                                window.focus(&view.focus_handle, ctx);
                            }
                            _ => {
                                logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in ArgPrompt actions dialog", key_lower));
                            }
                        }
                    }
                } else {
                    // Normal arg prompt key handling
                    let prompt_id = id.clone();
                    match key_lower.as_str() {
                        "up" | "arrowup" => {
                            if view.arg_selected_index > 0 {
                                view.arg_selected_index -= 1;
                                view.arg_list_scroll_handle.scroll_to_item(view.arg_selected_index, ScrollStrategy::Nearest);
                                logging::log("STDIN", &format!("SimulateKey: Arg up, index={}", view.arg_selected_index));
                            }
                        }
                        "down" | "arrowdown" => {
                            let filtered = view.filtered_arg_choices();
                            if view.arg_selected_index < filtered.len().saturating_sub(1) {
                                view.arg_selected_index += 1;
                                view.arg_list_scroll_handle.scroll_to_item(view.arg_selected_index, ScrollStrategy::Nearest);
                                logging::log("STDIN", &format!("SimulateKey: Arg down, index={}", view.arg_selected_index));
                            }
                        }
                        "enter" => {
                            logging::log("STDIN", "SimulateKey: Enter - submit mini prompt selection");
                            view.submit_arg_prompt_from_current_state(&prompt_id, ctx);
                        }
                        "escape" => {
                            logging::log("STDIN", "SimulateKey: Escape - cancel script");
                            view.submit_prompt_response(prompt_id, None, ctx);
                            view.cancel_script_execution(ctx);
                        }
                        _ => {
                            logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in ArgPrompt", key_lower));
                        }
                    }
                }
            }
            AppView::FormPrompt { entity, id } => {
                logging::log("STDIN", &format!("SimulateKey: Dispatching '{}' to FormPrompt", key_lower));
                let entity_clone = entity.clone();
                let prompt_id_clone = id.clone();

                if has_cmd
                    && !has_shift
                    && !_has_alt
                    && !_has_ctrl
                    && key_lower == "k"
                {
                    logging::log("STDIN", "SimulateKey: Cmd+K - toggle form actions");
                    view.dispatch_actions_toggle_for_current_view(
                        window,
                        ctx,
                        "stdin_simulate_key_form_prompt",
                    );
                } else {
                    match key_lower.as_str() {
                        "enter" | "return" if !has_shift && !has_cmd => {
                            let validation_message = entity_clone.update(ctx, |form, cx| {
                                form.submit_validation_message(cx)
                            });
                            if let Some(message) = validation_message {
                                logging::log("STDIN", &format!("SimulateKey: Enter blocked FormPrompt validation: {}", message));
                                view.show_hud(message, Some(3000), ctx);
                            } else {
                                logging::log("STDIN", "SimulateKey: Enter in FormPrompt - submitting form");
                                let values = entity_clone.update(ctx, |form, cx| {
                                    form.collect_values(cx)
                                });
                                view.submit_prompt_response(
                                    prompt_id_clone.clone(),
                                    Some(values),
                                    ctx,
                                );
                            }
                        }
                        "escape" | "esc" if !has_cmd => {
                            logging::log("STDIN", "SimulateKey: Escape - cancel FormPrompt");
                            view.submit_prompt_response(
                                prompt_id_clone.clone(),
                                None,
                                ctx,
                            );
                            view.cancel_script_execution(ctx);
                        }
                        "tab" if !has_cmd && !has_shift => {
                            logging::log("STDIN", "SimulateKey: Tab - next FormPrompt field");
                            entity_clone.update(ctx, |form, cx| {
                                form.focus_next(window, cx);
                            });
                        }
                        "tab" if !has_cmd && has_shift => {
                            logging::log("STDIN", "SimulateKey: Shift+Tab - previous FormPrompt field");
                            entity_clone.update(ctx, |form, cx| {
                                form.focus_previous(window, cx);
                            });
                        }
                        _ => {
                            logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in FormPrompt", key_lower));
                        }
                    }
                }
            }
            AppView::EditorPrompt { entity, id, .. } => {
                // Editor prompt key handling for template/snippet navigation and choice popup
                logging::log("STDIN", &format!("SimulateKey: Dispatching '{}' to EditorPrompt", key_lower));
                let entity_clone = entity.clone();
                let prompt_id_clone = id.clone();

                // Check if choice popup is visible
                let has_choice_popup = entity_clone.update(ctx, |editor: &mut EditorPrompt, _| {
                    editor.is_choice_popup_visible()
                });

                if has_choice_popup {
                    // Handle choice popup navigation
                    match key_lower.as_str() {
                        "up" | "arrowup" => {
                            logging::log("STDIN", "SimulateKey: Up in choice popup");
                            entity_clone.update(ctx, |editor, cx| {
                                editor.choice_popup_up_public(cx);
                            });
                        }
                        "down" | "arrowdown" => {
                            logging::log("STDIN", "SimulateKey: Down in choice popup");
                            entity_clone.update(ctx, |editor, cx| {
                                editor.choice_popup_down_public(cx);
                            });
                        }
                        "enter" if !has_cmd => {
                            logging::log("STDIN", "SimulateKey: Enter in choice popup - confirming");
                            entity_clone.update(ctx, |editor, cx| {
                                editor.choice_popup_confirm_public(window, cx);
                            });
                        }
                        "escape" => {
                            logging::log("STDIN", "SimulateKey: Escape in choice popup - cancelling");
                            entity_clone.update(ctx, |editor, cx| {
                                editor.choice_popup_cancel_public(cx);
                            });
                        }
                        "tab" if !has_shift => {
                            logging::log("STDIN", "SimulateKey: Tab in choice popup - confirm and next");
                            entity_clone.update(ctx, |editor, cx| {
                                editor.choice_popup_confirm_public(window, cx);
                                editor.next_tabstop_public(window, cx);
                            });
                        }
                        _ => {
                            logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in choice popup", key_lower));
                        }
                    }
                } else if key_lower == "tab" && !has_cmd {
                    // Handle Tab key for snippet navigation
                    entity_clone.update(ctx, |editor: &mut EditorPrompt, editor_cx| {
                        logging::log("STDIN", "SimulateKey: Tab in EditorPrompt - calling next_tabstop");
                        if editor.in_snippet_mode() {
                            editor.next_tabstop_public(window, editor_cx);
                        } else {
                            logging::log("STDIN", "SimulateKey: Tab - not in snippet mode");
                        }
                    });
                } else if key_lower == "enter" && has_cmd {
                    // Cmd+Enter submits - get content from editor
                    logging::log("STDIN", "SimulateKey: Cmd+Enter in EditorPrompt - submitting");
                    let content = entity_clone.update(ctx, |editor, editor_cx| {
                        editor.content(editor_cx)
                    });
                    view.submit_prompt_response(prompt_id_clone.clone(), Some(content), ctx);
                } else if key_lower == "escape" && !has_cmd {
                    logging::log("STDIN", "SimulateKey: Escape in EditorPrompt - cancelling");
                    view.submit_prompt_response(prompt_id_clone.clone(), None, ctx);
                    view.cancel_script_execution(ctx);
                } else {
                    logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in EditorPrompt", key_lower));
                }
            }
            AppView::TemplatePrompt { entity, id, .. } => {
                logging::log("STDIN", &format!("SimulateKey: Dispatching '{}' to TemplatePrompt (actions_popup={})", key_lower, view.show_actions_popup));
                let entity_clone = entity.clone();
                let prompt_id_clone = id.clone();

                if has_cmd
                    && !has_shift
                    && !_has_alt
                    && !_has_ctrl
                    && key_lower == "k"
                {
                    logging::log("STDIN", "SimulateKey: Cmd+K - toggle template actions");
                    view.dispatch_actions_toggle_for_current_view(
                        window,
                        ctx,
                        "stdin_simulate_key_template_prompt",
                    );
                } else {
                    match key_lower.as_str() {
                        "enter" | "return" if !has_shift && !has_cmd => {
                            logging::log("STDIN", "SimulateKey: Enter - submit TemplatePrompt");
                            entity_clone.update(ctx, |prompt, cx| {
                                prompt.submit(cx);
                            });
                        }
                        "escape" | "esc" if !has_cmd => {
                            logging::log("STDIN", "SimulateKey: Escape - cancel TemplatePrompt");
                            view.submit_prompt_response(
                                prompt_id_clone.clone(),
                                None,
                                ctx,
                            );
                            view.cancel_script_execution(ctx);
                        }
                        "tab" if !has_cmd && !has_shift => {
                            logging::log("STDIN", "SimulateKey: Tab - next TemplatePrompt field");
                            entity_clone.update(ctx, |prompt, cx| {
                                prompt.next_input(cx);
                            });
                        }
                        "tab" if !has_cmd && has_shift => {
                            logging::log("STDIN", "SimulateKey: Shift+Tab - previous TemplatePrompt field");
                            entity_clone.update(ctx, |prompt, cx| {
                                prompt.prev_input(cx);
                            });
                        }
                        "backspace" if !has_cmd && !_has_alt && !_has_ctrl => {
                            logging::log("STDIN", "SimulateKey: Backspace - edit TemplatePrompt field");
                            entity_clone.update(ctx, |prompt, cx| {
                                prompt.handle_backspace(cx);
                            });
                        }
                        _ if !has_cmd
                            && !has_shift
                            && !_has_alt
                            && !_has_ctrl
                            && key_lower.chars().count() == 1 =>
                        {
                            let ch = key_lower.chars().next().unwrap();
                            logging::log("STDIN", &format!("SimulateKey: Char '{}' - edit TemplatePrompt field", ch));
                            entity_clone.update(ctx, |prompt, cx| {
                                prompt.handle_char(ch, cx);
                            });
                        }
                        _ => {
                            logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in TemplatePrompt", key_lower));
                        }
                    }
                }
            }
            AppView::HotkeyPrompt { entity, id, .. } => {
                logging::log("STDIN", &format!("SimulateKey: Dispatching '{}' to HotkeyPrompt", key_lower));
                let entity_clone = entity.clone();
                let prompt_id_clone = id.clone();

                if ((key_lower == "escape" || key_lower == "esc")
                    && !has_cmd)
                    || (has_cmd && key_lower == "w")
                {
                    logging::log("STDIN", "SimulateKey: cancel HotkeyPrompt");
                    view.submit_prompt_response(prompt_id_clone, None, ctx);
                    view.cancel_script_execution(ctx);
                } else {
                    let mut modifiers = gpui::Modifiers::default();
                    modifiers.platform = has_cmd;
                    modifiers.control = _has_ctrl;
                    modifiers.alt = _has_alt;
                    modifiers.shift = has_shift;
                    let submitted = entity_clone.update(ctx, |prompt, cx| {
                        prompt.handle_key_down(&key_lower, modifiers, cx);
                        if prompt.shortcut.is_complete() {
                            Some(prompt.shortcut.to_hotkey_info_json())
                        } else {
                            None
                        }
                    });
                    if let Some(value) = submitted {
                        logging::log("STDIN", "SimulateKey: captured HotkeyPrompt shortcut");
                        view.submit_prompt_response(
                            prompt_id_clone,
                            Some(value),
                            ctx,
                        );
                    }
                }
            }
            AppView::ChatPrompt { entity, .. } => {
                // ChatPrompt key handling
                logging::log("STDIN", &format!("SimulateKey: Dispatching '{}' to ChatPrompt (actions_popup={})", key_lower, view.show_actions_popup));

                if has_cmd && key_lower == "k" {
                    logging::log("STDIN", "SimulateKey: Cmd+K - toggle chat actions");
                    view.toggle_chat_actions(ctx, window);
                } else if view.show_actions_popup {
                    // If actions popup is open, route to it
                    if let Some(ref dialog) = view.actions_dialog {
                        match key_lower.as_str() {
                            "up" | "arrowup" => {
                                logging::log("STDIN", "SimulateKey: Up in chat actions dialog");
                                dialog.update(ctx, |d, cx| d.move_up(cx));
                            }
                            "down" | "arrowdown" => {
                                logging::log("STDIN", "SimulateKey: Down in chat actions dialog");
                                dialog.update(ctx, |d, cx| d.move_down(cx));
                            }
                            "enter" => {
                                logging::log("STDIN", "SimulateKey: Enter in chat actions dialog");
                                let action_id = dialog.read(ctx).get_selected_action_id();
                                let should_close = dialog.read(ctx).selected_action_should_close();
                                if let Some(action_id) = action_id {
                                    logging::log("ACTIONS", &format!("SimulateKey: Executing chat action: {} (close={})", action_id, should_close));
                                    if should_close {
                                        view.close_actions_popup(ActionsDialogHost::ChatPrompt, window, ctx);
                                    }
                                    view.execute_chat_action(&action_id, ctx);
                                }
                            }
                            "escape" => {
                                logging::log("STDIN", "SimulateKey: Escape - close chat actions dialog");
                                view.close_actions_popup(ActionsDialogHost::ChatPrompt, window, ctx);
                            }
                            _ => {
                                // Handle printable characters for search
                                if let Some(ch) = key_lower.chars().next() {
                                    if ch.is_alphanumeric() || ch.is_whitespace() || ch == '-' || ch == '_' {
                                        logging::log("STDIN", &format!("SimulateKey: Char '{}' in chat actions dialog", ch));
                                        dialog.update(ctx, |d, cx| d.handle_char(ch, cx));
                                    } else {
                                        logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in ChatPrompt actions dialog", key_lower));
                                    }
                                }
                            }
                        }
                        // Notify the actions window to re-render
                        crate::actions::notify_actions_window(ctx);
                    }
                } else {
                    // Route setup keys (tab, arrows, enter, escape) to ChatPrompt
                    entity.update(ctx, |chat, cx| {
                        if key_lower == "escape" {
                            chat.handle_escape(cx);
                            logging::log(
                                "STDIN",
                                "SimulateKey: Escape handled by ChatPrompt",
                            );
                        } else if chat.handle_setup_key(&key_lower, has_shift, cx) {
                            logging::log("STDIN", &format!("SimulateKey: Setup handled '{}'", key_lower));
                        } else {
                            logging::log("STDIN", &format!("SimulateKey: Unhandled '{}' in ChatPrompt", key_lower));
                        }
                    });
                }
            }
            AppView::EmojiPickerView { .. } if has_cmd && key_lower == "k" => {
                // Mirrors the live GPUI handler at
                // src/render_builtins/emoji_picker.rs:140-158 which
                // routes every key through route_key_to_actions_dialog
                // (host=EmojiPicker) first; the canonical open path
                // lives in view.toggle_actions (resolves host via
                // actions_dialog_host_for_current_view). Without this
                // arm, the automation simulateKey path fell through
                // to the per-view match below and emitted
                //   SimulateKey: Unhandled key 'k' in EmojiPicker
                // so the actions-cmdk-builtin-emoji-picker story could
                // not be verified via stdin — actions dialog never
                // opened. Same tool-gap class as ClipboardHistoryView
                // (Run 7 Pass #17).
                logging::log("STDIN", "SimulateKey: Cmd+K - toggle emoji actions");
                view.toggle_actions(ctx, window);
            }
            AppView::EmojiPickerView { filter, selected_index, selected_category } => {
                let filter_clone = filter.clone();
                let cat = *selected_category;
                let old_idx = *selected_index;
                let ordered = crate::emoji::filtered_ordered_emojis(&filter_clone, cat);
                let filtered_len = ordered.len();
                if filtered_len == 0 {
                    return;
                }
                let cols = crate::emoji::GRID_COLS;
                let new_idx = match key_lower.as_str() {
                    "up" | "arrowup" => old_idx.saturating_sub(cols),
                    "down" | "arrowdown" => (old_idx + cols).min(filtered_len.saturating_sub(1)),
                    "left" | "arrowleft" => old_idx.saturating_sub(1),
                    "right" | "arrowright" => (old_idx + 1).min(filtered_len.saturating_sub(1)),
                    "enter" => {
                        if let Some(emoji) = ordered.get(old_idx) {
                            ctx.write_to_clipboard(gpui::ClipboardItem::new_string(emoji.emoji.to_string()));
                            view.close_and_reset_window(ctx);
                        }
                        return;
                    }
                    "escape" => {
                        view.close_and_reset_window(ctx);
                        return;
                    }
                    _ => {
                        logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in EmojiPicker", key_lower));
                        return;
                    }
                };
                // Apply new index
                if let AppView::EmojiPickerView { selected_index, .. } = &mut view.current_view {
                    *selected_index = new_idx;
                }
                let row = crate::emoji::compute_scroll_row(new_idx, &ordered);
                view.emoji_scroll_handle.scroll_to_item(row, ScrollStrategy::Nearest);
                view.input_mode = InputMode::Keyboard;
                view.hovered_index = None;
                ctx.notify();
            }
            AppView::ClipboardHistoryView { .. } => {
                // Mirrors the live GPUI handler at
                // src/render_builtins/clipboard.rs:183 (Cmd+K opens
                // clipboard actions). Without this arm, the automation
                // simulateKey path fell through to "unhandled_view" and
                // the actions-cmdk-builtin-clipboard-history story could
                // not be verified via stdin — actions dialog never opened.
                if has_cmd && key_lower == "k" {
                    if let Some(entry) = view.selected_clipboard_entry() {
                        logging::log(
                            "STDIN",
                            "SimulateKey: Cmd+K - toggle clipboard actions",
                        );
                        view.toggle_clipboard_actions(entry, window, ctx);
                    } else {
                        logging::log(
                            "STDIN",
                            "SimulateKey: Cmd+K ignored - no selected clipboard entry",
                        );
                    }
                } else {
                    logging::log(
                        "STDIN",
                        &format!(
                            "SimulateKey: Unhandled key '{}' in ClipboardHistoryView",
                            key_lower
                        ),
                    );
                }
            }
            AppView::WindowSwitcherView { .. } => {
                // Run 14 Pass 25 (re-fix; Pass 23 was a no-op
                // because it edited the dead
                // `runtime_stdin_match_simulate_key.rs`
                // — see `[?] tool-runtime-stdin-match-simulate-key-rs-is-dead-code`).
                // Mirrors the live GPUI handler at
                // `src/render_builtins/window_switcher.rs:80-85`
                // (clear filter first, then go_back_or_close).
                // Without this arm, simulateKey escape against
                // an empty-choices windowSwitcher dropped to the
                // generic-fallback `simulateKey_unhandled_view`
                // warn (Pass 20 finding
                // `[?] tool-windowswitcher-empty-state-not-dismissable-by-escape`).
                if has_cmd && key_lower == "k" {
                    logging::log(
                        "STDIN",
                        "SimulateKey: Cmd+K - toggle window-switcher actions",
                    );
                    view.toggle_actions(ctx, window);
                } else {
                    match key_lower.as_str() {
                        "escape" => {
                            logging::log(
                                "STDIN",
                                "SimulateKey: Escape - clear filter or close WindowSwitcher",
                            );
                            if !view.clear_builtin_view_filter(ctx) {
                                view.go_back_or_close(window, ctx);
                            }
                        }
                        _ => {
                            logging::log(
                                "STDIN",
                                &format!(
                                    "SimulateKey: Unhandled key '{}' in WindowSwitcher",
                                    key_lower
                                ),
                            );
                        }
                    }
                }
            }
            AppView::AcpChatView { ref entity, .. } => {
                logging::log("STDIN", &format!("SimulateKey: Dispatching '{}' to AcpChatView", key_lower));
                let entity_clone = entity.clone();
                if has_cmd && key_lower == "k" {
                    logging::log("STDIN", "SimulateKey: Cmd+K - open actions in Agent Chat");
                    view.toggle_actions(ctx, window);
                } else if has_cmd && key_lower == "p" {
                    logging::log("STDIN", "SimulateKey: Cmd+P - open history command from Agent Chat");
                    view.handle_action("acp_show_history".into(), window, ctx);
                } else if {
                    // Spine projection in ACP owns Up/Down for row selection
                    // and Escape to dismiss. These short-circuit before the
                    // legacy actions / cancel-streaming paths.
                    let spine_handled = entity_clone.update(ctx, |chat, cx| {
                        if !chat.acp_spine_owns_list() {
                            return false;
                        }
                        match key_lower.as_str() {
                            "up" | "arrowup" => {
                                chat.move_acp_spine_selection(-1, cx);
                                true
                            }
                            "down" | "arrowdown" => {
                                chat.move_acp_spine_selection(1, cx);
                                true
                            }
                            "escape" if !view.show_actions_popup => {
                                chat.dismiss_acp_spine_projection(cx);
                                true
                            }
                            _ => false,
                        }
                    });
                    if spine_handled {
                        logging::log("STDIN", &format!("SimulateKey: '{}' - spine handled (ACP)", key_lower));
                    }
                    spine_handled
                } {
                    // Spine handled it; no further action.
                } else if view.show_actions_popup && key_lower == "escape" {
                    logging::log("STDIN", "SimulateKey: Escape - close Agent Chat actions dialog");
                    view.close_actions_popup(ActionsDialogHost::AcpChat, window, ctx);
                } else if key_lower == "escape" {
                    let cancelled_streaming = entity_clone.update(ctx, |chat, cx| {
                        if chat.is_focused_text_mini()
                            || chat.focused_text_originated_from_quick_prompt()
                        {
                            false
                        } else {
                            chat.cancel_streaming_from_escape(cx)
                        }
                    });
                    if cancelled_streaming {
                        logging::log("STDIN", "SimulateKey: Escape - cancel Agent Chat streaming");
                    } else if {
                        let chat = entity_clone.read(ctx);
                        chat.is_focused_text_mini() || chat.focused_text_originated_from_quick_prompt()
                    } {
                        logging::log("STDIN", "SimulateKey: Escape - hide focused-text quick prompt Agent Chat");
                        view.close_acp_chat_main_window_state_first(ctx);
                    } else if view.opened_from_main_menu {
                        logging::log("STDIN", "SimulateKey: Escape - return to main menu from Agent Chat (opened from main menu)");
                        view.close_tab_ai_harness_terminal_with_window(window, ctx);
                    } else {
                        logging::log("STDIN", "SimulateKey: Escape - close Agent Chat window (opened directly)");
                        view.close_acp_chat_main_window_state_first(ctx);
                    }
                } else if has_cmd && key_lower == "w" {
                    logging::log("STDIN", "SimulateKey: Cmd+W - close window from Agent Chat");
                    view.close_tab_ai_harness_terminal_with_window(window, ctx);
                    view.close_and_reset_window(ctx);
                } else if has_cmd && key_lower == "enter" && !has_shift {
                    // Spine prompt submission takes precedence in ACP. If the
                    // composer parses a valid prompt plan (resolved context,
                    // free-text, etc.), submit it. Otherwise fall back to the
                    // focused-text mini replace action.
                    let spine_submitted = entity_clone.update(ctx, |chat, cx| {
                        chat.try_submit_acp_spine_prompt_plan(cx)
                    });
                    if spine_submitted {
                        logging::log("STDIN", "SimulateKey: Cmd+Enter - spine submitted (ACP)");
                    } else {
                        logging::log(
                            "STDIN",
                            "SimulateKey: Cmd+Enter - replace focused-text mini output",
                        );
                        entity_clone.update(ctx, |chat, cx| {
                            if chat.is_focused_text_mini() {
                                chat.perform_focused_text_mini_action(
                                    crate::ai::acp::view::FocusedTextMiniAction::Replace,
                                    cx,
                                );
                            }
                        });
                    }
                } else if key_lower == "enter" && !has_shift && {
                    // When the ACP composer's Spine projection owns the list,
                    // Enter must accept the selected sigil row, not submit the
                    // prompt. Protocol dispatch bypasses GPUI handle_key_down,
                    // so we must check here too.
                    let spine_consumed = entity_clone.update(ctx, |chat, cx| {
                        if chat.acp_spine_owns_list() {
                            chat.accept_acp_spine_projection_row(window, cx)
                        } else {
                            false
                        }
                    });
                    if spine_consumed {
                        logging::log("STDIN", "SimulateKey: Enter - spine accepted row (ACP)");
                    }
                    !spine_consumed
                } {
                    logging::log("STDIN", "SimulateKey: Enter - submit ACP input");
                    entity_clone.update(ctx, |chat, cx| {
                        if chat.has_focused_text_context() {
                            if let Err(error) = chat.submit_focused_text_from_enter(cx) {
                                tracing::warn!(
                                    target: "script_kit::focused_text",
                                    event = "focused_text_submit_failed",
                                    error = %error,
                                );
                            }
                        } else if let Some(thread) = chat.thread() {
                            let _ = thread
                                .update(cx, |thread, cx| thread.submit_input(cx));
                        }
                    });
                } else if key_lower == "backspace" {
                    let atomic = entity_clone.update(ctx, |chat, cx| {
                        chat.try_atomic_token_backspace(cx)
                    });
                    if !atomic {
                        entity_clone.update(ctx, |chat, cx| {
                            if let Some(thread) = chat.thread() {
                                thread.update(cx, |thread, cx| {
                                    thread.input.backspace();
                                    cx.notify();
                                });
                            }
                        });
                    } else {
                        logging::log("STDIN", "SimulateKey: Backspace - atomic token removed (ACP)");
                    }
                } else if key_lower.chars().count() == 1 {
                    let ch = key_lower.chars().next().unwrap_or(' ');
                    entity_clone.update(ctx, |chat, cx| {
                        if let Some(thread) = chat.thread() {
                            thread.update(cx, |thread, cx| {
                                thread.input.insert_char(ch);
                                cx.notify();
                            });
                        }
                    });
                } else {
                    logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in AcpChatView", key_lower));
                }
            }
            AppView::QuickTerminalView { .. } => {
                if has_cmd && key_lower == "w" {
                    logging::log(
                        "STDIN",
                        "SimulateKey: Cmd+W - close QuickTerminal state-first",
                    );
                    view.close_quick_terminal_main_window_state_first(ctx);
                } else {
                    logging::log(
                        "STDIN",
                        &format!(
                            "SimulateKey: Unhandled key '{}' in QuickTerminalView",
                            key_lower
                        ),
                    );
                }
            }
            AppView::NonListStatesView { .. } => {
                if crate::ui_foundation::is_key_left(&key_lower) {
                    logging::log(
                        "STDIN",
                        "SimulateKey: Left - previous non-list state",
                    );
                    view.move_non_list_showcase_selection(-1, ctx);
                } else if crate::ui_foundation::is_key_right(&key_lower) {
                    logging::log(
                        "STDIN",
                        "SimulateKey: Right - next non-list state",
                    );
                    view.move_non_list_showcase_selection(1, ctx);
                } else if crate::ui_foundation::is_key_escape(&key_lower) {
                    logging::log(
                        "STDIN",
                        "SimulateKey: Escape - return from non-list states to main menu",
                    );
                    view.go_back_or_close(window, ctx);
                } else {
                    logging::log(
                        "STDIN",
                        &format!(
                            "SimulateKey: Unhandled key '{}' in NonListStatesView",
                            key_lower
                        ),
                    );
                }
            }
            _ => {
                // Generic fallback: any view whose current_actions_host()
                // resolves (i.e. participates in the shared ActionsDialog)
                // should honor Cmd+K even if there is no per-view arm.
                // Closes the recurring tool-gap class where new views
                // silently dropped Cmd+K because the dispatcher table
                // was maintained per-view (Run 7 Pass #17 clipboard,
                // Run 8 Pass #2 emoji). Distinguishing log line makes
                // per-view-arm vs fallback traceable in audit receipts.
                if view.simulate_key_requests_generic_actions_toggle(
                    &key_lower,
                    has_cmd,
                    has_shift,
                    _has_alt,
                    _has_ctrl,
                ) {
                    let view_name = view.app_view_name();
                    logging::log(
                        "STDIN",
                        &format!(
                            "SimulateKey: Cmd+K - generic actions toggle (fallback for view={})",
                            view_name
                        ),
                    );
                    view.toggle_actions(ctx, window);
                } else {
                    // Loud-fail when a view has no simulateKey arm.
                    // Agentic-testing callers expect a structured receipt
                    // so CI / audit tools can assert "no view was silently
                    // dropped". See stories.md `tool-table-driven-simulatekey`.
                    let view_name = view.app_view_name();
                    tracing::warn!(
                        target: "script_kit::stdin",
                        event = "simulateKey_unhandled_view",
                        code = "unhandled_view",
                        view = %view_name,
                        key = %key_lower,
                        modifiers = ?modifiers,
                        "simulateKey has no dispatcher arm for the current view; keystroke dropped"
                    );
                    logging::log(
                        "STDIN",
                        &format!(
                            "SimulateKey: UNHANDLED_VIEW view={} key='{}' modifiers={:?} code=unhandled_view — no arm in dispatcher",
                            view_name, key_lower, modifiers
                        ),
                    );
                }
            }
        }
        } // end if !actions_popup_consumed_key
    }
}
