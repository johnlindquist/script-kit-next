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
fn test_message_body_content_does_not_truncate_long_messages() {
    let long_message = "lorem ipsum ".repeat(200);
    let display_content = AiApp::message_body_content(&long_message);
    assert_eq!(display_content, long_message);
}
