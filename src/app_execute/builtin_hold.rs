impl ScriptListApp {
    /// Runs a built-in whose wait in the line above the input ran out untouched.
    ///
    /// The `dctx` is propagated from the originating `execute_builtin_with_query`
    /// call so the entire flow (start → hold → dispatch → outcome) can be
    /// correlated in logs.
    fn run_held_builtin(
        &mut self,
        entry_id: String,
        query_override: Option<String>,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) {
        let start = std::time::Instant::now();

        tracing::info!(
            builtin_id = %entry_id,
            trace_id = %dctx.trace_id,
            has_query = query_override.is_some(),
            "Held builtin ran out its wait, executing"
        );

        // Find the builtin entry by ID and execute through the shared inner path
        let builtin_entries = builtins::get_builtin_entries(&self.config.get_builtins());
        if let Some(entry) = builtin_entries.iter().find(|b| b.id == entry_id) {
            let entry = entry.clone();

            let outcome = self.execute_builtin_inner(&entry, query_override.as_deref(), dctx, cx);

            Self::log_builtin_outcome(&entry.id, dctx, "held_builtin_execution", &outcome, &start);
        } else {
            tracing::error!(
                builtin_id = %entry_id,
                trace_id = %dctx.trace_id,
                "Builtin entry not found for held action"
            );
            self.notice_failed("That command is no longer available", cx);

            let outcome = Self::builtin_error(
                dctx,
                crate::action_helpers::ERROR_ACTION_FAILED,
                format!("Builtin not found: {}", entry_id),
                "held_builtin_missing_entry",
            );
            Self::log_builtin_outcome(&entry_id, dctx, "held_builtin_execution", &outcome, &start);
        }
    }
}
