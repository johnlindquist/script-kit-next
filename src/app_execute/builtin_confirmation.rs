impl ScriptListApp {
    /// Handle builtin confirmation modal result.
    /// Called when user confirms or cancels a dangerous action from the modal.
    ///
    /// The `dctx` is propagated from the originating `execute_builtin_with_query`
    /// call so the entire flow (start → confirmation → dispatch → outcome) can be
    /// correlated in logs.
    fn handle_builtin_confirmation(
        &mut self,
        entry_id: String,
        confirmed: bool,
        query_override: Option<String>,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) {
        let start = std::time::Instant::now();

        if !confirmed {
            let outcome = crate::action_helpers::DispatchOutcome::cancelled()
                .with_trace_id(dctx.trace_id.clone())
                .with_detail("builtin_confirmation_cancelled");
            Self::log_builtin_outcome(&entry_id, dctx, "confirmation_gate", &outcome, &start);
            return;
        }

        tracing::info!(
            builtin_id = %entry_id,
            trace_id = %dctx.trace_id,
            has_query = query_override.is_some(),
            "Builtin confirmation accepted, executing"
        );

        // Find the builtin entry by ID and execute through the shared inner path
        let builtin_entries = builtins::get_builtin_entries(&self.config.get_builtins());
        if let Some(entry) = builtin_entries.iter().find(|b| b.id == entry_id) {
            let entry = entry.clone();

            let outcome = self.execute_builtin_inner(
                &entry,
                query_override.as_deref(),
                dctx,
                cx,
            );

            Self::log_builtin_outcome(
                &entry.id,
                dctx,
                "confirmed_builtin_execution",
                &outcome,
                &start,
            );
        } else {
            tracing::error!(
                builtin_id = %entry_id,
                trace_id = %dctx.trace_id,
                "Builtin entry not found for confirmed action"
            );
            self.show_error_toast(format!("Builtin not found: {}", entry_id), cx);

            let outcome = Self::builtin_error(
                dctx,
                crate::action_helpers::ERROR_ACTION_FAILED,
                format!("Builtin not found: {}", entry_id),
                "confirmed_builtin_missing_entry",
            );
            Self::log_builtin_outcome(
                &entry_id,
                dctx,
                "confirmed_builtin_execution",
                &outcome,
                &start,
            );
        }
    }
}
