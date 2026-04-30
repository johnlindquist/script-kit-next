//! Run 11 Pass 20 — attacker probe of [[src/menu_syntax/skill.rs#skill_specs_from_value]]
//! (added Pass 18). Pure-function attacker — no UI surface.
//!
//! Categories: Boundary, Composition, Resurrection. Actions: 24.
//!
//! Two `[?]` near-anomalies surfaced (filed in stories.md):
//!   - `skill-spec-family-filter-case-sensitive` (uppercase "SKILL.V1" filtered)
//!   - `skill-spec-no-dedup-on-duplicate-slugs` (multiple identical slugs all surface)

use script_kit_gpui::menu_syntax::{skill_specs_from_value, SkillSpec};
use serde_json::json;

// ============================================================================
// BOUNDARY (10 actions)
// ============================================================================

#[test]
fn boundary_01_empty_array_yields_empty() {
    assert!(skill_specs_from_value(&json!([])).is_empty());
}

#[test]
fn boundary_02_array_of_arrays_yields_empty() {
    let v = json!([[{ "family": "skill.v1", "slug": "x" }]]);
    assert!(skill_specs_from_value(&v).is_empty());
}

#[test]
fn boundary_03_huge_slug_preserved() {
    let big = "a".repeat(10_000);
    let v = json!({ "family": "skill.v1", "slug": big });
    let specs = skill_specs_from_value(&v);
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].slug.len(), 10_000);
}

#[test]
fn boundary_04_slug_with_nul_byte_preserved() {
    let v = json!({ "family": "skill.v1", "slug": "a\0b" });
    let specs = skill_specs_from_value(&v);
    assert_eq!(specs[0].slug, "a\0b");
}

#[test]
fn boundary_05_slug_with_emoji_preserved_after_trim() {
    let v = json!({ "family": "skill.v1", "slug": "  \u{1F680}rocket  " });
    let specs = skill_specs_from_value(&v);
    assert_eq!(specs[0].slug, "\u{1F680}rocket");
}

#[test]
fn boundary_06_context_requirements_string_instead_of_array_yields_empty() {
    let v = json!({
        "family": "skill.v1",
        "slug": "x",
        "contextRequirements": "currentFile",
    });
    let specs = skill_specs_from_value(&v);
    assert!(specs[0].context_requirements.is_empty());
}

#[test]
fn boundary_07_context_requirements_with_nested_arrays_drops_nested() {
    let v = json!({
        "family": "skill.v1",
        "slug": "x",
        "contextRequirements": ["a", ["b", "c"], "d"],
    });
    let specs = skill_specs_from_value(&v);
    assert_eq!(specs[0].context_requirements, vec!["a", "d"]);
}

#[test]
fn boundary_08_context_requirements_empty_array_yields_empty_vec() {
    let v = json!({
        "family": "skill.v1",
        "slug": "x",
        "contextRequirements": [],
    });
    let specs = skill_specs_from_value(&v);
    assert!(specs[0].context_requirements.is_empty());
}

#[test]
fn boundary_09_label_as_number_is_ignored() {
    let v = json!({ "family": "skill.v1", "slug": "x", "label": 42 });
    let specs = skill_specs_from_value(&v);
    assert!(specs[0].label.is_none());
}

#[test]
fn boundary_10_uppercase_family_matches_case_insensitively_PINNED() {
    // Pinned by Run 11 Pass 30: closes
    // `skill-spec-family-filter-case-sensitive`. The family check now uses
    // `eq_ignore_ascii_case("skill.v1")`, so SKILL.V1 / Skill.V1 / sKiLl.v1
    // all extract cleanly. Falsifier from the [?]: a lowercase `skill.v1`
    // must still match — guarded by the lib `family_match_is_case_insensitive`
    // test in src/menu_syntax/skill.rs.
    let v = json!({ "family": "SKILL.V1", "slug": "x" });
    let specs = skill_specs_from_value(&v);
    assert_eq!(specs.len(), 1, "SKILL.V1 must match case-insensitively");
    assert_eq!(specs[0].slug, "x");
}

// ============================================================================
// COMPOSITION (8 actions)
// ============================================================================

#[test]
fn composition_11_duplicate_slugs_dedupe_first_wins_PINNED() {
    // Pinned by Run 11 Pass 30: closes `skill-spec-no-dedup-on-duplicate-slugs`.
    // Duplicate slugs collapse to one entry; declaration-order wins so the
    // FIRST occurrence (no label here) is kept and the LATER `label: "later one"`
    // entry is dropped.
    let v = json!([
        { "family": "skill.v1", "slug": "review" },
        { "family": "skill.v1", "slug": "review", "label": "later one" },
    ]);
    let specs = skill_specs_from_value(&v);
    assert_eq!(specs.len(), 1, "duplicate slugs must collapse");
    assert_eq!(specs[0].slug, "review");
    assert!(
        specs[0].label.is_none(),
        "first occurrence (no label) wins; later `label:later one` dropped"
    );
}

#[test]
fn composition_12_unknown_fields_are_ignored() {
    let v = json!({
        "family": "skill.v1",
        "slug": "x",
        "magicField": "ignored",
        "another": [1, 2, 3],
    });
    let specs = skill_specs_from_value(&v);
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].slug, "x");
}

#[test]
fn composition_13_family_wrapped_in_array_filters_out() {
    let v = json!({ "family": ["skill.v1"], "slug": "x" });
    assert!(skill_specs_from_value(&v).is_empty());
}

#[test]
fn composition_14_mixed_array_with_strings_drops_strings() {
    let v = json!([
        "stray-string",
        { "family": "skill.v1", "slug": "ok" },
        42,
        null,
        { "family": "skill.v1", "slug": "also-ok" },
    ]);
    let specs = skill_specs_from_value(&v);
    assert_eq!(specs.len(), 2);
    assert_eq!(specs[0].slug, "ok");
    assert_eq!(specs[1].slug, "also-ok");
}

#[test]
fn composition_15_whitespace_only_slug_skipped() {
    let v = json!([
        { "family": "skill.v1", "slug": "\t\n " },
        { "family": "skill.v1", "slug": "valid" },
    ]);
    let specs = skill_specs_from_value(&v);
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].slug, "valid");
}

#[test]
fn composition_16_nested_skill_in_inner_property_is_not_extracted() {
    // The extractor only inspects the top-level value's family field
    // (single-object) or each top-level array entry. A skill spec nested
    // inside an unrelated key is invisible.
    let v = json!({
        "family": "capture.v1",
        "extra": { "family": "skill.v1", "slug": "hidden" },
    });
    assert!(skill_specs_from_value(&v).is_empty());
}

#[test]
fn composition_17_object_form_with_wrong_family_yields_empty() {
    let v = json!({ "family": "command.v1", "slug": "x" });
    assert!(skill_specs_from_value(&v).is_empty());
}

#[test]
fn composition_18_accepts_capture_target_as_number_is_ignored() {
    let v = json!({
        "family": "skill.v1",
        "slug": "x",
        "acceptsCaptureTarget": 42,
    });
    let specs = skill_specs_from_value(&v);
    assert!(specs[0].accepts_capture_target.is_none());
}

// ============================================================================
// RESURRECTION / IDEMPOTENCE (6 actions)
// ============================================================================

#[test]
fn resurrection_19_same_value_parsed_twice_is_equal() {
    let v = json!([
        { "family": "skill.v1", "slug": "a" },
        { "family": "skill.v1", "slug": "b" },
    ]);
    assert_eq!(skill_specs_from_value(&v), skill_specs_from_value(&v));
}

#[test]
fn resurrection_20_cloned_value_yields_equal_parse() {
    let v = json!({ "family": "skill.v1", "slug": "x", "label": "X" });
    let cloned = v.clone();
    assert_eq!(skill_specs_from_value(&v), skill_specs_from_value(&cloned));
}

#[test]
fn resurrection_21_mutated_value_re_parses_with_change() {
    let mut v = json!([{ "family": "skill.v1", "slug": "old" }]);
    let before = skill_specs_from_value(&v);
    assert_eq!(before[0].slug, "old");
    v[0]["slug"] = json!("new");
    let after = skill_specs_from_value(&v);
    assert_eq!(after[0].slug, "new");
}

#[test]
fn resurrection_22_null_input_does_not_panic() {
    assert!(skill_specs_from_value(&json!(null)).is_empty());
}

#[test]
fn resurrection_23_deeply_nested_input_does_not_stack_overflow() {
    // 50 nested arrays — extractor only looks at top level, so this
    // resolves immediately to "outer is array, inner entry is array
    // (not object), skip" without recursion.
    let mut v = json!({ "family": "skill.v1", "slug": "x" });
    for _ in 0..50 {
        v = json!([v]);
    }
    // Only the OUTERMOST array is iterated; each entry is a nested array,
    // not an object — none extract. No panic.
    assert!(skill_specs_from_value(&v).is_empty());
}

#[test]
fn resurrection_24_increasing_length_arrays_preserve_order() {
    let mut entries: Vec<serde_json::Value> = Vec::new();
    for i in 0..50 {
        entries.push(json!({ "family": "skill.v1", "slug": format!("s{i}") }));
    }
    let v = serde_json::Value::Array(entries);
    let specs: Vec<SkillSpec> = skill_specs_from_value(&v);
    assert_eq!(specs.len(), 50);
    for (i, spec) in specs.iter().enumerate() {
        assert_eq!(spec.slug, format!("s{i}"));
    }
}
