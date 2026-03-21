use super::*;
use crate::ai::model::ImageAttachment;
use crate::ai::providers::{ProviderImage, ProviderMessage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StreamingStartMode {
    MockMode,
    MissingSelectedModel,
    RealProviderStream,
}

fn resolve_streaming_start_mode(
    selected_model: Option<&ModelInfo>,
    available_models: &[ModelInfo],
) -> StreamingStartMode {
    if available_models.is_empty() {
        StreamingStartMode::MockMode
    } else if selected_model.is_none() {
        StreamingStartMode::MissingSelectedModel
    } else {
        StreamingStartMode::RealProviderStream
    }
}

fn streaming_provider_panic_payload_to_message(
    panic_payload: &(dyn std::any::Any + Send),
) -> String {
    if let Some(message) = panic_payload.downcast_ref::<String>() {
        return message.clone();
    }

    if let Some(message) = panic_payload.downcast_ref::<&str>() {
        return (*message).to_string();
    }

    "unknown panic payload".to_string()
}

fn ai_window_drain_streaming_deltas(
    shared_deltas: &std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    accumulated_content: &mut String,
) -> Result<Option<String>, String> {
    let mut pending_deltas = shared_deltas
        .lock()
        .map_err(|err| format!("failed to lock streaming delta queue: {err}"))?;

    if pending_deltas.is_empty() {
        return Ok(None);
    }

    let mut drained_delta = String::new();
    for chunk in pending_deltas.drain(..) {
        drained_delta.push_str(&chunk);
    }

    if drained_delta.is_empty() {
        return Ok(None);
    }

    accumulated_content.push_str(&drained_delta);
    Ok(Some(drained_delta))
}

fn ai_window_submit_message_create_chat_error_message() -> String {
    "Failed to create a new chat. Check storage/database configuration and retry.".to_string()
}

fn ai_window_submit_message_save_error_message(
    chat_id: ChatId,
    error: &impl std::fmt::Display,
) -> String {
    format!("Failed to save your message for chat '{chat_id}': {error}")
}

impl AiApp {
    pub(super) fn submit_message(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let content = self.input_state.read(cx).value().to_string();
        let has_pending_image = self.pending_image.is_some();
        let has_pending_context_parts = !self.pending_context_parts.is_empty();

        if !ai_window_can_submit_message(&content, has_pending_image, has_pending_context_parts) {
            tracing::debug!(
                checkpoint = "submit_guard",
                content_empty = content.trim().is_empty(),
                has_pending_image,
                has_pending_context_parts,
                "submit_guard: rejected — no submittable content"
            );
            return;
        }

        tracing::info!(
            checkpoint = "submit_guard",
            content_len = content.len(),
            has_pending_image,
            has_pending_context_parts,
            "submit_guard: passed"
        );

        // If we are in editing mode, delegate to the edit-submit flow
        if self.editing_message_id.is_some() {
            self.submit_edited_message(window, cx);
            return;
        }

        // Don't allow new messages while streaming for the CURRENT chat
        // (streaming for a different chat is fine - the guard handles it)
        if self.is_streaming && self.streaming_chat_id == self.selected_chat_id {
            return;
        }

        // Clear previous generation stats so they don't persist across messages
        self.last_streaming_completed_at = None;
        self.last_streaming_duration = None;

        // If no chat selected, create a new one
        let chat_id = if let Some(id) = self.selected_chat_id {
            id
        } else {
            match self.create_chat(window, cx) {
                Some(id) => id,
                None => {
                    let message = ai_window_submit_message_create_chat_error_message();
                    tracing::error!(error = %message, "Failed to create chat for message submission");
                    self.streaming_error = Some(message);
                    cx.notify();
                    return;
                }
            }
        };

        // Capture pending image only after all early-return guards so we don't drop attachments.
        let pending_image = self.pending_image.take();
        let has_image = pending_image.is_some();

        if let Some(ref image_base64) = pending_image {
            // Calculate approximate image size for logging
            let image_size_kb = image_base64.len() / 1024;
            tracing::info!(target: "ai", image_size_kb = image_size_kb, "Message includes attached image");
        }

        // Resolve pending context parts into a prompt prefix just-in-time.
        // pending_context_parts is the single source of truth — no separate attachments vec.
        let pending_parts = std::mem::take(&mut self.pending_context_parts);

        // --- Shared outbound compiler: parse directives, merge, resolve ---
        let super::chat::OutboundUserMessagePreparation {
            receipt,
            authored_content,
            has_context_parts,
        } = self.prepare_outbound_user_message(&content, &pending_parts);

        self.last_prepared_message_receipt = Some(receipt.clone());

        // Build correlation-scoped preflight audit and persist before send.
        let mut preflight_audit = crate::ai::AiPreflightAudit::new(
            &chat_id,
            &content,
            &authored_content,
            has_image,
            has_context_parts,
            receipt.clone(),
        );
        self.last_preflight_audit = Some(preflight_audit.clone());

        if let Err(error) = crate::ai::save_message_preparation_audit(&preflight_audit) {
            tracing::warn!(
                chat_id = %chat_id,
                correlation_id = %preflight_audit.correlation_id,
                error = %error,
                "Failed to persist preflight audit"
            );
        }

        crate::ai::log_preflight_audit(&preflight_audit, "prepared");

        self.last_context_receipt = if has_context_parts {
            Some(receipt.context.clone())
        } else {
            None
        };

        let final_user_content = match receipt.decision {
            crate::ai::message_parts::PreparedMessageDecision::Ready => {
                self.streaming_error = None;
                receipt.final_user_content
            }
            crate::ai::message_parts::PreparedMessageDecision::Partial => {
                tracing::warn!(
                    checkpoint = "resolution_partial",
                    attempted = receipt.context.attempted,
                    resolved = receipt.context.resolved,
                    outcomes = ?receipt.outcomes,
                    failures = ?receipt.context.failures,
                    correlation_id = %preflight_audit.correlation_id,
                    "submit_context: partial resolution failure"
                );
                self.pending_context_parts = receipt.unresolved_parts;
                self.streaming_error = crate::ai::build_actionable_preflight_error(&preflight_audit)
                    .or_else(|| receipt.user_error.clone());
                receipt.final_user_content
            }
            crate::ai::message_parts::PreparedMessageDecision::Blocked => {
                tracing::warn!(
                    checkpoint = "resolution_blocked",
                    attempted = receipt.context.attempted,
                    resolved = receipt.context.resolved,
                    outcomes = ?receipt.outcomes,
                    failures = ?receipt.context.failures,
                    correlation_id = %preflight_audit.correlation_id,
                    "submit_context: blocked due to unresolved context"
                );
                self.pending_context_parts = receipt.unresolved_parts;
                self.streaming_error = crate::ai::build_actionable_preflight_error(&preflight_audit)
                    .or_else(|| receipt.user_error.clone());
                cx.notify();
                return;
            }
        };

        // Clear preflight state since context parts have been consumed
        self.clear_context_preflight(cx);

        tracing::info!(
            checkpoint = "cleanup",
            cleared_parts = pending_parts.len(),
            "submit_context: cleared pending context parts and attachments"
        );

        // Update chat title if this is the first message — use cleaned authored content
        let display_source =
            ai_window_outbound_display_source(&authored_content, has_image, has_context_parts);
        if let Some(chat) = self.chats.iter_mut().find(|c| c.id == chat_id) {
            if chat.title == "New Chat" {
                let new_title = Chat::generate_title_from_content(&display_source);
                chat.set_title(&new_title);

                // Persist title update
                if let Err(e) = storage::update_chat_title(&chat_id, &new_title) {
                    tracing::error!(error = %e, "Failed to update chat title");
                }
            }
        }

        // Create and save user message with resolved content (context prefix + user text)
        let mut user_message = Message::user(chat_id, &final_user_content);

        // Attach image if present
        if let Some(image_base64) = pending_image {
            user_message.images.push(ImageAttachment::png(image_base64));
        }

        if let Err(e) = storage::save_message(&user_message) {
            tracing::error!(error = %e, "Failed to save user message");
            self.streaming_error = Some(ai_window_submit_message_save_error_message(chat_id, &e));
            cx.notify();
            return;
        }

        // Link the saved message ID to the preflight audit and re-persist.
        preflight_audit.attach_message_id(&user_message.id);
        self.last_preflight_audit = Some(preflight_audit.clone());

        if let Err(error) = crate::ai::save_message_preparation_audit(&preflight_audit) {
            tracing::warn!(
                chat_id = %chat_id,
                message_id = %user_message.id,
                correlation_id = %preflight_audit.correlation_id,
                error = %error,
                "Failed to upsert linked preflight audit"
            );
        }

        crate::ai::log_preflight_audit(&preflight_audit, "message_saved");

        // Add to current messages for display
        self.current_messages.push(user_message);

        // Force scroll to bottom when user sends a new message (always scroll, even if scrolled up)
        self.force_scroll_to_bottom();

        // Update message preview and count cache — derived from cleaned authored content
        let preview_source = display_source.as_str();
        let preview: String = preview_source.chars().take(60).collect();
        let preview = if preview.len() < preview_source.len() {
            format!("{}...", preview.trim())
        } else {
            preview
        };
        self.message_previews.insert(chat_id, preview);
        self.message_counts
            .insert(chat_id, self.current_messages.len());

        // Update chat timestamp and move to top of list
        self.touch_and_reorder_chat(chat_id);

        // Clear the input (pending image was already taken above)
        self.clear_composer(window, cx);

        // Update placeholder to "Reply to..." now that we have messages
        self.update_input_placeholder(window, cx);

        info!(
            chat_id = %chat_id,
            content_len = content.len(),
            has_image = has_image,
            "User message submitted"
        );

        // Start streaming response
        self.start_streaming_response(chat_id, cx);

        cx.notify();
    }

    /// Start streaming an AI response (or mock response if no providers configured)
    pub(super) fn start_streaming_response(&mut self, chat_id: ChatId, cx: &mut Context<Self>) {
        match resolve_streaming_start_mode(self.selected_model.as_ref(), &self.available_models) {
            StreamingStartMode::MockMode => {
                info!(chat_id = %chat_id, "No AI providers configured - using mock mode");
                self.start_mock_streaming_response(chat_id, cx);
                return;
            }
            StreamingStartMode::MissingSelectedModel => {
                let message = "Selected model not found".to_string();
                self.streaming_error = Some(message.clone());
                tracing::error!(
                    chat_id = %chat_id,
                    available_models = self.available_models.len(),
                    error = %message,
                    "Cannot start streaming response"
                );
                cx.notify();
                return;
            }
            StreamingStartMode::RealProviderStream => {}
        }

        // Get the selected model
        let model = match &self.selected_model {
            Some(m) => m.clone(),
            None => {
                let message = "Selected model not found".to_string();
                self.streaming_error = Some(message.clone());
                tracing::error!(
                    chat_id = %chat_id,
                    error = %message,
                    "Selected model unexpectedly missing after streaming mode resolution"
                );
                cx.notify();
                return;
            }
        };

        // Find the provider for this model
        let provider = match self.provider_registry.get_provider(&model.provider) {
            Some(p) => p.clone(),
            None => {
                let message = Self::provider_unavailable_error_message(&model.id, &model.provider);
                tracing::error!(
                    model_id = %model.id,
                    provider = %model.provider,
                    error = %message,
                    "No provider found for selected model provider"
                );
                self.clear_streaming_state_with_error(message, cx);
                return;
            }
        };

        // Build messages for the API call
        let api_messages: Vec<ProviderMessage> = self
            .current_messages
            .iter()
            .map(|m| ProviderMessage {
                role: m.role.to_string(),
                content: m.content.clone(),
                images: m
                    .images
                    .iter()
                    .map(|img| ProviderImage {
                        data: img.data.clone(),
                        media_type: img.media_type.clone(),
                    })
                    .collect(),
            })
            .collect();

        // Set streaming state with chat-scoping guards
        self.is_streaming = true;
        self.streaming_content.clear();
        self.streaming_error = None;
        self.streaming_chat_id = Some(chat_id);
        self.streaming_generation = self.streaming_generation.wrapping_add(1);
        self.streaming_started_at = Some(std::time::Instant::now());
        let generation = self.streaming_generation;

        // Publish streaming state for SDK handlers
        publish_streaming_state(AiStreamingSnapshot {
            is_streaming: true,
            chat_id: Some(chat_id.as_str()),
            partial_content: None,
        });

        info!(
            chat_id = %chat_id,
            generation = generation,
            model = model.id,
            provider = model.provider,
            message_count = api_messages.len(),
            "Starting AI streaming response"
        );

        // Shared delta queue: provider pushes chunks, UI drains and appends incrementally.
        let shared_deltas = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let shared_done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let shared_error = std::sync::Arc::new(std::sync::Mutex::new(None::<String>));
        let cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        self.streaming_cancel = Some(cancelled.clone());

        let model_id = model.id.clone();
        let deltas_for_thread = shared_deltas.clone();
        let done_clone = shared_done.clone();
        let error_clone = shared_error.clone();
        let cancelled_clone = cancelled.clone();
        let thread_chat_id = chat_id;
        let thread_generation = generation;
        // Use chat_id as session_id for Claude Code CLI conversation continuity
        let session_id = chat_id.to_string();

        // Spawn background thread for streaming
        std::thread::spawn(move || {
            let callback_deltas = deltas_for_thread.clone();
            let callback_error = error_clone.clone();
            let callback_model_id = model_id.clone();
            let stream_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                provider.stream_message(
                    &api_messages,
                    &model_id,
                    Box::new(move |chunk| {
                        if cancelled_clone.load(std::sync::atomic::Ordering::SeqCst) {
                            return false;
                        }

                        match callback_deltas.lock() {
                            Ok(mut pending_deltas) => {
                                pending_deltas.push(chunk);
                                true
                            }
                            Err(err) => {
                                let error_message =
                                    format!("Failed to queue streaming delta: {err}");
                                tracing::error!(
                                    chat_id = %thread_chat_id,
                                    generation = thread_generation,
                                    model_id = %callback_model_id,
                                    error = %error_message,
                                    "Streaming delta queue lock poisoned"
                                );
                                if let Ok(mut error_state) = callback_error.lock() {
                                    *error_state = Some(error_message);
                                }
                                false
                            }
                        }
                    }),
                    Some(&session_id),
                )
            }));

            match stream_result {
                Ok(Ok(())) => {
                    done_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                }
                Ok(Err(e)) => {
                    if let Ok(mut err) = error_clone.lock() {
                        *err = Some(e.to_string());
                    }
                    done_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                }
                Err(panic_payload) => {
                    let panic_message =
                        streaming_provider_panic_payload_to_message(panic_payload.as_ref());
                    let error_message = format!("Streaming provider panicked: {panic_message}");
                    tracing::error!(
                        chat_id = %thread_chat_id,
                        generation = thread_generation,
                        model_id = %model_id,
                        panic = %panic_message,
                        "Streaming provider thread panicked"
                    );
                    if let Ok(mut err) = error_clone.lock() {
                        *err = Some(error_message);
                    }
                    done_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                }
            }
        });

        // Poll for streaming updates using background executor
        let deltas_for_poll = shared_deltas.clone();
        let done_for_poll = shared_done.clone();
        let error_for_poll = shared_error.clone();

        cx.spawn(async move |this, cx| {
            let mut accumulated_content = String::new();
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(50))
                    .await;

                let drained_delta = match ai_window_drain_streaming_deltas(
                    &deltas_for_poll,
                    &mut accumulated_content,
                ) {
                    Ok(delta) => delta,
                    Err(lock_error) => {
                        tracing::error!(
                            chat_id = %chat_id,
                            generation = generation,
                            error = %lock_error,
                            "Failed to drain streaming deltas"
                        );
                        let lock_error_for_ui = lock_error.clone();
                        let _ = cx.update(|cx| {
                            this.update(cx, move |app, cx| {
                                if app.streaming_generation != generation
                                    || app.streaming_chat_id != Some(chat_id)
                                {
                                    return;
                                }
                                app.streaming_error = Some(lock_error_for_ui);
                                app.streaming_started_at = None;
                                app.is_streaming = false;
                                app.streaming_content.clear();
                                app.streaming_chat_id = None;
                                app.streaming_cancel = None;
                                publish_streaming_state(AiStreamingSnapshot::default());
                                cx.notify();
                            })
                        });
                        break;
                    }
                };

                if let Some(delta) = drained_delta {
                    let _ = cx.update(|cx| {
                        this.update(cx, move |app, cx| {
                            // Guard: only update UI if this is the current streaming session
                            if app.streaming_generation != generation
                                || app.streaming_chat_id != Some(chat_id)
                            {
                                return; // Stale update, ignore
                            }
                            if app.selected_chat_id != Some(chat_id) {
                                return; // Belt-and-suspenders: don't render into a different active chat
                            }
                            app.streaming_content.push_str(&delta);
                            // Auto-scroll to bottom as new content arrives
                            app.sync_messages_list_and_scroll_to_bottom();
                            cx.notify();
                        })
                    });
                }

                // Check if done or errored
                if done_for_poll.load(std::sync::atomic::Ordering::SeqCst) {
                    // Final content has already been assembled incrementally from drained deltas.
                    let final_content = if accumulated_content.is_empty() {
                        None
                    } else {
                        Some(accumulated_content.clone())
                    };
                    let error = error_for_poll.lock().ok().and_then(|e| e.clone());

                    let _ = cx.update(|cx| {
                        this.update(cx, |app, cx| {
                            // CRITICAL: Guard against stale updates from chat-switch
                            // If generation doesn't match, this is an old streaming task
                            if app.streaming_generation != generation
                                || app.streaming_chat_id != Some(chat_id)
                            {
                                tracing::debug!(
                                    expected_gen = generation,
                                    actual_gen = app.streaming_generation,
                                    expected_chat = %chat_id,
                                    actual_chat = ?app.streaming_chat_id,
                                    "Ignoring stale streaming completion (user switched chats)"
                                );
                                let should_persist =
                                    app.should_persist_orphaned_completion(chat_id, generation);

                                if !should_persist {
                                    tracing::info!(
                                        chat_id = %chat_id,
                                        generation = generation,
                                        "Dropping stale completion after explicit stop/delete"
                                    );
                                    return;
                                }

                                // Persist stale completion for chat-switch continuity.
                                if let Some(err) = &error {
                                    tracing::error!(error = %err, chat_id = %chat_id, "Stale streaming error");
                                } else if let Some(content) = &final_content {
                                    // Save orphaned message to DB
                                    if !content.is_empty() {
                                        let assistant_message =
                                            Message::assistant(chat_id, content);
                                        if let Err(e) = storage::save_message(&assistant_message) {
                                            tracing::error!(error = %e, "Failed to save orphaned assistant message");
                                        } else {
                                            tracing::info!(
                                                chat_id = %chat_id,
                                                content_len = content.len(),
                                                "Orphaned streaming response saved to DB"
                                            );
                                            let preview: String =
                                                content.chars().take(60).collect();
                                            let preview = if preview.len() < content.len() {
                                                format!("{}...", preview.trim())
                                            } else {
                                                preview
                                            };
                                            app.message_previews.insert(chat_id, preview);
                                            let count = app
                                                .message_counts
                                                .get(&chat_id)
                                                .copied()
                                                .unwrap_or(0);
                                            app.message_counts.insert(chat_id, count + 1);
                                            app.touch_and_reorder_chat(chat_id);
                                            cx.notify();
                                        }
                                    }
                                }
                                return;
                            }

                            if let Some(err) = error {
                                tracing::error!(error = %err, "Streaming error");
                                app.streaming_error = Some(err);
                                app.streaming_started_at = None;
                                app.is_streaming = false;
                                app.streaming_content.clear();
                                app.streaming_chat_id = None;
                                app.streaming_cancel = None;
                                publish_streaming_state(AiStreamingSnapshot::default());
                            } else if let Some(content) = final_content {
                                app.streaming_content = content;
                                app.finish_streaming(chat_id, generation, cx);
                            }
                            cx.notify();
                        })
                    });
                    break;
                }
            }
        })
        .detach();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_streaming_start_mode_uses_mock_only_when_no_models_available() {
        let mode = resolve_streaming_start_mode(None, &[]);
        assert_eq!(
            mode,
            StreamingStartMode::MockMode,
            "Mock mode should only be used when there are no configured models"
        );
    }

    #[test]
    fn test_resolve_streaming_start_mode_reports_missing_selected_model_when_models_exist() {
        let available_models = vec![ModelInfo::new(
            "shared-model",
            "Shared",
            "openai",
            true,
            128_000,
        )];

        let mode = resolve_streaming_start_mode(None, &available_models);
        assert_eq!(
            mode,
            StreamingStartMode::MissingSelectedModel,
            "When models exist but selected_model is None, streaming should fail with a model-not-found path"
        );
    }

    #[test]
    fn test_resolve_streaming_start_mode_uses_real_stream_when_model_is_selected() {
        let selected = ModelInfo::new("shared-model", "Shared", "openai", true, 128_000);
        let available_models = vec![selected.clone()];

        let mode = resolve_streaming_start_mode(Some(&selected), &available_models);
        assert_eq!(
            mode,
            StreamingStartMode::RealProviderStream,
            "Real provider streaming should continue when both selected model and available models exist"
        );
    }

    #[test]
    fn test_ai_window_drain_streaming_deltas_drains_queue_and_appends_accumulator_when_chunks_exist(
    ) {
        let shared_deltas = std::sync::Arc::new(std::sync::Mutex::new(vec![
            "hello".to_string(),
            " world".to_string(),
        ]));
        let mut accumulated_content = String::new();

        let drained_delta =
            ai_window_drain_streaming_deltas(&shared_deltas, &mut accumulated_content)
                .expect("draining streaming deltas should succeed");

        assert_eq!(
            drained_delta.as_deref(),
            Some("hello world"),
            "Draining should concatenate all pending chunks in order"
        );
        assert_eq!(
            accumulated_content, "hello world",
            "Accumulated content should include drained deltas"
        );
        assert!(
            shared_deltas
                .lock()
                .expect("delta queue should still be lockable")
                .is_empty(),
            "Drain should remove all queued deltas"
        );
    }

    #[test]
    fn test_ai_window_drain_streaming_deltas_returns_none_when_queue_has_no_chunks() {
        let shared_deltas = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let mut accumulated_content = "existing".to_string();

        let drained_delta =
            ai_window_drain_streaming_deltas(&shared_deltas, &mut accumulated_content)
                .expect("draining empty queue should still succeed");

        assert_eq!(
            drained_delta, None,
            "No UI delta should be emitted when queue is empty"
        );
        assert_eq!(
            accumulated_content, "existing",
            "Accumulator should not change when no deltas are pending"
        );
    }

    #[test]
    fn test_streaming_provider_panic_payload_to_message_extracts_string_and_str_payloads() {
        let owned_message = "owned panic payload".to_string();
        let borrowed_message = "borrowed panic payload";

        assert_eq!(
            streaming_provider_panic_payload_to_message(&owned_message),
            owned_message,
            "Owned String panic payloads should preserve their message"
        );
        assert_eq!(
            streaming_provider_panic_payload_to_message(&borrowed_message),
            borrowed_message,
            "&str panic payloads should preserve their message"
        );
    }

    #[test]
    fn test_submit_message_create_chat_error_message_is_actionable() {
        let message = ai_window_submit_message_create_chat_error_message();
        assert!(
            message.contains("storage/database"),
            "Create-chat submission error should direct users to storage configuration"
        );
    }

    #[test]
    fn test_submit_message_save_error_message_includes_chat_and_error() {
        let chat_id = ChatId::new();
        let message = ai_window_submit_message_save_error_message(chat_id, &"disk full");

        assert!(
            message.contains(&chat_id.to_string()),
            "Save-message submission error should include chat ID"
        );
        assert!(
            message.contains("disk full"),
            "Save-message submission error should include underlying storage error"
        );
    }

    #[test]
    fn test_orphaned_completion_preview_truncates_and_appends_ellipsis() {
        let content = "a".repeat(70);
        let preview: String = content.chars().take(60).collect();
        let preview = if preview.len() < content.len() {
            format!("{}...", preview.trim())
        } else {
            preview
        };

        assert_eq!(
            preview.len(),
            63,
            "Long orphaned completion previews should truncate to 60 chars plus ellipsis"
        );
        assert!(
            preview.ends_with("..."),
            "Long orphaned completion previews should end with ellipsis"
        );
    }

    #[test]
    fn test_orphaned_completion_preview_keeps_short_content() {
        let content = "short response";
        let preview: String = content.chars().take(60).collect();
        let preview = if preview.len() < content.len() {
            format!("{}...", preview.trim())
        } else {
            preview
        };

        assert_eq!(
            preview, content,
            "Short orphaned completion previews should remain unchanged"
        );
    }

    #[test]
    fn test_orphaned_completion_count_increments_cached_count() {
        let chat_id = ChatId::new();
        let mut message_counts = std::collections::HashMap::new();
        message_counts.insert(chat_id, 5usize);

        let count = message_counts.get(&chat_id).copied().unwrap_or(0);
        message_counts.insert(chat_id, count + 1);

        assert_eq!(
            message_counts.get(&chat_id).copied(),
            Some(6usize),
            "Orphaned completion should increment existing cached message count"
        );
    }

    #[test]
    fn test_orphaned_completion_count_starts_from_zero_when_missing() {
        let chat_id = ChatId::new();
        let mut message_counts = std::collections::HashMap::new();

        let count = message_counts.get(&chat_id).copied().unwrap_or(0);
        message_counts.insert(chat_id, count + 1);

        assert_eq!(
            message_counts.get(&chat_id).copied(),
            Some(1usize),
            "Missing cached message count should initialize to one after orphaned completion"
        );
    }

    #[test]
    fn test_streaming_ui_update_guard_rejects_when_selected_chat_differs() {
        let chat_id = ChatId::new();
        let different_selected_chat = ChatId::new();
        let generation = 10_u64;
        let streaming_generation = 10_u64;
        let streaming_chat_id = Some(chat_id);
        let selected_chat_id = Some(different_selected_chat);

        let should_ignore_update = streaming_generation != generation
            || streaming_chat_id != Some(chat_id)
            || selected_chat_id != Some(chat_id);

        assert!(
            should_ignore_update,
            "Streaming updates must be ignored when user has switched to another selected chat"
        );
    }

    #[test]
    fn test_streaming_ui_update_guard_allows_matching_selected_chat() {
        let chat_id = ChatId::new();
        let generation = 42_u64;
        let streaming_generation = 42_u64;
        let streaming_chat_id = Some(chat_id);
        let selected_chat_id = Some(chat_id);

        let should_ignore_update = streaming_generation != generation
            || streaming_chat_id != Some(chat_id)
            || selected_chat_id != Some(chat_id);

        assert!(
            !should_ignore_update,
            "Streaming updates should continue only while generation, streaming chat, and selected chat all match"
        );
    }

    #[test]
    fn test_chat_switch_generation_bump_makes_existing_streaming_update_stale() {
        let chat_id = ChatId::new();
        let stream_generation = 7_u64;
        let generation_after_chat_switch = stream_generation.wrapping_add(1);
        let streaming_chat_id = Some(chat_id);

        let should_ignore_update =
            generation_after_chat_switch != stream_generation || streaming_chat_id != Some(chat_id);

        assert!(
            should_ignore_update,
            "Bumping generation on chat switch must invalidate prior streaming poll-loop updates"
        );
    }

    #[test]
    fn test_edit_submit_path_is_not_entered_when_submission_input_is_invalid() {
        let editing_message_id = Some("editing-id");
        let content = "   ";
        let has_pending_image = false;

        let should_delegate_to_edit_submit =
            if !ai_window_can_submit_message(content, has_pending_image, false) {
                false
            } else {
                editing_message_id.is_some()
            };

        assert!(
            !should_delegate_to_edit_submit,
            "Empty edits must be treated as no-op and should not enter edit-submit truncation flow"
        );
    }
}
