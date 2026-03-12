impl ScriptListApp {
    fn trigger_sdk_action_internal(&mut self, action_name: &str) {
        if let Some(ref actions) = self.sdk_actions {
            if let Some(action) = actions.iter().find(|a| a.name == action_name) {
                let result = crate::action_helpers::trigger_sdk_action(
                    action_name,
                    action,
                    self.arg_input.text(),
                    self.response_sender.as_ref(),
                );

                let status = result.status();
                let error_code = result.error_code();
                tracing::info!(
                    action_name = %action_name,
                    status = %status,
                    error_code = error_code,
                    handler = "sdk",
                    "SDK action outcome"
                );

                // Surface send errors as Toast so the user knows the action failed
                if let Some(msg) = result.error_message(action_name) {
                    self.toast_manager.push(
                        components::toast::Toast::error(msg, &self.theme)
                            .duration_ms(Some(TOAST_ERROR_MS)),
                    );
                }
            } else {
                tracing::warn!(action = %action_name, "Unknown SDK action");
            }
        } else {
            tracing::warn!(action = %action_name, "Unknown SDK action (no actions defined)");
        }
    }

    /// Trigger an SDK action by name
    /// Returns true if the action was found and triggered
    fn trigger_action_by_name(&mut self, action_name: &str, cx: &mut Context<Self>) -> bool {
        if let Some(ref actions) = self.sdk_actions {
            if actions.iter().any(|a| a.name == action_name) {
                tracing::info!(action = %action_name, "Triggering SDK action via shortcut");
                self.trigger_sdk_action_internal(action_name);
                cx.notify();
                return true;
            }
        }
        false
    }
}
