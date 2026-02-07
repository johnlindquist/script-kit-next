impl ScriptListApp {
    fn trigger_sdk_action_internal(&mut self, action_name: &str) {
        if let Some(ref actions) = self.sdk_actions {
            if let Some(action) = actions.iter().find(|a| a.name == action_name) {
                let send_result = if action.has_action {
                    logging::log(
                        "ACTIONS",
                        &format!(
                            "SDK action with handler: '{}' (has_action=true), sending ActionTriggered",
                            action_name
                        ),
                    );
                    if let Some(ref sender) = self.response_sender {
                        let msg = protocol::Message::action_triggered(
                            action_name.to_string(),
                            action.value.clone(),
                            self.arg_input.text().to_string(),
                        );
                        Some(sender.try_send(msg))
                    } else {
                        None
                    }
                } else if let Some(ref value) = action.value {
                    logging::log(
                        "ACTIONS",
                        &format!(
                            "SDK action without handler: '{}' (has_action=false), submitting value: {:?}",
                            action_name, value
                        ),
                    );
                    if let Some(ref sender) = self.response_sender {
                        let msg = protocol::Message::Submit {
                            id: "action".to_string(),
                            value: Some(value.clone()),
                        };
                        Some(sender.try_send(msg))
                    } else {
                        None
                    }
                } else {
                    logging::log(
                        "ACTIONS",
                        &format!(
                            "SDK action '{}' has no value and has_action=false",
                            action_name
                        ),
                    );
                    None
                };

                // Log any send errors
                if let Some(result) = send_result {
                    match result {
                        Ok(()) => {}
                        Err(std::sync::mpsc::TrySendError::Full(_)) => {
                            logging::log(
                                "WARN",
                                &format!(
                                    "Response channel full - action '{}' dropped",
                                    action_name
                                ),
                            );
                        }
                        Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                            logging::log("UI", "Response channel disconnected - script exited");
                        }
                    }
                }
            } else {
                logging::log("UI", &format!("Unknown action: {}", action_name));
            }
        } else {
            logging::log("UI", &format!("Unknown action: {}", action_name));
        }
    }

    /// Trigger an SDK action by name
    /// Returns true if the action was found and triggered
    fn trigger_action_by_name(&mut self, action_name: &str, cx: &mut Context<Self>) -> bool {
        if let Some(ref actions) = self.sdk_actions {
            if actions.iter().any(|a| a.name == action_name) {
                logging::log(
                    "ACTIONS",
                    &format!("Triggering SDK action '{}' via shortcut", action_name),
                );
                self.trigger_sdk_action_internal(action_name);
                cx.notify();
                return true;
            }
        }
        false
    }
}
