//! Run 11 Pass 24 — attacker probe of [[src/menu_syntax/metadata.rs#parse_field_requirement_token]]
//! and [[src/menu_syntax/metadata.rs#dynamic_capture_schema_from_spec]] (added Pass 21).
//! Pure-function attacker — no UI surface.
//!
//! Categories: Boundary, Composition, Resurrection. Actions: 24.
//!
//! One `[?]` near-anomaly surfaced (filed in stories.md as
//! `dynamic-schema-no-dedup-across-required-optional`).

use script_kit_gpui::menu_syntax::capture_schema::FieldRequirement;
use script_kit_gpui::menu_syntax::metadata::{
    dynamic_capture_schema_from_spec, parse_field_requirement_token,
};
use script_kit_gpui::menu_syntax::payload::{DateRole, MenuSyntaxHandlerSpec};

fn capture_spec_with(
    target: &str,
    required: Vec<&str>,
    optional: Vec<&str>,
    forbidden: Vec<&str>,
) -> MenuSyntaxHandlerSpec {
    MenuSyntaxHandlerSpec {
        family: "capture.v1".to_string(),
        targets: vec![target.to_string()],
        required: required.into_iter().map(String::from).collect(),
        optional: optional.into_iter().map(String::from).collect(),
        forbidden: forbidden.into_iter().map(String::from).collect(),
        ..Default::default()
    }
}

// ============================================================================
// BOUNDARY (10 actions)
// ============================================================================

#[test]
fn boundary_01_empty_token_yields_none() {
    assert_eq!(parse_field_requirement_token(""), None);
}

#[test]
fn boundary_02_whitespace_only_token_yields_none() {
    assert_eq!(parse_field_requirement_token("   \t\n"), None);
}

#[test]
fn boundary_03_kv_with_empty_key_yields_none() {
    assert_eq!(parse_field_requirement_token("kv:"), None);
}

#[test]
fn boundary_04_kv_with_whitespace_only_key_yields_none() {
    assert_eq!(parse_field_requirement_token("kv:   "), None);
}

#[test]
fn boundary_05_kv_with_emoji_key_preserved() {
    assert_eq!(
        parse_field_requirement_token("kv:\u{1F680}rocket"),
        Some(FieldRequirement::Kv("\u{1F680}rocket".to_string()))
    );
}

#[test]
fn boundary_06_date_role_uppercase_works() {
    assert_eq!(
        parse_field_requirement_token("DATE:Start"),
        Some(FieldRequirement::DateRole(DateRole::Start))
    );
}

#[test]
fn boundary_07_kv_with_internal_colon_in_key_preserved() {
    // strip_prefix("kv:") removes only the first prefix; the rest of the
    // key (including additional colons) is preserved verbatim.
    assert_eq!(
        parse_field_requirement_token("kv:meta:vendor"),
        Some(FieldRequirement::Kv("meta:vendor".to_string()))
    );
}

#[test]
fn boundary_08_date_with_empty_role_yields_none() {
    assert_eq!(parse_field_requirement_token("date:"), None);
    assert_eq!(parse_field_requirement_token("date:   "), None);
}

#[test]
fn boundary_09_tag_alias_recognized() {
    assert_eq!(
        parse_field_requirement_token("tags"),
        Some(FieldRequirement::Tag)
    );
    assert_eq!(
        parse_field_requirement_token("TAG"),
        Some(FieldRequirement::Tag)
    );
}

#[test]
fn boundary_10_any_date_aliases_recognized() {
    assert_eq!(
        parse_field_requirement_token("date"),
        Some(FieldRequirement::AnyDate)
    );
    assert_eq!(
        parse_field_requirement_token("anydate"),
        Some(FieldRequirement::AnyDate)
    );
    assert_eq!(
        parse_field_requirement_token("ANY-DATE"),
        Some(FieldRequirement::AnyDate)
    );
}

// ============================================================================
// COMPOSITION (8 actions)
// ============================================================================

#[test]
fn composition_11_all_empty_lists_yields_empty_schema() {
    let spec = capture_spec_with("expense", vec![], vec![], vec![]);
    let schema = dynamic_capture_schema_from_spec(&spec).expect("schema");
    assert_eq!(schema.target, "expense");
    assert!(schema.required.is_empty());
    assert!(schema.optional.is_empty());
    assert!(schema.forbidden.is_empty());
}

#[test]
fn composition_12_whitespace_only_target_yields_none() {
    let spec = capture_spec_with("   ", vec!["body"], vec![], vec![]);
    assert!(dynamic_capture_schema_from_spec(&spec).is_none());
}

#[test]
fn composition_13_first_non_empty_target_is_used() {
    let spec = MenuSyntaxHandlerSpec {
        family: "capture.v1".to_string(),
        targets: vec!["".to_string(), "valid".to_string(), "second".to_string()],
        required: vec!["body".to_string()],
        ..Default::default()
    };
    let schema = dynamic_capture_schema_from_spec(&spec).expect("schema");
    assert_eq!(schema.target, "valid");
}

#[test]
fn composition_14_duplicate_required_tokens_dedupe_keep_first_pinned() {
    // Run 11 Pass #33 (Fix): `[?] dynamic-schema-no-dedup-across-required-optional`
    // CLOSED. Within-list dedup keeps the first occurrence; three identical
    // `body` tokens collapse to a single `FieldRequirement::Body`.
    let spec = capture_spec_with("expense", vec!["body", "body", "body"], vec![], vec![]);
    let schema = dynamic_capture_schema_from_spec(&spec).expect("schema");
    assert_eq!(schema.required.len(), 1);
    assert!(matches!(schema.required[0], FieldRequirement::Body));
}

#[test]
fn composition_14b_cross_list_required_wins_over_optional_and_forbidden_pinned() {
    // Cross-list precedence: required > optional > forbidden. A `kv:amount`
    // declared in all three lists ends up only in `required`. This is the
    // coherence-bug shape `doctor.rs` should also warn about, but the
    // schema itself silently resolves it the same way.
    let spec = capture_spec_with(
        "expense",
        vec!["kv:amount", "body"],
        vec!["kv:amount", "tag"],
        vec!["kv:amount", "url"],
    );
    let schema = dynamic_capture_schema_from_spec(&spec).expect("schema");
    assert!(schema
        .required
        .iter()
        .any(|r| matches!(r, FieldRequirement::Kv(k) if k == "amount")));
    assert!(!schema
        .optional
        .iter()
        .any(|r| matches!(r, FieldRequirement::Kv(k) if k == "amount")));
    assert!(!schema
        .forbidden
        .iter()
        .any(|r| matches!(r, FieldRequirement::Kv(k) if k == "amount")));
    // Falsifier guard: distinct fields in each list still surface.
    assert!(schema
        .optional
        .iter()
        .any(|r| matches!(r, FieldRequirement::Tag)));
    assert!(schema
        .forbidden
        .iter()
        .any(|r| matches!(r, FieldRequirement::Url)));
}

#[test]
fn composition_14c_optional_wins_over_forbidden_when_required_silent_pinned() {
    // When a token only appears in optional + forbidden (not required),
    // optional wins. Falsifier: if dedup ran the other direction, the
    // forbidden Url would survive instead.
    let spec = capture_spec_with("expense", vec!["body"], vec!["url"], vec!["url"]);
    let schema = dynamic_capture_schema_from_spec(&spec).expect("schema");
    assert!(schema
        .optional
        .iter()
        .any(|r| matches!(r, FieldRequirement::Url)));
    assert!(schema.forbidden.is_empty());
}

#[test]
fn composition_14d_distinct_fields_across_lists_all_survive_pinned() {
    // Falsifier for over-dedup: when no token overlaps across lists, all
    // three lists keep their full content.
    let spec = capture_spec_with("expense", vec!["body"], vec!["tag"], vec!["url"]);
    let schema = dynamic_capture_schema_from_spec(&spec).expect("schema");
    assert_eq!(schema.required.len(), 1);
    assert_eq!(schema.optional.len(), 1);
    assert_eq!(schema.forbidden.len(), 1);
}

#[test]
fn composition_15_non_capture_family_yields_none() {
    let spec = MenuSyntaxHandlerSpec {
        family: "skill.v1".to_string(),
        targets: vec!["expense".to_string()],
        required: vec!["body".to_string()],
        ..Default::default()
    };
    assert!(dynamic_capture_schema_from_spec(&spec).is_none());
}

#[test]
fn composition_16_target_lowercased_in_output_schema() {
    let spec = capture_spec_with("EXPENSE", vec!["body"], vec![], vec![]);
    let schema = dynamic_capture_schema_from_spec(&spec).expect("schema");
    assert_eq!(schema.target, "expense");
}

#[test]
fn composition_17_long_required_list_dedupes_to_single_entry_pinned() {
    // Run 11 Pass #33 (Fix): post-dedup a 50-token list of identical
    // `kv:k` entries collapses to one. Falsifier: removing the dedup loop
    // would push this back to 50.
    let mut required: Vec<&str> = Vec::new();
    for _ in 0..50 {
        required.push("kv:k");
    }
    let spec = capture_spec_with("expense", required, vec![], vec![]);
    let schema = dynamic_capture_schema_from_spec(&spec).expect("schema");
    assert_eq!(schema.required.len(), 1);
    assert_eq!(schema.required[0], FieldRequirement::Kv("k".to_string()));
}

#[test]
fn composition_18_unknown_tokens_dropped_silently() {
    let spec = capture_spec_with(
        "expense",
        vec!["body", "bogus", "kv:amount", "also-bogus"],
        vec!["nope", "tag"],
        vec!["url", "weird"],
    );
    let schema = dynamic_capture_schema_from_spec(&spec).expect("schema");
    assert_eq!(
        schema.required,
        vec![
            FieldRequirement::Body,
            FieldRequirement::Kv("amount".to_string())
        ]
    );
    assert_eq!(schema.optional, vec![FieldRequirement::Tag]);
    assert_eq!(schema.forbidden, vec![FieldRequirement::Url]);
}

// ============================================================================
// RESURRECTION / IDEMPOTENCE (6 actions)
// ============================================================================

#[test]
fn resurrection_19_same_spec_parsed_twice_is_equal() {
    let spec = capture_spec_with(
        "expense",
        vec!["body", "kv:amount"],
        vec!["tag"],
        vec!["url"],
    );
    let s1 = dynamic_capture_schema_from_spec(&spec);
    let s2 = dynamic_capture_schema_from_spec(&spec);
    assert_eq!(s1, s2);
    assert!(s1.is_some());
}

#[test]
fn resurrection_20_clone_yields_equal_schema() {
    let spec = capture_spec_with("expense", vec!["body"], vec![], vec![]);
    let cloned = spec.clone();
    assert_eq!(
        dynamic_capture_schema_from_spec(&spec),
        dynamic_capture_schema_from_spec(&cloned)
    );
}

#[test]
fn resurrection_21_mutating_target_to_empty_returns_none() {
    let mut spec = capture_spec_with("expense", vec!["body"], vec![], vec![]);
    assert!(dynamic_capture_schema_from_spec(&spec).is_some());
    spec.targets = vec![String::new()];
    assert!(dynamic_capture_schema_from_spec(&spec).is_none());
}

#[test]
fn resurrection_22_empty_required_list_yields_empty_required_vec() {
    let spec = capture_spec_with("expense", vec![], vec!["body"], vec![]);
    let schema = dynamic_capture_schema_from_spec(&spec).unwrap();
    assert!(schema.required.is_empty());
    assert_eq!(schema.optional, vec![FieldRequirement::Body]);
}

#[test]
fn resurrection_23_kv_in_both_required_and_optional_required_wins_pinned() {
    // Run 11 Pass #33 (Fix): same kv key in both required and optional →
    // required wins, optional is dropped. The doctor surface should still
    // warn about this as an authoring smell, but the extractor resolves
    // the precedence directly.
    let spec = capture_spec_with("expense", vec!["kv:amount"], vec!["kv:amount"], vec![]);
    let schema = dynamic_capture_schema_from_spec(&spec).unwrap();
    assert_eq!(
        schema.required,
        vec![FieldRequirement::Kv("amount".to_string())]
    );
    assert!(schema.optional.is_empty());
}

#[test]
fn resurrection_24_repeated_calls_dont_accumulate_state() {
    let spec = capture_spec_with("expense", vec!["body"], vec![], vec![]);
    let s1 = dynamic_capture_schema_from_spec(&spec).unwrap();
    let s2 = dynamic_capture_schema_from_spec(&spec).unwrap();
    let s3 = dynamic_capture_schema_from_spec(&spec).unwrap();
    assert_eq!(s1, s2);
    assert_eq!(s2, s3);
    assert_eq!(s1.required.len(), 1);
}
