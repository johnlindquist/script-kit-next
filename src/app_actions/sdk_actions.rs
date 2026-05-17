#[derive(Debug, Clone, Copy)]
enum SdkActionLookupPlan<'a> {
    NoActionsDefined,
    ActionFound(&'a crate::protocol::ProtocolAction),
    ActionMissing,
}

impl<'a> SdkActionLookupPlan<'a> {
    fn from_actions(
        actions: Option<&'a [crate::protocol::ProtocolAction]>,
        action_name: &str,
    ) -> Self {
        match actions {
            Some(actions) => actions
                .iter()
                .find(|action| action.name == action_name)
                .map(Self::ActionFound)
                .unwrap_or(Self::ActionMissing),
            None => Self::NoActionsDefined,
        }
    }

    fn is_found(self) -> bool {
        matches!(self, Self::ActionFound(_))
    }
}

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
        match SdkActionLookupPlan::from_actions(self.sdk_actions.as_deref(), action_name) {
            SdkActionLookupPlan::ActionFound(action) => {
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
            }
            SdkActionLookupPlan::ActionMissing => {
                tracing::warn!(action = %action_name, trace_id = %trace_id, "Unknown SDK action");
                crate::action_helpers::DispatchOutcome::not_handled()
            }
            SdkActionLookupPlan::NoActionsDefined => {
                tracing::warn!(action = %action_name, trace_id = %trace_id, "Unknown SDK action (no actions defined)");
                crate::action_helpers::DispatchOutcome::not_handled()
            }
        }
    }

    /// Trigger an SDK action by name
    /// Returns true if the action was found and triggered
    fn trigger_action_by_name(&mut self, action_name: &str, cx: &mut Context<Self>) -> bool {
        if SdkActionLookupPlan::from_actions(self.sdk_actions.as_deref(), action_name).is_found() {
            tracing::info!(action = %action_name, "Triggering SDK action via shortcut");
            let outcome = self.trigger_sdk_action_internal(action_name);
            self.show_outcome_feedback(&outcome, cx);
            cx.notify();
            return true;
        }
        false
    }
}
