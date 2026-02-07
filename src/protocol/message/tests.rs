use super::*;

/// Test: Chat message with useBuiltinAi flag should be parsed correctly
///
/// When SDK sends chat with `useBuiltinAi: true`, the app should use
/// its built-in AI providers instead of relying on SDK callbacks.
#[test]
fn test_chat_message_with_use_builtin_ai() {
    let json = r#"{
            "type": "chat",
            "id": "chat-1",
            "placeholder": "Ask a question...",
            "messages": [{"role": "user", "content": "Hello"}],
            "hint": "AI Chat",
            "useBuiltinAi": true
        }"#;

    let msg: Message = serde_json::from_str(json).expect("Should parse chat message");

    match msg {
        Message::Chat {
            id,
            placeholder,
            use_builtin_ai,
            ..
        } => {
            assert_eq!(id, "chat-1");
            assert_eq!(placeholder, Some("Ask a question...".to_string()));
            assert!(use_builtin_ai, "useBuiltinAi should be true");
        }
        _ => panic!("Expected Chat message"),
    }
}

/// Test: Chat message without useBuiltinAi should default to false
#[test]
fn test_chat_message_without_use_builtin_ai_defaults_to_false() {
    let json = r#"{
            "type": "chat",
            "id": "chat-2",
            "messages": []
        }"#;

    let msg: Message = serde_json::from_str(json).expect("Should parse chat message");

    match msg {
        Message::Chat { use_builtin_ai, .. } => {
            assert!(!use_builtin_ai, "useBuiltinAi should default to false");
        }
        _ => panic!("Expected Chat message"),
    }
}
