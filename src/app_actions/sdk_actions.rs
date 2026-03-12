impl ScriptListApp {
    fn trigger_sdk_action_internal(&mut self, action_name: &str) -> crate::action_helpers::DispatchOutcome {
        // Generate a trace_id when called outside the normal dispatch path
        // (e.g. from trigger_action_by_name shortcuts).
        let trace_id = uuid::Uuid::new_v4().to_string();
        self.trigger_sdk_action_with_trace(action_name, &trace_id)
    }

    fn trigger_sdk_action_with_trace(
        &mut self,
        action_name: &str,
        trace_id: &str,
    ) -> crate::action_helpers::DispatchOutcome {
        if let Some(ref actions) = self.sdk_actions {
            if let Some(action) = actions.iter().find(|a| a.name == action_name) {
                let result = crate::action_helpers::trigger_sdk_action(
                    action_name,
                    action,
                    self.arg_input.text(),
                    self.response_sender.as_ref(),
                    trace_id,
                );

                // Outcome carries user_message for errors and trace_id for
                // correlation — show_outcome_feedback() will show the toast.
                crate::action_helpers::DispatchOutcome::from_sdk_with_trace(
                    &result,
                    action_name,
                    trace_id,
                )
            } else {
                tracing::warn!(action = %action_name, trace_id = %trace_id, "Unknown SDK action");
                crate::action_helpers::DispatchOutcome::not_handled()
            }
        } else {
            tracing::warn!(action = %action_name, trace_id = %trace_id, "Unknown SDK action (no actions defined)");
            crate::action_helpers::DispatchOutcome::not_handled()
        }
    }

    /// Trigger an SDK action by name
    /// Returns true if the action was found and triggered
    fn trigger_action_by_name(&mut self, action_name: &str, cx: &mut Context<Self>) -> bool {
        if let Some(ref actions) = self.sdk_actions {
            if actions.iter().any(|a| a.name == action_name) {
                tracing::info!(action = %action_name, "Triggering SDK action via shortcut");
                let outcome = self.trigger_sdk_action_internal(action_name);
                self.show_outcome_feedback(&outcome, cx);
                cx.notify();
                return true;
            }
        }
        false
    }
}
