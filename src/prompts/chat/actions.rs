use super::*;

const TRANSFER_TO_AI_WINDOW_READY_RETRY_DELAY_MS: u64 = 16;
const TRANSFER_TO_AI_WINDOW_READY_MAX_WAITS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChatPromptDismissalKind {
    CloseInline,
    TransferToAiWindow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransferToAiWindowReadyBarrierStep {
    Ready,
    Wait,
    TimedOut,
}

fn should_persist_chat_before_prompt_dismissal(
    save_history: bool,
    dismissal_kind: ChatPromptDismissalKind,
) -> bool {
    save_history && dismissal_kind == ChatPromptDismissalKind::CloseInline
}

fn next_transfer_to_ai_window_ready_barrier_step(
    is_ready: bool,
    waits_completed: usize,
) -> TransferToAiWindowReadyBarrierStep {
    if is_ready {
        TransferToAiWindowReadyBarrierStep::Ready
    } else if waits_completed < TRANSFER_TO_AI_WINDOW_READY_MAX_WAITS {
        TransferToAiWindowReadyBarrierStep::Wait
    } else {
        TransferToAiWindowReadyBarrierStep::TimedOut
    }
}

impl ChatPrompt {
    pub(super) fn handle_escape(&mut self, _cx: &mut Context<Self>) {
        logging::log("CHAT", "Escape pressed - closing chat");

        // Save conversation to database if save_history is enabled
        if should_persist_chat_before_prompt_dismissal(
            self.save_history,
            ChatPromptDismissalKind::CloseInline,
        ) {
            self.save_to_database();
        }

        if let Some(ref callback) = self.on_escape {
            callback(self.id.clone());
        }
    }

    /// Save the current conversation to the AI chats database
    pub(super) fn save_to_database(&self) {
        // Only save if we have messages
        if self.messages.is_empty() {
            logging::log("CHAT", "No messages to save");
            return;
        }

        // Initialize the AI database if needed
        if let Err(e) = ai::init_ai_db() {
            logging::log("CHAT", &format!("Failed to init AI db: {}", e));
            return;
        }

        // Generate title from first user message
        let title = self
            .messages
            .iter()
            .find(|m| m.is_user())
            .map(|m| Chat::generate_title_from_content(m.get_content()))
            .unwrap_or_else(|| "Chat Prompt Conversation".to_string());

        // Determine the model and provider
        let model_id = self.model.clone().unwrap_or_else(|| "unknown".to_string());
        let provider = self
            .models
            .iter()
            .find(|m| m.name == model_id || m.id == model_id)
            .map(|m| m.provider.clone())
            .unwrap_or_else(|| "unknown".to_string());

        // Create the chat record with ChatPrompt source
        let chat = Chat::new(&model_id, &provider).with_source(ChatSource::ChatPrompt);
        let mut chat = chat;
        chat.set_title(&title);

        // Save the chat
        if let Err(e) = ai::create_chat(&chat) {
            logging::log("CHAT", &format!("Failed to save chat: {}", e));
            return;
        }

        // Save all messages
        for (i, msg) in self.messages.iter().enumerate() {
            let role = if msg.is_user() {
                MessageRole::User
            } else {
                MessageRole::Assistant
            };

            let message = Message::new(chat.id, role, msg.get_content());
            if let Err(e) = ai::save_message(&message) {
                logging::log("CHAT", &format!("Failed to save message {}: {}", i, e));
            }
        }

        logging::log(
            "CHAT",
            &format!(
                "Saved conversation with {} messages (id: {})",
                self.messages.len(),
                chat.id
            ),
        );
    }

    pub fn handle_continue_in_chat(&mut self, cx: &mut Context<Self>) {
        self.transfer_to_ai_window(false, cx);
    }

    pub fn handle_expand_full_chat(&mut self, cx: &mut Context<Self>) {
        self.transfer_to_ai_window(true, cx);
    }

    /// Shared handoff: collect messages, reset inline state, dismiss,
    /// then open the AI window in the requested mode.
    fn transfer_to_ai_window(&mut self, full_mode: bool, cx: &mut Context<Self>) {
        let mode_label = if full_mode { "full" } else { "mini" };
        let transfer_start = std::time::Instant::now();
        tracing::info!(
            action = "transfer_to_ai_window",
            target_mode = mode_label,
            "=== BEACHBALL TRACE: transfer_to_ai_window START ==="
        );
        logging::log(
            "CHAT",
            &format!("Transfer to AI window (mode={})", mode_label),
        );

        // Collect conversation history from messages, including image attachments
        let messages: Vec<ai::PendingChatMessage> = self
            .messages
            .iter()
            .map(|m| {
                let role = if m.is_user() {
                    MessageRole::User
                } else {
                    MessageRole::Assistant
                };
                ai::PendingChatMessage {
                    role,
                    content: m.get_content().to_string(),
                    image_base64: m.image.clone(),
                }
            })
            .collect();

        let message_count = messages.len();
        let image_count = messages.iter().filter(|m| m.image_base64.is_some()).count();
        tracing::info!(
            action = "transfer_to_ai_window",
            target_mode = mode_label,
            message_count = message_count,
            image_count = image_count,
            "Transferring conversation to AI window"
        );

        if should_persist_chat_before_prompt_dismissal(
            self.save_history,
            ChatPromptDismissalKind::TransferToAiWindow,
        ) {
            self.save_to_database();
        } else if self.save_history {
            tracing::info!(
                action = "transfer_to_ai_window",
                target_mode = mode_label,
                persistence = "initialize_with_pending_chat",
                message_count,
                image_count,
                "Skipping inline save_to_database before AI handoff"
            );
        }

        // Reset the inline prompt to empty state BEFORE the deferred AI open
        self.messages.clear();
        self.streaming_message_id = None;
        self.user_has_scrolled_up = false;
        self.input.clear();
        self.pending_image = None;
        self.pending_image_render = None;
        self.image_render_cache.clear();
        self.mark_conversation_turns_dirty();
        self.ensure_conversation_turns_cache();
        cx.notify();

        tracing::info!(
            action = "transfer_to_ai_window",
            elapsed_ms = transfer_start.elapsed().as_millis(),
            "BEACHBALL TRACE: state reset done, about to dismiss"
        );

        // Dismiss the main prompt window.
        // Use on_continue (hides main window) for transfer, falling back to on_escape
        // (returns to script list) if on_continue is not wired.
        if let Some(ref callback) = self.on_continue {
            callback(self.id.clone());
        } else if let Some(ref callback) = self.on_escape {
            callback(self.id.clone());
        }

        tracing::info!(
            action = "transfer_to_ai_window",
            elapsed_ms = transfer_start.elapsed().as_millis(),
            "BEACHBALL TRACE: dismiss done, spawning async open"
        );

        // Defer AI window open so the inline prompt dismisses first,
        // avoiding synchronous image transfer work on the original prompt path.
        cx.spawn(async move |_this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(1))
                .await;

            let open_start = std::time::Instant::now();
            tracing::info!(
                action = "transfer_to_ai_window",
                target_mode = mode_label,
                "BEACHBALL TRACE: async open starting"
            );

            let open_result = cx.update(|cx| {
                if full_mode {
                    ai::open_ai_window(cx).map_err(|error| {
                        format!("failed to open full AI window for chat transfer: {error}")
                    })?;
                } else {
                    ai::open_mini_ai_window(cx).map_err(|error| {
                        format!("failed to open mini AI window for chat transfer: {error}")
                    })?;
                }
                Ok::<(), String>(())
            });

            tracing::info!(
                action = "transfer_to_ai_window",
                target_mode = mode_label,
                open_elapsed_ms = open_start.elapsed().as_millis(),
                "BEACHBALL TRACE: window open complete"
            );

            let handoff_result = match open_result {
                Ok(()) => {
                    tracing::info!(
                        action = "transfer_to_ai_window",
                        target_mode = mode_label,
                        max_waits = TRANSFER_TO_AI_WINDOW_READY_MAX_WAITS,
                        retry_delay_ms = TRANSFER_TO_AI_WINDOW_READY_RETRY_DELAY_MS,
                        "Waiting for AI window readiness before pending chat handoff"
                    );

                    let mut waits_completed = 0usize;
                    loop {
                        let ready_now = cx.update(ai::is_ai_window_ready);
                        match next_transfer_to_ai_window_ready_barrier_step(
                            ready_now,
                            waits_completed,
                        ) {
                            TransferToAiWindowReadyBarrierStep::Ready => {
                                break cx.update(|cx| {
                                    ai::set_ai_pending_chat(cx, messages).map_err(|error| {
                                        format!(
                                            "failed to stash pending chat after AI window became ready: {error}"
                                        )
                                    })
                                });
                            }
                            TransferToAiWindowReadyBarrierStep::Wait => {
                                waits_completed += 1;
                                tracing::debug!(
                                    action = "transfer_to_ai_window",
                                    target_mode = mode_label,
                                    waits_completed,
                                    max_waits = TRANSFER_TO_AI_WINDOW_READY_MAX_WAITS,
                                    retry_delay_ms = TRANSFER_TO_AI_WINDOW_READY_RETRY_DELAY_MS,
                                    "AI window not ready yet; retrying pending chat handoff"
                                );
                                cx.background_executor()
                                    .timer(std::time::Duration::from_millis(
                                        TRANSFER_TO_AI_WINDOW_READY_RETRY_DELAY_MS,
                                    ))
                                    .await;
                            }
                            TransferToAiWindowReadyBarrierStep::TimedOut => {
                                break Err(format!(
                                    "AI window not ready after open; cannot hand off pending chat (mode={mode_label}, waits_completed={waits_completed}, max_waits={}, retry_delay_ms={}, message_count={message_count}, image_count={image_count})",
                                    TRANSFER_TO_AI_WINDOW_READY_MAX_WAITS,
                                    TRANSFER_TO_AI_WINDOW_READY_RETRY_DELAY_MS,
                                ));
                            }
                        }
                    }
                }
                Err(error) => Err(error),
            };

            match handoff_result {
                Ok(()) => {
                    tracing::info!(
                        action = "transfer_to_ai_window",
                        target_mode = mode_label,
                        message_count,
                        image_count,
                        "AI window opened with deferred pending chat"
                    );
                }
                Err(error) => {
                    tracing::error!(
                        error = %error,
                        target_mode = mode_label,
                        message_count,
                        image_count,
                        "Failed to open AI window for chat transfer"
                    );
                }
            }
        })
        .detach();
    }

    pub fn handle_copy_last_response(&mut self, cx: &mut Context<Self>) {
        // Find the last assistant message
        if let Some(last_assistant) = self.messages.iter().rev().find(|m| !m.is_user()) {
            let content = last_assistant.get_content().to_string();
            self.last_copied_response = Some(content.clone());
            logging::log("CHAT", &format!("Copied response: {} chars", content.len()));
            // Copy to clipboard via cx
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(content));
        }
    }

    pub(super) fn handle_clear(&mut self, cx: &mut Context<Self>) {
        logging::log("CHAT", "Clearing conversation (⌘+⌫)");
        self.clear_messages(cx);
    }

    pub(super) fn handle_script_generation_action(
        &mut self,
        action: ScriptGenerationAction,
        cx: &mut Context<Self>,
    ) {
        let Some((prompt_description, raw_response)) = self.latest_script_generation_draft() else {
            self.set_script_generation_status(true, "No generated script to save yet.", cx);
            return;
        };

        logging::log(
            "CHAT_SCRIPT_GEN",
            &format!(
                "state=save_requested action={:?} prompt_len={} response_len={}",
                action,
                prompt_description.len(),
                raw_response.len()
            ),
        );

        let script_path = match crate::ai::script_generation::save_generated_script_from_response(
            &prompt_description,
            &raw_response,
        ) {
            Ok(path) => path,
            Err(error) => {
                self.set_script_generation_status(
                    true,
                    format!("Failed to save script: {}", error),
                    cx,
                );
                return;
            }
        };

        let script_name = script_path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "generated script".to_string());

        if action.should_run_after_save() {
            self.set_script_generation_status(false, format!("Running {}...", script_name), cx);
            if let Some(ref callback) = self.on_run_script {
                callback(script_path.clone(), cx);
                self.set_script_generation_status(
                    false,
                    format!("Saved and running {}", script_name),
                    cx,
                );
                logging::log(
                    "CHAT_SCRIPT_GEN",
                    &format!(
                        "state=run_dispatched action={:?} path={}",
                        action,
                        script_path.display()
                    ),
                );
            } else {
                self.set_script_generation_status(
                    true,
                    format!("Saved {} but run action is unavailable", script_name),
                    cx,
                );
                logging::log(
                    "CHAT_SCRIPT_GEN",
                    &format!(
                        "state=run_dispatch_failed action={:?} path={} reason=missing_callback",
                        action,
                        script_path.display()
                    ),
                );
            }
            return;
        }

        self.set_script_generation_status(false, format!("Saved {}", script_name), cx);
        logging::log(
            "CHAT_SCRIPT_GEN",
            &format!(
                "state=saved_only action={:?} path={}",
                action,
                script_path.display()
            ),
        );

        // Notify parent to show CreationFeedback panel
        if let Some(ref callback) = self.on_script_saved {
            callback(script_path, cx);
        }
    }

    // ============================================
    // Actions Menu Methods
    // ============================================

    pub(super) fn toggle_actions_menu(&mut self, _cx: &mut Context<Self>) {
        // Delegate to parent via callback to open standard ActionsDialog
        if let Some(ref callback) = self.on_show_actions {
            tracing::info!(
                event = "toggle_actions_menu.delegated",
                id = %self.id,
                mini_mode = self.mini_mode,
                "ChatPrompt delegating actions toggle to parent via callback"
            );
            callback(self.id.clone());
        } else {
            tracing::warn!(
                event = "toggle_actions_menu.no_callback",
                id = %self.id,
                "No on_show_actions callback set — actions toggle request dropped"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        next_transfer_to_ai_window_ready_barrier_step, should_persist_chat_before_prompt_dismissal,
        ChatPromptDismissalKind, TransferToAiWindowReadyBarrierStep,
        TRANSFER_TO_AI_WINDOW_READY_MAX_WAITS,
    };

    #[test]
    fn test_should_persist_chat_before_prompt_dismissal_when_closing_inline_with_history_enabled() {
        assert!(should_persist_chat_before_prompt_dismissal(
            true,
            ChatPromptDismissalKind::CloseInline
        ));
    }

    #[test]
    fn test_should_not_persist_chat_before_prompt_dismissal_when_transferring_to_ai_window() {
        assert!(!should_persist_chat_before_prompt_dismissal(
            true,
            ChatPromptDismissalKind::TransferToAiWindow
        ));
    }

    #[test]
    fn test_should_not_persist_chat_before_prompt_dismissal_when_history_is_disabled() {
        assert!(!should_persist_chat_before_prompt_dismissal(
            false,
            ChatPromptDismissalKind::CloseInline
        ));
    }

    #[test]
    fn test_next_transfer_to_ai_window_ready_barrier_step_returns_ready_immediately() {
        assert_eq!(
            next_transfer_to_ai_window_ready_barrier_step(true, 0),
            TransferToAiWindowReadyBarrierStep::Ready
        );
        assert_eq!(
            next_transfer_to_ai_window_ready_barrier_step(
                true,
                TRANSFER_TO_AI_WINDOW_READY_MAX_WAITS
            ),
            TransferToAiWindowReadyBarrierStep::Ready
        );
    }

    #[test]
    fn test_next_transfer_to_ai_window_ready_barrier_step_retries_before_timeout() {
        assert_eq!(
            next_transfer_to_ai_window_ready_barrier_step(false, 0),
            TransferToAiWindowReadyBarrierStep::Wait
        );
        assert_eq!(
            next_transfer_to_ai_window_ready_barrier_step(
                false,
                TRANSFER_TO_AI_WINDOW_READY_MAX_WAITS - 1
            ),
            TransferToAiWindowReadyBarrierStep::Wait
        );
        assert_eq!(
            next_transfer_to_ai_window_ready_barrier_step(
                false,
                TRANSFER_TO_AI_WINDOW_READY_MAX_WAITS
            ),
            TransferToAiWindowReadyBarrierStep::TimedOut
        );
    }
}
