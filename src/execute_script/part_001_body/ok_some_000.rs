                                let message_id = msg
                                    .id()
                                    .map(str::to_string)
                                    .unwrap_or_else(|| format!("msg:{}", uuid::Uuid::new_v4()));
                                let _message_guard =
                                    logging::set_correlation_id(format!("protocol:{}", message_id));
                                let payload_summary = serde_json::to_string(&msg)
                                    .map(|json| logging::summarize_payload(&json))
                                    .unwrap_or_else(|_| "{serialize_error}".to_string());
                                tracing::debug!(
                                    category = "EXEC",
                                    event_type = "protocol_message_received",
                                    message_id = %message_id,
                                    payload_summary = %payload_summary,
                                    "Received protocol message"
                                );

                                // First, try to handle selected text messages directly (no UI needed)
                                match executor::handle_selected_text_message(&msg) {
                                    executor::SelectedTextHandleResult::Handled(response) => {
                                        let response_summary = serde_json::to_string(&response)
                                            .map(|json| logging::summarize_payload(&json))
                                            .unwrap_or_else(|_| "{serialize_error}".to_string());
                                        tracing::debug!(
                                            category = "EXEC",
                                            event_type = "protocol_selected_text_response",
                                            message_id = %message_id,
                                            payload_summary = %response_summary,
                                            "Handled selected text message, sending response"
                                        );
                                        if let Err(e) = reader_response_tx.send(response) {
                                            logging::log(
                                                "EXEC",
                                                &format!(
                                                    "Failed to send selected text response: {}",
                                                    e
                                                ),
                                            );
                                        }
                                        continue;
                                    }
                                    executor::SelectedTextHandleResult::NotHandled => {
                                        // Fall through to other message handling
                                    }
                                }

