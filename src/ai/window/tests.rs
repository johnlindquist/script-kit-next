use super::*;

#[test]
fn test_context_inspector_shortcut_requires_cmd_alt_i_only() {
    let enabled = crate::ai::window::render_keydown::is_context_inspector_shortcut(
        "i",
        &gpui::Modifiers {
            platform: true,
            alt: true,
            ..Default::default()
        },
    );
    assert!(enabled, "Cmd+Alt+I should toggle the context inspector");

    let wrong_key = crate::ai::window::render_keydown::is_context_inspector_shortcut(
        "k",
        &gpui::Modifiers {
            platform: true,
            alt: true,
            ..Default::default()
        },
    );
    assert!(
        !wrong_key,
        "Cmd+Alt+K must not toggle the context inspector"
    );

    let missing_alt = crate::ai::window::render_keydown::is_context_inspector_shortcut(
        "i",
        &gpui::Modifiers {
            platform: true,
            ..Default::default()
        },
    );
    assert!(
        !missing_alt,
        "Cmd+I without Alt must not toggle the context inspector"
    );

    let extra_shift = crate::ai::window::render_keydown::is_context_inspector_shortcut(
        "i",
        &gpui::Modifiers {
            platform: true,
            alt: true,
            shift: true,
            ..Default::default()
        },
    );
    assert!(
        !extra_shift,
        "Cmd+Alt+Shift+I must not match the dedicated inspector shortcut"
    );
}

#[test]
fn test_streaming_generation_guard_logic() {
    // Simulate the guard check logic used in streaming updates
    let update_chat_id = ChatId::new();
    let streaming_chat_id: Option<ChatId> = Some(update_chat_id);
    let streaming_generation: u64 = 5;

    // Scenario 1: Matching generation and chat - should NOT be stale
    let update_generation = 5;
    let is_stale =
        streaming_generation != update_generation || streaming_chat_id != Some(update_chat_id);
    assert!(
        !is_stale,
        "Matching generation and chat should not be stale"
    );

    // Scenario 2: Generation mismatch - should be stale (old streaming task)
    let old_generation = 4;
    let is_stale =
        streaming_generation != old_generation || streaming_chat_id != Some(update_chat_id);
    assert!(is_stale, "Old generation should be stale");

    // Scenario 3: Chat ID mismatch - should be stale (user switched chats)
    let different_chat_id = ChatId::new();
    let is_stale =
        streaming_generation != update_generation || streaming_chat_id != Some(different_chat_id);
    assert!(is_stale, "Different chat ID should be stale");

    // Scenario 4: No streaming chat - should be stale
    let no_streaming: Option<ChatId> = None;
    let is_stale =
        streaming_generation != update_generation || no_streaming != Some(update_chat_id);
    assert!(is_stale, "No streaming chat should be stale");
}

/// Test that generation counter wraps correctly
#[test]
fn test_streaming_generation_wrapping() {
    let mut generation: u64 = u64::MAX;

    // Simulate multiple streaming sessions
    for expected in [0, 1, 2, 3, 4] {
        generation = generation.wrapping_add(1);
        assert_eq!(generation, expected, "Generation should wrap correctly");
    }
}

/// Test the submit_message guard logic - should only block if streaming
/// for the SAME chat
#[test]
fn test_submit_while_streaming_different_chat() {
    // Setup: streaming in chat A, trying to submit in chat B
    let chat_a = ChatId::new();
    let chat_b = ChatId::new();

    let is_streaming = true;
    let streaming_chat_id = Some(chat_a);
    let selected_chat_id = Some(chat_b);

    // The guard: block only if streaming AND same chat
    let should_block = is_streaming && streaming_chat_id == selected_chat_id;
    assert!(
        !should_block,
        "Should NOT block submission when streaming different chat"
    );

    // Same chat scenario should block
    let selected_chat_id = Some(chat_a);
    let should_block = is_streaming && streaming_chat_id == selected_chat_id;
    assert!(
        should_block,
        "Should block submission when streaming same chat"
    );
}

#[test]
fn test_ai_window_can_submit_message_returns_true_when_only_image_is_attached() {
    assert!(
        ai_window_can_submit_message("", true, false),
        "Image-only messages should be allowed"
    );
    assert!(
        ai_window_can_submit_message("   ", true, false),
        "Whitespace text with an attachment should be allowed"
    );
    assert!(
        ai_window_can_submit_message("hello", false, false),
        "Non-empty text should be allowed"
    );
    assert!(
        !ai_window_can_submit_message("   ", false, false),
        "Whitespace-only messages without attachments should be blocked"
    );
    assert!(
        ai_window_can_submit_message("", false, true),
        "Context-parts-only messages should be allowed"
    );
}

#[test]
fn test_ai_window_prune_deleted_message_ui_state_removes_only_deleted_ids() {
    let mut collapsed = std::collections::HashSet::from([
        "message-a".to_string(),
        "message-b".to_string(),
        "message-c".to_string(),
    ]);
    let mut expanded = std::collections::HashSet::from([
        "message-b".to_string(),
        "message-d".to_string(),
        "message-e".to_string(),
    ]);
    let deleted_message_ids = vec!["message-b".to_string(), "message-x".to_string()];

    ai_window_prune_deleted_message_ui_state(&mut collapsed, &mut expanded, &deleted_message_ids);

    assert!(
        !collapsed.contains("message-b"),
        "Deleted message IDs must be removed from collapsed state"
    );
    assert!(
        !expanded.contains("message-b"),
        "Deleted message IDs must be removed from expanded state"
    );
    assert!(
        collapsed.contains("message-a"),
        "Unrelated collapsed IDs must be preserved"
    );
    assert!(
        expanded.contains("message-d"),
        "Unrelated expanded IDs must be preserved"
    );
}

#[test]
fn test_ai_window_queue_command_if_open_enqueues_only_when_window_is_open() {
    let mut pending_commands = Vec::new();

    let was_queued = ai_window_queue_command_if_open(
        &mut pending_commands,
        false,
        AiCommand::SetSearch("hidden".to_string()),
    );
    assert!(
        !was_queued,
        "Commands must not queue when the AI window is closed"
    );
    assert!(
        pending_commands.is_empty(),
        "Queue should remain empty when closed"
    );

    let was_queued = ai_window_queue_command_if_open(
        &mut pending_commands,
        true,
        AiCommand::SetSearch("visible".to_string()),
    );
    assert!(
        was_queued,
        "Commands should queue when the AI window is open"
    );
    assert_eq!(
        pending_commands.len(),
        1,
        "Exactly one command should have been queued"
    );
    match pending_commands.first() {
        Some(AiCommand::SetSearch(query)) => assert_eq!(query, "visible"),
        _ => panic!("Expected queued command to be AiCommand::SetSearch"),
    }
}

#[test]
fn test_ai_window_queue_command_if_open_enqueues_add_attachment_command() {
    let mut pending_commands = Vec::new();
    let path = "/tmp/notes.md".to_string();

    let was_queued = ai_window_queue_command_if_open(
        &mut pending_commands,
        true,
        AiCommand::AddAttachment { path: path.clone() },
    );
    assert!(
        was_queued,
        "Attachment command should queue when window is open"
    );

    match pending_commands.first() {
        Some(AiCommand::AddAttachment { path: queued_path }) => {
            assert_eq!(queued_path, &path);
        }
        _ => panic!("Expected queued command to be AiCommand::AddAttachment"),
    }
}

#[test]
fn test_ai_window_queue_command_if_open_preserves_start_chat_provider_metadata() {
    let mut pending_commands = Vec::new();
    let callback = std::sync::Arc::new(|_model_id: String, _provider: String| {});

    let was_queued = ai_window_queue_command_if_open(
        &mut pending_commands,
        true,
        AiCommand::StartChat {
            chat_id: ChatId::new(),
            message: "hello".to_string(),
            parts: vec![crate::ai::message_parts::AiContextPart::ResourceUri {
                uri: "kit://context?profile=minimal".to_string(),
                label: "Current Context".to_string(),
            }],
            image: None,
            system_prompt: None,
            model_id: Some("gpt-4o".to_string()),
            provider: Some("openai".to_string()),
            on_created: Some(callback),
            submit: true,
        },
    );

    assert!(
        was_queued,
        "StartChat command should queue when window is open"
    );

    match pending_commands.first() {
        Some(AiCommand::StartChat {
            parts,
            model_id,
            provider,
            on_created,
            ..
        }) => {
            assert_eq!(
                parts.len(),
                1,
                "Queued StartChat command should retain context parts"
            );
            assert_eq!(model_id.as_deref(), Some("gpt-4o"));
            assert_eq!(provider.as_deref(), Some("openai"));
            assert!(
                on_created.is_some(),
                "Queued StartChat command should retain its creation callback"
            );
        }
        _ => panic!("Expected queued command to be AiCommand::StartChat"),
    }
}

#[test]
fn test_should_retry_existing_user_turn_only_when_last_message_is_user() {
    let chat_id = ChatId::new();

    let ends_with_user = vec![
        Message::assistant(chat_id, "previous assistant"),
        Message::user(chat_id, "latest user"),
    ];
    assert!(
        should_retry_existing_user_turn(&ends_with_user),
        "Retry should reuse the request when the latest message is a user turn"
    );

    let ends_with_assistant = vec![
        Message::user(chat_id, "latest user"),
        Message::assistant(chat_id, "latest assistant"),
    ];
    assert!(
        !should_retry_existing_user_turn(&ends_with_assistant),
        "Retry should not assume a reusable user turn when an assistant turn is last"
    );

    let empty_messages: Vec<Message> = Vec::new();
    assert!(
        !should_retry_existing_user_turn(&empty_messages),
        "Retry should be disabled without user messages"
    );
}

#[test]
fn test_should_persist_stale_completion_respects_suppression_set() {
    let chat_id = ChatId::new();
    let session = StreamingSessionKey {
        chat_id,
        generation: 42,
    };
    let mut suppressed = std::collections::HashSet::new();
    suppressed.insert(session);

    let should_persist = should_persist_stale_completion(&mut suppressed, session);
    assert!(
        !should_persist,
        "Explicitly suppressed sessions should not persist stale completions"
    );
    assert!(
        !suppressed.contains(&session),
        "Suppression should be consumed after one stale completion handling pass"
    );

    let unrelated_session = StreamingSessionKey {
        chat_id,
        generation: 99,
    };
    assert!(
        should_persist_stale_completion(&mut suppressed, unrelated_session),
        "Untracked sessions should persist stale completions for chat-switch continuity"
    );
}

/// Test ChatId comparison behavior
#[test]
fn test_chat_id_equality() {
    let id1 = ChatId::new();
    let id2 = ChatId::new();
    let id1_copy = id1;

    assert_eq!(id1, id1_copy, "Same ID should be equal");
    assert_ne!(id1, id2, "Different IDs should not be equal");
    assert_eq!(Some(id1), Some(id1_copy), "Option<ChatId> equality works");
    assert_ne!(Some(id1), Some(id2), "Option<ChatId> inequality works");
    assert_ne!(Some(id1), None, "Some vs None inequality works");
}

#[test]
fn test_setup_button_focus_index_wraps() {
    assert_eq!(AiApp::next_setup_button_focus_index(0, 1), 1);
    assert_eq!(AiApp::next_setup_button_focus_index(1, 1), 0);
    assert_eq!(AiApp::next_setup_button_focus_index(0, -1), 1);
    assert_eq!(AiApp::next_setup_button_focus_index(1, -1), 0);
}

/// Test setup mode detection logic
#[test]
fn test_setup_mode_detection() {
    // Setup mode is when: no models available AND not showing API key input
    struct SetupState {
        available_models_empty: bool,
        showing_api_key_input: bool,
    }

    let test_cases = vec![
        // (state, expected_in_setup_mode)
        (
            SetupState {
                available_models_empty: true,
                showing_api_key_input: false,
            },
            true,
            "No models and not showing input = setup mode",
        ),
        (
            SetupState {
                available_models_empty: true,
                showing_api_key_input: true,
            },
            false,
            "No models but showing input = NOT setup mode (keyboard routes to input)",
        ),
        (
            SetupState {
                available_models_empty: false,
                showing_api_key_input: false,
            },
            false,
            "Has models = NOT setup mode (normal chat mode)",
        ),
        (
            SetupState {
                available_models_empty: false,
                showing_api_key_input: true,
            },
            false,
            "Has models and showing input = NOT setup mode",
        ),
    ];

    for (state, expected, description) in test_cases {
        let in_setup_mode = state.available_models_empty && !state.showing_api_key_input;
        assert_eq!(in_setup_mode, expected, "{}", description);
    }
}

/// Test that setup button navigation covers all directions
#[test]
fn test_setup_button_navigation_directions() {
    // Test Tab (forward)
    assert_eq!(
        AiApp::next_setup_button_focus_index(0, 1),
        1,
        "Tab from 0 -> 1"
    );
    assert_eq!(
        AiApp::next_setup_button_focus_index(1, 1),
        0,
        "Tab from 1 -> 0 (wrap)"
    );

    // Test Shift+Tab / Up (backward)
    assert_eq!(
        AiApp::next_setup_button_focus_index(0, -1),
        1,
        "Shift+Tab from 0 -> 1 (wrap)"
    );
    assert_eq!(
        AiApp::next_setup_button_focus_index(1, -1),
        0,
        "Shift+Tab from 1 -> 0"
    );

    // Test multiple steps
    let mut index = 0usize;
    index = AiApp::next_setup_button_focus_index(index, 1); // 0 -> 1
    index = AiApp::next_setup_button_focus_index(index, 1); // 1 -> 0
    index = AiApp::next_setup_button_focus_index(index, 1); // 0 -> 1
    assert_eq!(index, 1, "Multiple forward steps should cycle correctly");

    let mut index = 0usize;
    index = AiApp::next_setup_button_focus_index(index, -1); // 0 -> 1
    index = AiApp::next_setup_button_focus_index(index, -1); // 1 -> 0
    index = AiApp::next_setup_button_focus_index(index, -1); // 0 -> 1
    assert_eq!(index, 1, "Multiple backward steps should cycle correctly");
}

/// Test SETUP_BUTTON_COUNT constant is correct
#[test]
fn test_setup_button_count() {
    // We have two buttons: "Configure Vercel AI Gateway" (index 0) and "Connect to Claude Code" (index 1)
    assert_eq!(
        AiApp::SETUP_BUTTON_COUNT,
        2,
        "Should have exactly 2 setup buttons"
    );

    // Index 0 should map to "Configure Vercel AI Gateway"
    // Index 1 should map to "Connect to Claude Code"
    // These are documented in the code: setup_button_focus_index: usize,
    // 0 = Configure Vercel AI Gateway, 1 = Connect to Claude Code
}

#[test]
fn test_build_sidebar_rows_inserts_headers_and_preserves_chat_order() {
    let now = Utc::now();

    let mut today_chat = Chat::new("model", "provider");
    today_chat.title = "Today".to_string();
    today_chat.updated_at = now;

    let mut yesterday_chat = Chat::new("model", "provider");
    yesterday_chat.title = "Yesterday".to_string();
    yesterday_chat.updated_at = now - chrono::Duration::days(1);

    let mut older_chat = Chat::new("model", "provider");
    older_chat.title = "Older".to_string();
    older_chat.updated_at = now - chrono::Duration::days(10);

    let chats = vec![
        today_chat.clone(),
        yesterday_chat.clone(),
        older_chat.clone(),
    ];
    let rows = build_sidebar_rows_for_chats(&chats);

    assert_eq!(
        rows.len(),
        6,
        "Expected 3 date headers + 3 chat rows for 3 cross-group chats"
    );

    match rows[0] {
        SidebarRow::Header {
            group: DateGroup::Today,
            is_first: true,
        } => {}
        _ => panic!("First row should be Today header"),
    }

    match rows[1] {
        SidebarRow::Chat { chat_id } => assert_eq!(chat_id, today_chat.id),
        _ => panic!("Second row should be the Today chat"),
    }

    match rows[2] {
        SidebarRow::Header {
            group: DateGroup::Yesterday,
            is_first: false,
        } => {}
        _ => panic!("Third row should be Yesterday header"),
    }

    match rows[3] {
        SidebarRow::Chat { chat_id } => assert_eq!(chat_id, yesterday_chat.id),
        _ => panic!("Fourth row should be the Yesterday chat"),
    }

    match rows[4] {
        SidebarRow::Header {
            group: DateGroup::Older,
            is_first: false,
        } => {}
        _ => panic!("Fifth row should be Older header"),
    }

    match rows[5] {
        SidebarRow::Chat { chat_id } => assert_eq!(chat_id, older_chat.id),
        _ => panic!("Sixth row should be the Older chat"),
    }
}

#[test]
fn test_welcome_suggestion_texts_pass_submit_guard() {
    // Welcome suggestion cards auto-submit on click. Verify that every
    // suggestion prompt passes the submit guard (non-empty text, no image).
    let suggestions = vec![
        "Write a script to automate a repetitive task",
        "Explain how this code works step by step",
        "Help me debug an error I'm seeing",
        "Generate a function that processes data",
    ];

    for suggestion in &suggestions {
        assert!(
            ai_window_can_submit_message(suggestion, false, false),
            "Welcome suggestion '{}' must pass the submit guard for auto-submit to work",
            suggestion
        );
    }
}

#[test]
fn test_message_body_content_does_not_truncate_long_messages() {
    let long_message = "lorem ipsum ".repeat(200);
    let display_content = AiApp::message_body_content(&long_message);
    assert_eq!(display_content, long_message);
}

/// Validates that new_conversation resets all per-conversation transient fields.
///
/// new_conversation() clears these fields before calling create_chat():
///   - pending_image
///   - pending_context_parts (single source of truth for attachments + context)
///   - collapsed_messages
///   - expanded_messages
///   - copied_message_id / copied_at
///   - last_streaming_duration / last_streaming_completed_at
///   - streaming_error
///   - editing_message_id
///
/// Additionally it cancels any active stream (is_streaming) before reset.
///
/// This test uses a struct mirroring AiApp's transient fields to verify the
/// reset contract without requiring a GPUI window context.
#[test]
fn test_new_conversation_reset_contract_clears_all_per_conversation_transient_fields() {
    /// Mirrors the per-conversation transient fields from AiApp that
    /// new_conversation() must reset.
    struct ConversationTransientState {
        pending_image: Option<String>,
        pending_context_parts: Vec<crate::ai::message_parts::AiContextPart>,
        context_picker_open: bool,
        collapsed_messages: std::collections::HashSet<String>,
        expanded_messages: std::collections::HashSet<String>,
        copied_message_id: Option<String>,
        copied_at: Option<std::time::Instant>,
        last_streaming_duration: Option<std::time::Duration>,
        last_streaming_completed_at: Option<std::time::Instant>,
        streaming_error: Option<String>,
        editing_message_id: Option<String>,
        last_prepared_message_receipt: bool,
        last_preflight_audit: bool,
        last_context_receipt: bool,
        show_context_inspector: bool,
        show_context_drawer: bool,
    }

    impl ConversationTransientState {
        /// Apply the same reset logic as AiApp::new_conversation()
        fn reset(&mut self) {
            self.pending_image = None;
            self.pending_context_parts.clear();
            self.context_picker_open = false;
            self.collapsed_messages.clear();
            self.expanded_messages.clear();
            self.copied_message_id = None;
            self.copied_at = None;
            self.last_streaming_duration = None;
            self.last_streaming_completed_at = None;
            self.streaming_error = None;
            self.editing_message_id = None;
            self.last_prepared_message_receipt = false;
            self.last_preflight_audit = false;
            self.last_context_receipt = false;
            self.show_context_inspector = false;
            self.show_context_drawer = false;
        }
    }

    // Build dirty state (simulates mid-conversation)
    let mut state = ConversationTransientState {
        pending_image: Some("base64data".to_string()),
        pending_context_parts: vec![crate::ai::message_parts::AiContextPart::FilePath {
            path: "/tmp/file.txt".to_string(),
            label: "file.txt".to_string(),
        }],
        context_picker_open: true,
        collapsed_messages: ["msg-1".to_string()].into_iter().collect(),
        expanded_messages: ["msg-2".to_string()].into_iter().collect(),
        copied_message_id: Some("msg-1".to_string()),
        copied_at: Some(std::time::Instant::now()),
        last_streaming_duration: Some(std::time::Duration::from_secs(5)),
        last_streaming_completed_at: Some(std::time::Instant::now()),
        streaming_error: Some("previous error".to_string()),
        editing_message_id: Some("msg-3".to_string()),
        last_prepared_message_receipt: true,
        last_preflight_audit: true,
        last_context_receipt: true,
        show_context_inspector: true,
        show_context_drawer: true,
    };

    // Verify dirty state is non-default
    assert!(state.pending_image.is_some());
    assert!(!state.pending_context_parts.is_empty());
    assert!(state.context_picker_open);
    assert!(!state.collapsed_messages.is_empty());
    assert!(!state.expanded_messages.is_empty());
    assert!(state.copied_message_id.is_some());
    assert!(state.copied_at.is_some());
    assert!(state.last_streaming_duration.is_some());
    assert!(state.last_streaming_completed_at.is_some());
    assert!(state.streaming_error.is_some());
    assert!(state.editing_message_id.is_some());
    assert!(state.last_prepared_message_receipt);
    assert!(state.last_preflight_audit);
    assert!(state.last_context_receipt);
    assert!(state.show_context_inspector);
    assert!(state.show_context_drawer);

    // Apply reset
    state.reset();

    // Assert all fields are at their default state
    assert!(
        state.pending_image.is_none(),
        "pending_image must be cleared on new conversation"
    );
    assert!(
        state.pending_context_parts.is_empty(),
        "pending_context_parts must be cleared on new conversation"
    );
    assert!(
        !state.context_picker_open,
        "context_picker must be cleared on new conversation"
    );
    assert!(
        state.collapsed_messages.is_empty(),
        "collapsed_messages must be cleared on new conversation"
    );
    assert!(
        state.expanded_messages.is_empty(),
        "expanded_messages must be cleared on new conversation"
    );
    assert!(
        state.copied_message_id.is_none(),
        "copied_message_id must be cleared on new conversation"
    );
    assert!(
        state.copied_at.is_none(),
        "copied_at must be cleared on new conversation"
    );
    assert!(
        state.last_streaming_duration.is_none(),
        "last_streaming_duration must be cleared on new conversation"
    );
    assert!(
        state.last_streaming_completed_at.is_none(),
        "last_streaming_completed_at must be cleared on new conversation"
    );
    assert!(
        state.streaming_error.is_none(),
        "streaming_error must be cleared on new conversation"
    );
    assert!(
        state.editing_message_id.is_none(),
        "editing_message_id must be cleared on new conversation"
    );
    assert!(
        !state.last_prepared_message_receipt,
        "last_prepared_message_receipt must be cleared on new conversation"
    );
    assert!(
        !state.last_preflight_audit,
        "last_preflight_audit must be cleared on new conversation"
    );
    assert!(
        !state.last_context_receipt,
        "last_context_receipt must be cleared on new conversation"
    );
    assert!(
        !state.show_context_inspector,
        "show_context_inspector must be cleared on new conversation"
    );
    assert!(
        !state.show_context_drawer,
        "show_context_drawer must be cleared on new conversation"
    );
}

#[test]
fn test_welcome_suggestion_click_with_no_provider_triggers_mock_streaming() {
    // When no provider is configured, clicking a welcome suggestion card
    // triggers mock streaming (demo mode) rather than an error.
    // The mock response educates the user about configuring API keys.
    let suggestions = [
        "Write a script to automate a repetitive task",
        "Explain how this code works step by step",
        "Help me debug an error I'm seeing",
        "Generate a function that processes data",
    ];

    for suggestion in &suggestions {
        let mock = generate_mock_response(suggestion);
        assert!(
            !mock.is_empty(),
            "Mock response for '{}' must not be empty — demo mode needs helpful content",
            suggestion
        );
        // Mock responses should mention API key setup or be contextually helpful
        let lower = mock.to_lowercase();
        assert!(
            lower.contains("api")
                || lower.contains("key")
                || lower.contains("configure")
                || lower.contains("script"),
            "Mock response for '{}' should guide user toward configuration or be contextually helpful, got: {}",
            suggestion,
            &mock[..mock.len().min(120)]
        );
        tracing::info!(
            suggestion = suggestion,
            response_len = mock.len(),
            "mock_streaming_validation: suggestion produced valid response"
        );
    }
}

/// Verify the debounce contract: each keystroke bumps generation and replaces the task,
/// empty query clears the task for instant feedback.
#[test]
fn test_search_debounce_generation_and_task_replacement_contract() {
    // The debounce contract:
    // 1. Each keystroke bumps search_generation
    // 2. Each keystroke replaces search_debounce_task (dropping/cancelling the old one)
    // 3. Empty query sets search_debounce_task = None (no debounce)
    // 4. The generation counter guards against stale results even without task cancellation

    // Simulate the state machine that on_search_change maintains
    fn simulate_keystroke(generation: &mut u64) -> u64 {
        *generation += 1;
        *generation // returns the generation that would be captured by the debounce task
    }

    let mut search_generation: u64 = 0;

    // Keystroke 1: "h" — starts a debounce task at gen 1
    let task1_gen = simulate_keystroke(&mut search_generation);
    assert_eq!(task1_gen, 1);

    // Keystroke 2: "he" — replaces task (gen 1 cancelled), new task at gen 2
    let task2_gen = simulate_keystroke(&mut search_generation);
    assert_eq!(task2_gen, 2);

    // Keystroke 3: "hel" — replaces task again, gen 3
    let task3_gen = simulate_keystroke(&mut search_generation);
    assert_eq!(task3_gen, 3);

    // Only gen 3 should match the current generation (stale guard)
    assert_ne!(search_generation, task1_gen, "Gen 1 should be stale");
    assert_ne!(search_generation, task2_gen, "Gen 2 should be stale");
    assert_eq!(
        search_generation, task3_gen,
        "Only gen 3 should match current"
    );

    // Clear search (empty query): bumps generation, no task needed
    let _clear_gen = simulate_keystroke(&mut search_generation);
    assert_eq!(search_generation, 4);
    // Empty query path is synchronous — no debounce task is stored
}

/// Verify the SEARCH_DEBOUNCE_MS constant is within reasonable UX bounds.
#[test]
fn test_search_debounce_constant_is_reasonable() {
    // 150ms is the standard debounce for search-as-you-type UX.
    // Too low (<50ms) provides no benefit; too high (>300ms) feels sluggish.
    assert_eq!(
        AiApp::SEARCH_DEBOUNCE_MS,
        150,
        "Search debounce should be 150ms for responsive feel without excess queries"
    );
}

/// Verify that on_search_change with empty string reloads all chats and clears search state.
/// This validates the synchronous clear path that Escape triggers after resetting search_query.
#[test]
fn test_escape_clear_search_state_contract() {
    // Simulate the state that Escape clears before triggering on_search_change("")
    let mut search_query = "test query".to_string();
    let mut search_generation: u64 = 5;
    let mut search_snippets: std::collections::HashMap<ChatId, String> =
        std::collections::HashMap::new();
    let mut search_matched_title: std::collections::HashMap<ChatId, bool> =
        std::collections::HashMap::new();

    // Populate some search state
    let fake_id = ChatId::new();
    search_snippets.insert(fake_id, "some snippet".to_string());
    search_matched_title.insert(fake_id, true);

    // Simulate Escape handler logic
    search_query.clear();
    search_generation += 1;
    search_snippets.clear();
    search_matched_title.clear();

    assert!(
        search_query.is_empty(),
        "Escape must clear the search_query string"
    );
    assert_eq!(
        search_generation, 6,
        "Escape must increment search_generation to invalidate in-flight results"
    );
    assert!(
        search_snippets.is_empty(),
        "Escape must clear search_snippets"
    );
    assert!(
        search_matched_title.is_empty(),
        "Escape must clear search_matched_title"
    );
}

/// Validates the auto-collapse threshold logic from `AiApp::is_message_collapsed`.
///
/// The rule (interactions.rs):
///   1. If msg_id is in `expanded_messages` → always expanded (not collapsed).
///   2. If msg_id is in `collapsed_messages` → always collapsed.
///   3. Otherwise auto-collapse via `compute_collapse_decision(content_len)`.
///
/// The render path (render_message.rs) gates the toggle button on
/// `content.len() > MSG_COLLAPSE_CHAR_THRESHOLD`.
#[test]
fn test_message_auto_collapse_threshold() {
    // Mirror the three sets from AiApp state
    let expanded_messages: std::collections::HashSet<String> =
        ["expanded-msg".to_string()].into_iter().collect();
    let collapsed_messages: std::collections::HashSet<String> =
        ["collapsed-msg".to_string()].into_iter().collect();

    // Uses the shared pure helper for auto-collapse, with user-override layer on top
    let is_collapsed = |msg_id: &str, content_len: usize| -> bool {
        if expanded_messages.contains(msg_id) {
            return false;
        }
        if collapsed_messages.contains(msg_id) {
            return true;
        }
        compute_collapse_decision(content_len).should_collapse
    };

    // --- Auto-collapse threshold (no explicit user override) ---
    let neutral_id = "neutral-msg";

    // Verify the helper returns structured data
    let decision = compute_collapse_decision(MSG_COLLAPSE_CHAR_THRESHOLD);
    assert_eq!(decision.char_count, MSG_COLLAPSE_CHAR_THRESHOLD);
    assert_eq!(decision.threshold, MSG_COLLAPSE_CHAR_THRESHOLD);
    assert!(
        !decision.should_collapse,
        "Exactly at threshold must NOT auto-collapse (> not >=)"
    );

    let decision_over = compute_collapse_decision(MSG_COLLAPSE_CHAR_THRESHOLD + 1);
    assert!(
        decision_over.should_collapse,
        "One over threshold must auto-collapse"
    );

    // Exactly at boundary: should NOT collapse (> threshold, not >=)
    assert!(
        !is_collapsed(neutral_id, MSG_COLLAPSE_CHAR_THRESHOLD),
        "At boundary must NOT auto-collapse"
    );
    // One char over: should collapse
    assert!(
        is_collapsed(neutral_id, MSG_COLLAPSE_CHAR_THRESHOLD + 1),
        "Over threshold must auto-collapse"
    );
    // Well under: should not collapse
    assert!(
        !is_collapsed(neutral_id, 100),
        "100-char message must not auto-collapse"
    );
    // Well over: should collapse
    assert!(
        is_collapsed(neutral_id, 5000),
        "5000-char message must auto-collapse"
    );
    // Zero length: should not collapse
    assert!(
        !is_collapsed(neutral_id, 0),
        "Empty message must not auto-collapse"
    );

    // --- Explicit expanded override beats auto-collapse ---
    assert!(
        !is_collapsed("expanded-msg", 5000),
        "Explicitly expanded message must not collapse even when over threshold"
    );

    // --- Explicit collapsed override beats auto-expand ---
    assert!(
        is_collapsed("collapsed-msg", 100),
        "Explicitly collapsed message must stay collapsed even when under threshold"
    );

    // --- Render toggle visibility uses the same threshold constant ---
    let should_show_toggle =
        |content_len: usize| -> bool { content_len > MSG_COLLAPSE_CHAR_THRESHOLD };
    assert!(
        !should_show_toggle(MSG_COLLAPSE_CHAR_THRESHOLD),
        "Toggle button must be hidden at exactly threshold"
    );
    assert!(
        should_show_toggle(MSG_COLLAPSE_CHAR_THRESHOLD + 1),
        "Toggle button must be visible above threshold"
    );

    tracing::info!(
        threshold = MSG_COLLAPSE_CHAR_THRESHOLD,
        "message_auto_collapse_threshold: all boundary and override cases validated"
    );
}

#[test]
fn ai_shortcuts_match_multiline_composer_behavior() {
    let input_section = AI_SHORTCUT_SECTIONS
        .iter()
        .find(|section| section.title == "Input")
        .expect("Input shortcut section should exist");

    assert!(
        input_section
            .items
            .iter()
            .any(|item| item.keys == "Enter" && item.description == "Send message"),
        "Input shortcuts should advertise Enter to send"
    );
    assert!(
        input_section
            .items
            .iter()
            .any(|item| item.keys == "Shift+Enter" && item.description == "Insert newline"),
        "Input shortcuts should advertise Shift+Enter for newline"
    );
    assert!(
        input_section
            .items
            .iter()
            .all(|item| !item.keys.contains("\u{2318}") || !item.keys.contains("Enter")),
        "Stale Cmd+Enter hint should be removed after multiline composer landed"
    );
}

#[test]
fn ai_shortcuts_cover_sidebar_search_and_new_chat() {
    let flat: Vec<(&str, &str)> = AI_SHORTCUT_SECTIONS
        .iter()
        .flat_map(|section| {
            section
                .items
                .iter()
                .map(|item| (item.keys, item.description))
        })
        .collect();

    assert!(flat.contains(&("\u{2318}B", "Toggle sidebar")));
    assert!(flat.contains(&("\u{2318}\u{21e7}F", "Focus search")));
    assert!(flat.contains(&("\u{2318}N", "New chat")));
}

#[test]
fn ai_shortcut_sections_have_four_categories() {
    assert_eq!(
        AI_SHORTCUT_SECTIONS.len(),
        4,
        "Should have exactly 4 shortcut sections: Navigation, Chat, Input, Actions"
    );
    assert_eq!(AI_SHORTCUT_SECTIONS[0].title, "Navigation");
    assert_eq!(AI_SHORTCUT_SECTIONS[1].title, "Chat");
    assert_eq!(AI_SHORTCUT_SECTIONS[2].title, "Input");
    assert_eq!(AI_SHORTCUT_SECTIONS[3].title, "Actions");
}

/// Verify that the normalization applied by `set_composer_value` converts
/// CR+LF to plain LF, preserves standalone LF, and leaves clean text alone.
#[test]
fn test_set_composer_value_normalizes_crlf_to_lf() {
    // Simulates the normalization logic inside set_composer_value
    let normalize = |input: &str| -> String { input.replace("\r\n", "\n") };

    // CR+LF → LF
    assert_eq!(normalize("hello\r\nworld"), "hello\nworld");

    // Multiple CR+LF
    assert_eq!(normalize("a\r\nb\r\nc"), "a\nb\nc");

    // Standalone LF preserved
    assert_eq!(normalize("hello\nworld"), "hello\nworld");

    // No newlines unchanged
    assert_eq!(normalize("hello world"), "hello world");

    // Empty string
    assert_eq!(normalize(""), "");

    // Mixed: CR+LF and LF
    assert_eq!(normalize("a\r\nb\nc\r\nd"), "a\nb\nc\nd");
}

/// Proves that a pending file-path context part changes the final user message content.
///
/// This mirrors the just-in-time resolution logic in `submit_message`:
/// resolve parts → prefix → combine with user text.
#[test]
fn test_pending_context_part_changes_final_user_message_content() {
    use crate::ai::message_parts::{resolve_context_parts_to_prompt_prefix, AiContextPart};

    let dir = tempfile::tempdir().expect("create temp dir");
    let file_path = dir.path().join("context.txt");
    std::fs::write(&file_path, "important context here").expect("write");

    let pending_parts = vec![AiContextPart::FilePath {
        path: file_path.to_string_lossy().to_string(),
        label: "context.txt".to_string(),
    }];

    let user_typed_content = "What does this mean?";

    // Resolve context parts (mirrors submit_message logic)
    let prefix = resolve_context_parts_to_prompt_prefix(&pending_parts, &[], &[])
        .expect("resolution should succeed");
    assert!(
        !prefix.is_empty(),
        "Resolved prefix must not be empty when parts are present"
    );

    // Compose final content (mirrors submit_message composition)
    let final_content = if !prefix.is_empty() && !user_typed_content.trim().is_empty() {
        format!("{prefix}\n\n{user_typed_content}")
    } else if !prefix.is_empty() {
        prefix.clone()
    } else {
        user_typed_content.to_string()
    };

    // The final content must differ from the raw typed content
    assert_ne!(
        final_content, user_typed_content,
        "Final content must include resolved context prefix, not just raw typed text"
    );
    assert!(
        final_content.contains("important context here"),
        "Final content must include the resolved file content"
    );
    assert!(
        final_content.contains(user_typed_content),
        "Final content must still include the user's typed text"
    );
    assert!(
        final_content.contains("<attachment"),
        "Final content must contain structured attachment tag from resolution"
    );
}

/// Proves that image-only submit (no text, no context parts) still works.
#[test]
fn test_image_only_submit_still_works_without_context_parts() {
    let content = "";
    let has_pending_image = true;
    let has_pending_context_parts = false;

    assert!(
        ai_window_can_submit_message(content, has_pending_image, has_pending_context_parts),
        "Image-only messages must pass the submit guard"
    );

    // With no context parts, the final content should be the raw content
    let pending_parts: Vec<crate::ai::message_parts::AiContextPart> = Vec::new();

    let prefix =
        crate::ai::message_parts::resolve_context_parts_to_prompt_prefix(&pending_parts, &[], &[])
            .expect("empty resolution should succeed");
    assert!(prefix.is_empty(), "Empty parts should produce empty prefix");

    // Final content with no parts and empty text should remain empty
    let final_content = if !prefix.is_empty() && !content.trim().is_empty() {
        format!("{prefix}\n\n{content}")
    } else if !prefix.is_empty() {
        prefix
    } else {
        content.to_string()
    };

    assert_eq!(
        final_content, "",
        "Image-only submit should leave message content empty (image is attached separately)"
    );
}

/// Proves that context-parts-only submit (no text, no image) produces valid content.
#[test]
fn test_context_parts_only_submit_produces_valid_content() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let file_path = dir.path().join("data.json");
    std::fs::write(&file_path, r#"{"key": "value"}"#).expect("write");

    let pending_parts = vec![crate::ai::message_parts::AiContextPart::FilePath {
        path: file_path.to_string_lossy().to_string(),
        label: "data.json".to_string(),
    }];

    let content = "";

    assert!(
        ai_window_can_submit_message(content, false, true),
        "Context-parts-only messages must pass the submit guard"
    );

    let prefix =
        crate::ai::message_parts::resolve_context_parts_to_prompt_prefix(&pending_parts, &[], &[])
            .expect("resolution should succeed");

    // With no user text, final content is just the prefix
    let final_content = if !prefix.is_empty() && !content.trim().is_empty() {
        format!("{prefix}\n\n{content}")
    } else if !prefix.is_empty() {
        prefix
    } else {
        content.to_string()
    };

    assert!(
        !final_content.is_empty(),
        "Context-parts-only submit must produce non-empty final content"
    );
    assert!(
        final_content.contains("<attachment"),
        "Context-parts-only content must contain structured attachment from resolution"
    );
}

/// Proves that failed context resolution falls back to raw content without crashing.
#[test]
fn test_failed_context_resolution_falls_back_to_raw_content() {
    let pending_parts = vec![crate::ai::message_parts::AiContextPart::FilePath {
        path: "/nonexistent/path/that/does/not/exist.txt".to_string(),
        label: "ghost.txt".to_string(),
    }];

    let user_typed_content = "My question";

    let result =
        crate::ai::message_parts::resolve_context_parts_to_prompt_prefix(&pending_parts, &[], &[]);

    // Resolution fails for nonexistent files
    assert!(
        result.is_err(),
        "Nonexistent file path should fail resolution"
    );

    // The submit path falls back to raw content on error
    let final_content = match result {
        Ok(prefix) if !prefix.is_empty() && !user_typed_content.trim().is_empty() => {
            format!("{prefix}\n\n{user_typed_content}")
        }
        Ok(prefix) if !prefix.is_empty() => prefix,
        _ => user_typed_content.to_string(),
    };

    assert_eq!(
        final_content, user_typed_content,
        "Failed resolution must fall back to raw user content"
    );
}

// =========================================================================
// Context action behavior tests
//
// Proves that each context command-bar action mutates pending_context_parts
// correctly: exact URI, dedup, and clear semantics.
//
// Uses a mirror struct that replicates the add/clear logic from
// AiApp::add_context_part / AiApp::clear_context_parts (dropdowns.rs)
// so we can test without a GPUI window context.
//
// Run with: cargo test --quiet context_action_behavior
// =========================================================================

/// Minimal mirror of AiApp's context-part mutation logic.
/// Replicates `add_context_part` dedup and `clear_context_parts` from dropdowns.rs.
struct ContextPartState {
    pending_context_parts: Vec<crate::ai::message_parts::AiContextPart>,
}

impl ContextPartState {
    fn new() -> Self {
        Self {
            pending_context_parts: Vec::new(),
        }
    }

    /// Mirror of AiApp::add_context_part (dropdowns.rs:373-399)
    fn add_context_part(&mut self, part: crate::ai::message_parts::AiContextPart) -> bool {
        let already_present = self
            .pending_context_parts
            .iter()
            .any(|existing| existing == &part);

        if already_present {
            tracing::info!(
                target: "ai",
                label = %part.label(),
                source = %part.source(),
                "context_action_behavior_add_skipped_duplicate"
            );
            return false;
        }

        let count_before = self.pending_context_parts.len();
        tracing::info!(
            target: "ai",
            label = %part.label(),
            source = %part.source(),
            count_before,
            "context_action_behavior_part_added"
        );

        self.pending_context_parts.push(part);
        true
    }

    /// Mirror of AiApp::clear_context_parts (dropdowns.rs:403-416)
    fn clear_context_parts(&mut self) -> usize {
        let cleared_count = self.pending_context_parts.len();
        if cleared_count == 0 {
            return 0;
        }
        self.pending_context_parts.clear();
        tracing::info!(
            target: "ai",
            cleared_count,
            "context_action_behavior_parts_cleared"
        );
        cleared_count
    }

    /// Simulate execute_action dispatch using the canonical context contract.
    fn execute_action(&mut self, action_id: &str) {
        if let Some(kind) =
            crate::ai::context_contract::ContextAttachmentKind::from_action_id(action_id)
        {
            self.add_context_part(kind.part());
            return;
        }

        if crate::ai::context_contract::is_clear_context_action(action_id) {
            self.clear_context_parts();
            return;
        }

        panic!("unexpected action: {action_id}");
    }
}

/// add_current_context inserts exactly one ResourceUri with the minimal profile URI.
#[test]
fn context_action_behavior_add_current_context_inserts_minimal_uri() {
    let mut state = ContextPartState::new();
    state.execute_action("add_current_context");

    assert_eq!(
        state.pending_context_parts.len(),
        1,
        "add_current_context must insert exactly one part"
    );
    match &state.pending_context_parts[0] {
        crate::ai::message_parts::AiContextPart::ResourceUri { uri, label } => {
            assert_eq!(uri, "kit://context?profile=minimal");
            assert_eq!(label, "Current Context");
        }
        other => panic!("expected ResourceUri, got {other:?}"),
    }

    tracing::info!(
        target: "ai",
        count = state.pending_context_parts.len(),
        parts = ?state.pending_context_parts,
        "context_action_behavior_add_current_context_result"
    );
}

/// add_context_full inserts exactly one ResourceUri with the full context URI.
#[test]
fn context_action_behavior_add_context_full_inserts_full_uri() {
    let mut state = ContextPartState::new();
    state.execute_action("add_context_full");

    assert_eq!(
        state.pending_context_parts.len(),
        1,
        "add_context_full must insert exactly one part"
    );
    match &state.pending_context_parts[0] {
        crate::ai::message_parts::AiContextPart::ResourceUri { uri, label } => {
            assert_eq!(uri, "kit://context");
            assert_eq!(label, "Current Context (Full)");
        }
        other => panic!("expected ResourceUri, got {other:?}"),
    }
}

/// add_context_diagnostics inserts the exact diagnostics URI.
#[test]
fn context_action_behavior_add_context_diagnostics_inserts_diagnostics_uri() {
    let mut state = ContextPartState::new();
    state.execute_action("add_context_diagnostics");

    assert_eq!(
        state.pending_context_parts.len(),
        1,
        "add_context_diagnostics must insert exactly one part"
    );
    match &state.pending_context_parts[0] {
        crate::ai::message_parts::AiContextPart::ResourceUri { uri, label } => {
            assert_eq!(
                uri, "kit://context?diagnostics=1",
                "diagnostics URI contract"
            );
            assert_eq!(label, "Context Diagnostics");
        }
        other => panic!("expected ResourceUri, got {other:?}"),
    }
}

/// clear_context removes all pending context parts.
#[test]
fn context_action_behavior_clear_context_removes_all_parts() {
    let mut state = ContextPartState::new();

    // Attach multiple different parts
    state.execute_action("add_current_context");
    state.execute_action("add_context_full");
    state.execute_action("add_context_diagnostics");
    assert_eq!(
        state.pending_context_parts.len(),
        3,
        "should have 3 parts before clear"
    );

    let count_before = state.pending_context_parts.len();
    state.execute_action("clear_context");

    tracing::info!(
        target: "ai",
        count_before,
        count_after = state.pending_context_parts.len(),
        "context_action_behavior_clear_result"
    );

    assert!(
        state.pending_context_parts.is_empty(),
        "clear_context must remove all pending context parts"
    );
}

/// Invoking the same attach action twice does not create duplicate pending context parts.
#[test]
fn context_action_behavior_duplicate_attach_is_idempotent() {
    let mut state = ContextPartState::new();

    // First invocation - should insert
    let inserted = state.add_context_part(crate::ai::message_parts::AiContextPart::ResourceUri {
        uri: "kit://context?profile=minimal".to_string(),
        label: "Current Context".to_string(),
    });
    assert!(inserted, "first add should succeed");
    assert_eq!(state.pending_context_parts.len(), 1);

    // Second invocation - should be a no-op (dedup)
    let inserted = state.add_context_part(crate::ai::message_parts::AiContextPart::ResourceUri {
        uri: "kit://context?profile=minimal".to_string(),
        label: "Current Context".to_string(),
    });
    assert!(!inserted, "duplicate add should be rejected");
    assert_eq!(
        state.pending_context_parts.len(),
        1,
        "duplicate add must not increase pending parts count"
    );

    tracing::info!(
        target: "ai",
        count = state.pending_context_parts.len(),
        parts = ?state.pending_context_parts,
        "context_action_behavior_dedup_result"
    );
}

/// All seven context actions via execute_action produce the expected state transitions.
#[test]
fn context_action_behavior_all_actions_via_execute_action() {
    let mut state = ContextPartState::new();

    // Attach all six add_* actions
    let add_actions = [
        "add_current_context",
        "add_context_full",
        "add_selection_context",
        "add_browser_context",
        "add_window_context",
        "add_context_diagnostics",
    ];

    for (i, action) in add_actions.iter().enumerate() {
        state.execute_action(action);
        assert_eq!(
            state.pending_context_parts.len(),
            i + 1,
            "after executing {action}, expected {} parts",
            i + 1
        );
    }

    // Verify each part's URI
    let expected_uris = [
        "kit://context?profile=minimal",
        "kit://context",
        "kit://context?selectedText=1&frontmostApp=0&menuBar=0&browserUrl=0&focusedWindow=0",
        "kit://context?selectedText=0&frontmostApp=0&menuBar=0&browserUrl=1&focusedWindow=0",
        "kit://context?selectedText=0&frontmostApp=1&menuBar=0&browserUrl=0&focusedWindow=1",
        "kit://context?diagnostics=1",
    ];

    for (i, expected_uri) in expected_uris.iter().enumerate() {
        match &state.pending_context_parts[i] {
            crate::ai::message_parts::AiContextPart::ResourceUri { uri, .. } => {
                assert_eq!(uri, expected_uri, "URI mismatch at index {i}");
            }
            other => panic!("expected ResourceUri at index {i}, got {other:?}"),
        }
    }

    tracing::info!(
        target: "ai",
        count = state.pending_context_parts.len(),
        "context_action_behavior_all_actions_attached"
    );

    // Now clear
    state.execute_action("clear_context");
    assert!(
        state.pending_context_parts.is_empty(),
        "clear_context must empty all parts after attaching all six"
    );
}

/// Duplicate execute_action calls for each action type are all idempotent.
#[test]
fn context_action_behavior_execute_action_dedup_all_types() {
    let add_actions = [
        "add_current_context",
        "add_context_full",
        "add_selection_context",
        "add_browser_context",
        "add_window_context",
        "add_context_diagnostics",
    ];

    for action in &add_actions {
        let mut state = ContextPartState::new();

        state.execute_action(action);
        assert_eq!(state.pending_context_parts.len(), 1, "{action}: first call");

        state.execute_action(action);
        assert_eq!(
            state.pending_context_parts.len(),
            1,
            "{action}: duplicate call must not increase count"
        );
    }
}

/// clear_context on empty state is a no-op (returns 0).
#[test]
fn context_action_behavior_clear_empty_is_noop() {
    let mut state = ContextPartState::new();
    let cleared = state.clear_context_parts();
    assert_eq!(cleared, 0, "clearing empty state should report 0 cleared");
    assert!(state.pending_context_parts.is_empty());
}

/// Serialized pending parts array is valid JSON (agent-parseable).
#[test]
fn context_action_behavior_pending_parts_serializable() {
    let mut state = ContextPartState::new();
    state.execute_action("add_current_context");
    state.execute_action("add_context_full");

    let json = serde_json::to_string(&state.pending_context_parts)
        .expect("pending_context_parts must serialize to JSON");
    let parsed: Vec<crate::ai::message_parts::AiContextPart> =
        serde_json::from_str(&json).expect("JSON must roundtrip");

    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed, state.pending_context_parts);

    tracing::info!(
        target: "ai",
        count_before = state.pending_context_parts.len(),
        count_after = parsed.len(),
        serialized = %json,
        "context_action_behavior_serialization_result"
    );
}

// ---------------------------------------------------------------------------
// Context preview UI tests
// ---------------------------------------------------------------------------

/// Helper: build a minimal stub with pending_context_parts for preview tests.
/// Uses canonical labels from `ContextAttachmentKind` to prevent drift.
fn make_preview_test_state() -> PreviewTestState {
    let parts = vec![
        crate::ai::context_contract::ContextAttachmentKind::Current.part(),
        crate::ai::context_contract::ContextAttachmentKind::Full.part(),
        crate::ai::context_contract::ContextAttachmentKind::Diagnostics.part(),
    ];
    PreviewTestState {
        pending_context_parts: parts,
        context_preview_index: None,
    }
}

/// Lightweight stand-in for the fields of AiApp used by preview logic.
struct PreviewTestState {
    pending_context_parts: Vec<crate::ai::message_parts::AiContextPart>,
    context_preview_index: Option<usize>,
}

impl PreviewTestState {
    fn toggle_preview(&mut self, index: usize) {
        if self.context_preview_index == Some(index) {
            self.context_preview_index = None;
        } else {
            self.context_preview_index = Some(index);
        }
    }

    fn close_preview(&mut self) {
        self.context_preview_index = None;
    }

    fn active_preview(&self) -> Option<(usize, context_preview::ContextPreviewInfo)> {
        let idx = self.context_preview_index?;
        let part = self.pending_context_parts.get(idx)?;
        Some((idx, context_preview::derive_context_preview_info(part)))
    }
}

#[test]
fn context_preview_ui_open_and_close_deterministic() {
    let mut state = make_preview_test_state();

    // Initially no preview
    assert!(state.active_preview().is_none(), "no preview at start");

    // Open preview for index 0 (minimal)
    state.toggle_preview(0);
    assert_eq!(state.context_preview_index, Some(0));
    let (idx, info) = state.active_preview().expect("preview should be active");
    assert_eq!(idx, 0);
    assert_eq!(
        info.profile,
        context_preview::ContextPreviewProfile::Minimal
    );

    // Close by toggling same index
    state.toggle_preview(0);
    assert!(
        state.active_preview().is_none(),
        "preview closed on re-toggle"
    );
}

#[test]
fn context_preview_ui_switch_between_chips() {
    let mut state = make_preview_test_state();

    // Open minimal
    state.toggle_preview(0);
    assert_eq!(state.context_preview_index, Some(0));

    // Switch to full
    state.toggle_preview(1);
    assert_eq!(state.context_preview_index, Some(1));
    let (_, info) = state.active_preview().expect("preview should be active");
    assert_eq!(info.profile, context_preview::ContextPreviewProfile::Full);
}

#[test]
fn context_preview_ui_close_explicit() {
    let mut state = make_preview_test_state();
    state.toggle_preview(2);
    assert!(state.active_preview().is_some());

    state.close_preview();
    assert!(state.active_preview().is_none());
}

#[test]
fn context_preview_ui_diagnostics_chip_shows_diagnostics() {
    let mut state = make_preview_test_state();
    state.toggle_preview(2); // diagnostics chip
    let (_, info) = state.active_preview().expect("preview should be active");
    assert!(
        info.has_diagnostics,
        "diagnostics chip must show diagnostics"
    );
    assert!(info.description.contains("diagnostics"));
}

#[test]
fn context_preview_ui_full_visually_distinct_from_minimal() {
    let mut state = make_preview_test_state();

    state.toggle_preview(0);
    let (_, minimal_info) = state.active_preview().expect("minimal preview");

    state.toggle_preview(1);
    let (_, full_info) = state.active_preview().expect("full preview");

    assert_ne!(
        minimal_info.profile, full_info.profile,
        "minimal and full must have different profiles"
    );
    assert_ne!(
        minimal_info.description, full_info.description,
        "minimal and full must have different descriptions"
    );
}

#[test]
fn context_preview_ui_stale_index_returns_none() {
    let mut state = make_preview_test_state();
    // Set preview to an out-of-bounds index
    state.context_preview_index = Some(99);
    assert!(
        state.active_preview().is_none(),
        "out-of-bounds index must return None"
    );
}

// =========================================================================
// End-to-end composer receipt tests
//
// Proves the full assembly → preflight → resolution → receipt pipeline
// with duplicate handling, partial failures, and sendability decisions.
//
// Run with: cargo test ai::window::tests -- --nocapture
// =========================================================================

/// A pending context chip plus an identical mention yields one attached part
/// and a receipt that records the duplicate provenance outcome.
#[test]
fn composer_receipt_pending_chip_plus_identical_mention_deduplicates() {
    crate::context_snapshot::enable_deterministic_context_capture();
    use crate::ai::context_contract::ContextAttachmentKind;
    use crate::ai::message_parts::{
        prepare_user_message_from_sources_with_receipt, ContextAssemblyOrigin,
        PreparedMessageDecision,
    };

    let current_part = ContextAttachmentKind::Current.part();

    // Mention has the same part as the pending chip
    let mention_parts = vec![current_part.clone()];
    let pending_parts = vec![current_part.clone()];

    let receipt = prepare_user_message_from_sources_with_receipt(
        "explain this",
        &mention_parts,
        &pending_parts,
        &[],
        &[],
    );

    let assembly = receipt
        .assembly
        .as_ref()
        .expect("assembly receipt must be present");

    // One merged part (the duplicate was removed)
    assert_eq!(
        assembly.merged_count, 1,
        "identical mention + pending chip must merge to 1 part"
    );
    assert_eq!(assembly.duplicates_removed, 1);
    assert_eq!(assembly.duplicates.len(), 1);
    assert_eq!(
        assembly.duplicates[0].kept_from,
        ContextAssemblyOrigin::Mention
    );
    assert_eq!(
        assembly.duplicates[0].dropped_from,
        ContextAssemblyOrigin::Pending
    );
    assert_eq!(
        assembly.duplicates[0].label,
        ContextAttachmentKind::Current.spec().label,
    );

    // The receipt is sendable (the MCP resource may fail to resolve in test
    // environment, but the assembly stage itself is valid)
    assert_ne!(
        receipt.decision,
        PreparedMessageDecision::Blocked,
        "duplicate dedup must not block the message"
    );

    // Emit JSON receipt snapshot for agent verification
    let snapshot = serde_json::json!({
        "test": "composer_receipt_pending_chip_plus_identical_mention_deduplicates",
        "assembly": {
            "mention_count": assembly.mention_count,
            "pending_count": assembly.pending_count,
            "merged_count": assembly.merged_count,
            "duplicates_removed": assembly.duplicates_removed,
        },
        "decision": format!("{:?}", receipt.decision),
        "sendable": receipt.can_send_message(),
    });
    println!(
        "--- RECEIPT_SNAPSHOT_JSON ---\n{}\n--- END_RECEIPT_SNAPSHOT_JSON ---",
        serde_json::to_string_pretty(&snapshot).expect("snapshot must serialize")
    );

    tracing::info!(
        checkpoint = "composer_receipt_dedup",
        merged_count = assembly.merged_count,
        duplicates_removed = assembly.duplicates_removed,
        decision = ?receipt.decision,
        "dedup composer receipt test passed"
    );
}

/// A mixed valid-invalid attachment set yields a partial receipt with at least
/// one successful attachment preserved and at least one failure surfaced.
#[test]
fn composer_receipt_mixed_valid_invalid_yields_partial_receipt() {
    use crate::ai::message_parts::{
        prepare_user_message_with_receipt, AiContextPart, PreparedMessageDecision,
    };

    let dir = tempfile::tempdir().expect("create temp dir");
    let good_file = dir.path().join("good.txt");
    std::fs::write(&good_file, "valid content").expect("write");

    let parts = vec![
        AiContextPart::FilePath {
            path: good_file.to_string_lossy().to_string(),
            label: "good.txt".to_string(),
        },
        AiContextPart::FilePath {
            path: "/nonexistent/path/bad.txt".to_string(),
            label: "bad.txt".to_string(),
        },
    ];

    let receipt = prepare_user_message_with_receipt("tell me about these", &parts, &[], &[]);

    assert_eq!(
        receipt.decision,
        PreparedMessageDecision::Partial,
        "mixed success/failure must yield Partial decision"
    );
    assert_eq!(receipt.context.attempted, 2);
    assert!(
        receipt.context.resolved >= 1,
        "at least one attachment must be preserved"
    );
    assert!(
        !receipt.context.failures.is_empty(),
        "at least one failure must be surfaced"
    );
    assert!(
        receipt.final_user_content.contains("valid content"),
        "successful attachment must appear in final content"
    );
    assert!(
        receipt.can_send_message(),
        "Partial decision must still be sendable"
    );
    assert!(
        receipt.user_error.is_some(),
        "failure must produce user-visible error"
    );

    // Emit JSON receipt snapshot
    let snapshot = serde_json::json!({
        "test": "composer_receipt_mixed_valid_invalid_yields_partial_receipt",
        "attempted": receipt.context.attempted,
        "resolved": receipt.context.resolved,
        "failure_count": receipt.context.failures.len(),
        "decision": format!("{:?}", receipt.decision),
        "sendable": receipt.can_send_message(),
    });
    println!(
        "--- RECEIPT_SNAPSHOT_JSON ---\n{}\n--- END_RECEIPT_SNAPSHOT_JSON ---",
        serde_json::to_string_pretty(&snapshot).expect("snapshot must serialize")
    );

    tracing::info!(
        checkpoint = "composer_receipt_partial",
        attempted = receipt.context.attempted,
        resolved = receipt.context.resolved,
        failures = receipt.context.failures.len(),
        decision = ?receipt.decision,
        "partial receipt test passed"
    );
}

/// An empty typed message with valid context parts still produces sendable
/// final user content (the resolved prefix becomes the entire message).
#[test]
fn composer_receipt_empty_message_with_valid_parts_is_sendable() {
    use crate::ai::message_parts::{
        prepare_user_message_with_receipt, AiContextPart, PreparedMessageDecision,
    };

    let dir = tempfile::tempdir().expect("create temp dir");
    let file_path = dir.path().join("context.txt");
    std::fs::write(&file_path, "ambient context data").expect("write");

    let parts = vec![AiContextPart::FilePath {
        path: file_path.to_string_lossy().to_string(),
        label: "context.txt".to_string(),
    }];

    let receipt = prepare_user_message_with_receipt("", &parts, &[], &[]);

    assert_eq!(receipt.decision, PreparedMessageDecision::Ready);
    assert!(
        receipt.can_send_message(),
        "empty text + valid parts must be sendable"
    );
    assert!(
        !receipt.final_user_content.is_empty(),
        "final content must be non-empty when parts resolve"
    );
    assert!(
        receipt.final_user_content.contains("ambient context data"),
        "resolved content must appear in final message"
    );
    assert_eq!(
        receipt.raw_content, "",
        "raw content must reflect the empty typed message"
    );

    // Emit JSON receipt snapshot
    let snapshot = serde_json::json!({
        "test": "composer_receipt_empty_message_with_valid_parts_is_sendable",
        "decision": format!("{:?}", receipt.decision),
        "sendable": receipt.can_send_message(),
        "final_content_len": receipt.final_user_content.len(),
        "raw_content_empty": receipt.raw_content.is_empty(),
    });
    println!(
        "--- RECEIPT_SNAPSHOT_JSON ---\n{}\n--- END_RECEIPT_SNAPSHOT_JSON ---",
        serde_json::to_string_pretty(&snapshot).expect("snapshot must serialize")
    );

    tracing::info!(
        checkpoint = "composer_receipt_empty_text_sendable",
        final_content_len = receipt.final_user_content.len(),
        decision = ?receipt.decision,
        "empty message sendability test passed"
    );
}

/// Diagnostics attachments remain machine-readable and do not block unrelated
/// healthy file-path parts.
#[test]
fn composer_receipt_diagnostics_does_not_block_healthy_parts() {
    crate::context_snapshot::enable_deterministic_context_capture();
    use crate::ai::context_contract::ContextAttachmentKind;
    use crate::ai::message_parts::{prepare_user_message_with_receipt, AiContextPart};

    let dir = tempfile::tempdir().expect("create temp dir");
    let good_file = dir.path().join("healthy.txt");
    std::fs::write(&good_file, "healthy content").expect("write");

    let diagnostics_part = ContextAttachmentKind::Diagnostics.part();

    let parts = vec![
        AiContextPart::FilePath {
            path: good_file.to_string_lossy().to_string(),
            label: "healthy.txt".to_string(),
        },
        diagnostics_part,
    ];

    let receipt = prepare_user_message_with_receipt("check this", &parts, &[], &[]);

    // The healthy file must be resolved regardless of diagnostics outcome
    assert!(
        receipt.context.resolved >= 1,
        "healthy part must resolve even when diagnostics is present"
    );
    assert!(
        receipt.final_user_content.contains("healthy content"),
        "healthy part content must appear in final message"
    );
    assert!(
        receipt.can_send_message(),
        "diagnostics must not block sendability when healthy parts exist"
    );

    // Verify diagnostics outcome is machine-readable
    let diag_outcome = receipt
        .outcomes
        .iter()
        .find(|o| o.label == ContextAttachmentKind::Diagnostics.spec().label);
    assert!(
        diag_outcome.is_some(),
        "diagnostics outcome must be present in outcomes list"
    );

    // Emit JSON receipt snapshot
    let snapshot = serde_json::json!({
        "test": "composer_receipt_diagnostics_does_not_block_healthy_parts",
        "attempted": receipt.context.attempted,
        "resolved": receipt.context.resolved,
        "failure_count": receipt.context.failures.len(),
        "decision": format!("{:?}", receipt.decision),
        "sendable": receipt.can_send_message(),
        "diagnostics_outcome_kind": diag_outcome.map(|o| format!("{:?}", o.kind)),
    });
    println!(
        "--- RECEIPT_SNAPSHOT_JSON ---\n{}\n--- END_RECEIPT_SNAPSHOT_JSON ---",
        serde_json::to_string_pretty(&snapshot).expect("snapshot must serialize")
    );

    tracing::info!(
        checkpoint = "composer_receipt_diagnostics_non_blocking",
        resolved = receipt.context.resolved,
        decision = ?receipt.decision,
        "diagnostics non-blocking test passed"
    );
}

/// Visible attachment labels in tests come from the canonical attachment spec
/// helper rather than duplicated string literals.
#[test]
fn composer_receipt_labels_match_canonical_spec() {
    use crate::ai::context_contract::{context_attachment_specs, ContextAttachmentKind};

    let all_kinds = [
        ContextAttachmentKind::Current,
        ContextAttachmentKind::Full,
        ContextAttachmentKind::Selection,
        ContextAttachmentKind::Browser,
        ContextAttachmentKind::Window,
        ContextAttachmentKind::Diagnostics,
    ];

    let specs = context_attachment_specs();

    for kind in &all_kinds {
        let part = kind.part();
        let spec = kind.spec();

        // The part's label must match the canonical spec label
        assert_eq!(
            part.label(),
            spec.label,
            "part label for {:?} must match canonical spec",
            kind
        );
        // The part's source URI must match the canonical spec URI
        assert_eq!(
            part.source(),
            spec.uri,
            "part source for {:?} must match canonical spec",
            kind
        );

        // The spec must exist in the global specs table
        assert!(
            specs.iter().any(|s| s.kind == *kind),
            "{:?} must be present in context_attachment_specs()",
            kind
        );
    }

    // Emit JSON receipt for agent verification
    let label_matrix: Vec<serde_json::Value> = all_kinds
        .iter()
        .map(|kind| {
            serde_json::json!({
                "kind": format!("{:?}", kind),
                "label": kind.spec().label,
                "uri": kind.spec().uri,
            })
        })
        .collect();

    let snapshot = serde_json::json!({
        "test": "composer_receipt_labels_match_canonical_spec",
        "label_matrix": label_matrix,
    });
    println!(
        "--- RECEIPT_SNAPSHOT_JSON ---\n{}\n--- END_RECEIPT_SNAPSHOT_JSON ---",
        serde_json::to_string_pretty(&snapshot).expect("snapshot must serialize")
    );

    tracing::info!(
        total_kinds = all_kinds.len(),
        "canonical label verification complete"
    );
}

// ============================================================================
// Context Palette wiring tests
// ============================================================================

/// Cmd+Shift+A shortcut detection is correct.
#[test]
fn test_context_palette_shortcut_requires_cmd_shift_a() {
    let correct = crate::ai::window::render_keydown::is_context_palette_shortcut(
        "a",
        &gpui::Modifiers {
            platform: true,
            shift: true,
            ..Default::default()
        },
    );
    assert!(
        correct,
        "Cmd+Shift+A should match the context palette shortcut"
    );

    let wrong_key = crate::ai::window::render_keydown::is_context_palette_shortcut(
        "b",
        &gpui::Modifiers {
            platform: true,
            shift: true,
            ..Default::default()
        },
    );
    assert!(!wrong_key, "Cmd+Shift+B must not match");

    let missing_shift = crate::ai::window::render_keydown::is_context_palette_shortcut(
        "a",
        &gpui::Modifiers {
            platform: true,
            ..Default::default()
        },
    );
    assert!(!missing_shift, "Cmd+A without Shift must not match");

    let extra_alt = crate::ai::window::render_keydown::is_context_palette_shortcut(
        "a",
        &gpui::Modifiers {
            platform: true,
            shift: true,
            alt: true,
            ..Default::default()
        },
    );
    assert!(!extra_alt, "Cmd+Shift+Alt+A must not match");
}

/// Picker items from all entry points (slash, mention, palette) produce identical parts.
#[test]
fn test_palette_slash_mention_produce_identical_parts() {
    use crate::ai::context_contract::{context_attachment_specs, ContextAttachmentKind};
    use crate::ai::message_parts::AiContextPart;
    use crate::ai::window::context_picker::{build_picker_items, types::ContextPickerItemKind};

    for spec in context_attachment_specs() {
        // Part from canonical contract
        let canonical_part = spec.kind.part();

        // Part from picker item
        let items = build_picker_items("");
        let picker_item = items
            .iter()
            .find(|i| matches!(&i.kind, ContextPickerItemKind::BuiltIn(k) if *k == spec.kind))
            .unwrap_or_else(|| panic!("picker should contain {:?}", spec.kind));
        let picker_part = match &picker_item.kind {
            ContextPickerItemKind::BuiltIn(k) => k.part(),
            _ => unreachable!(),
        };

        // Part from slash command (if available)
        if let Some(slash) = spec.slash_command {
            let slash_kind = ContextAttachmentKind::from_slash_command(slash)
                .unwrap_or_else(|| panic!("slash command {slash} should parse"));
            let slash_part = slash_kind.part();
            assert_eq!(
                canonical_part, slash_part,
                "Slash command part for {:?} must match canonical",
                spec.kind,
            );
        }

        // Part from mention (if available)
        if let Some(mention) = spec.mention {
            let mention_kind = ContextAttachmentKind::from_mention_line(mention)
                .unwrap_or_else(|| panic!("mention {mention} should parse"));
            let mention_part = mention_kind.part();
            assert_eq!(
                canonical_part, mention_part,
                "Mention part for {:?} must match canonical",
                spec.kind,
            );
        }

        assert_eq!(
            canonical_part, picker_part,
            "Picker part for {:?} must match canonical",
            spec.kind,
        );

        // Verify label consistency: chip label == spec label
        assert_eq!(
            canonical_part.label(),
            spec.label,
            "Part label for {:?} must match spec label",
            spec.kind,
        );
    }
}

/// Picker snapshot is machine-readable and matches actual state.
#[test]
fn test_picker_snapshot_is_serializable_and_consistent() {
    use crate::ai::window::context_picker::{build_picker_items, types::ContextPickerState};

    let items = build_picker_items("sel");
    let mut state = ContextPickerState::new("sel".to_string(), items);
    state.selected_index = 0;

    let snapshot = state.snapshot();

    // Must serialize to JSON
    let json = serde_json::to_string_pretty(&snapshot).expect("snapshot must serialize");
    assert!(
        json.contains("\"query\": \"sel\""),
        "snapshot must contain query"
    );
    assert!(
        json.contains("\"selected_index\": 0"),
        "snapshot must contain selected_index"
    );

    // Items must match
    assert_eq!(
        snapshot.items.len(),
        state.items.len(),
        "snapshot item count must match state"
    );

    // First item should be Selection (highest score for "sel" query)
    assert!(
        snapshot.items[0].label.contains("Selection"),
        "first snapshot item for 'sel' query should be Selection, got: {}",
        snapshot.items[0].label,
    );

    println!("--- PICKER_SNAPSHOT_JSON ---\n{json}\n--- END_PICKER_SNAPSHOT_JSON ---");
}

/// Preflight snapshot is machine-readable.
#[test]
fn test_preflight_snapshot_is_serializable() {
    use crate::ai::window::context_preflight::{
        preflight_state_from_receipt, ContextPreflightStatus,
    };

    let receipt = crate::ai::message_parts::PreparedMessageReceipt {
        schema_version: crate::ai::message_parts::AI_MESSAGE_PREPARATION_SCHEMA_VERSION,
        decision: crate::ai::message_parts::PreparedMessageDecision::Ready,
        raw_content: "test".to_string(),
        final_user_content: "prefix\n\ntest".to_string(),
        context: crate::ai::message_parts::ContextResolutionReceipt {
            attempted: 2,
            resolved: 2,
            failures: vec![],
            prompt_prefix: "some resolved content".to_string(),
        },
        assembly: None,
        outcomes: vec![],
        unresolved_parts: vec![],
        user_error: None,
    };

    let state = preflight_state_from_receipt(7, receipt);
    let snapshot = state.snapshot();

    let json = serde_json::to_string_pretty(&snapshot).expect("preflight snapshot must serialize");
    assert!(
        json.contains("\"generation\": 7"),
        "must contain generation"
    );
    assert!(json.contains("\"Ready\""), "must contain Ready status");
    assert!(json.contains("\"attempted\": 2"), "must contain attempted");
    assert!(json.contains("\"resolved\": 2"), "must contain resolved");

    println!("--- PREFLIGHT_SNAPSHOT_JSON ---\n{json}\n--- END_PREFLIGHT_SNAPSHOT_JSON ---");
}

/// The send button is enabled when context parts are attached (no text needed).
#[test]
fn test_send_button_enabled_by_context_parts_alone() {
    assert!(
        ai_window_can_submit_message("", false, true),
        "Context-parts-only should enable send button"
    );
    assert!(
        ai_window_can_submit_message("hello", false, true),
        "Text + context parts should enable send button"
    );
    assert!(
        !ai_window_can_submit_message("", false, false),
        "Empty message without parts should disable send button"
    );
}

/// End-to-end: picker selection → pending part → preflight receipt → valid content.
#[test]
fn test_palette_to_preflight_to_send_end_to_end() {
    crate::context_snapshot::enable_deterministic_context_capture();
    use crate::ai::context_contract::ContextAttachmentKind;
    use crate::ai::message_parts::{prepare_user_message_with_receipt, PreparedMessageDecision};
    use crate::ai::window::context_picker::{build_picker_items, types::ContextPickerItemKind};
    use crate::ai::window::context_preflight::preflight_state_from_receipt;

    // Step 1: User opens palette, selects "Current Context"
    let items = build_picker_items("");
    let current_item = items
        .iter()
        .find(|i| {
            matches!(
                &i.kind,
                ContextPickerItemKind::BuiltIn(ContextAttachmentKind::Current)
            )
        })
        .expect("palette should contain Current Context");

    // Step 2: Accept creates a pending part
    let part = match &current_item.kind {
        ContextPickerItemKind::BuiltIn(kind) => kind.part(),
        _ => unreachable!(),
    };

    // Step 3: Run preflight with the pending part
    let receipt = prepare_user_message_with_receipt("What do you see?", &[part], &[], &[]);

    assert_eq!(
        receipt.decision,
        PreparedMessageDecision::Ready,
        "kit://context URI should resolve; got {:?}",
        receipt.decision,
    );
    assert_eq!(receipt.context.attempted, 1);
    assert_eq!(receipt.context.resolved, 1);
    assert!(
        receipt.final_user_content.contains("kit://context"),
        "final content should contain resolved context"
    );
    assert!(
        receipt.final_user_content.contains("What do you see?"),
        "final content should preserve user text"
    );

    // Step 4: Verify preflight state derivation
    let preflight = preflight_state_from_receipt(1, receipt);
    assert_eq!(
        preflight.status,
        crate::ai::window::context_preflight::ContextPreflightStatus::Ready
    );
    assert!(
        preflight.approx_tokens > 0,
        "resolved context should have tokens"
    );

    // Step 5: Verify send button would be enabled
    assert!(ai_window_can_submit_message(
        "What do you see?",
        false,
        true
    ));
}

/// Duplicate attachment via palette + mention deduplicates correctly.
#[test]
fn test_palette_plus_mention_deduplication() {
    crate::context_snapshot::enable_deterministic_context_capture();
    use crate::ai::context_contract::ContextAttachmentKind;
    use crate::ai::context_mentions::parse_context_mentions;
    use crate::ai::message_parts::{
        merge_context_parts_with_receipt, prepare_user_message_with_receipt,
        PreparedMessageDecision,
    };

    // User added Selection via palette (pending part)
    let palette_part = ContextAttachmentKind::Selection.part();

    // User also typed @selection in the message
    let parsed = parse_context_mentions("@selection\nWhat is selected?");
    assert_eq!(parsed.parts.len(), 1, "mention should parse");

    // Merge should deduplicate
    let assembly =
        merge_context_parts_with_receipt(&parsed.parts, std::slice::from_ref(&palette_part));
    assert_eq!(
        assembly.duplicates_removed, 1,
        "identical parts from mention + palette should dedup"
    );
    assert_eq!(
        assembly.merged_count, 1,
        "should have exactly one unique part"
    );

    // Full pipeline should still work
    let receipt = prepare_user_message_with_receipt(
        &parsed.cleaned_content,
        &assembly.merged_parts,
        &[],
        &[],
    );
    assert_eq!(receipt.decision, PreparedMessageDecision::Ready);
}

/// Mixed valid and invalid context parts yield Partial, not silent loss.
#[test]
fn test_mixed_valid_invalid_parts_yield_partial() {
    crate::context_snapshot::enable_deterministic_context_capture();
    use crate::ai::context_contract::ContextAttachmentKind;
    use crate::ai::message_parts::{
        prepare_user_message_with_receipt, AiContextPart, PreparedMessageDecision,
    };

    let valid_part = ContextAttachmentKind::Current.part();
    let invalid_part = AiContextPart::FilePath {
        path: "/nonexistent/path/that/does/not/exist/ever.txt".to_string(),
        label: "Missing File".to_string(),
    };

    let receipt = prepare_user_message_with_receipt(
        "Show me everything",
        &[valid_part, invalid_part],
        &[],
        &[],
    );

    assert_eq!(
        receipt.decision,
        PreparedMessageDecision::Partial,
        "mixed valid+invalid should be Partial, not Ready or Blocked"
    );
    assert_eq!(receipt.context.resolved, 1, "valid part should resolve");
    assert_eq!(
        receipt.context.failures.len(),
        1,
        "invalid part should fail"
    );
    assert!(
        receipt.final_user_content.contains("kit://context"),
        "valid part content should be present"
    );
}

// ---------------------------------------------------------------------------
// Mini mode tests
// ---------------------------------------------------------------------------

#[test]
fn test_mini_mode_reports_as_mini() {
    assert!(AiWindowMode::Mini.is_mini());
    assert!(!AiWindowMode::Full.is_mini());
}

#[test]
fn test_mini_mode_defaults_to_full() {
    assert_eq!(AiWindowMode::default(), AiWindowMode::Full);
    assert!(!AiWindowMode::default().is_mini());
}

#[test]
fn test_mini_mode_dimensions_differ_from_full() {
    let full = AiWindowMode::Full;
    let mini = AiWindowMode::Mini;
    assert!(
        mini.default_width() < full.default_width(),
        "Mini should be narrower than Full"
    );
    assert!(
        mini.default_height() < full.default_height(),
        "Mini should be shorter than Full"
    );
}

#[test]
fn test_mini_mode_has_distinct_title() {
    assert_ne!(
        AiWindowMode::Full.title(),
        AiWindowMode::Mini.title(),
        "Mini and Full should have different window titles"
    );
}

#[test]
fn test_window_role_for_mode_maps_correctly() {
    use crate::window_state::WindowRole;

    let full_role = super::window_api::window_role_for_mode(AiWindowMode::Full);
    let mini_role = super::window_api::window_role_for_mode(AiWindowMode::Mini);

    assert_eq!(full_role, WindowRole::Ai);
    assert_eq!(mini_role, WindowRole::AiMini);
}

// ---------------------------------------------------------------------------
// Mini AI Window — Source-audit regression tests
// These verify the mini interaction contract at the source level so refactors
// don't silently break the Esc chain, Cmd+N routing, or overlay focus.
// ---------------------------------------------------------------------------

/// The Esc chain in render_keydown must close the mini history overlay
/// BEFORE the final mini-close handler. If the overlay handler appears after
/// the close handler, pressing Esc with the overlay open would close the
/// entire window instead of just dismissing the overlay.
#[test]
fn test_mini_esc_overlay_precedes_close_in_source() {
    let source = include_str!("render_keydown.rs");
    let overlay_pos = source
        .find("showing_mini_history_overlay")
        .expect("mini history overlay Esc handler must exist in render_keydown.rs");
    let close_pos = source
        .find("mini_escape_close")
        .expect("mini Esc close handler must exist in render_keydown.rs");
    assert!(
        overlay_pos < close_pos,
        "Mini history overlay Esc handler (byte {overlay_pos}) must appear \
         before the mini close handler (byte {close_pos}) in render_keydown.rs"
    );
}

/// Cmd+N in mini mode must route to `show_new_chat_command_bar` (model/preset
/// picker) rather than `new_conversation` (blank chat). This source audit
/// ensures the mini branch exists in the "n" match arm.
#[test]
fn test_mini_cmd_n_routes_to_new_chat_command_bar() {
    let source = include_str!("render_keydown.rs");
    let n_arm = source
        .find("\"n\" => {")
        .expect("Cmd+N handler must exist in render_keydown.rs");
    let after_n = &source[n_arm..];
    let mini_branch = after_n
        .find("window_mode.is_mini()")
        .expect("Mini mode branch must exist in Cmd+N handler");
    let new_chat_bar = after_n
        .find("show_new_chat_command_bar")
        .expect("show_new_chat_command_bar must be called in Cmd+N handler");
    assert!(
        mini_branch < new_chat_bar,
        "Mini mode check must precede show_new_chat_command_bar call"
    );
}

/// The mini header's "New" button must call `show_new_chat_command_bar`
/// (not `new_conversation`) for consistency with Cmd+N.
#[test]
fn test_mini_header_new_button_uses_command_bar() {
    let source = include_str!("render_root.rs");
    let mini_new = source
        .find("ai-mini-new")
        .expect("Mini header New button must have id 'ai-mini-new'");
    // Scope to the New button region — bounded by the next button element
    let after = &source[mini_new..];
    let region_end = after.find("ai-mini-actions").unwrap_or(after.len());
    let new_button_region = &after[..region_end];
    assert!(
        new_button_region.contains("show_new_chat_command_bar"),
        "Mini New button must call show_new_chat_command_bar, not new_conversation"
    );
}

/// The mini history overlay toggle must call `focus_search` when opening,
/// so typing immediately filters chats without an extra click.
#[test]
fn test_mini_history_overlay_focuses_search_on_open() {
    let source = include_str!("render_root.rs");
    let toggle_fn = source
        .find("fn toggle_mini_history_overlay")
        .expect("toggle_mini_history_overlay must exist in render_root.rs");
    // Search from the function definition to the next function boundary
    let after = &source[toggle_fn..];
    assert!(
        after.contains("focus_search"),
        "toggle_mini_history_overlay must call focus_search when opening"
    );
}

/// The canonical set_window_mode helper must save bounds before switching,
/// ensuring the user's custom window size is preserved per mode.
/// toggle_window_mode and SetWindowMode both delegate to it.
#[test]
fn test_set_window_mode_saves_bounds_before_switch() {
    let source = include_str!("interactions.rs");
    let set_fn = source
        .find("fn set_window_mode")
        .expect("set_window_mode must exist in interactions.rs");
    let after = &source[set_fn..];
    let save_pos = after
        .find("save_window_from_gpui")
        .expect("Must save bounds before mode switch");
    let mode_assign = after
        .find("self.window_mode = new_mode")
        .expect("Must assign new mode");
    assert!(
        save_pos < mode_assign,
        "Bounds must be saved (byte +{save_pos}) before mode assignment (byte +{mode_assign})"
    );

    // toggle_window_mode must delegate to set_window_mode
    let toggle_fn = source
        .find("fn toggle_window_mode")
        .expect("toggle_window_mode must exist");
    let toggle_end = (toggle_fn + 500).min(source.len());
    let toggle_body = &source[toggle_fn..toggle_end];
    assert!(
        toggle_body.contains("self.set_window_mode("),
        "toggle_window_mode must delegate to set_window_mode"
    );

    // Command bar must delegate to toggle_window_mode
    let cmd_source = include_str!("command_bar.rs");
    let cmd_section = cmd_source
        .find("\"toggle_window_mode\"")
        .expect("toggle_window_mode action must exist in command_bar.rs");
    let after_cmd = &cmd_source[cmd_section..];
    assert!(
        after_cmd.contains("toggle_window_mode(window, cx)"),
        "Command bar must delegate to toggle_window_mode method"
    );
}

/// SetWindowMode command (via stdin) must delegate to set_window_mode helper.
#[test]
fn test_set_window_mode_command_delegates() {
    let source = include_str!("render_root.rs");
    let cmd_section = source
        .find("AiCommand::SetWindowMode")
        .expect("SetWindowMode command handler must exist in render_root.rs");
    let handler_end = (cmd_section + 200).min(source.len());
    let handler_body = &source[cmd_section..handler_end];
    assert!(
        handler_body.contains("self.set_window_mode("),
        "SetWindowMode command handler must delegate to set_window_mode"
    );
}

/// Simulated key input must support Cmd+Shift+M so stdin automation and tests
/// can trigger the same mode toggle path as real keyboard events.
#[test]
fn test_simulated_key_supports_mode_toggle_shortcut() {
    let source = include_str!("command_bar.rs");
    let handler_section = source
        .find("fn handle_simulated_key")
        .expect("handle_simulated_key must exist in command_bar.rs");
    let after = &source[handler_section..];
    assert!(
        after.contains("modifiers.contains(&KeyModifier::Shift) && key_lower == \"m\""),
        "Simulated key handler must recognize Cmd+Shift+M"
    );
    assert!(
        after.contains("self.toggle_window_mode(window, cx);"),
        "Simulated key handler must delegate Cmd+Shift+M to toggle_window_mode"
    );
}
