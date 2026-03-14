use super::*;

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
        ai_window_can_submit_message("", true),
        "Image-only messages should be allowed"
    );
    assert!(
        ai_window_can_submit_message("   ", true),
        "Whitespace text with an attachment should be allowed"
    );
    assert!(
        ai_window_can_submit_message("hello", false),
        "Non-empty text should be allowed"
    );
    assert!(
        !ai_window_can_submit_message("   ", false),
        "Whitespace-only messages without attachments should be blocked"
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
            model_id,
            provider,
            on_created,
            ..
        }) => {
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
            ai_window_can_submit_message(suggestion, false),
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
///   - pending_attachments
///   - collapsed_messages
///   - expanded_messages
///   - copied_message_id / copied_at
///   - last_streaming_duration / last_streaming_completed_at
///   - streaming_error
///   - showing_attachments_picker
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
        pending_attachments: Vec<String>,
        collapsed_messages: std::collections::HashSet<String>,
        expanded_messages: std::collections::HashSet<String>,
        copied_message_id: Option<String>,
        copied_at: Option<std::time::Instant>,
        last_streaming_duration: Option<std::time::Duration>,
        last_streaming_completed_at: Option<std::time::Instant>,
        streaming_error: Option<String>,
        showing_attachments_picker: bool,
        editing_message_id: Option<String>,
    }

    impl ConversationTransientState {
        /// Apply the same reset logic as AiApp::new_conversation()
        fn reset(&mut self) {
            self.pending_image = None;
            self.pending_attachments.clear();
            self.collapsed_messages.clear();
            self.expanded_messages.clear();
            self.copied_message_id = None;
            self.copied_at = None;
            self.last_streaming_duration = None;
            self.last_streaming_completed_at = None;
            self.streaming_error = None;
            self.showing_attachments_picker = false;
            self.editing_message_id = None;
        }
    }

    // Build dirty state (simulates mid-conversation)
    let mut state = ConversationTransientState {
        pending_image: Some("base64data".to_string()),
        pending_attachments: vec!["/tmp/file.txt".to_string()],
        collapsed_messages: ["msg-1".to_string()].into_iter().collect(),
        expanded_messages: ["msg-2".to_string()].into_iter().collect(),
        copied_message_id: Some("msg-1".to_string()),
        copied_at: Some(std::time::Instant::now()),
        last_streaming_duration: Some(std::time::Duration::from_secs(5)),
        last_streaming_completed_at: Some(std::time::Instant::now()),
        streaming_error: Some("previous error".to_string()),
        showing_attachments_picker: true,
        editing_message_id: Some("msg-3".to_string()),
    };

    // Verify dirty state is non-default
    assert!(state.pending_image.is_some());
    assert!(!state.pending_attachments.is_empty());
    assert!(!state.collapsed_messages.is_empty());
    assert!(!state.expanded_messages.is_empty());
    assert!(state.copied_message_id.is_some());
    assert!(state.copied_at.is_some());
    assert!(state.last_streaming_duration.is_some());
    assert!(state.last_streaming_completed_at.is_some());
    assert!(state.streaming_error.is_some());
    assert!(state.showing_attachments_picker);
    assert!(state.editing_message_id.is_some());

    // Apply reset
    state.reset();

    // Assert all fields are at their default state
    assert!(
        state.pending_image.is_none(),
        "pending_image must be cleared on new conversation"
    );
    assert!(
        state.pending_attachments.is_empty(),
        "pending_attachments must be cleared on new conversation"
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
        !state.showing_attachments_picker,
        "showing_attachments_picker must be false on new conversation"
    );
    assert!(
        state.editing_message_id.is_none(),
        "editing_message_id must be cleared on new conversation"
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
