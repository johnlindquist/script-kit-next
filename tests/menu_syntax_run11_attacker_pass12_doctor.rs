//! Run 11 Pass #12 — attacker probe over the Pass-11 `doctor::validate`
//! engine in `src/menu_syntax/doctor.rs`. The engine claims to emit
//! actionable diagnostics with stable JSONPath pointers regardless of
//! input pathology — this probe stresses every adversarial dimension
//! the attacker-mode rule menu permits.
//!
//! Categories covered (≥3 required by attacker-mode rule):
//!   1. Boundary — empty / oversize / NUL / null / nested
//!   2. Composition — duplicate paths reported correctly across N entries
//!   3. Localization & Encoding — RTL, emoji, smart quotes, NFC vs NFD
//!   4. Determinism — repeated validate calls yield equal reports;
//!                    serialize→deserialize stable
//!   5. Lifecycle — order-independence (shuffling input order doesn't
//!                  affect issue *content*, only paths)
//!
//! Receipt: `cargo test --test menu_syntax_run11_attacker_pass12_doctor`.

use script_kit_gpui::menu_syntax::{doctor_validate, DoctorIssue, DoctorReport, DoctorSeverity};
use serde_json::{json, Value};

fn err_count(v: &Value) -> usize {
    doctor_validate(v).errors().count()
}

fn paths_of(v: &Value) -> Vec<String> {
    doctor_validate(v)
        .issues
        .iter()
        .map(|i| i.path.clone())
        .collect()
}

// ============================================================================
// 1. BOUNDARY (7 actions)
// ============================================================================

#[test]
fn boundary_01_empty_array_yields_empty_report() {
    let r = doctor_validate(&json!([]));
    assert!(!r.has_errors());
    assert!(r.issues.is_empty());
}

#[test]
fn boundary_02_null_top_level_yields_warning_not_error() {
    let r = doctor_validate(&json!(null));
    assert!(
        !r.has_errors(),
        "null should warn, not error. Issues: {:?}",
        r.issues
    );
    assert_eq!(r.issues.len(), 1);
    assert_eq!(r.issues[0].severity, DoctorSeverity::Warning);
    assert_eq!(r.issues[0].path, "$");
}

#[test]
fn boundary_03_number_top_level_errors_with_type_label() {
    let r = doctor_validate(&json!(3.14));
    assert!(r.has_errors());
    assert_eq!(r.issues[0].path, "$");
    assert!(r.issues[0].message.contains("number"));
}

#[test]
fn boundary_04_oversize_target_slug_does_not_panic() {
    let big = "x".repeat(50_000);
    let r = doctor_validate(&json!([{"family": "capture.v1", "targets": [big]}]));
    // Custom slug → Warning only; no error.
    assert!(!r.has_errors());
    assert!(
        r.issues
            .iter()
            .any(|i| i.severity == DoctorSeverity::Warning),
        "expected non-built-in warning"
    );
}

#[test]
fn boundary_05_target_with_nul_byte_does_not_corrupt_path() {
    let r = doctor_validate(&json!([{"family": "capture.v1", "targets": ["before\u{0000}after"]}]));
    // NUL is not whitespace, not a built-in → Warning only, but no error.
    assert!(!r.has_errors());
    assert!(r
        .issues
        .iter()
        .all(|i| i.path.starts_with("$[0].targets[0]") || i.path == "$[0]"));
}

#[test]
fn boundary_06_50_invalid_args_each_get_distinct_indexed_path() {
    let mut args = Vec::new();
    for _ in 0..50 {
        args.push(json!({})); // each missing `name`
    }
    let v = json!([{"family": "command.v1", "head": "deploy", "args": args}]);
    let r = doctor_validate(&v);
    assert_eq!(r.errors().count(), 50, "one error per missing-name arg");
    for i in 0..50 {
        let expected = format!("$[0].args[{i}].name");
        assert!(
            r.errors().any(|e| e.path == expected),
            "missing path {expected}"
        );
    }
}

#[test]
fn boundary_07_object_form_uses_dollar_path_not_index() {
    let r = doctor_validate(&json!({"family": "capture.v1", "targets": []}));
    // Single-spec object form skips the array index — paths start at "$".
    assert!(r.has_errors());
    assert_eq!(r.issues[0].path, "$.targets");
}

// ============================================================================
// 2. COMPOSITION (5 actions)
// ============================================================================

#[test]
fn composition_01_duplicate_command_path_lists_all_indices_in_order() {
    let v = json!([
        {"family": "command.v1", "head": "deploy"},
        {"family": "command.v1", "head": "rollback"},
        {"family": "command.v1", "head": "deploy"},
        {"family": "command.v1", "head": "deploy"},
    ]);
    let r = doctor_validate(&v);
    // Exactly one duplicate-error grouping all `deploy` indices.
    let dup_errors: Vec<&DoctorIssue> = r
        .errors()
        .filter(|e| e.message.contains("duplicate command.v1 head"))
        .collect();
    assert_eq!(dup_errors.len(), 1);
    // Indices reported in input order: 0, 2, 3.
    assert_eq!(dup_errors[0].path, "$[0], $[2], $[3]");
    assert!(dup_errors[0].message.contains("registered 3 times"));
}

#[test]
fn composition_02_mixed_family_array_doesnt_cross_pollinate_required_fields() {
    let v = json!([
        {"family": "capture.v1", "targets": ["todo"]},
        {"family": "command.v1"}, // missing head
        {"family": "skill.v1"},   // missing slug
    ]);
    let r = doctor_validate(&v);
    let paths: Vec<&str> = r.errors().map(|e| e.path.as_str()).collect();
    // Capture entry is fine (no error). Command misses head, skill misses slug.
    // No "missing targets" error should leak from command/skill rows.
    assert!(paths.contains(&"$[1].head"));
    assert!(paths.contains(&"$[2].slug"));
    assert!(
        !paths
            .iter()
            .any(|p| p.contains(".targets") && p.starts_with("$[1]")),
        "command must not have a targets diagnostic: {paths:?}"
    );
    assert!(
        !paths
            .iter()
            .any(|p| p.contains(".targets") && p.starts_with("$[2]")),
        "skill must not have a targets diagnostic: {paths:?}"
    );
}

#[test]
fn composition_03_one_bad_entry_doesnt_block_subsequent_entries() {
    let v = json!([
        {"family": "totally-bogus-family"},   // error halts validate_one for this one
        {"family": "command.v1", "head": "deploy"}, // must still be reachable
    ]);
    let r = doctor_validate(&v);
    let errors: Vec<&DoctorIssue> = r.errors().collect();
    // Exactly one error (the unknown family); the second entry validates clean.
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].path, "$[0].family");
}

#[test]
fn composition_04_empty_command_args_array_is_not_an_error() {
    let r = doctor_validate(&json!([{"family": "command.v1", "head": "deploy", "args": []}]));
    assert!(!r.has_errors(), "empty args is allowed: {:?}", r.issues);
}

#[test]
fn composition_05_capture_targets_only_first_entry_validated_when_targets_not_array() {
    // If targets is not an array, validate_capture short-circuits and
    // doesn't try to index into it. Pin that behavior so a future
    // refactor doesn't add ghost diagnostics like "$[0].targets[0]".
    let v = json!([{"family": "capture.v1", "targets": "todo"}]);
    let r = doctor_validate(&v);
    let paths: Vec<&str> = r.errors().map(|e| e.path.as_str()).collect();
    assert_eq!(paths, vec!["$[0].targets"]);
    assert!(!paths.iter().any(|p| p.contains("targets[")));
}

// ============================================================================
// 3. LOCALIZATION & ENCODING (5 actions)
// ============================================================================

#[test]
fn localization_01_rtl_capture_target_passes_validation() {
    let r = doctor_validate(&json!([{"family": "capture.v1", "targets": ["مهمة"]}]));
    assert!(
        !r.has_errors(),
        "RTL slug should be a Warning at most, not Error: {:?}",
        r.issues
    );
}

#[test]
fn localization_02_emoji_command_head_passes_validation() {
    let r = doctor_validate(&json!([{"family": "command.v1", "head": "🚀"}]));
    assert!(
        !r.has_errors(),
        "emoji head is a single grapheme without whitespace: {:?}",
        r.issues
    );
}

#[test]
fn localization_03_smart_quotes_preserved_in_diagnostic_message() {
    // The doctor's whitespace check uses `slug.chars().any(char::is_whitespace)`;
    // U+201C and U+201D are punctuation, not whitespace, so the slug is
    // technically "valid" (just a custom target → Warning).
    let r =
        doctor_validate(&json!([{"family": "capture.v1", "targets": ["\u{201C}quoted\u{201D}"]}]));
    assert!(!r.has_errors());
    let msgs: Vec<&str> = r.issues.iter().map(|i| i.message.as_str()).collect();
    // The non-built-in warning includes the original slug bytes.
    assert!(
        msgs.iter()
            .any(|m| m.contains('\u{201C}') && m.contains('\u{201D}')),
        "smart quotes should round-trip into the warning message: {msgs:?}"
    );
}

#[test]
fn localization_04_command_head_with_zero_width_joiner_is_not_whitespace() {
    // ZWJ (U+200D) is not whitespace per `char::is_whitespace`, so the
    // doctor accepts it. Pin so a future Unicode-aware whitespace check
    // is a deliberate decision.
    let r = doctor_validate(&json!([{"family": "command.v1", "head": "head\u{200D}joined"}]));
    assert!(
        !r.has_errors(),
        "ZWJ should not trigger whitespace error: {:?}",
        r.issues
    );
}

#[test]
fn localization_05_full_width_space_is_whitespace_for_command_head() {
    // U+3000 (IDEOGRAPHIC SPACE) is whitespace per `char::is_whitespace`.
    // Pin that the doctor catches this — it would silently break command
    // dispatch on the runtime side.
    let r = doctor_validate(&json!([{"family": "command.v1", "head": "head\u{3000}tail"}]));
    assert!(r.has_errors());
    assert_eq!(r.issues[0].path, "$[0].head");
    assert!(r.issues[0].message.contains("whitespace"));
}

// ============================================================================
// 4. DETERMINISM (4 actions)
// ============================================================================

#[test]
fn determinism_01_same_input_yields_equal_report_across_50_runs() {
    let v = json!([
        {"family": "capture.v1", "targets": ["bad slug"]},
        {"family": "command.v1", "head": "deploy"},
        {"family": "command.v1", "head": "deploy"},
    ]);
    let first = doctor_validate(&v);
    for _ in 0..50 {
        let again = doctor_validate(&v);
        assert_eq!(again, first);
    }
}

#[test]
fn determinism_02_report_serializes_to_camelcase_json() {
    let v = json!([{"family": "capture.v2"}]);
    let r = doctor_validate(&v);
    let json_value = serde_json::to_value(&r).expect("serialize");
    // Field name `hasErrors` is on the JSON via... wait — actually the
    // struct fields are `issues` only, no derived flag. Just confirm
    // the issue shape is camelCase (severity / path / message).
    let issues = json_value["issues"].as_array().expect("array");
    assert_eq!(issues.len(), 1);
    assert!(issues[0].get("path").is_some());
    assert!(issues[0].get("severity").is_some());
    assert!(issues[0].get("message").is_some());
    // Severity uses lowercase variants (per #[serde(rename_all = "lowercase")]).
    assert_eq!(issues[0]["severity"], "error");
}

#[test]
fn determinism_03_round_trip_through_serde() {
    let v = json!([{"family": "capture.v1", "targets": ["bad slug"]}]);
    let r = doctor_validate(&v);
    let json_str = serde_json::to_string(&r).expect("serialize");
    let restored: DoctorReport = serde_json::from_str(&json_str).expect("deserialize");
    assert_eq!(restored, r);
}

#[test]
fn determinism_04_warning_vs_error_severity_drives_has_errors() {
    let warning_only = doctor_validate(&json!([{"family": "capture.v1", "targets": ["expense"]}]));
    assert!(!warning_only.has_errors());
    assert!(warning_only
        .issues
        .iter()
        .all(|i| i.severity == DoctorSeverity::Warning));

    let with_error = doctor_validate(&json!([{"family": "unknown"}]));
    assert!(with_error.has_errors());
    assert!(with_error
        .errors()
        .any(|i| i.severity == DoctorSeverity::Error));
}

// ============================================================================
// 5. LIFECYCLE / ORDER (3 actions)
// ============================================================================

#[test]
fn lifecycle_01_swapping_two_entries_only_changes_indices_not_message_content() {
    let a = json!([
        {"family": "capture.v1", "targets": ["bad slug"]},
        {"family": "command.v1", "head": "deploy"}
    ]);
    let b = json!([
        {"family": "command.v1", "head": "deploy"},
        {"family": "capture.v1", "targets": ["bad slug"]}
    ]);
    let ra = doctor_validate(&a);
    let rb = doctor_validate(&b);
    // Same set of error MESSAGES (paths shift, content stays).
    let mut msgs_a: Vec<&str> = ra.errors().map(|e| e.message.as_str()).collect();
    let mut msgs_b: Vec<&str> = rb.errors().map(|e| e.message.as_str()).collect();
    msgs_a.sort();
    msgs_b.sort();
    assert_eq!(msgs_a, msgs_b);
    // But the "bad slug" path moved from $[0].targets[0] → $[1].targets[0].
    assert!(paths_of(&a).contains(&"$[0].targets[0]".to_string()));
    assert!(paths_of(&b).contains(&"$[1].targets[0]".to_string()));
}

#[test]
fn lifecycle_02_adding_a_clean_entry_doesnt_remove_dirty_diagnostics() {
    let dirty = json!([{"family": "capture.v1", "targets": []}]);
    let dirty_with_clean = json!([
        {"family": "capture.v1", "targets": []},
        {"family": "command.v1", "head": "deploy"}
    ]);
    assert_eq!(err_count(&dirty), 1);
    assert_eq!(err_count(&dirty_with_clean), 1);
}

#[test]
fn lifecycle_03_doctor_is_pure_no_side_effects_visible_to_caller() {
    // We can't assert "no side effects" directly, but we can pin that
    // mutating the input AFTER doctor_validate doesn't change the
    // returned report (i.e. the report owns its strings).
    let mut v = json!([{"family": "capture.v1", "targets": ["bad slug"]}]);
    let r1 = doctor_validate(&v);
    *v.get_mut(0).unwrap() = json!({"family": "command.v1", "head": "deploy"});
    let r1_paths_after_mutation: Vec<String> = r1.issues.iter().map(|i| i.path.clone()).collect();
    assert!(r1_paths_after_mutation.contains(&"$[0].targets[0]".to_string()));
}
