// Favorites browse action handlers for handle_action dispatch.
//
// Contains: favorites_run, favorites_edit_script, favorites_copy_script_url,
// favorites_move_up, favorites_move_down, favorites_remove.

impl ScriptListApp {
    fn handle_favorites_action(
        &mut self,
        action_id: &str,
        dctx: &DispatchContext,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> DispatchOutcome {
        match action_id {
            "favorites_run" => self
                .run_selected_favorite(window, cx)
                .map(|_| DispatchOutcome::success())
                .unwrap_or_else(|message| {
                    DispatchOutcome::error(crate::action_helpers::ERROR_ACTION_FAILED, message)
                }),
            "favorites_edit_script" => {
                let Some((favorite_id, path)) = self.selected_favorite_source_path() else {
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        "Select a favorite to edit.",
                    );
                };

                let editor_launch_rx =
                    self.launch_editor_with_feedback_async(&path, &dctx.trace_id);
                let trace_id = dctx.trace_id.clone();
                let start = std::time::Instant::now();
                cx.spawn(async move |this, cx| {
                    let Ok(launch_result) = editor_launch_rx.recv().await else {
                        return;
                    };

                    let _ = this.update(cx, |this, cx| match launch_result {
                        Ok(()) => {
                            tracing::info!(
                                trace_id = %trace_id,
                                favorite_id = %favorite_id,
                                status = "completed",
                                duration_ms = start.elapsed().as_millis() as u64,
                                "Async action completed: favorites_edit_script"
                            );
                            this.hide_main_and_reset(cx);
                        }
                        Err(message) => {
                            tracing::error!(
                                trace_id = %trace_id,
                                favorite_id = %favorite_id,
                                status = "failed",
                                duration_ms = start.elapsed().as_millis() as u64,
                                error = %message,
                                "Async action failed: favorites_edit_script"
                            );
                            this.show_error_toast_with_code(
                                message,
                                Some(crate::action_helpers::ERROR_LAUNCH_FAILED),
                                cx,
                            );
                        }
                    });
                })
                .detach();
                DispatchOutcome::success()
            }
            "favorites_copy_script_url" => {
                let Some(favorite_id) = self.selected_favorite_id() else {
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        "Select a favorite to copy its URL.",
                    );
                };
                let deeplink_name = crate::actions::to_deeplink_name(&favorite_id);
                let deeplink_url = format!("scriptkit://run/{}", deeplink_name);
                self.copy_to_clipboard_with_feedback(
                    &deeplink_url,
                    format!("Copied: {}", deeplink_url),
                    true,
                    cx,
                );
                DispatchOutcome::success()
            }
            "favorites_move_up" => self
                .move_selected_favorite_up(cx)
                .map(|message| {
                    let mut outcome = DispatchOutcome::success();
                    outcome.user_message = Some(message);
                    outcome
                })
                .unwrap_or_else(|message| {
                    DispatchOutcome::error(crate::action_helpers::ERROR_ACTION_FAILED, message)
                }),
            "favorites_move_down" => self
                .move_selected_favorite_down(cx)
                .map(|message| {
                    let mut outcome = DispatchOutcome::success();
                    outcome.user_message = Some(message);
                    outcome
                })
                .unwrap_or_else(|message| {
                    DispatchOutcome::error(crate::action_helpers::ERROR_ACTION_FAILED, message)
                }),
            "favorites_remove" => self
                .remove_selected_favorite(cx)
                .map(|message| {
                    let mut outcome = DispatchOutcome::success();
                    outcome.user_message = Some(message);
                    outcome
                })
                .unwrap_or_else(|message| {
                    DispatchOutcome::error(crate::action_helpers::ERROR_ACTION_FAILED, message)
                }),
            _ => DispatchOutcome::not_handled(),
        }
    }
}
