use super::*;

#[derive(Debug)]
struct SidebarMessageMetadata {
    previews: std::collections::HashMap<ChatId, String>,
    counts: std::collections::HashMap<ChatId, usize>,
}

#[derive(Debug)]
struct SelectedChatInitData {
    chat_id: ChatId,
    messages: Vec<Message>,
    image_requests: Vec<super::images::ImageCacheRequest>,
}

impl AiApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self::new_with_mode(AiWindowMode::Full, window, cx)
    }

    pub fn new_with_mode(
        window_mode: AiWindowMode,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let constructor_start = std::time::Instant::now();
        tracing::info!(
            window_mode = ?window_mode,
            "BEACHBALL TRACE: AiApp::new_with_mode START"
        );

        // Initialize storage
        if let Err(e) = storage::init_ai_db() {
            tracing::error!(error = %e, "Failed to initialize AI database");
        }

        tracing::info!(
            elapsed_ms = constructor_start.elapsed().as_millis(),
            "BEACHBALL TRACE: init_ai_db done"
        );

        // Load chats from storage
        let chats = storage::get_all_chats().unwrap_or_default();

        tracing::info!(
            elapsed_ms = constructor_start.elapsed().as_millis(),
            chat_count = chats.len(),
            "BEACHBALL TRACE: get_all_chats done"
        );
        let selected_chat_id = chats.first().map(|c| c.id);
        let message_previews = std::collections::HashMap::new();
        let message_counts = std::collections::HashMap::new();

        // Initialize provider registry from environment with config
        let config = crate::config::load_config();
        let provider_registry = ProviderRegistry::from_environment_with_config(Some(&config));
        let available_models = provider_registry.get_all_models();

        // Select default model (prefer Claude Haiku 4.5, then 3.5 Haiku, then Sonnet, then GPT-4o)
        let selected_model = available_models
            .iter()
            .find(|m| m.id.contains("haiku-4-5"))
            .or_else(|| {
                available_models
                    .iter()
                    .find(|m| m.id.contains("claude-3-5-haiku"))
            })
            .or_else(|| {
                available_models
                    .iter()
                    .find(|m| m.id.contains("claude-3-5-sonnet"))
            })
            .or_else(|| available_models.iter().find(|m| m.id == "gpt-4o"))
            .or_else(|| available_models.first())
            .cloned();

        info!(
            providers = provider_registry.provider_ids().len(),
            models = available_models.len(),
            selected = selected_model
                .as_ref()
                .map(|m| m.display_name.as_str())
                .unwrap_or("none"),
            "AI providers initialized"
        );

        // Create input states
        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Ask anything...")
                .auto_grow(1, 6)
                .submit_on_enter(true)
        });

        let search_state = cx.new(|cx| InputState::new(window, cx).placeholder("Search chats..."));

        let api_key_input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Enter your Vercel API key...")
                .masked(true)
        });

        let focus_handle = cx.focus_handle();

        // Subscribe to input changes and Enter key
        let input_sub = cx.subscribe_in(&input_state, window, {
            move |this, _, ev: &InputEvent, window, cx| match ev {
                InputEvent::Change => this.on_input_change(cx),
                // Plain Enter → submit; Shift+Enter (secondary) inserted a newline already
                InputEvent::PressEnter { secondary: false } => {
                    // Intercept slash commands before normal submission
                    if !this.try_handle_slash_command(window, cx) {
                        this.submit_message(window, cx);
                    }
                }
                InputEvent::PressEnter { secondary: true } => {
                    // Newline was inserted by the Input component; just notify for resize
                    cx.notify();
                }
                _ => {}
            }
        });

        // Subscribe to search changes
        let search_sub = cx.subscribe_in(&search_state, window, {
            move |this, _, ev: &InputEvent, _window, cx| {
                if matches!(ev, InputEvent::Change) {
                    this.on_search_change(cx);
                }
            }
        });

        // Subscribe to API key input changes (Enter submits the key)
        let api_key_sub = cx.subscribe_in(&api_key_input_state, window, {
            move |this, _, ev: &InputEvent, window, cx| {
                if matches!(ev, InputEvent::PressEnter { .. }) {
                    this.submit_api_key(window, cx);
                }
            }
        });

        // Rename input for sidebar chat rename
        let rename_input_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("Chat name..."));
        let rename_sub = cx.subscribe_in(&rename_input_state, window, {
            move |this: &mut Self, _, ev: &InputEvent, window, cx| {
                if let InputEvent::PressEnter { .. } = ev {
                    this.commit_rename(window, cx)
                }
            }
        });

        let current_messages = Vec::new();
        let image_cache = std::collections::HashMap::new();
        let app_weak = cx.entity().downgrade();

        // Publish initial active chat ID for SDK handlers
        publish_active_chat_id(selected_chat_id.as_ref());

        tracing::info!(
            elapsed_ms = constructor_start.elapsed().as_millis(),
            "BEACHBALL TRACE: AiApp struct about to be built"
        );
        info!(chat_count = chats.len(), "AI app initialized");

        // Pre-compute box shadows from theme (avoid reloading on every render)
        let cached_box_shadows = Self::compute_box_shadows();

        // Compute last used settings before moving chats and available_models
        let last_used_settings = Self::compute_last_used_settings(&chats, &available_models);

        let initial_msg_count = 0;
        let initial_sidebar_item_count = build_sidebar_rows_for_chats(&chats).len();

        // Mirror the window mode in the global atomic so close_ai_window() can
        // determine the correct WindowRole without accessing the entity.
        super::types::AI_CURRENT_WINDOW_MODE
            .store(window_mode.to_u8(), std::sync::atomic::Ordering::SeqCst);

        let app = Self {
            window_mode,
            chats,
            selected_chat_id,
            message_previews,
            message_counts,
            input_state,
            search_state,
            search_query: String::new(),
            search_generation: 0,
            search_snippets: std::collections::HashMap::new(),
            search_matched_title: std::collections::HashMap::new(),
            sidebar_collapsed: false,
            showing_mini_history_overlay: false,
            provider_registry,
            available_models,
            selected_model,
            focus_handle,
            _subscriptions: vec![input_sub, search_sub, api_key_sub, rename_sub],
            // Streaming state
            is_streaming: false,
            streaming_content: String::new(),
            streaming_chat_id: None,
            streaming_generation: 0,
            streaming_cancel: None,
            suppressed_orphan_sessions: std::collections::HashSet::new(),
            current_messages,
            messages_list_state: ListState::new(
                initial_msg_count,
                ListAlignment::Bottom,
                px(1024.),
            ),
            sidebar_list_state: ListState::new(
                initial_sidebar_item_count,
                ListAlignment::Top,
                px(512.),
            ),
            cached_box_shadows,
            needs_focus_input: false,
            needs_command_bar_focus: false,
            last_persisted_bounds: None,
            last_bounds_save: std::time::Instant::now(),
            theme_rev_seen: crate::theme::service::theme_revision(),
            pending_image: None,
            image_cache,
            setup_copied_at: None,
            claude_code_setup_feedback: None,
            showing_api_key_input: false,
            setup_button_focus_index: 0,
            api_key_input_state,
            // Command bar state (uses the unified CommandBar component)
            command_bar: CommandBar::new(
                get_ai_command_bar_actions(),
                CommandBarConfig::ai_style(),
                std::sync::Arc::new(theme::get_cached_theme()),
            ),
            // New chat dropdown (Raycast-style, positioned at top-right)
            new_chat_command_bar: CommandBar::new(
                Vec::new(),                   // Actions will be set dynamically when opened
                CommandBarConfig::ai_style(), // Same style as Cmd+K (search at top, headers)
                std::sync::Arc::new(theme::get_cached_theme()),
            ),
            // Presets state
            showing_presets_dropdown: false,
            presets: AiPreset::load_all_presets(),
            presets_selected_index: 0,
            last_used_settings,
            // Context picker state
            context_picker: None,
            // Attachments state
            pending_context_parts: Vec::new(),
            context_preview_index: None,
            // Mouse cursor state
            mouse_cursor_hidden: false,
            input_mode: InputMode::Mouse,
            // Copy feedback state
            copied_message_id: None,
            copied_at: None,
            streaming_started_at: None,
            // Smart auto-scroll state
            user_has_scrolled_up: false,
            // Streaming completion feedback
            last_streaming_duration: None,
            last_streaming_completed_at: None,
            // UX enhancements
            streaming_error: None,
            chat_drafts: std::collections::HashMap::new(),
            editing_message_id: None,
            renaming_chat_id: None,
            pending_delete_chat_id: None,
            rename_input_state,
            // UX Batch 5 state
            showing_shortcuts_overlay: false,
            collapsed_messages: std::collections::HashSet::new(),
            expanded_messages: std::collections::HashSet::new(),
            export_copied_at: None,
            chat_transcript_copied_at: None,
            search_debounce_task: None,
            last_prepared_message_receipt: None,
            last_preflight_audit: None,
            last_context_receipt: None,
            show_context_inspector: false,
            show_context_drawer: false,
            context_preflight: super::context_preflight::ContextPreflightState::default(),
        };

        let sidebar_chat_ids: Vec<ChatId> = app.chats.iter().map(|chat| chat.id).collect();
        Self::spawn_sidebar_metadata_init(app_weak.clone(), sidebar_chat_ids, cx);
        Self::spawn_selected_chat_init(app_weak, selected_chat_id, cx);

        tracing::info!(
            elapsed_ms = constructor_start.elapsed().as_millis(),
            "BEACHBALL TRACE: AiApp::new_with_mode END (deferred loads spawned)"
        );

        app
    }

    /// Debounce interval for bounds persistence (in milliseconds)
    pub(super) const BOUNDS_DEBOUNCE_MS: u64 = 250;

    fn truncate_sidebar_preview(content: &str) -> String {
        let preview: String = content.chars().take(60).collect();
        if preview.len() < content.len() {
            format!("{}...", preview.trim())
        } else {
            preview
        }
    }

    fn load_sidebar_message_metadata(chat_ids: &[ChatId]) -> SidebarMessageMetadata {
        let mut previews = std::collections::HashMap::new();
        let mut counts = std::collections::HashMap::new();

        for chat_id in chat_ids {
            match storage::get_chat_messages(chat_id) {
                Ok(messages) => {
                    counts.insert(*chat_id, messages.len());
                    if let Some(last_message) = messages.last() {
                        previews.insert(
                            *chat_id,
                            Self::truncate_sidebar_preview(&last_message.content),
                        );
                    }
                }
                Err(error) => {
                    tracing::warn!(
                        chat_id = %chat_id,
                        error = %error,
                        "Failed to load sidebar message metadata during AI init"
                    );
                }
            }
        }

        SidebarMessageMetadata { previews, counts }
    }

    fn load_selected_chat_init_data(chat_id: ChatId) -> Option<SelectedChatInitData> {
        match storage::get_chat_messages(&chat_id) {
            Ok(messages) => {
                let image_requests = Self::collect_message_image_payloads(&messages);
                Some(SelectedChatInitData {
                    chat_id,
                    messages,
                    image_requests,
                })
            }
            Err(error) => {
                tracing::warn!(
                    chat_id = %chat_id,
                    error = %error,
                    "Failed to load selected chat messages during AI init"
                );
                None
            }
        }
    }

    fn spawn_sidebar_metadata_init(
        app_weak: gpui::WeakEntity<Self>,
        chat_ids: Vec<ChatId>,
        cx: &mut Context<Self>,
    ) {
        if chat_ids.is_empty() {
            return;
        }

        tracing::info!(
            chat_count = chat_ids.len(),
            "Scheduling deferred sidebar metadata load during AI init"
        );

        cx.spawn(async move |_this, cx| {
            let metadata = cx
                .background_executor()
                .spawn(async move { Self::load_sidebar_message_metadata(&chat_ids) })
                .await;
            let SidebarMessageMetadata { previews, counts } = metadata;

            cx.update(|cx| {
                if let Some(app) = app_weak.upgrade() {
                    app.update(cx, |this, cx| {
                        tracing::info!(
                            preview_count = previews.len(),
                            count_entries = counts.len(),
                            "Applying deferred sidebar metadata during AI init"
                        );

                        this.message_previews.extend(previews);
                        this.message_counts.extend(counts);
                        cx.notify();
                    });
                }
            });
        })
        .detach();
    }

    fn spawn_selected_chat_init(
        app_weak: gpui::WeakEntity<Self>,
        selected_chat_id: Option<ChatId>,
        cx: &mut Context<Self>,
    ) {
        let Some(chat_id) = selected_chat_id else {
            return;
        };

        tracing::info!(
            chat_id = %chat_id,
            "Scheduling deferred selected chat load during AI init"
        );

        cx.spawn(async move |_this, cx| {
            let init_data = cx
                .background_executor()
                .spawn(async move { Self::load_selected_chat_init_data(chat_id) })
                .await;

            cx.update(|cx| {
                if let Some(app) = app_weak.upgrade() {
                    app.update(cx, |this, cx| {
                        let Some(SelectedChatInitData {
                            chat_id,
                            messages,
                            image_requests,
                        }) = init_data
                        else {
                            return;
                        };

                        if this.selected_chat_id != Some(chat_id) {
                            tracing::debug!(
                                selected_chat_id = ?this.selected_chat_id,
                                loaded_chat_id = %chat_id,
                                "Discarding stale selected chat init data"
                            );
                            return;
                        }

                        if !this.current_messages.is_empty() {
                            tracing::debug!(
                                selected_chat_id = %chat_id,
                                existing_message_count = this.current_messages.len(),
                                "Skipping deferred selected chat init because messages are already loaded"
                            );
                            return;
                        }

                        tracing::info!(
                            selected_chat_id = %chat_id,
                            message_count = messages.len(),
                            image_request_count = image_requests.len(),
                            "Applying deferred selected chat load during AI init"
                        );

                        this.current_messages = messages;
                        this.sync_messages_list_and_scroll_to_bottom();
                        if !image_requests.is_empty() {
                            this.defer_cache_message_images(image_requests, cx);
                        }
                        cx.notify();
                    });
                }
            });
        })
        .detach();
    }
}

#[cfg(test)]
mod init_tests {
    use super::*;

    #[test]
    fn test_truncate_sidebar_preview_adds_ellipsis_only_when_content_exceeds_limit() {
        let short = "short sidebar preview";
        assert_eq!(AiApp::truncate_sidebar_preview(short), short);

        let long = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let truncated = AiApp::truncate_sidebar_preview(long);
        assert_eq!(
            truncated,
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ01234567..."
        );
        assert!(truncated.ends_with("..."));
        assert_eq!(truncated.chars().count(), 63);
    }
}
