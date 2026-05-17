// Favorites browse action handlers for handle_action dispatch.
//
// Contains: favorites_run, favorites_edit_script, favorites_copy_script_url,
// favorites_move_up, favorites_move_down, favorites_remove.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FavoritesBrowseHandlerAction {
    EditScript,
    CopyScriptUrl,
    MoveUp,
    MoveDown,
    Remove,
}

impl FavoritesBrowseHandlerAction {
    fn from_action_id(action_id: &str) -> Option<Self> {
        match action_id {
            "favorites_edit_script" => Some(Self::EditScript),
            "favorites_copy_script_url" => Some(Self::CopyScriptUrl),
            "favorites_move_up" => Some(Self::MoveUp),
            "favorites_move_down" => Some(Self::MoveDown),
            "favorites_remove" => Some(Self::Remove),
            _ => None,
        }
    }

    fn selection_required_message(self) -> &'static str {
        match self {
            Self::EditScript => "Select a favorite to edit.",
            Self::CopyScriptUrl => "Select a favorite to copy its URL.",
            Self::MoveUp | Self::MoveDown | Self::Remove => "Select a favorite.",
        }
    }

    fn copied_url_hud(self, url: &str) -> String {
        match self {
            Self::CopyScriptUrl => format!("Copied: {url}"),
            Self::EditScript | Self::MoveUp | Self::MoveDown | Self::Remove => url.to_string(),
        }
    }

    fn apply_list_mutation(
        self,
        app: &mut ScriptListApp,
        cx: &mut Context<ScriptListApp>,
    ) -> Option<Result<String, String>> {
        match self {
            Self::MoveUp => Some(app.move_selected_favorite_up(cx)),
            Self::MoveDown => Some(app.move_selected_favorite_down(cx)),
            Self::Remove => Some(app.remove_selected_favorite(cx)),
            Self::EditScript | Self::CopyScriptUrl => None,
        }
    }

    fn mutation_outcome(self, message_result: Result<String, String>) -> DispatchOutcome {
        match message_result {
            Ok(message) => {
                let mut outcome = DispatchOutcome::success();
                outcome.user_message = Some(message);
                outcome
            }
            Err(message) => {
                DispatchOutcome::error(crate::action_helpers::ERROR_ACTION_FAILED, message)
            }
        }
    }
}

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
                let Some(favorites_action) = FavoritesBrowseHandlerAction::from_action_id(action_id)
                else {
                    return DispatchOutcome::not_handled();
                };
                let Some((favorite_id, path)) = self.selected_favorite_source_path() else {
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        favorites_action.selection_required_message(),
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
                let Some(favorites_action) = FavoritesBrowseHandlerAction::from_action_id(action_id)
                else {
                    return DispatchOutcome::not_handled();
                };
                let Some(favorite_id) = self.selected_favorite_id() else {
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        favorites_action.selection_required_message(),
                    );
                };
                let deeplink_name = crate::actions::to_deeplink_name(&favorite_id);
                let deeplink_url = format!("scriptkit://run/{}", deeplink_name);
                self.copy_to_clipboard_with_feedback(
                    &deeplink_url,
                    favorites_action.copied_url_hud(&deeplink_url),
                    true,
                    cx,
                );
                DispatchOutcome::success()
            }
            "favorites_move_up" | "favorites_move_down" | "favorites_remove" => {
                let Some(favorites_action) = FavoritesBrowseHandlerAction::from_action_id(action_id)
                else {
                    return DispatchOutcome::not_handled();
                };
                let Some(message_result) = favorites_action.apply_list_mutation(self, cx) else {
                    return DispatchOutcome::not_handled();
                };
                favorites_action.mutation_outcome(message_result)
            }
            _ => DispatchOutcome::not_handled(),
        }
    }
}
