//! Run 11 Pass 28 — attacker probe of [[src/menu_syntax/capture_gate.rs#decide_capture_gate]]
//! and [[src/menu_syntax/capture_gate.rs#resolve_capture_schema_for_script]] (added Pass 25).
//! Pure-function attacker — no UI surface.
//!
//! Categories: Boundary, Composition, Resurrection. Actions: 22.
//!
//! Probes the Pass-25 gate that the Pass-26 forbidden/NaN tightening flows
//! through. Targets surface area never attacker-tested before.

use script_kit_gpui::menu_syntax::payload::{
    CaptureAlias, CaptureInvocation, DatePhrase, DateRole,
};
use script_kit_gpui::menu_syntax::{
    builtin_schema, decide_capture_gate, CaptureFieldSchema, CaptureGateDecision, FieldRequirement,
};

fn empty(target: &str) -> CaptureInvocation {
    CaptureInvocation {
        target: target.to_string(),
        alias_form: CaptureAlias::CapturePrefix,
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

fn fully_satisfied_cal() -> CaptureInvocation {
    let mut inv = with_body("cal", "Design review");
    inv.date_phrases.push(DatePhrase {
        role: DateRole::Inferred,
        source: "friday 2pm".to_string(),
        source_span: (0, 10),
    });
    inv
}

// ============================================================================
// BOUNDARY (8 actions) — None / empty / single-field edge cases.
// ============================================================================

#[test]
fn boundary_01_none_schema_returns_allow_for_any_invocation() {
    // Permissive contract: handlers without declared shape must execute.
    let inv = empty("custom-target");
    assert_eq!(decide_capture_gate(&inv, None), CaptureGateDecision::Allow);
}

#[test]
fn boundary_02_none_schema_allow_even_with_garbage_url() {
    let mut inv = empty("custom");
    inv.url = Some("not-a-url".to_string());
    // Without a schema, the URL well-formedness check is bypassed (Allow).
    assert_eq!(decide_capture_gate(&inv, None), CaptureGateDecision::Allow);
}

#[test]
fn boundary_03_empty_schema_required_means_always_allow() {
    let schema = CaptureFieldSchema {
        target: "freeform".to_string(),
        required: vec![],
        optional: vec![],
        forbidden: vec![],
    };
    let inv = empty("freeform");
    assert_eq!(
        decide_capture_gate(&inv, Some(&schema)),
        CaptureGateDecision::Allow
    );
}

#[test]
fn boundary_04_block_missing_carries_all_missing_fields_in_order() {
    // cal requires Body + AnyDate; neither satisfied → both surface in order.
    let inv = empty("cal");
    let schema = builtin_schema("cal").unwrap();
    match decide_capture_gate(&inv, Some(&schema)) {
        CaptureGateDecision::BlockMissing { missing, .. } => {
            assert_eq!(missing.len(), 2);
            assert_eq!(missing[0], FieldRequirement::Body);
            assert_eq!(missing[1], FieldRequirement::AnyDate);
        }
        other => panic!("expected BlockMissing with 2 fields, got {other:?}"),
    }
}

#[test]
fn boundary_05_hud_message_for_link_missing_url_is_human() {
    let inv = empty("link");
    let schema = builtin_schema("link").unwrap();
    match decide_capture_gate(&inv, Some(&schema)) {
        CaptureGateDecision::BlockMissing { hud_message, .. } => {
            // link only requires url, so single-field "needs url" with no oxford.
            assert_eq!(hud_message, ";link needs url");
        }
        other => panic!("expected BlockMissing url, got {other:?}"),
    }
}

#[test]
fn boundary_06_target_with_uppercase_in_invocation_propagates_to_hud() {
    // The HUD uses the invocation target verbatim — case is not normalized.
    // (Schema lookup itself is case-insensitive, but the HUD echoes the user's
    // typed prefix.)
    let mut inv = empty("CAL");
    inv.target = "CAL".to_string();
    let schema = builtin_schema("cal").unwrap();
    match decide_capture_gate(&inv, Some(&schema)) {
        CaptureGateDecision::BlockMissing { hud_message, .. } => {
            assert!(hud_message.starts_with("+CAL needs "), "{hud_message}");
        }
        other => panic!("expected BlockMissing, got {other:?}"),
    }
}

#[test]
fn boundary_07_target_with_special_chars_does_not_crash_hud_format() {
    // Defensive: a `+target` with regex-meta chars should still produce a HUD
    // string; format! is not a regex and these are byte-safe.
    let mut inv = empty("re.*x");
    inv.target = "re.*x".to_string();
    let schema = CaptureFieldSchema {
        target: "re.*x".to_string(),
        required: vec![FieldRequirement::Body],
        optional: vec![],
        forbidden: vec![],
    };
    match decide_capture_gate(&inv, Some(&schema)) {
        CaptureGateDecision::BlockMissing { hud_message, .. } => {
            assert_eq!(hud_message, "+re.*x needs body");
        }
        other => panic!("expected BlockMissing, got {other:?}"),
    }
}

#[test]
fn boundary_08_malformed_kv_amount_takes_precedence_over_missing_required() {
    // Wrong-shape (Malformed) beats missing-shape (Incomplete) per Pass 16/26.
    let schema = CaptureFieldSchema {
        target: "expense".to_string(),
        required: vec![
            FieldRequirement::Body,
            FieldRequirement::Kv("amount".to_string()),
        ],
        optional: vec![],
        forbidden: vec![],
    };
    let mut inv = empty("expense"); // body still missing
    inv.kv.push(("amount".to_string(), "NaN".to_string()));
    match decide_capture_gate(&inv, Some(&schema)) {
        CaptureGateDecision::BlockMalformed { field, .. } => {
            assert_eq!(field, FieldRequirement::Kv("amount".to_string()));
        }
        other => panic!("expected BlockMalformed amount over missing body, got {other:?}"),
    }
}

// ============================================================================
// COMPOSITION (8 actions) — multi-field interactions, precedence, oxford.
// ============================================================================

#[test]
fn composition_09_three_missing_fields_use_oxford_comma() {
    let schema = CaptureFieldSchema {
        target: "triple".to_string(),
        required: vec![
            FieldRequirement::Body,
            FieldRequirement::AnyDate,
            FieldRequirement::Url,
        ],
        optional: vec![],
        forbidden: vec![],
    };
    let inv = empty("triple");
    match decide_capture_gate(&inv, Some(&schema)) {
        CaptureGateDecision::BlockMissing { hud_message, .. } => {
            assert_eq!(hud_message, "+triple needs body, date, and url");
        }
        other => panic!("expected oxford-joined HUD, got {other:?}"),
    }
}

#[test]
fn composition_10_forbidden_url_on_cal_takes_precedence_over_missing() {
    // Pass 26 contract: schema.forbidden runs BEFORE missing_required.
    let mut inv = empty("cal"); // body + date both missing
    inv.url = Some("https://example.com".to_string()); // well-formed but forbidden
    let schema = builtin_schema("cal").unwrap();
    match decide_capture_gate(&inv, Some(&schema)) {
        CaptureGateDecision::BlockMalformed { field, .. } => {
            assert_eq!(field, FieldRequirement::Url);
        }
        other => panic!("expected forbidden Url Malformed, got {other:?}"),
    }
}

#[test]
fn composition_11_url_well_formedness_beats_forbidden_url() {
    // url check (well-formedness) runs BEFORE forbidden sweep, so a +cal
    // with an ill-formed forbidden url surfaces as Malformed.Url with the
    // url-scheme reason — NOT the "not allowed for +cal" reason. Pin the
    // current ordering since both reach the same field.
    let mut inv = empty("cal");
    inv.url = Some("ftp://nope".to_string());
    let schema = builtin_schema("cal").unwrap();
    match decide_capture_gate(&inv, Some(&schema)) {
        CaptureGateDecision::BlockMalformed { field, reason, .. } => {
            assert_eq!(field, FieldRequirement::Url);
            assert!(
                reason.contains("URL must start with http"),
                "well-formedness reason wins, got: {reason}"
            );
        }
        other => panic!("expected Malformed url scheme, got {other:?}"),
    }
}

#[test]
fn composition_12_optional_fields_present_do_not_block() {
    // Optional declarations must never gate Allow — only required + forbidden
    // affect the decision.
    let schema = CaptureFieldSchema {
        target: "todo".to_string(),
        required: vec![FieldRequirement::Body],
        optional: vec![FieldRequirement::Tag, FieldRequirement::Priority],
        forbidden: vec![],
    };
    let mut inv = with_body("todo", "Buy milk");
    inv.tags.push("errands".to_string());
    inv.priority = Some(2u8);
    assert_eq!(
        decide_capture_gate(&inv, Some(&schema)),
        CaptureGateDecision::Allow
    );
}

#[test]
fn composition_13_kv_case_insensitive_satisfaction() {
    // KV requirement matches case-insensitively per
    // FieldRequirement::is_satisfied (capture_schema.rs).
    let schema = CaptureFieldSchema {
        target: "expense".to_string(),
        required: vec![
            FieldRequirement::Body,
            FieldRequirement::Kv("amount".to_string()),
        ],
        optional: vec![],
        forbidden: vec![],
    };
    let mut inv = with_body("expense", "Lunch");
    // Author typed AMOUNT, requirement is amount — must still satisfy.
    inv.kv.push(("AMOUNT".to_string(), "12.50".to_string()));
    assert_eq!(
        decide_capture_gate(&inv, Some(&schema)),
        CaptureGateDecision::Allow
    );
}

#[test]
fn composition_14_date_role_specific_requirement_distinct_from_anydate() {
    // A schema requiring DateRole::Start is NOT satisfied by an Inferred date
    // phrase. This is the explicit-key vs body-suffix-inference distinction.
    let schema = CaptureFieldSchema {
        target: "meeting".to_string(),
        required: vec![
            FieldRequirement::Body,
            FieldRequirement::DateRole(DateRole::Start),
        ],
        optional: vec![],
        forbidden: vec![],
    };
    let mut inv = with_body("meeting", "Standup");
    inv.date_phrases.push(DatePhrase {
        role: DateRole::Inferred, // wrong role!
        source: "9am".to_string(),
        source_span: (0, 3),
    });
    match decide_capture_gate(&inv, Some(&schema)) {
        CaptureGateDecision::BlockMissing { missing, .. } => {
            assert!(missing
                .iter()
                .any(|m| matches!(m, FieldRequirement::DateRole(DateRole::Start))));
        }
        other => panic!("expected BlockMissing start time, got {other:?}"),
    }
}

#[test]
fn composition_15_date_role_satisfied_when_role_matches() {
    // Same schema as above but the date phrase carries the right role → Allow.
    let schema = CaptureFieldSchema {
        target: "meeting".to_string(),
        required: vec![
            FieldRequirement::Body,
            FieldRequirement::DateRole(DateRole::Start),
        ],
        optional: vec![],
        forbidden: vec![],
    };
    let mut inv = with_body("meeting", "Standup");
    inv.date_phrases.push(DatePhrase {
        role: DateRole::Start,
        source: "9am".to_string(),
        source_span: (0, 3),
    });
    assert_eq!(
        decide_capture_gate(&inv, Some(&schema)),
        CaptureGateDecision::Allow
    );
}

#[test]
fn composition_16_empty_kv_value_treated_as_missing_not_malformed() {
    // Empty kv value is "not provided" — falls through Malformed checks (no
    // amount-shape concern) and surfaces as BlockMissing.
    let schema = CaptureFieldSchema {
        target: "expense".to_string(),
        required: vec![
            FieldRequirement::Body,
            FieldRequirement::Kv("amount".to_string()),
        ],
        optional: vec![],
        forbidden: vec![],
    };
    let mut inv = with_body("expense", "Lunch");
    inv.kv.push(("amount".to_string(), "  ".to_string()));
    match decide_capture_gate(&inv, Some(&schema)) {
        CaptureGateDecision::BlockMissing { missing, .. } => {
            assert!(missing
                .iter()
                .any(|m| matches!(m, FieldRequirement::Kv(k) if k == "amount")));
        }
        other => panic!("expected BlockMissing for empty amount, got {other:?}"),
    }
}

// ============================================================================
// RESURRECTION (6 actions) — idempotence, repeated calls, cloning.
// ============================================================================

#[test]
fn resurrection_17_same_inputs_yield_same_decision() {
    let inv = fully_satisfied_cal();
    let schema = builtin_schema("cal").unwrap();
    let d1 = decide_capture_gate(&inv, Some(&schema));
    let d2 = decide_capture_gate(&inv, Some(&schema));
    let d3 = decide_capture_gate(&inv, Some(&schema));
    assert_eq!(d1, d2);
    assert_eq!(d2, d3);
    assert_eq!(d1, CaptureGateDecision::Allow);
}

#[test]
fn resurrection_18_clone_schema_yields_equal_decision() {
    let inv = empty("cal");
    let schema = builtin_schema("cal").unwrap();
    let cloned = schema.clone();
    let d1 = decide_capture_gate(&inv, Some(&schema));
    let d2 = decide_capture_gate(&inv, Some(&cloned));
    assert_eq!(d1, d2);
}

#[test]
fn resurrection_19_clone_decision_equality() {
    let inv = empty("note");
    let schema = builtin_schema("note").unwrap();
    let d = decide_capture_gate(&inv, Some(&schema));
    let cloned = d.clone();
    assert_eq!(d, cloned);
    assert!(!d.is_allow());
}

#[test]
fn resurrection_20_mutation_changes_decision_no_residual_state() {
    let mut inv = empty("todo");
    let schema = builtin_schema("todo").unwrap();
    // Empty body → BlockMissing.
    assert!(matches!(
        decide_capture_gate(&inv, Some(&schema)),
        CaptureGateDecision::BlockMissing { .. }
    ));
    // Set body → Allow on the SAME inputs (no cached prior decision).
    inv.body = "Buy milk".to_string();
    assert_eq!(
        decide_capture_gate(&inv, Some(&schema)),
        CaptureGateDecision::Allow
    );
    // Clear body again → back to BlockMissing.
    inv.body = String::new();
    assert!(matches!(
        decide_capture_gate(&inv, Some(&schema)),
        CaptureGateDecision::BlockMissing { .. }
    ));
}

#[test]
fn resurrection_21_decision_equality_across_constructors() {
    // Two BlockMissing decisions with the same target + missing list are
    // structurally equal — important for snapshot/diff tooling.
    let inv_a = empty("cal");
    let inv_b = empty("cal");
    let schema = builtin_schema("cal").unwrap();
    assert_eq!(
        decide_capture_gate(&inv_a, Some(&schema)),
        decide_capture_gate(&inv_b, Some(&schema)),
    );
}

#[test]
fn resurrection_22_is_allow_predicate_consistent_with_match() {
    let cases: Vec<(CaptureInvocation, CaptureFieldSchema, bool)> = vec![
        (fully_satisfied_cal(), builtin_schema("cal").unwrap(), true),
        (empty("cal"), builtin_schema("cal").unwrap(), false),
        (
            with_body("note", "x"),
            builtin_schema("note").unwrap(),
            true,
        ),
    ];
    for (inv, schema, expect_allow) in cases {
        let d = decide_capture_gate(&inv, Some(&schema));
        assert_eq!(
            d.is_allow(),
            expect_allow,
            "is_allow() must equal `matches!(d, Allow)` for {inv:?}"
        );
        assert_eq!(matches!(d, CaptureGateDecision::Allow), d.is_allow());
    }
}
