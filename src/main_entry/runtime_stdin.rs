// External command listener - receives commands via stdin (event-driven, no polling)
let stdin_rx = start_stdin_listener();
let window_for_stdin = window;
let app_entity_for_stdin = app_entity.clone();

// Track if we've received any stdin commands (for timeout warning)
static STDIN_RECEIVED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

#[derive(Clone, Copy)]
enum DevtoolsSessionLifecycleAction {
    None,
    Touch {
        command_type: &'static str,
        reason: &'static str,
    },
    ExplicitClose {
        command_type: &'static str,
        reason: &'static str,
    },
}

fn devtools_keep_actions_window_open_enabled() -> bool {
    std::env::var("SCRIPT_KIT_AGENTIC_KEEP_ACTIONS_WINDOW_OPEN").ok().as_deref() == Some("1")
}

fn devtools_lifecycle_action_for_stdin(cmd: &StdinCommand) -> DevtoolsSessionLifecycleAction {
    let command_type = cmd.command_type();
    match cmd {
        StdinCommand::External(ExternalCommand::Hide { .. }) => {
            DevtoolsSessionLifecycleAction::ExplicitClose {
                command_type,
                reason: "explicit_hide",
            }
        }
        _ if devtools_keep_actions_window_open_enabled() => DevtoolsSessionLifecycleAction::Touch {
            command_type,
            reason: "stdin_devtools_activity",
        },
        _ => DevtoolsSessionLifecycleAction::None,
    }
}

fn apply_devtools_lifecycle_action(action: DevtoolsSessionLifecycleAction) {
    match action {
        DevtoolsSessionLifecycleAction::None => {}
        DevtoolsSessionLifecycleAction::Touch {
            command_type,
            reason,
        } => {
            script_kit_gpui::mark_window_shown();
            tracing::info!(
                event = "devtools_session_activity",
                keep_actions_window_open = true,
                command_type,
                reason
            );
        }
        DevtoolsSessionLifecycleAction::ExplicitClose {
            command_type,
            reason,
        } => {
            tracing::info!(
                event = "devtools_session_explicit_close",
                keep_actions_window_open = devtools_keep_actions_window_open_enabled(),
                command_type,
                reason
            );
        }
    }
}

// Spawn a timeout warning task - helps AI agents detect when they forgot to use stdin protocol
cx.spawn(async move |_cx: &mut gpui::AsyncApp| {
    Timer::after(std::time::Duration::from_secs(2)).await;
    if !STDIN_RECEIVED.load(std::sync::atomic::Ordering::SeqCst) {
        logging::log("STDIN", "");
        logging::log(
            "STDIN",
            "╔════════════════════════════════════════════════════════════════════════════╗",
        );
        logging::log(
            "STDIN",
            "║  WARNING: No stdin JSON received after 2 seconds                          ║",
        );
        logging::log(
            "STDIN",
            "║                                                                            ║",
        );
        logging::log(
            "STDIN",
            "║  If you're testing, use the stdin JSON protocol:                          ║",
        );
        logging::log(
            "STDIN",
            "║  echo '{\"type\":\"run\",\"path\":\"...\"}' | ./target/debug/script-kit-gpui     ║",
        );
        logging::log(
            "STDIN",
            "║                                                                            ║",
        );
        logging::log(
            "STDIN",
            "║  Command line args do NOT work:                                           ║",
        );
        logging::log(
            "STDIN",
            "║  ./target/debug/script-kit-gpui test.ts  # WRONG - does nothing!          ║",
        );
        logging::log(
            "STDIN",
            "╚════════════════════════════════════════════════════════════════════════════╝",
        );
        logging::log("STDIN", "");
    }
})
.detach();

cx.spawn(async move |cx: &mut gpui::AsyncApp| {
    logging::log("STDIN", "Async stdin command handler started");

    // Event-driven: recv().await yields until a command arrives
    while let Ok(StdinCommandEnvelope {
        command: cmd,
        correlation_id,
    }) = stdin_rx.recv().await
    {
        let _guard = logging::set_correlation_id(correlation_id);
        // Mark that we've received stdin (clears the timeout warning)
        STDIN_RECEIVED.store(true, std::sync::atomic::Ordering::SeqCst);
        logging::log(
            "STDIN",
            &format!("Processing external command type={}", cmd.command_type()),
        );

        let lifecycle_action = devtools_lifecycle_action_for_stdin(&cmd);
        let app_entity_inner = app_entity_for_stdin.clone();
        let _ = cx.update(|cx| {
            apply_devtools_lifecycle_action(lifecycle_action);
            // Use the Root window to get Window reference, then update the app entity
            let _ = window_for_stdin.update(cx, |_root, window, root_cx| {
                app_entity_inner.update(root_cx, |view, ctx| {
                    // Note: We have both `window` from Root and `view` from entity here
                    // ctx is Context<ScriptListApp>, window is &mut Window
                    match cmd {
                        StdinCommand::External(cmd) => match cmd {
                            ExternalCommand::Run { ref path, ref request_id } => {
                                let rid = request_id.as_deref().unwrap_or("-");
                                logging::log("STDIN", &format!("[{}] Executing script: {}", rid, path));

                                // NOTE: This is a simplified show path for script execution.
                                // We show the window, then immediately run the script.
                                // The core logic matches show_main_window_helper().

                                script_kit_gpui::set_main_window_visible(true);
                                script_kit_gpui::mark_window_shown(); // Focus grace period
                                platform::ensure_move_to_active_space();

                                // Use Window::defer via window_ops to coalesce and defer window move.
                                // This avoids RefCell borrow conflicts from synchronous macOS window operations.
                                let window_size = gpui::size(px(750.), initial_window_height());
                                let bounds = platform::calculate_eye_line_bounds_on_mouse_display(window_size);
                                window_ops::queue_move(bounds, window, ctx);

                                // Oracle-Session `window-activation-invariants-guard` PR1.
                                platform::ensure_main_panel_configured("runtime_stdin::run");

                                // Show window WITHOUT activating (floating panel behavior)
                                platform::show_main_window_without_activation();
                                window.activate_window();

                                // Send AI window to back so it doesn't come forward with main menu
                                platform::send_ai_window_to_back();

                                let focus_handle = view.focus_handle(ctx);
                                window.focus(&focus_handle, ctx);
                                sync_main_automation_window(None, true, true);

                                // Ensure render-loop focus state is set so the input autofocuses
                                view.focused_input = FocusedInput::MainFilter;
                                view.pending_focus = Some(FocusTarget::MainFilter);

                                // Send RunScript message to be handled
                                view.handle_prompt_message(PromptMessage::RunScript { path: path.clone() }, ctx);
                            }
                            ExternalCommand::Show { ref request_id } => {
                                let rid = request_id.as_deref().unwrap_or("-");
                                logging::log("STDIN", &format!("[{}] Showing window", rid));

                                // NOTE: This is a simplified show path for explicit stdin commands.
                                // Unlike the hotkey handler, we don't need NEEDS_RESET handling
                                // because this is an explicit show (not a toggle).
                                // The core logic matches show_main_window_helper().

                                script_kit_gpui::set_main_window_visible(true);
                                script_kit_gpui::mark_window_shown(); // Focus grace period
                                platform::ensure_move_to_active_space();

                                // Position window - try per-display saved position first, then fall back to eye-line
                                let window_size = gpui::size(px(750.), initial_window_height());
                                let displays = platform::get_macos_displays();
                                let bounds = if let Some((mouse_x, mouse_y)) = platform::get_global_mouse_position() {
                                    // Try to restore saved position for the mouse display
                                    if let Some((saved, display)) =
                                        window_state::get_main_position_for_mouse_display(mouse_x, mouse_y, &displays)
                                    {
                                        // Validate the saved position is still visible
                                        if window_state::is_bounds_visible(&saved, &displays) {
                                            logging::log(
                                                "STDIN",
                                                &format!(
                                                    "Restoring saved position for display {}: ({:.0}, {:.0})",
                                                    window_state::display_key(&display),
                                                    saved.x,
                                                    saved.y
                                                ),
                                            );
                                            // Use saved position but with current window height (may have changed)
                                            gpui::Bounds {
                                                origin: gpui::point(px(saved.x as f32), px(saved.y as f32)),
                                                size: window_size,
                                            }
                                        } else {
                                            logging::log("STDIN", "Saved position no longer visible, using eye-line");
                                            platform::calculate_eye_line_bounds_on_mouse_display(window_size)
                                        }
                                    } else {
                                        logging::log("STDIN", "No saved position for this display, using eye-line");
                                        platform::calculate_eye_line_bounds_on_mouse_display(window_size)
                                    }
                                } else {
                                    logging::log("STDIN", "Could not get mouse position, using eye-line");
                                    platform::calculate_eye_line_bounds_on_mouse_display(window_size)
                                };
                                window_ops::queue_move(bounds, window, ctx);

                                // Oracle-Session `window-activation-invariants-guard` PR1.
                                platform::ensure_main_panel_configured("runtime_stdin::show");

                                // Show window WITHOUT activating (floating panel behavior)
                                platform::show_main_window_without_activation();
                                window.activate_window();

                                // Send AI window to back so it doesn't come forward with main menu
                                platform::send_ai_window_to_back();

                                let focus_handle = view.focus_handle(ctx);
                                window.focus(&focus_handle, ctx);
                                sync_main_automation_window(None, true, true);

                                // Ensure render-loop focus state is set so the input autofocuses
                                view.focused_input = FocusedInput::MainFilter;
                                view.pending_focus = Some(FocusTarget::MainFilter);
                            }
                            ExternalCommand::Hide { ref request_id } => {
                                let rid = request_id.as_deref().unwrap_or("-");
                                logging::log("STDIN", &format!("[{}] Hiding main window", rid));

                                // Save window position for the current display BEFORE hiding
                                if let Some((x, y, width, height)) = platform::get_main_window_bounds() {
                                    let displays = platform::get_macos_displays();
                                    let bounds = window_state::PersistedWindowBounds::new(x, y, width, height);
                                    if let Some(display) = window_state::find_display_for_bounds(&bounds, &displays) {
                                        logging::log(
                                            "STDIN",
                                            &format!(
                                                "Saving position for display {}: ({:.0}, {:.0})",
                                                window_state::display_key(display),
                                                x,
                                                y
                                            ),
                                        );
                                        window_state::save_main_position_for_display(display, bounds);
                                    }
                                }

                                script_kit_gpui::set_main_window_visible(false);
                                sync_main_automation_window(current_main_automation_bounds(), false, false);

                                // Reset the view back to the script list and re-key the
                                // automation `semanticSurface` to `"scriptList"` so the
                                // next list snapshot reports the truth. Without this, a
                                // hide issued while in e.g. `FileSearchView` would leak
                                // the `"fileSearch"` surface tag across its next show
                                // and leave the automation introspection channel
                                // diverged from `getState.promptType` (Pass #19 side
                                // finding; covered by `tool-hide-rpc-surface-reset`).
                                view.reset_to_script_list(ctx);
                                crate::windows::update_automation_semantic_surface(
                                    "main",
                                    Some("scriptList".to_string()),
                                );
                                // Sibling teardown for the embedded AI (`kind: Ai`,
                                // `id: "ai"`) registry entry. See the matching
                                // `ensure_embedded_ai_window(false)` in
                                // `src/app_impl/tab_ai_mode/mod.rs::close_acp_chat_to_script_list`
                                // and the three-site lock-step across the Hide dispatchers
                                // (this file, runtime_stdin_match_core.rs, app_run_setup.rs,
                                // + window_visibility.rs::hide_main_window_helper).
                                // Idempotent no-op when the entry isn't present. Closes
                                // Run 9 Pass #20 `attacker-hide-path-embedded-ai-registry-stale`.
                                crate::windows::ensure_embedded_ai_window(false);
                                // Full teardown for actions-dialog
                                // (`id: "actions-dialog"`). Pass #29 fix
                                // (`cmd-k-on-unfocused-clipboard-pops-overlay-not-actions`):
                                // upgraded from bare `remove_automation_window` to full
                                // `close_actions_window`. Pass #23's bare registry op
                                // left the `ACTIONS_WINDOW` static holding a stale handle;
                                // a later `simulateKey cmd+k` on an unfocused window read
                                // `is_actions_window_open()=true` and took the CLOSE branch,
                                // popping whichever overlay was on top instead of opening
                                // the actions dialog. `close_actions_window` clears the
                                // static AND the registry; idempotent.
                                crate::actions::close_actions_window(ctx);
                                // Sibling teardown for confirm-popup
                                // (`id: "confirm-popup"`, PromptPopup kind).
                                // Pass #25 fix: close_confirm_window at
                                // src/confirm/window.rs:385 is the only
                                // production removal path; no hide dispatcher
                                // calls it (`attacker-hide-path-confirm-popup-registry-stale`).
                                // Pure registry op; idempotent.
                                crate::windows::remove_automation_window("confirm-popup");

                                // Check if Notes or AI windows are open for logging only.
                                let notes_open = notes::is_notes_window_open();
                                let ai_open = ai::is_ai_window_open();

                                // CRITICAL: Always hide only the main panel. `ctx.hide()`
                                // app-hides all windows, so a stale/false-negative Notes
                                // handle can hide Notes together with main.
                                logging::log(
                                    "STDIN",
                                    &format!(
                                        "Using defer_hide_main_window() - main-only hide, secondary_windows_open={}",
                                        notes_open || ai_open
                                    ),
                                );
                                platform::defer_hide_main_window(ctx);
                            }
                            ExternalCommand::SetFilter { ref text, ref request_id } => {
                                let rid = request_id.as_deref().unwrap_or("-");
                                logging::log("STDIN", &format!("[{}] Setting filter to: '{}'", rid, text));
                                view.set_filter_text_immediate(text.clone(), window, ctx);
                                let _ = view.get_filtered_results_cached(); // Update cache
                                ctx.notify();
                            }
                            ref cmd @ ExternalCommand::TriggerBuiltin { .. } => {
                                // Canonical dispatch lives in the shared helper — see
                                // src/app_impl/trigger_builtin_dispatch.rs. This
                                // file is only consumed by the source-audit tests
                                // in src/app_impl/tests.rs, so keep it in lock-step
                                // with app_run_setup.rs.
                                logging::log("STDIN", "Triggering built-in (see structured logs)");
                                let _ = view.dispatch_trigger_builtin(cmd, window, ctx);
                                let _ = view
                                    .rekey_main_automation_surface_after_trigger_builtin_dispatch();
                            }

                            ExternalCommand::SimulateKey { ref key, ref modifiers, ref target, .. } => {
                                logging::log("STDIN", &format!("Simulating key: '{}' with modifiers: {:?}", key, modifiers));

                                // Parse modifiers
                                let has_cmd = modifiers.contains(&KeyModifier::Cmd);
                                let has_shift = modifiers.contains(&KeyModifier::Shift);
                                let _has_alt = modifiers.contains(&KeyModifier::Alt);
                                let _has_ctrl = modifiers.contains(&KeyModifier::Ctrl);

                                // Handle key based on current view
                                let key_lower = key.to_lowercase();

                                let simulate_key_target_is_notes = target.as_ref().map_or_else(
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
                                    |target| {
                                        crate::windows::resolve_automation_window(Some(target))
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

                                match &view.current_view {
                                    AppView::ScriptList => {
                                        // Main script list key handling
                                        if has_cmd && key_lower == "k" {
                                            logging::log("STDIN", "SimulateKey: Cmd+K - toggle actions");
                                            view.toggle_actions(ctx, window);
                                        } else if view.main_menu_fallback_state.is_active() {
                                            // Handle keys in fallback mode
                                            match key_lower.as_str() {
                                                "tab" => {
                                                    let _ = view
                                                        .try_route_plain_tab_to_acp_context_capture(
                                                            ctx,
                                                        );
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
                                        } else {
                                            match key_lower.as_str() {
                                                "tab" => {
                                                    let _ = view
                                                        .try_route_plain_tab_to_acp_context_capture(
                                                            ctx,
                                                        );
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
                                                    logging::log("STDIN", "SimulateKey: Enter - execute selected");
                                                    view.execute_selected(ctx);
                                                }
                                                "escape" => {
                                                    logging::log("STDIN", "SimulateKey: Escape - clear filter or hide");
                                                    if !view.filter_text.is_empty() {
                                                        view.clear_filter(window, ctx);
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
                                    AppView::ArgPrompt { id, .. } => {
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
                                                    logging::log("STDIN", "SimulateKey: Enter - submit selection");
                                                    let filtered = view.filtered_arg_choices();
                                                    if let Some((_, choice)) = filtered.get(view.arg_selected_index) {
                                                        let value = choice.value.clone();
                                                        view.submit_prompt_response(prompt_id, Some(value), ctx);
                                                    } else if !view.arg_input.is_empty() {
                                                        let value = view.arg_input.text().to_string();
                                                        view.submit_prompt_response(prompt_id, Some(value), ctx);
                                                    }
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
                                                if chat.handle_setup_key(&key_lower, has_shift, cx) {
                                                    logging::log("STDIN", &format!("SimulateKey: Setup handled '{}'", key_lower));
                                                } else {
                                                    logging::log("STDIN", &format!("SimulateKey: Unhandled '{}' in ChatPrompt", key_lower));
                                                }
                                            });
                                        }
                                    }
                                    AppView::AcpChatView { ref entity, .. } => {
                                        logging::log("STDIN", &format!("SimulateKey: Dispatching '{}' to AcpChatView", key_lower));
                                        let entity_clone = entity.clone();
                                        if has_cmd && key_lower == "k" {
                                            logging::log("STDIN", "SimulateKey: Cmd+K - open actions in Agent Chat");
                                            view.toggle_actions(ctx, window);
                                        } else if has_cmd && key_lower == "f" {
                                            logging::log("STDIN", "SimulateKey: Cmd+F - toggle search in Agent Chat");
                                            entity_clone.update(ctx, |chat, cx| {
                                                if chat.search_state.is_some() {
                                                    chat.search_state = None;
                                                } else {
                                                    chat.search_state = Some((String::new(), 0));
                                                }
                                                cx.notify();
                                            });
                                        } else if has_cmd && key_lower == "p" {
                                            logging::log("STDIN", "SimulateKey: Cmd+P - open history command from Agent Chat");
                                            view.handle_action("acp_show_history".into(), window, ctx);
                                        } else if has_cmd && key_lower == "n" {
                                            logging::log("STDIN", "SimulateKey: Cmd+N - new conversation in Agent Chat");
                                            entity_clone.update(ctx, |chat, cx| {
                                                if let Some(t) = chat.thread() {
                                                    t.update(cx, |thread, cx| {
                                                        thread.clear_messages(cx);
                                                    });
                                                }
                                                chat.collapsed_ids.clear();
                                                cx.notify();
                                            });
                                        } else if view.show_actions_popup && key_lower == "escape" {
                                            logging::log("STDIN", "SimulateKey: Escape - close Agent Chat actions dialog");
                                            view.close_actions_popup(ActionsDialogHost::AcpChat, window, ctx);
                                        } else if key_lower == "escape" {
                                            let cancelled_streaming = entity_clone.update(ctx, |chat, cx| {
                                                chat.cancel_streaming_from_escape(cx)
                                            });
                                            if cancelled_streaming {
                                                logging::log("STDIN", "SimulateKey: Escape - cancel Agent Chat streaming");
                                            } else {
                                                logging::log("STDIN", "SimulateKey: Escape - return to main menu from Agent Chat");
                                                view.close_tab_ai_harness_terminal_with_window(window, ctx);
                                            }
                                        } else if has_cmd && key_lower == "w" {
                                            logging::log("STDIN", "SimulateKey: Cmd+W - close window from Agent Chat");
                                            view.close_tab_ai_harness_terminal_with_window(window, ctx);
                                            view.close_and_reset_window(ctx);
                                        } else if key_lower == "enter" && !has_shift {
                                            logging::log("STDIN", "SimulateKey: Enter - submit ACP input");
                                            entity_clone.update(ctx, |chat, cx| {
                                                if let Some(t) = chat.thread() { let _ = t.update(cx, |thread, cx| thread.submit_input(cx)); }
                                            });
                                        } else if key_lower == "backspace" {
                                            entity_clone.update(ctx, |chat, cx| {
                                                if let Some(t) = chat.thread() {
                                                    t.update(cx, |thread, cx| {
                                                        thread.input.backspace();
                                                        cx.notify();
                                                    });
                                                }
                                            });
                                        } else if key_lower.chars().count() == 1 {
                                            // Single character — insert at cursor
                                            let ch = key_lower.chars().next().unwrap_or(' ');
                                            entity_clone.update(ctx, |chat, cx| {
                                                if let Some(t) = chat.thread() {
                                                    t.update(cx, |thread, cx| {
                                                        thread.input.insert_char(ch);
                                                        cx.notify();
                                                    });
                                                }
                                            });
                                        } else {
                                            logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in AcpChatView", key_lower));
                                        }
                                    }
                                    _ => {
                                        logging::log("STDIN", &format!("SimulateKey: View {:?} not supported for key simulation", std::mem::discriminant(&view.current_view)));
                                    }
                                }
                            }

                            ExternalCommand::OpenNotes => {
                                logging::log("STDIN", "Opening notes window via stdin command");
                                if let Err(e) = notes::open_notes_window(ctx) {
                                    logging::log("STDIN", &format!("Failed to open notes window: {}", e));
                                }
                            }
                            ExternalCommand::OpenAbout => {
                                logging::log("STDIN", "Opening About surface via stdin command");
                                script_kit_gpui::set_main_window_visible(true);
                                script_kit_gpui::mark_window_shown();
                                platform::show_main_window_without_activation();
                                window.activate_window();
                                sync_main_automation_window(current_main_automation_bounds(), true, true);
                                view.open_about_surface(
                                    std::sync::Arc::new(std::sync::RwLock::new(
                                        crate::updates::UpdateState::Idle,
                                    )),
                                    ctx,
                                );
                            }
                            ExternalCommand::OpenAi => {
                                logging::log("STDIN", "Opening Agent Chat via openAi compatibility alias");
                                view.open_tab_ai_acp_with_entry_intent(None, ctx);
                            }
                            ExternalCommand::OpenMiniAi => {
                                logging::log("STDIN", "Opening Agent Chat via openMiniAi compatibility alias");
                                view.open_tab_ai_acp_with_entry_intent(None, ctx);
                            }
                            ExternalCommand::OpenAiWithMockData => {
                                logging::log(
                                    "STDIN",
                                    "Ignoring deprecated mock-data AI alias and opening Agent Chat",
                                );
                                view.open_tab_ai_acp_with_entry_intent(None, ctx);
                            }
                            ExternalCommand::OpenMiniAiWithMockData => {
                                logging::log(
                                    "STDIN",
                                    "Ignoring deprecated mini mock-data AI alias and opening Agent Chat",
                                );
                                view.open_tab_ai_acp_with_entry_intent(None, ctx);
                            }
                            ExternalCommand::ShowAiCommandBar => {
                                logging::log("STDIN", "Showing AI command bar via stdin command");
                                ai::show_ai_command_bar(ctx);
                            }
                            ExternalCommand::SimulateAiKey { key, modifiers } => {
                                logging::log(
                                    "STDIN",
                                    &format!("Simulating AI key: '{}' with modifiers: {:?}", key, modifiers),
                                );
                                ai::simulate_ai_key(ctx, &key, modifiers);
                            }
                            ExternalCommand::CaptureWindow { title, path } => {
                                // Extend grace period to prevent auto-hide during capture.
                                script_kit_gpui::mark_window_shown();
                                logging::log("STDIN", &format!("Capturing window with title '{}' to '{}'", title, path));
                                match validate_capture_window_output_path(&path) {
                                    Ok(validated_path) => {
                                        match capture_window_by_title_via_resolver(&title, false) {
                                            Ok((png_data, width, height)) => {
                                                let mut can_write = true;
                                                if let Some(parent) = validated_path.parent() {
                                                    if let Err(e) = std::fs::create_dir_all(parent) {
                                                        can_write = false;
                                                        logging::log(
                                                            "STDIN",
                                                            &format!(
                                                                "Failed to create screenshot directory '{}': {}",
                                                                parent.display(),
                                                                e
                                                            ),
                                                        );
                                                    }
                                                }

                                                if can_write {
                                                    if let Err(e) = std::fs::write(&validated_path, &png_data) {
                                                        logging::log(
                                                            "STDIN",
                                                            &format!("Failed to write screenshot: {}", e),
                                                        );
                                                    } else {
                                                        logging::log(
                                                            "STDIN",
                                                            &format!(
                                                                "Screenshot saved: {} ({}x{})",
                                                                validated_path.display(),
                                                                width,
                                                                height
                                                            ),
                                                        );
                                                    }
                                                } else {
                                                    tracing::warn!(
                                                        category = "STDIN",
                                                        event_type = "stdin_capture_window_dir_create_failed",
                                                        requested_path = %path,
                                                        resolved_path = %validated_path.display(),
                                                        correlation_id = %logging::current_correlation_id(),
                                                        "Skipping screenshot write due to directory creation failure"
                                                    );
                                                }
                                            }
                                            Err(e) => {
                                                tracing::error!(
                                                    category = "STDIN",
                                                    event_type = "stdin_capture_window_failed",
                                                    requested_title = %title,
                                                    requested_path = %path,
                                                    error = %e,
                                                    correlation_id = %logging::current_correlation_id(),
                                                    "captureWindow failed before writing screenshot"
                                                );
                                                logging::log("STDIN", &format!("Failed to capture window: {}", e));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        let correlation_id = logging::current_correlation_id();
                                        tracing::warn!(
                                            category = "STDIN",
                                            event_type = "stdin_capture_window_path_rejected",
                                            requested_path = %path,
                                            reason = %e,
                                            correlation_id = %correlation_id,
                                            "Rejected captureWindow output path"
                                        );
                                        logging::log(
                                            "STDIN",
                                            &format!("Rejected captureWindow path '{}': {}", path, e),
                                        );
                                    }
                                }
                            }
                            ExternalCommand::SetAiSearch { text, ref request_id } => {
                                let request_id = request_id.as_ref().map(|id| id.as_str());
                                tracing::info!(
                                    category = "STDIN",
                                    event = "stdin_ai_command_received",
                                    command = "setAiSearch",
                                    request_id = ?request_id,
                                    text_len = text.len(),
                                    "STDIN AI command received"
                                );
                                match ai::set_ai_search(ctx, &text) {
                                    Ok(()) => {
                                        tracing::info!(
                                            category = "STDIN",
                                            event = "stdin_ai_command_finished",
                                            command = "setAiSearch",
                                            request_id = ?request_id,
                                            status = "success",
                                            "STDIN AI command finished"
                                        );
                                    }
                                    Err(error) => {
                                        logging::log("STDIN", &format!("Failed to set AI search filter: {}", error));
                                        tracing::error!(
                                            category = "STDIN",
                                            event = "stdin_ai_command_finished",
                                            command = "setAiSearch",
                                            request_id = ?request_id,
                                            status = "error",
                                            error = %error,
                                            "STDIN AI command finished"
                                        );
                                    }
                                }
                            }
                            ExternalCommand::SetAiInput { text, submit, ref request_id } => {
                                let request_id = request_id.as_ref().map(|id| id.as_str());
                                tracing::info!(
                                    category = "STDIN",
                                    event = "stdin_ai_command_received",
                                    command = "setAiInput",
                                    request_id = ?request_id,
                                    submit,
                                    text_len = text.len(),
                                    "STDIN AI command received"
                                );
                                match ai::set_ai_input(ctx, &text, submit) {
                                    Ok(()) => {
                                        tracing::info!(
                                            category = "STDIN",
                                            event = "stdin_ai_command_finished",
                                            command = "setAiInput",
                                            request_id = ?request_id,
                                            submit,
                                            status = "success",
                                            "STDIN AI command finished"
                                        );
                                    }
                                    Err(error) => {
                                        logging::log("STDIN", &format!("Failed to set AI input: {}", error));
                                        tracing::error!(
                                            category = "STDIN",
                                            event = "stdin_ai_command_finished",
                                            command = "setAiInput",
                                            request_id = ?request_id,
                                            submit,
                                            status = "error",
                                            error = %error,
                                            "STDIN AI command finished"
                                        );
                                    }
                                }
                            }
                            ExternalCommand::SetAcpInput { text, submit, ref request_id } => {
                                let request_id = request_id.as_ref().map(|id| id.as_str());
                                tracing::info!(
                                    category = "STDIN",
                                    event = "stdin_acp_command_received",
                                    command = "setAcpInput",
                                    request_id = ?request_id,
                                    submit,
                                    text_len = text.len(),
                                    "STDIN ACP command received"
                                );
                                let result = match &view.current_view {
                                    AppView::AcpChatView { entity } => {
                                        let entity = entity.clone();
                                        entity.update(ctx, |chat, cx| {
                                            chat.set_input_in_window(text.clone(), window, cx);
                                            if submit {
                                                if let Some(t) = chat.thread() {
                                                    let _ = t.update(cx, |thread, cx| thread.submit_input(cx));
                                                }
                                            }
                                        });
                                        Ok(())
                                    }
                                    _ => Err("Agent Chat view is not active".to_string()),
                                };
                                match result {
                                    Ok(()) => {
                                        tracing::info!(
                                            category = "STDIN",
                                            event = "stdin_acp_command_finished",
                                            command = "setAcpInput",
                                            request_id = ?request_id,
                                            submit,
                                            status = "success",
                                            "STDIN ACP command finished"
                                        );
                                    }
                                    Err(error) => {
                                        logging::log(
                                            "STDIN",
                                            &format!("Failed to set ACP input: {}", error),
                                        );
                                        tracing::error!(
                                            category = "STDIN",
                                            event = "stdin_acp_command_finished",
                                            command = "setAcpInput",
                                            request_id = ?request_id,
                                            submit,
                                            status = "error",
                                            error = %error,
                                            "STDIN ACP command finished"
                                        );
                                    }
                                }
                            }
                            ExternalCommand::PasteClipboardIntoAcp { ref request_id } => {
                                let request_id = request_id.as_ref().map(|id| id.as_str());
                                tracing::info!(
                                    category = "STDIN",
                                    event = "stdin_acp_command_received",
                                    command = "pasteClipboardIntoAcp",
                                    request_id = ?request_id,
                                    "STDIN ACP command received"
                                );
                                let result = match &view.current_view {
                                    AppView::AcpChatView { entity } => {
                                        let entity = entity.clone();
                                        let pasted = entity
                                            .update(ctx, |chat, cx| chat.paste_text_from_clipboard(cx));
                                        if pasted {
                                            Ok(())
                                        } else {
                                            Err("clipboard is empty or text fetch failed"
                                                .to_string())
                                        }
                                    }
                                    _ => Err("Agent Chat view is not active".to_string()),
                                };
                                match result {
                                    Ok(()) => {
                                        tracing::info!(
                                            category = "STDIN",
                                            event = "stdin_acp_command_finished",
                                            command = "pasteClipboardIntoAcp",
                                            request_id = ?request_id,
                                            status = "success",
                                            "STDIN ACP command finished"
                                        );
                                    }
                                    Err(error) => {
                                        logging::log(
                                            "STDIN",
                                            &format!("Failed to paste clipboard into ACP: {}", error),
                                        );
                                        tracing::error!(
                                            category = "STDIN",
                                            event = "stdin_acp_command_finished",
                                            command = "pasteClipboardIntoAcp",
                                            request_id = ?request_id,
                                            status = "error",
                                            error = %error,
                                            "STDIN ACP command finished"
                                        );
                                    }
                                }
                            }
                            ExternalCommand::PushDictationResult {
                                ref transcript,
                                ref target,
                                ref request_id,
                            } => {
                                let rid = request_id.as_ref().map(|id| id.as_str());
                                let target_label = target.as_deref().unwrap_or("unspecified");
                                match view.deliver_stdin_dictation_result(
                                    transcript.clone(),
                                    target.as_deref(),
                                    ctx,
                                ) {
                                    Ok(delivery_target) => {
                                        tracing::info!(
                                            category = "STDIN",
                                            event = "push_dictation_result_delivered",
                                            command = "pushDictationResult",
                                            request_id = ?rid,
                                            transcript_len = transcript.len(),
                                            requested_target = target_label,
                                            delivery_target = ?delivery_target,
                                            "pushDictationResult RPC delivered through dictation pipeline"
                                        );
                                    }
                                    Err(error) => {
                                        tracing::error!(
                                            category = "STDIN",
                                            event = "push_dictation_result_failed",
                                            command = "pushDictationResult",
                                            request_id = ?rid,
                                            transcript_len = transcript.len(),
                                            requested_target = target_label,
                                            error = %error,
                                            "pushDictationResult RPC failed"
                                        );
                                    }
                                }
                            }
                            ExternalCommand::GetAiWindowState { ref request_id } => {
                                let request_id = request_id.as_ref().map(|id| id.as_str());
                                match ai::get_ai_window_state(ctx) {
                                    Some(snapshot) => {
                                        let json = serde_json::to_string(&snapshot).unwrap_or_default();
                                        tracing::info!(
                                            category = "STDIN",
                                            event = "ai_window_state_result",
                                            command = "getAiWindowState",
                                            request_id = ?request_id,
                                            ok = true,
                                            state = %json,
                                            "AI window state snapshot"
                                        );
                                    }
                                    None => {
                                        tracing::info!(
                                            category = "STDIN",
                                            event = "ai_window_state_result",
                                            command = "getAiWindowState",
                                            request_id = ?request_id,
                                            ok = false,
                                            error_code = "ai_window_not_open",
                                            "AI window not open or entity dropped"
                                        );
                                    }
                                }
                            }
                            ExternalCommand::GetConfigFingerprint { ref request_id } => {
                                let rid = request_id.as_ref().map(|id| id.as_str());
                                match crate::config::current_config_fingerprint_receipt() {
                                    Some(receipt) => {
                                        let json = serde_json::to_string(&receipt).unwrap_or_default();
                                        tracing::info!(
                                            category = "STDIN",
                                            event = "config_fingerprint_result",
                                            command = "getConfigFingerprint",
                                            request_id = ?rid,
                                            ok = true,
                                            state = %json,
                                            "config.ts fingerprint snapshot"
                                        );
                                    }
                                    None => {
                                        tracing::info!(
                                            category = "STDIN",
                                            event = "config_fingerprint_result",
                                            command = "getConfigFingerprint",
                                            request_id = ?rid,
                                            ok = false,
                                            error_code = "config_file_missing",
                                            "config.ts not found or metadata unreadable"
                                        );
                                    }
                                }
                            }
                            ExternalCommand::ShowGrid { grid_size, show_bounds, show_box_model, show_alignment_guides, show_dimensions, ref depth } => {
                                logging::log("STDIN", &format!(
                                    "ShowGrid: size={}, bounds={}, box_model={}, guides={}, dimensions={}, depth={:?}",
                                    grid_size, show_bounds, show_box_model, show_alignment_guides, show_dimensions, depth
                                ));
                                let options = protocol::GridOptions {
                                    grid_size,
                                    show_bounds,
                                    show_box_model,
                                    show_alignment_guides,
                                    show_dimensions,
                                    depth: depth.clone(),
                                    color_scheme: None,
                                };
                                view.show_grid(options, ctx);
                            }
                            ExternalCommand::HideGrid => {
                                logging::log("STDIN", "HideGrid: hiding debug grid overlay");
                                view.hide_grid(ctx);
                            }
                            ExternalCommand::ExecuteFallback { ref fallback_id, ref input } => {
                                logging::log("STDIN", &format!("ExecuteFallback: id='{}', input='{}'", fallback_id, input));
                                execute_fallback_action(view, fallback_id, input, window, ctx);
                            }
                            ExternalCommand::ShowShortcutRecorder { ref command_id, ref command_name } => {
                                logging::log("STDIN", &format!("ShowShortcutRecorder: command_id='{}', command_name='{}'", command_id, command_name));
                                view.show_shortcut_recorder(command_id.clone(), command_name.clone(), window, ctx);
                            }
                        },
                        StdinCommand::Protocol(message) => {
                            logging::log("STDIN", "Routing stdin protocol message");
                            view.handle_stdin_protocol_message(*message, ctx);
                        }

                    }
                    view.sync_main_footer_popup(window, ctx);
                    ctx.notify();
                }); // close app_entity_inner.update
            }); // close window_for_stdin.update
        }); // close cx.update
    }

    logging::log("STDIN", "Async stdin command handler exiting");
})
.detach();
