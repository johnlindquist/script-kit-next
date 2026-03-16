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
                self.show_hud(format!("Model: {}", model_name), Some(HUD_SHORT_MS), cx);
            }
            return;
        }

        // Handle other chat actions
        match action_id {
            "continue_in_chat" => {
                if let AppView::ChatPrompt { entity, .. } = &self.current_view {
                    entity.update(cx, |chat, cx| {
                        chat.handle_continue_in_chat(cx);
                    });
                }
            }
            "copy_response" => {
                if let AppView::ChatPrompt { entity, .. } = &self.current_view {
                    entity.update(cx, |chat, cx| {
                        chat.handle_copy_last_response(cx);
                    });
                    self.show_hud("Copied response".to_string(), Some(HUD_SHORT_MS), cx);
                }
            }
            "clear_conversation" => {
                let chat_entity = if let AppView::ChatPrompt { entity, .. } = &self.current_view {
                    entity.clone()
                } else {
                    return;
                };

                let message = "Are you sure you want to clear this conversation?".to_string();
                cx.spawn(async move |this, cx| {
                    let (confirm_tx, confirm_rx) = async_channel::bounded::<bool>(1);
                    let sender_ok = confirm_tx.clone();
                    let sender_cancel = confirm_tx.clone();

                    let window_handle = crate::get_main_window_handle();
                    let open_result = if let Some(handle) = window_handle {
                        cx.update_window(handle, |_, window, cx| {
                            crate::confirm::open_parent_confirm_dialog(
                                window,
                                cx,
                                crate::confirm::ParentConfirmOptions {
                                    title: "Clear Conversation".into(),
                                    body: gpui::SharedString::from(message),
                                    confirm_text: "Yes".into(),
                                    cancel_text: "Cancel".into(),
                                    ..Default::default()
                                },
                                move |_window, _cx| {
                                    let _ = sender_ok.try_send(true);
                                },
                                move |_window, _cx| {
                                    let _ = sender_cancel.try_send(false);
                                },
                            );
                        })
                    } else {
                        Err(anyhow::anyhow!("Main window handle not available"))
                    };

                    if let Err(e) = open_result {
                        let _ = this.update(cx, |this, cx| {
                            logging::log(
                                "ERROR",
                                &format!("Failed to open confirmation dialog: {}", e),
                            );
                            this.show_hud(
                                "Failed to open confirmation dialog".to_string(),
                                Some(HUD_2500_MS),
                                cx,
                            );
                        });
                        return;
                    }

                    let Ok(confirmed) = confirm_rx.recv().await else {
                        return;
                    };
                    if !confirmed {
                        return;
                    }

                    let _ = this.update(cx, |_, cx| {
                        chat_entity.update(cx, |chat, cx| {
                            chat.clear_messages(cx);
                        });
                    });
                })
                .detach();
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
