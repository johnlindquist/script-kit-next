use super::*;

impl ScriptListApp {
    pub fn execute_chat_action(&mut self, action_id: &str, cx: &mut Context<Self>) {
        logging::log("ACTIONS", &format!("execute_chat_action: {}", action_id));

        let action_id = action_id.strip_prefix("chat:").unwrap_or(action_id);

        // Handle model selection (action_id starts with "select_model_")
        if let Some(model_id) = action_id.strip_prefix("select_model_") {
            let mut selected_model_name: Option<String> = None;
            if let AppView::ChatPrompt { entity, .. } = &self.current_view {
                let model_id_owned = model_id.to_string();
                entity.update(cx, |chat, cx| {
                    // Find model by ID and set it
                    if let Some(model) = chat.models.iter().find(|m| m.id == model_id_owned) {
                        chat.model = Some(model.name.clone());
                        selected_model_name = Some(model.name.clone());
                        logging::log("CHAT", &format!("Model changed to: {}", model.name));
                        cx.notify();
                    }
                });
            }
            if let Some(model_name) = selected_model_name {
                self.one_window_notice(
                    crate::line_notice::INFO,
                    format!("Model: {}", model_name),
                    cx,
                );
            }
            return;
        }

        // Handle other chat actions
        match action_id {
            "continue_in_chat" => {
                tracing::info!(
                    event = "execute_chat_action",
                    action = "continue_in_chat",
                    reuses = "handle_continue_in_chat",
                    "Handoff to harness terminal via existing continue_in_chat path"
                );
                if let AppView::ChatPrompt { entity, .. } = &self.current_view {
                    entity.update(cx, |chat, cx| {
                        chat.handle_continue_in_chat(cx);
                    });
                }
            }
            "expand_full_chat" => {
                tracing::info!(
                    event = "execute_chat_action",
                    action = "expand_full_chat",
                    reuses = "handle_expand_full_chat",
                    "Handoff to full AI window via expand_full_chat path"
                );
                if let AppView::ChatPrompt { entity, .. } = &self.current_view {
                    entity.update(cx, |chat, cx| {
                        chat.handle_expand_full_chat(cx);
                    });
                }
            }
            "send" | "stop" | "retry" | "back" | "close" => {
                let command = match action_id {
                    "send" => crate::components::conversation_actions::ChatPromptConversationCommand::Send,
                    "stop" => crate::components::conversation_actions::ChatPromptConversationCommand::Stop,
                    "retry" => crate::components::conversation_actions::ChatPromptConversationCommand::Retry,
                    "back" | "close" => crate::components::conversation_actions::ChatPromptConversationCommand::Dismiss,
                    _ => unreachable!(),
                };
                if let AppView::ChatPrompt { entity, .. } = &self.current_view {
                    entity.update(cx, |chat, cx| {
                        let _ = chat.execute_conversation_command(command, cx);
                    });
                }
            }
            "copy_response" => {
                if let AppView::ChatPrompt { entity, .. } = &self.current_view {
                    let copied = entity.update(cx, |chat, cx| {
                        chat.execute_conversation_command(
                            crate::components::conversation_actions::ChatPromptConversationCommand::CopyLastResponse,
                            cx,
                        )
                    });
                    if matches!(
                        copied,
                        crate::components::conversation_actions::ConversationCommandExecution::Executed
                    ) {
                        self.one_window_notice(crate::line_notice::DONE, "Copied the reply".to_string(), cx);
                    }
                }
            }
            "clear_conversation" => {
                let chat_entity = if let AppView::ChatPrompt { entity, .. } = &self.current_view {
                    entity.clone()
                } else {
                    return;
                };

                self.one_window_hold(
                    "trash",
                    "Clear this conversation",
                    Box::new(move |_, cx| {
                        chat_entity.update(cx, |chat, cx| chat.clear_messages(cx));
                    }),
                    cx,
                );
            }
            "capture_screen_area" => {
                if let AppView::ChatPrompt { entity, .. } = &self.current_view {
                    entity.update(cx, |chat, cx| {
                        chat.capture_screen_area_attachment(cx);
                    });
                }
            }
            _ => {
                logging::log("ACTIONS", &format!("Unknown chat action: {}", action_id));
            }
        }
    }
}
