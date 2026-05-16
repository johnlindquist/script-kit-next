// @lat: [[tests#ACP Chat#Live subscription runtime]]

use script_kit_gpui::protocol::Message;

#[test]
fn ai_unsubscribe_requires_subscription_id_and_round_trips() {
    let json = serde_json::json!({
        "type": "aiUnsubscribe",
        "requestId": "req-unsub",
        "subscriptionId": "sub-1"
    });

    let msg: Message = serde_json::from_value(json).expect("should deserialize aiUnsubscribe");
    let reserialized = serde_json::to_value(&msg).expect("should reserialize");

    assert_eq!(reserialized["type"], "aiUnsubscribe");
    assert_eq!(reserialized["requestId"], "req-unsub");
    assert_eq!(reserialized["subscriptionId"], "sub-1");
}

#[test]
fn ai_unsubscribed_ack_reports_success_and_subscription_id() {
    let msg = Message::AiUnsubscribed {
        request_id: "req-ack".to_string(),
        subscription_id: "sub-1".to_string(),
        success: true,
        error: None,
    };

    let json = serde_json::to_value(&msg).expect("should serialize aiUnsubscribed");

    assert_eq!(json["type"], "aiUnsubscribed");
    assert_eq!(json["requestId"], "req-ack");
    assert_eq!(json["subscriptionId"], "sub-1");
    assert_eq!(json["success"], true);
    assert!(json.get("error").is_none());
}

#[test]
fn pushed_events_keep_subscription_and_chat_scope() {
    let msg = Message::AiStreamChunk {
        subscription_id: "sub-1".to_string(),
        chat_id: "chat-a".to_string(),
        chunk: "hi".to_string(),
        accumulated_content: "hi".to_string(),
    };

    let json = serde_json::to_value(&msg).expect("should serialize aiStreamChunk");

    assert_eq!(json["type"], "aiStreamChunk");
    assert_eq!(json["subscriptionId"], "sub-1");
    assert_eq!(json["chatId"], "chat-a");
}
