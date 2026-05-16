// doc-anchor-removed: [[tests#ACP Chat#Existing chat mutation runtime]]

fn read(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn mutation_messages_are_handled_by_direct_sdk_storage_path() {
    let source = read("src/ai/sdk_handlers.rs");
    let direct_dispatch = source
        .split("pub fn try_handle_ai_message")
        .nth(1)
        .expect("try_handle_ai_message should exist");

    for needle in [
        "Message::AiAppendMessage",
        "handle_ai_append_message(",
        "Message::AiSendMessage",
        "handle_ai_send_message(",
        "Message::AiSetSystemPrompt",
        "handle_ai_set_system_prompt(",
    ] {
        assert!(
            direct_dispatch.contains(needle),
            "direct SDK dispatch should include {needle}"
        );
    }

    let none_block = direct_dispatch
        .split("// These need UI thread or script-owned response channels")
        .nth(1)
        .expect("UI-thread fallback block should exist");
    assert!(
        !none_block.contains("AiAppendMessage")
            && !none_block.contains("AiSendMessage")
            && !none_block.contains("AiSetSystemPrompt"),
        "mutation messages must not fall through to unhandled prompt routing"
    );
}

#[test]
fn mutation_handlers_validate_active_chat_and_return_typed_request_errors() {
    let source = read("src/ai/sdk_handlers.rs");
    for code in [
        "AI_INVALID_CHAT_ID",
        "AI_CHAT_DELETED",
        "AI_CHAT_NOT_FOUND",
        "AI_INVALID_MESSAGE_ROLE",
        "AI_CONTEXT_PARTS_UNSUPPORTED",
    ] {
        assert!(
            source.contains(code),
            "mutation handlers should surface typed error code {code}"
        );
    }
    assert!(
        source.contains("request_id: Some(request_id)"),
        "typed mutation errors must carry requestId so SDK promises settle"
    );
}

#[test]
fn append_send_and_system_prompt_persist_messages_for_readback() {
    let source = read("src/ai/sdk_handlers.rs");
    assert!(
        source.contains("let message = AiMessage::new(chat.id, role, content);")
            && source.contains("storage::save_message(&message)")
            && source.contains("Message::AiMessageAppended"),
        "aiAppendMessage should save a real message and return its id"
    );
    assert!(
        source.contains("let mut message = AiMessage::user(chat.id, content);")
            && source.contains("message.images.push(ImageAttachment::png(image));")
            && source.contains("streaming_started: false"),
        "aiSendMessage should save a user message and honestly report no direct streaming turn"
    );
    assert!(
        source.contains(".find(|message| message.role == MessageRole::System)")
            && source.contains("message.content = prompt;")
            && source.contains("Message::AiSystemPromptSet"),
        "aiSetSystemPrompt should update or insert the stored system message representation"
    );
}

#[test]
fn sdk_mutation_promises_reject_typed_ai_errors() {
    let source = read("scripts/kit-sdk.ts");
    for api in [
        "Unexpected aiAppendMessage response",
        "Unexpected aiSendMessage response",
        "Unexpected aiSetSystemPrompt response",
    ] {
        assert!(
            source.contains(api),
            "SDK mutation promise should reject unexpected response path: {api}"
        );
    }
    assert!(
        source
            .matches("reject(new Error(`${error.code}: ${error.message}`));")
            .count()
            >= 3,
        "append/send/system-prompt SDK promises should reject request-scoped aiError responses"
    );
}
