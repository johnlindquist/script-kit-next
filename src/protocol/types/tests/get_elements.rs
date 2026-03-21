use super::*;

// ============================================================
// getElements request parsing
// ============================================================

#[test]
fn test_get_elements_request_parses_without_limit() {
    let json = r#"{"type":"getElements","requestId":"elm-1"}"#;
    let msg: crate::protocol::Message =
        serde_json::from_str(json).expect("Should parse getElements");

    match msg {
        crate::protocol::Message::GetElements { request_id, limit } => {
            assert_eq!(request_id, "elm-1");
            assert_eq!(limit, None);
        }
        other => panic!("Expected GetElements, got: {:?}", other),
    }
}

#[test]
fn test_get_elements_request_parses_with_limit() {
    let json = r#"{"type":"getElements","requestId":"elm-2","limit":5}"#;
    let msg: crate::protocol::Message =
        serde_json::from_str(json).expect("Should parse getElements with limit");

    match msg {
        crate::protocol::Message::GetElements { request_id, limit } => {
            assert_eq!(request_id, "elm-2");
            assert_eq!(limit, Some(5));
        }
        other => panic!("Expected GetElements, got: {:?}", other),
    }
}

// ============================================================
// elementsResult response serialization
// ============================================================

#[test]
fn test_elements_result_contains_expected_semantic_ids() {
    let elements = vec![
        ElementInfo::input("filter", Some("app"), true),
        ElementInfo::list("choices", 2),
        ElementInfo::choice(0, "Apple", "apple", true),
        ElementInfo::choice(1, "Banana", "banana", false),
    ];

    let response = crate::protocol::Message::elements_result("elm-1".to_string(), elements, 4);
    let json = serde_json::to_string(&response).expect("Should serialize elementsResult");

    // Must contain the correct type
    assert!(json.contains(r#""type":"elementsResult"#), "Missing type field");
    assert!(json.contains(r#""requestId":"elm-1"#), "Missing requestId");

    // Must contain semantic IDs
    assert!(json.contains("input:filter"), "Missing input:filter semantic ID");
    assert!(json.contains("list:choices"), "Missing list:choices semantic ID");
    assert!(json.contains("choice:0:apple"), "Missing choice:0:apple semantic ID");
    assert!(json.contains("choice:1:banana"), "Missing choice:1:banana semantic ID");

    // Must contain totalCount
    assert!(json.contains(r#""totalCount":4"#), "Missing totalCount");
}

#[test]
fn test_elements_result_roundtrip_preserves_structure() {
    let elements = vec![
        ElementInfo::input("filter", Some("test"), true),
        ElementInfo::list("results", 1),
        ElementInfo::choice(0, "Item One", "item-one", true),
    ];

    let original =
        crate::protocol::Message::elements_result("rt-1".to_string(), elements.clone(), 3);
    let json = serde_json::to_string(&original).expect("Should serialize");
    let parsed: crate::protocol::Message =
        serde_json::from_str(&json).expect("Should deserialize elementsResult");

    match parsed {
        crate::protocol::Message::ElementsResult {
            request_id,
            elements: parsed_elements,
            total_count,
        } => {
            assert_eq!(request_id, "rt-1");
            assert_eq!(total_count, 3);
            assert_eq!(parsed_elements.len(), 3);

            // Verify input element
            assert_eq!(parsed_elements[0].semantic_id, "input:filter");
            assert_eq!(parsed_elements[0].element_type, ElementType::Input);
            assert_eq!(parsed_elements[0].value, Some("test".to_string()));
            assert_eq!(parsed_elements[0].focused, Some(true));

            // Verify list element
            assert_eq!(parsed_elements[1].semantic_id, "list:results");
            assert_eq!(parsed_elements[1].element_type, ElementType::List);

            // Verify choice element
            assert_eq!(parsed_elements[2].semantic_id, "choice:0:item-one");
            assert_eq!(parsed_elements[2].element_type, ElementType::Choice);
            assert_eq!(parsed_elements[2].text, Some("Item One".to_string()));
            assert_eq!(parsed_elements[2].value, Some("item-one".to_string()));
            assert_eq!(parsed_elements[2].selected, Some(true));
            assert_eq!(parsed_elements[2].index, Some(0));
        }
        other => panic!("Expected ElementsResult, got: {:?}", other),
    }
}

// ============================================================
// ElementInfo constructors
// ============================================================

#[test]
fn test_element_info_choice_semantic_id_format() {
    let el = ElementInfo::choice(0, "Apple", "apple", true);
    assert_eq!(el.semantic_id, "choice:0:apple");
    assert_eq!(el.element_type, ElementType::Choice);
    assert_eq!(el.text, Some("Apple".to_string()));
    assert_eq!(el.value, Some("apple".to_string()));
    assert_eq!(el.selected, Some(true));
    assert_eq!(el.index, Some(0));
    assert_eq!(el.focused, None);
}

#[test]
fn test_element_info_input_semantic_id_format() {
    let el = ElementInfo::input("filter", Some("search text"), false);
    assert_eq!(el.semantic_id, "input:filter");
    assert_eq!(el.element_type, ElementType::Input);
    assert_eq!(el.value, Some("search text".to_string()));
    assert_eq!(el.focused, Some(false));
    assert_eq!(el.selected, None);
    assert_eq!(el.index, None);
}

#[test]
fn test_element_info_list_semantic_id_format() {
    let el = ElementInfo::list("choices", 5);
    assert_eq!(el.semantic_id, "list:choices");
    assert_eq!(el.element_type, ElementType::List);
    assert_eq!(el.text, Some("5 items".to_string()));
    assert_eq!(el.selected, None);
    assert_eq!(el.focused, None);
}

#[test]
fn test_element_info_panel_semantic_id_format() {
    let el = ElementInfo::panel("div-prompt");
    assert_eq!(el.semantic_id, "panel:div-prompt");
    assert_eq!(el.element_type, ElementType::Panel);
}

// ============================================================
// Missing/wrong response detection
// ============================================================

#[test]
fn test_get_elements_wrong_type_does_not_parse_as_get_elements() {
    let json = r#"{"type":"getState","requestId":"s-1"}"#;
    let msg: crate::protocol::Message = serde_json::from_str(json).expect("Should parse");
    assert!(
        !matches!(msg, crate::protocol::Message::GetElements { .. }),
        "getState must not parse as GetElements"
    );
}

#[test]
fn test_element_type_unknown_variant_forward_compat() {
    let json = r#""futureType""#;
    let parsed: ElementType = serde_json::from_str(json).expect("Should parse unknown type");
    assert_eq!(parsed, ElementType::Unknown);
}

// ============================================================
// Populated choice prompt produces expected element structure
// ============================================================

#[test]
fn test_simulated_choice_prompt_elements_structure() {
    // Simulate what collect_choice_view_elements would produce
    // for an arg prompt with choices and filter "app"
    let elements = vec![
        ElementInfo::input("filter", Some("app"), true),
        ElementInfo::list("choices", 2),
        ElementInfo::choice(0, "Apple", "apple", true),
        ElementInfo::choice(1, "Application", "application", false),
    ];

    let total_count = elements.len();
    let response =
        crate::protocol::Message::elements_result("sim-1".to_string(), elements, total_count);

    match response {
        crate::protocol::Message::ElementsResult {
            elements, total_count, ..
        } => {
            // Acceptance criteria: input:filter present
            assert!(
                elements.iter().any(|e| e.semantic_id == "input:filter"),
                "Must contain input:filter"
            );

            // Acceptance criteria: list semantic ID present
            assert!(
                elements.iter().any(|e| e.semantic_id.starts_with("list:")),
                "Must contain a list:* semantic ID"
            );

            // Acceptance criteria: at least one choice:*:* element
            assert!(
                elements.iter().any(|e| e.semantic_id.starts_with("choice:")),
                "Must contain at least one choice:*:* element"
            );

            assert!(total_count >= 3, "Total count should include input + list + choices");
        }
        other => panic!("Expected ElementsResult, got: {:?}", other),
    }
}
