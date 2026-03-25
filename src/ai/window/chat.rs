use super::window_api::get_pending_chat;
use super::*;
use crate::ai::model::ImageAttachment;

/// Intermediate result from the shared outbound message compiler.
///
/// Both `handle_start_chat` (SDK path) and `submit_message` (composer path)
/// produce this struct, ensuring directive parsing, context merging, receipt
/// generation, and display-text derivation happen in exactly one place.
#[derive(Debug, Clone)]
pub(super) struct OutboundUserMessagePreparation {
    pub(super) receipt: crate::ai::message_parts::PreparedMessageReceipt,
    pub(super) authored_content: String,
    pub(super) has_context_parts: bool,
}

/// Choose the correct saved user content based on the compiler decision.
///
/// For `Ready` and `Partial`, persists `receipt.final_user_content` (compiled
/// with resolved context blocks). For `Blocked`, persists the authored content
/// (directives stripped but no context injected) so saved turns never contain
/// raw `@file` / `@context` directive lines.
fn start_chat_saved_user_content(
    decision: &crate::ai::message_parts::PreparedMessageDecision,
    receipt: &crate::ai::message_parts::PreparedMessageReceipt,
    authored_content: &str,
) -> String {
    match decision {
        crate::ai::message_parts::PreparedMessageDecision::Ready
        | crate::ai::message_parts::PreparedMessageDecision::Partial => {
            tracing::info!(
                checkpoint = "start_chat_saved_content",
                source = "final_user_content",
                len = receipt.final_user_content.len(),
                "persisting compiled content"
            );
            receipt.final_user_content.clone()
        }
        crate::ai::message_parts::PreparedMessageDecision::Blocked => {
            tracing::info!(
                checkpoint = "start_chat_saved_content",
                source = "authored_content",
                len = authored_content.len(),
                "persisting authored content (blocked resolution)"
            );
            authored_content.to_string()
        }
    }
}

impl AiApp {
    /// Shared outbound message compiler used by both send paths.
    ///
    /// 1. Parses `@context` / `@file` directives from `raw_content`.
    /// 2. Merges directive-derived parts with `explicit_parts`.
    /// 3. Runs the receipt pipeline (`prepare_user_message_with_receipt`).
    /// 4. Emits the `ai_context_mentions_compiled` structured log checkpoint.
    /// 5. Stores the receipt in `self.last_prepared_message_receipt`.
    pub(super) fn prepare_outbound_user_message(
        &mut self,
        raw_content: &str,
        explicit_parts: &[crate::ai::message_parts::AiContextPart],
    ) -> OutboundUserMessagePreparation {
        let parsed_mentions = crate::ai::context_mentions::parse_context_mentions(raw_content);
        let has_any_parts = !explicit_parts.is_empty() || parsed_mentions.has_parts();

        tracing::info!(
            target: "ai",
            raw_len = raw_content.len(),
            authored_len = parsed_mentions.cleaned_content.len(),
            explicit_parts = explicit_parts.len(),
            directive_parts = parsed_mentions.parts.len(),
            "ai_context_mentions_compiled"
        );

        let receipt = if has_any_parts {
            let scripts = crate::scripts::read_scripts();
            let scriptlets = crate::scripts::load_scriptlets();

            crate::ai::message_parts::prepare_user_message_from_sources_with_receipt(
                &parsed_mentions.cleaned_content,
                &parsed_mentions.parts,
                explicit_parts,
                &scripts,
                &scriptlets,
            )
        } else {
            crate::ai::message_parts::prepare_user_message_with_receipt(
                &parsed_mentions.cleaned_content,
                &[],
                &[],
                &[],
            )
        };

        let has_context_parts = receipt
            .assembly
            .as_ref()
            .map(|a| a.merged_count > 0)
            .unwrap_or(false);

        self.last_prepared_message_receipt = Some(receipt.clone());

        OutboundUserMessagePreparation {
            receipt,
            authored_content: parsed_mentions.cleaned_content,
            has_context_parts,
        }
    }
    fn model_matches_chat_identity(model: &ModelInfo, chat: &Chat) -> bool {
        model.id == chat.model_id && model.provider == chat.provider
    }

    pub(crate) fn resolve_start_chat_metadata(
        available_models: &[ModelInfo],
        selected_model: Option<&ModelInfo>,
        requested_model_id: Option<&str>,
        requested_provider: Option<&str>,
    ) -> StartChatResolvedMetadata {
        if let Some(requested_model_id) = requested_model_id {
            let matched_model = requested_provider
                .and_then(|requested_provider| {
                    available_models.iter().find(|model| {
                        model.id == requested_model_id && model.provider == requested_provider
                    })
                })
                .or_else(|| {
                    available_models
                        .iter()
                        .find(|model| model.id == requested_model_id)
                });

            return matched_model
                .map(|model| StartChatResolvedMetadata {
                    model_id: model.id.clone(),
                    provider: model.provider.clone(),
                })
                .unwrap_or_else(|| StartChatResolvedMetadata {
                    model_id: requested_model_id.to_string(),
                    provider: requested_provider.unwrap_or("anthropic").to_string(),
                });
        }

        selected_model
            .map(|model| StartChatResolvedMetadata {
                model_id: model.id.clone(),
                provider: model.provider.clone(),
            })
            .unwrap_or_else(|| StartChatResolvedMetadata {
                model_id: "claude-3-5-sonnet-20241022".to_string(),
                provider: "anthropic".to_string(),
            })
    }

    pub(super) fn provider_unavailable_error_message(model_id: &str, provider: &str) -> String {
        format!(
            "Model '{model_id}' uses provider '{provider}', but that provider is unavailable. Configure the '{provider}' API key or pick a different model."
        )
    }

    pub(super) fn clear_streaming_state_with_error(
        &mut self,
        message: impl Into<String>,
        cx: &mut Context<Self>,
    ) {
        let message = message.into();

        if let Some(cancelled) = self.streaming_cancel.take() {
            cancelled.store(true, std::sync::atomic::Ordering::SeqCst);
        }

        self.is_streaming = false;
        self.streaming_content.clear();
        self.streaming_generation = self.streaming_generation.wrapping_add(1);
        self.streaming_chat_id = None;
        self.streaming_started_at = None;
        self.streaming_error = Some(message);
        publish_streaming_state(AiStreamingSnapshot::default());
        cx.notify();
    }

    pub(super) fn initialize_with_pending_chat(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Take the pending messages from the global state
        let pending_messages = get_pending_chat()
            .lock()
            .ok()
            .and_then(|mut pending| pending.take());

        let messages = match pending_messages {
            Some(msgs) if !msgs.is_empty() => msgs,
            _ => {
                tracing::debug!(target: "ai", "No pending messages to initialize chat with");
                return;
            }
        };

        tracing::info!(target: "ai", message_count = messages.len(), "Initializing chat with pending messages");

        // Get model and provider from selected model, or use defaults
        let (model_id, provider) = self
            .selected_model
            .as_ref()
            .map(|m| (m.id.clone(), m.provider.clone()))
            .unwrap_or_else(|| {
                (
                    "claude-3-5-sonnet-20241022".to_string(),
                    "anthropic".to_string(),
                )
            });

        // Create a new chat with the ChatPrompt source
        let mut chat = Chat::new(&model_id, &provider);
        chat.source = ChatSource::ChatPrompt;
        let chat_id = chat.id;

        // Generate title from the first user message (if any)
        if let Some(msg) = messages.iter().find(|m| m.role == MessageRole::User) {
            let title = Chat::generate_title_from_content(&msg.content);
            chat.set_title(&title);
        }

        // Save chat to storage
        if let Err(e) = storage::create_chat(&chat) {
            tracing::error!(error = %e, "Failed to create chat for transferred conversation");
            return;
        }

        // Save all messages to storage and build the current_messages list
        let mut saved_messages = Vec::new();
        for msg in messages {
            let mut message = Message::new(chat_id, msg.role, msg.content);
            // Attach image if present (transferred from inline ChatPrompt)
            if let Some(image_data) = msg.image_base64 {
                message.images.push(ImageAttachment::png(image_data));
            }
            if let Err(e) = storage::save_message(&message) {
                tracing::error!(error = %e, "Failed to save message in transferred conversation");
                continue;
            }
            saved_messages.push(message);
        }

        // Update message preview and count with the last message
        if let Some(last_msg) = saved_messages.last() {
            let preview: String = last_msg.content.chars().take(60).collect();
            let preview = if preview.len() < last_msg.content.len() {
                format!("{}...", preview.trim())
            } else {
                preview
            };
            self.message_previews.insert(chat_id, preview);
        }
        self.message_counts.insert(chat_id, saved_messages.len());

        // Add chat to the list and select it
        self.chats.insert(0, chat);
        self.selected_chat_id = Some(chat_id);
        publish_active_chat_id(Some(&chat_id));
        self.defer_cache_message_images(Self::collect_message_image_payloads(&saved_messages), cx);
        self.current_messages = saved_messages;

        // Force scroll to bottom when initializing with a transferred conversation
        self.force_scroll_to_bottom();

        info!(
            chat_id = %chat_id,
            message_count = self.current_messages.len(),
            "Chat initialized with transferred conversation"
        );

        cx.notify();
    }

    /// Start a new conversation, fully resetting all per-conversation transient state.
    ///
    /// If a response is actively streaming, it is cancelled before reset.
    /// The previous conversation (if any) is preserved in the sidebar history.
    pub(super) fn new_conversation(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<ChatId> {
        // Cancel any active stream before switching
        if self.is_streaming {
            info!(
                streaming_chat_id = ?self.streaming_chat_id,
                "Cancelling active stream for new conversation"
            );
            self.stop_streaming(cx);
        }

        // Clear per-conversation transient state that select_chat does not cover
        let had_image = self.pending_image.is_some();
        let context_parts_count = self.pending_context_parts.len();
        self.pending_image = None;
        self.pending_context_parts.clear();
        self.clear_context_preflight(cx);
        if had_image || context_parts_count > 0 {
            tracing::info!(
                had_pending_image = had_image,
                cleared_context_parts = context_parts_count,
                "chat_switch_cleared_context_parts"
            );
        }
        self.context_picker = None;
        self.collapsed_messages.clear();
        self.expanded_messages.clear();
        self.copied_message_id = None;
        self.copied_at = None;
        self.last_streaming_duration = None;
        self.last_streaming_completed_at = None;
        self.streaming_error = None;
        self.editing_message_id = None;
        self.last_prepared_message_receipt = None;
        self.last_preflight_audit = None;
        self.last_context_receipt = None;
        self.show_context_inspector = false;
        self.show_context_drawer = false;
        self.showing_mini_history_overlay = false;

        let chat_id = self.create_chat(window, cx);

        info!(chat_id = ?chat_id, "New conversation started with full state reset");
        chat_id
    }

    /// Record a structured, user-facing error with a machine-readable code.
    ///
    /// Writes an `ai_user_error` log line, sets `streaming_error` so the UI
    /// surfaces the message, and notifies the view.
    pub(super) fn set_user_error(
        &mut self,
        code: super::types::AiErrorCode,
        message: impl Into<String>,
        suggested_action: &'static str,
        cx: &mut Context<Self>,
    ) {
        let err = super::types::AiUserFacingError {
            code,
            message: message.into(),
            suggested_action,
        };

        tracing::error!(
            target: "ai",
            code = ?err.code,
            suggested_action = err.suggested_action,
            message = %err.message,
            "ai_user_error"
        );

        self.streaming_error = Some(err.to_display_string());
        cx.notify();
    }

    /// Create a new chat
    pub(super) fn create_chat(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<ChatId> {
        // Get model and provider from selected model, or use defaults
        let (model_id, provider) = self
            .selected_model
            .as_ref()
            .map(|m| (m.id.clone(), m.provider.clone()))
            .unwrap_or_else(|| {
                (
                    "claude-3-5-sonnet-20241022".to_string(),
                    "anthropic".to_string(),
                )
            });

        // Create a new chat with selected model
        let chat = Chat::new(&model_id, &provider);
        let id = chat.id;

        // Save to storage
        if let Err(e) = storage::create_chat(&chat) {
            self.set_user_error(
                super::types::AiErrorCode::CreateChatFailed,
                format!("Could not create a new conversation: {e}"),
                "retry Cmd+N or inspect storage/log output",
                cx,
            );
            return None;
        }

        // Add to cache and select it
        self.chats.insert(0, chat);
        self.select_chat(id, window, cx);

        info!(chat_id = %id, model = model_id, "New chat created");
        Some(id)
    }

    /// Select a chat
    pub(super) fn select_chat(&mut self, id: ChatId, window: &mut Window, cx: &mut Context<Self>) {
        self.select_chat_internal(id, true, window, cx);
    }

    /// Select a chat but keep the mini history overlay open.
    ///
    /// Used by arrow-key navigation inside the overlay so the user can
    /// preview different chats without dismissing the list.
    pub(super) fn select_chat_preserving_overlay(
        &mut self,
        id: ChatId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_chat_internal(id, false, window, cx);
    }

    /// Internal select-chat implementation.
    ///
    /// When `dismiss_mini_overlay` is true the mini history overlay is closed
    /// (normal select path). When false the overlay stays open so the user can
    /// continue browsing (arrow-key preview path).
    fn select_chat_internal(
        &mut self,
        id: ChatId,
        dismiss_mini_overlay: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Save draft for outgoing chat
        self.save_draft(cx);

        // Clear any pending delete confirmation
        self.pending_delete_chat_id = None;

        self.selected_chat_id = Some(id);
        publish_active_chat_id(Some(&id));

        // Load messages for this chat
        let mut provider_error_message: Option<String> = None;
        let mut storage_error_message: Option<String> = None;
        match storage::get_chat_messages(&id) {
            Ok(messages) => {
                self.current_messages = messages;
            }
            Err(error) => {
                let message = format!("Failed to load chat messages for chat '{id}': {error}");
                tracing::error!(
                    chat_id = %id,
                    error = %error,
                    "Failed to load chat messages during chat switch"
                );
                self.current_messages = Vec::new();
                storage_error_message = Some(message);
            }
        }
        self.defer_cache_message_images(
            Self::collect_message_image_payloads(&self.current_messages),
            cx,
        );

        // Sync selected_model with the chat's stored model (BYOK per chat)
        if let Some(chat) = self.chats.iter().find(|c| c.id == id) {
            // Find the model in available_models that matches the chat's provider+model_id
            self.selected_model = self
                .available_models
                .iter()
                .find(|m| Self::model_matches_chat_identity(m, chat))
                .cloned();

            if self.selected_model.is_none() && !chat.model_id.is_empty() {
                // Chat has a model_id but it's not in our available models
                // (provider may not be configured). Log for debugging.
                tracing::debug!(
                    chat_id = %id,
                    model_id = %chat.model_id,
                    provider = %chat.provider,
                    "Chat's model not found in available models (provider may not be configured)"
                );

                if self
                    .provider_registry
                    .get_provider(&chat.provider)
                    .is_none()
                {
                    let message =
                        Self::provider_unavailable_error_message(&chat.model_id, &chat.provider);
                    tracing::error!(
                        chat_id = %id,
                        model_id = %chat.model_id,
                        provider = %chat.provider,
                        error = %message,
                        "Provider missing for selected chat model"
                    );
                    provider_error_message = Some(message);
                }
            }
        }

        // Force scroll to bottom when switching chats (always scroll)
        self.force_scroll_to_bottom();

        // Clear streaming state for display purposes, but don't clear streaming_chat_id
        // The streaming task may still be running for the previous chat - it will be
        // ignored via the generation guard when it tries to update
        self.is_streaming = false;
        self.streaming_content.clear();
        self.streaming_generation = self.streaming_generation.wrapping_add(1);
        // Note: streaming_chat_id is intentionally NOT cleared here
        // This allows the background streaming to complete and save to DB correctly
        // while UI shows the newly selected chat's messages
        publish_streaming_state(AiStreamingSnapshot::default());

        // Reset UX state for new chat — only dismiss overlay when caller requests it
        if dismiss_mini_overlay {
            self.showing_mini_history_overlay = false;
        }
        self.editing_message_id = None;
        if let Some(message) = provider_error_message {
            self.clear_streaming_state_with_error(message, cx);
        } else if let Some(message) = storage_error_message {
            self.streaming_error = Some(message);
        } else {
            self.streaming_error = None;
        }

        // Restore draft for incoming chat
        self.restore_draft(window, cx);

        // Update placeholder based on chat context
        self.update_input_placeholder(window, cx);

        cx.notify();
    }

    /// Update input placeholder text based on current context.
    /// Shows model name when in an active chat, generic text otherwise.
    pub(super) fn update_input_placeholder(&self, window: &mut Window, cx: &mut Context<Self>) {
        let placeholder = if !self.current_messages.is_empty() {
            if let Some(ref model) = self.selected_model {
                format!("Reply to {}...", model.display_name)
            } else {
                "Type a reply...".to_string()
            }
        } else if let Some(ref model) = self.selected_model {
            format!("Ask {}...", model.display_name)
        } else {
            "Ask anything...".to_string()
        };
        self.input_state.update(cx, |state, cx| {
            state.set_placeholder(placeholder, window, cx);
        });
    }

    /// Delete the currently selected chat (soft delete)
    pub(super) fn delete_selected_chat(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_chat_id {
            self.delete_chat_by_id(id, cx);
        }
    }

    /// Handle an SDK-initiated aiStartChat command.
    ///
    /// Creates a new chat with the pre-generated ChatId, saves the user message
    /// (with optional image), and optionally triggers AI streaming.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn handle_start_chat(
        &mut self,
        chat_id: ChatId,
        message: String,
        parts: Vec<crate::ai::message_parts::AiContextPart>,
        image: Option<String>,
        system_prompt: Option<String>,
        model_id: Option<String>,
        provider: Option<String>,
        on_created: Option<std::sync::Arc<dyn Fn(String, String) + Send + Sync + 'static>>,
        submit: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let resolved = Self::resolve_start_chat_metadata(
            &self.available_models,
            self.selected_model.as_ref(),
            model_id.as_deref(),
            provider.as_deref(),
        );
        let resolved_model_id = resolved.model_id.clone();
        let resolved_provider = resolved.provider.clone();

        // --- Shared outbound compiler: parse directives, merge, resolve ---
        let OutboundUserMessagePreparation {
            receipt,
            authored_content,
            has_context_parts: has_parts,
        } = self.prepare_outbound_user_message(&message, &parts);

        let decision = receipt.decision.clone();
        self.last_context_receipt = if has_parts {
            Some(receipt.context.clone())
        } else {
            None
        };

        let final_user_content =
            start_chat_saved_user_content(&decision, &receipt, &authored_content);

        match decision {
            crate::ai::message_parts::PreparedMessageDecision::Ready => {
                self.streaming_error = None;
            }
            crate::ai::message_parts::PreparedMessageDecision::Partial => {
                tracing::warn!(
                    checkpoint = "resolution_partial",
                    attempted = receipt.context.attempted,
                    resolved = receipt.context.resolved,
                    outcomes = ?receipt.outcomes,
                    failures = ?receipt.context.failures,
                    chat_id = %chat_id,
                    "start_chat_context: partial resolution failure"
                );
                self.streaming_error = receipt.user_error.clone();
            }
            crate::ai::message_parts::PreparedMessageDecision::Blocked => {
                tracing::warn!(
                    checkpoint = "resolution_blocked",
                    attempted = receipt.context.attempted,
                    resolved = receipt.context.resolved,
                    outcomes = ?receipt.outcomes,
                    failures = ?receipt.context.failures,
                    chat_id = %chat_id,
                    "start_chat_context: blocked due to unresolved context"
                );
                self.streaming_error = receipt.user_error.clone();
            }
        }

        if let Some(on_created) = on_created {
            on_created(resolved_model_id.clone(), resolved_provider.clone());
        }

        // Create the chat with the pre-generated ChatId
        let mut chat = Chat::new(&resolved_model_id, &resolved_provider);
        chat.id = chat_id;
        chat.source = ChatSource::Script;

        // Derive title from cleaned authored content, not raw directive lines
        let has_image = image.is_some();
        let display_source =
            ai_window_outbound_display_source(&authored_content, has_image, has_parts);
        let title = Chat::generate_title_from_content(&display_source);
        chat.set_title(&title);

        // Apply system prompt if provided
        if let Some(ref prompt) = system_prompt {
            // Save system prompt as the first message
            let sys_msg = Message::new(chat_id, MessageRole::System, prompt.clone());
            if let Err(e) = storage::save_message(&sys_msg) {
                tracing::error!(error = %e, chat_id = %chat_id, "Failed to save system prompt");
            }
        }

        // Save chat to storage
        if let Err(e) = storage::create_chat(&chat) {
            tracing::error!(error = %e, chat_id = %chat_id, "Failed to create chat for aiStartChat");
            return;
        }

        // Create and save the user message with optional image
        let mut user_message = Message::user(chat_id, &final_user_content);
        if let Some(ref img_base64) = image {
            user_message
                .images
                .push(crate::ai::model::ImageAttachment::png(img_base64.clone()));
        }

        if let Err(e) = storage::save_message(&user_message) {
            tracing::error!(error = %e, chat_id = %chat_id, "Failed to save user message for aiStartChat");
            return;
        }

        // Update in-memory state
        self.chats.insert(0, chat);
        self.selected_chat_id = Some(chat_id);
        publish_active_chat_id(Some(&chat_id));

        // Load messages for display (includes system prompt if any)
        match storage::get_chat_messages(&chat_id) {
            Ok(messages) => {
                self.defer_cache_message_images(
                    Self::collect_message_image_payloads(&messages),
                    cx,
                );
                self.current_messages = messages;
            }
            Err(e) => {
                tracing::error!(error = %e, chat_id = %chat_id, "Failed to load messages after aiStartChat");
                self.current_messages = vec![user_message];
            }
        }

        // Update preview and count caches — derived from cleaned authored content
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

        // Force scroll to bottom
        self.force_scroll_to_bottom();

        // Clear input and update placeholder
        self.clear_composer(window, cx);
        self.update_input_placeholder(window, cx);

        info!(
            chat_id = %chat_id,
            submit = submit,
            has_image = has_image,
            has_parts = has_parts,
            has_system_prompt = system_prompt.is_some(),
            message_len = message.len(),
            authored_content_len = authored_content.len(),
            "ai_sdk.start_chat created"
        );

        if submit {
            self.start_streaming_response(chat_id, cx);
        }

        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_matches_chat_identity_requires_provider_match() {
        let matching_model = ModelInfo::new("shared-model", "Shared", "openai", true, 128_000);
        let wrong_provider_model =
            ModelInfo::new("shared-model", "Shared", "anthropic", true, 128_000);
        let chat = Chat::new("shared-model", "openai");

        assert!(
            AiApp::model_matches_chat_identity(&matching_model, &chat),
            "Model should match when both model_id and provider match the chat"
        );
        assert!(
            !AiApp::model_matches_chat_identity(&wrong_provider_model, &chat),
            "Model should not match when provider differs even if model_id is identical"
        );
    }

    #[test]
    fn test_provider_unavailable_error_message_includes_model_id_and_provider_name() {
        let message =
            AiApp::provider_unavailable_error_message("claude-3-5-sonnet-20241022", "anthropic");

        assert!(
            message.contains("claude-3-5-sonnet-20241022"),
            "Provider unavailability message should include model ID"
        );
        assert!(
            message.contains("anthropic"),
            "Provider unavailability message should include provider name"
        );
    }

    #[test]
    fn test_resolve_start_chat_metadata_prefers_requested_provider_match() {
        let available_models = vec![
            ModelInfo::new("shared-model", "Shared", "openai", true, 128_000),
            ModelInfo::new("shared-model", "Shared", "anthropic", true, 128_000),
        ];
        let resolved = AiApp::resolve_start_chat_metadata(
            &available_models,
            None,
            Some("shared-model"),
            Some("anthropic"),
        );

        assert_eq!(
            resolved,
            StartChatResolvedMetadata {
                model_id: "shared-model".to_string(),
                provider: "anthropic".to_string(),
            },
            "Requested provider should disambiguate shared model IDs"
        );
    }

    #[test]
    fn test_resolve_start_chat_metadata_uses_selected_model_when_request_missing() {
        let available_models = vec![ModelInfo::new("gpt-4o", "GPT-4o", "openai", true, 128_000)];
        let selected_model = available_models.first();
        let resolved =
            AiApp::resolve_start_chat_metadata(&available_models, selected_model, None, None);

        assert_eq!(
            resolved,
            StartChatResolvedMetadata {
                model_id: "gpt-4o".to_string(),
                provider: "openai".to_string(),
            },
            "Missing aiStartChat model_id should fall back to the active selected model"
        );
    }

    fn test_start_chat_receipt(
        decision: crate::ai::message_parts::PreparedMessageDecision,
    ) -> crate::ai::message_parts::PreparedMessageReceipt {
        crate::ai::message_parts::PreparedMessageReceipt {
            schema_version: crate::ai::message_parts::AI_MESSAGE_PREPARATION_SCHEMA_VERSION,
            decision,
            raw_content: "@file /tmp/missing.txt\nExplain this".to_string(),
            final_user_content:
                "<context source=\"kit://selection\">selected</context>\n\nExplain this".to_string(),
            context: crate::ai::message_parts::ContextResolutionReceipt {
                attempted: 1,
                resolved: 0,
                failures: vec![],
                prompt_prefix: String::new(),
            },
            assembly: None,
            outcomes: vec![],
            unresolved_parts: vec![],
            user_error: Some("Failed".to_string()),
        }
    }

    #[test]
    fn test_start_chat_saved_user_content_uses_authored_content_when_blocked() {
        let receipt =
            test_start_chat_receipt(crate::ai::message_parts::PreparedMessageDecision::Blocked);

        let saved = start_chat_saved_user_content(
            &crate::ai::message_parts::PreparedMessageDecision::Blocked,
            &receipt,
            "Explain this",
        );

        assert_eq!(saved, "Explain this");
    }

    #[test]
    fn test_start_chat_saved_user_content_uses_final_user_content_when_partial() {
        let receipt =
            test_start_chat_receipt(crate::ai::message_parts::PreparedMessageDecision::Partial);

        let saved = start_chat_saved_user_content(
            &crate::ai::message_parts::PreparedMessageDecision::Partial,
            &receipt,
            "Explain this",
        );

        assert_eq!(
            saved,
            "<context source=\"kit://selection\">selected</context>\n\nExplain this"
        );
    }

    #[test]
    fn test_start_chat_saved_user_content_uses_final_user_content_when_ready() {
        let receipt =
            test_start_chat_receipt(crate::ai::message_parts::PreparedMessageDecision::Ready);

        let saved = start_chat_saved_user_content(
            &crate::ai::message_parts::PreparedMessageDecision::Ready,
            &receipt,
            "Explain this",
        );

        assert_eq!(
            saved,
            "<context source=\"kit://selection\">selected</context>\n\nExplain this"
        );
    }

    #[test]
    fn test_resolve_start_chat_metadata_preserves_requested_provider_on_lookup_miss() {
        let available_models = vec![ModelInfo::new(
            "claude-3-5-sonnet-20241022",
            "Claude 3.5 Sonnet",
            "anthropic",
            true,
            200_000,
        )];
        let resolved = AiApp::resolve_start_chat_metadata(
            &available_models,
            None,
            Some("custom-model"),
            Some("openai"),
        );

        assert_eq!(
            resolved,
            StartChatResolvedMetadata {
                model_id: "custom-model".to_string(),
                provider: "openai".to_string(),
            },
            "Explicit provider fallback should survive when the model is not in available_models"
        );
    }
}
