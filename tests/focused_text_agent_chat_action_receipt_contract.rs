use script_kit_gpui::protocol::{AcpFocusedTextActionReceipt, AcpFocusedTextState};

#[test]
fn focused_text_action_receipt_serializes_only_redacted_fields() {
    let receipt = AcpFocusedTextActionReceipt {
        action: "copy".to_string(),
        success: true,
        changed_text: false,
        copied_to_clipboard: true,
        before_ui_variant: "focused-text-mini".to_string(),
        after_ui_variant: "focused-text-mini".to_string(),
        output_length: 42,
        error_code: None,
    };

    let json = serde_json::to_value(&receipt).expect("serialize focused-text action receipt");
    assert_eq!(json["action"], "copy");
    assert_eq!(json["success"], true);
    assert_eq!(json["changedText"], false);
    assert_eq!(json["copiedToClipboard"], true);
    assert_eq!(json["beforeUiVariant"], "focused-text-mini");
    assert_eq!(json["afterUiVariant"], "focused-text-mini");
    assert_eq!(json["outputLength"], 42);
    assert!(json.get("errorCode").is_none());

    let raw = json.to_string();
    for forbidden in [
        "Hello world",
        "capturedText",
        "assistantOutput",
        "prompt",
        "instruction",
        "clipboard",
    ] {
        assert!(
            !raw.contains(forbidden),
            "receipt leaked sensitive field or fixture text: {forbidden}"
        );
    }
}

#[test]
fn focused_text_state_carries_redacted_context_identity_and_last_action() {
    let state = AcpFocusedTextState {
        mode: "mini".to_string(),
        phase: "result".to_string(),
        footer_visible: true,
        actions_visible: true,
        can_expand_to_chat: true,
        session_id: "focused-text-session-for-tests".to_string(),
        app_name: "TextEdit".to_string(),
        char_count: 11,
        word_count: 2,
        context_present: true,
        context_status: "captured".to_string(),
        context_failure_code: None,
        context_fingerprint: Some("fnv1a64:0123456789abcdef".to_string()),
        submitted_prompt_locked: true,
        submitted_prompt_char_count: Some(12),
        input_redacted: true,
        can_replace: true,
        can_append: true,
        can_copy: true,
        has_output: true,
        last_apply_action: Some("copy".to_string()),
        last_action_receipt: Some(AcpFocusedTextActionReceipt {
            action: "copy".to_string(),
            success: true,
            changed_text: false,
            copied_to_clipboard: true,
            before_ui_variant: "focused-text-mini".to_string(),
            after_ui_variant: "focused-text-mini".to_string(),
            output_length: 25,
            error_code: None,
        }),
    };

    let json = serde_json::to_value(&state).expect("serialize focused-text state");
    assert_eq!(json["mode"], "mini");
    assert_eq!(json["phase"], "result");
    assert_eq!(json["footerVisible"], true);
    assert_eq!(json["actionsVisible"], true);
    assert_eq!(json["canExpandToChat"], true);
    assert_eq!(json["charCount"], 11);
    assert_eq!(json["wordCount"], 2);
    assert_eq!(json["contextPresent"], true);
    assert_eq!(json["contextStatus"], "captured");
    assert_eq!(json["contextFingerprint"], "fnv1a64:0123456789abcdef");
    assert_eq!(json["submittedPromptLocked"], true);
    assert_eq!(json["submittedPromptCharCount"], 12);
    assert_eq!(json["inputRedacted"], true);
    assert_eq!(json["lastActionReceipt"]["action"], "copy");
    assert_eq!(json["lastActionReceipt"]["outputLength"], 25);

    let raw = json.to_string();
    for forbidden in [
        "Hello world",
        "capturedText",
        "assistantOutput",
        "promptXml",
        "instructionText",
        "clipboardText",
    ] {
        assert!(
            !raw.contains(forbidden),
            "focused-text ACP state leaked sensitive field or fixture text: {forbidden}"
        );
    }
}
