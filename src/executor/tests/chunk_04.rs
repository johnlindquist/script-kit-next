// ============================================================
// AutoSubmitConfig Tests
// ============================================================

use super::{get_auto_submit_config, AutoSubmitConfig};
use crate::protocol::Choice;

/// Test AutoSubmitConfig default values.
#[test]
fn test_auto_submit_config_default() {
    let config = AutoSubmitConfig::default();

    assert!(!config.enabled, "Default should be disabled");
    assert_eq!(
        config.delay,
        Duration::from_millis(100),
        "Default delay should be 100ms"
    );
    assert!(
        config.value_override.is_none(),
        "Default should have no value override"
    );
    assert_eq!(config.index, 0, "Default index should be 0");
}

/// Test AutoSubmitConfig::from_env() captures env vars.
#[test]
fn test_auto_submit_config_from_env() {
    // Set all env vars
    std::env::set_var("AUTO_SUBMIT", "true");
    std::env::set_var("AUTO_SUBMIT_DELAY_MS", "250");
    std::env::set_var("AUTO_SUBMIT_VALUE", "override_value");
    std::env::set_var("AUTO_SUBMIT_INDEX", "3");

    let config = AutoSubmitConfig::from_env();

    assert!(config.enabled, "Should be enabled when AUTO_SUBMIT=true");
    assert_eq!(
        config.delay,
        Duration::from_millis(250),
        "Delay should be 250ms"
    );
    assert_eq!(
        config.value_override,
        Some("override_value".to_string()),
        "Should have override value"
    );
    assert_eq!(config.index, 3, "Index should be 3");

    // Clean up
    std::env::remove_var("AUTO_SUBMIT");
    std::env::remove_var("AUTO_SUBMIT_DELAY_MS");
    std::env::remove_var("AUTO_SUBMIT_VALUE");
    std::env::remove_var("AUTO_SUBMIT_INDEX");
}

/// Test get_auto_submit_config() convenience function.
#[test]
fn test_get_auto_submit_config() {
    // Clean state
    std::env::remove_var("AUTO_SUBMIT");
    std::env::remove_var("AUTO_SUBMIT_DELAY_MS");
    std::env::remove_var("AUTO_SUBMIT_VALUE");
    std::env::remove_var("AUTO_SUBMIT_INDEX");

    let config = get_auto_submit_config();

    assert!(!config.enabled, "Default should be disabled");
    assert_eq!(
        config.delay,
        Duration::from_millis(100),
        "Default delay should be 100ms"
    );
}

/// Test get_arg_value() with choices.
#[test]
fn test_auto_submit_config_get_arg_value() {
    let choices = vec![
        Choice {
            name: "Apple".to_string(),
            value: "apple".to_string(),
            description: None,
            key: None,
            semantic_id: None,
        },
        Choice {
            name: "Banana".to_string(),
            value: "banana".to_string(),
            description: None,
            key: None,
            semantic_id: None,
        },
        Choice {
            name: "Cherry".to_string(),
            value: "cherry".to_string(),
            description: None,
            key: None,
            semantic_id: None,
        },
    ];

    // Test default behavior (first choice)
    let config = AutoSubmitConfig::default();
    assert_eq!(
        config.get_arg_value(&choices),
        Some("apple".to_string()),
        "Default should return first choice value"
    );

    // Test with index
    let config = AutoSubmitConfig {
        index: 1,
        ..Default::default()
    };
    assert_eq!(
        config.get_arg_value(&choices),
        Some("banana".to_string()),
        "Index 1 should return second choice value"
    );

    // Test with out-of-bounds index (should clamp)
    let config = AutoSubmitConfig {
        index: 100,
        ..Default::default()
    };
    assert_eq!(
        config.get_arg_value(&choices),
        Some("cherry".to_string()),
        "Out-of-bounds index should clamp to last choice"
    );

    // Test with value override
    let config = AutoSubmitConfig {
        value_override: Some("custom".to_string()),
        index: 1,
        ..Default::default()
    };
    assert_eq!(
        config.get_arg_value(&choices),
        Some("custom".to_string()),
        "Override value should take precedence over index"
    );

    // Test with empty choices
    let config = AutoSubmitConfig::default();
    assert_eq!(
        config.get_arg_value(&[]),
        None,
        "Empty choices should return None"
    );
}

/// Test get_div_value() returns None (just dismissal).
#[test]
fn test_auto_submit_config_get_div_value() {
    let config = AutoSubmitConfig::default();
    assert_eq!(
        config.get_div_value(),
        None,
        "Div prompt should return None for dismissal"
    );
}

/// Test get_editor_value() returns original content.
#[test]
fn test_auto_submit_config_get_editor_value() {
    let config = AutoSubmitConfig::default();
    assert_eq!(
        config.get_editor_value("original content"),
        Some("original content".to_string()),
        "Editor should return original content unchanged"
    );

    // Test with override
    let config = AutoSubmitConfig {
        value_override: Some("modified".to_string()),
        ..Default::default()
    };
    assert_eq!(
        config.get_editor_value("original content"),
        Some("modified".to_string()),
        "Override should take precedence"
    );
}

/// Test get_term_value() returns "0".
#[test]
fn test_auto_submit_config_get_term_value() {
    let config = AutoSubmitConfig::default();
    assert_eq!(
        config.get_term_value(),
        Some("0".to_string()),
        "Term prompt should return exit code 0"
    );
}

/// Test get_form_value() returns empty JSON object.
#[test]
fn test_auto_submit_config_get_form_value() {
    let config = AutoSubmitConfig::default();
    assert_eq!(
        config.get_form_value(),
        Some("{}".to_string()),
        "Form prompt should return empty JSON object"
    );
}

/// Test get_select_value() returns JSON array.
#[test]
fn test_auto_submit_config_get_select_value() {
    let choices = vec![
        Choice {
            name: "Apple".to_string(),
            value: "apple".to_string(),
            description: None,
            key: None,
            semantic_id: None,
        },
        Choice {
            name: "Banana".to_string(),
            value: "banana".to_string(),
            description: None,
            key: None,
            semantic_id: None,
        },
    ];

    let config = AutoSubmitConfig::default();
    assert_eq!(
        config.get_select_value(&choices),
        Some(r#"["apple"]"#.to_string()),
        "Select should return JSON array with first choice"
    );

    let config = AutoSubmitConfig {
        index: 1,
        ..Default::default()
    };
    assert_eq!(
        config.get_select_value(&choices),
        Some(r#"["banana"]"#.to_string()),
        "Select with index 1 should return second choice"
    );

    let config = AutoSubmitConfig::default();
    assert_eq!(
        config.get_select_value(&[]),
        Some("[]".to_string()),
        "Empty choices should return empty array"
    );
}

/// Test get_fields_value() returns JSON array of empty strings.
#[test]
fn test_auto_submit_config_get_fields_value() {
    let config = AutoSubmitConfig::default();

    assert_eq!(
        config.get_fields_value(0),
        Some("[]".to_string()),
        "0 fields should return empty array"
    );
    assert_eq!(
        config.get_fields_value(1),
        Some(r#"[""]"#.to_string()),
        "1 field should return array with one empty string"
    );
    assert_eq!(
        config.get_fields_value(3),
        Some(r#"["","",""]"#.to_string()),
        "3 fields should return array with three empty strings"
    );
}

/// Test get_path_value() returns test path.
#[test]
fn test_auto_submit_config_get_path_value() {
    let config = AutoSubmitConfig::default();
    assert_eq!(
        config.get_path_value(),
        Some("/tmp/test-path".to_string()),
        "Path prompt should return /tmp/test-path"
    );
}

/// Test get_hotkey_value() returns Cmd+A.
#[test]
fn test_auto_submit_config_get_hotkey_value() {
    let config = AutoSubmitConfig::default();
    assert_eq!(
        config.get_hotkey_value(),
        Some(r#"{"key":"a","command":true}"#.to_string()),
        "Hotkey prompt should return Cmd+A JSON"
    );
}

/// Test get_drop_value() returns test file array.
#[test]
fn test_auto_submit_config_get_drop_value() {
    let config = AutoSubmitConfig::default();
    assert_eq!(
        config.get_drop_value(),
        Some(r#"[{"path":"/tmp/test.txt"}]"#.to_string()),
        "Drop prompt should return test file array"
    );
}

