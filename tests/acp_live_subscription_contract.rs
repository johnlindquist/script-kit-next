// @lat: [[tests#ACP Chat#Live subscription runtime]]

fn read(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn protocol_unsubscribe_carries_subscription_id() {
    let source = read("src/protocol/message/variants/ai.rs");
    let unsubscribe = source
        .split("AiUnsubscribe {")
        .nth(1)
        .and_then(|tail| tail.split("AiUnsubscribed {").next())
        .expect("AiUnsubscribe block should exist");
    assert!(
        unsubscribe.contains("subscription_id: String"),
        "aiUnsubscribe must identify the app-side subscription to remove"
    );

    let sdk = read("scripts/kit-sdk.ts");
    assert!(
        sdk.contains("subscriptionId: string;"),
        "SDK unsubscribe wire shape must include subscriptionId"
    );
    assert!(
        sdk.contains("subscriptionId,"),
        "unsubscribe closure must send the captured subscriptionId"
    );
}

#[test]
fn script_reader_owns_subscribe_unsubscribe_and_exit_cleanup() {
    let source = read("src/execute_script/mod.rs");
    let subscribe_idx = source
        .find("Message::AiSubscribe")
        .expect("execute_script should handle AiSubscribe directly");
    let direct_idx = source
        .find("try_handle_ai_message")
        .expect("direct AI handler call should exist");
    assert!(
        subscribe_idx < direct_idx,
        "subscription handling needs the script response channel before stateless direct handlers"
    );
    assert!(
        source.contains("handle_subscribe(")
            && source.contains("handle_unsubscribe(")
            && source.contains("cleanup_owner(&subscription_owner_id)"),
        "script reader must register, unregister, and clean subscriptions on reader exit"
    );
}

#[test]
fn acp_thread_fans_out_stream_message_complete_and_error_events() {
    let source = read("src/ai/acp/thread.rs");
    for needle in [
        "publish_new_message(",
        "publish_stream_chunk(",
        "publish_stream_complete(",
        "publish_error(",
    ] {
        assert!(
            source.contains(needle),
            "AcpThread should fan out live event hook {needle}"
        );
    }
}

#[test]
fn sdk_dispatch_uses_subscription_id_before_event_type() {
    let source = read("scripts/kit-sdk.ts");
    assert!(
        source.contains("aiSubscriptions.get(event.subscriptionId)"),
        "SDK event dispatch should target the server-selected subscriptionId"
    );
    assert!(
        source.contains("!sub.chatId || sub.chatId === event.chatId"),
        "SDK event dispatch should preserve chat scoping defensively"
    );
}
