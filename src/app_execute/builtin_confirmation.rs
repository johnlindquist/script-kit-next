impl ScriptListApp {
    /// Handle builtin confirmation modal result.
    /// Called when user confirms or cancels a dangerous action from the modal.
    fn handle_builtin_confirmation(
        &mut self,
        entry_id: String,
        confirmed: bool,
        cx: &mut Context<Self>,
    ) {
        if !confirmed {
            tracing::info!(message = %&format!("Builtin confirmation cancelled: {}", entry_id),
            );
            return;
        }

        tracing::info!(message = %&format!("Builtin confirmation accepted, executing: {}", entry_id),
        );

        // Find the builtin entry by ID and execute it
        let builtin_entries = builtins::get_builtin_entries(&self.config.get_builtins());
        if let Some(entry) = builtin_entries.iter().find(|b| b.id == entry_id) {
            // Execute the confirmed builtin action directly
            // Skip confirmation check since we're coming from the modal callback
            self.execute_builtin_confirmed(entry, cx);
        } else {
            tracing::error!(message = %&format!("Builtin entry not found for confirmed action: {}", entry_id),
            );
            self.show_error_toast(
                format!("Builtin not found: {}", entry_id),
                cx,
            );
        }
    }

    /// Execute a builtin that has already been confirmed.
    /// This skips the confirmation check and directly executes the action.
    fn execute_builtin_confirmed(
        &mut self,
        entry: &builtins::BuiltInEntry,
        cx: &mut Context<Self>,
    ) {
        tracing::info!(message = %&format!(
                "Executing confirmed built-in: {} (id: {})",
                entry.name, entry.id
            ),
        );

        match &entry.feature {
            builtins::BuiltInFeature::SystemAction(action_type) => {
                tracing::info!(message = %&format!("Executing confirmed system action: {:?}", action_type));
                self.dispatch_system_action(action_type, cx);
            }

            builtins::BuiltInFeature::UtilityCommand(builtins::UtilityCommandType::StopAllProcesses) => {
                tracing::info!(message = %"Executing confirmed stop-all-processes");
                let process_count = crate::process_manager::PROCESS_MANAGER.active_count();
                if process_count == 0 {
                    self.show_hud(
                        "No running scripts to stop.".to_string(),
                        Some(HUD_2200_MS),
                        cx,
                    );
                } else {
                    crate::process_manager::PROCESS_MANAGER.kill_all_processes();
                    self.show_hud(
                        format!("Stopped {} running script process(es).", process_count),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                    self.close_and_reset_window(cx);
                }
            }

            builtins::BuiltInFeature::FrecencyCommand(builtins::FrecencyCommandType::ClearSuggested) => {
                tracing::info!(message = %"Executing confirmed clear-suggested");
                self.frecency_store.clear();
                if let Err(e) = self.frecency_store.save() {
                    tracing::error!(message = %&format!("Failed to save frecency data: {}", e));
                    self.toast_manager.push(
                        components::toast::Toast::error(
                            format!("Failed to clear suggested: {}", e),
                            &self.theme,
                        )
                        .duration_ms(Some(TOAST_ERROR_MS)),
                    );
                } else {
                    tracing::info!(message = %"Cleared all suggested items");
                    self.invalidate_grouped_cache();
                    self.reset_to_script_list(cx);
                    resize_to_view_sync(ViewType::ScriptList, 0);
                    self.show_hud("Suggested items cleared".to_string(), Some(HUD_SHORT_MS), cx);
                }
                cx.notify();
            }

            // For any other builtin type that somehow got confirmed,
            // just execute it normally (shouldn't happen in practice)
            _ => {
                tracing::warn!(message = %&format!("Unexpected confirmed builtin type: {:?}", entry.feature));
            }
        }
    }
}
