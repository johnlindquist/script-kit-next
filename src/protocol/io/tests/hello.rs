use super::*;

// ============================================================
// Hello/HelloAck Handshake Tests
// ============================================================

#[test]
fn test_hello_message_parse() {
    let json = r#"{"type":"hello","protocol":1,"sdkVersion":"1.0.0","capabilities":["submitJson","semanticIdV2"]}"#;
    match parse_message_graceful(json) {
        ParseResult::Ok(Message::Hello {
            protocol,
            sdk_version,
            capabilities,
        }) => {
            assert_eq!(protocol, 1);
            assert_eq!(sdk_version, "1.0.0");
            assert_eq!(capabilities.len(), 2);
            assert!(capabilities.contains(&"submitJson".to_string()));
            assert!(capabilities.contains(&"semanticIdV2".to_string()));
        }
        other => panic!("Expected Hello message, got {:?}", other),
    }
}

#[test]
fn test_hello_message_empty_capabilities() {
    let json = r#"{"type":"hello","protocol":1,"sdkVersion":"0.9.0"}"#;
    match parse_message_graceful(json) {
        ParseResult::Ok(Message::Hello {
            protocol,
            sdk_version,
            capabilities,
        }) => {
            assert_eq!(protocol, 1);
            assert_eq!(sdk_version, "0.9.0");
            assert!(capabilities.is_empty()); // default empty vec
        }
        other => panic!("Expected Hello message, got {:?}", other),
    }
}

#[test]
fn test_hello_ack_message_parse() {
    let json = r#"{"type":"helloAck","protocol":1,"capabilities":["submitJson"]}"#;
    match parse_message_graceful(json) {
        ParseResult::Ok(Message::HelloAck {
            protocol,
            capabilities,
        }) => {
            assert_eq!(protocol, 1);
            assert_eq!(capabilities.len(), 1);
            assert_eq!(capabilities[0], "submitJson");
        }
        other => panic!("Expected HelloAck message, got {:?}", other),
    }
}

#[test]
fn test_hello_constructor() {
    let msg = Message::hello(1, "2.0.0", vec!["cap1".to_string(), "cap2".to_string()]);
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains(r#""type":"hello""#));
    assert!(json.contains(r#""protocol":1"#));
    assert!(json.contains(r#""sdkVersion":"2.0.0""#));
    assert!(json.contains(r#""capabilities":["cap1","cap2"]"#));
}

#[test]
fn test_hello_ack_constructor() {
    let msg = Message::hello_ack(1, vec!["feature1".to_string()]);
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains(r#""type":"helloAck""#));
    assert!(json.contains(r#""protocol":1"#));
    assert!(json.contains(r#""capabilities":["feature1"]"#));
}

#[test]
fn test_hello_roundtrip() {
    let original = Message::hello(
        1,
        "1.2.3",
        vec![
            crate::protocol::capabilities::SUBMIT_JSON.to_string(),
            crate::protocol::capabilities::SEMANTIC_ID_V2.to_string(),
        ],
    );
    let json = serde_json::to_string(&original).unwrap();
    let restored: Message = serde_json::from_str(&json).unwrap();

    match restored {
        Message::Hello {
            protocol,
            sdk_version,
            capabilities,
        } => {
            assert_eq!(protocol, 1);
            assert_eq!(sdk_version, "1.2.3");
            assert_eq!(capabilities.len(), 2);
        }
        _ => panic!("Expected Hello message"),
    }
}

#[test]
fn test_hello_ack_full() {
    let msg = Message::hello_ack_full(1);
    match msg {
        Message::HelloAck {
            protocol,
            capabilities,
        } => {
            assert_eq!(protocol, 1);
            // Should include all known capabilities
            assert!(capabilities.contains(&"submitJson".to_string()));
            assert!(capabilities.contains(&"semanticIdV2".to_string()));
            assert!(capabilities.contains(&"unknownTypeOk".to_string()));
            assert!(capabilities.contains(&"forwardCompat".to_string()));
            assert!(capabilities.contains(&"choiceKey".to_string()));
            assert!(capabilities.contains(&"mouseDataV2".to_string()));
        }
        _ => panic!("Expected HelloAck message"),
    }
}
