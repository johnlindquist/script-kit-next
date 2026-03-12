impl ScriptListApp {
    fn trigger_sdk_action_internal(&mut self, action_name: &str) -> crate::action_helpers::DispatchOutcome {
        if let Some(ref actions) = self.sdk_actions {
            if let Some(action) = actions.iter().find(|a| a.name == action_name) {
                let result = crate::action_helpers::trigger_sdk_action(
                    action_name,
                    action,
                    self.arg_input.text(),
                    self.response_sender.as_ref(),
                );

                // Outcome carries user_message for errors — the dispatch
                // boundary's show_outcome_feedback() will show the toast.
                crate::action_helpers::DispatchOutcome::from_sdk(&result, action_name)
            } else {
                tracing::warn!(action = %action_name, "Unknown SDK action");
                crate::action_helpers::DispatchOutcome::not_handled()
            }
        } else {
            tracing::warn!(action = %action_name, "Unknown SDK action (no actions defined)");
            crate::action_helpers::DispatchOutcome::not_handled()
        }
    }

    /// Trigger an SDK action by name
    /// Returns true if the action was found and triggered
    fn trigger_action_by_name(&mut self, action_name: &str, cx: &mut Context<Self>) -> bool {
        if let Some(ref actions) = self.sdk_actions {
            if actions.iter().any(|a| a.name == action_name) {
                tracing::info!(action = %action_name, "Triggering SDK action via shortcut");
                let _outcome = self.trigger_sdk_action_internal(action_name);
                cx.notify();
                return true;
            }
        }
        false
    }
}
