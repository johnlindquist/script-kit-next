use crate::main_window_preflight::types::{
    MainWindowPreflightAction, MainWindowPreflightActionKind, MainWindowPreflightReceipt,
};

#[test]
fn receipt_serializes_with_camel_case_keys() {
    let receipt = MainWindowPreflightReceipt {
        filter_text: "resize images".to_string(),
        selected_index: 2,
        enter_action: MainWindowPreflightAction {
            kind: MainWindowPreflightActionKind::RunScript,
            label: "Run Script".to_string(),
            subject: "Resize Images".to_string(),
            type_label: "Script".to_string(),
            source_name: Some("kit".to_string()),
            description: Some("Batch resize images".to_string()),
        },
        tab_action: None,
        warnings: vec![],
    };

    let value = serde_json::to_value(&receipt).expect("receipt should serialize");
    assert!(value.get("filterText").is_some(), "expected camelCase filterText key");
    assert!(
        value.get("selectedIndex").is_some(),
        "expected camelCase selectedIndex key"
    );
    assert!(
        value.get("enterAction").is_some(),
        "expected camelCase enterAction key"
    );
}

#[test]
fn ask_ai_kind_serializes_as_camel_case_enum() {
    let value = serde_json::to_value(MainWindowPreflightActionKind::AskAi)
        .expect("enum should serialize");
    assert_eq!(value, serde_json::Value::String("askAi".to_string()));
}

#[test]
fn all_action_kinds_round_trip_to_camel_case() {
    let cases = vec![
        (MainWindowPreflightActionKind::RunScript, "runScript"),
        (MainWindowPreflightActionKind::RunSnippet, "runSnippet"),
        (MainWindowPreflightActionKind::RunCommand, "runCommand"),
        (MainWindowPreflightActionKind::LaunchApp, "launchApp"),
        (MainWindowPreflightActionKind::SwitchWindow, "switchWindow"),
        (MainWindowPreflightActionKind::RunAgent, "runAgent"),
        (MainWindowPreflightActionKind::RunFallback, "runFallback"),
        (MainWindowPreflightActionKind::AskAi, "askAi"),
    ];

    for (kind, expected) in cases {
        let value = serde_json::to_value(&kind).expect("should serialize");
        assert_eq!(
            value,
            serde_json::Value::String(expected.to_string()),
            "ActionKind {:?} should serialize to \"{}\"",
            kind,
            expected
        );
    }
}

#[test]
fn receipt_with_tab_action_serializes_correctly() {
    let receipt = MainWindowPreflightReceipt {
        filter_text: "hello".to_string(),
        selected_index: 0,
        enter_action: MainWindowPreflightAction {
            kind: MainWindowPreflightActionKind::RunScript,
            label: "Run Script".to_string(),
            subject: "Hello World".to_string(),
            type_label: "Script".to_string(),
            source_name: None,
            description: None,
        },
        tab_action: Some(MainWindowPreflightAction {
            kind: MainWindowPreflightActionKind::AskAi,
            label: "Ask AI".to_string(),
            subject: "hello".to_string(),
            type_label: "AI".to_string(),
            source_name: None,
            description: Some("Opens the AI window".to_string()),
        }),
        warnings: vec![],
    };

    let value = serde_json::to_value(&receipt).expect("should serialize");
    let tab = value.get("tabAction").expect("tabAction should be present");
    assert_eq!(
        tab.get("kind").and_then(|v| v.as_str()),
        Some("askAi"),
        "tab action kind should be askAi"
    );
    assert_eq!(
        tab.get("typeLabel").and_then(|v| v.as_str()),
        Some("AI"),
        "tab action typeLabel should be AI"
    );
}

#[test]
fn receipt_with_warnings_serializes_correctly() {
    let receipt = MainWindowPreflightReceipt {
        filter_text: String::new(),
        selected_index: 0,
        enter_action: MainWindowPreflightAction {
            kind: MainWindowPreflightActionKind::RunAgent,
            label: "Run Agent".to_string(),
            subject: "Test Agent".to_string(),
            type_label: "Agent".to_string(),
            source_name: None,
            description: None,
        },
        tab_action: None,
        warnings: vec![
            "Agent execution is not fully implemented.".to_string(),
            "Tab-to-AI is inactive.".to_string(),
        ],
    };

    let value = serde_json::to_value(&receipt).expect("should serialize");
    let warnings = value
        .get("warnings")
        .and_then(|v| v.as_array())
        .expect("warnings should be an array");
    assert_eq!(warnings.len(), 2);
}
