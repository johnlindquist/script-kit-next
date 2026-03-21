//! Protocol serde tests for the `parts` field on AI messages.
//!
//! Validates that:
//! - Existing payloads without `parts` still deserialize unchanged.
//! - Explicit `parts` values round-trip correctly.
//! - The `AiContextPartInput` type is wire-compatible with `AiContextPart`.

use script_kit_gpui::protocol::Message;

// ---------- Wire compatibility: omission of `parts` ----------

#[test]
fn protocol_ai_start_chat_without_parts_still_deserializes() {
    let json = serde_json::json!({
        "type": "aiStartChat",
        "requestId": "req-001",
        "message": "Hello, AI!",
    });

    let msg: Message = serde_json::from_value(json).expect("should deserialize without parts");
    let reserialized = serde_json::to_value(&msg).expect("should reserialize");

    // `parts` should not appear in output when empty
    assert!(
        reserialized.get("parts").is_none(),
        "empty parts should be skipped in serialization"
    );
}

#[test]
fn protocol_ai_send_message_without_parts_still_deserializes() {
    let json = serde_json::json!({
        "type": "aiSendMessage",
        "requestId": "req-002",
        "chatId": "chat-abc",
        "content": "Follow up question",
    });

    let msg: Message = serde_json::from_value(json).expect("should deserialize without parts");
    let reserialized = serde_json::to_value(&msg).expect("should reserialize");

    assert!(
        reserialized.get("parts").is_none(),
        "empty parts should be skipped in serialization"
    );
}

// ---------- Explicit parts round-trip ----------

#[test]
fn protocol_ai_start_chat_with_resource_uri_part_round_trips() {
    let json = serde_json::json!({
        "type": "aiStartChat",
        "requestId": "req-003",
        "message": "What's on my screen?",
        "parts": [
            {
                "kind": "resourceUri",
                "uri": "kit://context?profile=minimal",
                "label": "Current Context"
            }
        ]
    });

    let msg: Message = serde_json::from_value(json.clone()).expect("should deserialize with parts");
    let reserialized = serde_json::to_value(&msg).expect("should reserialize");

    let parts = reserialized
        .get("parts")
        .expect("parts should be present when non-empty");
    assert!(parts.is_array());
    assert_eq!(parts.as_array().expect("array").len(), 1);

    let part = &parts[0];
    assert_eq!(part["kind"], "resourceUri");
    assert_eq!(part["uri"], "kit://context?profile=minimal");
    assert_eq!(part["label"], "Current Context");
}

#[test]
fn protocol_ai_send_message_with_file_path_part_round_trips() {
    let json = serde_json::json!({
        "type": "aiSendMessage",
        "requestId": "req-004",
        "chatId": "chat-xyz",
        "content": "Review this file",
        "parts": [
            {
                "kind": "filePath",
                "path": "/tmp/example.rs",
                "label": "example.rs"
            }
        ]
    });

    let msg: Message = serde_json::from_value(json).expect("should deserialize with file part");
    let reserialized = serde_json::to_value(&msg).expect("should reserialize");

    let parts = reserialized.get("parts").expect("parts should be present");
    assert_eq!(parts.as_array().expect("array").len(), 1);

    let part = &parts[0];
    assert_eq!(part["kind"], "filePath");
    assert_eq!(part["path"], "/tmp/example.rs");
    assert_eq!(part["label"], "example.rs");
}

#[test]
fn protocol_ai_start_chat_with_mixed_parts_round_trips() {
    let json = serde_json::json!({
        "type": "aiStartChat",
        "requestId": "req-005",
        "message": "Analyze",
        "parts": [
            {
                "kind": "resourceUri",
                "uri": "kit://context",
                "label": "Full Context"
            },
            {
                "kind": "filePath",
                "path": "/tmp/data.json",
                "label": "data.json"
            }
        ]
    });

    let msg: Message = serde_json::from_value(json).expect("should deserialize mixed parts");
    let reserialized = serde_json::to_value(&msg).expect("should reserialize");

    let parts = reserialized.get("parts").expect("parts should be present");
    let parts_array = parts.as_array().expect("array");
    assert_eq!(parts_array.len(), 2);
    assert_eq!(parts_array[0]["kind"], "resourceUri");
    assert_eq!(parts_array[1]["kind"], "filePath");
}

// ---------- Existing fields preserved alongside parts ----------

#[test]
fn protocol_ai_start_chat_with_parts_preserves_existing_fields() {
    let json = serde_json::json!({
        "type": "aiStartChat",
        "requestId": "req-006",
        "message": "Look at this",
        "systemPrompt": "You are a code reviewer.",
        "image": "base64-image-data",
        "modelId": "claude-3-5-sonnet-20241022",
        "noResponse": true,
        "parts": [
            {
                "kind": "resourceUri",
                "uri": "kit://context?selectedText=1",
                "label": "Selection"
            }
        ]
    });

    let msg: Message = serde_json::from_value(json).expect("should deserialize all fields");
    let reserialized = serde_json::to_value(&msg).expect("should reserialize");

    assert_eq!(reserialized["message"], "Look at this");
    assert_eq!(reserialized["systemPrompt"], "You are a code reviewer.");
    assert_eq!(reserialized["image"], "base64-image-data");
    assert_eq!(reserialized["modelId"], "claude-3-5-sonnet-20241022");
    assert_eq!(reserialized["noResponse"], true);
    assert_eq!(
        reserialized["parts"][0]["uri"],
        "kit://context?selectedText=1"
    );
}
