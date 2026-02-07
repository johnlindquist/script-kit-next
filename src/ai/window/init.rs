use super::*;

impl AiApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Initialize storage
        if let Err(e) = storage::init_ai_db() {
            tracing::error!(error = %e, "Failed to initialize AI database");
        }

        // Load chats from storage
        let chats = storage::get_all_chats().unwrap_or_default();
        let selected_chat_id = chats.first().map(|c| c.id);

        // Load message previews and counts for each chat
        let mut message_previews = std::collections::HashMap::new();
        let mut message_counts = std::collections::HashMap::new();
        for chat in &chats {
            if let Ok(messages) = storage::get_chat_messages(&chat.id) {
                message_counts.insert(chat.id, messages.len());
                if let Some(last_msg) = messages.last() {
                    // Truncate preview to ~60 chars
                    let preview: String = last_msg.content.chars().take(60).collect();
                    let preview = if preview.len() < last_msg.content.len() {
                        format!("{}...", preview.trim())
                    } else {
                        preview
                    };
                    message_previews.insert(chat.id, preview);
                }
            }
        }

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
        let input_state = cx.new(|cx| InputState::new(window, cx).placeholder("Ask anything..."));

        let search_state = cx.new(|cx| InputState::new(window, cx).placeholder("Search chats..."));

        let api_key_input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Enter your Vercel API key...")
                .masked(true)
        });

        // New chat dropdown search input
        let new_chat_dropdown_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("New chat with..."));

        let focus_handle = cx.focus_handle();

        // Subscribe to input changes and Enter key
        let input_sub = cx.subscribe_in(&input_state, window, {
            move |this, _, ev: &InputEvent, window, cx| match ev {
                InputEvent::Change => this.on_input_change(cx),
                InputEvent::PressEnter { .. } => this.submit_message(window, cx),
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

        // Subscribe to new chat dropdown input changes
        let new_chat_dropdown_sub = cx.subscribe_in(&new_chat_dropdown_input, window, {
            move |this, _, ev: &InputEvent, window, cx| match ev {
                InputEvent::Change => this.on_new_chat_dropdown_filter_change(cx),
                InputEvent::PressEnter { .. } => this.select_from_new_chat_dropdown(window, cx),
                _ => {}
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

        // Load messages for the selected chat
        let current_messages = selected_chat_id
            .and_then(|id| storage::get_chat_messages(&id).ok())
            .unwrap_or_default();

        // Pre-cache any image attachments from loaded messages
        let mut image_cache = std::collections::HashMap::new();
        for msg in &current_messages {
            for attachment in &msg.images {
                let cache_key = Self::image_cache_key(&attachment.data);
                if let std::collections::hash_map::Entry::Vacant(e) = image_cache.entry(cache_key) {
                    use base64::Engine;
                    if let Ok(bytes) =
                        base64::engine::general_purpose::STANDARD.decode(&attachment.data)
                    {
                        if let Ok(render_image) =
                            crate::list_item::decode_png_to_render_image_with_bgra_conversion(
                                &bytes,
                            )
                        {
                            e.insert(render_image);
                        }
                    }
                }
            }
        }

        info!(chat_count = chats.len(), "AI app initialized");

        // Pre-compute box shadows from theme (avoid reloading on every render)
        let cached_box_shadows = Self::compute_box_shadows();

        // Compute last used settings before moving chats and available_models
        let last_used_settings = Self::compute_last_used_settings(&chats, &available_models);

        let initial_msg_count = current_messages.len();
        let initial_sidebar_item_count = build_sidebar_rows_for_chats(&chats).len();

        Self {
            chats,
            selected_chat_id,
            message_previews,
            message_counts,
            input_state,
            search_state,
            search_query: String::new(),
            sidebar_collapsed: false,
            provider_registry,
            available_models,
            selected_model,
            focus_handle,
            _subscriptions: vec![
                input_sub,
                search_sub,
                api_key_sub,
                new_chat_dropdown_sub,
                rename_sub,
            ],
            // Streaming state
            is_streaming: false,
            streaming_content: String::new(),
            streaming_chat_id: None,
            streaming_generation: 0,
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
                std::sync::Arc::new(theme::load_theme()),
            ),
            // New chat dropdown (Raycast-style, positioned at top-right)
            new_chat_command_bar: CommandBar::new(
                Vec::new(),                   // Actions will be set dynamically when opened
                CommandBarConfig::ai_style(), // Same style as Cmd+K (search at top, headers)
                std::sync::Arc::new(theme::load_theme()),
            ),
            // Presets state
            showing_presets_dropdown: false,
            presets: AiPreset::default_presets(),
            presets_selected_index: 0,
            // New chat dropdown state (Raycast-style)
            showing_new_chat_dropdown: false,
            new_chat_dropdown_filter: String::new(),
            new_chat_dropdown_input,
            new_chat_dropdown_section: 0,
            new_chat_dropdown_index: 0,
            last_used_settings,
            // Attachments state
            showing_attachments_picker: false,
            pending_attachments: Vec::new(),
            // Mouse cursor state
            mouse_cursor_hidden: false,
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
        }
    }

    /// Debounce interval for bounds persistence (in milliseconds)
    pub(super) const BOUNDS_DEBOUNCE_MS: u64 = 250;
}
