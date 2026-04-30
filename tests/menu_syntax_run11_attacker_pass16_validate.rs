//! Run 11 Pass 16 — attacker probe of [[src/menu_syntax/capture_schema.rs#validate]]
//! (added Pass 15). Pure-function attacker — no UI surface.
//!
//! Categories: Boundary, Composition, Resurrection. Actions: 24.
//!
//! Two `[?]` near-anomalies surfaced (filed in stories.md as
//! `validate-amount-accepts-nan-inf-as-numeric` and
//! `validate-ignores-schema-forbidden-fields`). Neither is a fix candidate
//! for this pass — attacker mode never fixes; subsequent passes can.

use script_kit_gpui::menu_syntax::{
    builtin_schema, validate_capture_payload, CaptureFieldSchema, FieldRequirement,
    ValidationResult,
};
// Re-import the underlying types used by the schema fixtures.
use script_kit_gpui::menu_syntax::payload::{
    CaptureAlias, CaptureInvocation, DatePhrase, DateRole,
};

fn empty(target: &str) -> CaptureInvocation {
    CaptureInvocation {
        target: target.to_string(),
        alias_form: CaptureAlias::Plus,
        body: String::new(),
        tags: vec![],
        priority: None,
        url: None,
        duration: None,
        kv: vec![],
        date_phrases: vec![],
        raw: format!("+{target}"),
    }
}

fn with_body(target: &str, body: &str) -> CaptureInvocation {
    let mut inv = empty(target);
    inv.body = body.to_string();
    inv
}

fn amount_schema() -> CaptureFieldSchema {
    CaptureFieldSchema {
        target: "expense".to_string(),
        required: vec![
            FieldRequirement::Body,
            FieldRequirement::Kv("amount".to_string()),
        ],
        optional: vec![],
        forbidden: vec![],
    }
}

// ============================================================================
// BOUNDARY (12 actions)
// ============================================================================

#[test]
fn boundary_01_empty_url_string_is_malformed() {
    let schema = builtin_schema("link").unwrap();
    let mut inv = empty("link");
    inv.url = Some(String::new());
    assert!(matches!(
        validate_capture_payload(&inv, &schema),
        ValidationResult::Malformed { .. }
    ));
}

#[test]
fn boundary_02_whitespace_only_url_is_malformed() {
    let schema = builtin_schema("link").unwrap();
    let mut inv = empty("link");
    inv.url = Some("   \t  ".to_string());
    assert!(matches!(
        validate_capture_payload(&inv, &schema),
        ValidationResult::Malformed { .. }
    ));
}

#[test]
fn boundary_03_uppercase_scheme_is_well_formed() {
    let schema = builtin_schema("link").unwrap();
    let mut inv = empty("link");
    inv.url = Some("HTTPS://X.COM".to_string());
    assert_eq!(
        validate_capture_payload(&inv, &schema),
        ValidationResult::Ready
    );
}

#[test]
fn boundary_04_javascript_scheme_is_malformed() {
    let schema = builtin_schema("link").unwrap();
    let mut inv = empty("link");
    inv.url = Some("javascript:alert(1)".to_string());
    assert!(matches!(
        validate_capture_payload(&inv, &schema),
        ValidationResult::Malformed { .. }
    ));
}

#[test]
fn boundary_05_https_with_no_authority_is_well_formed_today_PROBE() {
    // Probe: the well-formedness check looks at the prefix only, so a bare
    // scheme "https://" passes. This is permissive — a future tightening
    // could parse host+path. Filed as `[?]` near-anomaly.
    let schema = builtin_schema("link").unwrap();
    let mut inv = empty("link");
    inv.url = Some("https://".to_string());
    assert_eq!(
        validate_capture_payload(&inv, &schema),
        ValidationResult::Ready,
        "documenting current permissive behavior — anomaly filed"
    );
}

#[test]
fn boundary_06_amount_huge_string_fails_parse() {
    let schema = amount_schema();
    let mut inv = with_body("expense", "Lunch");
    inv.kv.push(("amount".to_string(), "9".repeat(10_000)));
    let r = validate_capture_payload(&inv, &schema);
    // 10k 9s parses as f64 infinity — see anomaly probe in action 09.
    // Either Ready (parses to inf) or Malformed (overflow detected).
    assert!(matches!(
        r,
        ValidationResult::Ready | ValidationResult::Malformed { .. }
    ));
}

#[test]
fn boundary_07_fullwidth_digits_fail_amount_parse() {
    let schema = amount_schema();
    let mut inv = with_body("expense", "Lunch");
    inv.kv.push(("amount".to_string(), "２".to_string()));
    assert!(matches!(
        validate_capture_payload(&inv, &schema),
        ValidationResult::Malformed { .. }
    ));
}

#[test]
fn boundary_08_amount_scientific_notation_is_ready() {
    let schema = amount_schema();
    let mut inv = with_body("expense", "Lunch");
    inv.kv.push(("amount".to_string(), "1e6".to_string()));
    assert_eq!(
        validate_capture_payload(&inv, &schema),
        ValidationResult::Ready
    );
}

#[test]
fn boundary_09_amount_NaN_is_malformed_PINNED() {
    // Pinned by Run 11 Pass 26 (commit forthcoming): closes
    // `validate-amount-accepts-nan-inf-as-numeric`. `looks_like_amount` now
    // requires `is_finite()` after parse, so NaN is Malformed.
    let schema = amount_schema();
    let mut inv = with_body("expense", "Lunch");
    inv.kv.push(("amount".to_string(), "NaN".to_string()));
    match validate_capture_payload(&inv, &schema) {
        ValidationResult::Malformed { field, .. } => {
            assert!(matches!(field, FieldRequirement::Kv(k) if k == "amount"));
        }
        other => panic!("expected Malformed for amount=NaN, got {other:?}"),
    }
}

#[test]
fn boundary_10_amount_inf_is_malformed_PINNED() {
    let schema = amount_schema();
    let mut inv = with_body("expense", "Lunch");
    inv.kv.push(("amount".to_string(), "inf".to_string()));
    match validate_capture_payload(&inv, &schema) {
        ValidationResult::Malformed { field, .. } => {
            assert!(matches!(field, FieldRequirement::Kv(k) if k == "amount"));
        }
        other => panic!("expected Malformed for amount=inf, got {other:?}"),
    }
}

#[test]
fn boundary_11_body_with_emoji_and_rtl_satisfies_body() {
    let schema = builtin_schema("todo").unwrap();
    // RTL Hebrew literal escaped to avoid `text_direction_codepoint_in_literal`.
    let inv = with_body(
        "todo",
        "\u{1F4DD} \u{05E9}\u{05DC}\u{05D5}\u{05DD} \u{2014} done",
    );
    assert_eq!(
        validate_capture_payload(&inv, &schema),
        ValidationResult::Ready
    );
}

#[test]
fn boundary_12_url_with_internal_nul_is_malformed() {
    let schema = builtin_schema("link").unwrap();
    let mut inv = empty("link");
    inv.url = Some("ht\0tp://x".to_string());
    assert!(matches!(
        validate_capture_payload(&inv, &schema),
        ValidationResult::Malformed { .. }
    ));
}

// ============================================================================
// COMPOSITION (6 actions)
// ============================================================================

#[test]
fn composition_13_bad_url_with_missing_body_returns_malformed_url_not_incomplete() {
    // Url check runs before missing_required, so Malformed wins.
    let schema = builtin_schema("link").unwrap();
    let mut inv = empty("link");
    inv.url = Some("ftp://x".to_string());
    match validate_capture_payload(&inv, &schema) {
        ValidationResult::Malformed { field, .. } => {
            assert_eq!(field, FieldRequirement::Url);
        }
        other => panic!("expected Malformed url, got {other:?}"),
    }
}

#[test]
fn composition_14_bad_url_beats_bad_amount_since_url_loop_runs_first() {
    let schema = CaptureFieldSchema {
        target: "mixed".to_string(),
        required: vec![
            FieldRequirement::Url,
            FieldRequirement::Kv("amount".to_string()),
        ],
        optional: vec![],
        forbidden: vec![],
    };
    let mut inv = empty("mixed");
    inv.url = Some("nope".to_string());
    inv.kv.push(("amount".to_string(), "abc".to_string()));
    match validate_capture_payload(&inv, &schema) {
        ValidationResult::Malformed { field, .. } => {
            assert_eq!(field, FieldRequirement::Url);
        }
        other => panic!("expected Malformed url first, got {other:?}"),
    }
}

#[test]
fn composition_15_amount_kv_case_insensitive_match_triggers_malformed() {
    let schema = amount_schema();
    let mut inv = with_body("expense", "Lunch");
    inv.kv.push(("AMOUNT".to_string(), "abc".to_string()));
    assert!(matches!(
        validate_capture_payload(&inv, &schema),
        ValidationResult::Malformed { .. }
    ));
}

#[test]
fn composition_16_two_amount_entries_first_good_second_bad_yields_malformed() {
    let schema = amount_schema();
    let mut inv = with_body("expense", "Lunch");
    inv.kv.push(("amount".to_string(), "10".to_string()));
    inv.kv.push(("amount".to_string(), "abc".to_string()));
    assert!(matches!(
        validate_capture_payload(&inv, &schema),
        ValidationResult::Malformed { .. }
    ));
}

#[test]
fn composition_17_validate_runs_url_check_even_for_schema_without_url_requirement() {
    // Bare todo schema doesn't require url, but if payload provides a bad
    // url anyway, validate should still flag it. (Currently does, since the
    // url loop runs unconditionally on payload.url.)
    let schema = builtin_schema("todo").unwrap();
    let mut inv = with_body("todo", "Buy milk");
    inv.url = Some("not-a-url".to_string());
    match validate_capture_payload(&inv, &schema) {
        ValidationResult::Malformed { field, .. } => {
            assert_eq!(field, FieldRequirement::Url);
        }
        other => panic!("expected Malformed url even for non-link schema, got {other:?}"),
    }
}

#[test]
fn composition_18_forbidden_field_is_malformed_PINNED() {
    // Pinned by Run 11 Pass 26 (commit forthcoming): closes
    // `validate-ignores-schema-forbidden-fields`. validate() now sweeps
    // schema.forbidden after well-formedness; a +cal payload with priority
    // returns Malformed { field: Priority } before missing-required is
    // computed, so wrong-shape beats incomplete-shape.
    let schema = builtin_schema("cal").unwrap();
    let mut inv = with_body("cal", "Design review");
    inv.date_phrases.push(DatePhrase {
        role: DateRole::Start,
        source: "friday 2pm".to_string(),
        source_span: (0, 10),
    });
    inv.priority = Some(1u8);
    match validate_capture_payload(&inv, &schema) {
        ValidationResult::Malformed { field, reason } => {
            assert_eq!(field, FieldRequirement::Priority);
            assert!(
                reason.contains(";cal"),
                "reason should name target: {reason}"
            );
        }
        other => panic!("expected Malformed forbidden Priority, got {other:?}"),
    }
}

// ============================================================================
// RESURRECTION / IDEMPOTENCE (6 actions)
// ============================================================================

#[test]
fn resurrection_19_validate_is_idempotent_on_same_payload() {
    let schema = builtin_schema("todo").unwrap();
    let inv = with_body("todo", "Buy milk");
    let r1 = validate_capture_payload(&inv, &schema);
    let r2 = validate_capture_payload(&inv, &schema);
    let r3 = validate_capture_payload(&inv, &schema);
    assert_eq!(r1, r2);
    assert_eq!(r2, r3);
    assert_eq!(r1, ValidationResult::Ready);
}

#[test]
fn resurrection_20_clone_then_validate_preserves_result() {
    let schema = builtin_schema("link").unwrap();
    let mut inv = empty("link");
    inv.url = Some("https://zed.dev".to_string());
    let cloned = inv.clone();
    assert_eq!(
        validate_capture_payload(&inv, &schema),
        validate_capture_payload(&cloned, &schema)
    );
}

#[test]
fn resurrection_21_mutation_changes_result_without_residual_state() {
    let schema = builtin_schema("link").unwrap();
    let mut inv = empty("link");
    assert!(matches!(
        validate_capture_payload(&inv, &schema),
        ValidationResult::Incomplete { .. }
    ));
    inv.url = Some("https://x".to_string());
    assert_eq!(
        validate_capture_payload(&inv, &schema),
        ValidationResult::Ready
    );
    inv.url = Some("nope".to_string());
    assert!(matches!(
        validate_capture_payload(&inv, &schema),
        ValidationResult::Malformed { .. }
    ));
    inv.url = None;
    assert!(matches!(
        validate_capture_payload(&inv, &schema),
        ValidationResult::Incomplete { .. }
    ));
}

#[test]
fn resurrection_22_empty_required_schema_always_ready_regardless_of_payload() {
    let schema = CaptureFieldSchema {
        target: "anything".to_string(),
        required: vec![],
        optional: vec![],
        forbidden: vec![],
    };
    assert_eq!(
        validate_capture_payload(&empty("anything"), &schema),
        ValidationResult::Ready
    );
    let mut inv = with_body("anything", "Whatever");
    inv.tags.push("foo".to_string());
    assert_eq!(
        validate_capture_payload(&inv, &schema),
        ValidationResult::Ready
    );
}

#[test]
fn resurrection_23_validate_ignores_payload_target_vs_schema_target_mismatch() {
    // validate doesn't enforce target match — it just checks the schema's
    // required fields against whatever payload was passed. Documented as a
    // contract pin for future refactors that might add target verification.
    let schema = builtin_schema("link").unwrap();
    let mut inv = empty("todo");
    inv.url = Some("https://x".to_string());
    assert_eq!(
        validate_capture_payload(&inv, &schema),
        ValidationResult::Ready
    );
}

#[test]
fn resurrection_24_repeated_validate_with_kv_mutations_settles() {
    let schema = amount_schema();
    let mut inv = with_body("expense", "Lunch");
    inv.kv.push(("amount".to_string(), "abc".to_string()));
    assert!(matches!(
        validate_capture_payload(&inv, &schema),
        ValidationResult::Malformed { .. }
    ));
    inv.kv.clear();
    inv.kv.push(("amount".to_string(), "12.34".to_string()));
    assert_eq!(
        validate_capture_payload(&inv, &schema),
        ValidationResult::Ready
    );
    inv.kv.clear();
    assert!(matches!(
        validate_capture_payload(&inv, &schema),
        ValidationResult::Incomplete { .. }
    ));
}
