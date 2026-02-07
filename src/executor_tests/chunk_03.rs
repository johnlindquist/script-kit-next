// ============================================================
// AUTO_SUBMIT Mode Tests
// ============================================================
//
// Note: These tests verify the AUTO_SUBMIT environment variable parsing.
// Since env vars are global and tests run in parallel, we use a single
// comprehensive test that exercises all cases sequentially to avoid races.

use super::{
    get_auto_submit_delay, get_auto_submit_index, get_auto_submit_value, is_auto_submit_enabled,
};
use std::time::Duration;

/// Comprehensive test for is_auto_submit_enabled() function.
/// Tests all cases in sequence to avoid env var race conditions.
#[test]
fn test_is_auto_submit_enabled_all_cases() {
    // Test "true" value
    std::env::set_var("AUTO_SUBMIT", "true");
    assert!(
        is_auto_submit_enabled(),
        "AUTO_SUBMIT=true should enable auto-submit"
    );

    // Test "1" value
    std::env::set_var("AUTO_SUBMIT", "1");
    assert!(
        is_auto_submit_enabled(),
        "AUTO_SUBMIT=1 should enable auto-submit"
    );

    // Test "false" value
    std::env::set_var("AUTO_SUBMIT", "false");
    assert!(
        !is_auto_submit_enabled(),
        "AUTO_SUBMIT=false should NOT enable auto-submit"
    );

    // Test "0" value
    std::env::set_var("AUTO_SUBMIT", "0");
    assert!(
        !is_auto_submit_enabled(),
        "AUTO_SUBMIT=0 should NOT enable auto-submit"
    );

    // Test other value
    std::env::set_var("AUTO_SUBMIT", "yes");
    assert!(
        !is_auto_submit_enabled(),
        "AUTO_SUBMIT=yes should NOT enable auto-submit"
    );

    // Test unset (default)
    std::env::remove_var("AUTO_SUBMIT");
    assert!(
        !is_auto_submit_enabled(),
        "Unset AUTO_SUBMIT should NOT enable auto-submit"
    );
}

/// Comprehensive test for get_auto_submit_delay() function.
#[test]
fn test_get_auto_submit_delay_all_cases() {
    // Test custom value
    std::env::set_var("AUTO_SUBMIT_DELAY_MS", "500");
    assert_eq!(
        get_auto_submit_delay(),
        Duration::from_millis(500),
        "AUTO_SUBMIT_DELAY_MS=500 should return 500ms"
    );

    // Test invalid value (falls back to default)
    std::env::set_var("AUTO_SUBMIT_DELAY_MS", "not_a_number");
    assert_eq!(
        get_auto_submit_delay(),
        Duration::from_millis(100),
        "Invalid AUTO_SUBMIT_DELAY_MS should default to 100ms"
    );

    // Test unset (default)
    std::env::remove_var("AUTO_SUBMIT_DELAY_MS");
    assert_eq!(
        get_auto_submit_delay(),
        Duration::from_millis(100),
        "Unset AUTO_SUBMIT_DELAY_MS should default to 100ms"
    );
}

/// Comprehensive test for get_auto_submit_value() function.
#[test]
fn test_get_auto_submit_value_all_cases() {
    // Test set value
    std::env::set_var("AUTO_SUBMIT_VALUE", "test_value");
    assert_eq!(
        get_auto_submit_value(),
        Some("test_value".to_string()),
        "AUTO_SUBMIT_VALUE=test_value should return Some(test_value)"
    );

    // Test empty value
    std::env::set_var("AUTO_SUBMIT_VALUE", "");
    assert_eq!(
        get_auto_submit_value(),
        Some("".to_string()),
        "AUTO_SUBMIT_VALUE='' should return Some('')"
    );

    // Test unset (None)
    std::env::remove_var("AUTO_SUBMIT_VALUE");
    assert_eq!(
        get_auto_submit_value(),
        None,
        "Unset AUTO_SUBMIT_VALUE should return None"
    );
}

/// Comprehensive test for get_auto_submit_index() function.
#[test]
fn test_get_auto_submit_index_all_cases() {
    // Test custom value
    std::env::set_var("AUTO_SUBMIT_INDEX", "5");
    assert_eq!(
        get_auto_submit_index(),
        5,
        "AUTO_SUBMIT_INDEX=5 should return 5"
    );

    // Test invalid value (falls back to default)
    std::env::set_var("AUTO_SUBMIT_INDEX", "invalid");
    assert_eq!(
        get_auto_submit_index(),
        0,
        "Invalid AUTO_SUBMIT_INDEX should default to 0"
    );

    // Test negative value (falls back to default since usize can't be negative)
    std::env::set_var("AUTO_SUBMIT_INDEX", "-1");
    assert_eq!(
        get_auto_submit_index(),
        0,
        "Negative AUTO_SUBMIT_INDEX should default to 0"
    );

    // Test unset (default)
    std::env::remove_var("AUTO_SUBMIT_INDEX");
    assert_eq!(
        get_auto_submit_index(),
        0,
        "Unset AUTO_SUBMIT_INDEX should default to 0"
    );
}

