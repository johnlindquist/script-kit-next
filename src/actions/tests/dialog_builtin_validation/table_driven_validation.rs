use serde_json::Value;

#[derive(Debug)]
struct BuiltinValidationCase {
    name: &'static str,
    input_json: &'static str,
    expected_error_codes: &'static [&'static str],
}

fn collect_builtin_schema_error_codes(input_json: &str) -> Vec<&'static str> {
    let parsed = match serde_json::from_str::<Value>(input_json) {
        Ok(value) => value,
        Err(_) => return vec!["invalid_json"],
    };

    let Some(obj) = parsed.as_object() else {
        return vec!["action_not_object"];
    };

    let mut errors = Vec::new();

    match obj.get("id") {
        Some(Value::String(id)) if !id.trim().is_empty() => {}
        Some(Value::String(_)) => errors.push("empty_id"),
        Some(_) => errors.push("invalid_id_type"),
        None => errors.push("missing_id"),
    }

    match obj.get("title") {
        Some(Value::String(title)) if !title.trim().is_empty() => {}
        Some(Value::String(_)) => errors.push("empty_title"),
        Some(_) => errors.push("invalid_title_type"),
        None => errors.push("missing_title"),
    }

    if matches!(obj.get("has_action"), Some(Value::Bool(true))) {
        errors.push("builtin_has_action_true");
    }

    errors
}

fn assert_builtin_schema_cases(cases: &[BuiltinValidationCase]) {
    for case in cases {
        let mut actual = collect_builtin_schema_error_codes(case.input_json);
        actual.sort_unstable();

        let mut expected = case.expected_error_codes.to_vec();
        expected.sort_unstable();

        assert_eq!(
            actual, expected,
            "case '{}' failed for input: {}",
            case.name, case.input_json
        );
    }
}

#[test]
fn test_builtin_dialog_schema_validation_reports_expected_error_codes_table_driven() {
    let cases = [
        BuiltinValidationCase {
            name: "valid_builtin_action",
            input_json: r#"{"id":"copy_path","title":"Copy Path","has_action":false}"#,
            expected_error_codes: &[],
        },
        BuiltinValidationCase {
            name: "missing_required_fields",
            input_json: r#"{}"#,
            expected_error_codes: &["missing_id", "missing_title"],
        },
        BuiltinValidationCase {
            name: "wrong_types_and_forbidden_has_action",
            input_json: r#"{"id":7,"title":null,"has_action":true}"#,
            expected_error_codes: &[
                "invalid_id_type",
                "invalid_title_type",
                "builtin_has_action_true",
            ],
        },
        BuiltinValidationCase {
            name: "empty_strings_are_invalid",
            input_json: r#"{"id":"","title":"   "}"#,
            expected_error_codes: &["empty_id", "empty_title"],
        },
        BuiltinValidationCase {
            name: "invalid_json",
            input_json: r#"{"id":"copy","title":"Copy""#,
            expected_error_codes: &["invalid_json"],
        },
    ];

    assert_builtin_schema_cases(&cases);
}
