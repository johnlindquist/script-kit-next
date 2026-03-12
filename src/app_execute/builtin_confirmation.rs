impl ScriptListApp {
    /// Handle builtin confirmation modal result.
    /// Called when user confirms or cancels a dangerous action from the modal.
    fn handle_builtin_confirmation(
        &mut self,
        entry_id: String,
        confirmed: bool,
        query_override: Option<String>,
        cx: &mut Context<Self>,
    ) {
        if !confirmed {
            tracing::info!(
                builtin_id = %entry_id,
                "Builtin confirmation cancelled"
            );
            return;
        }

        tracing::info!(
            builtin_id = %entry_id,
            has_query = query_override.is_some(),
            "Builtin confirmation accepted, executing"
        );

        // Find the builtin entry by ID and execute through the shared inner path
        let builtin_entries = builtins::get_builtin_entries(&self.config.get_builtins());
        if let Some(entry) = builtin_entries.iter().find(|b| b.id == entry_id) {
            let entry = entry.clone();
            let trace_id = uuid::Uuid::new_v4().to_string();
            let start = std::time::Instant::now();
            self.execute_builtin_inner(
                &entry,
                query_override.as_deref(),
                &trace_id,
                start,
                cx,
            );
        } else {
            tracing::error!(
                builtin_id = %entry_id,
                "Builtin entry not found for confirmed action"
            );
            self.show_error_toast(
                format!("Builtin not found: {}", entry_id),
                cx,
            );
        }
    }
}
