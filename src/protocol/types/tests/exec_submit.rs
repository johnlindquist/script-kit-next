use super::*;

#[test]
fn test_exec_options_extra_fields_preserved() {
    // JSON with unknown future field
    let json = r#"{"cwd":"/tmp","timeout":5000,"futureField":"someValue","anotherNew":123}"#;
    let opts: ExecOptions = serde_json::from_str(json).unwrap();

    // Known fields work
    assert_eq!(opts.cwd, Some("/tmp".to_string()));
    assert_eq!(opts.timeout, Some(5000));

    // Extra fields are preserved
    assert!(opts.extra.contains_key("futureField"));
    assert!(opts.extra.contains_key("anotherNew"));
}

#[test]
fn test_exec_options_extra_roundtrip() {
    let json = r#"{"cwd":"/home","newField":"preserved"}"#;
    let opts: ExecOptions = serde_json::from_str(json).unwrap();
    let serialized = serde_json::to_string(&opts).unwrap();

    // newField should still be in the output
    assert!(serialized.contains("newField"));
    assert!(serialized.contains("preserved"));
}

// ============================================================
// SubmitValue Tests
// ============================================================

#[test]
fn test_submit_value_text() {
    let val = SubmitValue::text("hello");
    assert!(val.is_text());
    assert!(!val.is_json());
    assert_eq!(val.as_str(), Some("hello"));
    assert_eq!(val.to_string_repr(), "hello");
}

#[test]
fn test_submit_value_json_array() {
    let arr = serde_json::json!(["a", "b", "c"]);
    let val = SubmitValue::json(arr);
    assert!(val.is_json());
    assert!(!val.is_text());
    assert!(val.as_str().is_none());
    assert_eq!(val.to_string_repr(), r#"["a","b","c"]"#);
}

#[test]
fn test_submit_value_json_object() {
    let obj = serde_json::json!({"name": "test", "count": 42});
    let val = SubmitValue::json(obj);
    assert!(val.is_json());
    // to_string_repr should serialize to JSON
    let repr = val.to_string_repr();
    assert!(repr.contains("name"));
    assert!(repr.contains("test"));
    assert!(repr.contains("42"));
}

#[test]
fn test_submit_value_deserialize_string() {
    // Old format: plain string
    let json = r#""hello world""#;
    let val: SubmitValue = serde_json::from_str(json).unwrap();
    assert!(val.is_text());
    assert_eq!(val.as_str(), Some("hello world"));
}

#[test]
fn test_submit_value_deserialize_array() {
    // New format: JSON array (for multi-select)
    let json = r#"["apple","banana"]"#;
    let val: SubmitValue = serde_json::from_str(json).unwrap();
    assert!(val.is_json());
    match val {
        SubmitValue::Json(v) => {
            assert!(v.is_array());
            assert_eq!(v.as_array().unwrap().len(), 2);
        }
        _ => panic!("Expected Json variant"),
    }
}

#[test]
fn test_submit_value_deserialize_object() {
    // New format: JSON object (for forms)
    let json = r#"{"field1":"value1","field2":123}"#;
    let val: SubmitValue = serde_json::from_str(json).unwrap();
    assert!(val.is_json());
    match val {
        SubmitValue::Json(v) => {
            assert!(v.is_object());
            let obj = v.as_object().unwrap();
            assert_eq!(obj.get("field1").unwrap(), "value1");
        }
        _ => panic!("Expected Json variant"),
    }
}

#[test]
fn test_submit_value_roundtrip_text() {
    let original = SubmitValue::text("hello");
    let json = serde_json::to_string(&original).unwrap();
    let restored: SubmitValue = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn test_submit_value_roundtrip_json() {
    let original = SubmitValue::json(serde_json::json!(["x", "y", "z"]));
    let json = serde_json::to_string(&original).unwrap();
    let restored: SubmitValue = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn test_submit_value_from_string() {
    let val: SubmitValue = "test".into();
    assert!(val.is_text());
    assert_eq!(val.as_str(), Some("test"));
}

#[test]
fn test_submit_value_from_json_value() {
    // JSON string should become Text
    let val: SubmitValue = serde_json::Value::String("hello".to_string()).into();
    assert!(val.is_text());
    assert_eq!(val.as_str(), Some("hello"));

    // JSON array should become Json
    let val: SubmitValue = serde_json::json!([1, 2, 3]).into();
    assert!(val.is_json());
}

#[test]
fn test_submit_value_to_option_string() {
    let text_val = SubmitValue::text("hello");
    assert_eq!(text_val.to_option_string(), Some("hello".to_string()));

    let json_val = SubmitValue::json(serde_json::json!(["a"]));
    assert_eq!(json_val.to_option_string(), Some(r#"["a"]"#.to_string()));
}

#[test]
fn test_submit_value_to_json_value() {
    let text_val = SubmitValue::text("hello");
    assert_eq!(
        text_val.to_json_value(),
        serde_json::Value::String("hello".to_string())
    );

    let json_val = SubmitValue::json(serde_json::json!({"key": "val"}));
    assert_eq!(json_val.to_json_value(), serde_json::json!({"key": "val"}));
}
