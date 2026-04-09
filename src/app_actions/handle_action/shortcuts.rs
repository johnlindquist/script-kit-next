// Shortcut and alias action handlers for handle_action dispatch.
//
// Contains: configure_shortcut, add_shortcut, update_shortcut, remove_shortcut,
// add_alias, update_alias, remove_alias.
//
// All branches resolve command IDs through `SearchResult::launcher_command_id()`
// so that plugin-qualified IDs are consistent across read and write paths.

impl ScriptListApp {
    /// Handle shortcut and alias actions. Returns `DispatchOutcome` indicating if handled.
    fn handle_shortcut_alias_action(
        &mut self,
        action_id: &str,
        dctx: &DispatchContext,
        cx: &mut Context<Self>,
    ) -> DispatchOutcome {
        let _ = dctx; // Reserved for future async path logging
        match action_id {
            // Handle both legacy "configure_shortcut" and new dynamic actions
            // "add_shortcut" and "update_shortcut" open the shortcut recorder
            "configure_shortcut" | "add_shortcut" | "update_shortcut" => {
                tracing::info!(category = "UI", action = action_id, "action triggered");
                if let Some(result) = self.get_selected_result() {
                    // Skills and windows are non-bindable
                    match result {
                        scripts::SearchResult::Window(_) | scripts::SearchResult::Skill(_) => {
                            return DispatchOutcome::error(
                                crate::action_helpers::ERROR_ACTION_FAILED,
                                "Shortcuts not supported for this item type",
                            );
                        }
                        scripts::SearchResult::Agent(_) => {
                            return DispatchOutcome::error(
                                crate::action_helpers::ERROR_ACTION_FAILED,
                                "Shortcuts not supported for this item type",
                            );
                        }
                        _ => {}
                    }

                    let Some(command_id) = result.launcher_command_id() else {
                        self.hide_main_and_reset(cx);
                        return DispatchOutcome::error(
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            "Cannot assign shortcut for this item type",
                        );
                    };
                    let command_name = result.launcher_command_name();

                    tracing::info!(
                        command_id = %command_id,
                        command_name = %command_name,
                        item_type = result.type_label(),
                        "launcher_shortcut_recorder_requested"
                    );
                    self.show_shortcut_recorder(command_id, command_name, cx);
                } else {
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        selection_required_message_for_action(action_id),
                    );
                }
                DispatchOutcome::success()
            }
            // "remove_shortcut" removes the existing shortcut from the registry
            "remove_shortcut" => {
                tracing::info!(category = "UI", "remove shortcut action");
                if let Some(result) = self.get_selected_result() {
                    let command_id_opt = result.launcher_command_id();

                    if let Some(command_id) = command_id_opt {
                        tracing::info!(
                            command_id = %command_id,
                            item_type = result.type_label(),
                            "launcher_shortcut_remove_requested"
                        );
                        // Remove the shortcut override from persistence
                        match crate::shortcuts::remove_shortcut_override(&command_id) {
                            Ok(()) => {
                                tracing::info!(
                                    category = "SHORTCUT",
                                    command_id = %command_id,
                                    "Removed shortcut override"
                                );
                                self.show_hud(
                                    "Shortcut removed".to_string(),
                                    Some(HUD_MEDIUM_MS),
                                    cx,
                                );
                                // Refresh scripts to update shortcut display
                                self.refresh_scripts(cx);
                            }
                            Err(e) => {
                                tracing::error!(error = %e, "failed to remove shortcut");
                                self.hide_main_and_reset(cx);
                                return DispatchOutcome::error(
                                    crate::action_helpers::ERROR_ACTION_FAILED,
                                    format!("Failed to remove shortcut: {}", e),
                                );
                            }
                        }
                    } else {
                        self.hide_main_and_reset(cx);
                        return DispatchOutcome::error(
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            "Cannot remove shortcut for this item type",
                        );
                    }
                    self.hide_main_and_reset(cx);
                } else {
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        selection_required_message_for_action(action_id),
                    );
                }
                DispatchOutcome::success()
            }
            // Alias actions: add_alias, update_alias open the alias input
            "add_alias" | "update_alias" => {
                tracing::info!(category = "UI", action = action_id, "action triggered");
                if let Some(result) = self.get_selected_result() {
                    // Skills, windows, and legacy agents are non-bindable
                    match result {
                        scripts::SearchResult::Window(_)
                        | scripts::SearchResult::Skill(_)
                        | scripts::SearchResult::Agent(_) => {
                            return DispatchOutcome::error(
                                crate::action_helpers::ERROR_ACTION_FAILED,
                                "Aliases not supported for this item type",
                            );
                        }
                        _ => {}
                    }

                    let Some(command_id) = result.launcher_command_id() else {
                        self.hide_main_and_reset(cx);
                        return DispatchOutcome::error(
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            "Cannot assign alias for this item type",
                        );
                    };
                    let command_name = result.launcher_command_name();

                    tracing::info!(
                        command_id = %command_id,
                        command_name = %command_name,
                        item_type = result.type_label(),
                        "launcher_alias_input_requested"
                    );
                    self.show_alias_input(command_id, command_name, cx);
                } else {
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        selection_required_message_for_action(action_id),
                    );
                }
                DispatchOutcome::success()
            }
            // "remove_alias" removes the existing alias from persistence
            "remove_alias" => {
                tracing::info!(category = "UI", "remove alias action");
                if let Some(result) = self.get_selected_result() {
                    let command_id_opt = result.launcher_command_id();

                    if let Some(command_id) = command_id_opt {
                        tracing::info!(
                            command_id = %command_id,
                            item_type = result.type_label(),
                            "launcher_alias_remove_requested"
                        );
                        // Remove the alias override from persistence
                        match crate::aliases::remove_alias_override(&command_id) {
                            Ok(()) => {
                                tracing::info!(
                                    category = "ALIAS",
                                    command_id = %command_id,
                                    "Removed alias override"
                                );
                                self.show_hud(
                                    "Alias removed".to_string(),
                                    Some(HUD_MEDIUM_MS),
                                    cx,
                                );
                                // Refresh scripts to update alias display and registry
                                self.refresh_scripts(cx);
                            }
                            Err(e) => {
                                tracing::error!(error = %e, "failed to remove alias");
                                self.hide_main_and_reset(cx);
                                return DispatchOutcome::error(
                                    crate::action_helpers::ERROR_ACTION_FAILED,
                                    format!("Failed to remove alias: {}", e),
                                );
                            }
                        }
                    } else {
                        self.hide_main_and_reset(cx);
                        return DispatchOutcome::error(
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            "Cannot remove alias for this item type",
                        );
                    }
                    self.hide_main_and_reset(cx);
                } else {
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        selection_required_message_for_action(action_id),
                    );
                }
                DispatchOutcome::success()
            }
            _ => DispatchOutcome::not_handled(),
        }
    }
}
