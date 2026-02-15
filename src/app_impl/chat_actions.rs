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
                    let open_result = cx.update(|cx| {
                        let main_bounds =
                            if let Some((x, y, w, h)) = platform::get_main_window_bounds() {
                                gpui::Bounds {
                                    origin: gpui::Point {
                                        x: gpui::px(x as f32),
                                        y: gpui::px(y as f32),
                                    },
                                    size: gpui::Size {
                                        width: gpui::px(w as f32),
                                        height: gpui::px(h as f32),
                                    },
                                }
                            } else {
                                gpui::Bounds {
                                    origin: gpui::Point {
                                        x: gpui::px(100.0),
                                        y: gpui::px(100.0),
                                    },
                                    size: gpui::Size {
                                        width: gpui::px(600.0),
                                        height: gpui::px(400.0),
                                    },
                                }
                            };

                        let sender = confirm_tx.clone();
                        let on_choice: ConfirmCallback = std::sync::Arc::new(move |confirmed| {
                            let _ = sender.try_send(confirmed);
                        });

                        open_confirm_window(
                            cx,
                            main_bounds,
                            None,
                            message,
                            Some("Yes".to_string()),
                            Some("Cancel".to_string()),
                            on_choice,
                        )
                    });

                    match open_result {
                        Ok(Ok(_)) => {}
                        Ok(Err(e)) => {
                            this.update(cx, |this, cx| {
                                logging::log(
                                    "ERROR",
                                    &format!("Failed to open confirmation modal: {}", e),
                                );
                                this.show_hud(
                                    "Failed to open confirmation dialog".to_string(),
                                    Some(HUD_2500_MS),
                                    cx,
                                );
                            })
                            .ok();
                            return;
                        }
                        Err(_) => return,
                    }

                    let Ok(confirmed) = confirm_rx.recv().await else {
                        return;
                    };
                    if !confirmed {
                        return;
                    }

                    this.update(cx, |_, cx| {
                        chat_entity.update(cx, |chat, cx| {
                            chat.clear_messages(cx);
                        });
                    })
                    .ok();
                })
                .detach();
            }
            _ => {
                logging::log("ACTIONS", &format!("Unknown chat action: {}", action_id));
            }
        }
    }
}
